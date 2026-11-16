//! The LaMa helper the PhotoCraft engine runs.
//!
//! Two ways in, plus the self-test:
//! - `lama-cleaner -i <in.png> -m <mask.png> -o <out.png> [--model <model>]` — one image, one
//!   process (the engine's fallback, and how this helper has always been called).
//! - `lama-cleaner --serve [--model <model>]` — load the model once and answer one JSON request
//!   per line on stdin with one JSON reply per line on stdout, so a stroke no longer pays for
//!   the model load:
//!   `{"image": "...", "mask": "...", "output": "..."}` →
//!   `{"ok": true, "infer_ms": 42.1, "provider": "CUDA"}`, or `{"ok": false, "error": "..."}`.
//!   Diagnostics go to stderr; stdout carries replies only.
//! - no arguments — the built-in synthetic test.
//!
//! Unknown options are an error, never silently ignored: `--serve` on an old helper must fail
//! loudly, because the engine treats a helper that answers something other than JSON as one that
//! cannot serve and falls back to a single run.

use image::{GrayImage, Rgb, RgbImage, open};
use lama_cleaner::LamaModel;
use serde::Deserialize;
use std::io::{BufRead, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

const USAGE: &str = "\
Native Rust LaMa Inpaint Cleaner
Usage:
  lama-cleaner --image <path> --mask <path> --output <path> [--model <path>]
  lama-cleaner --serve [--model <path>]
  lama-cleaner (no arguments: runs the built-in synthetic test)";

/// One request line in `--serve` mode.
#[derive(Deserialize)]
struct Request {
    #[serde(default)]
    warmup: bool,
    #[serde(default)]
    image: String,
    #[serde(default)]
    mask: String,
    #[serde(default)]
    output: String,
}

#[derive(Debug, PartialEq)]
enum Mode {
    /// The built-in synthetic test.
    Test,
    /// One image in, one image out.
    OneShot { image: PathBuf, mask: PathBuf, output: PathBuf },
    /// The resident JSON-lines loop.
    Serve,
}

#[derive(Debug, PartialEq)]
struct Parsed {
    mode: Mode,
    model: Option<PathBuf>,
}

fn parse_args(args: &[String]) -> Result<Parsed, String> {
    let mut image = None;
    let mut mask = None;
    let mut output = None;
    let mut model = None;
    let mut serve = false;
    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        let mut value = |name: &str| -> Result<PathBuf, String> {
            i += 1;
            args.get(i).map(PathBuf::from).ok_or_else(|| format!("{name} needs a path"))
        };
        match arg {
            "-i" | "--image" => image = Some(value("--image")?),
            "-m" | "--mask" => mask = Some(value("--mask")?),
            "-o" | "--output" => output = Some(value("--output")?),
            "--model" => model = Some(value("--model")?),
            "--serve" => serve = true,
            other => return Err(format!("unknown argument `{other}`")),
        }
        i += 1;
    }
    if serve {
        if image.is_some() || mask.is_some() || output.is_some() {
            return Err("--serve takes no --image/--mask/--output; it reads them from stdin".into());
        }
        return Ok(Parsed { mode: Mode::Serve, model });
    }
    match (image, mask, output) {
        (None, None, None) => Ok(Parsed { mode: Mode::Test, model }),
        (Some(image), Some(mask), Some(output)) => Ok(Parsed { mode: Mode::OneShot { image, mask, output }, model }),
        _ => Err("give --image, --mask and --output together".into()),
    }
}

