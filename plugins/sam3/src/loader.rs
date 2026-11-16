//! Direct CUDA loader for the HSWQ ConvRot INT8 SAM 3.1 pack.
//!
//! The pack is the SAM 3.1 Multiplex checkpoint with 41 `nn.Linear` layers stored as ConvRot
//! INT8: `W_rot = W @ H_256^T`, `q = round(W_rot / s)`, `s = amax(row) / 127`. Because `H_256` is
//! the normalised fourth Kronecker power of the 4x4 regular Hadamard matrix it is symmetric and
//! orthogonal, so `H^T = H` and `H^2 = I` and the float weight comes back as `W = (q * s) @ H`.
//!
//! This module memory-maps the pack and resolves logical module names to GPU tensors:
//!
//! * **dequantisation** of the 41 ConvRot INT8 weights on the device (`W` above, cast to F16);
//! * **`tracker.model.*` → `tracker.*`** (457 keys in the pack);
//! * **fused QKV split** for the 24 CLIP `in_proj_weight` / `in_proj_bias` pairs;
//! * **fused QKV split** for the 37 detector `SplitMHA` `in_proj_weight` / `in_proj_bias` pairs;
//! * **`TwoWayTransformer` rename** (`mlp.lin1` → `mlp.0`, `mlp.lin2` → `mlp.2`,
//!   `norm_final_attn` → `norm_final`).
//!
//! Nothing here needs ComfyUI or Python: `clip_convert/convert_sam31_convrot_int8.py` and
//! `comfy/supported_models.py` are the written specification this file reproduces.

use std::collections::HashMap;
use std::path::Path;

use candle_core::{DType, Device, Tensor};
use memmap2::Mmap;
use safetensors::{Dtype as StDtype, SafeTensors};
use thiserror::Error;

/// The ConvRot group size used by the pack; the per-layer value is read from `comfy_quant`.
pub const GROUP_SIZE: usize = 256;

