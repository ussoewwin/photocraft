//! Native Rust + CUDA SAM 3.1 selection helper for PhotoCraft.
//!
//! The crate is standalone (it builds outside the PhotoCraft workspace) and depends on nothing but
//! its own data files, the HSWQ ConvRot INT8 pack, and CUDA. It never links against ComfyUI or
//! Python; the ComfyUI SAM 3.1 code is only the numerical reference the implementation was written
//! against.

pub mod clip;
pub mod detector;
pub mod loader;
pub mod nn;
pub mod select;
pub mod selection;
pub mod tokenizer;
pub mod tracker;
pub mod vitdet;
