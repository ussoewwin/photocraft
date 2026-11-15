//! Shared neural-network pieces used by the SAM 3.1 modules.
//!
//! These are deliberately plain: a linear layer, a layer norm that accumulates in f32, and the
//! small helpers the reference implementation composes. The weights come from [`crate::loader::Pack`]
//! by logical name, so the ConvRot dequantisation and the key remaps stay in one place.

use candle_core::{DType, Device, Result, Tensor, D};
use candle_nn::ops::softmax_last_dim;

use crate::loader::{LoadError, Pack};

/// A `nn.Linear` in PyTorch layout: weight is `[out, in]`.
pub struct Linear {
    weight: Tensor,
    bias: Option<Tensor>,
}

impl Linear {
    /// Loads `<prefix>.weight` (and `<prefix>.bias` when present).
    pub fn load(pack: &Pack, prefix: &str) -> std::result::Result<Self, LoadError> {
        let weight = pack.tensor(&format!("{prefix}.weight"))?;
        let bias = match pack.tensor(&format!("{prefix}.bias")) {
            Ok(bias) => Some(bias),
            Err(LoadError::Missing(_)) => None,
            Err(err) => return Err(err),
        };
        Ok(Self { weight, bias })
    }

    pub fn from_parts(weight: Tensor, bias: Option<Tensor>) -> Self {
        Self { weight, bias }
    }

    pub fn weight(&self) -> &Tensor {
        &self.weight
    }

    /// `x @ weight^T + bias` for any leading shape; candle's matmul wants matching batch axes, so
    /// the leading dimensions are folded into rows.
    pub fn forward(&self, x: &Tensor) -> Result<Tensor> {
        let dims = x.dims().to_vec();
        let in_dim = *dims.last().unwrap_or(&0);
        let rows: usize = dims[..dims.len().saturating_sub(1)].iter().product();
        let out_dim = self.weight.dim(0)?;
        let flat = x.reshape((rows, in_dim))?;
        let out = flat.matmul(&self.weight.t()?.contiguous()?)?;
        let mut out_dims = dims[..dims.len().saturating_sub(1)].to_vec();
        out_dims.push(out_dim);
        let out = out.reshape(out_dims)?;
        match &self.bias {
            Some(bias) => out.broadcast_add(bias),
            None => Ok(out),
        }
    }
}

/// `nn.LayerNorm` with f32 statistics so a half-precision activations do not lose the mean.
pub struct LayerNorm {
    weight: Option<Tensor>,
    bias: Option<Tensor>,
    eps: f64,
}

impl LayerNorm {
    pub fn load(pack: &Pack, prefix: &str, eps: f64) -> std::result::Result<Self, LoadError> {
        let weight = match pack.tensor(&format!("{prefix}.weight")) {
            Ok(t) => Some(t),
            Err(LoadError::Missing(_)) => None,
            Err(err) => return Err(err),
        };
        let bias = match pack.tensor(&format!("{prefix}.bias")) {
            Ok(t) => Some(t),
            Err(LoadError::Missing(_)) => None,
            Err(err) => return Err(err),
        };
        Ok(Self { weight, bias, eps })
    }

    pub fn forward(&self, x: &Tensor) -> Result<Tensor> {
        let dtype = x.dtype();
        let xf = x.to_dtype(DType::F32)?;
        let mean = xf.mean_keepdim(D::Minus1)?;
        let centred = xf.broadcast_sub(&mean)?;
        let var = centred.sqr()?.mean_keepdim(D::Minus1)?;
        let denom = (var + self.eps)?.sqrt()?;
        let mut out = centred.broadcast_div(&denom)?;
        if let Some(weight) = &self.weight {
            out = out.broadcast_mul(&weight.to_dtype(DType::F32)?)?;
        }
        if let Some(bias) = &self.bias {
            out = out.broadcast_add(&bias.to_dtype(DType::F32)?)?;
        }
        out.to_dtype(dtype)
    }
}

/// `quick_gelu`: `x * sigmoid(1.702 * x)`, the CLIP text activation.
pub fn quick_gelu(x: &Tensor) -> Result<Tensor> {
    let scaled = (x * 1.702)?;
    x.mul(&candle_nn::ops::sigmoid(&scaled)?)
}

