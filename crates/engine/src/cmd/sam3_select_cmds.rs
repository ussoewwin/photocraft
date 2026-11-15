//! `select.sam3`: the SAM 3.1 selection command.
//!
//! The model runs in the standalone `plugins/sam3` helper (`sam3-select`), which never links against
//! ComfyUI or Python. This module only moves pixels: it hands the helper interleaved 8-bit RGB, gets
//! `[N, H, W]` binary masks back, and turns them into a selection through
//! [`combine_region`]. The prompt semantics match `SAM3_Detect`
//! (`comfy_extras/nodes_sam3.py:116-255`): a text prompt detects and refines, a box segments inside
//! the box, a point segments from the click.

use std::path::{Path, PathBuf};
use std::process::Command;

use photocraft_algo::selection::{Region, SelectionMode, combine_region};
use photocraft_geom::Rect;
use serde_json::{Value, json};

use crate::commands::CommandSpec;
use crate::smartselect_cmds::with_doc_sampler;
use crate::{EngineError, Result, Session};

/// Set to the helper binary to bypass the search.
const HELPER_ENV: &str = "CRAFT_SAM3_HELPER";
/// Set to the HSWQ pack to bypass the search.
const PACK_ENV: &str = "CRAFT_SAM3_PACK";
/// The pack the helper ships with.
const PACK_NAME: &str = "sam3.1_multiplex_hswq_convrot_int8.safetensors";
/// The image the model is fed.
const SAM_SIZE: f32 = 1008.0;

fn bad(cmd: &str, msg: &str) -> EngineError {
    EngineError::BadParams { cmd: cmd.into(), msg: msg.into() }
}

fn helper_name() -> String {
    format!("sam3-select{}", std::env::consts::EXE_SUFFIX)
}

/// Every place the helper is looked for, in order. A missing binary names all of them.
fn helper_candidates() -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = Vec::new();
    let exe_dir = std::env::current_exe().ok().and_then(|e| e.parent().map(Path::to_path_buf));
    if let Some(dir) = &exe_dir {
        out.push(dir.join(helper_name()));
    }
    let mut roots: Vec<PathBuf> = Vec::new();
    if let Some(dir) = &exe_dir {
        roots.extend(dir.ancestors().map(Path::to_path_buf));
    }
    if let Ok(cwd) = std::env::current_dir() {
        roots.extend(cwd.ancestors().map(Path::to_path_buf));
    }
    for root in roots {
        out.push(root.join("plugins").join("sam3").join("target").join("release").join(helper_name()));
    }
    out.dedup();
    out
}

/// The helper binary: `CRAFT_SAM3_HELPER`, else the first candidate that exists.
fn resolve_helper() -> Result<PathBuf> {
    if let Some(path) = std::env::var_os(HELPER_ENV) {
        let path = PathBuf::from(path);
        return if path.is_file() { Ok(path) } else { Err(bad("select.sam3", &format!("{HELPER_ENV} points at {}, which is not a file", path.display()))) };
    }
    let candidates = helper_candidates();
    if let Some(found) = candidates.iter().find(|path| path.is_file()) {
        return Ok(found.clone());
    }
    let searched: Vec<String> = candidates.iter().map(|p| p.display().to_string()).collect();
    Err(bad(
        "select.sam3",
        &format!("the SAM 3.1 helper was not found. Build it with `cargo build --release` in plugins/sam3, or set {HELPER_ENV}. Searched: {}", searched.join(", ")),
    ))
}

