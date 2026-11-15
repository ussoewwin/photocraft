//! `sam3-select` — native Rust + CUDA SAM 3.1 selection helper for PhotoCraft.
//!
//! Stage M1: open the HSWQ ConvRot INT8 pack on the CUDA device, dequantise and un-rotate the
//! quantised linears, apply the four key remaps, and report what resolved. The forward pass is
//! added in M2; the command-line surface keeps the shape the engine will call.

use std::io::{BufRead, BufWriter, Write};
use std::path::{Path, PathBuf};

use candle_core::Tensor;
use sam3_select::loader::{Pack, default_device};
use sam3_select::tokenizer::{ClipBpe, parse_prompts};
use serde::Deserialize;

fn usage() {
    println!("sam3-select (native SAM 3.1 helper for PhotoCraft)");
    println!();
    println!("  --tokens <text>          print the 32 token ids and the attention mask");
    println!("  --parse <text>           print the prompt categories and their detection caps");
    println!("  --verify-weights         open the pack, dequantise the ConvRot layers, print a report");
    println!("  --dump-dequant <out>     write the dequantised layers' statistics as JSON");
    println!("  --model <path>           the HSWQ ConvRot INT8 pack");
    println!("  --data <dir>             directory holding vocab.json and merges.txt");
    println!("                           (default: <exe folder>/data)");
    println!("  --detect-image <png> --detect-prompt <text> --detect-out <bin>   run the detector");
    println!("  --segment-image <png> --segment-box x1,y1,x2,y2 --segment-out <bin>");
    println!("                       --segment-point x,y    (one point, multimask on)");
    println!("                       --segment-mask <f32>   (1008x1008 coarse logits, refinement)");
    println!("  --serve                  resident JSON-lines worker mode on stdin/stdout");
}

const PACK_FILENAME: &str = "sam3.1_multiplex_hswq_convrot_int8.safetensors";

