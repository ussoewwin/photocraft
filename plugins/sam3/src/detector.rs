//! SAM 3.1 DETR detector: sine positional encoding, encoder/decoder transformers, dot-product
//! scoring and the segmentation head.
//!
//! Mirrors `comfy/ldm/sam3/detector.py`: `SplitMHA` (36-58), `MLPWithNorm` (61-81), `EncoderLayer`
//! (84-104), `DecoderLayer` (122-142), `TransformerDecoder` (145-214), `PixelDecoder` (314-325),
//! `MaskPredictor` (328-335), `SegmentationHead` (338-364), `DotProductScoring` (367-385) and
//! `SAM3Detector._detect` (422-470) for `multiplex=True` (`scalp = 0`, three FPN levels).
//!
//! The MLP activation is ReLU (`comfy/ldm/sam3/sam.py:24`), not GELU: only the ViTDet FFN uses GELU.

use candle_core::{D, DType, Device, Result, Tensor};

use crate::loader::{LoadError, Pack};
use crate::nn::{Conv2d, LayerNorm, Linear, interpolate_nearest, multi_head_attention};

const D_MODEL: usize = 256;
const HEADS: usize = 8;
const DIM_FF: usize = 2048;
const ENC_LAYERS: usize = 6;
const DEC_LAYERS: usize = 6;
const QUERIES: usize = 200;
const EPS: f64 = 1e-5;
const GRID: usize = 72;

fn relu(x: &Tensor) -> Result<Tensor> {
    x.relu()
}

/// `MLP`: `num_layers` linears with ReLU between them (`sam.py:15-25`).
pub struct Mlp {
    layers: Vec<Linear>,
}

impl Mlp {
    pub fn load(pack: &Pack, prefix: &str, in_dim: usize, hidden: usize, out_dim: usize, num_layers: usize) -> std::result::Result<Self, LoadError> {
        let mut dims = vec![in_dim];
        dims.extend(std::iter::repeat_n(hidden, num_layers - 1));
        dims.push(out_dim);
        let mut layers = Vec::with_capacity(num_layers);
        for index in 0..num_layers {
            layers.push(Linear::load(pack, &format!("{prefix}.layers.{index}"))?);
            let _ = (dims[index], dims[index + 1]);
        }
        Ok(Self { layers })
    }

    pub fn forward(&self, x: &Tensor) -> Result<Tensor> {
        let mut x = x.clone();
        for (index, layer) in self.layers.iter().enumerate() {
            let out = layer.forward(&x)?;
            x = if index < self.layers.len() - 1 { relu(&out)? } else { out };
        }
        Ok(x)
    }
}

/// `SplitMHA`: separate q/k/v projections, cross attention when `k` is given.
pub struct SplitMha {
    q: Linear,
    k: Linear,
    v: Linear,
    out: Linear,
}

impl SplitMha {
    fn load(pack: &Pack, prefix: &str) -> std::result::Result<Self, LoadError> {
        Ok(Self {
            q: Linear::load(pack, &format!("{prefix}.q_proj"))?,
            k: Linear::load(pack, &format!("{prefix}.k_proj"))?,
            v: Linear::load(pack, &format!("{prefix}.v_proj"))?,
            out: Linear::load(pack, &format!("{prefix}.out_proj"))?,
        })
    }

    /// `q_input` is `[B, Nq, C]`; `k_input`/`v_input` default to `q_input`.
    fn forward(&self, q_input: &Tensor, k_input: Option<&Tensor>, v_input: Option<&Tensor>, mask: Option<&Tensor>) -> Result<Tensor> {
        let q = self.q.forward(q_input)?;
        let (k, v) = match k_input {
            Some(k_in) => {
                let k = self.k.forward(k_in)?;
                let v = self.v.forward(v_input.unwrap_or(k_in))?;
                (k, v)
            }
            None => (self.k.forward(q_input)?, self.v.forward(q_input)?),
        };
        let out = multi_head_attention(&q, &k, &v, HEADS, mask)?;
        self.out.forward(&out)
    }
}

/// `stack([raw[..., 0::2].sin(), raw[..., 1::2].cos()], -1).flatten(-2)`: the even entries become
/// sines, the odd entries cosines, and the pair axis is folded back into the feature axis.
fn interleave_sin_cos(scaled: &Tensor, half: usize) -> Result<Tensor> {
    let dims = scaled.dims().to_vec();
    let last = dims.len() - 1;
    let mut pair_dims = dims[..last].to_vec();
    pair_dims.push(half / 2);
    pair_dims.push(2);
    let pairs = scaled.reshape(pair_dims)?;
    let even = pairs.narrow(last + 1, 0, 1)?.squeeze(last + 1)?;
    let odd = pairs.narrow(last + 1, 1, 1)?.squeeze(last + 1)?;
    let stacked = Tensor::stack(&[even.sin()?, odd.cos()?], last + 1)?;
    stacked.flatten_from(last)
}

