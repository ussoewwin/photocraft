//! SAM 3.1 vision trunk (ViTDet) and the three-scale FPN necks.
//!
//! Mirrors `comfy/ldm/sam3/sam.py` `ViTDet` (223-298), `Block` (185-211), `Attention` (165-183),
//! `PatchEmbed` (214-221), `window_partition` / `window_unpartition` (122-142), `rope_2d` (145-150)
//! and `FPNScaleConv` (301-330) for `multiplex=True`.
//!
//! Measured geometry for a 1008x1008 input: patch 14 → 72x72 = 5,184 tokens; `pos_embed` is the
//! stored 24x24 grid tiled 4x4 and cropped to 72x72, and its class token is dropped (`pos[:, 1:]`);
//! attention is windowed 24x24 in blocks 0-6, 8-14, 16-22, 24-30 and global in blocks 7, 15, 23, 31;
//! the FPN produces 288x288, 144x144 and 72x72 at scales 4.0, 2.0 and 1.0.

use candle_core::{DType, Device, Result, Tensor};

use crate::loader::{LoadError, Pack};
use crate::nn::{Conv2d, LayerNorm, Linear, conv_transpose2x2, gelu};

const EMBED: usize = 1024;
const DEPTH: usize = 32;
const HEADS: usize = 16;
const HEAD_DIM: usize = EMBED / HEADS;
const GRID: usize = 72;
const WINDOW: usize = 24;
const GLOBAL_BLOCKS: [usize; 4] = [7, 15, 23, 31];
/// `pretrain_img_size / patch_size` = 336 / 14.
const PRETRAIN_GRID: usize = 24;

/// Precomputed axial RoPE cosines and sines for one token grid.
pub struct RopeTable {
    cos: Tensor,
    sin: Tensor,
}

impl RopeTable {
    /// `rope_2d(w, w, head_dim, theta, scale_pos)` reduced to its cos/sin factors.
    fn build(w: usize, h: usize, dim: usize, theta: f64, scale_pos: f64, device: &Device, dtype: DType) -> Result<Self> {
        // `rope_2d` calls `EmbedND(dim=head_dim, axes_dim=[head_dim/2, head_dim/2])` on two axes, and
        // each axis runs `rope(pos, axis_dim, theta)` with its own `dim = head_dim/2`. So there are
        // head_dim/4 frequencies per axis, and the axes concatenate along the feature axis.
        let axis_dim = dim / 2;
        let axis_half = axis_dim / 2;
        let mut omega = Vec::with_capacity(axis_half);
        for i in 0..axis_half {
            let scale = (2.0 * i as f64) / axis_dim as f64;
            omega.push(1.0 / theta.powf(scale));
        }
        let half = dim / 2;
        let mut cos = vec![0.0f32; w * h * half];
        let mut sin = vec![0.0f32; w * h * half];
        for y in 0..h {
            for x in 0..w {
                let token = y * w + x;
                let pos_x = x as f64 * scale_pos;
                let pos_y = y as f64 * scale_pos;
                for i in 0..axis_half {
                    let angle_x = pos_x * omega[i];
                    let angle_y = pos_y * omega[i];
                    cos[token * half + i] = angle_x.cos() as f32;
                    sin[token * half + i] = angle_x.sin() as f32;
                    cos[token * half + axis_half + i] = angle_y.cos() as f32;
                    sin[token * half + axis_half + i] = angle_y.sin() as f32;
                }
            }
        }
        Ok(Self {
            cos: Tensor::from_vec(cos, (1, 1, w * h, half), device)?.to_dtype(dtype)?,
            sin: Tensor::from_vec(sin, (1, 1, w * h, half), device)?.to_dtype(dtype)?,
        })
    }

