//! SAM 3.1 interactive segmenter: prompt encoder and the single-object mask decoder.
//!
//! Mirrors `comfy/ldm/sam3/sam.py` (`SAMAttention` 28-43, `TwoWayAttentionBlock` 46-70,
//! `TwoWayTransformer` 73-90, `PositionEmbeddingRandom` 93-117) and `comfy/ldm/sam3/tracker.py`
//! (`SAMMaskDecoder` 436-515, `SAMPromptEncoder` 518-577, `_upscale_masks` 425-433,
//! `forward_sam_heads` 268-329).
//!
//! The pipeline matches `SAM3Model.forward_segment` (`detector.py:524-565`): take the interactive
//! FPN necks, add `interactivity_no_mem_embed` to the 72x72 level, encode the prompt, decode, gate
//! the masks by the object score, then interpolate to 1008x1008.

use candle_core::{DType, Device, Result, Tensor};

use crate::detector::Mlp;
use crate::loader::{LoadError, Pack};
use crate::nn::{conv_transpose2x2, interpolate_bilinear, resample_bilinear_antialias, Conv2d, LayerNorm, Linear};

const D_MODEL: usize = 256;
const HEADS: usize = 8;
const GRID: usize = 72;
/// The prompt encoder resizes mask prompts to `image_embedding_size * 4`.
const PROMPT_GRID: usize = GRID * 4;
const IMAGE: usize = 1008;
const EPS: f64 = 1e-5;
const NO_OBJ_SCORE: f32 = -1024.0;

/// `LayerNorm2d`: LayerNorm over the channel axis of `[B, C, H, W]` (`cascade/common.py:64-69`).
fn layer_norm_2d(x: &Tensor, weight: Option<&Tensor>, bias: Option<&Tensor>, eps: f64) -> Result<Tensor> {
    let dtype = x.dtype();
    let xf = x.to_dtype(DType::F32)?;
    let mean = xf.mean_keepdim(1)?;
    let centred = xf.broadcast_sub(&mean)?;
    let var = centred.sqr()?.mean_keepdim(1)?;
    let mut out = centred.broadcast_div(&(var + eps)?.sqrt()?)?;
    if let Some(weight) = weight {
        out = out.broadcast_mul(&weight.to_dtype(DType::F32)?.reshape((1, (), 1, 1))?)?;
    }
    if let Some(bias) = bias {
        out = out.broadcast_add(&bias.to_dtype(DType::F32)?.reshape((1, (), 1, 1))?)?;
    }
    out.to_dtype(dtype)
}

/// `PositionEmbeddingRandom`: Fourier features of normalized coordinates via a fixed projection.
pub struct PositionEmbeddingRandom {
    matrix: Tensor,
}

impl PositionEmbeddingRandom {
    fn load(pack: &Pack, name: &str) -> std::result::Result<Self, LoadError> {
        Ok(Self { matrix: pack.tensor(name)? })
    }

    /// `_encode`: `sin/cos(2*pi * (2u - 1) @ M)`, computed in f32 then cast back (`sam.py:99-104`).
    fn encode(&self, normalized: &Tensor) -> Result<Tensor> {
        let dtype = normalized.dtype();
        let dims = normalized.dims().to_vec();
        let feat = *dims.last().unwrap_or(&2);
        let rows: usize = dims[..dims.len().saturating_sub(1)].iter().product();

        let coords = normalized.to_dtype(DType::F32)?.reshape((rows, feat))?;
        // The reference multiplies by the scalar before the matmul, so the rounding matches:
        // `2*pi * (2u - 1)` is `2*pi*2*u - 2*pi`, one fused affine.
        let scale = 2.0 * std::f64::consts::PI;
        let centred = coords.affine(2.0 * scale, -scale)?;
        let matrix = self.matrix.to_dtype(DType::F32)?;
        let projected = centred.matmul(&matrix)?;

        let out_feats = self.matrix.dim(1)?;
        let mut out_dims = dims[..dims.len().saturating_sub(1)].to_vec();
        out_dims.push(out_feats);
        let projected = projected.reshape(out_dims)?;
        Tensor::cat(&[projected.sin()?, projected.cos()?], dims.len() - 1)?.to_dtype(dtype)
    }