/// `PositionEmbeddingSine(num_pos_feats=256, normalize=True, scale=2π)`.
pub struct PositionEmbeddingSine {
    half_dim: usize,
    temperature: f64,
    scale: f64,
}

impl PositionEmbeddingSine {
    pub fn new(num_pos_feats: usize) -> Self {
        Self { half_dim: num_pos_feats / 2, temperature: 10_000.0, scale: 2.0 * std::f64::consts::PI }
    }

    fn freqs(&self, device: &Device) -> Result<Tensor> {
        // `temperature ** (2 * (arange(half_dim) // 2) / half_dim)`
        let mut values = Vec::with_capacity(self.half_dim);
        for i in 0..self.half_dim {
            let exponent = (2.0 * (i / 2) as f64) / self.half_dim as f64;
            values.push(self.temperature.powf(exponent) as f32);
        }
        Tensor::from_vec(values, self.half_dim, device)
    }

    /// `_sincos(vals)`: interleaved sin/cos features of one axis, `[..., half_dim]`.
    fn sincos(&self, vals: &Tensor) -> Result<Tensor> {
        let freqs = self.freqs(&vals.device())?;
        // `raw = vals * scale / freqs` — the frequencies divide, they are not multiplied through.
        let freqs = freqs.to_dtype(vals.dtype())?;
        let scaled = (vals * self.scale)?.unsqueeze(D::Minus1)?.broadcast_div(&freqs)?;
        interleave_sin_cos(&scaled, self.half_dim)
    }

    /// `encode_boxes(cx, cy, w, h)` → `[N, half_dim*2 + 2]`.
    pub fn encode_boxes(&self, cx: &Tensor, cy: &Tensor, w: &Tensor, h: &Tensor) -> Result<Tensor> {
        let pos_x = self.sincos(cx)?;
        let pos_y = self.sincos(cy)?;
        Tensor::cat(&[&pos_y, &pos_x, &h.unsqueeze(1)?, &w.unsqueeze(1)?], 1)
    }

    /// `forward(f)`: `[1, 256, H, W]` for a feature map of shape `[B, C, H, W]`.
    pub fn forward(&self, x: &Tensor) -> Result<Tensor> {
        let (_, _, h, w) = x.dims4()?;
        let device = x.device();
        let mut gy = Vec::with_capacity(h);
        for i in 0..h {
            gy.push(i as f32 / (h as f32 - 1.0 + 1e-6));
        }
        let mut gx = Vec::with_capacity(w);
        for i in 0..w {
            gx.push(i as f32 / (w as f32 - 1.0 + 1e-6));
        }
        let gy = Tensor::from_vec(gy, (h, 1), &device)?;
        let gx = Tensor::from_vec(gx, (1, w), &device)?;
        let yy = gy.broadcast_as((h, w))?;
        let xx = gx.broadcast_as((h, w))?;
        let sin_y = self.sincos(&yy)?;
        let sin_x = self.sincos(&xx)?;
        // [H, W, half_dim*2] -> [1, half_dim*2, H, W]
        let encoded = Tensor::cat(&[sin_y, sin_x], D::Minus1)?;
        encoded.permute((2, 0, 1))?.unsqueeze(0)?.to_dtype(x.dtype())
    }
}

struct EncoderLayer {
    self_attn: SplitMha,
    cross_attn: SplitMha,
    linear1: Linear,
    linear2: Linear,
    norm1: LayerNorm,
    norm2: LayerNorm,
    norm3: LayerNorm,
}

impl EncoderLayer {
    fn load(pack: &Pack, prefix: &str) -> std::result::Result<Self, LoadError> {
        Ok(Self {
            self_attn: SplitMha::load(pack, &format!("{prefix}.self_attn"))?,
            cross_attn: SplitMha::load(pack, &format!("{prefix}.cross_attn_image"))?,
            linear1: Linear::load(pack, &format!("{prefix}.linear1"))?,
            linear2: Linear::load(pack, &format!("{prefix}.linear2"))?,
            norm1: LayerNorm::load(pack, &format!("{prefix}.norm1"), EPS)?,
            norm2: LayerNorm::load(pack, &format!("{prefix}.norm2"), EPS)?,
            norm3: LayerNorm::load(pack, &format!("{prefix}.norm3"), EPS)?,
        })
    }

    fn forward(&self, x: &Tensor, pos: &Tensor, text: Option<&Tensor>, text_mask: Option<&Tensor>) -> Result<Tensor> {
        let normed = self.norm1.forward(x)?;
        let qk = (&normed + pos)?;
        let mut x = (x + self.self_attn.forward(&qk, Some(&qk), Some(&normed), None)?)?;
        if let Some(text) = text {
            let normed = self.norm2.forward(&x)?;
            x = (&x + self.cross_attn.forward(&normed, Some(text), Some(text), text_mask)?)?;
        }
        let normed = self.norm3.forward(&x)?;
        let ffn = self.linear2.forward(&relu(&self.linear1.forward(&normed)?)?)?;
        x + ffn
    }
}