#[derive(Debug, Error)]
pub enum LoadError {
    #[error("cannot read {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("the pack is not valid safetensors: {0}")]
    Safetensors(String),
    #[error("tensor {0} is missing from the pack")]
    Missing(String),
    #[error("tensor {0} has an unexpected layout: {1}")]
    Layout(String, String),
    #[error("tensor {0} carries an unsupported dtype {1:?}")]
    Dtype(String, StDtype),
    #[error("{0}")]
    Format(String),
    #[error(transparent)]
    Candle(#[from] candle_core::Error),
}

type Result<T> = std::result::Result<T, LoadError>;

/// One tensor as it sits in the memory-mapped file.
struct RawEntry {
    dtype: StDtype,
    shape: Vec<usize>,
    offset: usize,
    len: usize,
}

/// How a logical module weight is produced from the file.
enum Plan {
    /// Already floating point: upload straight from the mapping.
    Direct { raw: String, dtype: DType, shape: Vec<usize> },
    /// ConvRot INT8: dequantise and un-rotate on the device.
    Dequant { raw: String, scale: String, shape: Vec<usize>, groupsize: usize },
    /// One third of a fused `in_proj` tensor.
    Split { raw: String, part: usize, shape: Vec<usize>, quant: Option<QuantSidecar> },
}

struct QuantSidecar {
    scale: String,
    groupsize: usize,
}

/// The pack, opened over a memory map and resolved through logical module names.
pub struct Pack {
    mmap: Mmap,
    device: Device,
    raw: HashMap<String, RawEntry>,
    plan: HashMap<String, Plan>,
    /// `H_size` matrices, built once per group size on the device.
    hadamard: HashMap<usize, Tensor>,
    /// Number of ConvRot INT8 weights that were found.
    pub dequantised: usize,
    /// Number of fused QKV tensors that were split.
    pub split: usize,
}

/// The four key remaps the reference loader applies before the model is built
/// (`comfy/supported_models.py:2569-2600`).
pub fn remap_name(name: &str) -> String {
    let mut out = match name.strip_prefix("tracker.model.") {
        Some(rest) => format!("tracker.{rest}"),
        None => name.to_string(),
    };
    out = out
        .replace(".mlp.lin1.", ".mlp.0.")
        .replace(".mlp.lin2.", ".mlp.2.")
        .replace(".norm_final_attn.", ".norm_final.");
    out
}

/// Maps a safetensors dtype to the candle dtype used to upload it.
fn upload_dtype(name: &str, dtype: StDtype) -> Result<DType> {
    Ok(match dtype {
        StDtype::F16 => DType::F16,
        StDtype::F32 => DType::F32,
        StDtype::BF16 => DType::BF16,
        other => return Err(LoadError::Dtype(name.to_string(), other)),
    })
}

impl Pack {
    /// Opens `path` and indexes every module, without uploading anything yet.
    pub fn open(path: &Path, device: Device) -> Result<Self> {
        let file = std::fs::File::open(path).map_err(|source| LoadError::Io {
            path: path.display().to_string(),
            source,
        })?;
        // SAFETY: the pack is read-only while mapped; no writer is expected to truncate it.
        let mmap = unsafe { Mmap::map(&file) }.map_err(|source| LoadError::Io {
            path: path.display().to_string(),
            source,
        })?;

        let mut raw: HashMap<String, RawEntry> = HashMap::new();
        {
            let st = SafeTensors::deserialize(&mmap).map_err(|e| LoadError::Safetensors(e.to_string()))?;
            let base = mmap.as_ptr() as usize;
            for name in st.names() {
                let view = st.tensor(name).map_err(|e| LoadError::Safetensors(e.to_string()))?;
                // The mapping owns the bytes, so the view is a borrow into it and the pointer
                // arithmetic below yields the tensor's offset inside the mapping.
                let bytes: &[u8] = view.data();
                let offset = bytes.as_ptr() as usize - base;
                raw.insert(
                    name.to_string(),
                    RawEntry {
                        dtype: view.dtype(),
                        shape: view.shape().to_vec(),
                        offset,
                        len: bytes.len(),
                    },
                );
            }
        }

        let mut plan: HashMap<String, Plan> = HashMap::new();
        let mut dequantised = 0usize;
        let mut split = 0usize;

        let mut names: Vec<&String> = raw.keys().collect();
        names.sort();

        for name in names {
            let entry = &raw[name];
            // Quantisation side-cars never become module weights.
            if name.ends_with(".comfy_quant") || name.ends_with(".weight_scale") {
                continue;
            }

            if let Some(stem) = name.strip_suffix(".in_proj_weight") {
                let quant = if entry.dtype == StDtype::I8 {
                    Some(QuantSidecar { scale: format!("{stem}.weight_scale"), groupsize: groupsize_of(&raw_reader(&mmap, &raw), stem)? })
                } else {
                    None
                };
                let half_dim = entry.shape[0] / 3;
                for (part, suffix) in ["q", "k", "v"].iter().enumerate() {
                    plan.insert(
                        format!("{}.{}_proj.weight", remap_name(stem), suffix),
                        Plan::Split {
                            raw: name.clone(),
                            part,
                            shape: vec![half_dim, entry.shape[1]],
                            quant: quant.as_ref().map(|q| QuantSidecar { scale: q.scale.clone(), groupsize: q.groupsize }),
                        },
                    );
                }
                split += 1;
                continue;
            }
            if let Some(stem) = name.strip_suffix(".in_proj_bias") {
                let quant = if entry.dtype == StDtype::I8 {
                    Some(QuantSidecar {
                        scale: format!("{stem}.weight_scale"),
                        groupsize: groupsize_of(&raw_reader(&mmap, &raw), &format!("{stem}.in_proj_weight"))?,
                    })
                } else {
                    None
                };
                let half_dim = entry.shape[0] / 3;
                for (part, suffix) in ["q", "k", "v"].iter().enumerate() {
                    plan.insert(
                        format!("{}.{}_proj.bias", remap_name(stem), suffix),
                        Plan::Split {
                            raw: name.clone(),
                            part,
                            shape: vec![half_dim],
                            quant: quant.as_ref().map(|q| QuantSidecar { scale: q.scale.clone(), groupsize: q.groupsize }),
                        },
                    );
                }
                split += 1;
                continue;
            }

            if entry.dtype == StDtype::I8 {
                let stem = name.strip_suffix(".weight").unwrap_or(name);
                plan.insert(
                    remap_name(name),
                    Plan::Dequant {
                        raw: name.clone(),
                        scale: format!("{stem}.weight_scale"),
                        shape: entry.shape.clone(),
                        groupsize: groupsize_of(&raw_reader(&mmap, &raw), stem)?,
                    },
                );
                dequantised += 1;
                continue;
            }

            plan.insert(
                remap_name(name),
                Plan::Direct { raw: name.clone(), dtype: upload_dtype(name, entry.dtype)?, shape: entry.shape.clone() },
            );
        }

        // Every group size the pack mentions gets its H matrix built once, here, so resolving a
        // tensor never needs a mutable borrow.
        let mut sizes: Vec<usize> = plan
            .values()
            .filter_map(|p| match p {
                Plan::Dequant { groupsize, .. } => Some(*groupsize),
                Plan::Split { quant: Some(q), .. } => Some(q.groupsize),
                _ => None,
            })
            .collect();
        sizes.sort_unstable();
        sizes.dedup();
        let mut hadamard = HashMap::new();
        for size in sizes {
            let h = build_hadamard(size)?;
            hadamard.insert(size, Tensor::from_vec(h, (size, size), &device)?);
        }

        Ok(Self { mmap, device, raw, plan, hadamard, dequantised, split })
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    /// Every logical module name the pack can serve.
    pub fn names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.plan.keys().map(|s| s.as_str()).collect();
        names.sort_unstable();
        names
    }