/// The HSWQ pack: `CRAFT_SAM3_PACK`, else beside the helper, else `plugins/sam3/`.
fn resolve_pack(helper: &Path) -> Result<PathBuf> {
    if let Some(path) = std::env::var_os(PACK_ENV) {
        let path = PathBuf::from(path);
        return if path.is_file() { Ok(path) } else { Err(bad("select.sam3", &format!("{PACK_ENV} points at {}, which is not a file", path.display()))) };
    }
    let mut searched: Vec<PathBuf> = Vec::new();
    if let Some(dir) = helper.parent() {
        searched.push(dir.join(PACK_NAME));
    }
    for root in helper.ancestors().filter(|p| p.file_name().is_some_and(|n| n == "sam3")) {
        searched.push(root.join(PACK_NAME));
    }
    if let Some(found) = searched.iter().find(|path| path.is_file()) {
        return Ok(found.clone());
    }
    let listed: Vec<String> = searched.iter().map(|p| p.display().to_string()).collect();
    Err(bad(
        "select.sam3",
        &format!("{PACK_NAME} was not found. Put it beside the helper or in plugins/sam3, or set {PACK_ENV}. Searched: {}", listed.join(", ")),
    ))
}

/// One mask as the helper returned it.
struct HelperMask {
    /// Coverage per document pixel, 0 or 255.
    bytes: Vec<u8>,
    /// The covered pixels' bounding box.
    bbox: Rect,
}

impl HelperMask {
    /// Turns a `[H, W]` 0/1 mask in document space into a [`Region`] payload, trimmed to its content.
    fn from_mask(mask: &[f32], width: usize, height: usize) -> Option<Self> {
        if mask.len() != width * height {
            return None;
        }
        let (mut x0, mut y0, mut x1, mut y1) = (usize::MAX, usize::MAX, 0usize, 0usize);
        for y in 0..height {
            for x in 0..width {
                if mask[y * width + x] > 0.0 {
                    x0 = x0.min(x);
                    y0 = y0.min(y);
                    x1 = x1.max(x);
                    y1 = y1.max(y);
                }
            }
        }
        if x0 == usize::MAX {
            return None;
        }
        let bbox = Rect::new(x0 as i32, y0 as i32, x1 as i32 + 1, y1 as i32 + 1);
        let (bw, bh) = (bbox.width() as usize, bbox.height() as usize);
        let mut bytes = vec![0u8; bw * bh];
        for y in 0..bh {
            for x in 0..bw {
                let src = (y0 + y) * width + (x0 + x);
                if mask[src] > 0.0 {
                    bytes[y * bw + x] = 255;
                }
            }
        }
        Some(Self { bytes, bbox })
    }

    fn region(&self) -> Region {
        Region { bbox: self.bbox, mask: self.bytes.clone() }
    }
}

/// The prompt the caller gave, in the form the helper takes.
enum SamPrompt {
    /// Text prompt: the helper detects, then refines.
    Text(String),
    /// Two corners in 1008 space.
    Box([f32; 4]),
    /// One point in 1008 space.
    Point([f32; 2]),
}

fn pairs(p: &Value, key: &str) -> Option<Vec<f32>> {
    Some(p.get(key)?.as_array()?.iter().map(|v| v.as_f64().unwrap_or(f64::NAN) as f32).collect())
}

fn prompt_from(s: &Session, p: &Value) -> Result<SamPrompt> {
    let doc = s.active().ok_or(EngineError::NoDocument)?;
    let bounds = doc.doc.bounds();
    let (w, h) = (bounds.width() as f32, bounds.height() as f32);
    if let Some(text) = p.get("prompt").and_then(Value::as_str).map(str::trim).filter(|t| !t.is_empty()) {
        return Ok(SamPrompt::Text(text.to_string()));
    }
    if let Some(box_pt) = pairs(p, "box") {
        if box_pt.len() != 4 || box_pt.iter().any(|v| !v.is_finite()) {
            return Err(bad("select.sam3", "\"box\" needs [x, y, width, height] in document pixels"));
        }
        let (x, y, bw, bh) = (box_pt[0], box_pt[1], box_pt[2], box_pt[3]);
        if bw <= 0.0 || bh <= 0.0 {
            return Err(bad("select.sam3", "\"box\" needs a positive width and height"));
        }
        let sx = SAM_SIZE / w.max(1.0);
        let sy = SAM_SIZE / h.max(1.0);
        return Ok(SamPrompt::Box([(x - bounds.x0 as f32) * sx, (y - bounds.y0 as f32) * sy, (x + bw - bounds.x0 as f32) * sx, (y + bh - bounds.y0 as f32) * sy]));
    }
    if let Some(point) = pairs(p, "point") {
        if point.len() != 2 || point.iter().any(|v| !v.is_finite()) {
            return Err(bad("select.sam3", "\"point\" needs [x, y] in document pixels"));
        }
        let sx = SAM_SIZE / w.max(1.0);
        let sy = SAM_SIZE / h.max(1.0);
        return Ok(SamPrompt::Point([(point[0] - bounds.x0 as f32) * sx, (point[1] - bounds.y0 as f32) * sy]));
    }
    Err(bad("select.sam3", "needs a \"prompt\", a \"box\" or a \"point\""))
}

