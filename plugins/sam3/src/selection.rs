//! `--select` driver: the node-level pipeline that turns a prompt into selection masks.
//!
//! Mirrors `SAM3_Detect.execute` (`comfy_extras/nodes_sam3.py:116-255`): the point/box paths
//! (`185-205`) and the text path (`207-236`), then the `individual_masks` assembly (`239-254`).

use std::path::{Path, PathBuf};

use candle_core::{DType, Device, Result, Tensor};

use crate::clip::ClipTextEncoder;
use crate::detector::{Detector, cxcywh_to_xyxy};
use crate::loader::Pack;
use crate::nn::interpolate_bilinear;
use crate::select::{refine_mask, union_masks};
use crate::tokenizer::ClipBpe;
use crate::tracker::Segmenter;
use crate::vitdet::{BackboneOutput, VisionBackbone};

/// The node defaults (`nodes_sam3.py:105-107`).
pub const DEFAULT_THRESHOLD: f32 = 0.5;
pub const DEFAULT_REFINE: usize = 2;

/// Everything `--select` needs from the pack, loaded once.
pub struct SelectionStack {
    pub backbone: VisionBackbone,
    pub segmenter: Segmenter,
    pub detector: Detector,
    pub clip: ClipTextEncoder,
    pub bpe: ClipBpe,
    pub device: Device,
    cached_backbone: Option<(u64, BackboneOutput)>,
}

impl SelectionStack {
    pub fn load(pack: &Pack, data_dir: &Path, device: &Device) -> std::result::Result<Self, crate::loader::LoadError> {
        Ok(Self {
            backbone: VisionBackbone::load(pack, device)?,
            segmenter: Segmenter::load(pack)?,
            detector: Detector::load(pack)?,
            clip: ClipTextEncoder::load(pack)?,
            bpe: ClipBpe::load(data_dir).map_err(|err| crate::loader::LoadError::Missing(format!("vocab/merges: {err:?}")))?,
            device: device.clone(),
            cached_backbone: None,
        })
    }

    /// Returns the full-frame backbone features for `frame`, reusing the cached features when
    /// `cache_key` matches the previous call.
    pub fn backbone_features(&mut self, frame: &Tensor, cache_key: Option<u64>) -> Result<BackboneOutput> {
        if let Some(key) = cache_key {
            if let Some((cached_key, cached)) = &self.cached_backbone {
                if *cached_key == key {
                    return Ok(cached.clone());
                }
            }
        }
        let out = self.backbone.forward(frame)?;
        if let Some(key) = cache_key {
            self.cached_backbone = Some((key, out.clone()));
        }
        Ok(out)
    }
}

/// `SAM3_Detect` output for one image.
pub struct Selection {
    pub masks: Vec<Tensor>,
    pub boxes: Vec<[f32; 4]>,
    pub scores: Vec<f32>,
}

impl Selection {
    /// `individual_masks = false` collapses to a single union mask.
    pub fn finish(&self, individual: bool) -> Result<Vec<Tensor>> {
        if individual || self.masks.is_empty() {
            return Ok(self.masks.clone());
        }
        Ok(vec![union_masks(&self.masks)?])
    }
}

/// The point/box path: one `forward_segment` call plus `refine_iterations - 1` mask re-feeds
/// (`nodes_sam3.py:187-205`), then the mask is brought back to the original resolution.
pub fn segment_prompt(stack: &mut SelectionStack, frame: &Tensor, prompt: Prompt, refine_iterations: usize, image_hw: (usize, usize)) -> Result<Tensor> {
    segment_prompt_cached(stack, frame, None, prompt, refine_iterations, image_hw)
}