struct DecoderLayer {
    self_attn: SplitMha,
    cross_attn: SplitMha,
    ca_text: SplitMha,
    norm1: LayerNorm,
    norm2: LayerNorm,
    norm3: LayerNorm,
    catext_norm: LayerNorm,
    linear1: Linear,
    linear2: Linear,
}

impl DecoderLayer {
    fn load(pack: &Pack, prefix: &str) -> std::result::Result<Self, LoadError> {
        Ok(Self {
            self_attn: SplitMha::load(pack, &format!("{prefix}.self_attn"))?,
            cross_attn: SplitMha::load(pack, &format!("{prefix}.cross_attn"))?,
            ca_text: SplitMha::load(pack, &format!("{prefix}.ca_text"))?,
            norm1: LayerNorm::load(pack, &format!("{prefix}.norm1"), EPS)?,
            norm2: LayerNorm::load(pack, &format!("{prefix}.norm2"), EPS)?,
            norm3: LayerNorm::load(pack, &format!("{prefix}.norm3"), EPS)?,
            catext_norm: LayerNorm::load(pack, &format!("{prefix}.catext_norm"), EPS)?,
            linear1: Linear::load(pack, &format!("{prefix}.linear1"))?,
            linear2: Linear::load(pack, &format!("{prefix}.linear2"))?,
        })
    }

    fn forward(
        &self,
        x: &Tensor,
        memory: &Tensor,
        x_pos: &Tensor,
        memory_pos: &Tensor,
        text: Option<&Tensor>,
        text_mask: Option<&Tensor>,
        cross_bias: Option<&Tensor>,
    ) -> Result<Tensor> {
        let qk = (x + x_pos)?;
        let mut x = self.norm2.forward(&(x + self.self_attn.forward(&qk, Some(&qk), Some(x), None)?)?)?;
        if let Some(text) = text {
            let attended = self.ca_text.forward(&(&x + x_pos)?, Some(text), Some(text), text_mask)?;
            x = self.catext_norm.forward(&(x + attended)?)?;
        }
        let cross = self.cross_attn.forward(&(&x + x_pos)?, Some(&(memory + memory_pos)?), Some(memory), cross_bias)?;
        let x = self.norm1.forward(&(x + cross)?)?;
        let ffn = self.linear2.forward(&relu(&self.linear1.forward(&x)?)?)?;
        self.norm3.forward(&(x + ffn)?)
    }
}

/// `gen_sineembed_for_position(pos_tensor, num_feats=256)`: `(..., N) -> (..., N * num_feats)`.
pub fn gen_sineembed_for_position(pos: &Tensor, num_feats: usize) -> Result<Tensor> {
    let half = num_feats / 2;
    let device = pos.device();
    let mut freqs = Vec::with_capacity(half);
    for i in 0..half {
        let exponent = (2.0 * (i / 2) as f64) / half as f64;
        freqs.push((10_000.0f64).powf(exponent) as f32);
    }
    let freqs = Tensor::from_vec(freqs, half, &device)?;
    let dims = pos.dims().to_vec();
    let coords = *dims.last().unwrap_or(&0);
    let mut embeds: Vec<Tensor> = Vec::with_capacity(coords);
    for c in 0..coords {
        let slice = pos.narrow(D::Minus1, c, 1)?;
        // `raw = vals * 2π / freqs`, then `sin(0::2)` and `cos(1::2)` interleaved.
        let freqs = freqs.to_dtype(slice.dtype())?;
        let scaled = (slice * (2.0 * std::f64::consts::PI))?.broadcast_div(&freqs)?;
        embeds.push(interleave_sin_cos(&scaled, half)?);
    }
    Tensor::cat(&embeds, D::Minus1)
}

/// GroupNorm over `[B, C, H, W]` with `num_groups`, as `PixelDecoder` uses.
fn group_norm(x: &Tensor, weight: Option<&Tensor>, bias: Option<&Tensor>, groups: usize, eps: f64) -> Result<Tensor> {
    let (b, c, h, w) = x.dims4()?;
    let xf = x.to_dtype(DType::F32)?;
    let per_group = c / groups;
    let reshaped = xf.reshape((b, groups, per_group * h * w))?;
    let mean = reshaped.mean_keepdim(D::Minus1)?;
    let centred = reshaped.broadcast_sub(&mean)?;
    let var = centred.sqr()?.mean_keepdim(D::Minus1)?;
    let normed = centred.broadcast_div(&(var + eps)?.sqrt()?)?;
    let mut out = normed.reshape((b, c, h, w))?;
    if let Some(weight) = weight {
        out = out.broadcast_mul(&weight.to_dtype(DType::F32)?.reshape((1, c, 1, 1))?)?;
    }
    if let Some(bias) = bias {
        out = out.broadcast_add(&bias.to_dtype(DType::F32)?.reshape((1, c, 1, 1))?)?;
    }
    out.to_dtype(x.dtype())
}