/// Runs the helper and returns the masks it wrote.
fn run_helper(p: &Value, bounds: Rect, pixels: &[[f32; 4]], prompt: &SamPrompt) -> Result<Vec<HelperMask>> {
    let helper = resolve_helper()?;
    let pack = resolve_pack(&helper)?;
    let (w, h) = (bounds.width() as usize, bounds.height() as usize);
    if w == 0 || h == 0 {
        return Err(bad("select.sam3", "the document has no pixels"));
    }

    let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("photocraft-sam3-{}-{stamp}", std::process::id()));
    std::fs::create_dir_all(&dir).map_err(|e| bad("select.sam3", &format!("cannot create {}: {e}", dir.display())))?;
    let raw_in = dir.join("image.rgb");
    let out = dir.join("masks.bin");

    let mut rgb = Vec::with_capacity(w * h * 3);
    for px in pixels.iter().take(w * h) {
        for c in 0..3 {
            rgb.push((px[c].clamp(0.0, 1.0) * 255.0).round() as u8);
        }
    }
    std::fs::write(&raw_in, &rgb).map_err(|e| bad("select.sam3", &format!("cannot write {}: {e}", raw_in.display())))?;

    let threshold = p.get("threshold").and_then(Value::as_f64).unwrap_or(0.5).clamp(0.0, 1.0);
    let refine = p.get("refine").and_then(Value::as_u64).unwrap_or(2).min(5);
    let individual = p.get("individual").and_then(Value::as_bool).unwrap_or(false);
    let data_dir = pack.parent().map(|d| d.join("data")).filter(|d| d.is_dir());

    let mut cmd = Command::new(&helper);
    cmd.arg("--verify-weights").arg("--model").arg(&pack);
    if let Some(data) = &data_dir {
        cmd.arg("--data").arg(data);
    }
    cmd.arg("--select-raw").arg(&raw_in).arg("--select-raw-size").arg(format!("{w}x{h}"));
    match prompt {
        SamPrompt::Text(text) => {
            cmd.arg("--select-prompt").arg(text);
            cmd.arg("--select-threshold").arg(format!("{threshold}"));
        }
        SamPrompt::Box(corners) => {
            cmd.arg("--select-box").arg(corners.iter().map(|v| format!("{v}")).collect::<Vec<_>>().join(","));
        }
        SamPrompt::Point(point) => {
            cmd.arg("--select-point").arg(point.iter().map(|v| format!("{v}")).collect::<Vec<_>>().join(","));
        }
    }
    cmd.arg("--select-refine").arg(format!("{refine}"));
    if individual {
        cmd.arg("--select-individual");
    }
    cmd.arg("--select-out").arg(&out);

    let run = cmd.output().map_err(|e| bad("select.sam3", &format!("cannot start {}: {e}", helper.display())))?;
    if !run.status.success() {
        let message = String::from_utf8_lossy(&run.stderr).lines().rev().take(4).collect::<Vec<_>>().into_iter().rev().collect::<Vec<_>>().join(" | ");
        let _ = std::fs::remove_dir_all(&dir);
        return Err(bad("select.sam3", &format!("the SAM 3.1 helper failed: {message}")));
    }

    let meta: Value = std::fs::read(out.with_extension("json"))
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .ok_or_else(|| bad("select.sam3", "the SAM 3.1 helper wrote no result"))?;
    let raw = std::fs::read(&out).map_err(|e| bad("select.sam3", &format!("cannot read {}: {e}", out.display())))?;
    let values: Vec<f32> = raw.chunks_exact(4).map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect();
    let shapes: Vec<(usize, usize)> = meta["masks"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|shape| {
                    let dims = shape.as_array()?;
                    let h = dims.get(dims.len().checked_sub(2)?)?.as_u64()? as usize;
                    let w = dims.get(dims.len().checked_sub(1)?)?.as_u64()? as usize;
                    Some((h, w))
                })
                .collect()
        })
        .unwrap_or_default();

    let mut masks = Vec::new();
    let mut offset = 0usize;
    for (mh, mw) in shapes {
        let count = mh * mw;
        if offset + count > values.len() {
            break;
        }
        if let Some(mask) = HelperMask::from_mask(&values[offset..offset + count], mw, mh) {
            masks.push(mask);
        }
        offset += count;
    }
    let _ = std::fs::remove_dir_all(&dir);
    Ok(masks)
}

