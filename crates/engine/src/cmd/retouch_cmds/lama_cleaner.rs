//! The LaMa Cleaner Tool as a command: brush over an object to remove it using LaMa AI inpainting.
//!
//! Takes a stroke (or strokes) drawn with the brush, computes the hole, crops the window
//! around it with context margin, feeds it to the native Pure Rust LaMa inference engine,
//! and blends the inpainted result back into the active layer in one undo step.

use super::content_aware_move::{Target, replace};
use super::patch::coverage;
use super::{
    Document, EngineError, LayerId, PixelFormat, Rect, Region, Result, SampleLayers, Session, Stroke, Surface, Value, alpha_index, bad, clamp_samples,
    composite_region, damage_json, flag, from_rgba_into, json, parse_brush, reject_huge_stroke, targets_pixels, to_rgba,
};
use photocraft_codecs::{ChannelLayout, Format, Image, SampleType};
use photocraft_doc::Layer;

const CMD: &str = "paint.lamaCleaner";

/// Cap on the sampling margin around the stroke.
const MAX_MARGIN: i32 = 512;

/// How far the hole grows beyond the stroke, in pixels (covers anti-aliased fringes around objects).
const GROW: usize = 5;

/// Stroke bounds before coverage rendering.
fn stroke_bounds(stroke: &Stroke) -> Rect {
    let r = f64::from(stroke.brush.size) / 2.0 + 2.0;
    let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
    for q in &stroke.points {
        (x0, y0, x1, y1) = (x0.min(q.x), y0.min(q.y), x1.max(q.x), y1.max(q.y));
    }
    Rect::new((x0 - r).floor() as i32, (y0 - r).floor() as i32, (x1 + r).ceil() as i32, (y1 + r).ceil() as i32)
}

fn parse_strokes(s: &Session, p: &Value) -> Result<(Vec<Stroke>, Option<LayerId>)> {
    let Some(list) = p.get("strokes") else {
        let (stroke, id) = parse_brush(s, p, CMD)?;
        reject_huge_stroke(CMD, &stroke)?;
        return Ok((vec![stroke], id));
    };
    const ENTRY: &str = "each `strokes` entry is a point list or {\"points\", \"size\"?, \"hardness\"?, \"spacing\"?}";
    let list = list.as_array().ok_or_else(|| bad(CMD, ENTRY))?;
    if list.is_empty() {
        return Err(bad(CMD, "`strokes` is empty"));
    }
    let mut strokes = Vec::with_capacity(list.len());
    let mut id = None;
    for entry in list {
        let mut one = p.clone();
        let Some(o) = one.as_object_mut() else { return Err(bad(CMD, "params must be an object")) };
        o.remove("strokes");
        match entry {
            Value::Object(e) => {
                let points = e.get("points").ok_or_else(|| bad(CMD, ENTRY))?;
                o.insert("points".into(), points.clone());
                for k in ["size", "hardness", "spacing"] {
                    if let Some(v) = e.get(k) {
                        o.insert(k.into(), v.clone());
                    }
                }
            }
            points => {
                o.insert("points".into(), points.clone());
            }
        }
        let (stroke, layer) = parse_brush(s, &one, CMD)?;
        reject_huge_stroke(CMD, &stroke)?;
        strokes.push(stroke);
        id = layer;
    }
    Ok((strokes, id))
}

struct Plan {
    id: Option<LayerId>,
    strokes: Vec<Stroke>,
    window: Rect,
    close_loops: bool,
    all_layers: bool,
    new_layer: bool,
}

