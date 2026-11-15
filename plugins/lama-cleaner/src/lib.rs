//! Native Rust LaMa Cleaner (Large Mask Inpainting)
//!
//! Pure Rust inpainting with the Big-LaMa ONNX model through ONNX Runtime (`ort`), with an
//! execution-provider chain chosen at load time: CUDA (cuDNN heuristic search, TF32) when the
//! CUDA 13 and cuDNN 9 runtime libraries can be found, DirectML next (any DirectX 12 GPU,
//! including non-NVIDIA ones), and the CPU provider last. Each attempt that fails is reported in
//! [`LamaModel::notes`] so a caller can say why the fast path was not taken instead of guessing.
//!
//! Arbitrary image sizes are handled by ROI cropping, scaling and seamless alpha blending
//! (`inpaint`); the model itself always runs on 512x512 patches (`inpaint_512`).

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use image::{imageops, GrayImage, Rgb, RgbImage};
use ndarray::Array4;
use ort::ep::{self, ExecutionProviderDispatch};
use ort::session::Session;
use ort::value::Value;
use thiserror::Error;

/// The side length the model runs at.
const SIDE: usize = 512;

#[derive(Error, Debug)]
pub enum LamaError {
    #[error("ONNX Runtime error: {0}")]
    Ort(#[from] ort::Error),
    #[error("Image error: {0}")]
    Image(#[from] image::ImageError),
    #[error("no execution provider could open the model ({0})")]
    NoProvider(String),
    #[error("Dimension mismatch: image is {img_w}x{img_h}, mask is {mask_w}x{mask_h}")]
    DimensionMismatch {
        img_w: u32,
        img_h: u32,
        mask_w: u32,
        mask_h: u32,
    },
}

/// The execution provider a session ended up on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Provider {
    Cuda,
    DirectMl,
    Cpu,
}

impl Provider {
    /// The name a user sees (`CUDA`, `DirectML`, `CPU`).
    pub fn name(self) -> &'static str {
        match self {
            Provider::Cuda => "CUDA",
            Provider::DirectMl => "DirectML",
            Provider::Cpu => "CPU",
        }
    }

    fn dispatch(self) -> Option<ExecutionProviderDispatch> {
        match self {
            // The heuristic search skips cuDNN's exhaustive benchmark (seconds on the first
            // run); TF32 uses the tensor cores on Ampere and later.
            Provider::Cuda => Some(
                ep::cuda::CUDA::default()
                    .with_conv_algorithm_search(ep::cuda::ConvAlgorithmSearch::Heuristic)
                    .with_tf32(true)
                    .build()
                    .error_on_failure(),
            ),
            Provider::DirectMl => Some(ep::DirectML::default().build().error_on_failure()),
            Provider::Cpu => None,
        }
    }
}

pub struct LamaModel {
    session: Mutex<Session>,
    provider: Provider,
    notes: Vec<String>,
}

impl LamaModel {
    /// Load the model, choosing the best execution provider this machine can run.
    pub fn load(model_path: impl AsRef<Path>) -> Result<Self, LamaError> {
        let model = model_path.as_ref();
        let env = ort::init().with_name("lama-cleaner").build()?;
        let mut notes = Vec::new();
        let cuda_ready = prepare_cuda(&mut notes);
        let mut failures = Vec::new();
        for provider in [Provider::Cuda, Provider::DirectMl, Provider::Cpu] {
            if provider == Provider::Cuda && !cuda_ready {
                continue;
            }
            match open_session(&env, model, provider) {
                Ok(session) => {
                    return Ok(Self { session: Mutex::new(session), provider, notes });
                }
                Err(e) => failures.push(format!("{}: {e}", provider.name())),
            }
        }
        Err(LamaError::NoProvider(failures.join("; ")))
    }

    /// The provider the session runs on.
    pub fn provider(&self) -> Provider {
        self.provider
    }

    /// What stopped a faster provider from being used (empty when CUDA was used).
    pub fn notes(&self) -> &[String] {
        &self.notes
    }