fn sam3_selection(s: &mut Session, p: &Value) -> Result<Value> {
    let prompt = prompt_from(s, p)?;
    let all_layers = p.get("sampleAllLayers").and_then(Value::as_bool).unwrap_or(false);
    let m = match p.get("mode").and_then(Value::as_str).unwrap_or("replace") {
        "add" => SelectionMode::Add,
        "subtract" => SelectionMode::Subtract,
        "intersect" => SelectionMode::Intersect,
        _ => SelectionMode::Replace,
    };

    let d = s.active().ok_or(EngineError::NoDocument)?;
    let doc = std::sync::Arc::clone(&d.doc);
    let bounds = doc.bounds();
    let pixels = with_doc_sampler(&doc, d.active_layer, all_layers, |smp, _| smp.rgba(bounds));
    let masks = run_helper(p, bounds, &pixels, &prompt)?;
    if masks.is_empty() {
        let selected = s.active().ok_or(EngineError::NoDocument)?.doc.selection.is_some();
        return Ok(json!({ "selected": selected, "changed": false, "objects": 0 }));
    }

    let regions: Vec<Region> = masks.iter().map(HelperMask::region).collect();
    let label = "SAM Selection";
    let selected = s.edit(label, |doc, _| {
        for region in &regions {
            doc.selection = combine_region(doc.selection.as_ref(), Some(region), m);
        }
        Ok(doc.selection.is_some())
    })?;
    let first = regions[0].bbox;
    Ok(json!({
        "selected": selected,
        "changed": true,
        "objects": regions.len(),
        "bounds": [first.x0, first.y0, first.width() as i32, first.height() as i32],
    }))
}