    fn apply(&self, x: &Tensor) -> Result<Tensor> {
        // x: [B, heads, N, head_dim]; the trailing axis is split into (half, 2) pairs.
        let (b, h, n, d) = x.dims4()?;
        let half = d / 2;
        let pairs = x.reshape((b, h, n, half, 2))?;
        let even = pairs.narrow(4, 0, 1)?.squeeze(4)?;
        let odd = pairs.narrow(4, 1, 1)?.squeeze(4)?;
        let cos = self.cos.narrow(2, 0, n.min(self.cos.dim(2)?))?;
        let sin = self.sin.narrow(2, 0, n.min(self.sin.dim(2)?))?;
        // The RoPE factors are one token grid; the leading window/batch axes broadcast.
        let even_cos = even.broadcast_mul(&cos)?;
        let odd_sin = odd.broadcast_mul(&sin)?;
        let even_sin = even.broadcast_mul(&sin)?;
        let odd_cos = odd.broadcast_mul(&cos)?;
        let out_even = (&even_cos - &odd_sin)?;
        let out_odd = (&even_sin + &odd_cos)?;
        Tensor::stack(&[out_even, out_odd], 4)?.reshape((b, h, n, d))
    }
}

struct VitBlock {
    norm1: LayerNorm,
    qkv: Linear,
    proj: Linear,
    norm2: LayerNorm,
    fc1: Linear,
    fc2: Linear,
    global: bool,
}

impl VitBlock {
    fn load(pack: &Pack, index: usize) -> std::result::Result<Self, LoadError> {
        let p = format!("detector.backbone.vision_backbone.trunk.blocks.{index}");
        Ok(Self {
            norm1: LayerNorm::load(pack, &format!("{p}.norm1"), 1e-5)?,
            qkv: Linear::load(pack, &format!("{p}.attn.qkv"))?,
            proj: Linear::load(pack, &format!("{p}.attn.proj"))?,
            norm2: LayerNorm::load(pack, &format!("{p}.norm2"), 1e-5)?,
            fc1: Linear::load(pack, &format!("{p}.mlp.fc1"))?,
            fc2: Linear::load(pack, &format!("{p}.mlp.fc2"))?,
            global: GLOBAL_BLOCKS.contains(&index),
        })
    }

    /// `x` is `[B, H, W, C]`; windowed blocks see 24x24 tokens, global blocks the whole grid.
    fn forward(&self, x: &Tensor, rope: &RopeTable) -> Result<Tensor> {
        let shortcut = x.clone();
        let normed = self.norm1.forward(x)?;
        let (b, h, w, c) = normed.dims4()?;
        let tokens = if self.global {
            // Global blocks flatten the whole grid.
            let flat = normed.reshape((b, h * w, c))?;
            let attended = self.attend(&flat, rope)?;
            attended.reshape((b, h, w, c))?
        } else {
            // 72 = 3 * 24, so the window partition needs no padding for SAM 3.1's geometry.
            if h % WINDOW != 0 || w % WINDOW != 0 {
                return Err(candle_core::Error::Msg(format!("windowed attention expects a multiple of {WINDOW}, got {h}x{w}")));
            }
            let gh = h / WINDOW;
            let gw = w / WINDOW;
            let windows =
                normed.reshape((b, gh, WINDOW, gw, WINDOW, c))?.permute((0, 1, 3, 2, 4, 5))?.contiguous()?.reshape((b * gh * gw, WINDOW * WINDOW, c))?;
            let attended = self.attend(&windows, rope)?;
            let back = attended.reshape((b, gh, gw, WINDOW, WINDOW, c))?.permute((0, 1, 3, 2, 4, 5))?.contiguous()?;
            back.reshape((b, h, w, c))?
        };
        let x = (&shortcut + &tokens)?;
        let mlp = self.fc2.forward(&gelu(&self.fc1.forward(&self.norm2.forward(&x)?)?)?)?;
        x + mlp
    }