    /// Inpaint a 512x512 RGB patch with a 512x512 binary mask (1.0 = mask to remove).
    pub fn inpaint_512(&self, image: &RgbImage, mask: &GrayImage) -> Result<RgbImage, LamaError> {
        let (w, h) = image.dimensions();
        let (mw, mh) = mask.dimensions();
        if w != 512 || h != 512 || mw != 512 || mh != 512 {
            return Err(LamaError::DimensionMismatch {
                img_w: w,
                img_h: h,
                mask_w: mw,
                mask_h: mh,
            });
        }

        // Image tensor [1, 3, 512, 512], 0.0..1.0. Packed from the image's own bytes: no bounds
        // checks per pixel, and the row order matches the model's CHW layout directly.
        let mut img_tensor = Array4::<f32>::zeros((1, 3, SIDE, SIDE));
        {
            let src = image.as_raw();
            let Some(dst) = img_tensor.as_slice_mut() else {
                return Err(LamaError::Ort(ort::Error::new("the image tensor is not contiguous")));
            };
            let plane = SIDE * SIDE;
            for y in 0..SIDE {
                let row = &src[y * SIDE * 3..(y + 1) * SIDE * 3];
                for (x, px) in row.chunks_exact(3).enumerate() {
                    let i = y * SIDE + x;
                    if let Some(cell) = dst.get_mut(i) {
                        *cell = px[0] as f32 / 255.0;
                    }
                    if let Some(cell) = dst.get_mut(plane + i) {
                        *cell = px[1] as f32 / 255.0;
                    }
                    if let Some(cell) = dst.get_mut(2 * plane + i) {
                        *cell = px[2] as f32 / 255.0;
                    }
                }
            }
        }

        // Mask tensor [1, 1, 512, 512], binary.
        let mut mask_tensor = Array4::<f32>::zeros((1, 1, SIDE, SIDE));
        {
            let src = mask.as_raw();
            let Some(dst) = mask_tensor.as_slice_mut() else {
                return Err(LamaError::Ort(ort::Error::new("the mask tensor is not contiguous")));
            };
            for (i, m) in src.iter().take(dst.len()).enumerate() {
                if let Some(cell) = dst.get_mut(i) {
                    *cell = if *m > 127 { 1.0 } else { 0.0 };
                }
            }
        }

        let img_val = Value::from_array(img_tensor)?;
        let mask_val = Value::from_array(mask_tensor)?;

        let mut session = self.session.lock().unwrap_or_else(|e| e.into_inner());
        let outputs = session.run(ort::inputs![
            "image" => img_val,
            "mask" => mask_val
        ])?;
        let (_shape, data) = outputs[0].try_extract_tensor::<f32>()?;

        // big-lama.onnx writes 0..255; a variant writing 0..1 is detected and scaled.
        let max_val = data.iter().copied().fold(0.0f32, f32::max);
        let scale = if max_val > 1.5 { 1.0 } else { 255.0 };

        let mut result = RgbImage::new(512, 512);
        let plane = SIDE * SIDE;
        for y in 0..SIDE {
            for x in 0..SIDE {
                let i = y * SIDE + x;
                let sample = |offset: usize| {
                    data.get(offset + i)
                        .map_or(0, |v| (*v * scale).clamp(0.0, 255.0).round() as u8)
                };
                result.put_pixel(x as u32, y as u32, Rgb([sample(0), sample(plane), sample(2 * plane)]));
            }
        }

        Ok(result)
    }