    pub fn contains(&self, name: &str) -> bool {
        self.plan.contains_key(name)
    }

    /// True when the logical name resolves through a ConvRot INT8 source.
    pub fn is_convrot(&self, name: &str) -> bool {
        match self.plan.get(name) {
            Some(Plan::Dequant { .. }) => true,
            Some(Plan::Split { quant: Some(_), .. }) => true,
            _ => false,
        }
    }

    pub fn raw_count(&self) -> usize {
        self.raw.len()
    }

    /// The bytes of one raw tensor.
    fn raw_bytes(&self, name: &str) -> Result<&[u8]> {
        let entry = self.raw.get(name).ok_or_else(|| LoadError::Missing(name.to_string()))?;
        Ok(&self.mmap[entry.offset..entry.offset + entry.len])
    }

    /// The un-rotated weight in f32 exactly as `tensor` computes it, before the f16 cast.
    pub fn debug_unrotated_f32(&self, name: &str) -> Result<Vec<f32>> {
        let plan = self.plan.get(name).ok_or_else(|| LoadError::Missing(name.to_string()))?;
        let Plan::Dequant { raw, scale, shape, groupsize } = plan else {
            return Err(LoadError::Format(format!("{name} is not a ConvRot INT8 weight")));
        };
        let (out_dim, in_dim) = (shape[0], shape[1]);
        let q = int8_to_f32(self.raw_bytes(raw)?);
        let s: Vec<f32> = self
            .raw_bytes(scale)?
            .chunks_exact(4)
            .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
            .collect();
        let q = Tensor::from_vec(q, (out_dim, in_dim), &self.device)?;
        let s = Tensor::from_vec(s, (out_dim, 1), &self.device)?;
        let deq = q.broadcast_mul(&s)?;
        let groups = in_dim / groupsize;
        let h = self.hadamard(*groupsize)?;
        let un = deq
            .reshape((out_dim * groups, *groupsize))?
            .contiguous()?
            .matmul(&h)?
            .reshape((out_dim, in_dim))?;
        Ok(un.flatten_all()?.to_vec1::<f32>()?)
    }

    /// The dequantised-but-still-rotated weight (`q * scale`) as host floats, for verification.
    pub fn debug_scaled(&self, name: &str) -> Result<Vec<f32>> {
        let plan = self.plan.get(name).ok_or_else(|| LoadError::Missing(name.to_string()))?;
        match plan {
            Plan::Dequant { raw, scale, shape, .. } => {
                let (out_dim, in_dim) = (shape[0], shape[1]);
                let q = int8_to_f32(self.raw_bytes(raw)?);
                let s: Vec<f32> = self
                    .raw_bytes(scale)?
                    .chunks_exact(4)
                    .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
                    .collect();
                let mut out = vec![0.0f32; out_dim * in_dim];
                for row in 0..out_dim {
                    let sr = s[row];
                    for col in 0..in_dim {
                        out[row * in_dim + col] = q[row * in_dim + col] * sr;
                    }
                }
                Ok(out)
            }
            _ => Err(LoadError::Format(format!("{name} is not a ConvRot INT8 weight"))),
        }
    }

    /// The `H_size` matrix for this pack, built when the pack was opened.
    fn hadamard(&self, size: usize) -> Result<Tensor> {
        self.hadamard
            .get(&size)
            .cloned()
            .ok_or_else(|| LoadError::Format(format!("no Hadamard matrix was built for group size {size}")))
    }