    /// `forward((h, w))`: the `[1, 2*feats, h, w]` dense grid encoding (`sam.py:106-111`).
    pub fn forward(&self, h: usize, w: usize, device: &Device) -> Result<Tensor> {
        let mut xs = Vec::with_capacity(h * w * 2);
        for y in 0..h {
            for x in 0..w {
                xs.push((x as f32 + 0.5) / w as f32);
                xs.push((y as f32 + 0.5) / h as f32);
            }
        }
        let coords = Tensor::from_vec(xs, (h, w, 2), device)?;
        self.encode(&coords)?.permute((2, 0, 1))?.unsqueeze(0)
    }

    /// `forward_with_coords`: `coords` are `[B, N, 2]` in pixels of `image_size = (H, W)`.
    pub fn forward_with_coords(&self, coords: &Tensor, image_size: (usize, usize)) -> Result<Tensor> {
        let (height, width) = image_size;
        let x = coords.narrow(2, 0, 1)?.affine(1.0 / width as f64, 0.0)?;
        let y = coords.narrow(2, 1, 1)?.affine(1.0 / height as f64, 0.0)?;
        self.encode(&Tensor::cat(&[&x, &y], 2)?)
    }
}

/// `SAMAttention`: q/k/v projections with an optional downsample rate on the internal width.
struct SamAttention {
    q: Linear,
    k: Linear,
    v: Linear,
    out: Linear,
}

impl SamAttention {
    fn load(pack: &Pack, prefix: &str) -> std::result::Result<Self, LoadError> {
        Ok(Self {
            q: Linear::load(pack, &format!("{prefix}.q_proj"))?,
            k: Linear::load(pack, &format!("{prefix}.k_proj"))?,
            v: Linear::load(pack, &format!("{prefix}.v_proj"))?,
            out: Linear::load(pack, &format!("{prefix}.out_proj"))?,
        })
    }

    fn forward(&self, q_in: &Tensor, k_in: &Tensor, v_in: &Tensor) -> Result<Tensor> {
        let q = self.q.forward(q_in)?;
        let k = self.k.forward(k_in)?;
        let v = self.v.forward(v_in)?;
        let attended = crate::nn::multi_head_attention(&q, &k, &v, HEADS, None)?;
        self.out.forward(&attended)
    }
}

struct TwoWayBlock {
    self_attn: SamAttention,
    cross_token_to_image: SamAttention,
    cross_image_to_token: SamAttention,
    mlp0: Linear,
    mlp2: Linear,
    norm1: LayerNorm,
    norm2: LayerNorm,
    norm3: LayerNorm,
    norm4: LayerNorm,
    skip_first_layer_pe: bool,
}

impl TwoWayBlock {
    fn load(pack: &Pack, prefix: &str, skip_first_layer_pe: bool) -> std::result::Result<Self, LoadError> {
        Ok(Self {
            self_attn: SamAttention::load(pack, &format!("{prefix}.self_attn"))?,
            cross_token_to_image: SamAttention::load(pack, &format!("{prefix}.cross_attn_token_to_image"))?,
            cross_image_to_token: SamAttention::load(pack, &format!("{prefix}.cross_attn_image_to_token"))?,
            mlp0: Linear::load(pack, &format!("{prefix}.mlp.0"))?,
            mlp2: Linear::load(pack, &format!("{prefix}.mlp.2"))?,
            norm1: LayerNorm::load(pack, &format!("{prefix}.norm1"), EPS)?,
            norm2: LayerNorm::load(pack, &format!("{prefix}.norm2"), EPS)?,
            norm3: LayerNorm::load(pack, &format!("{prefix}.norm3"), EPS)?,
            norm4: LayerNorm::load(pack, &format!("{prefix}.norm4"), EPS)?,
            skip_first_layer_pe,
        })
    }