struct PixelDecoderStage {
    conv: Conv2d,
    norm_weight: Option<Tensor>,
    norm_bias: Option<Tensor>,
}

/// The detector front half: encoder/decoder + scoring + segmentation.
pub struct Detector {
    encoder: Vec<EncoderLayer>,
    decoder: Vec<DecoderLayer>,
    decoder_norm: LayerNorm,
    query_embed: Tensor,
    reference_points: Tensor,
    ref_point_head: Mlp,
    bbox_embed: Mlp,
    box_rpb_x: Mlp,
    box_rpb_y: Mlp,
    presence_token: Tensor,
    presence_head: Mlp,
    presence_norm: LayerNorm,
    // Geometry encoder pieces that run even for text-only prompts (`geo_cls`).
    geo_cls_embed: Tensor,
    geo_norm: LayerNorm,
    geo_img_pre_norm: LayerNorm,
    geo_encode: Vec<EncoderLayer>,
    geo_encode_norm: LayerNorm,
    geo_final_proj: Linear,
    // Segmentation head.
    seg_cross_attn: SplitMha,
    seg_cross_attn_norm: LayerNorm,
    pixel_stages: Vec<PixelDecoderStage>,
    instance_head: Conv2d,
    mask_embed: Mlp,
    // Scoring.
    hs_proj: Linear,
    prompt_proj: Linear,
    prompt_mlp_layers: Vec<Linear>,
    prompt_mlp_norm: LayerNorm,
}

/// What the detector produces for one image.
pub struct Detection {
    /// Normalized `cxcywh` boxes `[1, 200, 4]`.
    pub boxes: Tensor,
    /// Raw dot-product scores `[1, 200]` (apply sigmoid and a threshold).
    pub scores: Tensor,
    /// Coarse masks at detector resolution `[1, 200, 288, 288]`.
    pub masks: Tensor,
    /// Decoder query states `[1, 200, 256]`.
    pub queries: Tensor,
    /// Encoder memory `[1, 5184, 256]`, for verification.
    pub memory: Tensor,
    /// Prompt tokens actually fed to the encoder `[1, 33, 256]`.
    pub prompt: Tensor,
}

impl Detector {
    pub fn load(pack: &Pack) -> std::result::Result<Self, LoadError> {
        let base = "detector";
        let mut encoder = Vec::with_capacity(ENC_LAYERS);
        for index in 0..ENC_LAYERS {
            encoder.push(EncoderLayer::load(pack, &format!("{base}.transformer.encoder.layers.{index}"))?);
        }
        let mut decoder = Vec::with_capacity(DEC_LAYERS);
        for index in 0..DEC_LAYERS {
            decoder.push(DecoderLayer::load(pack, &format!("{base}.transformer.decoder.layers.{index}"))?);
        }
        let mut pixel_stages = Vec::with_capacity(3);
        for index in 0..3 {
            pixel_stages.push(PixelDecoderStage {
                conv: Conv2d::load(pack, &format!("{base}.segmentation_head.pixel_decoder.conv_layers.{index}"), 1, 1)?,
                norm_weight: Some(pack.tensor(&format!("{base}.segmentation_head.pixel_decoder.norms.{index}.weight"))?),
                norm_bias: Some(pack.tensor(&format!("{base}.segmentation_head.pixel_decoder.norms.{index}.bias"))?),
            });
        }
        let mut geo_encode = Vec::with_capacity(3);
        for index in 0..3 {
            geo_encode.push(EncoderLayer::load(pack, &format!("{base}.geometry_encoder.encode.{index}"))?);
        }
        let mut prompt_mlp_layers = Vec::with_capacity(2);
        for index in 0..2 {
            prompt_mlp_layers.push(Linear::load(pack, &format!("{base}.dot_prod_scoring.prompt_mlp.layers.{index}"))?);
        }

        Ok(Self {
            encoder,
            decoder,
            decoder_norm: LayerNorm::load(pack, &format!("{base}.transformer.decoder.norm"), EPS)?,
            query_embed: pack.tensor(&format!("{base}.transformer.decoder.query_embed.weight"))?,
            reference_points: pack.tensor(&format!("{base}.transformer.decoder.reference_points.weight"))?,
            ref_point_head: Mlp::load(pack, &format!("{base}.transformer.decoder.ref_point_head"), D_MODEL * 2, D_MODEL, D_MODEL, 2)?,
            bbox_embed: Mlp::load(pack, &format!("{base}.transformer.decoder.bbox_embed"), D_MODEL, D_MODEL, 4, 3)?,
            box_rpb_x: Mlp::load(pack, &format!("{base}.transformer.decoder.boxRPB_embed_x"), 2, D_MODEL, HEADS, 2)?,
            box_rpb_y: Mlp::load(pack, &format!("{base}.transformer.decoder.boxRPB_embed_y"), 2, D_MODEL, HEADS, 2)?,
            presence_token: pack.tensor(&format!("{base}.transformer.decoder.presence_token.weight"))?,
            presence_head: Mlp::load(pack, &format!("{base}.transformer.decoder.presence_token_head"), D_MODEL, D_MODEL, 1, 3)?,
            presence_norm: LayerNorm::load(pack, &format!("{base}.transformer.decoder.presence_token_out_norm"), EPS)?,
            geo_cls_embed: pack.tensor(&format!("{base}.geometry_encoder.cls_embed.weight"))?,
            geo_norm: LayerNorm::load(pack, &format!("{base}.geometry_encoder.norm"), EPS)?,
            geo_img_pre_norm: LayerNorm::load(pack, &format!("{base}.geometry_encoder.img_pre_norm"), EPS)?,
            geo_encode,
            geo_encode_norm: LayerNorm::load(pack, &format!("{base}.geometry_encoder.encode_norm"), EPS)?,
            geo_final_proj: Linear::load(pack, &format!("{base}.geometry_encoder.final_proj"))?,
            seg_cross_attn: SplitMha::load(pack, &format!("{base}.segmentation_head.cross_attend_prompt"))?,
            seg_cross_attn_norm: LayerNorm::load(pack, &format!("{base}.segmentation_head.cross_attn_norm"), EPS)?,
            pixel_stages,
            instance_head: Conv2d::load(pack, &format!("{base}.segmentation_head.instance_seg_head"), 1, 0)?,
            mask_embed: Mlp::load(pack, &format!("{base}.segmentation_head.mask_predictor.mask_embed"), D_MODEL, D_MODEL, D_MODEL, 3)?,
            hs_proj: Linear::load(pack, &format!("{base}.dot_prod_scoring.hs_proj"))?,
            prompt_proj: Linear::load(pack, &format!("{base}.dot_prod_scoring.prompt_proj"))?,
            prompt_mlp_layers,
            prompt_mlp_norm: LayerNorm::load(pack, &format!("{base}.dot_prod_scoring.prompt_mlp.out_norm"), EPS)?,
        })
    }