fn model_path(arg: Option<PathBuf>) -> Option<PathBuf> {
    if let Some(explicit) = arg {
        return Some(explicit);
    }
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?;
    for candidate in
        [dir.join(PACK_FILENAME), dir.join("sam3").join(PACK_FILENAME), dir.join("..").join(PACK_FILENAME), dir.join("..").join("sam3").join(PACK_FILENAME)]
    {
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    for ancestor in dir.ancestors() {
        let candidate = ancestor.join("plugins").join("sam3").join(PACK_FILENAME);
        if candidate.is_file() {
            return Some(candidate);
        }
        let direct = ancestor.join(PACK_FILENAME);
        if direct.is_file() {
            return Some(direct);
        }
    }
    Some(dir.join(PACK_FILENAME))
}

fn resolve_data_dir(explicit: Option<PathBuf>, model: Option<&Path>) -> PathBuf {
    if let Some(dir) = explicit {
        return dir;
    }
    if let Some(model_dir) = model.and_then(Path::parent) {
        for candidate in [model_dir.join("data"), model_dir.join("sam3").join("data")] {
            if candidate.join("vocab.json").is_file() {
                return candidate;
            }
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            for candidate in [dir.join("data"), dir.join("sam3").join("data"), dir.join("..").join("data"), dir.join("..").join("sam3").join("data")] {
                if candidate.join("vocab.json").is_file() {
                    return candidate;
                }
            }
            for ancestor in dir.ancestors() {
                let candidate = ancestor.join("plugins").join("sam3").join("data");
                if candidate.join("vocab.json").is_file() {
                    return candidate;
                }
            }
            return dir.join("data");
        }
    }
    PathBuf::from("data")
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut tokens: Option<String> = None;
    let mut parse: Option<String> = None;
    let mut data_dir: Option<PathBuf> = None;
    let mut model: Option<PathBuf> = None;
    let mut verify = false;
    let mut dump: Option<PathBuf> = None;
    let mut dump_layer: Option<String> = None;
    let mut dump_layer_out: Option<PathBuf> = None;
    let mut device_pref: Option<String> = None;
    let mut dump_hadamard: Option<(usize, PathBuf)> = None;
    let mut dump_deq_raw: Option<(String, PathBuf)> = None;
    let mut clip_check: Option<String> = None;
    let mut clip_out: Option<PathBuf> = None;
    let mut trunk_check: Option<PathBuf> = None;
    let mut trunk_out: Option<PathBuf> = None;
    let mut trunk_blocks: Option<usize> = None;
    let mut detect_prompt: Option<String> = None;
    let mut detect_out: Option<PathBuf> = None;
    let mut detect_image: Option<PathBuf> = None;
    let mut segment_image: Option<PathBuf> = None;
    let mut segment_box: Option<String> = None;
    let mut segment_point: Option<String> = None;
    let mut segment_mask: Option<PathBuf> = None;
    let mut segment_out: Option<PathBuf> = None;
    let mut select_image: Option<PathBuf> = None;
    let mut select_raw: Option<PathBuf> = None;
    let mut select_raw_size: Option<(usize, usize)> = None;
    let mut select_prompt: Option<String> = None;
    let mut select_box: Option<String> = None;
    let mut select_point: Option<String> = None;
    let mut select_coarse: Option<PathBuf> = None;
    let mut select_threshold: Option<f32> = None;
    let mut select_refine: Option<usize> = None;
    let mut select_individual = false;
    let mut select_out: Option<PathBuf> = None;
    let mut serve_mode = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--tokens" if i + 1 < args.len() => {
                tokens = Some(args[i + 1].clone());
                i += 1;
            }
            "--parse" if i + 1 < args.len() => {
                parse = Some(args[i + 1].clone());
                i += 1;
            }
            "--data" if i + 1 < args.len() => {
                data_dir = Some(PathBuf::from(&args[i + 1]));
                i += 1;
            }
            "--model" if i + 1 < args.len() => {
                model = Some(PathBuf::from(&args[i + 1]));
                i += 1;
            }
            "--dump-dequant" if i + 1 < args.len() => {
                dump = Some(PathBuf::from(&args[i + 1]));
                verify = true;
                i += 1;
            }
            "--dump-layer" if i + 1 < args.len() => {
                dump_layer = Some(args[i + 1].clone());
                verify = true;
                i += 1;
            }
            "--dump-layer-out" if i + 1 < args.len() => {
                dump_layer_out = Some(PathBuf::from(&args[i + 1]));
                i += 1;
            }
            "--device" if i + 1 < args.len() => {
                device_pref = Some(args[i + 1].clone());
                i += 1;
            }
            "--dump-deq-raw" if i + 2 < args.len() => {
                dump_deq_raw = Some((args[i + 1].clone(), PathBuf::from(&args[i + 2])));
                verify = true;
                i += 2;
            }
            "--dump-hadamard" if i + 2 < args.len() => {
                dump_hadamard = Some((args[i + 1].parse()?, PathBuf::from(&args[i + 2])));
                verify = true;
                i += 2;
            }
            "--clip-check" if i + 1 < args.len() => {
                clip_check = Some(args[i + 1].clone());
                verify = true;
                i += 1;
            }
            "--clip-out" if i + 1 < args.len() => {
                clip_out = Some(PathBuf::from(&args[i + 1]));
                i += 1;
            }
            "--trunk-check" if i + 1 < args.len() => {
                trunk_check = Some(PathBuf::from(&args[i + 1]));
                verify = true;
                i += 1;
            }
            "--trunk-out" if i + 1 < args.len() => {
                trunk_out = Some(PathBuf::from(&args[i + 1]));
                i += 1;
            }
            "--trunk-blocks" if i + 1 < args.len() => {
                trunk_blocks = Some(args[i + 1].parse()?);
                i += 1;
            }
            "--detect-image" if i + 1 < args.len() => {
                detect_image = Some(PathBuf::from(&args[i + 1]));
                verify = true;
                i += 1;
            }
            "--detect-prompt" if i + 1 < args.len() => {
                detect_prompt = Some(args[i + 1].clone());
                verify = true;
                i += 1;
            }
            "--detect-out" if i + 1 < args.len() => {
                detect_out = Some(PathBuf::from(&args[i + 1]));
                i += 1;
            }
            "--segment-image" if i + 1 < args.len() => {
                segment_image = Some(PathBuf::from(&args[i + 1]));
                verify = true;
                i += 1;
            }
            "--segment-box" if i + 1 < args.len() => {
                segment_box = Some(args[i + 1].clone());
                i += 1;
            }
            "--segment-point" if i + 1 < args.len() => {
                segment_point = Some(args[i + 1].clone());
                i += 1;
            }
            "--segment-mask" if i + 1 < args.len() => {
                segment_mask = Some(PathBuf::from(&args[i + 1]));
                i += 1;
            }
            "--segment-out" if i + 1 < args.len() => {
                segment_out = Some(PathBuf::from(&args[i + 1]));
                i += 1;
            }
            "--select-image" if i + 1 < args.len() => {
                select_image = Some(PathBuf::from(&args[i + 1]));
                verify = true;
                i += 1;
            }
            "--select-raw" if i + 1 < args.len() => {
                select_raw = Some(PathBuf::from(&args[i + 1]));
                verify = true;
                i += 1;
            }
            "--select-raw-size" if i + 1 < args.len() => {
                let parts: Vec<usize> = args[i + 1].split('x').filter_map(|v| v.parse().ok()).collect();
                if parts.len() != 2 {
                    return Err("--select-raw-size needs <width>x<height>".into());
                }
                select_raw_size = Some((parts[0], parts[1]));
                i += 1;
            }
            "--select-prompt" if i + 1 < args.len() => {
                select_prompt = Some(args[i + 1].clone());
                i += 1;
            }
            "--select-box" if i + 1 < args.len() => {
                select_box = Some(args[i + 1].clone());
                i += 1;
            }
            "--select-point" if i + 1 < args.len() => {
                select_point = Some(args[i + 1].clone());
                i += 1;
            }
            "--select-coarse" if i + 1 < args.len() => {
                select_coarse = Some(PathBuf::from(&args[i + 1]));
                i += 1;
            }
            "--select-threshold" if i + 1 < args.len() => {
                select_threshold = Some(args[i + 1].parse()?);
                i += 1;
            }
            "--select-refine" if i + 1 < args.len() => {
                select_refine = Some(args[i + 1].parse()?);
                i += 1;
            }
            "--select-individual" => select_individual = true,
            "--select-out" if i + 1 < args.len() => {
                select_out = Some(PathBuf::from(&args[i + 1]));
                i += 1;
            }
            "--verify-weights" => verify = true,
            "--serve" => serve_mode = true,
            "--help" | "-h" => {
                usage();
                return Ok(());
            }
            other => {
                return Err(format!("unknown argument `{other}`").into());
            }
        }
        i += 1;
    }

    if serve_mode {
        return serve(model, data_dir, device_pref);
    }

    if let Some(text) = parse {
        for category in parse_prompts(&text) {
            println!("{}\t{}", category.text, category.max_detections);
        }
        return Ok(());
    }

    if verify {
        return verify_weights(
            model,
            dump,
            dump_layer,
            dump_layer_out,
            device_pref,
            dump_hadamard,
            dump_deq_raw,
            clip_check,
            clip_out,
            data_dir,
            trunk_check,
            trunk_out,
            trunk_blocks,
            detect_image,
            detect_prompt,
            detect_out,
            segment_image,
            segment_box,
            segment_point,
            segment_mask,
            segment_out,
            select_image,
            select_raw,
            select_raw_size,
            select_prompt,
            select_box,
            select_point,
            select_coarse,
            select_threshold,
            select_refine,
            select_individual,
            select_out,
        );
    }

    if let Some(text) = tokens {
        let data_dir = resolve_data_dir(data_dir, model.as_deref());
        let mut bpe = ClipBpe::load(&data_dir)?;
        let encoded = bpe.encode(&text);
        println!("{:?}", encoded.ids);
        println!("{:?}", encoded.attention_mask);
        return Ok(());
    }

    usage();
    Ok(())
}