/// Same as [`segment_prompt`], with an optional `cache_key` to reuse full-frame ViTDet features
/// across repeated interactive clicks/boxes on the same document image.
pub fn segment_prompt_cached(
    stack: &mut SelectionStack,
    frame: &Tensor,
    cache_key: Option<u64>,
    prompt: Prompt,
    refine_iterations: usize,
    image_hw: (usize, usize),
) -> Result<Tensor> {
    let features = stack.backbone_features(frame, cache_key)?;
    let interactive = &features.interactive_convs;
    let mut mask = match prompt {
        Prompt::Box(corners) => {
            let corners = Tensor::from_vec(corners.to_vec(), (1, 2, 2), frame.device())?.to_dtype(frame.dtype())?;
            stack.segmenter.segment_box(interactive, &corners)?.mask
        }
        Prompt::Point(coords) => {
            let coords = Tensor::from_vec(coords.to_vec(), (1, 1, 2), frame.device())?.to_dtype(frame.dtype())?;
            let labels = Tensor::from_vec(vec![1i64], (1, 1), frame.device())?;
            stack.segmenter.segment_points(interactive, &coords, &labels)?.mask
        }
    };
    // The backbone features depend only on `frame`; each refinement iteration re-feeds the new mask
    // logits through the lightweight mask decoder against the same `interactive_convs`.
    for _ in 1..refine_iterations {
        mask = stack.segmenter.segment_mask(interactive, &mask)?.mask;
    }
    let (h, w) = image_hw;
    let out = interpolate_bilinear(&mask, h, w)?.squeeze(0)?.squeeze(0)?;
    out.gt(0.0)?.to_dtype(DType::F32)
}

/// Which prompt the caller supplied.
pub enum Prompt {
    /// `[x1, y1, x2, y2]` in 1008 space, as `forward_segment(box_inputs=...)` expects.
    Box([f32; 4]),
    /// `[x, y]` in 1008 space.
    Point([f32; 2]),
}

/// The text path: detect per category, keep the queries above `threshold`, order by score, cap at
/// `max_det`, and refine each coarse mask (`nodes_sam3.py:207-236`).
pub fn detect_and_refine(
    stack: &mut SelectionStack,
    frame: &Tensor,
    native: &Tensor,
    prompt: &str,
    threshold: f32,
    refine_iterations: usize,
    image_hw: (usize, usize),
) -> Result<Selection> {
    detect_and_refine_cached(stack, frame, None, native, prompt, threshold, refine_iterations, image_hw)
}

/// Same as [`detect_and_refine`], with an optional `cache_key` to reuse full-frame ViTDet features.
pub fn detect_and_refine_cached(
    stack: &mut SelectionStack,
    frame: &Tensor,
    cache_key: Option<u64>,
    native: &Tensor,
    prompt: &str,
    threshold: f32,
    refine_iterations: usize,
    image_hw: (usize, usize),
) -> Result<Selection> {
    let features = stack.backbone_features(frame, cache_key)?;
    let (h, w) = image_hw;
    let mut out = Selection { masks: Vec::new(), boxes: Vec::new(), scores: Vec::new() };

    for category in crate::tokenizer::parse_prompts(prompt) {
        let encoded = stack.bpe.tokens_for_prompt(&category.text);
        let text = stack.clip.forward(&encoded.ids, &encoded.attention_mask)?;
        let det = stack.detector.detect(&features.convs, &text.projected, &text.padding_mask)?;
        let boxes_norm = cxcywh_to_xyxy(&det.boxes)?;
        let boxes_norm: Vec<f32> = boxes_norm.flatten_all()?.to_dtype(DType::F32)?.to_vec1()?;
        // `orig_size` scaling (`detector.py:483-486`): boxes to pixels, masks to the image size.
        let queries = boxes_norm.len() / 4;
        let mut boxes_px: Vec<[f32; 4]> = Vec::with_capacity(queries);
        for index in 0..queries {
            let b = &boxes_norm[index * 4..index * 4 + 4];
            boxes_px.push([b[0] * w as f32, b[1] * h as f32, b[2] * w as f32, b[3] * h as f32]);
        }

        let probs = candle_nn::ops::sigmoid(&det.scores)?;
        let probs: Vec<f32> = probs.flatten_all()?.to_dtype(DType::F32)?.to_vec1()?;

        let coarse = interpolate_bilinear(&det.masks, h, w)?;

        let mut ranked: Vec<(usize, f32)> = probs.iter().enumerate().filter(|(_, score)| **score > threshold).map(|(index, score)| (index, *score)).collect();
        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        ranked.truncate(category.max_detections.max(1));

        for (index, score) in ranked {
            let coarse_mask = coarse.narrow(1, index, 1)?.squeeze(0)?.squeeze(0)?;
            let refined: Tensor = refine_mask(&stack.backbone, &stack.segmenter, native, &coarse_mask, boxes_px[index], refine_iterations, DType::F16)?;
            out.masks.push(refined.unsqueeze(0)?);
            out.boxes.push(boxes_px[index]);
            out.scores.push(score);
        }
    }
    Ok(out)
}