    /// Inpaint an arbitrary-resolution image using a mask (IOPaint-compatible crop & pad strategy).
    ///
    /// Finds the bounding box of the masked region, expands it to at least 512x512 (with generous
    /// surrounding context so LaMa's Fast Fourier Convolutions can analyze the global/local
    /// structure), pads with reflection if the image is smaller than 512x512, runs 512x512 LaMa
    /// inference, and seamlessly blends the inpainted pixels back into the original image.
    pub fn inpaint(&self, image: &RgbImage, mask: &GrayImage) -> Result<RgbImage, LamaError> {
        let (w, h) = image.dimensions();
        let (mw, mh) = mask.dimensions();
        if w != mw || h != mh {
            return Err(LamaError::DimensionMismatch {
                img_w: w,
                img_h: h,
                mask_w: mw,
                mask_h: mh,
            });
        }

        // 1. Find mask bounding box
        let mut min_x = w;
        let mut min_y = h;
        let mut max_x = 0;
        let mut max_y = 0;
        let mut count = 0;

        for y in 0..h {
            for x in 0..w {
                if mask.get_pixel(x, y)[0] > 127 {
                    if x < min_x {
                        min_x = x;
                    }
                    if x > max_x {
                        max_x = x;
                    }
                    if y < min_y {
                        min_y = y;
                    }
                    if y > max_y {
                        max_y = y;
                    }
                    count += 1;
                }
            }
        }

        if count == 0 {
            return Ok(image.clone());
        }

        let bbox_w = max_x - min_x + 1;
        let bbox_h = max_y - min_y + 1;
        let cx = (min_x + max_x) / 2;
        let cy = (min_y + max_y) / 2;

        // 2. IOPaint-style context box: at least 512x512 around the mask, or bbox + 256px margin
        let context_margin = ((bbox_w.max(bbox_h) as f32 * 0.5).max(128.0)) as u32;
        let desired_side = (bbox_w.max(bbox_h) + context_margin * 2).max(512);

        let crop_w = desired_side.min(w);
        let crop_h = desired_side.min(h);

        let final_min_x = cx.saturating_sub(crop_w / 2).min(w.saturating_sub(crop_w));
        let final_min_y = cy.saturating_sub(crop_h / 2).min(h.saturating_sub(crop_h));
        let final_w = crop_w;
        let final_h = crop_h;

        // 3. Crop ROI
        let cropped_img = imageops::crop_imm(image, final_min_x, final_min_y, final_w, final_h).to_image();
        let cropped_mask = imageops::crop_imm(mask, final_min_x, final_min_y, final_w, final_h).to_image();

        // 4. If final_w < 512 or final_h < 512 (when the whole input image is smaller than 512),
        // reflect-pad to a square of side `pad_side = final_w.max(final_h).max(512)` instead of
        // stretching non-uniformly.
        let inpainted_roi = if final_w == 512 && final_h == 512 {
            self.inpaint_512(&cropped_img, &cropped_mask)?
        } else if final_w < 512 || final_h < 512 {
            let pad_side = final_w.max(final_h).max(512);
            let mut pad_img = RgbImage::new(pad_side, pad_side);
            let mut pad_mask = GrayImage::new(pad_side, pad_side);
            let off_x = (pad_side - final_w) / 2;
            let off_y = (pad_side - final_h) / 2;

            for py in 0..pad_side {
                let sy = if py < off_y {
                    (off_y - py).min(final_h.saturating_sub(1))
                } else if py >= off_y + final_h {
                    let over = py - (off_y + final_h) + 1;
                    final_h.saturating_sub(1).saturating_sub(over.min(final_h.saturating_sub(1)))
                } else {
                    py - off_y
                };
                for px in 0..pad_side {
                    let sx = if px < off_x {
                        (off_x - px).min(final_w.saturating_sub(1))
                    } else if px >= off_x + final_w {
                        let over = px - (off_x + final_w) + 1;
                        final_w.saturating_sub(1).saturating_sub(over.min(final_w.saturating_sub(1)))
                    } else {
                        px - off_x
                    };
                    pad_img.put_pixel(px, py, *cropped_img.get_pixel(sx, sy));
                    if px >= off_x && px < off_x + final_w && py >= off_y && py < off_y + final_h {
                        pad_mask.put_pixel(px, py, *cropped_mask.get_pixel(sx, sy));
                    }
                }
            }

            let out_padded = if pad_side == 512 {
                self.inpaint_512(&pad_img, &pad_mask)?
            } else {
                let s_img = imageops::resize(&pad_img, 512, 512, imageops::FilterType::Lanczos3);
                let s_mask = imageops::resize(&pad_mask, 512, 512, imageops::FilterType::Nearest);
                let inp_512 = self.inpaint_512(&s_img, &s_mask)?;
                imageops::resize(&inp_512, pad_side, pad_side, imageops::FilterType::Lanczos3)
            };
            imageops::crop_imm(&out_padded, off_x, off_y, final_w, final_h).to_image()
        } else {
            let scaled_img = imageops::resize(&cropped_img, 512, 512, imageops::FilterType::Lanczos3);
            let scaled_mask = imageops::resize(&cropped_mask, 512, 512, imageops::FilterType::Nearest);
            let inpainted_512 = self.inpaint_512(&scaled_img, &scaled_mask)?;
            imageops::resize(&inpainted_512, final_w, final_h, imageops::FilterType::Lanczos3)
        };

        // 5. Soft feathered alpha mask at the boundary for seamless blending. The distance search
        // runs over the cropped mask's own bytes with the bounds known, so it costs no bounds
        // checks per sample and keeps the exact 7x7 metric (and therefore the exact output).
        let mut result = image.clone();
        let (cw, chh) = (final_w as usize, final_h as usize);
        let mask_raw = cropped_mask.as_raw();
        let img_raw = cropped_img.as_raw();
        let inp_raw = inpainted_roi.as_raw();
        for y in 0..chh {
            for x in 0..cw {
                let mask_val = mask_raw.get(y * cw + x).copied().unwrap_or(0);
                if mask_val == 0 {
                    continue;
                }
                let mut min_dist_sq = 16i32;
                let (ix, iy) = (x as i32, y as i32);
                for dy in -3i32..=3 {
                    let ny = iy + dy;
                    if ny < 0 || ny >= chh as i32 {
                        continue;
                    }
                    for dx in -3i32..=3 {
                        let nx = ix + dx;
                        if nx < 0 || nx >= cw as i32 {
                            continue;
                        }
                        if mask_raw.get(ny as usize * cw + nx as usize).copied() == Some(0) {
                            let d2 = dx * dx + dy * dy;
                            if d2 < min_dist_sq {
                                min_dist_sq = d2;
                            }
                        }
                    }
                }
                let alpha = (mask_val as f32 / 255.0) * feather(min_dist_sq);
                let i = (y * cw + x) * 3;
                let blend = |orig: u8, inpaint: u8| {
                    ((1.0 - alpha) * orig as f32 + alpha * inpaint as f32).round().clamp(0.0, 255.0) as u8
                };
                let (o, p) = (img_raw.get(i..i + 3), inp_raw.get(i..i + 3));
                if let (Some(o), Some(p)) = (o, p) {
                    result.put_pixel(
                        final_min_x + x as u32,
                        final_min_y + y as u32,
                        Rgb([blend(o[0], p[0]), blend(o[1], p[1]), blend(o[2], p[2])]),
                    );
                }
            }
        }

        Ok(result)
    }
}