    /// Resolves one logical module weight to a device tensor.
    pub fn tensor(&self, name: &str) -> Result<Tensor> {
        let plan = self.plan.get(name).ok_or_else(|| LoadError::Missing(name.to_string()))?;
        match plan {
            Plan::Direct { raw, dtype, shape } => {
                let bytes = self.raw_bytes(raw)?;
                Ok(Tensor::from_raw_buffer(bytes, *dtype, shape, &self.device)?)
            }
            Plan::Dequant { raw, scale, shape, groupsize } => {
                let (out_dim, in_dim) = (shape[0], shape[1]);
                let q = int8_to_f32(self.raw_bytes(raw)?);
                let s = self.raw_bytes(scale)?.chunks_exact(4).map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect::<Vec<f32>>();
                if s.len() != out_dim {
                    return Err(LoadError::Layout(name.to_string(), format!("scale has {} rows, weight has {out_dim}", s.len())));
                }
                let q = Tensor::from_vec(q, (out_dim, in_dim), &self.device)?;
                let s = Tensor::from_vec(s, (out_dim, 1), &self.device)?;
                let deq = q.broadcast_mul(&s)?;
                let groups = in_dim / groupsize;
                if groups * groupsize != in_dim {
                    return Err(LoadError::Layout(name.to_string(), format!("input dim {in_dim} is not a multiple of {groupsize}")));
                }
                let h = self.hadamard(*groupsize)?;
                // candle's matmul multiplies the trailing two dimensions, so the groups are folded
                // into the row axis: (O*G, gs) @ (gs, gs) -> (O*G, gs).
                let un = deq
                    .reshape((out_dim * groups, *groupsize))?
                    .contiguous()?
                    .matmul(&h)?
                    .reshape((out_dim, in_dim))?;
                Ok(un.to_dtype(DType::F16)?)
            }
            Plan::Split { raw, part, shape, quant } => {
                let full = match quant {
                    Some(sidecar) => {
                        let entry = self.raw.get(raw).ok_or_else(|| LoadError::Missing(raw.clone()))?;
                        let rows = entry.shape[0];
                        let inner = if entry.shape.len() > 1 { entry.shape[1] } else { 1 };
                        let q = int8_to_f32(self.raw_bytes(raw)?);
                        let s = self.raw_bytes(&sidecar.scale)?.chunks_exact(4).map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect::<Vec<f32>>();
                        let q = Tensor::from_vec(q, (rows, inner), &self.device)?;
                        let s = Tensor::from_vec(s.clone(), (rows, 1), &self.device)?;
                        let deq = q.broadcast_mul(&s)?;
                        if inner > 1 {
                            let groups = inner / sidecar.groupsize;
                            let h = self.hadamard(sidecar.groupsize)?;
                            deq
                                .reshape((rows * groups, sidecar.groupsize))?
                                .contiguous()?
                                .matmul(&h)?
                                .reshape((rows, inner))?
                                .to_dtype(DType::F16)?
                        } else {
                            deq.to_dtype(DType::F16)?
                        }
                    }
                    None => {
                        let entry = self.raw.get(raw).ok_or_else(|| LoadError::Missing(raw.clone()))?;
                        let dtype = upload_dtype(raw, entry.dtype)?;
                        let bytes = self.raw_bytes(raw)?;
                        Tensor::from_raw_buffer(bytes, dtype, &entry.shape, &self.device)?
                    }
                };
                let per = shape[0];
                Ok(full.narrow(0, *part * per, per)?)
            }
        }
    }
}

/// Borrowing view over the mapping plus the index, for side-car lookups during indexing.
struct RawReader<'a> {
    mmap: &'a [u8],
    raw: &'a HashMap<String, RawEntry>,
}

impl<'a> RawReader<'a> {
    fn bytes(&self, name: &str) -> Result<&'a [u8]> {
        let entry = self.raw.get(name).ok_or_else(|| LoadError::Missing(name.to_string()))?;
        Ok(&self.mmap[entry.offset..entry.offset + entry.len])
    }
}

fn raw_reader<'a>(mmap: &'a Mmap, raw: &'a HashMap<String, RawEntry>) -> RawReader<'a> {
    RawReader { mmap: &mmap[..], raw }
}

/// Reads `convrot_groupsize` out of the layer's `comfy_quant` side-car.
fn groupsize_of(reader: &RawReader<'_>, stem: &str) -> Result<usize> {
    let name = format!("{stem}.comfy_quant");
    let bytes = reader.bytes(&name)?;
    let json: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|e| LoadError::Format(format!("{name} is not JSON: {e}")))?;
    if !json["convrot"].as_bool().unwrap_or(false) {
        return Err(LoadError::Format(format!("{name} does not set convrot")));
    }
    Ok(json["convrot_groupsize"].as_u64().unwrap_or(GROUP_SIZE as u64) as usize)
}

fn int8_to_f32(bytes: &[u8]) -> Vec<f32> {
    bytes.iter().map(|b| (*b as i8) as f32).collect()
}