    /// Runs the detector on the three FPN levels and the projected text embeddings.
    ///
    /// `features`: 288x288, 144x144, 72x72, each `[1, 256, H, W]`.
    /// `text`: `[1, 32, 256]` already projected by `language_backbone.resizer`.
    /// `text_mask`: `[1, 32]` with `1` on padding.
    pub fn detect(&self, features: &[Tensor], text: &Tensor, text_mask: &Tensor) -> Result<Detection> {
        let device = features[0].device();
        let pos_enc = PositionEmbeddingSine::new(D_MODEL);
        let positions: Vec<Tensor> = features.iter().map(|f| pos_enc.forward(f)).collect::<Result<_>>()?;

        let enc_feat = &features[2];
        let enc_pos = &positions[2];
        let (_, _, h, w) = enc_feat.dims4()?;
        let img_flat = enc_feat.flatten_from(2)?.permute((0, 2, 1))?.contiguous()?;
        let pos_flat = enc_pos.flatten_from(2)?.permute((0, 2, 1))?.contiguous()?;

        // Geometry class token, concatenated to the text tokens (no point/box prompts yet).
        let batch = img_flat.dim(0)?;
        let mut geo_cls = self.geo_final_proj.forward(&self.geo_cls_embed.reshape((1, 1, D_MODEL))?)?.broadcast_as((batch, 1, D_MODEL))?;
        geo_cls = self.geo_norm.forward(&geo_cls)?;
        for layer in &self.geo_encode {
            geo_cls = self.run_geo_layer(layer, &geo_cls, &img_flat, &pos_flat)?;
        }
        geo_cls = self.geo_encode_norm.forward(&geo_cls)?;

        let prompt = Tensor::cat(&[text, &geo_cls], 1)?;
        let ones = Tensor::ones((batch, 1), DType::U8, &device)?;
        let mask_u8 = text_mask.to_dtype(DType::U8)?;
        let prompt_mask = Tensor::cat(&[&mask_u8, &ones], 1)?;
        let prompt_mask_f = self.additive_mask(&prompt_mask)?;

        // Encoder.
        let limit: usize = std::env::var("SAM3_ENC_LAYERS").ok().and_then(|v| v.parse().ok()).unwrap_or(self.encoder.len());
        let mut memory = img_flat.clone();
        for layer in self.encoder.iter().take(limit) {
            memory = layer.forward(&memory, &pos_flat, Some(&prompt), Some(&prompt_mask_f))?;
        }
        if limit == 0 {
            // Layer 0 comparison needs the pre-encoder states.
            return Ok(Detection {
                boxes: img_flat.clone(),
                scores: prompt_mask_f.clone(),
                masks: img_flat.clone(),
                queries: img_flat,
                memory: memory.clone(),
                prompt: prompt.clone(),
            });
        }

        // Decoder.
        let (query_out, boxes, presence) = self.decode(&memory, &pos_flat, &prompt, &prompt_mask_f, h, w)?;

        // Scoring.
        let scores = self.score(&query_out, &prompt, &prompt_mask)?;

        // Segmentation.
        let masks = self.segment(&query_out, features, &memory, &prompt, &prompt_mask_f, h, w)?;

        let _ = presence;
        Ok(Detection { boxes, scores, masks, queries: query_out, memory: memory.clone(), prompt: prompt.clone() })
    }