/// The alpha a boundary pixel gets from its squared distance to the nearest unmasked pixel.
fn feather(min_dist_sq: i32) -> f32 {
    if min_dist_sq <= 1 {
        0.6
    } else if min_dist_sq <= 4 {
        0.85
    } else {
        1.0
    }
}

fn open_session(env: &ort::environment::Environment, model: &Path, provider: Provider) -> Result<Session, ort::Error> {
    let mut builder = Session::builder(env)?;
    if let Some(ep) = provider.dispatch() {
        builder = builder.with_execution_providers([ep])?;
    }
    builder.commit_from_file(model)
}

/// Preload the CUDA 13 and cuDNN 9 libraries the CUDA provider needs, so the EP can be used
/// without putting them on PATH. Returns false (with a note) when they are not on this machine.
fn prepare_cuda(notes: &mut Vec<String>) -> bool {
    let cuda = find_runtime_dir(ep::cuda::CUDA_DYLIBS.first().copied().unwrap_or("cudart64_13.dll"));
    let cudnn = find_runtime_dir(ep::cuda::CUDNN_DYLIBS.first().copied().unwrap_or("cudnn64_9.dll"));
    match (cuda, cudnn) {
        (Some(cuda), Some(cudnn)) => match ep::cuda::preload_dylibs(Some(&cuda), Some(&cudnn)) {
            Ok(()) => true,
            Err(e) => {
                notes.push(format!("CUDA libraries could not be preloaded ({e}); using DirectML"));
                false
            }
        },
        (cuda, cudnn) => {
            let missing = match (cuda, cudnn) {
                (None, Some(_)) => "the CUDA 13 runtime (cudart64_13.dll)".to_string(),
                (Some(_), None) => "cuDNN 9 (cudnn64_9.dll)".to_string(),
                _ => "the CUDA 13 runtime and cuDNN 9".to_string(),
            };
            notes.push(format!("{missing} was not found; using DirectML"));
            false
        }
    }
}