/// `H_size = H_4^{⊗ log4(size)} / sqrt(size)`, row-major on the host.
pub fn build_hadamard(size: usize) -> Result<Vec<f32>> {
    let h4 = [
        1.0f32, 1.0, 1.0, -1.0, //
        1.0, 1.0, -1.0, 1.0, //
        1.0, -1.0, 1.0, 1.0, //
        -1.0, 1.0, 1.0, 1.0,
    ];
    if size < 4 || size % 4 != 0 || !(size as f64).log(4.0).fract().eq(&0.0) {
        return Err(LoadError::Format(format!("Hadamard size must be a power of four, got {size}")));
    }
    let mut h = h4.to_vec();
    let mut current = 4usize;
    while current < size {
        let n = current;
        let mut next = vec![0.0f32; n * n * 16];
        for r in 0..n {
            for c in 0..n {
                let v = h[r * n + c];
                for i in 0..4 {
                    for j in 0..4 {
                        next[(r * 4 + i) * n * 4 + (c * 4 + j)] = v * h4[i * 4 + j];
                    }
                }
            }
        }
        h = next;
        current *= 4;
    }
    let scale = 1.0 / (size as f32).sqrt();
    Ok(h.iter().map(|v| v * scale).collect())
}

/// Opens the CUDA device, falling back to the CPU when no GPU is present.
pub fn default_device() -> (Device, bool) {
    match Device::new_cuda(0) {
        Ok(device) => (device, true),
        Err(_) => (Device::Cpu, false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `H_4` is symmetric and orthogonal, so `H_256^T = H_256` and `H_256^2 = I`.
    #[test]
    fn hadamard_is_symmetric_and_orthogonal() {
        let h = build_hadamard(16).expect("H_16");
        for r in 0..16 {
            for c in 0..16 {
                assert!((h[r * 16 + c] - h[c * 16 + r]).abs() < 1e-7, "H is not symmetric at {r},{c}");
            }
        }
        // H^2 = I: the (r,c) entry of H*H is the dot product of row r and column c.
        for r in 0..16 {
            for c in 0..16 {
                let mut acc = 0.0f32;
                for k in 0..16 {
                    acc += h[r * 16 + k] * h[k * 16 + c];
                }
                let expected = if r == c { 1.0 } else { 0.0 };
                assert!((acc - expected).abs() < 1e-6, "H^2 at {r},{c} is {acc}, expected {expected}");
            }
        }
    }

    /// `H_256` is what the pack uses; check its first row and the normalisation.
    #[test]
    fn hadamard_256_matches_the_converter() {
        let h = build_hadamard(256).expect("H_256");
        assert_eq!(h.len(), 256 * 256);
        // Every entry is ±1/16.
        for value in &h {
            assert!((value.abs() - 0.0625).abs() < 1e-6, "unexpected magnitude {value}");
        }
        // First row of H_4^⊗4 is the outer product of the first rows of H_4.
        let first_row = [1.0f32, 1.0, 1.0, -1.0];
        for (index, value) in h[..256].iter().enumerate() {
            let mut expected = 1.0f32;
            for shift in [0usize, 2, 4, 6] {
                expected *= first_row[(index >> shift) & 3];
            }
            assert!((value - expected / 16.0).abs() < 1e-6, "H_256[0][{index}] = {value}, expected {}", expected / 16.0);
        }
    }

    /// The four remaps, including the tracker rename and the TwoWayTransformer field names.
    #[test]
    fn remaps_cover_the_four_rules() {
        assert_eq!(remap_name("tracker.model.interactivity_no_mem_embed"), "tracker.interactivity_no_mem_embed");
        assert_eq!(
            remap_name("tracker.model.interactive_sam_mask_decoder.transformer.layers.0.mlp.lin1.weight"),
            "tracker.interactive_sam_mask_decoder.transformer.layers.0.mlp.0.weight"
        );
        assert_eq!(
            remap_name("tracker.model.sam_mask_decoder.transformer.layers.1.mlp.lin2.weight"),
            "tracker.sam_mask_decoder.transformer.layers.1.mlp.2.weight"
        );
        assert_eq!(
            remap_name("tracker.model.sam_mask_decoder.transformer.norm_final_attn.weight"),
            "tracker.sam_mask_decoder.transformer.norm_final.weight"
        );
        // Untouched keys stay identical.
        assert_eq!(
            remap_name("detector.backbone.vision_backbone.trunk.blocks.0.norm1.weight"),
            "detector.backbone.vision_backbone.trunk.blocks.0.norm1.weight"
        );
    }
}