/// The model to use: `--model`, then `PHOTOCRAFT_LAMA_MODEL`, then `big-lama.onnx` next to the
/// helper or in the folder above it (the `plugins` folder). No machine's path is written here.
fn resolve_model(explicit: Option<PathBuf>) -> Result<PathBuf, String> {
    if let Some(path) = explicit {
        return Ok(path);
    }
    if let Ok(value) = std::env::var("PHOTOCRAFT_LAMA_MODEL") {
        let value = value.trim();
        if !value.is_empty() {
            return Ok(PathBuf::from(value));
        }
    }
    let exe = std::env::current_exe().map_err(|e| format!("cannot find the helper's own path: {e}"))?;
    let dir = exe.parent().ok_or("the helper has no folder")?;
    for candidate in [dir.join("big-lama.onnx"), dir.join("..").join("big-lama.onnx")] {
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    Err(format!("big-lama.onnx was not found beside {}; pass --model <path>", exe.display()))
}

fn load(model: &Path) -> Result<LamaModel, String> {
    let started = Instant::now();
    let lama = LamaModel::load(model).map_err(|e| e.to_string())?;
    for note in lama.notes() {
        eprintln!("lama-cleaner: {note}");
    }
    eprintln!("lama-cleaner: {} provider, model loaded in {:.2?}", lama.provider().name(), started.elapsed());
    Ok(lama)
}

/// Inpaint `image` with `mask` into `output`, returning the second the inference took.
fn run_one(lama: &LamaModel, image: &str, mask: &str, output: &str) -> Result<f64, String> {
    let img = open(image).map_err(|e| format!("{image}: {e}"))?.to_rgb8();
    let msk = open(mask).map_err(|e| format!("{mask}: {e}"))?.to_luma8();
    let started = Instant::now();
    let result = lama.inpaint(&img, &msk).map_err(|e| e.to_string())?;
    let seconds = started.elapsed().as_secs_f64();
    if let Some(parent) = Path::new(output).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
        }
    }
    result.save(output).map_err(|e| format!("{output}: {e}"))?;
    Ok(seconds)
}

/// Warm up the session with a single 512x512 pass so cuDNN initializes its kernels once.
fn warmup_once(lama: &LamaModel) -> Result<f64, String> {
    let image = RgbImage::new(512, 512);
    let mask = GrayImage::new(512, 512);
    let started = Instant::now();
    lama.inpaint_512(&image, &mask).map_err(|e| e.to_string())?;
    Ok(started.elapsed().as_secs_f64())
}

/// The resident loop: one JSON request per line in, one JSON reply per line out.
///
/// No background warm-up thread is spawned blindly here: when the engine starts `--serve` on a
/// stroke, the first line on stdin is already waiting, and a background thread would grab the
/// session lock while `run_one` decodes the PNGs and force the first stroke to run `inpaint_512`
/// twice. Callers that pre-launch the worker while idle can send `{"warmup": true}` explicitly.
fn serve(model: &Path) -> Result<(), String> {
    let lama = load(model)?;
    let stdin = std::io::stdin();
    let mut out = BufWriter::new(std::io::stdout());
    for line in stdin.lock().lines() {
        let line = line.map_err(|e| format!("stdin: {e}"))?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let reply = match serde_json::from_str::<Request>(line) {
            Ok(request) => {
                let outcome = if request.warmup {
                    warmup_once(&lama)
                } else if request.image.is_empty() || request.mask.is_empty() || request.output.is_empty() {
                    Err("missing image, mask or output path".into())
                } else {
                    run_one(&lama, &request.image, &request.mask, &request.output)
                };
                match outcome {
                    Ok(seconds) => serde_json::json!({
                        "ok": true,
                        "infer_ms": seconds * 1000.0,
                        "provider": lama.provider().name(),
                    })
                    .to_string(),
                    Err(error) => serde_json::json!({ "ok": false, "error": error }).to_string(),
                }
            }
            Err(e) => serde_json::json!({ "ok": false, "error": format!("bad request: {e}") }).to_string(),
        };
        writeln!(out, "{reply}").map_err(|e| format!("stdout: {e}"))?;
        out.flush().map_err(|e| format!("stdout: {e}"))?;
    }
    Ok(())
}

fn one_shot(lama: &LamaModel, image: &Path, mask: &Path, output: &Path) -> Result<(), String> {
    let seconds = run_one(lama, &image.to_string_lossy(), &mask.to_string_lossy(), &output.to_string_lossy())?;
    println!(
        "SUCCESS: Inpainted image saved to {} (Inference: {:.2?}, Provider: {})",
        output.display(),
        std::time::Duration::from_secs_f64(seconds),
        lama.provider().name()
    );
    Ok(())
}

