//! The `SAM3 Detect` selection pipeline: detector output → `_refine_mask` → `doc.selection` masks.
//!
//! Mirrors `comfy_extras/nodes_sam3.py`: `_refine_mask` (42-84) and the mask assembly in
//! `SAM3_Detect.execute` (193-255). The detector itself lives in `detector.rs`; this module only
//! covers the refinement loop and the combination rules.

use candle_core::{DType, Result, Tensor};

use crate::nn::interpolate_bilinear;
use crate::tracker::Segmenter;
use crate::vitdet::VisionBackbone;

/// `_refine_mask` padding fraction (`nodes_sam3.py:54`).
const PAD_FRAC: f32 = 0.1;
const SIZE: usize = 1008;

/// `_coarse_fallback`: resize the coarse mask to `(h, w)` and threshold at zero.
fn coarse_fallback(coarse: &Tensor, h: usize, w: usize) -> Result<Tensor> {
    let resized = interpolate_bilinear(&coarse.unsqueeze(0)?.unsqueeze(0)?, h, w)?;
    resized.squeeze(0)?.squeeze(0)?.gt(0.0)?.to_dtype(DType::F32)
}

/// `_refine_mask` (`nodes_sam3.py:42-84`).
///
/// `native` is the original image as `[1, 3, H, W]` in `[0, 1]`, `coarse` is `[H, W]` coarse mask
/// logits at the original resolution (the detector's masks are resized by `orig_size` first), and
/// `box_xyxy` is in original pixel coordinates.
pub fn refine_mask(
    backbone: &VisionBackbone,
    segmenter: &Segmenter,
    native: &Tensor,
    coarse: &Tensor,
    box_xyxy: [f32; 4],
    iterations: usize,
    model_dtype: DType,
) -> Result<Tensor> {
    let (h, w) = (coarse.dim(0)?, coarse.dim(1)?);
    if iterations == 0 {
        return coarse_fallback(coarse, h, w);
    }

    let [x1, y1, x2, y2] = box_xyxy;
    let bw = x2 - x1;
    let bh = y2 - y1;
    // The reference truncates with `int()`, which rounds toward zero.
    let cx1 = ((x1 - bw * PAD_FRAC).max(0.0) as usize).min(w);
    let cy1 = ((y1 - bh * PAD_FRAC).max(0.0) as usize).min(h);
    let cx2 = ((x2 + bw * PAD_FRAC) as usize).min(w);
    let cy2 = ((y2 + bh * PAD_FRAC) as usize).min(h);
    if cx2 <= cx1 || cy2 <= cy1 {
        return coarse_fallback(coarse, h, w);
    }
    let (crop_h, crop_w) = (cy2 - cy1, cx2 - cx1);

    // Crop the image and resize it to 1008x1008 (`common_upscale(..., "bilinear")`), then cast to the
    // model dtype the way `crop_1008.to(device=device, dtype=dtype)` does.
    let crop = native.narrow(2, cy1, crop_h)?.narrow(3, cx1, crop_w)?;
    let crop_frame = interpolate_bilinear(&crop, SIZE, SIZE)?.to_dtype(model_dtype)?;

    // Crop the coarse mask by the proportional indices the reference uses. The coarse mask is at the
    // original resolution here (the detector resizes it to `orig_size`), so these are `cx1..cx2`.
    let mx1 = cx1 * w / w;
    let my1 = cy1 * h / h;
    let mx2 = (cx2 * w / w).min(w);
    let my2 = (cy2 * h / h).min(h);
    if mx2 <= mx1 || my2 <= my1 {
        return coarse_fallback(coarse, h, w);
    }
    let mut mask_logit = coarse.narrow(0, my1, my2 - my1)?.narrow(1, mx1, mx2 - mx1)?.unsqueeze(0)?.unsqueeze(0)?;

    let features = backbone.forward(&crop_frame)?;
    for _ in 0..iterations {
        let coarse_input = interpolate_bilinear(&mask_logit, SIZE, SIZE)?;
        let result = segmenter.segment_mask(&features.interactive_convs, &coarse_input)?;
        mask_logit = result.mask;
    }

    let refined = interpolate_bilinear(&mask_logit, crop_h, crop_w)?.squeeze(0)?.squeeze(0)?;
    let full = place_crop(&refined, h, w, cy1, cx1)?;
    let coarse_full = interpolate_bilinear(&coarse.unsqueeze(0)?.unsqueeze(0)?, h, w)?.squeeze(0)?.squeeze(0)?;
    full.gt(0.0)?.to_dtype(DType::F32)?.maximum(&coarse_full.gt(0.0)?.to_dtype(DType::F32)?)
}

/// Writes `[crop_h, crop_w]` into a zero `[h, w]` at `(top, left)`, the way the reference assigns
/// into a zeroed `full_mask`.
fn place_crop(crop: &Tensor, h: usize, w: usize, top: usize, left: usize) -> Result<Tensor> {
    let (crop_h, crop_w) = (crop.dim(0)?, crop.dim(1)?);
    let dtype = crop.dtype();

    let mut band_parts: Vec<Tensor> = Vec::new();
    if left > 0 {
        band_parts.push(Tensor::zeros((crop_h, left), dtype, crop.device())?);
    }
    band_parts.push(crop.clone());
    if w > left + crop_w {
        band_parts.push(Tensor::zeros((crop_h, w - left - crop_w), dtype, crop.device())?);
    }
    let band = if band_parts.len() == 1 { band_parts.remove(0) } else { Tensor::cat(&band_parts, 1)? };

    let mut parts: Vec<Tensor> = Vec::new();
    if top > 0 {
        parts.push(Tensor::zeros((top, w), dtype, crop.device())?);
    }
    parts.push(band);
    if h > top + crop_h {
        parts.push(Tensor::zeros((h - top - crop_h, w), dtype, crop.device())?);
    }
    if parts.len() == 1 { Ok(parts.remove(0)) } else { Tensor::cat(&parts, 0) }
}

/// `individual_masks = false`: `(combined > 0).any(dim=0)` (`nodes_sam3.py:244`).
pub fn union_masks(masks: &[Tensor]) -> Result<Tensor> {
    if masks.is_empty() {
        return Err(candle_core::Error::Msg("union_masks needs at least one mask".into()));
    }
    let mut acc = masks[0].clone();
    for mask in &masks[1..] {
        acc = acc.maximum(mask)?;
    }
    Ok(acc)
}

#[cfg(test)]
mod tests {
    use super::*;
    use candle_core::Device;

    #[test]
    fn place_crop_writes_only_the_crop_region() {
        let crop = Tensor::ones((2, 3), DType::F32, &Device::Cpu).expect("crop");
        let full = place_crop(&crop, 4, 5, 1, 2).expect("place");
        assert_eq!(full.dims(), &[4, 5]);
        let values = full.flatten_all().expect("flat").to_vec1::<f32>().expect("values");
        for y in 0..4 {
            for x in 0..5 {
                let expected = if (1..3).contains(&y) && (2..5).contains(&x) { 1.0 } else { 0.0 };
                assert_eq!(values[y * 5 + x], expected, "({y}, {x})");
            }
        }
    }

    #[test]
    fn place_crop_handles_the_origin() {
        let crop = Tensor::ones((4, 5), DType::F32, &Device::Cpu).expect("crop");
        let full = place_crop(&crop, 4, 5, 0, 0).expect("place");
        assert_eq!(full.dims(), &[4, 5]);
        for value in full.flatten_all().expect("flat").to_vec1::<f32>().expect("values") {
            assert_eq!(value, 1.0);
        }
    }
}
