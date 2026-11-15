//! SAM 3.1 CLIP text encoder.
//!
//! Mirrors `comfy/clip_model.py` `CLIPTextModel` with the SAM 3.1 configuration
//! (`comfy/text_encoders/sam3_clip.py`): 24 pre-norm blocks, hidden 1024, 16 heads, intermediate
//! 4096, `quick_gelu`, `layer_norm_eps = 1e-5`, 32 positions, vocabulary 49,408.
//!
//! The attention mask combines the padding mask with the causal mask exactly as the reference does:
//! `mask = (attention_mask == 0 ? -inf : 0) + triu(-inf, 1)`. Only the hidden states matter for the
//! detector; `text_projection` is not applied (`return_projected_pooled=False`), and the pooled
//! output is not consumed by SAM 3.1 detection.

use candle_core::{Result, Tensor, D};

use crate::loader::{LoadError, Pack};
use crate::nn::{Linear, LayerNorm, multi_head_attention, quick_gelu};

const HIDDEN: usize = 1024;
const HEADS: usize = 16;
const LAYERS: usize = 24;
const POSITIONS: usize = 32;
const EPS: f64 = 1e-5;

struct ClipLayer {
    ln1: LayerNorm,
    q: Linear,
    k: Linear,
    v: Linear,
    out: Linear,
    ln2: LayerNorm,
    fc1: Linear,
    fc2: Linear,
}

impl ClipLayer {
    fn load(pack: &Pack, index: usize) -> std::result::Result<Self, LoadError> {
        let prefix = format!("detector.backbone.language_backbone.encoder.transformer.resblocks.{index}");
        Ok(Self {
            ln1: LayerNorm::load(pack, &format!("{prefix}.ln_1"), EPS)?,
            q: Linear::load(pack, &format!("{prefix}.attn.q_proj"))?,
            k: Linear::load(pack, &format!("{prefix}.attn.k_proj"))?,
            v: Linear::load(pack, &format!("{prefix}.attn.v_proj"))?,
            out: Linear::load(pack, &format!("{prefix}.attn.out_proj"))?,
            ln2: LayerNorm::load(pack, &format!("{prefix}.ln_2"), EPS)?,
            fc1: Linear::load(pack, &format!("{prefix}.mlp.c_fc"))?,
            fc2: Linear::load(pack, &format!("{prefix}.mlp.c_proj"))?,
        })
    }

    fn forward(&self, x: &Tensor, mask: &Tensor) -> Result<Tensor> {
        let normed = self.ln1.forward(x)?;
        let q = self.q.forward(&normed)?;
        let k = self.k.forward(&normed)?;
        let v = self.v.forward(&normed)?;
        let attn = multi_head_attention(&q, &k, &v, HEADS, Some(mask))?;
        let x = (x + self.out.forward(&attn)?)?;
        let normed = self.ln2.forward(&x)?;
        let mlp = self.fc2.forward(&quick_gelu(&self.fc1.forward(&normed)?)?)?;
        x + mlp
    }
}

/// The text tower plus the `resizer` that projects its hidden states to the detector width.
pub struct ClipTextEncoder {
    token_embedding: Tensor,
    position_embedding: Tensor,
    layers: Vec<ClipLayer>,
    final_ln: LayerNorm,
    resizer: Linear,
}

/// What the detector consumes: the projected tokens and the padding mask.
pub struct TextFeatures {
    /// `[B, 32, 256]`
    pub projected: Tensor,
    /// `[B, 32]` — `true` where the token is padding.
    pub padding_mask: Tensor,
}