/// The built-in test: a blue/green field with a red square masked out, inpainted and saved.
fn self_test(model: &Path) -> Result<(), String> {
    println!("=== Testing Native Rust LaMa Cleaner ===");
    let lama = load(model)?;

    let mut img = RgbImage::new(512, 512);
    for y in 0..512 {
        for x in 0..512 {
            let color = if y < 256 { Rgb([100, 180, 255]) } else { Rgb([60, 180, 75]) };
            img.put_pixel(x, y, color);
        }
    }

    let mut mask = GrayImage::new(512, 512);
    for y in 180..330 {
        for x in 180..330 {
            img.put_pixel(x, y, Rgb([255, 30, 30]));
            mask.put_pixel(x, y, image::Luma([255]));
        }
    }

    let out_dir = std::env::temp_dir().join("photocraft-lama-selftest");
    std::fs::create_dir_all(&out_dir).map_err(|e| format!("{}: {e}", out_dir.display()))?;
    let started = Instant::now();
    let inpainted = lama.inpaint_512(&img, &mask).map_err(|e| e.to_string())?;
    println!("Inference completed in {:.2?} ({})", started.elapsed(), lama.provider().name());

    let result_path = out_dir.join("test_inpainted.png");
    inpainted.save(&result_path).map_err(|e| format!("{}: {e}", result_path.display()))?;
    println!("Saved inpainted result to {}", result_path.display());
    println!("Original center pixel: [255, 30, 30] (Red)");
    println!("Inpainted center pixel: {:?}", inpainted.get_pixel(256, 256).0);
    println!("=== LaMa Inpaint Test Succeeded! ===");
    Ok(())
}

fn main() -> std::process::ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("{USAGE}");
        return std::process::ExitCode::SUCCESS;
    }
    let parsed = match parse_args(&args) {
        Ok(parsed) => parsed,
        Err(error) => {
            eprintln!("lama-cleaner: {error}\n\n{USAGE}");
            return std::process::ExitCode::from(2);
        }
    };
    let model = match resolve_model(parsed.model) {
        Ok(model) => model,
        Err(error) => {
            eprintln!("lama-cleaner: {error}");
            return std::process::ExitCode::FAILURE;
        }
    };
    let result = match parsed.mode {
        Mode::Test => self_test(&model),
        Mode::OneShot { image, mask, output } => load(&model).and_then(|lama| one_shot(&lama, &image, &mask, &output)),
        Mode::Serve => serve(&model),
    };
    match result {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("lama-cleaner: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn no_arguments_run_the_self_test() {
        assert_eq!(parse_args(&args(&[])).unwrap().mode, Mode::Test);
    }

    #[test]
    fn the_single_shot_form_takes_all_three_paths() {
        let parsed = parse_args(&args(&["-i", "in.png", "-m", "mask.png", "-o", "out.png", "--model", "m.onnx"])).unwrap();
        assert_eq!(parsed.mode, Mode::OneShot { image: "in.png".into(), mask: "mask.png".into(), output: "out.png".into() });
        assert_eq!(parsed.model, Some(PathBuf::from("m.onnx")));
    }

    #[test]
    fn serve_is_its_own_mode_and_takes_no_paths() {
        assert_eq!(parse_args(&args(&["--serve", "--model", "m.onnx"])).unwrap().mode, Mode::Serve);
        let error = parse_args(&args(&["--serve", "-i", "in.png"])).unwrap_err();
        assert!(error.contains("--serve takes no"), "{error}");
    }

    /// An unknown option is an error: a helper that ignores it would hide a typo or an old build.
    #[test]
    fn an_unknown_option_is_an_error() {
        let error = parse_args(&args(&["--serv"])).unwrap_err();
        assert!(error.contains("unknown argument"), "{error}");
    }

    #[test]
    fn a_missing_path_is_an_error() {
        let error = parse_args(&args(&["-i"])).unwrap_err();
        assert!(error.contains("needs a path"), "{error}");
    }

    #[test]
    fn a_partial_set_of_paths_is_an_error() {
        let error = parse_args(&args(&["-i", "in.png", "-m", "mask.png"])).unwrap_err();
        assert!(error.contains("together"), "{error}");
    }
}