    fn forward(&self, queries: &Tensor, keys: &Tensor, query_pe: &Tensor, key_pe: &Tensor) -> Result<(Tensor, Tensor)> {
        let queries = if self.skip_first_layer_pe {
            self.norm1.forward(&self.self_attn.forward(queries, queries, queries)?)?
        } else {
            let q = (queries + query_pe)?;
            let attended = self.self_attn.forward(&q, &q, queries)?;
            self.norm1.forward(&(queries + attended)?)?
        };
        let q = (&queries + query_pe)?;
        let k = (keys + key_pe)?;
        let attended = self.cross_token_to_image.forward(&q, &k, keys)?;
        let queries = self.norm2.forward(&(&queries + attended)?)?;
        let mlp = self.mlp2.forward(&self.mlp0.forward(&queries)?.relu()?)?;
        let queries = self.norm3.forward(&(&queries + mlp)?)?;
        let q = (&queries + query_pe)?;
        let k = (keys + key_pe)?;
        let attended = self.cross_image_to_token.forward(&k, &q, &queries)?;
        let keys = self.norm4.forward(&(keys + attended)?)?;
        Ok((queries, keys))
    }
}

struct TwoWayTransformer {
    layers: Vec<TwoWayBlock>,
    final_attn: SamAttention,
    norm_final: LayerNorm,
}

impl TwoWayTransformer {
    fn load(pack: &Pack, prefix: &str, depth: usize) -> std::result::Result<Self, LoadError> {
        let mut layers = Vec::with_capacity(depth);
        for index in 0..depth {
            layers.push(TwoWayBlock::load(pack, &format!("{prefix}.layers.{index}"), index == 0)?);
        }
        Ok(Self {
            layers,
            final_attn: SamAttention::load(pack, &format!("{prefix}.final_attn_token_to_image"))?,
            norm_final: LayerNorm::load(pack, &format!("{prefix}.norm_final"), EPS)?,
        })
    }

    fn forward(&self, image_embedding: &Tensor, image_pe: &Tensor, point_embedding: &Tensor) -> Result<(Tensor, Tensor)> {
        let mut queries = point_embedding.clone();
        let mut keys = image_embedding.clone();
        for layer in &self.layers {
            let (next_queries, next_keys) = layer.forward(&queries, &keys, point_embedding, image_pe)?;
            queries = next_queries;
            keys = next_keys;
        }
        let q = (&queries + point_embedding)?;
        let k = (&keys + image_pe)?;
        let attended = self.final_attn.forward(&q, &k, &keys)?;
        let queries = self.norm_final.forward(&(&queries + attended)?)?;
        Ok((queries, keys))
    }
}

/// The sparse and dense prompt tensors the decoder consumes.
pub struct PromptEmbeddings {
    pub sparse: Tensor,
    pub dense: Tensor,
}

/// `SAMPromptEncoder` (`tracker.py:518-577`).
pub struct SamPromptEncoder {
    pe: PositionEmbeddingRandom,
    point_embeddings: Vec<Tensor>,
    not_a_point: Tensor,
    mask_down_conv0: Conv2d,
    mask_down_norm0: (Tensor, Tensor),
    mask_down_conv3: Conv2d,
    mask_down_norm4: (Tensor, Tensor),
    mask_down_conv6: Conv2d,
    no_mask: Tensor,
}

impl SamPromptEncoder {
    pub fn load(pack: &Pack) -> std::result::Result<Self, LoadError> {
        let base = "tracker.interactive_sam_prompt_encoder";
        let mut point_embeddings = Vec::with_capacity(4);
        for index in 0..4 {
            point_embeddings.push(pack.tensor(&format!("{base}.point_embeddings.{index}.weight"))?);
        }
        Ok(Self {
            pe: PositionEmbeddingRandom::load(pack, &format!("{base}.pe_layer.positional_encoding_gaussian_matrix"))?,
            point_embeddings,
            not_a_point: pack.tensor(&format!("{base}.not_a_point_embed.weight"))?,
            mask_down_conv0: Conv2d::load(pack, &format!("{base}.mask_downscaling.0"), 2, 0)?,
            mask_down_norm0: (
                pack.tensor(&format!("{base}.mask_downscaling.1.weight"))?,
                pack.tensor(&format!("{base}.mask_downscaling.1.bias"))?,
            ),
            mask_down_conv3: Conv2d::load(pack, &format!("{base}.mask_downscaling.3"), 2, 0)?,
            mask_down_norm4: (
                pack.tensor(&format!("{base}.mask_downscaling.4.weight"))?,
                pack.tensor(&format!("{base}.mask_downscaling.4.bias"))?,
            ),
            mask_down_conv6: Conv2d::load(pack, &format!("{base}.mask_downscaling.6"), 1, 0)?,
            no_mask: pack.tensor(&format!("{base}.no_mask_embed.weight"))?,
        })
    }

