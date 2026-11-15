//! CLIP BPE tokenizer and SAM 3.1 prompt parsing.
//!
//! The rules reproduced here are the ones the reference pipeline applies to a SAM 3.1 prompt
//! (`comfy/sd1_clip.py` `SDTokenizer`, `comfy/text_encoders/sam3_clip.py` `SAM3Tokenizer` /
//! `_parse_prompts`):
//!
//! * text is stripped and lowercased, then split with the CLIP pre-tokenize pattern;
//! * every word is UTF-8 byte-encoded into the CLIP alphabet and merged with the BPE ranks;
//! * the sequence is `[start] + words + [end]`, truncated to `max_length - 1` words plus the end
//!   token, then padded with `0` (SAM 3.1 pads with `0`, not with the end token);
//! * the attention mask is `1` from the start token through the end token and `0` on the padding.
//!
//! The reference ids this module is tested against were produced by running the real
//! `SAM3TokenizerWrapper` from the ComfyUI checkout once and storing the result in
//! `data/tokenizer_reference.json`; the tokenizer itself never touches ComfyUI.

use std::collections::HashMap;
use std::path::Path;

use regex::Regex;
use thiserror::Error;

/// CLIP's beginning-of-text token (`<|startoftext|>`).
pub const START_TOKEN: u32 = 49406;
/// CLIP's end-of-text token (`<|endoftext|>`).
pub const END_TOKEN: u32 = 49407;
/// SAM 3.1 pads with token id `0` (`pad_with_end = False`, `pad_token = 0`).
pub const PAD_TOKEN: u32 = 0;
/// `SAM3Tokenizer(max_length=32)`, and the CLIP text tower's `max_position_embeddings`.
pub const MAX_LENGTH: usize = 32;

#[derive(Debug, Error)]
pub enum TokenizerError {
    #[error("cannot read {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("{file} is not the expected JSON: {source}")]
    Json {
        file: String,
        #[source]
        source: serde_json::Error,
    },
    #[error("the vocabulary lacks <|startoftext|>/<|endoftext|>")]
    MissingSpecialTokens,
}

/// One comma-separated category of a SAM 3.1 prompt (`face:3` → text `face`, three detections).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PromptCategory {
    pub text: String,
    pub max_detections: usize,
}

/// A tokenized prompt: fixed-length ids and the matching attention mask.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EncodedPrompt {
    pub ids: Vec<u32>,
    pub attention_mask: Vec<i64>,
}

/// The CLIP BPE vocabulary with the SAM 3.1 special tokens and length.
pub struct ClipBpe {
    encoder: HashMap<String, u32>,
    bpe_ranks: HashMap<(String, String), u32>,
    byte_encoder: [char; 256],
    pattern: Regex,
    cache: HashMap<String, Vec<u32>>,
}

impl ClipBpe {
    /// Loads `vocab.json` and `merges.txt` from `data_dir` (the CLIP BPE files, 49,408 entries).
    pub fn load(data_dir: &Path) -> Result<Self, TokenizerError> {
        let vocab_path = data_dir.join("vocab.json");
        let merges_path = data_dir.join("merges.txt");

        let vocab_text = std::fs::read_to_string(&vocab_path).map_err(|source| TokenizerError::Io {
            path: vocab_path.display().to_string(),
            source,
        })?;
        let raw: HashMap<String, u32> =
            serde_json::from_str(&vocab_text).map_err(|source| TokenizerError::Json {
                file: vocab_path.display().to_string(),
                source,
            })?;

        if !raw.contains_key("<|startoftext|>") || !raw.contains_key("<|endoftext|>") {
            return Err(TokenizerError::MissingSpecialTokens);
        }

        let merges_text = std::fs::read_to_string(&merges_path).map_err(|source| TokenizerError::Io {
            path: merges_path.display().to_string(),
            source,
        })?;
        let mut bpe_ranks = HashMap::new();
        for line in merges_text.lines().skip(1) {
            let line = line.trim_end();
            if line.is_empty() {
                continue;
            }
            let mut parts = line.split(' ');
            let (Some(first), Some(second)) = (parts.next(), parts.next()) else { continue };
            if first.is_empty() || second.is_empty() {
                continue;
            }
            let rank = bpe_ranks.len() as u32;
            bpe_ranks.entry((first.to_string(), second.to_string())).or_insert(rank);
        }

        // The CLIP pre-tokenize pattern (transformers `CLIPTokenizer`, case-insensitive).
        let pattern = Regex::new(
            r"(?i)<\|startoftext\|>|<\|endoftext\|>|'s|'t|'re|'ve|'m|'ll|'d|[\p{L}]+|[\p{N}]|[^\s\p{L}\p{N}]+",
        )
        .expect("the CLIP pre-tokenize pattern is a constant");

        Ok(Self {
            encoder: raw,
            bpe_ranks,
            byte_encoder: bytes_to_unicode(),
            pattern,
            cache: HashMap::new(),
        })
    }