fn plan(s: &Session, p: &Value) -> Result<Plan> {
    let (strokes, id) = parse_strokes(s, p)?;
    let d = s.active().ok_or(EngineError::NoDocument)?;
    let canvas = d.doc.bounds();
    let bounds = strokes.iter().map(stroke_bounds).reduce(|a, b| a.union(&b)).ok_or_else(|| bad(CMD, "`strokes` is empty"))?;
    let area = bounds.intersect(&canvas);
    if area.is_empty() {
        return Err(bad(CMD, "the stroke is outside the canvas"));
    }
    let margin = ((area.width().max(area.height()) as f32 * 0.5) as i32).clamp(128, MAX_MARGIN);
    let mut win = area.inflate(margin);
    // Ensure at least a 512x512 context window around the stroke so LaMa sees rich surrounding structure at 1:1 scale
    if win.width() < 512 {
        let pad = (512 - win.width() as i32) / 2;
        win = Rect::new(win.x0 - pad, win.y0, win.x0 - pad + 512, win.y1);
    }
    if win.height() < 512 {
        let pad = (512 - win.height() as i32) / 2;
        win = Rect::new(win.x0, win.y0 - pad, win.x1, win.y0 - pad + 512);
    }
    // Shift window inside canvas bounds before intersecting so edge strokes keep full 512px context
    if win.x0 < canvas.x0 {
        let shift = canvas.x0 - win.x0;
        win = Rect::new(win.x0 + shift, win.y0, win.x1 + shift, win.y1);
    }
    if win.x1 > canvas.x1 {
        let shift = win.x1 - canvas.x1;
        win = Rect::new(win.x0 - shift, win.y0, win.x1 - shift, win.y1);
    }
    if win.y0 < canvas.y0 {
        let shift = canvas.y0 - win.y0;
        win = Rect::new(win.x0, win.y0 + shift, win.x1, win.y1 + shift);
    }
    if win.y1 > canvas.y1 {
        let shift = win.y1 - canvas.y1;
        win = Rect::new(win.x0, win.y0 - shift, win.x1, win.y1 - shift);
    }
    let window = win.intersect(&canvas);
    let close_loops = flag(p, "closeLoops", false);
    let all_layers = flag(p, "sampleAllLayers", false) && targets_pixels(p);
    let new_layer = flag(p, "newLayer", false) && targets_pixels(p);
    Ok(Plan { id, strokes, window, close_loops, all_layers, new_layer })
}

fn hole(plan: &Plan, sel: Option<&Surface>) -> Result<Vec<bool>> {
    let win = plan.window;
    let (w, h) = (win.width() as usize, win.height() as usize);
    let mut m = vec![false; w * h];
    for stroke in &plan.strokes {
        let (bounds, cov) = super::coverage(CMD, stroke)?;
        let bw = bounds.width() as usize;
        for (i, c) in cov.iter().enumerate() {
            let (x, y) = (bounds.x0 + (i % bw.max(1)) as i32, bounds.y0 + (i / bw.max(1)) as i32);
            if *c > 0.0 && win.contains(x, y) {
                let j = (y - win.y0) as usize * w + (x - win.x0) as usize;
                if let Some(v) = m.get_mut(j) {
                    *v = true;
                }
            }
        }
    }
    if plan.close_loops {
        m = photocraft_algo::remove::close_loops(w, h, &m);
    }
    m = photocraft_algo::remove::dilate(w, h, &m, GROW);
    if let Some(sel) = sel {
        for (v, c) in m.iter_mut().zip(coverage(sel, win)) {
            *v &= c > 0.0;
        }
    }
    Ok(m)
}

/// The LaMa helper program and its model, resolved before any work starts.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug)]
struct LamaTool {
    exe: std::path::PathBuf,
    model: std::path::PathBuf,
}

/// The wasm build cannot run a helper process: the command reports that it needs the desktop app.
#[cfg(target_arch = "wasm32")]
struct LamaTool;

#[cfg(target_arch = "wasm32")]
impl LamaTool {
    fn resolve(_s: &Session, _p: &Value) -> Result<Self> {
        Err(EngineError::Other("LaMa Cleaner is only available in the desktop app".into()))
    }
}

/// The helper's file name on this platform.
#[cfg(not(target_arch = "wasm32"))]
const HELPER: &str = if cfg!(windows) { "lama-cleaner.exe" } else { "lama-cleaner" };

#[cfg(not(target_arch = "wasm32"))]
impl LamaTool {
    /// Where the helper is looked for, most specific first: the plug-ins folder set in
    /// Preferences › Plug-ins, then the `plugins` folder beside the running executable, then the
    /// executable's own folder. The working directory is never searched (it depends on how the app
    /// was started), and no machine's absolute path is written into the code.
    fn folders(s: &Session) -> Vec<std::path::PathBuf> {
        let mut folders: Vec<std::path::PathBuf> = Vec::new();
        let plug_ins = &s.prefs().plug_ins;
        let chosen = plug_ins.additional_plugins_folder.trim();
        if plug_ins.use_additional_plugins_folder && !chosen.is_empty() {
            folders.push(std::path::PathBuf::from(chosen));
        }
        if let Some(dir) = std::env::current_exe().ok().and_then(|e| e.parent().map(std::path::Path::to_path_buf)) {
            for folder in [dir.join("plugins"), dir] {
                if !folders.contains(&folder) {
                    folders.push(folder);
                }
            }
        }
        folders
    }