    /// `get_dense_pe()` over the 72x72 embedding grid.
    pub fn dense_pe(&self, device: &Device) -> Result<Tensor> {
        self.pe.forward(GRID, GRID, device)
    }

    /// `forward(points, boxes, masks)`. `points` are `[B, N, 2]` pixels with `[B, N]` labels,
    /// `box_corners` is `[B, 2, 2]` in pixels, `masks` is `[B, 1, 288, 288]` logits.
    pub fn forward(&self, points: Option<(&Tensor, &Tensor)>, box_corners: Option<&Tensor>, masks: Option<&Tensor>) -> Result<PromptEmbeddings> {
        let device = self.no_mask.device();
        let mut sparse_parts: Vec<Tensor> = Vec::new();
        let mut batch = 1usize;

        if let Some((coords, labels)) = points {
            let (b, _, _) = coords.dims3()?;
            batch = b;
            // A `label = -1` padding point is appended when no box is supplied (`tracker.py:553-555`).
            let (coords, labels) = if box_corners.is_none() {
                let pad = Tensor::zeros((b, 1, 2), coords.dtype(), &device)?;
                let pad_label = Tensor::full(-1i64, (b, 1), &device)?;
                (Tensor::cat(&[coords, &pad], 1)?, Tensor::cat(&[labels, &pad_label], 1)?)
            } else {
                (coords.clone(), labels.clone())
            };
            let pe = self.pe.forward_with_coords(&(coords + 0.5)?, (IMAGE, IMAGE))?;
            let flat_labels: Vec<i64> = labels.flatten_all()?.to_dtype(DType::I64)?.to_vec1()?;
            let total = pe.dim(1)?;
            let mut pieces: Vec<Tensor> = Vec::with_capacity(total);
            for index in 0..total {
                let token = pe.narrow(1, index, 1)?;
                let label = flat_labels[index];
                // `pe[labels == i] += point_embeddings[i]`; `label = -1` is zeroed then replaced by
                // `not_a_point` (`tracker.py:557-561`).
                let table = if label < 0 {
                    self.not_a_point.reshape((1, 1, D_MODEL))?
                } else {
                    self.point_embeddings[label.clamp(0, 3) as usize].reshape((1, 1, D_MODEL))?
                };
                let base = if label < 0 { token.zeros_like()? } else { token };
                pieces.push(base.broadcast_add(&table)?);
            }
            sparse_parts.push(Tensor::cat(&pieces, 1)?);
        }

        if let Some(corners) = box_corners {
            let (b, _, _) = corners.dims3()?;
            batch = b;
            let pe = self.pe.forward_with_coords(&(corners + 0.5)?, (IMAGE, IMAGE))?;
            let first = pe.narrow(1, 0, 1)?.broadcast_add(&self.point_embeddings[2].reshape((1, 1, D_MODEL))?)?;
            let second = pe.narrow(1, 1, 1)?.broadcast_add(&self.point_embeddings[3].reshape((1, 1, D_MODEL))?)?;
            sparse_parts.push(Tensor::cat(&[first, second], 1)?);
        }

        let sparse = if sparse_parts.is_empty() {
            Tensor::zeros((batch, 0, D_MODEL), self.no_mask.dtype(), &device)?
        } else {
            Tensor::cat(&sparse_parts, 1)?
        };

        let dense = match masks {
            Some(masks) => {
                let x = self.mask_down_conv0.forward(masks)?;
                let x = layer_norm_2d(&x, Some(&self.mask_down_norm0.0), Some(&self.mask_down_norm0.1), EPS)?;
                let x = x.gelu_erf()?;
                let x = self.mask_down_conv3.forward(&x)?;
                let x = layer_norm_2d(&x, Some(&self.mask_down_norm4.0), Some(&self.mask_down_norm4.1), EPS)?;
                let x = x.gelu_erf()?;
                self.mask_down_conv6.forward(&x)?
            }
            None => self
                .no_mask
                .reshape((1, D_MODEL, 1, 1))?
                .broadcast_as((batch, D_MODEL, GRID, GRID))?
                .contiguous()?,
        };

        Ok(PromptEmbeddings { sparse, dense })
    }
}