    fn attend(&self, x: &Tensor, rope: &RopeTable) -> Result<Tensor> {
        let (b, n, c) = x.dims3()?;
        let qkv = self.qkv.forward(x)?;
        let qkv = qkv.reshape((b, n, 3, HEADS, HEAD_DIM))?;
        let q = qkv.narrow(2, 0, 1)?.squeeze(2)?.permute((0, 2, 1, 3))?.contiguous()?;
        let k = qkv.narrow(2, 1, 1)?.squeeze(2)?.permute((0, 2, 1, 3))?.contiguous()?;
        let v = qkv.narrow(2, 2, 1)?.squeeze(2)?.permute((0, 2, 1, 3))?.contiguous()?;
        let q = rope.apply(&q)?;
        let k = rope.apply(&k)?;
        let out = attention(&q, &k, &v)?;
        let out = out.permute((0, 2, 1, 3))?.contiguous()?.reshape((b, n, c))?;
        self.proj.forward(&out)
    }
}

/// `optimized_attention(q, k, v, heads, skip_reshape=True)`: q/k/v are `[B, heads, N, head_dim]`.
fn attention(q: &Tensor, k: &Tensor, v: &Tensor) -> Result<Tensor> {
    let scale = 1.0 / (HEAD_DIM as f64).sqrt();
    let qf = q.to_dtype(DType::F32)?;
    let kf = k.to_dtype(DType::F32)?.t()?.contiguous()?;
    let scores = (qf.matmul(&kf)? * scale)?;
    let probs = candle_nn::ops::softmax_last_dim(&scores)?.to_dtype(v.dtype())?;
    probs.matmul(v)
}

/// One FPN neck: optional upsampling, then 1x1 and 3x3 convolutions.
struct FpnScale {
    /// `conv_transpose2d` kernels are kept as raw tensors because candle takes them directly.
    up0_w: Option<Tensor>,
    up0_b: Option<Tensor>,
    gelu_between: bool,
    up1_w: Option<Tensor>,
    up1_b: Option<Tensor>,
    conv_1x1: Conv2d,
    conv_3x3: Conv2d,
}

impl FpnScale {
    fn load(pack: &Pack, prefix: &str, scale: f64) -> std::result::Result<Self, LoadError> {
        let mut me = Self {
            up0_w: None,
            up0_b: None,
            gelu_between: false,
            up1_w: None,
            up1_b: None,
            conv_1x1: Conv2d::load(pack, &format!("{prefix}.conv_1x1"), 1, 0)?,
            conv_3x3: Conv2d::load(pack, &format!("{prefix}.conv_3x3"), 1, 1)?,
        };
        if scale == 4.0 {
            me.up0_w = Some(pack.tensor(&format!("{prefix}.dconv_2x2_0.weight"))?);
            me.up0_b = Some(pack.tensor(&format!("{prefix}.dconv_2x2_0.bias"))?);
            me.gelu_between = true;
            me.up1_w = Some(pack.tensor(&format!("{prefix}.dconv_2x2_1.weight"))?);
            me.up1_b = Some(pack.tensor(&format!("{prefix}.dconv_2x2_1.bias"))?);
        } else if scale == 2.0 {
            me.up0_w = Some(pack.tensor(&format!("{prefix}.dconv_2x2.weight"))?);
            me.up0_b = Some(pack.tensor(&format!("{prefix}.dconv_2x2.bias"))?);
        }
        Ok(me)
    }

    fn forward(&self, x: &Tensor) -> Result<Tensor> {
        let mut x = x.clone();
        if let Some(w) = &self.up0_w {
            x = conv_transpose2x2(w, self.up0_b.as_ref(), &x)?;
            if self.gelu_between {
                x = gelu(&x)?;
            }
        }
        if let Some(w) = &self.up1_w {
            x = conv_transpose2x2(w, self.up1_b.as_ref(), &x)?;
        }
        let x = self.conv_1x1.forward(&x)?;
        self.conv_3x3.forward(&x)
    }
}