impl ClipTextEncoder {
    pub fn load(pack: &Pack) -> std::result::Result<Self, LoadError> {
        let base = "detector.backbone.language_backbone";
        let token_embedding = pack.tensor(&format!("{base}.encoder.token_embedding.weight"))?;
        let position_embedding = pack.tensor(&format!("{base}.encoder.positional_embedding"))?;
        let (rows, cols) = position_embedding.dims2().map_err(LoadError::Candle)?;
        if rows != POSITIONS || cols != HIDDEN {
            return Err(LoadError::Layout(
                format!("{base}.encoder.positional_embedding"),
                format!("expected [{POSITIONS}, {HIDDEN}], got [{rows}, {cols}]"),
            ));
        }
        let mut layers = Vec::with_capacity(LAYERS);
        for index in 0..LAYERS {
            layers.push(ClipLayer::load(pack, index)?);
        }
        Ok(Self {
            token_embedding,
            position_embedding,
            layers,
            final_ln: LayerNorm::load(pack, &format!("{base}.encoder.ln_final"), EPS)?,
            // The resizer is one of the ConvRot INT8 linears, so the pack dequantises it on load.
            resizer: Linear::load(pack, &format!("{base}.resizer"))?,
        })
    }

    /// `ids` and `attention_mask` are one prompt of length `POSITIONS`.
    pub fn forward(&self, ids: &[u32], attention_mask: &[i64]) -> Result<TextFeatures> {
        let device = &self.token_embedding.device();
        if ids.len() != POSITIONS || attention_mask.len() != POSITIONS {
            return Err(candle_core::Error::Msg(format!(
                "the prompt must be {POSITIONS} long, got {} ids and {} mask entries",
                ids.len(),
                attention_mask.len()
            )));
        }
        let id_tensor = Tensor::from_vec(ids.to_vec(), ids.len(), device)?;
        let mut x = self.token_embedding.index_select(&id_tensor, 0)?;
        x = x.broadcast_add(&self.position_embedding)?;
        x = x.unsqueeze(0)?;

        let mask = build_mask(attention_mask, device, x.dtype())?;
        for layer in &self.layers {
            x = layer.forward(&x, &mask)?;
        }
        let x = self.final_ln.forward(&x)?;
        let projected = self.resizer.forward(&x)?;

        // The detector receives `1 - attention_mask` as its padding mask (true where padding).
        let padding: Vec<u32> = attention_mask.iter().map(|m| if *m == 0 { 1 } else { 0 }).collect();
        let padding_mask = Tensor::from_vec(padding, (1, POSITIONS), device)?;
        Ok(TextFeatures { projected, padding_mask })
    }

    /// The final hidden states without the resizer projection, for verification against the oracle.
    pub fn hidden_states(&self, ids: &[u32], attention_mask: &[i64]) -> Result<Tensor> {
        let device = &self.token_embedding.device();
        let id_tensor = Tensor::from_vec(ids.to_vec(), ids.len(), device)?;
        let mut x = self.token_embedding.index_select(&id_tensor, 0)?;
        x = x.broadcast_add(&self.position_embedding)?.unsqueeze(0)?;
        let mask = build_mask(attention_mask, device, x.dtype())?;
        for layer in &self.layers {
            x = layer.forward(&x, &mask)?;
        }
        self.final_ln.forward(&x)
    }
}

/// `(attention_mask == 0 ? -inf : 0) + triu(-inf, 1)`, the reference's additive mask.
fn build_mask(attention_mask: &[i64], device: &candle_core::Device, dtype: candle_core::DType) -> Result<Tensor> {
    const NEG: f32 = -1.0e9;
    let n = attention_mask.len();
    let mut data = vec![0.0f32; n * n];
    for row in 0..n {
        for col in 0..n {
            let padded = attention_mask[col] == 0;
            let future = col > row;
            data[row * n + col] = if padded || future { NEG } else { 0.0 };
        }
    }
    // Scores are [B, heads, n, n]; the mask broadcasts over the batch and head axes.
    Tensor::from_vec(data, (1, 1, n, n), device)?.to_dtype(dtype)
}

/// Convenience: the final hidden state of the end-of-text position, used by nothing yet but handy
/// when comparing against the oracle.
pub fn pooled_at(hidden: &Tensor, num_tokens: usize) -> Result<Tensor> {
    let index = num_tokens.saturating_sub(1);
    let seq = hidden.dim(1)?;
    let index = index.min(seq.saturating_sub(1));
    hidden.narrow(1, index, 1)?.squeeze(1)
}

/// `D::Minus1` is re-exported for callers that squeeze the last axis.
pub const LAST_DIM: D = D::Minus1;