    fn additive_mask(&self, mask: &Tensor) -> Result<Tensor> {
        // `SplitMHA` masks arrive as a boolean `[B, T]`; convert to an additive `[B, 1, 1, T]`.
        let (b, t) = mask.dims2()?;
        let values = mask.flatten_all()?.to_vec1::<u8>()?;
        let mut data = vec![0.0f32; b * t];
        for (index, value) in values.iter().enumerate() {
            data[index] = if *value != 0 { -1.0e9 } else { 0.0 };
        }
        Tensor::from_vec(data, (b, 1, 1, t), &mask.device())
    }

    fn run_geo_layer(&self, layer: &EncoderLayer, x: &Tensor, memory: &Tensor, memory_pos: &Tensor) -> Result<Tensor> {
        let x = (x + layer.self_attn.forward(&layer.norm1.forward(x)?, None, None, None)?)?;
        let cross = layer.cross_attn.forward(&layer.norm2.forward(&x)?, Some(&(memory + memory_pos)?), Some(memory), None)?;
        let x = (&x + cross)?;
        let ffn = layer.linear2.forward(&relu(&layer.linear1.forward(&layer.norm3.forward(&x)?)?)?)?;
        x + ffn
    }

    fn decode(&self, memory: &Tensor, memory_pos: &Tensor, prompt: &Tensor, prompt_mask: &Tensor, h: usize, w: usize) -> Result<(Tensor, Tensor, Tensor)> {
        let device = memory.device();
        let batch = memory.dim(0)?;
        let mut tgt = self.query_embed.unsqueeze(0)?.broadcast_as((batch, QUERIES, D_MODEL))?.contiguous()?;
        let mut presence = self.presence_token.reshape((1, 1, D_MODEL))?.broadcast_as((batch, 1, D_MODEL))?.contiguous()?;
        let mut ref_points = candle_nn::ops::sigmoid(&self.reference_points.unsqueeze(0)?.broadcast_as((batch, QUERIES, 4))?)?;

        for (index, layer) in self.decoder.iter().enumerate() {
            let sine = gen_sineembed_for_position(&ref_points, D_MODEL)?;
            let query_pos = self.ref_point_head.forward(&sine)?;
            let tgt_with_presence = Tensor::cat(&[&presence, &tgt], 1)?;
            let zeros = Tensor::zeros((batch, 1, D_MODEL), tgt_with_presence.dtype(), &device)?;
            let pos_with_presence = Tensor::cat(&[&zeros, &query_pos], 1)?;
            let rpb = self.compute_box_rpb(&ref_points, h, w)?;
            let out = layer.forward(&tgt_with_presence, memory, &pos_with_presence, memory_pos, Some(prompt), Some(prompt_mask), Some(&rpb))?;
            presence = out.narrow(1, 0, 1)?.contiguous()?;
            tgt = out.narrow(1, 1, QUERIES)?.contiguous()?;
            if index < self.decoder.len() - 1 {
                let inv = inverse_sigmoid(&ref_points)?;
                let delta = self.bbox_embed.forward(&self.decoder_norm.forward(&tgt)?)?;
                ref_points = candle_nn::ops::sigmoid(&(&inv + &delta)?)?;
            }
        }

        let query_out = self.decoder_norm.forward(&tgt)?;
        let inv = inverse_sigmoid(&ref_points)?;
        let boxes = candle_nn::ops::sigmoid(&(&inv + self.bbox_embed.forward(&query_out)?)?)?;
        let presence_logits = self.presence_head.forward(&self.presence_norm.forward(&presence)?)?.squeeze(2)?;
        Ok((query_out, boxes, presence_logits))
    }