/// SAM 3.1 selection command specs.
pub fn specs() -> Vec<CommandSpec> {
    vec![CommandSpec {
        id: "select.sam3",
        label: "SAM Selection",
        menu: &[],
        shortcut: None,
        params: r##"{"prompt":"text" | "box":[x,y,width,height] | "point":[x,y],"threshold":0..1=0.5,"refine":0..5=2,"individual":bool=false,"sampleAllLayers":bool=false,"mode":"replace|add|subtract|intersect"="replace"} (prompt: open-vocabulary detection on the whole layer; point: the object under the click; box: the object inside the box)"##,
        enabled: |s: &Session| s.active().map(|_| ()).ok_or_else(|| "no document open".to_string()),
        run: sam3_selection,
        journal: true,
    }]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_mask_is_trimmed_to_its_content() {
        let mut mask = vec![0.0f32; 6 * 5];
        mask[2 * 6 + 1] = 1.0;
        mask[3 * 6 + 3] = 1.0;
        let trimmed = HelperMask::from_mask(&mask, 6, 5).expect("content");
        assert_eq!(trimmed.bbox, Rect::new(1, 2, 4, 4));
        assert_eq!(trimmed.bytes.len(), 3 * 2);
        assert_eq!(trimmed.bytes[0], 255);
        assert_eq!(trimmed.bytes[5], 255);
        assert_eq!(trimmed.bytes[1], 0);
    }

    #[test]
    fn an_empty_mask_finds_no_region() {
        assert!(HelperMask::from_mask(&vec![0.0f32; 12], 4, 3).is_none());
        assert!(HelperMask::from_mask(&[0.0, 1.0], 4, 3).is_none());
    }

    #[test]
    fn a_region_reports_its_coverage() {
        let mask = vec![255u8, 0, 0, 255];
        let region = Region { bbox: Rect::new(0, 0, 2, 2), mask };
        assert_eq!(region.at(0, 0), 1.0);
        assert_eq!(region.at(1, 0), 0.0);
        assert_eq!(region.at(5, 5), 0.0);
    }

    #[test]
    fn a_prompt_needs_a_target() {
        let mut s = Session::new();
        s.execute("file.new", json!({"width": 16, "height": 16, "depth": 8})).unwrap();
        assert!(prompt_from(&s, &json!({})).is_err());
        assert!(prompt_from(&s, &json!({"box": [0, 0]})).is_err());
        assert!(prompt_from(&s, &json!({"box": [0, 0, -1, 5]})).is_err());
        assert!(prompt_from(&s, &json!({"point": [1, 2, 3]})).is_err());
    }

    #[test]
    fn a_point_and_a_box_are_scaled_to_the_model_grid() {
        let mut s = Session::new();
        s.execute("file.new", json!({"width": 504, "height": 252, "depth": 8})).unwrap();
        match prompt_from(&s, &json!({"point": [252, 126]})).unwrap() {
            SamPrompt::Point(p) => {
                assert!((p[0] - 504.0).abs() < 1e-3, "{p:?}");
                assert!((p[1] - 504.0).abs() < 1e-3, "{p:?}");
            }
            _ => panic!("expected a point prompt"),
        }
        match prompt_from(&s, &json!({"box": [0, 0, 504, 252]})).unwrap() {
            SamPrompt::Box(c) => {
                assert_eq!(c, [0.0, 0.0, 1008.0, 1008.0]);
            }
            _ => panic!("expected a box prompt"),
        }
    }

    /// Runs the real helper. Ignored by default: it needs the built `sam3-select` and the 1.7 GB
    /// HSWQ pack, which are not part of the repository.
    #[test]
    #[ignore = "needs the built SAM 3.1 helper and its HSWQ pack (CRAFT_SAM3_HELPER / CRAFT_SAM3_PACK)"]
    fn a_point_selects_the_disc_under_it_end_to_end() {
        let mut s = Session::new();
        s.execute("file.new", json!({"width": 256, "height": 256, "depth": 8})).unwrap();
        s.edit("paint", |doc, _| {
            let bg = doc.layers[0].surface_mut().unwrap();
            let n = bg.channels();
            let mut data = Vec::with_capacity(256 * 256 * n);
            for y in 0..256i32 {
                for x in 0..256i32 {
                    let inside = (x - 128).pow(2) + (y - 128).pow(2) <= 40 * 40;
                    let c = if inside { [0.1f32, 0.7, 0.2] } else { [1.0, 1.0, 1.0] };
                    data.extend_from_slice(&c[..n.min(3)]);
                    if n == 4 {
                        data.push(1.0);
                    }
                }
            }
            bg.write_region(Rect::new(0, 0, 256, 256), &data);
            Ok(())
        })
        .unwrap();

        let out = s.execute("select.sam3", json!({"point": [128, 128], "refine": 1})).expect("select.sam3 runs");
        assert_eq!(out["changed"], json!(true));
        let bounds = s.active().expect("document").doc.selection.as_ref().expect("a selection").content_bounds();
        assert!(bounds.width() > 40 && bounds.width() < 160, "bounds {bounds:?}");
    }
}