/// Decoder output before the multimask selection.
pub struct MaskDecoderOutput {
    /// `[B, kept_tokens, 288, 288]`.
    pub masks: Tensor,
    /// `[B, kept_tokens]` predicted IoU.
    pub iou: Tensor,
    /// `[B, 1, 1]` object score logits.
    pub object_score: Tensor,
}

/// `SAMMaskDecoder`, the interactive single-object decoder (`tracker.py:436-515`).
pub struct SamMaskDecoder {
    transformer: TwoWayTransformer,
    iou_token: Tensor,
    mask_tokens: Tensor,
    obj_score_token: Tensor,
    up_conv0: Tensor,
    up_bias0: Tensor,
    up_norm0: (Tensor, Tensor),
    up_conv3: Tensor,
    up_bias3: Tensor,
    conv_s0: Conv2d,
    conv_s1: Conv2d,
    hypernetworks: Vec<Mlp>,
    iou_head: Mlp,
    obj_head: Mlp,
}

impl SamMaskDecoder {
    pub fn load(pack: &Pack) -> std::result::Result<Self, LoadError> {
        let base = "tracker.interactive_sam_mask_decoder";
        let mut hypernetworks = Vec::with_capacity(4);
        for index in 0..4 {
            hypernetworks.push(Mlp::load(pack, &format!("{base}.output_hypernetworks_mlps.{index}"), D_MODEL, D_MODEL, D_MODEL / 8, 3)?);
        }
        Ok(Self {
            transformer: TwoWayTransformer::load(pack, &format!("{base}.transformer"), 2)?,
            iou_token: pack.tensor(&format!("{base}.iou_token.weight"))?,
            mask_tokens: pack.tensor(&format!("{base}.mask_tokens.weight"))?,
            obj_score_token: pack.tensor(&format!("{base}.obj_score_token.weight"))?,
            up_conv0: pack.tensor(&format!("{base}.output_upscaling.0.weight"))?,
            up_bias0: pack.tensor(&format!("{base}.output_upscaling.0.bias"))?,
            up_norm0: (
                pack.tensor(&format!("{base}.output_upscaling.1.weight"))?,
                pack.tensor(&format!("{base}.output_upscaling.1.bias"))?,
            ),
            up_conv3: pack.tensor(&format!("{base}.output_upscaling.3.weight"))?,
            up_bias3: pack.tensor(&format!("{base}.output_upscaling.3.bias"))?,
            conv_s0: Conv2d::load(pack, &format!("{base}.conv_s0"), 1, 0)?,
            conv_s1: Conv2d::load(pack, &format!("{base}.conv_s1"), 1, 0)?,
            hypernetworks,
            iou_head: Mlp::load(pack, &format!("{base}.iou_prediction_head"), D_MODEL, D_MODEL, 4, 3)?,
            obj_head: Mlp::load(pack, &format!("{base}.pred_obj_score_head"), D_MODEL, D_MODEL, 1, 3)?,
        })
    }