    /// `_compute_box_rpb`: `[B, heads, 201, H*W]` additive bias for the cross attention.
    fn compute_box_rpb(&self, ref_points: &Tensor, h: usize, w: usize) -> Result<Tensor> {
        let device = ref_points.device();
        // cxcywh -> xyxy
        let cx = ref_points.narrow(2, 0, 1)?;
        let cy = ref_points.narrow(2, 1, 1)?;
        let bw = ref_points.narrow(2, 2, 1)?;
        let bh = ref_points.narrow(2, 3, 1)?;
        let half_w = (&bw * 0.5)?;
        let half_h = (&bh * 0.5)?;
        let x1 = (&cx - &half_w)?;
        let x2 = (&cx + &half_w)?;
        let y1 = (&cy - &half_h)?;
        let y2 = (&cy + &half_h)?;
        // deltas_x uses columns 0 and 2 (x1, x2); deltas_y uses 1 and 3 (y1, y2).
        let xs = Tensor::cat(&[&x1, &x2], 2)?;
        let ys = Tensor::cat(&[&y1, &y2], 2)?;

        let (batch, queries, _) = xs.dims3()?;
        let mut coords_h = Vec::with_capacity(h);
        for i in 0..h {
            coords_h.push(i as f32 / h as f32);
        }
        let mut coords_w = Vec::with_capacity(w);
        for i in 0..w {
            coords_w.push(i as f32 / w as f32);
        }
        let coords_h = Tensor::from_vec(coords_h, (1, 1, h, 1), &device)?.to_dtype(xs.dtype())?;
        let coords_w = Tensor::from_vec(coords_w, (1, 1, w, 1), &device)?.to_dtype(xs.dtype())?;
        // [B, Q, H, 2] and [B, Q, W, 2]
        let deltas_x = coords_w.broadcast_sub(&xs.unsqueeze(2)?.contiguous()?)?;
        let deltas_y = coords_h.broadcast_sub(&ys.unsqueeze(2)?.contiguous()?)?;

        let rpb_x = self.box_rpb_x.forward(&log_scale(&deltas_x)?)?;
        let rpb_y = self.box_rpb_y.forward(&log_scale(&deltas_y)?)?;
        // bias = (rpb_y.unsqueeze(3) + rpb_x.unsqueeze(2)).flatten(2, 3).permute(0, 3, 1, 2)
        // rpb_y: [B, Q, H, heads] -> [B, Q, H, 1, heads]; rpb_x: [B, Q, W, heads] -> [B, Q, 1, W, heads]
        let ry = rpb_y.unsqueeze(3)?;
        let rx = rpb_x.unsqueeze(2)?;
        let summed = ry.broadcast_add(&rx)?;
        // `flatten(2, 3)`: fold the (H, W) axes only, keeping heads last.
        let flat = summed.reshape((batch, queries, h * w, HEADS))?;
        let bias = flat.permute((0, 3, 1, 2))?.contiguous()?;
        let (b2, heads, q2, hw) = bias.dims4()?;
        let _ = (b2, heads, queries);
        let pres = Tensor::zeros((batch, HEADS, 1, hw), bias.dtype(), &device)?;
        let _ = q2;
        Tensor::cat(&[&pres, &bias], 2)?.contiguous()
    }

    fn score(&self, query_out: &Tensor, prompt: &Tensor, prompt_mask: &Tensor) -> Result<Tensor> {
        // `prompt_mlp`: MLPWithNorm(d_model, 2048, d_model, 2) — residual + out_norm.
        let original = prompt.clone();
        let mut hidden = prompt.clone();
        for (index, layer) in self.prompt_mlp_layers.iter().enumerate() {
            let out = layer.forward(&hidden)?;
            hidden = if index < self.prompt_mlp_layers.len() - 1 { relu(&out)? } else { out };
        }
        let prompt_embed = self.prompt_mlp_norm.forward(&(&hidden + &original)?)?;

        // Masked mean over the prompt tokens.
        let mask = prompt_mask.to_dtype(prompt_embed.dtype())?;
        let weight = mask.unsqueeze(2)?;
        let summed = prompt_embed.broadcast_mul(&weight)?.sum(1)?;
        let count = weight.sum(1)?.clamp(1.0, f32::MAX as f64)?;
        let pooled = summed.broadcast_div(&count)?;

        let hs = self.hs_proj.forward(query_out)?;
        let pp = self.prompt_proj.forward(&pooled)?.unsqueeze(2)?;
        let scale = 1.0 / (D_MODEL as f64).sqrt();
        let scores = (hs.matmul(&pp)? * scale)?.clamp(-12.0, 12.0)?;
        scores.squeeze(2)
    }