/// `gelu` with the exact (erf) definition, as used by the ViTDet and the detector FFNs.
///
/// candle ships `GeluErf` as a unary kernel (CUDA included), so this is the same `erf` the
/// reference uses rather than a polynomial approximation.
pub fn gelu(x: &Tensor) -> Result<Tensor> {
    x.gelu_erf()
}
/// A `nn.Conv2d` restricted to what SAM 3.1 uses (square kernels, same-padded 3x3/1x1 or stride-2).
pub struct Conv2d {
    weight: Tensor,
    bias: Option<Tensor>,
    stride: usize,
    padding: usize,
}

impl Conv2d {
    pub fn load(pack: &Pack, prefix: &str, stride: usize, padding: usize) -> std::result::Result<Self, LoadError> {
        let weight = pack.tensor(&format!("{prefix}.weight"))?;
        let bias = match pack.tensor(&format!("{prefix}.bias")) {
            Ok(b) => Some(b),
            Err(LoadError::Missing(_)) => None,
            Err(err) => return Err(err),
        };
        Ok(Self { weight, bias, stride, padding })
    }

    /// Expected input: `[B, C, H, W]` (channels first, as in the checkpoint).
    pub fn forward(&self, x: &Tensor) -> Result<Tensor> {
        let cout = self.weight.dim(0)?;
        let out = x.conv2d(&self.weight, self.padding, self.stride, 1, 1)?;
        match &self.bias {
            Some(b) => out.broadcast_add(&b.reshape((1, cout, 1, 1))?),
            None => Ok(out),
        }
    }
}

/// `F.conv_transpose2d` with kernel 2, stride 2 and no padding (the FPN upsamplers).
pub fn conv_transpose2x2(weight: &Tensor, bias: Option<&Tensor>, x: &Tensor) -> Result<Tensor> {
    let cout = weight.dim(1)?;
    let out = x.conv_transpose2d(weight, 0, 0, 2, 1)?;
    match bias {
        Some(b) => out.broadcast_add(&b.reshape((1, cout, 1, 1))?),
        None => Ok(out),
    }
}

/// The separable triangle filter PyTorch uses for `antialias = True` downsampling.
///
/// `scale = in/out`, `support = max(scale, 1)`, `center = (o + 0.5) * scale - 0.5` clamped at zero,
/// taps `i` in `[floor(center - support + 0.5), ceil(center + support - 0.5)]` inside the input, weight
/// `1 - |i - center| / support`, normalized. Measured against `F.interpolate(..., antialias=True)`
/// on ten size pairs: max deviation 3.2e-07.
fn antialias_weights(in_size: usize, out_size: usize) -> Vec<f32> {
    let scale = in_size as f64 / out_size as f64;
    let support = scale.max(1.0);
    let mut weights = vec![0.0f32; out_size * in_size];
    for o in 0..out_size {
        let center = ((o as f64 + 0.5) * scale - 0.5).max(0.0);
        let lo = (center - support + 0.5).floor() as i64;
        let hi = (center + support - 0.5).ceil() as i64;
        let mut taps: Vec<(usize, f64)> = Vec::new();
        let mut total = 0.0;
        for i in lo..=hi {
            if i < 0 || i >= in_size as i64 {
                continue;
            }
            let tap = 1.0 - (i as f64 - center).abs() / support;
            if tap > 0.0 {
                taps.push((i as usize, tap));
                total += tap;
            }
        }
        if total > 0.0 {
            for (i, tap) in taps {
                weights[o * in_size + i] = (tap / total) as f32;
            }
        }
    }
    weights
}

/// Bilinear resize with `antialias = True` (PyTorch's triangle filter, applied separably).
pub fn resample_bilinear_antialias(x: &Tensor, out_h: usize, out_w: usize) -> Result<Tensor> {
    let (batch, channels, in_h, in_w) = x.dims4()?;
    if in_h == out_h && in_w == out_w {
        return Ok(x.clone());
    }
    let device = x.device();
    let flat = x.to_dtype(DType::F32)?.reshape((batch * channels, in_h, in_w))?;

    let wh = Tensor::from_vec(antialias_weights(in_h, out_h), (out_h, in_h), device)?;
    let wh = wh.unsqueeze(0)?.broadcast_as((batch * channels, out_h, in_h))?.contiguous()?;
    let rows = wh.matmul(&flat)?;

    let ww = Tensor::from_vec(antialias_weights(in_w, out_w), (out_w, in_w), device)?.t()?.contiguous()?;
    let ww = ww.unsqueeze(0)?.broadcast_as((batch * channels, in_w, out_w))?.contiguous()?;
    let rows = rows.matmul(&ww)?;
    rows.reshape((batch, channels, out_h, out_w))?.to_dtype(x.dtype())
}