    /// `image_embedding` is `[1, 256, 72, 72]`; `high_res` is the (288x288, 144x144) interactive pair.
    pub fn forward(
        &self,
        image_embedding: &Tensor,
        image_pe: &Tensor,
        prompt: &PromptEmbeddings,
        high_res: (&Tensor, &Tensor),
        multimask_output: bool,
    ) -> Result<MaskDecoderOutput> {
        let batch = prompt.sparse.dim(0)?;

        // Token order is [obj_score(1), iou(1), mask(4)] (`tracker.py:471-475`).
        let tokens = Tensor::cat(
            &[
                &self.obj_score_token.reshape((1, D_MODEL))?,
                &self.iou_token.reshape((1, D_MODEL))?,
                &self.mask_tokens,
            ],
            0,
        )?;
        let token_count = tokens.dim(0)?;
        let tokens = Tensor::cat(&[&tokens.unsqueeze(0)?.broadcast_as((batch, token_count, D_MODEL))?, &prompt.sparse], 1)?;

        let src = (image_embedding.broadcast_as((batch, D_MODEL, GRID, GRID))? + &prompt.dense)?;
        let pos = image_pe.broadcast_as((batch, image_pe.dim(1)?, GRID, GRID))?;
        let src_flat = src.flatten_from(2)?.permute((0, 2, 1))?.contiguous()?;
        let pos_flat = pos.flatten_from(2)?.permute((0, 2, 1))?.contiguous()?;

        let (hs, keys) = self.transformer.forward(&src_flat, &pos_flat, &tokens)?;
        let obj_token = hs.narrow(1, 0, 1)?.reshape((batch, D_MODEL))?;
        let iou_token = hs.narrow(1, 1, 1)?.reshape((batch, D_MODEL))?;
        let mask_tokens = hs.narrow(1, 2, 4)?.contiguous()?;

        let src_out = keys.permute((0, 2, 1))?.contiguous()?.reshape((batch, D_MODEL, GRID, GRID))?;

        // `_upscale_masks` (`tracker.py:425-433`): deconv, add the 144x144 neck, deconv, add 288x288.
        let dc1 = conv_transpose2x2(&self.up_conv0, Some(&self.up_bias0), &src_out)?;
        let dc1 = layer_norm_2d(&dc1, Some(&self.up_norm0.0), Some(&self.up_norm0.1), EPS)?;
        let upscaled = (dc1 + self.conv_s1.forward(high_res.1)?)?.gelu_erf()?;
        let dc2 = conv_transpose2x2(&self.up_conv3, Some(&self.up_bias3), &upscaled)?;
        let upscaled = (dc2 + self.conv_s0.forward(high_res.0)?)?.gelu_erf()?;

        let (_, _, out_h, out_w) = upscaled.dims4()?;
        let flat = upscaled.reshape((batch, D_MODEL / 8, out_h * out_w))?;

        let mut hyper: Vec<Tensor> = Vec::with_capacity(4);
        for (index, mlp) in self.hypernetworks.iter().enumerate() {
            let token = mask_tokens.narrow(1, index, 1)?.reshape((batch, D_MODEL))?;
            hyper.push(mlp.forward(&token)?);
        }
        let masks = Tensor::stack(&hyper, 1)?.matmul(&flat)?.reshape((batch, 4, out_h, out_w))?;

        let iou = self.iou_head.forward(&iou_token)?;
        let object_score = self.obj_head.forward(&obj_token)?;

        let (masks, iou) = if multimask_output {
            (masks.narrow(1, 1, 3)?.contiguous()?, iou.narrow(1, 1, 3)?.contiguous()?)
        } else {
            (masks.narrow(1, 0, 1)?.contiguous()?, iou.narrow(1, 0, 1)?.contiguous()?)
        };
        Ok(MaskDecoderOutput { masks, iou, object_score })
    }
}

/// The full interactive segmenter: prompt encoder, decoder and the selection rule.
pub struct Segmenter {
    prompt: SamPromptEncoder,
    decoder: SamMaskDecoder,
    no_mem_embed: Tensor,
}

/// A mask in 1008x1008 space with the scores the decoder produced for it.
pub struct SegmentResult {
    pub mask: Tensor,
    pub iou: f32,
    pub object_score: f32,
}

impl Segmenter {
    pub fn load(pack: &Pack) -> std::result::Result<Self, LoadError> {
        Ok(Self {
            prompt: SamPromptEncoder::load(pack)?,
            decoder: SamMaskDecoder::load(pack)?,
            no_mem_embed: pack.tensor("tracker.interactivity_no_mem_embed")?,
        })
    }

    /// `forward_segment` with a box prompt given as two corners in 1008x1008 pixel space.
    pub fn segment_box(&self, interactive: &[Tensor], corners: &Tensor) -> Result<SegmentResult> {
        let backbone = self.prepare(interactive)?;
        let prompt = self.prompt.forward(None, Some(corners), None)?;
        self.decode(&backbone, &prompt, (&interactive[0], &interactive[1]), false)
    }

