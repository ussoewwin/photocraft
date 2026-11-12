//! Native Rust LaMa Cleaner (Large Mask Inpainting)
//!
//! Provides pure Rust inpainting using the Big-LaMa ONNX model via ONNX Runtime (`ort`).
//! Supports arbitrary image dimensions by intelligent ROI cropping, scaling, and seamless alpha blending.

use std::path::Path;
use std::sync::Mutex;
use image::{imageops, GrayImage, Rgb, RgbImage};
use ndarray::Array4;
use ort::session::Session;
use ort::value::Value;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LamaError {
    #[error("ONNX Runtime error: {0}")]
    Ort(#[from] ort::Error),
    #[error("Image error: {0}")]
    Image(#[from] image::ImageError),
    #[error("Dimension mismatch: image is {img_w}x{img_h}, mask is {mask_w}x{mask_h}")]
    DimensionMismatch {
        img_w: u32,
        img_h: u32,
        mask_w: u32,
        mask_h: u32,
    },
}

pub struct LamaModel {
    session: Mutex<Session>,
}

impl LamaModel {
    /// Load the LaMa ONNX model from the given path.
    pub fn load(model_path: impl AsRef<Path>) -> Result<Self, LamaError> {
        let env = ort::init()
            .with_name("lama-cleaner")
            .build()?;
        let session = Session::builder(&env)?
            .commit_from_file(model_path)?;
        Ok(Self {
            session: Mutex::new(session),
        })
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

        // Prepare image tensor: [1, 3, 512, 512] normalized to 0.0..1.0
        let mut img_tensor = Array4::<f32>::zeros((1, 3, 512, 512));
        for y in 0..512 {
            for x in 0..512 {
                let pixel = image.get_pixel(x, y);
                img_tensor[[0, 0, y as usize, x as usize]] = pixel[0] as f32 / 255.0;
                img_tensor[[0, 1, y as usize, x as usize]] = pixel[1] as f32 / 255.0;
                img_tensor[[0, 2, y as usize, x as usize]] = pixel[2] as f32 / 255.0;
            }
        }

        // Prepare mask tensor: [1, 1, 512, 512] binary (0.0 or 1.0)
        let mut mask_tensor = Array4::<f32>::zeros((1, 1, 512, 512));
        for y in 0..512 {
            for x in 0..512 {
                let m = mask.get_pixel(x, y)[0];
                mask_tensor[[0, 0, y as usize, x as usize]] = if m > 127 { 1.0 } else { 0.0 };
            }
        }

        // Convert ndarray to ort Value
        let img_val = Value::from_array(img_tensor)?;
        let mask_val = Value::from_array(mask_tensor)?;

        // Run inference
        let mut session = self.session.lock().unwrap_or_else(|e| e.into_inner());
        let outputs = session.run(ort::inputs![
            "image" => img_val,
            "mask" => mask_val
        ])?;
        let (_shape, data) = outputs[0].try_extract_tensor::<f32>()?;

        // Construct result image
        // Note: big-lama.onnx outputs tensor values in [0.0, 255.0] directly.
        // Auto-detect scale in case a model variant outputs [0.0, 1.0].
        let max_val = data.iter().copied().fold(0.0f32, f32::max);
        let scale = if max_val > 1.5 { 1.0 } else { 255.0 };

        let mut result = RgbImage::new(512, 512);
        for y in 0..512 {
            for x in 0..512 {
                let r_idx = 0 * 512 * 512 + (y as usize) * 512 + (x as usize);
                let g_idx = 1 * 512 * 512 + (y as usize) * 512 + (x as usize);
                let b_idx = 2 * 512 * 512 + (y as usize) * 512 + (x as usize);

                let r = (data[r_idx] * scale).clamp(0.0, 255.0).round() as u8;
                let g = (data[g_idx] * scale).clamp(0.0, 255.0).round() as u8;
                let b = (data[b_idx] * scale).clamp(0.0, 255.0).round() as u8;
                result.put_pixel(x, y, Rgb([r, g, b]));
            }
        }

        Ok(result)
    }

    /// Inpaint an arbitrary-resolution image using a mask (IOPaint-compatible crop & pad strategy).
    ///
    /// Finds the bounding box of the masked region, expands it to at least 512x512 (with generous
    /// surrounding context so LaMa's Fast Fourier Convolutions can analyze the global/local structure),
    /// pads with reflection if the image is smaller than 512x512, runs 512x512 LaMa inference,
    /// and seamlessly blends the inpainted pixels back into the original image.
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
                    if x < min_x { min_x = x; }
                    if x > max_x { max_x = x; }
                    if y < min_y { min_y = y; }
                    if y > max_y { max_y = y; }
                    count += 1;
                }
            }
        }

        // If nothing is masked, return original image copy
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

        // 5. Compute soft feathered alpha mask at the boundary for seamless blending
        let mut result = image.clone();
        for y in 0..final_h {
            for x in 0..final_w {
                let mask_val = cropped_mask.get_pixel(x, y)[0];
                if mask_val > 0 {
                    // Check distance to nearest unmasked pixel within 3px for feathering
                    let mut min_dist_sq = 16i32;
                    let ix = x as i32;
                    let iy = y as i32;
                    for dy in -3i32..=3 {
                        for dx in -3i32..=3 {
                            let nx = ix + dx;
                            let ny = iy + dy;
                            if nx >= 0 && nx < final_w as i32 && ny >= 0 && ny < final_h as i32 {
                                if cropped_mask.get_pixel(nx as u32, ny as u32)[0] == 0 {
                                    let d2 = dx * dx + dy * dy;
                                    if d2 < min_dist_sq {
                                        min_dist_sq = d2;
                                    }
                                }
                            }
                        }
                    }
                    let feather = if min_dist_sq <= 1 {
                        0.6
                    } else if min_dist_sq <= 4 {
                        0.85
                    } else {
                        1.0
                    };
                    let alpha = (mask_val as f32 / 255.0) * feather;
                    let orig_p = cropped_img.get_pixel(x, y);
                    let inpaint_p = inpainted_roi.get_pixel(x, y);

                    let r = ((1.0 - alpha) * orig_p[0] as f32 + alpha * inpaint_p[0] as f32).round().clamp(0.0, 255.0) as u8;
                    let g = ((1.0 - alpha) * orig_p[1] as f32 + alpha * inpaint_p[1] as f32).round().clamp(0.0, 255.0) as u8;
                    let b = ((1.0 - alpha) * orig_p[2] as f32 + alpha * inpaint_p[2] as f32).round().clamp(0.0, 255.0) as u8;

                    result.put_pixel(final_min_x + x, final_min_y + y, Rgb([r, g, b]));
                }
            }
        }

        Ok(result)
    }
}