    /// Tokenizes `text` into `MAX_LENGTH` ids plus the attention mask.
    pub fn encode(&mut self, text: &str) -> EncodedPrompt {
        let lowered = text.trim().to_lowercase();
        // Collect first: the matcher borrows `self.pattern` while `word_ids` needs `&mut self`.
        let words: Vec<String> = self.pattern.find_iter(&lowered).map(|m| m.as_str().to_string()).collect();
        let mut ids: Vec<u32> = Vec::with_capacity(MAX_LENGTH);
        ids.push(START_TOKEN);
        'outer: for token in &words {
            match token.as_str() {
                "<|startoftext|>" => ids.push(START_TOKEN),
                "<|endoftext|>" => ids.push(END_TOKEN),
                _ => {
                    for id in self.word_ids(token) {
                        if ids.len() >= MAX_LENGTH - 1 {
                            break 'outer;
                        }
                        ids.push(id);
                    }
                }
            }
            if ids.len() >= MAX_LENGTH - 1 {
                break;
            }
        }
        ids.truncate(MAX_LENGTH - 1);
        ids.push(END_TOKEN);

        let used = ids.len().min(MAX_LENGTH);
        ids.truncate(used);
        let mut attention_mask = vec![0i64; MAX_LENGTH];
        for slot in attention_mask.iter_mut().take(used) {
            *slot = 1;
        }
        ids.resize(MAX_LENGTH, PAD_TOKEN);
        EncodedPrompt { ids, attention_mask }
    }

    /// Tokenizes the prompt the way the SAM 3.1 wrapper does.
    ///
    /// `SAM3TokenizerWrapper` tokenizes the raw text when the prompt yields at most one category
    /// with the default cap (so a trailing comma stays in the ids, exactly as the reference does),
    /// and tokenizes the first category alone when the prompt names several categories or a cap.
    pub fn tokens_for_prompt(&mut self, text: &str) -> EncodedPrompt {
        let parsed = parse_prompts(text);
        let single_default = parsed.len() <= 1 && parsed.first().is_none_or(|c| c.max_detections == 1);
        if single_default {
            return self.encode(text);
        }
        let first = parsed.into_iter().next().map(|c| c.text).unwrap_or_default();
        self.encode(&first)
    }

    /// The ids of one pre-tokenized word, cached by spelling.
    ///
    /// The local vocabulary is the original CLIP BPE, which marks the last character of a word with
    /// `</w>` instead of using a byte alphabet, so the word is split into characters and the final
    /// character carries the marker.
    fn word_ids(&mut self, token: &str) -> Vec<u32> {
        if let Some(hit) = self.cache.get(token) {
            return hit.clone();
        }
        let ids: Vec<u32> = self
            .bpe(token)
            .iter()
            .filter_map(|piece| self.encoder.get(piece).copied())
            .collect();
        self.cache.insert(token.to_string(), ids.clone());
        ids
    }

    fn bpe(&self, token: &str) -> Vec<String> {
        // The vocabulary is the original CLIP BPE: the word is taken as UTF-8 bytes, each byte is
        // mapped through the CLIP byte alphabet (identity for most of Latin-1), and the last symbol
        // carries the end-of-word marker `</w>`. Splitting by `char` instead of by byte would break
        // every non-ASCII script (a CJK character is three bytes and one character).
        let encoded: Vec<char> = token.as_bytes().iter().map(|b| self.byte_encoder[*b as usize]).collect();
        if encoded.is_empty() {
            return Vec::new();
        }
        let last = *encoded.last().expect("non-empty");
        let mut word: Vec<String> = encoded[..encoded.len() - 1].iter().map(|c| c.to_string()).collect();
        word.push(format!("{last}</w>"));
        if word.len() <= 1 {
            return word;
        }
        loop {
            let pairs = word_pairs(&word);
            if pairs.is_empty() {
                break;
            }
            let mut best: Option<((String, String), u32)> = None;
            for pair in pairs {
                let Some(rank) = self.bpe_ranks.get(&pair).copied() else { continue };
                if best.as_ref().is_none_or(|(_, r)| rank < *r) {
                    best = Some((pair, rank));
                }
            }
            let Some(((first, second), _)) = best else { break };

            let mut merged: Vec<String> = Vec::with_capacity(word.len());
            let mut i = 0;
            while i < word.len() {
                let Some(offset) = word[i..].iter().position(|w| *w == first) else {
                    merged.extend_from_slice(&word[i..]);
                    break;
                };
                merged.extend_from_slice(&word[i..i + offset]);
                i += offset;
                if i + 1 < word.len() && word[i] == first && word[i + 1] == second {
                    merged.push(format!("{first}{second}"));
                    i += 2;
                } else {
                    merged.push(word[i].clone());
                    i += 1;
                }
            }
            word = merged;
            if word.len() == 1 {
                break;
            }
        }
        word
    }
}