/// Bilinear resize with `align_corners = False`, matching `F.interpolate`.
pub fn interpolate_bilinear(x: &Tensor, out_h: usize, out_w: usize) -> Result<Tensor> {
    let (batch, channels, in_h, in_w) = x.dims4()?;
    if in_h == out_h && in_w == out_w {
        return Ok(x.clone());
    }
    let xf = x.to_dtype(DType::F32)?;
    let scale_h = in_h as f64 / out_h as f64;
    let scale_w = in_w as f64 / out_w as f64;

    let mut rows: Vec<Tensor> = Vec::with_capacity(out_h);
    for oy in 0..out_h {
        let src = (oy as f64 + 0.5) * scale_h - 0.5;
        let src = src.max(0.0);
        let y0 = (src.floor() as usize).min(in_h.saturating_sub(1));
        let y1 = (y0 + 1).min(in_h.saturating_sub(1));
        let wy = src - y0 as f64;
        let top = xf.narrow(2, y0, 1)?;
        let bottom = xf.narrow(2, y1, 1)?;
        let blended = ((top * (1.0 - wy))? + (bottom * wy)?)?;
        rows.push(blended);
    }
    let stacked = Tensor::cat(&rows, 2)?;

    let mut cols: Vec<Tensor> = Vec::with_capacity(out_w);
    for ox in 0..out_w {
        let src = (ox as f64 + 0.5) * scale_w - 0.5;
        let src = src.max(0.0);
        let x0 = (src.floor() as usize).min(in_w.saturating_sub(1));
        let x1 = (x0 + 1).min(in_w.saturating_sub(1));
        let wx = src - x0 as f64;
        let left = stacked.narrow(3, x0, 1)?;
        let right = stacked.narrow(3, x1, 1)?;
        cols.push(((left * (1.0 - wx))? + (right * wx)?)?);
    }
    let out = Tensor::cat(&cols, 3)?;
    let _ = (batch, channels);
    out.to_dtype(x.dtype())
}

/// Multi-head attention over `[B, N, C]` with an optional additive mask.
pub fn multi_head_attention(
    q: &Tensor,
    k: &Tensor,
    v: &Tensor,
    heads: usize,
    mask: Option<&Tensor>,
) -> Result<Tensor> {
    let (batch, n_q, dim) = q.dims3()?;
    let n_k = k.dim(1)?;
    let head_dim = dim / heads;
    let scale = 1.0 / (head_dim as f64).sqrt();

    let q = q.reshape((batch, n_q, heads, head_dim))?.permute((0, 2, 1, 3))?.contiguous()?;
    let k = k.reshape((batch, n_k, heads, head_dim))?.permute((0, 2, 1, 3))?.contiguous()?;
    let v = v.reshape((batch, n_k, heads, head_dim))?.permute((0, 2, 1, 3))?.contiguous()?;

    let qf = q.to_dtype(DType::F32)?;
    let kf = k.to_dtype(DType::F32)?.t()?.contiguous()?;
    let scores = (qf.matmul(&kf)? * scale)?;
    let scores = match mask {
        Some(mask) => scores.broadcast_add(&mask.to_dtype(DType::F32)?)?,
        None => scores,
    };
    let probs = softmax_last_dim(&scores)?.to_dtype(v.dtype())?;
    let out = probs.matmul(&v)?;
    out.permute((0, 2, 1, 3))?.contiguous()?.reshape((batch, n_q, dim))
}