/// The trunk plus the two FPN groups still-image selection needs.
pub struct VisionBackbone {
    patch_embed: Conv2d,
    pos_embed: Tensor,
    ln_pre: LayerNorm,
    blocks: Vec<VitBlock>,
    rope_global: RopeTable,
    rope_window: RopeTable,
    convs: Vec<FpnScale>,
    interactive_convs: Vec<FpnScale>,
}

/// Trunk features plus the two FPN pyramids.
#[derive(Clone)]
pub struct BackboneOutput {
    /// `[1, 1024, 72, 72]`, the raw trunk output.
    pub trunk: Tensor,
    /// Detector pyramid: 288x288, 144x144, 72x72, each `[1, 256, H, W]`.
    pub convs: Vec<Tensor>,
    /// Interactive tracker pyramid, same shapes.
    pub interactive_convs: Vec<Tensor>,
}

impl VisionBackbone {
    pub fn load(pack: &Pack, device: &Device) -> std::result::Result<Self, LoadError> {
        let base = "detector.backbone.vision_backbone";
        let patch_embed = Conv2d::load(pack, &format!("{base}.trunk.patch_embed.proj"), 14, 0)?;
        let pos_embed = pack.tensor(&format!("{base}.trunk.pos_embed"))?;
        let ln_pre = LayerNorm::load(pack, &format!("{base}.trunk.ln_pre"), 1e-5)?;
        let mut blocks = Vec::with_capacity(DEPTH);
        for index in 0..DEPTH {
            blocks.push(VitBlock::load(pack, index)?);
        }
        // The pack stores everything in f16 and the model runs in f16, as the reference does.
        let dtype = DType::F16;
        let rope_scale = PRETRAIN_GRID as f64 / GRID as f64;
        let rope_global = RopeTable::build(GRID, GRID, HEAD_DIM, 10_000.0, rope_scale, device, dtype).map_err(LoadError::Candle)?;
        let rope_window = RopeTable::build(WINDOW, WINDOW, HEAD_DIM, 10_000.0, 1.0, device, dtype).map_err(LoadError::Candle)?;
        let mut convs = Vec::with_capacity(3);
        let mut interactive_convs = Vec::with_capacity(3);
        for (index, scale) in [4.0f64, 2.0, 1.0].iter().enumerate() {
            convs.push(FpnScale::load(pack, &format!("{base}.convs.{index}"), *scale)?);
            interactive_convs.push(FpnScale::load(pack, &format!("{base}.interactive_convs.{index}"), *scale)?);
        }
        Ok(Self { patch_embed, pos_embed, ln_pre, blocks, rope_global, rope_window, convs, interactive_convs })
    }

    /// `images` must be `[1, 3, 1008, 1008]` in `[0, 1]` with no mean/std normalisation.
    pub fn forward(&self, images: &Tensor) -> Result<BackboneOutput> {
        let trunk = self.trunk(images)?;
        let mut convs = Vec::with_capacity(3);
        let mut interactive = Vec::with_capacity(3);
        for neck in &self.convs {
            convs.push(neck.forward(&trunk)?);
        }
        for neck in &self.interactive_convs {
            interactive.push(neck.forward(&trunk)?);
        }
        Ok(BackboneOutput { trunk, convs, interactive_convs: interactive })
    }

    /// Runs the first `limit` blocks of the trunk, for measuring how the error grows with depth.
    pub fn trunk_limited(&self, images: &Tensor, limit: usize) -> Result<Tensor> {
        let patches = self.patch_embed.forward(images)?;
        let (b, c, hp, wp) = patches.dims4()?;
        let mut x = patches.permute((0, 2, 3, 1))?.contiguous()?.reshape((b, hp * wp, c))?;
        let pos = self.tiled_pos_embed(hp, wp)?;
        x = x.broadcast_add(&pos)?;
        let mut x = x.reshape((b, hp, wp, c))?;
        x = self.ln_pre.forward(&x)?;
        for block in self.blocks.iter().take(limit) {
            let rope = if block.global { &self.rope_global } else { &self.rope_window };
            x = block.forward(&x, rope)?;
        }
        x.permute((0, 3, 1, 2))?.contiguous()
    }