    /// `key` in the command's params names the file outright; without it the first copy in
    /// [`Self::folders`] is used. The error names every place searched, so a missing helper is a
    /// message the user can act on rather than a silent fallback to something else.
    fn file(s: &Session, p: &Value, key: &str, name: &str) -> Result<std::path::PathBuf> {
        if let Some(given) = p.get(key) {
            let given = given.as_str().ok_or_else(|| bad(CMD, format!("`{key}` must be a file path")))?;
            let path = std::path::PathBuf::from(given);
            return if path.exists() { Ok(path) } else { Err(EngineError::Other(format!("`{key}` names {given}, which does not exist"))) };
        }
        let folders = Self::folders(s);
        if let Some(found) = folders.iter().map(|folder| folder.join(name)).find(|path| path.exists()) {
            return Ok(found);
        }
        let searched: Vec<String> = folders.iter().map(|folder| folder.join(name).display().to_string()).collect();
        Err(EngineError::Other(format!(
            "{name} is not installed. Pass `{key}`, or put it in Preferences › Plug-ins › Additional Plug-ins Folder, or in a `plugins` folder beside the app. Looked in: {}",
            searched.join(", ")
        )))
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn resolve(s: &Session, p: &Value) -> Result<Self> {
        Ok(Self { exe: Self::file(s, p, "exe", HELPER)?, model: Self::file(s, p, "model", "big-lama.onnx")? })
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn execute_lama(w: usize, h: usize, ch: usize, fmt: &PixelFormat, img_data: &[f32], hole: &[bool], tool: &LamaTool) -> Result<Vec<f32>> {
    let mut rgb_bytes = Vec::with_capacity(w * h * 3);
    for i in 0..(w * h) {
        let rgba = to_rgba(fmt, &img_data[i * ch..(i + 1) * ch]);
        rgb_bytes.push((rgba[0].clamp(0.0, 1.0) * 255.0) as u8);
        rgb_bytes.push((rgba[1].clamp(0.0, 1.0) * 255.0) as u8);
        rgb_bytes.push((rgba[2].clamp(0.0, 1.0) * 255.0) as u8);
    }

    let mut mask_bytes = Vec::with_capacity(w * h);
    for m in hole {
        mask_bytes.push(if *m { 255u8 } else { 0u8 });
    }

    let img_obj = Image::from_raw(w as u32, h as u32, ChannelLayout::Rgb, SampleType::U8, rgb_bytes).map_err(|e| EngineError::Other(e.to_string()))?;
    let mask_obj = Image::from_raw(w as u32, h as u32, ChannelLayout::Gray, SampleType::U8, mask_bytes).map_err(|e| EngineError::Other(e.to_string()))?;

    let img_png = photocraft_codecs::encode(&img_obj, Format::Png, &Default::default()).map_err(|e| EngineError::Other(e.to_string()))?;
    let mask_png = photocraft_codecs::encode(&mask_obj, Format::Png, &Default::default()).map_err(|e| EngineError::Other(e.to_string()))?;

    let temp_dir = tempfile::tempdir().map_err(|e| EngineError::Other(e.to_string()))?;
    let in_p = temp_dir.path().join("lama_in.png");
    let mask_p = temp_dir.path().join("lama_mask.png");
    let out_p = temp_dir.path().join("lama_out.png");

    std::fs::write(&in_p, img_png).map_err(|e| EngineError::Other(e.to_string()))?;
    std::fs::write(&mask_p, mask_png).map_err(|e| EngineError::Other(e.to_string()))?;

    let output = std::process::Command::new(&tool.exe)
        .arg("-i")
        .arg(&in_p)
        .arg("-m")
        .arg(&mask_p)
        .arg("-o")
        .arg(&out_p)
        .arg("--model")
        .arg(&tool.model)
        .output()
        .map_err(|e| EngineError::Other(format!("lama-cleaner failed: {e}")))?;

    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        return Err(EngineError::Other(format!("LaMa inpainting failed: {err_msg}")));
    }

    let out_bytes = std::fs::read(&out_p).map_err(|e| EngineError::Other(e.to_string()))?;
    let decoded = photocraft_codecs::decode_as(Format::Png, &out_bytes).map_err(|e| EngineError::Other(e.to_string()))?;

    let inpainted_rgb = decoded.data();
    let mut out_data = img_data.to_vec();
    for i in 0..(w * h) {
        if hole[i] {
            let r = inpainted_rgb[i * 3] as f32 / 255.0;
            let g = inpainted_rgb[i * 3 + 1] as f32 / 255.0;
            let b = inpainted_rgb[i * 3 + 2] as f32 / 255.0;
            let orig_rgba = to_rgba(fmt, &img_data[i * ch..(i + 1) * ch]);
            let final_rgba = [r, g, b, orig_rgba[3]];
            from_rgba_into(fmt, final_rgba, &mut out_data[i * ch..(i + 1) * ch]);
        }
    }

    Ok(out_data)
}

#[cfg(target_arch = "wasm32")]
fn execute_lama(_w: usize, _h: usize, _ch: usize, _fmt: &PixelFormat, _img_data: &[f32], _hole: &[bool], _tool: &LamaTool) -> Result<Vec<f32>> {
    Err(EngineError::Other("LaMa Cleaner is only available in the desktop app".into()))
}

fn run_lama_cleaner(
    doc: &mut Document,
    plan: &Plan,
    p: &Value,
    ctx: &crate::jobs::JobCtx,
    label: &str,
    tool: &LamaTool,
) -> Result<Option<(Region, Vec<bool>)>> {
    let hole = hole(plan, doc.selection.as_ref())?;
    if !hole.iter().any(|m| *m) {
        return Err(bad(CMD, "the stroke doesn't cover anything that can change"));
    }
    ctx.check()?;
    let win = plan.window;
    let (img, fmt) = {
        let (surf, _) = crate::channel_cmds::target_surface(doc, plan.id, p)?;
        let fmt = surf.format();
        let img = if plan.all_layers { composite_region(doc, None, SampleLayers::All, win, fmt) } else { Region::read(surf, win) };
        (img, fmt)
    };
    if !plan.all_layers && alpha_index(&fmt).is_some_and(|a| img.data.chunks_exact(img.ch).all(|px| px.get(a).is_none_or(|v| *v <= 0.0))) {
        return Ok(None);
    }
    let (w, h) = (win.width() as usize, win.height() as usize);
    let mut data = ctx.stage(0.0, 1.0, label, |_| execute_lama(w, h, img.ch, &fmt, &img.data, &hole, tool)).map_err(|_| EngineError::Cancelled)?;
    clamp_samples(&fmt, &mut data);
    Ok(Some((Region { rect: win, ch: img.ch, data }, hole)))
}

fn fill_to_new_layer(doc: &mut Document, id: Option<LayerId>, out: &Region, hole: &[bool]) -> Result<(Rect, LayerId)> {
    let base = doc.pixel_format();
    let fmt = PixelFormat::new(base.mode, base.sample, true);
    let n = fmt.channels();
    let colours = n.saturating_sub(1);
    if out.ch != n && out.ch != colours {
        return Err(EngineError::Other("internal error: the fill's channels don't match the document".into()));
    }
    let mut data = vec![0.0f32; hole.len().saturating_mul(n)];
    for ((dst, src), m) in data.chunks_exact_mut(n).zip(out.data.chunks_exact(out.ch)).zip(hole) {
        if !*m {
            continue;
        }
        for (d, s) in dst.iter_mut().zip(src.iter().take(colours)) {
            *d = *s;
        }
        if let Some(a) = dst.last_mut() {
            *a = if out.ch == n { src.last().copied().unwrap_or(1.0) } else { 1.0 };
        }
    }
    let mut l = Layer::raster(doc.next_layer_name("Layer"), fmt);
    crate::pixels_mut(&mut l)?.write_region(out.rect, &data);
    let nid = crate::layer_place::insert_new(doc, id, l, crate::layer_place::Place::Above);
    Ok((out.rect, nid))
}

pub(super) fn lama_cleaner(s: &mut Session, p: &Value) -> Result<Value> {
    let plan = plan(s, p)?;
    let tool = LamaTool::resolve(s, p)?;
    let label = "LaMa Cleaner";
    let d = s.active().ok_or(EngineError::NoDocument)?;
    let doc = d.doc.clone();
    let p = p.clone();
    let id = plan.id;
    let new_layer = plan.new_layer;
    crate::jobs::run(
        s,
        label,
        true,
        {
            let p = p.clone();
            move |ctx| {
                ctx.progress(0.0, label);
                let mut work = (*doc).clone();
                run_lama_cleaner(&mut work, &plan, &p, ctx, label, &tool)
            }
        },
        move |s, filled| {
            let Some((out, hole)) = filled else { return Ok(json!({"changed": false})) };
            if new_layer {
                let (damage, nid) = s.edit(label, |doc, active| {
                    let r = fill_to_new_layer(doc, id, &out, &hole)?;
                    *active = Some(r.1);
                    Ok(r)
                })?;
                s.hint_damage(damage);
                return Ok(json!({ "damage": damage_json(damage), "layer": nid.0 }));
            }
            let damage = s.edit(label, |doc, _| replace(doc, Target { id, all_layers: false }, &p, &out, &hole))?;
            s.hint_damage(damage);
            Ok(json!({ "damage": damage_json(damage) }))
        },
    )
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;

    fn session() -> Session {
        Session::new()
    }

    /// The preferences folder comes first, then the app's own folders, built from wherever the
    /// running executable is: nothing depends on the machine this test runs on.
    #[test]
    fn folders_are_the_preferences_folder_then_the_apps_own() {
        let d = tempfile::tempdir().unwrap();
        let mut s = session();
        let chosen = d.path().to_string_lossy().into_owned();
        s.edit_prefs(|p| {
            p.plug_ins.use_additional_plugins_folder = true;
            p.plug_ins.additional_plugins_folder = chosen.clone();
        });
        let exe_dir = std::env::current_exe().unwrap().parent().unwrap().to_path_buf();
        assert_eq!(LamaTool::folders(&s), [d.path().to_path_buf(), exe_dir.join("plugins"), exe_dir]);
    }

    /// The plug-ins folder is only searched when Preferences says to use it, as for the WebAssembly
    /// plug-ins in the same folder.
    #[test]
    fn the_preferences_folder_is_searched_only_when_it_is_enabled() {
        let d = tempfile::tempdir().unwrap();
        let mut s = session();
        let chosen = d.path().to_string_lossy().into_owned();
        s.edit_prefs(|p| {
            p.plug_ins.use_additional_plugins_folder = false;
            p.plug_ins.additional_plugins_folder = chosen;
        });
        assert!(!LamaTool::folders(&s).contains(&d.path().to_path_buf()));
    }

    #[test]
    fn a_path_given_in_the_params_is_used_as_given() {
        let d = tempfile::tempdir().unwrap();
        let exe = d.path().join(HELPER);
        let model = d.path().join("big-lama.onnx");
        std::fs::write(&exe, b"helper").unwrap();
        std::fs::write(&model, b"model").unwrap();
        let tool = LamaTool::resolve(&session(), &serde_json::json!({"exe": exe, "model": model})).unwrap();
        assert_eq!((tool.exe, tool.model), (exe, model));
    }

    #[test]
    fn a_path_given_in_the_params_that_is_not_there_says_so() {
        let d = tempfile::tempdir().unwrap();
        let exe = d.path().join("absent-helper");
        let error = LamaTool::resolve(&session(), &serde_json::json!({"exe": exe})).unwrap_err().to_string();
        assert!(error.contains("absent-helper"), "{error}");
    }

    /// A helper that is nowhere says every place it looked, so the fix is in the message: no
    /// silent fallback, no machine-specific path baked into the code.
    #[test]
    fn a_missing_helper_names_every_place_searched() {
        let d = tempfile::tempdir().unwrap();
        let mut s = session();
        let chosen = d.path().to_string_lossy().into_owned();
        s.edit_prefs(|p| {
            p.plug_ins.use_additional_plugins_folder = true;
            p.plug_ins.additional_plugins_folder = chosen;
        });
        let error = LamaTool::resolve(&s, &serde_json::json!({})).unwrap_err().to_string();
        for folder in LamaTool::folders(&s) {
            assert!(error.contains(&folder.join(HELPER).display().to_string()), "{error}");
        }
    }

    /// Guards the rule that repository code carries no machine's absolute path: every path this
    /// file builds comes from the executable, the preferences folder or a parameter.
    #[test]
    fn no_machine_absolute_path_is_written_in_this_file() {
        let src = include_str!("lama_cleaner.rs");
        for letter in ['C', 'D', 'E', 'F', 'G', 'H', 'I', 'J'] {
            let drive = format!("{letter}:\\");
            assert!(!src.contains(&drive), "this file writes the absolute path {drive}");
        }
        for home in [concat!("/Us", "ers/"), concat!("/ho", "me/")] {
            assert!(!src.contains(home), "this file writes the absolute path {home}");
        }
    }
}