/// `nn.Embedding` lookup: `[N] -> [1, N, C]`.
pub fn embedding_lookup(table: &Tensor, ids: &[u32], device: &Device) -> Result<Tensor> {
    let ids = Tensor::from_vec(ids.iter().map(|i| *i as u32).collect::<Vec<u32>>(), ids.len(), device)?;
    table.index_select(&ids, 0)?.unsqueeze(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quick_gelu_matches_the_formula() {
        let x = Tensor::from_vec(vec![-2.0f32, -1.0, 0.0, 1.0, 2.0], 5, &Device::Cpu).expect("tensor");
        let got = quick_gelu(&x).expect("quick_gelu").to_vec1::<f32>().expect("values");
        for (input, value) in [-2.0f32, -1.0, 0.0, 1.0, 2.0].iter().zip(got.iter()) {
            let expected = input * (1.0 / (1.0 + (-1.702 * input).exp()));
            assert!((value - expected).abs() < 1e-6, "quick_gelu({input}) = {value}, expected {expected}");
        }
    }

    #[test]
    fn gelu_matches_the_exact_definition() {
        let x = Tensor::from_vec(vec![-1.5f32, -0.5, 0.0, 0.5, 1.5], 5, &Device::Cpu).expect("tensor");
        let got = gelu(&x).expect("gelu").to_vec1::<f32>().expect("values");
        // Reference values measured from torch.nn.functional.gelu (exact erf form).
        let expected = [-0.10021086f32, -0.15426877, 0.0, 0.34573123, 1.39978909];
        for (a, b) in got.iter().zip(expected.iter()) {
            assert!((a - b).abs() < 2e-6, "gelu gave {a}, expected {b}");
        }
    }

    #[test]
    fn layer_norm_matches_the_reference() {
        let x = Tensor::from_vec(vec![1.0f32, 2.0, 3.0, 4.0], (1, 4), &Device::Cpu).expect("tensor");
        let ln = LayerNorm { weight: None, bias: None, eps: 1e-5 };
        let out = ln.forward(&x).expect("layer_norm").to_vec2::<f32>().expect("values");
        let mean = 2.5f32;
        let var = ((1.0 - mean).powi(2) + (2.0 - mean).powi(2) + (3.0 - mean).powi(2) + (4.0 - mean).powi(2)) / 4.0;
        for (index, expected_input) in [1.0f32, 2.0, 3.0, 4.0].iter().enumerate() {
            let expected = (expected_input - mean) / (var + 1e-5).sqrt();
            assert!((out[0][index] - expected).abs() < 1e-6, "got {}, expected {expected}", out[0][index]);
        }
    }

    #[test]
    fn interpolate_upsamples_by_two() {
        let x = Tensor::from_vec(vec![1.0f32, 2.0, 3.0, 4.0], (1, 1, 2, 2), &Device::Cpu).expect("tensor");
        let out = interpolate_bilinear(&x, 4, 4).expect("interpolate");
        assert_eq!(out.dims(), &[1, 1, 4, 4]);
        let values = out.flatten_all().expect("flat").to_vec1::<f32>().expect("values");
        // Corner alignment with align_corners=False keeps the extremes at the corners.
        assert!((values[0] - 1.0).abs() < 1e-6, "top-left {}", values[0]);
        assert!((values[15] - 4.0).abs() < 1e-6, "bottom-right {}", values[15]);
    }

    #[test]
    fn antialias_weights_match_pytorch() {
        // Measured with `F.interpolate(torch.eye(8)[p].view(1,1,1,8), size=(1,3),
        // mode="bilinear", align_corners=False, antialias=True)` for p in 0..8.
        let w = antialias_weights(8, 3);
        let expected = [
            0.289474, 0.394737, 0.236842, 0.078947, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.023810, 0.166667, 0.309524, 0.309524, 0.166667, 0.023810, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.078947, 0.236842, 0.394737, 0.289474,
        ];
        for (index, want) in expected.iter().enumerate() {
            assert!((w[index] - want).abs() < 1e-5, "w[{index}] = {}, expected {want}", w[index]);
        }
        // The filter is normalized, so a constant field is preserved.
        for row in 0..3 {
            let sum: f32 = w[row * 8..(row + 1) * 8].iter().sum();
            assert!((sum - 1.0).abs() < 1e-6, "row {row} sums to {sum}");
        }
    }

    #[test]
    fn antialias_keeps_a_constant_field() {
        let x = Tensor::from_vec(vec![0.5f32; 64 * 64], (1, 1, 64, 64), &Device::Cpu).expect("tensor");
        let out = resample_bilinear_antialias(&x, 16, 16).expect("resample");
        assert_eq!(out.dims(), &[1, 1, 16, 16]);
        for value in out.flatten_all().expect("flat").to_vec1::<f32>().expect("values") {
            assert!((value - 0.5).abs() < 1e-6, "constant field drifted to {value}");
        }
    }
}