    fn segment(&self, query_out: &Tensor, features: &[Tensor], memory: &Tensor, prompt: &Tensor, prompt_mask: &Tensor, h: usize, w: usize) -> Result<Tensor> {
        // cross_attend_prompt on the normalized encoder memory, residual over the memory.
        let enc_normed = self.seg_cross_attn_norm.forward(memory)?;
        let enc_cross = self.seg_cross_attn.forward(&enc_normed, Some(prompt), Some(prompt), Some(prompt_mask))?;
        let encoded = (&enc_cross + memory)?;

        let batch = encoded.dim(0)?;
        let visual = encoded.narrow(1, 0, h * w)?.permute((0, 2, 1))?.contiguous()?.reshape((batch, D_MODEL, h, w))?;

        // Replace the last FPN level with the encoder visual features, then run the pixel decoder.
        let mut levels: Vec<Tensor> = features.to_vec();
        levels[2] = visual;
        // `backbone_features[:-1][::-1]` walks 144 -> 72's parent, so only the two upper levels run;
        // the third `conv_layers` entry exists in the checkpoint but the reference never calls it.
        let mut prev = levels[levels.len() - 1].clone();
        for (index, stage) in self.pixel_stages.iter().take(levels.len() - 1).enumerate() {
            let level = &levels[levels.len() - 2 - index];
            let (_, _, lh, lw) = level.dims4()?;
            let upsampled = interpolate_nearest(&prev, lh, lw)?;
            let summed = (level + upsampled)?;
            let convolved = stage.conv.forward(&summed)?;
            let normed = group_norm(&convolved, stage.norm_weight.as_ref(), stage.norm_bias.as_ref(), 8, EPS)?;
            prev = relu(&normed)?;
        }

        let instance = self.instance_head.forward(&prev)?;
        let mask_embed = self.mask_embed.forward(query_out)?;
        // einsum("bqc,bchw->bqhw")
        let (b, q, c) = mask_embed.dims3()?;
        let (_, c2, ph, pw) = instance.dims4()?;
        if c != c2 {
            return Err(candle_core::Error::Msg(format!("mask embed width {c} does not match {c2}")));
        }
        let flat = instance.reshape((b, c, ph * pw))?;
        let out = mask_embed.matmul(&flat)?;
        out.reshape((b, q, ph, pw))
    }
}

fn inverse_sigmoid(x: &Tensor) -> Result<Tensor> {
    // log(x / (1 - x + 1e-6) + 1e-6), computed in f32 because halffloat `log` is too coarse.
    let dtype = x.dtype();
    let xf = x.to_dtype(DType::F32)?;
    let numerator = xf.log()?;
    let denominator = ((xf.neg()? + 1.0)? + 1e-6)?.log()?;
    let ratio = (&numerator - &denominator)?;
    (&ratio + 1e-6)?.to_dtype(dtype)
}

/// `log_scale(d) = sign(8d) * log2(|8d| + 1) / log2(8)`.
fn log_scale(d: &Tensor) -> Result<Tensor> {
    let dtype = d.dtype();
    let scaled = (d.to_dtype(DType::F32)? * 8.0)?;
    let sign = scaled.sign()?;
    // log2(v) = ln(v) / ln(2)
    let magnitude = ((scaled.abs()? + 1.0)?.log()? * (1.0 / std::f64::consts::LN_2))?;
    let out = magnitude.mul(&sign)?;
    // log2(8) = 3
    out.affine(1.0 / 3.0, 0.0)?.to_dtype(dtype)
}

/// `box_cxcywh_to_xyxy`.
pub fn cxcywh_to_xyxy(x: &Tensor) -> Result<Tensor> {
    let cx = x.narrow(D::Minus1, 0, 1)?;
    let cy = x.narrow(D::Minus1, 1, 1)?;
    let w = x.narrow(D::Minus1, 2, 1)?;
    let h = x.narrow(D::Minus1, 3, 1)?;
    let half_w = (&w * 0.5)?;
    let half_h = (&h * 0.5)?;
    Tensor::cat(&[(&cx - &half_w)?, (&cy - &half_h)?, (&cx + &half_w)?, (&cy + &half_h)?], D::Minus1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cxcywh_converts_to_xyxy() {
        let x = Tensor::from_vec(vec![0.5f32, 0.5, 0.2, 0.4], (1, 1, 4), &Device::Cpu).expect("tensor");
        let out = cxcywh_to_xyxy(&x).expect("convert").flatten_all().expect("flat").to_vec1::<f32>().expect("values");
        let expected = [0.4f32, 0.3, 0.6, 0.7];
        for (a, b) in out.iter().zip(expected.iter()) {
            assert!((a - b).abs() < 1e-6, "got {a}, expected {b}");
        }
    }

    #[test]
    fn log_scale_matches_the_formula() {
        let d = Tensor::from_vec(vec![0.0f32, 0.125, -0.125, 1.0], (1, 4), &Device::Cpu).expect("tensor");
        let out = log_scale(&d).expect("log_scale").flatten_all().expect("flat").to_vec1::<f32>().expect("values");
        // 8 * 0.125 = 1 -> log2(2)/log2(8) = 1/3
        assert!((out[1] - 1.0 / 3.0).abs() < 1e-6, "0.125 -> {}", out[1]);
        assert!((out[2] + 1.0 / 3.0).abs() < 1e-6, "-0.125 -> {}", out[2]);
        assert!(out[0].abs() < 1e-6, "0 -> {}", out[0]);
        // 8 * 1 = 8 -> log2(9)/log2(8)
        assert!((out[3] - 9.0f32.log2() / 8.0f32.log2()).abs() < 1e-5, "1.0 -> {}", out[3]);
    }
}