    /// `forward_segment` with point prompts (pixel coordinates plus labels).
    pub fn segment_points(&self, interactive: &[Tensor], coords: &Tensor, labels: &Tensor) -> Result<SegmentResult> {
        let backbone = self.prepare(interactive)?;
        let prompt = self.prompt.forward(Some((coords, labels)), None, None)?;
        // `multimask_output = 0 < num_pts <= 1` (`detector.py:556-563`).
        let multimask = coords.dim(1)? == 1;
        self.decode(&backbone, &prompt, (&interactive[0], &interactive[1]), multimask)
    }

    /// `forward_segment` with a coarse mask logit prompt of shape `[B, 1, 1008, 1008]` (refinement).
    pub fn segment_mask(&self, interactive: &[Tensor], mask_logits: &Tensor) -> Result<SegmentResult> {
        let backbone = self.prepare(interactive)?;
        let (b, _, _, _) = mask_logits.dims4()?;
        // No point prompt: a single `label = -1` point, which is then padded once more.
        let coords = Tensor::zeros((b, 1, 2), mask_logits.dtype(), mask_logits.device())?;
        let labels = Tensor::full(-1i64, (b, 1), mask_logits.device())?;
        // `forward_sam_heads` resizes the mask prompt with `antialias=True` (`tracker.py:291-295`).
        let resized = resample_bilinear_antialias(mask_logits, PROMPT_GRID, PROMPT_GRID)?;
        let prompt = self.prompt.forward(Some((&coords, &labels)), None, Some(&resized))?;
        self.decode(&backbone, &prompt, (&interactive[0], &interactive[1]), false)
    }

    /// Adds `interactivity_no_mem_embed` to the 72x72 level (`detector.py:548-554`).
    fn prepare(&self, interactive: &[Tensor]) -> Result<Tensor> {
        let feat = &interactive[2];
        let (b, c, h, w) = feat.dims4()?;
        let flat = feat.flatten_from(2)?.permute((0, 2, 1))?.contiguous()?;
        let embedded = flat.broadcast_add(&self.no_mem_embed.to_dtype(flat.dtype())?)?;
        embedded.reshape((b, h, w, c))?.permute((0, 3, 1, 2))?.contiguous()
    }

    fn decode(&self, backbone: &Tensor, prompt: &PromptEmbeddings, high_res: (&Tensor, &Tensor), multimask: bool) -> Result<SegmentResult> {
        // `SAM3_DUMP_PROMPT=<prefix>` writes the prompt embeddings as f32 for the reference comparison.
        if let Ok(prefix) = std::env::var("SAM3_DUMP_PROMPT") {
            for (suffix, tensor) in [("dense", &prompt.dense), ("sparse", &prompt.sparse)] {
                let values = tensor.to_dtype(DType::F32)?.flatten_all()?.to_vec1::<f32>()?;
                let mut bytes = Vec::with_capacity(values.len() * 4);
                for v in &values {
                    bytes.extend_from_slice(&v.to_le_bytes());
                }
                let path = format!("{prefix}_{suffix}.bin");
                std::fs::write(&path, &bytes)?;
                println!("dumped {path} {:?}", tensor.dims());
            }
        }
        let image_pe = self.prompt.dense_pe(backbone.device())?.to_dtype(backbone.dtype())?;
        let out = self.decoder.forward(backbone, &image_pe, prompt, high_res, multimask)?;

        let object_score = out.object_score.flatten_all()?.to_dtype(DType::F32)?.to_vec1::<f32>()?[0];
        // `torch.where(object_score > 0, masks, NO_OBJ_SCORE)`.
        let masks = if object_score > 0.0 {
            out.masks
        } else {
            Tensor::full(NO_OBJ_SCORE, out.masks.dims().to_vec(), out.masks.device())?.to_dtype(out.masks.dtype())?
        };
        let ious: Vec<f32> = out.iou.flatten_all()?.to_dtype(DType::F32)?.to_vec1()?;
        let selected = if multimask {
            ious.iter()
                .enumerate()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(index, _)| index)
                .unwrap_or(0)
        } else {
            0
        };
        let low_res = masks.narrow(1, selected, 1)?.contiguous()?;
        let mask = interpolate_bilinear(&low_res, IMAGE, IMAGE)?;
        Ok(SegmentResult { mask, iou: ious.get(selected).copied().unwrap_or(0.0), object_score })
    }
}
