use std::path::{Path, PathBuf};
use image::{RgbImage, GrayImage, Rgb, Luma, open};
use lama_cleaner::LamaModel;

fn print_usage() {
    println!("Native Rust LaMa Inpaint Cleaner");
    println!("Usage:");
    println!("  lama-cleaner --image <path> --mask <path> --output <path> [--model <path>]");
    println!("  lama-cleaner (runs built-in synthetic test)");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    let mut image_path: Option<PathBuf> = None;
    let mut mask_path: Option<PathBuf> = None;
    let mut output_path: Option<PathBuf> = None;
    let mut model_path: Option<PathBuf> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--image" | "-i" => {
                if i + 1 < args.len() {
                    image_path = Some(PathBuf::from(&args[i + 1]));
                    i += 1;
                }
            }
            "--mask" | "-m" => {
                if i + 1 < args.len() {
                    mask_path = Some(PathBuf::from(&args[i + 1]));
                    i += 1;
                }
            }
            "--output" | "-o" => {
                if i + 1 < args.len() {
                    output_path = Some(PathBuf::from(&args[i + 1]));
                    i += 1;
                }
            }
            "--model" => {
                if i + 1 < args.len() {
                    model_path = Some(PathBuf::from(&args[i + 1]));
                    i += 1;
                }
            }
            "--help" | "-h" => {
                print_usage();
                return Ok(());
            }
            _ => {}
        }
        i += 1;
    }

    // Determine model path
    let model = model_path.unwrap_or_else(|| {
        let default_loc = PathBuf::from(r"D:\USERFILES\GitHub\photocraft\plugins\big-lama.onnx");
        if default_loc.exists() {
            default_loc
        } else {
            // Check next to exe
            if let Ok(exe) = std::env::current_exe() {
                let sibling = exe.parent().unwrap_or(Path::new("")).join("big-lama.onnx");
                if sibling.exists() {
                    return sibling;
                }
            }
            default_loc
        }
    });

    if let (Some(img_p), Some(mask_p), Some(out_p)) = (image_path, mask_path, output_path) {
        if !model.exists() {
            eprintln!("Error: LaMa model not found at {}", model.display());
            std::process::exit(1);
        }

        let start = std::time::Instant::now();
        let lama = LamaModel::load(&model)?;
        let load_time = start.elapsed();

        let img = open(&img_p)?.to_rgb8();
        let mask = open(&mask_p)?.to_luma8();

        let infer_start = std::time::Instant::now();
        let result = lama.inpaint(&img, &mask)?;
        let infer_time = infer_start.elapsed();

        if let Some(parent) = out_p.parent() {
            std::fs::create_dir_all(parent)?;
        }
        result.save(&out_p)?;

        println!(
            "SUCCESS: Inpainted image saved to {} (Model load: {:.2?}, Inference: {:.2?})",
            out_p.display(),
            load_time,
            infer_time
        );
        return Ok(());
    }

    // If no arguments, run built-in test
    println!("=== Testing Native Rust LaMa Cleaner ===");

    if !model.exists() {
        eprintln!("Model not found at {}", model.display());
        return Ok(());
    }

    println!("Loading LaMa ONNX model from {}...", model.display());
    let lama = LamaModel::load(&model)?;
    println!("Model loaded successfully!");

    let mut img = RgbImage::new(512, 512);
    for y in 0..512 {
        for x in 0..512 {
            let color = if y < 256 {
                Rgb([100, 180, 255])
            } else {
                Rgb([60, 180, 75])
            };
            img.put_pixel(x, y, color);
        }
    }

    let mut mask = GrayImage::new(512, 512);
    for y in 180..330 {
        for x in 180..330 {
            img.put_pixel(x, y, Rgb([255, 30, 30]));
            mask.put_pixel(x, y, Luma([255]));
        }
    }

    let out_dir = Path::new(r"D:\USERFILES\GitHub\photocraft\plugins\lama-cleaner\output");
    std::fs::create_dir_all(out_dir)?;

    let input_path = out_dir.join("test_input.png");
    let mask_path = out_dir.join("test_mask.png");
    let result_path = out_dir.join("test_inpainted.png");

    img.save(&input_path)?;
    mask.save(&mask_path)?;

    println!("Running LaMa inference in pure Rust...");
    let start = std::time::Instant::now();
    let inpainted = lama.inpaint_512(&img, &mask)?;
    let elapsed = start.elapsed();
    println!("Inference completed in {:.2?}!", elapsed);

    inpainted.save(&result_path)?;
    println!("Saved inpainted result to {}", result_path.display());

    let center_pixel = inpainted.get_pixel(256, 256);
    println!("Original center pixel: [255, 30, 30] (Red)");
    println!("Inpainted center pixel: {:?}", center_pixel.0);

    println!("=== LaMa Inpaint Test Succeeded! ===");
    Ok(())
}