/// The verification hook: take the detections from a `--detect-out` dump instead of running the
/// detector, so `_refine_mask` can be checked against the oracle on identical coarse input.
pub fn refine_from_dump(
    stack: &SelectionStack,
    dump: &Path,
    native: &Tensor,
    threshold: f32,
    refine_iterations: usize,
    image_hw: (usize, usize),
) -> std::result::Result<Selection, Box<dyn std::error::Error>> {
    let meta: serde_json::Value = serde_json::from_slice(&std::fs::read(dump.with_extension("json"))?)?;
    let entries = meta.as_array().ok_or("detector dump side-car is not an array")?;

    let locate = |name: &str| -> Option<(usize, usize, Vec<usize>)> {
        entries.iter().find(|entry| entry["name"] == name).map(|entry| {
            let shape: Vec<usize> = entry["shape"].as_array().unwrap().iter().map(|v| v.as_u64().unwrap() as usize).collect();
            // The `masks` entry has no offset (it lives in its own file); the others always do.
            let offset = entry["offset"].as_u64().unwrap_or(0) as usize;
            let count = entry["count"].as_u64().unwrap_or(0) as usize;
            (offset, count, shape)
        })
    };

    let bytes = std::fs::read(dump)?;
    let read = |entry: &(usize, usize, Vec<usize>)| -> Vec<f32> {
        let (offset, count, _) = entry;
        bytes[*offset * 4..(*offset + *count) * 4].chunks_exact(4).map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect()
    };

    let boxes_entry = locate("boxes").ok_or("detector dump has no boxes")?;
    let scores_entry = locate("scores_raw").ok_or("detector dump has no scores_raw")?;
    let masks_entry = locate("masks").ok_or("detector dump has no masks")?;

    let boxes_norm = read(&boxes_entry);
    let scores_raw = read(&scores_entry);
    let (h, w) = image_hw;
    let queries = scores_raw.len();
    let mut boxes_px: Vec<[f32; 4]> = Vec::with_capacity(queries);
    for index in 0..queries {
        let b = &boxes_norm[index * 4..index * 4 + 4];
        boxes_px.push([b[0] * w as f32, b[1] * h as f32, b[2] * w as f32, b[3] * h as f32]);
    }

    let mask_file =
        entries.iter().find(|entry| entry["name"] == "masks").and_then(|entry| entry["file"].as_str()).ok_or("detector dump mask entry has no file")?;
    let mask_path: PathBuf = dump.parent().unwrap_or(Path::new(".")).join(mask_file);
    let raw = std::fs::read(&mask_path)?;
    let mask_shape = masks_entry.2.clone();
    let (mh, mw) = (mask_shape[mask_shape.len() - 2], mask_shape[mask_shape.len() - 1]);
    let values: Vec<f32> = raw.chunks_exact(4).map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect();
    let masks = Tensor::from_vec(values, (1, queries, mh, mw), &stack.device)?.to_dtype(DType::F16)?;
    let coarse = interpolate_bilinear(&masks, h, w)?;

    let mut probs: Vec<f32> = Vec::with_capacity(scores_raw.len());
    for value in &scores_raw {
        probs.push(1.0 / (1.0 + (-value).exp()));
    }
    let mut ranked: Vec<(usize, f32)> = probs.iter().enumerate().filter(|(_, score)| **score > threshold).map(|(index, score)| (index, *score)).collect();
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let mut out = Selection { masks: Vec::new(), boxes: Vec::new(), scores: Vec::new() };
    for (index, score) in ranked {
        let coarse_mask = coarse.narrow(1, index, 1)?.squeeze(0)?.squeeze(0)?;
        let refined = refine_mask(&stack.backbone, &stack.segmenter, native, &coarse_mask, boxes_px[index], refine_iterations, DType::F16)?;
        out.masks.push(refined.unsqueeze(0)?);
        out.boxes.push(boxes_px[index]);
        out.scores.push(score);
    }
    Ok(out)
}