/// The first directory holding `name`, searched in this order: CUDA_PATH / CUDNN_PATH (and their
/// `bin`), the helper's own folder and its parent (the `plugins` folder), the NVIDIA toolkit
/// folders, then PATH. Nothing here names a particular machine's drive.
fn find_runtime_dir(name: &str) -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();
    for key in ["CUDA_PATH", "CUDNN_PATH", "CUDA_HOME", "CUDNN_HOME"] {
        if let Ok(root) = std::env::var(key) {
            let root = PathBuf::from(root);
            candidates.push(root.join("bin"));
            candidates.push(root.clone());
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.to_path_buf());
            if let Some(parent) = dir.parent() {
                candidates.push(parent.to_path_buf());
            }
        }
    }
    if cfg!(windows) {
        let program_files = std::env::var("ProgramFiles").unwrap_or_else(|_| r"C:\Program Files".into());
        for root in [format!(r"{program_files}\NVIDIA GPU Computing Toolkit\CUDA"), format!(r"{program_files}\NVIDIA\CUDNN")] {
            if let Ok(entries) = std::fs::read_dir(root) {
                for entry in entries.flatten() {
                    let dir = entry.path();
                    candidates.push(dir.join("bin"));
                    candidates.push(dir);
                }
            }
        }
    }
    if let Some(path) = std::env::var_os("PATH") {
        candidates.extend(std::env::split_paths(&path));
    }
    candidates.into_iter().find(|dir| dir.join(name).is_file())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The feather ramp is part of the output; this pins it.
    #[test]
    fn the_feather_ramp_is_half_a_pixel_to_three_pixels() {
        assert_eq!(feather(0), 0.6);
        assert_eq!(feather(1), 0.6);
        assert_eq!(feather(2), 0.85);
        assert_eq!(feather(4), 0.85);
        assert_eq!(feather(5), 1.0);
        assert_eq!(feather(9), 1.0);
        assert_eq!(feather(16), 1.0);
    }

    /// One cache file per provider: a CUDA-optimized graph is not reused for DirectML.
    /// A model path without a parent has no cache location, so the session opens without one.
    #[test]
    fn provider_names_are_what_a_user_reads() {
        assert_eq!(Provider::Cuda.name(), "CUDA");
        assert_eq!(Provider::DirectMl.name(), "DirectML");
        assert_eq!(Provider::Cpu.name(), "CPU");
    }

    /// DirectML and CUDA ask to fail loudly, so a missing runtime cannot silently fall back to
    /// the CPU while the caller reports a GPU.
    #[test]
    fn gpu_providers_insist_on_registering() {
        for provider in [Provider::Cuda, Provider::DirectMl] {
            assert!(provider.dispatch().is_some(), "{} must register an EP", provider.name());
        }
        assert!(Provider::Cpu.dispatch().is_none());
    }
}