fn verify_weights(
    model: Option<PathBuf>,
    dump: Option<PathBuf>,
    dump_layer: Option<String>,
    dump_layer_out: Option<PathBuf>,
    device_pref: Option<String>,
    dump_hadamard: Option<(usize, PathBuf)>,
    dump_deq_raw: Option<(String, PathBuf)>,
    clip_check: Option<String>,
    clip_out: Option<PathBuf>,
    data_dir: Option<PathBuf>,
    trunk_check: Option<PathBuf>,
    trunk_out: Option<PathBuf>,
    trunk_blocks: Option<usize>,
    detect_image: Option<PathBuf>,
    detect_prompt: Option<String>,
    detect_out: Option<PathBuf>,
    segment_image: Option<PathBuf>,
    segment_box: Option<String>,
    segment_point: Option<String>,
    segment_mask: Option<PathBuf>,
    segment_out: Option<PathBuf>,
    select_image: Option<PathBuf>,
    select_raw: Option<PathBuf>,
    select_raw_size: Option<(usize, usize)>,
    select_prompt: Option<String>,
    select_box: Option<String>,
    select_point: Option<String>,
    select_coarse: Option<PathBuf>,
    select_threshold: Option<f32>,
    select_refine: Option<usize>,
    select_individual: bool,
    select_out: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some((size, out)) = dump_hadamard {
        let h = sam3_select::loader::build_hadamard(size)?;
        let mut bytes = Vec::with_capacity(h.len() * 4);
        for value in &h {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        std::fs::write(&out, &bytes)?;
        println!("wrote {} (H_{} , {} values)", out.display(), size, h.len());
        return Ok(());
    }
    let path = model_path(model).ok_or("--model is required (or place the pack beside the executable)")?;
    if !path.exists() {
        return Err(format!("pack not found: {}", path.display()).into());
    }

    if let Some((layer, out)) = dump_deq_raw {
        let (device, _) = match device_pref.as_deref() {
            Some("cpu") => (candle_core::Device::Cpu, false),
            Some("cuda") => (candle_core::Device::new_cuda(0)?, true),
            _ => default_device(),
        };
        let pack = Pack::open(&path, device)?;
        let values = if std::env::var("SAM3_DUMP_STAGE").as_deref() == Ok("pre-f16") { pack.debug_unrotated_f32(&layer)? } else { pack.debug_scaled(&layer)? };
        let mut bytes = Vec::with_capacity(values.len() * 4);
        for value in &values {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        std::fs::write(&out, &bytes)?;
        println!("wrote {} ({} f32 values of q*scale) for {}", out.display(), values.len(), layer);
        return Ok(());
    }

    let started = std::time::Instant::now();
    let (device, is_cuda) = match device_pref.as_deref() {
        Some("cpu") => (candle_core::Device::Cpu, false),
        Some("cuda") => (candle_core::Device::new_cuda(0)?, true),
        _ => default_device(),
    };
    println!("device: {:?} (cuda: {is_cuda})", device);
    println!("pack: {} ({:.3} GB)", path.display(), std::fs::metadata(&path)?.len() as f64 / 1e9);

    let pack = Pack::open(&path, device.clone())?;
    if let Some(prompt) = clip_check {
        use sam3_select::clip::ClipTextEncoder;
        use sam3_select::tokenizer::ClipBpe;
        let dir = resolve_data_dir(data_dir, Some(&path));
        let mut bpe = ClipBpe::load(&dir)?;
        let encoded = bpe.tokens_for_prompt(&prompt);
        println!("prompt {prompt:?} -> {} ids, {} active", encoded.ids.len(), encoded.attention_mask.iter().filter(|m| **m == 1).count());
        let encoder = ClipTextEncoder::load(&pack)?;
        let t0 = std::time::Instant::now();
        let hidden = encoder.hidden_states(&encoded.ids, &encoded.attention_mask)?;
        let hidden_cpu = hidden.to_dtype(candle_core::DType::F32)?.flatten_all()?.to_vec1::<f32>()?;
        let features = encoder.forward(&encoded.ids, &encoded.attention_mask)?;
        let projected_cpu = features.projected.to_dtype(candle_core::DType::F32)?.flatten_all()?.to_vec1::<f32>()?;
        println!("clip forward: hidden {:?} projected {:?} in {:.2?}", hidden.dims(), features.projected.dims(), t0.elapsed());
        if let Some(out) = clip_out {
            let mut bytes = Vec::with_capacity(hidden_cpu.len() * 4 + projected_cpu.len() * 4);
            for v in &hidden_cpu {
                bytes.extend_from_slice(&v.to_le_bytes());
            }
            for v in &projected_cpu {
                bytes.extend_from_slice(&v.to_le_bytes());
            }
            std::fs::write(&out, &bytes)?;
            println!("wrote {} ({} hidden + {} projected f32)", out.display(), hidden_cpu.len(), projected_cpu.len());
            // Side-car: the ids and mask the run used, so the oracle can feed exactly the same input.
            let meta = serde_json::json!({
                "prompt": prompt,
                "ids": encoded.ids,
                "attention_mask": encoded.attention_mask,
                "hidden_shape": hidden.dims(),
                "projected_shape": features.projected.dims(),
            });
            std::fs::write(out.with_extension("json"), serde_json::to_vec_pretty(&meta)?)?;
        }
        println!("hidden[0..6] = {:?}", &hidden_cpu[..6.min(hidden_cpu.len())]);
        return Ok(());
    }

    if let (Some(image_path), Some(prompt)) = (detect_image, detect_prompt) {
        use sam3_select::clip::ClipTextEncoder;
        use sam3_select::detector::{Detector, cxcywh_to_xyxy};
        use sam3_select::tokenizer::ClipBpe;
        use sam3_select::vitdet::VisionBackbone;
        let dir = resolve_data_dir(data_dir.clone(), Some(&path));
        let mut bpe = ClipBpe::load(&dir)?;
        let encoded = bpe.tokens_for_prompt(&prompt);
        println!("prompt {prompt:?} -> {} active tokens", encoded.attention_mask.iter().filter(|m| **m == 1).count());

        let t0 = std::time::Instant::now();
        let clip = ClipTextEncoder::load(&pack)?;
        let text = clip.forward(&encoded.ids, &encoded.attention_mask)?;
        println!("text features {:?} in {:.2?}", text.projected.dims(), t0.elapsed());

        let image = load_image_1008(&image_path)?.to_device(&device)?;
        let backbone = VisionBackbone::load(&pack, &device)?;
        let t1 = std::time::Instant::now();
        let feats = backbone.forward(&image)?;
        println!("backbone in {:.2?}: convs {:?}", t1.elapsed(), feats.convs.iter().map(|t| t.dims().to_vec()).collect::<Vec<_>>());

        let detector = Detector::load(&pack)?;
        let t2 = std::time::Instant::now();
        let det = detector.detect(&feats.convs, &text.projected, &text.padding_mask)?;
        println!("detector in {:.2?}", t2.elapsed());

        let boxes_xyxy = cxcywh_to_xyxy(&det.boxes)?;
        let scores = candle_nn::ops::sigmoid(&det.scores)?;
        let score_values: Vec<f32> = scores.flatten_all()?.to_dtype(candle_core::DType::F32)?.to_vec1()?;
        let box_values: Vec<f32> = boxes_xyxy.flatten_all()?.to_dtype(candle_core::DType::F32)?.to_vec1()?;

        let mut ranked: Vec<(usize, f32)> = score_values.iter().enumerate().map(|(i, s)| (i, *s)).collect();
        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        println!("top scores:");
        for (rank, (index, score)) in ranked.iter().take(5).enumerate() {
            let b = &box_values[index * 4..index * 4 + 4];
            println!("  #{rank} query {index}: score {score:.6}  box [{:.4}, {:.4}, {:.4}, {:.4}]", b[0], b[1], b[2], b[3]);
        }
        let above = score_values.iter().filter(|s| **s >= 0.5).count();
        println!("queries above 0.5: {above}");

        if let Some(out) = detect_out {
            let mut bytes = Vec::new();
            let mut meta = Vec::new();
            for (name, tensor) in [
                ("boxes", boxes_xyxy.clone()),
                ("scores_raw", det.scores.clone()),
                ("queries", det.queries.clone()),
                ("memory", det.memory.clone()),
                ("prompt", det.prompt.clone()),
            ] {
                let values = tensor.flatten_all()?.to_dtype(candle_core::DType::F32)?.to_vec1::<f32>()?;
                meta.push(serde_json::json!({"name": name, "shape": tensor.dims(), "offset": bytes.len() / 4, "count": values.len()}));
                for v in &values {
                    bytes.extend_from_slice(&v.to_le_bytes());
                }
            }
            // The coarse masks are large; keep them separate so the JSON side-car stays readable.
            let mask_values = det.masks.flatten_all()?.to_dtype(candle_core::DType::F32)?.to_vec1::<f32>()?;
            let mask_path = out.with_extension("masks.f32");
            let mut mb = Vec::with_capacity(mask_values.len() * 4);
            for v in &mask_values {
                mb.extend_from_slice(&v.to_le_bytes());
            }
            std::fs::write(&mask_path, &mb)?;
            std::fs::write(&out, &bytes)?;
            meta.push(serde_json::json!({"name": "masks", "shape": det.masks.dims(), "file": mask_path.file_name().and_then(|s| s.to_str())}));
            std::fs::write(out.with_extension("json"), serde_json::to_vec_pretty(&meta)?)?;
            println!("wrote {} and {}", out.display(), mask_path.display());
        }
        return Ok(());
    }

    if let Some(image_path) = select_image.or_else(|| select_raw.clone()) {
        use sam3_select::selection::{DEFAULT_REFINE, DEFAULT_THRESHOLD, Prompt, SelectionStack, detect_and_refine, refine_from_dump, segment_prompt};

        // `--select-raw` takes interleaved 8-bit RGB, which is what the PhotoCraft engine hands over;
        // `--select-image` is the PNG path used by the reference comparisons.
        let (native, frame) = if let Some(raw_path) = select_raw.clone() {
            let (width, height) = select_raw_size.ok_or("--select-raw-size <width>x<height> is required with --select-raw")?;
            let native = load_raw_rgb(&raw_path, width, height)?.to_device(&device)?;
            let frame = sam3_select::nn::interpolate_bilinear(&native, 1008, 1008)?.to_dtype(candle_core::DType::F16)?;
            (native, frame)
        } else {
            (load_image_native(&image_path)?.to_device(&device)?, load_image_1008(&image_path)?.to_device(&device)?)
        };
        let (h, w) = (native.dim(2)?, native.dim(3)?);
        println!("image {image_path:?} -> {w}x{h} native");

        let dir = resolve_data_dir(data_dir.clone(), Some(&path));
        let t0 = std::time::Instant::now();
        let mut stack = SelectionStack::load(&pack, &dir, &device)?;
        println!("stack loaded in {:.2?}", t0.elapsed());

        let threshold = select_threshold.unwrap_or(DEFAULT_THRESHOLD);
        let refine = select_refine.unwrap_or(DEFAULT_REFINE);
        println!("threshold {threshold} refine_iterations {refine} individual {select_individual}");

        let t1 = std::time::Instant::now();
        let selection = if let Some(dump) = select_coarse {
            println!("coarse from {dump:?}");
            refine_from_dump(&stack, &dump, &native, threshold, refine, (h, w))?
        } else if let Some(text) = select_prompt {
            println!("prompt {text:?}");
            detect_and_refine(&mut stack, &frame, &native, &text, threshold, refine, (h, w))?
        } else if let Some(box_arg) = select_box {
            let values: Vec<f32> = box_arg.split(',').map(|part| part.trim().parse::<f32>()).collect::<std::result::Result<Vec<f32>, _>>()?;
            if values.len() != 4 {
                return Err("--select-box needs x1,y1,x2,y2 in 1008 space".into());
            }
            let mask = segment_prompt(&mut stack, &frame, Prompt::Box([values[0], values[1], values[2], values[3]]), refine, (h, w))?;
            sam3_select::selection::Selection { masks: vec![mask], boxes: vec![values[0..4].try_into()?], scores: vec![1.0] }
        } else if let Some(point_arg) = select_point {
            let values: Vec<f32> = point_arg.split(',').map(|part| part.trim().parse::<f32>()).collect::<std::result::Result<Vec<f32>, _>>()?;
            if values.len() != 2 {
                return Err("--select-point needs x,y in 1008 space".into());
            }
            let mask = segment_prompt(&mut stack, &frame, Prompt::Point([values[0], values[1]]), refine, (h, w))?;
            sam3_select::selection::Selection { masks: vec![mask], boxes: vec![[0.0; 4]], scores: vec![1.0] }
        } else {
            return Err("one of --select-prompt / --select-box / --select-point / --select-coarse is required".into());
        };
        println!("selection in {:.2?}: {} object(s)", t1.elapsed(), selection.masks.len());
        for (index, (bx, score)) in selection.boxes.iter().zip(selection.scores.iter()).enumerate() {
            println!("  #{index} score {score:.6} box [{:.1}, {:.1}, {:.1}, {:.1}]", bx[0], bx[1], bx[2], bx[3]);
        }

        let masks = selection.finish(select_individual)?;
        if let Some(out) = select_out {
            let mut bytes: Vec<u8> = Vec::new();
            let mut shapes: Vec<serde_json::Value> = Vec::new();
            let mut coverage: Vec<f64> = Vec::new();
            for mask in &masks {
                let values = mask.to_dtype(candle_core::DType::F32)?.flatten_all()?.to_vec1::<f32>()?;
                let positive = values.iter().filter(|v| **v > 0.0).count() as f64;
                coverage.push(positive / values.len() as f64);
                shapes.push(serde_json::json!(mask.dims()));
                for value in &values {
                    bytes.extend_from_slice(&value.to_le_bytes());
                }
            }
            std::fs::write(&out, &bytes)?;
            let meta = serde_json::json!({
                "masks": shapes,
                "coverage": coverage,
                "boxes": selection.boxes,
                "scores": selection.scores,
                "threshold": threshold,
                "refine_iterations": refine,
                "individual": select_individual,
            });
            std::fs::write(out.with_extension("json"), serde_json::to_vec_pretty(&meta)?)?;
            println!("wrote {} ({} f32, coverage {:?})", out.display(), bytes.len() / 4, coverage);
        }
        return Ok(());
    }

    if let Some(image_path) = segment_image {
        use sam3_select::tracker::Segmenter;
        use sam3_select::vitdet::VisionBackbone;
        let image = load_image_1008(&image_path)?.to_device(&device)?;
        let backbone = VisionBackbone::load(&pack, &device)?;
        let t0 = std::time::Instant::now();
        let feats = backbone.forward(&image)?;
        println!("backbone in {:.2?}", t0.elapsed());
        for (index, level) in feats.interactive_convs.iter().enumerate() {
            println!("  interactive_convs[{index}] {:?}", level.dims());
        }

        let segmenter = Segmenter::load(&pack)?;
        let t1 = std::time::Instant::now();
        let result = if let Some(box_arg) = segment_box {
            let values: Vec<f32> = box_arg.split(',').map(|part| part.trim().parse::<f32>()).collect::<std::result::Result<Vec<f32>, _>>()?;
            if values.len() != 4 {
                return Err("--segment-box needs x1,y1,x2,y2".into());
            }
            let corners = Tensor::from_vec(values, (1, 2, 2), &device)?.to_dtype(image.dtype())?;
            println!("box prompt {box_arg:?}");
            segmenter.segment_box(&feats.interactive_convs, &corners)?
        } else if let Some(point) = segment_point {
            let values: Vec<f32> = point.split(',').map(|part| part.trim().parse::<f32>()).collect::<std::result::Result<Vec<f32>, _>>()?;
            if values.len() != 2 {
                return Err("--segment-point needs x,y".into());
            }
            let coords = Tensor::from_vec(values, (1, 1, 2), &device)?.to_dtype(image.dtype())?;
            let labels = Tensor::from_vec(vec![1i64], (1, 1), &device)?;
            println!("point prompt {point:?}");
            segmenter.segment_points(&feats.interactive_convs, &coords, &labels)?
        } else if let Some(mask_path) = segment_mask {
            let raw = std::fs::read(&mask_path)?;
            let expected = 1008 * 1008 * 4;
            if raw.len() != expected {
                return Err(format!("--segment-mask must be {expected} bytes of f32 (1008x1008), got {}", raw.len()).into());
            }
            let values: Vec<f32> = raw.chunks_exact(4).map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect();
            let logits = Tensor::from_vec(values, (1, 1, 1008, 1008), &device)?.to_dtype(image.dtype())?;
            println!("mask prompt {mask_path:?}");
            segmenter.segment_mask(&feats.interactive_convs, &logits)?
        } else {
            return Err("one of --segment-box / --segment-point / --segment-mask is required".into());
        };
        println!("segment in {:.2?}: mask {:?} iou {:.6} object_score {:.6}", t1.elapsed(), result.mask.dims(), result.iou, result.object_score);

        if let Some(out) = segment_out {
            let values = result.mask.to_dtype(candle_core::DType::F32)?.flatten_all()?.to_vec1::<f32>()?;
            let mut bytes = Vec::with_capacity(values.len() * 4);
            for v in &values {
                bytes.extend_from_slice(&v.to_le_bytes());
            }
            std::fs::write(&out, &bytes)?;
            let count = values.len() as f32;
            let positive = values.iter().filter(|v| **v > 0.0).count() as f32;
            let meta = serde_json::json!({
                "mask_shape": result.mask.dims(),
                "iou": result.iou,
                "object_score": result.object_score,
                "mask_min": values.iter().cloned().fold(f32::INFINITY, f32::min),
                "mask_max": values.iter().cloned().fold(f32::NEG_INFINITY, f32::max),
                "mask_mean": values.iter().sum::<f32>() / count,
                "mask_positive_frac": positive / count,
            });
            std::fs::write(out.with_extension("json"), serde_json::to_vec_pretty(&meta)?)?;
            println!("wrote {} ({} f32)", out.display(), values.len());
        }
        return Ok(());
    }

    if let Some(image_path) = trunk_check {
        use sam3_select::vitdet::VisionBackbone;
        let image = load_image_1008(&image_path)?.to_device(&device)?;
        println!("image {image_path:?} -> {:?}", image.dims());
        let backbone = VisionBackbone::load(&pack, &device)?;
        let t0 = std::time::Instant::now();
        let out = match trunk_blocks {
            Some(limit) => {
                let trunk = backbone.trunk_limited(&image, limit)?;
                println!("trunk (first {limit} blocks) {:?}", trunk.dims());
                let mut bytes = Vec::new();
                let values = trunk.to_dtype(candle_core::DType::F32)?.flatten_all()?.to_vec1::<f32>()?;
                for v in &values {
                    bytes.extend_from_slice(&v.to_le_bytes());
                }
                if let Some(path) = &trunk_out {
                    std::fs::write(path, &bytes)?;
                    println!("wrote {}", path.display());
                }
                return Ok(());
            }
            None => backbone.forward(&image)?,
        };
        println!("trunk {:?} in {:.2?}", out.trunk.dims(), t0.elapsed());
        for (index, level) in out.convs.iter().enumerate() {
            println!("  convs[{index}] {:?}", level.dims());
        }
        for (index, level) in out.interactive_convs.iter().enumerate() {
            println!("  interactive_convs[{index}] {:?}", level.dims());
        }
        if let Some(path) = trunk_out {
            let mut bytes: Vec<u8> = Vec::new();
            let mut labelled: Vec<(String, Tensor)> = vec![("trunk".into(), out.trunk.clone())];
            for (i, level) in out.convs.iter().enumerate() {
                labelled.push((format!("convs.{i}"), level.clone()));
            }
            for (i, level) in out.interactive_convs.iter().enumerate() {
                labelled.push((format!("interactive_convs.{i}"), level.clone()));
            }
            let mut meta = Vec::new();
            for (name, tensor) in &labelled {
                let values = tensor.to_dtype(candle_core::DType::F32)?.flatten_all()?.to_vec1::<f32>()?;
                meta.push(serde_json::json!({"name": name, "shape": tensor.dims(), "offset": bytes.len() / 4, "count": values.len()}));
                for v in &values {
                    bytes.extend_from_slice(&v.to_le_bytes());
                }
            }
            std::fs::write(&path, &bytes)?;
            std::fs::write(path.with_extension("json"), serde_json::to_vec_pretty(&meta)?)?;
            println!("wrote {} ({:.1} MB) with {} tensors", path.display(), bytes.len() as f64 / 1e6, labelled.len());
        }
        return Ok(());
    }

    println!("raw tensors: {}", pack.raw_count());
    println!("logical names: {}", pack.names().len());
    println!("convrot int8 weights: {}", pack.dequantised);
    println!("fused qkv tensors split: {}", pack.split);

    // Every quantised linear must resolve, and the count must be the 41 the plan measured.
    let dequant_names: Vec<String> = pack.names().iter().filter(|name| name.contains("detector.")).map(|name| name.to_string()).collect();
    println!("detector modules: {}", dequant_names.len());

    let mut stats: Vec<(String, Vec<f64>)> = Vec::new();
    let mut dequantised_layers = 0usize;
    for name in pack.names() {
        let name = name.to_string();
        // Only the tensors that came from INT8 are dequantised here; the rest is uploaded as-is.
        if !is_convrot_source(&pack, &name) {
            continue;
        }
        let tensor = pack.tensor(&name)?;
        let halves: Vec<half::f16> = tensor.flatten_all()?.to_vec1()?;
        // The exact f16 bit pattern sum: a single ulp of difference anywhere changes it, so it is
        // the sharpest comparison against the reference dequantiser.
        let bitsum: u64 = halves.iter().map(|v| v.to_bits() as u64).sum();
        let values: Vec<f32> = halves.iter().map(|v| v.to_f32()).collect();
        let sum: f64 = values.iter().map(|v| *v as f64).sum();
        let absmax = values.iter().fold(0.0f32, |acc, v| acc.max(v.abs())) as f64;
        let mean = sum / values.len() as f64;
        stats.push((name.clone(), vec![sum, absmax, mean, values.len() as f64, values[0] as f64, bitsum as f64]));
        dequantised_layers += 1;
        if dequantised_layers % 10 == 0 {
            println!("  dequantised {dequantised_layers} layers...");
        }
    }
    println!("dequantised layers verified: {dequantised_layers}");
    println!("elapsed: {:.2?}", started.elapsed());

    // A single layer can be written out for element-wise comparison. The f32 form isolates the
    // matmul from the f16 rounding so a difference can be attributed to one or the other.
    if let (Some(layer), Some(out)) = (dump_layer.as_ref(), dump_layer_out.as_ref()) {
        let tensor = pack.tensor(layer)?;
        let f16_form = out.extension().and_then(|e| e.to_str()) == Some("f16");
        let values: Vec<f32> = tensor.flatten_all()?.to_dtype(candle_core::DType::F32)?.to_vec1()?;
        let mut bytes = Vec::with_capacity(values.len() * 4);
        if f16_form {
            let halves: Vec<half::f16> = tensor.flatten_all()?.to_vec1()?;
            for value in &halves {
                bytes.extend_from_slice(&value.to_bits().to_le_bytes());
            }
        } else {
            for value in &values {
                bytes.extend_from_slice(&value.to_le_bytes());
            }
        }
        std::fs::write(out, &bytes)?;
        println!(
            "wrote {} ({} {}, {:.1} MB) for {}",
            out.display(),
            values.len(),
            if f16_form { "f16 values" } else { "f32 values" },
            bytes.len() as f64 / 1e6,
            layer
        );
    }

    if let Some(out) = dump {
        let json = serde_json::json!({
            "pack": path.file_name().and_then(|n| n.to_str()).unwrap_or(""),
            "device": format!("{device:?}"),
            "cuda": is_cuda,
            "raw_tensors": pack.raw_count(),
            "logical_names": pack.names().len(),
            "convrot_int8": pack.dequantised,
            "fused_qkv_split": pack.split,
            "dequantised_layers": dequantised_layers,
            "layers": stats.iter().map(|(name, s)| serde_json::json!({
                "name": name,
                "sum": s[0],
                "absmax": s[1],
                "mean": s[2],
                "elements": s[3],
                "first": s[4],
                "f16_bitsum": s[5] as u64,
            })).collect::<Vec<_>>(),
        });
        std::fs::write(&out, serde_json::to_vec_pretty(&json)?)?;
        println!("wrote {}", out.display());
    }
    Ok(())
}

/// True when the logical name resolves through a ConvRot INT8 source.
fn is_convrot_source(pack: &Pack, name: &str) -> bool {
    pack.is_convrot(name)
}

/// Loads an image as `[1, 3, 1008, 1008]` in `[0, 1]`, bilinear with `align_corners = False`.
fn load_raw_rgb(path: &std::path::Path, width: usize, height: usize) -> Result<Tensor, Box<dyn std::error::Error>> {
    let raw = std::fs::read(path)?;
    let expected = width * height * 3;
    if raw.len() != expected {
        return Err(format!("--select-raw must hold {expected} bytes of interleaved RGB ({width}x{height}), got {}", raw.len()).into());
    }
    let mut data = vec![0.0f32; 3 * height * width];
    for index in 0..width * height {
        for c in 0..3 {
            data[c * height * width + index] = raw[index * 3 + c] as f32 / 255.0;
        }
    }
    Ok(Tensor::from_vec(data, (1, 3, height, width), &candle_core::Device::Cpu)?)
}

fn load_image_native(path: &std::path::Path) -> Result<Tensor, Box<dyn std::error::Error>> {
    let img = image::open(path)?.to_rgb8();
    let (width, height) = (img.width() as usize, img.height() as usize);
    let mut data = vec![0.0f32; 3 * height * width];
    for y in 0..height {
        for x in 0..width {
            let pixel = img.get_pixel(x as u32, y as u32);
            for c in 0..3 {
                data[c * height * width + y * width + x] = pixel[c] as f32 / 255.0;
            }
        }
    }
    Ok(Tensor::from_vec(data, (1, 3, height, width), &candle_core::Device::Cpu)?)
}

fn load_image_1008(path: &std::path::Path) -> Result<Tensor, Box<dyn std::error::Error>> {
    let img = image::open(path)?.to_rgb8();
    let (src_w, src_h) = (img.width() as usize, img.height() as usize);
    let (dst_w, dst_h) = (1008usize, 1008usize);
    let mut data = vec![0.0f32; 3 * dst_h * dst_w];
    let scale_x = src_w as f64 / dst_w as f64;
    let scale_y = src_h as f64 / dst_h as f64;
    for oy in 0..dst_h {
        let sy = ((oy as f64 + 0.5) * scale_y - 0.5).max(0.0);
        let y0 = (sy.floor() as usize).min(src_h - 1);
        let y1 = (y0 + 1).min(src_h - 1);
        let wy = sy - y0 as f64;
        for ox in 0..dst_w {
            let sx = ((ox as f64 + 0.5) * scale_x - 0.5).max(0.0);
            let x0 = (sx.floor() as usize).min(src_w - 1);
            let x1 = (x0 + 1).min(src_w - 1);
            let wx = sx - x0 as f64;
            for c in 0..3 {
                let p00 = img.get_pixel(x0 as u32, y0 as u32)[c] as f64 / 255.0;
                let p10 = img.get_pixel(x1 as u32, y0 as u32)[c] as f64 / 255.0;
                let p01 = img.get_pixel(x0 as u32, y1 as u32)[c] as f64 / 255.0;
                let p11 = img.get_pixel(x1 as u32, y1 as u32)[c] as f64 / 255.0;
                let top = p00 * (1.0 - wx) + p10 * wx;
                let bottom = p01 * (1.0 - wx) + p11 * wx;
                let value = top * (1.0 - wy) + bottom * wy;
                data[c * dst_h * dst_w + oy * dst_w + ox] = value as f32;
            }
        }
    }
    Ok(Tensor::from_vec(data, (1, 3, dst_h, dst_w), &candle_core::Device::Cpu)?.to_dtype(candle_core::DType::F16)?)
}

fn default_serve_threshold() -> f32 {
    sam3_select::selection::DEFAULT_THRESHOLD
}

fn default_serve_refine() -> usize {
    sam3_select::selection::DEFAULT_REFINE
}

/// One JSON request line in `--serve` mode.
#[derive(Debug, Deserialize)]
struct ServeRequest {
    #[serde(default)]
    warmup: bool,
    #[serde(default)]
    raw: String,
    #[serde(default)]
    width: usize,
    #[serde(default)]
    height: usize,
    #[serde(default)]
    prompt: Option<String>,
    #[serde(default)]
    box_xyxy: Option<[f32; 4]>,
    #[serde(default)]
    point_xy: Option<[f32; 2]>,
    #[serde(default = "default_serve_threshold")]
    threshold: f32,
    #[serde(default = "default_serve_refine")]
    refine: usize,
    #[serde(default)]
    individual: bool,
    #[serde(default)]
    out: String,
}

/// 64-bit FNV-1a digest of the raw interleaved RGB image and its dimensions.
fn fnv1a_hash(bytes: &[u8], width: usize, height: usize) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in (width as u64).to_le_bytes().into_iter().chain((height as u64).to_le_bytes()) {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    for &b in bytes {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

struct CachedFrame {
    hash: u64,
    width: usize,
    height: usize,
    native: Tensor,
    frame: Tensor,
}

fn serve_one(
    stack: &mut sam3_select::selection::SelectionStack,
    cached_frame: &mut Option<CachedFrame>,
    req: &ServeRequest,
) -> Result<(usize, f64), Box<dyn std::error::Error>> {
    use sam3_select::selection::{Prompt, Selection, detect_and_refine_cached, segment_prompt_cached};

    if req.warmup {
        return Ok((0, 0.0));
    }
    if req.raw.is_empty() || req.out.is_empty() || req.width == 0 || req.height == 0 {
        return Err("missing raw, out, width or height in --serve request".into());
    }
    let started = std::time::Instant::now();
    let raw_bytes = std::fs::read(&req.raw)?;
    let expected = req.width * req.height * 3;
    if raw_bytes.len() != expected {
        return Err(format!("raw image must hold {expected} bytes ({}x{}), got {}", req.width, req.height, raw_bytes.len()).into());
    }
    let hash = fnv1a_hash(&raw_bytes, req.width, req.height);
    let reuse = cached_frame.as_ref().is_some_and(|c| c.hash == hash && c.width == req.width && c.height == req.height);
    if !reuse {
        let (width, height) = (req.width, req.height);
        let mut data = vec![0.0f32; 3 * height * width];
        for index in 0..width * height {
            for c in 0..3 {
                data[c * height * width + index] = raw_bytes[index * 3 + c] as f32 / 255.0;
            }
        }
        let native = Tensor::from_vec(data, (1, 3, height, width), &stack.device)?;
        let frame = sam3_select::nn::interpolate_bilinear(&native, 1008, 1008)?.to_dtype(candle_core::DType::F16)?;
        *cached_frame = Some(CachedFrame { hash, width, height, native, frame });
    }
    let cached = cached_frame.as_ref().ok_or("cached frame missing")?;
    let (h, w) = (cached.height, cached.width);

    let selection = if let Some(text) = req.prompt.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        detect_and_refine_cached(stack, &cached.frame, Some(hash), &cached.native, text, req.threshold, req.refine, (h, w))?
    } else if let Some(corners) = req.box_xyxy {
        let mask = segment_prompt_cached(stack, &cached.frame, Some(hash), Prompt::Box(corners), req.refine, (h, w))?;
        Selection { masks: vec![mask], boxes: vec![corners], scores: vec![1.0] }
    } else if let Some(point) = req.point_xy {
        let mask = segment_prompt_cached(stack, &cached.frame, Some(hash), Prompt::Point(point), req.refine, (h, w))?;
        Selection { masks: vec![mask], boxes: vec![[0.0; 4]], scores: vec![1.0] }
    } else {
        return Err("request needs one of prompt, box_xyxy or point_xy".into());
    };

    let masks = selection.finish(req.individual)?;
    let out_path = Path::new(&req.out);
    if let Some(parent) = out_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    let mut bytes: Vec<u8> = Vec::new();
    let mut shapes: Vec<serde_json::Value> = Vec::new();
    let mut coverage: Vec<f64> = Vec::new();
    for mask in &masks {
        let values = mask.to_dtype(candle_core::DType::F32)?.flatten_all()?.to_vec1::<f32>()?;
        let positive = values.iter().filter(|v| **v > 0.0).count() as f64;
        coverage.push(positive / values.len() as f64);
        shapes.push(serde_json::json!(mask.dims()));
        for value in &values {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
    }
    std::fs::write(out_path, &bytes)?;
    let meta = serde_json::json!({
        "masks": shapes,
        "coverage": coverage,
        "boxes": selection.boxes,
        "scores": selection.scores,
        "threshold": req.threshold,
        "refine_iterations": req.refine,
        "individual": req.individual,
    });
    std::fs::write(out_path.with_extension("json"), serde_json::to_vec_pretty(&meta)?)?;
    Ok((masks.len(), started.elapsed().as_secs_f64()))
}

/// Resident JSON-lines worker mode: loads the pack and selection stack once on the target device,
/// then answers one JSON line on `stdin` with one JSON line on `stdout`. All diagnostics go to
/// `stderr`.
fn serve(model: Option<PathBuf>, data_dir: Option<PathBuf>, device_pref: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    use sam3_select::selection::SelectionStack;

    let path = model_path(model).ok_or("--model is required (or place the pack beside the executable)")?;
    if !path.exists() {
        return Err(format!("pack not found: {}", path.display()).into());
    }
    let dir = resolve_data_dir(data_dir, Some(&path));
    let (device, is_cuda) = match device_pref.as_deref() {
        Some("cpu") => (candle_core::Device::Cpu, false),
        Some("cuda") => (candle_core::Device::new_cuda(0)?, true),
        _ => default_device(),
    };
    let t0 = std::time::Instant::now();
    let pack = Pack::open(&path, device.clone())?;
    let mut stack = SelectionStack::load(&pack, &dir, &device)?;
    eprintln!("sam3-select: resident worker ready on {:?} (cuda: {is_cuda}) in {:.2?}", device, t0.elapsed());

    let mut cached_frame: Option<CachedFrame> = None;
    let stdin = std::io::stdin();
    let mut out = BufWriter::new(std::io::stdout());
    for line in stdin.lock().lines() {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let reply = match serde_json::from_str::<ServeRequest>(trimmed) {
            Ok(req) => match serve_one(&mut stack, &mut cached_frame, &req) {
                Ok((objects, seconds)) => serde_json::json!({
                    "ok": true,
                    "objects": objects,
                    "infer_ms": seconds * 1000.0,
                    "cuda": is_cuda,
                })
                .to_string(),
                Err(err) => serde_json::json!({
                    "ok": false,
                    "error": err.to_string(),
                })
                .to_string(),
            },
            Err(err) => serde_json::json!({
                "ok": false,
                "error": format!("bad request: {err}"),
            })
            .to_string(),
        };
        writeln!(out, "{reply}")?;
        out.flush()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serve_request_parses_point_box_and_prompt() {
        let pt: ServeRequest =
            serde_json::from_str(r#"{"raw":"in.rgb","width":64,"height":48,"point_xy":[32.0,24.0],"out":"out.bin"}"#).expect("point request");
        assert_eq!(pt.width, 64);
        assert_eq!(pt.height, 48);
        assert_eq!(pt.point_xy, Some([32.0, 24.0]));
        assert_eq!(pt.refine, sam3_select::selection::DEFAULT_REFINE);
        assert!((pt.threshold - sam3_select::selection::DEFAULT_THRESHOLD).abs() < 1e-6);

        let bx: ServeRequest =
            serde_json::from_str(r#"{"raw":"in.rgb","width":64,"height":48,"box_xyxy":[1.0,2.0,30.0,40.0],"refine":1,"out":"out.bin"}"#).expect("box request");
        assert_eq!(bx.box_xyxy, Some([1.0, 2.0, 30.0, 40.0]));
        assert_eq!(bx.refine, 1);

        let pr: ServeRequest =
            serde_json::from_str(r#"{"raw":"in.rgb","width":64,"height":48,"prompt":"person","threshold":0.35,"individual":true,"out":"out.bin"}"#)
                .expect("prompt request");
        assert_eq!(pr.prompt.as_deref(), Some("person"));
        assert!((pr.threshold - 0.35).abs() < 1e-6);
        assert!(pr.individual);
    }

    #[test]
    fn fnv1a_hash_distinguishes_dimensions_and_pixels() {
        let a = fnv1a_hash(&[1, 2, 3, 4, 5, 6], 2, 1);
        let b = fnv1a_hash(&[1, 2, 3, 4, 5, 6], 1, 2);
        let c = fnv1a_hash(&[1, 2, 3, 4, 5, 7], 2, 1);
        assert_ne!(a, b);
        assert_ne!(a, c);
    }
}