fn word_pairs(word: &[String]) -> Vec<(String, String)> {
    if word.len() < 2 {
        return Vec::new();
    }
    let mut seen: Vec<(String, String)> = Vec::with_capacity(word.len() - 1);
    for window in word.windows(2) {
        let pair = (window[0].clone(), window[1].clone());
        if !seen.contains(&pair) {
            seen.push(pair);
        }
    }
    seen
}

fn bytes_to_unicode() -> [char; 256] {
    let mut bytes: Vec<u32> = (b'!' as u32..=b'~' as u32)
        .chain(0xA1..=0xAC)
        .chain(0xAE..=0xFF)
        .collect();
    let mut chars: Vec<u32> = bytes.clone();
    let mut extra = 0u32;
    for byte in 0..256u32 {
        if !bytes.contains(&byte) {
            bytes.push(byte);
            chars.push(256 + extra);
            extra += 1;
        }
    }
    let mut table = ['\0'; 256];
    for (byte, code) in bytes.iter().zip(chars.iter()) {
        table[*byte as usize] = char::from_u32(*code).expect("byte table code points are valid");
    }
    table
}

/// Splits a SAM 3.1 prompt into categories (`face:3, person` → `face`×3, `person`×1).
///
/// Parentheses are stripped and an empty part is dropped, matching the reference parser.
pub fn parse_prompts(text: &str) -> Vec<PromptCategory> {
    let cleaned = text.replace('(', "").replace(')', "");
    let mut out = Vec::new();
    for part in cleaned.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let mut pushed = false;
        for (index, _) in part.match_indices(':').rev() {
            let (head, tail) = (&part[..index], &part[index + 1..]);
            let tail = tail.trim();
            if tail.is_empty() || !tail.chars().all(|c| c.is_ascii_digit() || c == '.') {
                continue;
            }
            let Ok(value) = tail.parse::<f64>() else { continue };
            let head = head.trim();
            if head.is_empty() {
                continue;
            }
            let max_detections = ((value.round() as i64).max(1)) as usize;
            out.push(PromptCategory { text: head.to_string(), max_detections });
            pushed = true;
            break;
        }
        if !pushed {
            out.push(PromptCategory { text: part.to_string(), max_detections: 1 });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn data_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data")
    }

    /// Every case recorded from the reference tokenizer must reproduce id-for-id.
    #[test]
    fn reference_token_ids_match() {
        let path = data_dir().join("tokenizer_reference.json");
        let text = std::fs::read_to_string(&path).expect("tokenizer_reference.json is present");
        let reference: serde_json::Value = serde_json::from_str(&text).expect("reference is JSON");
        let cases = reference["cases"].as_object().expect("cases");
        let mut bpe = ClipBpe::load(&data_dir()).expect("vocabulary loads");

        let mut checked = 0;
        for (prompt, entry) in cases {
            let expected: Vec<u32> = entry["ids"]
                .as_array()
                .expect("ids array")
                .iter()
                .map(|v| v.as_u64().expect("id") as u32)
                .collect();
            // The reference records the wrapper's main output; `tokens_for_prompt` applies the same
            // rule (raw text for a single default category, first category otherwise).
            let encoded = bpe.tokens_for_prompt(prompt);
            assert_eq!(
                encoded.ids, expected,
                "prompt {prompt:?} produced {:?}, reference {:?}",
                encoded.ids, expected
            );
            assert_eq!(encoded.ids.len(), MAX_LENGTH, "prompt {prompt:?} is 32 long");
            checked += 1;
        }
        assert!(checked >= 18, "checked only {checked} reference prompts");
    }

    /// The attention mask is 1 from the start token through the end token, 0 on the padding.
    #[test]
    fn attention_mask_covers_start_through_end() {
        let mut bpe = ClipBpe::load(&data_dir()).expect("vocabulary loads");
        let encoded = bpe.encode("face");
        let end_at = encoded.ids.iter().position(|id| *id == END_TOKEN).expect("end token");
        assert_eq!(encoded.ids[0], START_TOKEN);
        assert_eq!(&encoded.attention_mask[..=end_at], vec![1i64; end_at + 1].as_slice());
        assert!(encoded.attention_mask[end_at + 1..].iter().all(|m| *m == 0));
        assert_eq!(encoded.attention_mask.iter().filter(|m| **m == 1).count(), end_at + 1);
    }

    /// Empty input is start + end + padding, as the reference produces.
    #[test]
    fn empty_prompt_is_start_end_padding() {
        let mut bpe = ClipBpe::load(&data_dir()).expect("vocabulary loads");
        let encoded = bpe.encode("");
        assert_eq!(encoded.ids[0], START_TOKEN);
        assert_eq!(encoded.ids[1], END_TOKEN);
        assert!(encoded.ids[2..].iter().all(|id| *id == PAD_TOKEN));
        assert_eq!(encoded.attention_mask[0], 1);
        assert_eq!(encoded.attention_mask[1], 1);
        assert!(encoded.attention_mask[2..].iter().all(|m| *m == 0));
    }

    /// `category:N` sets the per-category detection cap; parentheses are stripped.
    #[test]
    fn prompts_parse_into_categories() {
        assert_eq!(
            parse_prompts("face:3, person"),
            vec![
                PromptCategory { text: "face".into(), max_detections: 3 },
                PromptCategory { text: "person".into(), max_detections: 1 },
            ]
        );
        assert_eq!(
            parse_prompts("(hand):2 , (foot)"),
            vec![
                PromptCategory { text: "hand".into(), max_detections: 2 },
                PromptCategory { text: "foot".into(), max_detections: 1 },
            ]
        );
        assert!(parse_prompts("   ").is_empty());
        // A colon that is not a numeric cap stays part of the text, as in the reference regex.
        assert_eq!(
            parse_prompts("a:b"),
            vec![PromptCategory { text: "a:b".into(), max_detections: 1 }]
        );
    }

    /// A prompt longer than the 30-token content budget keeps its start and end tokens.
    #[test]
    fn long_prompt_is_truncated_with_end_token() {
        let mut bpe = ClipBpe::load(&data_dir()).expect("vocabulary loads");
        let long = "face person hand foot hair eye mouth ear nose arm leg dress \
                    face person hand foot hair eye mouth ear nose arm leg dress \
                    face person hand foot hair eye mouth ear nose arm leg dress \
                    face person hand foot";
        let encoded = bpe.encode(long);
        assert_eq!(encoded.ids.len(), MAX_LENGTH);
        assert_eq!(encoded.ids[0], START_TOKEN);
        assert_eq!(encoded.ids[MAX_LENGTH - 1], END_TOKEN);
        assert!(encoded.attention_mask.iter().all(|m| *m == 1));
    }
}