    /// Runs the trunk only, which is what `_refine_mask` needs for a cropped region.
    pub fn trunk(&self, images: &Tensor) -> Result<Tensor> {
        let patches = self.patch_embed.forward(images)?;
        let (b, c, hp, wp) = patches.dims4()?;
        let mut x = patches.permute((0, 2, 3, 1))?.contiguous()?.reshape((b, hp * wp, c))?;

        // `pos[:, 1:]` tiled by PRETRAIN_GRID and cropped to the run grid; the class token is dropped.
        let pos = self.tiled_pos_embed(hp, wp)?;
        x = x.broadcast_add(&pos)?;
        let mut x = x.reshape((b, hp, wp, c))?;
        x = self.ln_pre.forward(&x)?;

        for block in &self.blocks {
            let rope = if block.global { &self.rope_global } else { &self.rope_window };
            x = block.forward(&x, rope)?;
        }
        x.permute((0, 3, 1, 2))?.contiguous()
    }

    fn tiled_pos_embed(&self, hp: usize, wp: usize) -> Result<Tensor> {
        let (_, tokens, dim) = self.pos_embed.dims3()?;
        if tokens == hp * wp + 1 {
            return self.pos_embed.narrow(1, 1, hp * wp);
        }
        let spatial = self.pos_embed.narrow(1, 1, tokens - 1)?;
        let old = ((tokens - 1) as f64).sqrt() as usize;
        // [1, old*old, C] -> [1, C, old, old]
        let grid = spatial.reshape((1, old, old, dim))?.permute((0, 3, 1, 2))?.contiguous()?;
        let tiles_h = hp / old + 1;
        let tiles_w = wp / old + 1;
        let tiled = grid.repeat((1, 1, tiles_h, tiles_w))?.narrow(2, 0, hp)?.narrow(3, 0, wp)?.contiguous()?;
        tiled.permute((0, 2, 3, 1))?.contiguous()?.reshape((1, hp * wp, dim))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `rope_2d` keeps the first token at angle zero, so its factors are exactly (1, 0).
    #[test]
    fn rope_first_token_is_identity() {
        let table = RopeTable::build(4, 4, 64, 10_000.0, 1.0, &Device::Cpu, DType::F32).expect("rope");
        let cos = table.cos.flatten_all().expect("flat").to_vec1::<f32>().expect("cos");
        let sin = table.sin.flatten_all().expect("flat").to_vec1::<f32>().expect("sin");
        assert!((cos[0] - 1.0).abs() < 1e-6, "cos[0] = {}", cos[0]);
        assert!(sin[0].abs() < 1e-6, "sin[0] = {}", sin[0]);
    }

    /// Applying the rotation twice by 90 degrees of the frequency grid returns the original signal
    /// only for the identity pair, so instead check the rotation preserves the pair's norm.
    #[test]
    fn rope_preserves_the_pair_norm() {
        let table = RopeTable::build(2, 2, 8, 10_000.0, 1.0, &Device::Cpu, DType::F32).expect("rope");
        let data: Vec<f32> = (0..1 * 2 * 4 * 8).map(|i| (i as f32 * 0.1) - 1.0).collect();
        let x = Tensor::from_vec(data, (1, 2, 4, 8), &Device::Cpu).expect("tensor");
        let rotated = table.apply(&x).expect("apply");
        let before = x.sqr().expect("sq").sum_all().expect("sum").to_scalar::<f32>().expect("scalar");
        let after = rotated.sqr().expect("sq").sum_all().expect("sum").to_scalar::<f32>().expect("scalar");
        assert!((before - after).abs() < 1e-4, "rotation changed the norm: {before} vs {after}");
    }
}
