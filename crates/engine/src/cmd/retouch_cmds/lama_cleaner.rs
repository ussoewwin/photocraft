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
    // A selection that selects nothing is no selection, as in Photoshop: Select › All then
    // Inverse leaves one, as can subtracting everything, and it shows no ants. Clipping the
    // hole by it would make every stroke \"cover nothing\" with no visible reason.
    if let Some(sel) = sel.filter(|s| !s.content_bounds().is_empty()) {
        for (v, c) in m.iter_mut().zip(coverage(sel, win)) {
            *v &= c > 0.0;
        }
    }
    Ok(m)
}

/// The LaMa helper program and its model, resolved before any work starts. Both are optional:
/// the command reports what is missing, and a missing model is downloaded from the PhotoCraft
/// model collection before the helper runs.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug)]
struct LamaTool {
    exe: Option<std::path::PathBuf>,
    model: Option<std::path::PathBuf>,
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

/// The model file and the subfolder of `plugins/lama-cleaner` it lives in, so `plugins/` itself
/// stays clean — the same layout as the SAM 3.1 pack.
#[cfg(not(target_arch = "wasm32"))]
const MODEL_NAME: &str = "big-lama.onnx";
#[cfg(not(target_arch = "wasm32"))]
const MODELS: &str = "models";
/// Where a missing model is downloaded from (the PhotoCraft model collection on Hugging Face).
#[cfg(not(target_arch = "wasm32"))]
const MODEL_URL: &str = "https://huggingface.co/ussoewwin/For_my_PhotoCraft/resolve/main/big-lama.onnx";

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

    /// Where the model is looked for, most specific first: every helper folder's `models`
    /// subfolder before the folder itself (the same layout as the SAM 3.1 pack), then the flat
    /// copies kept working for older setups. `Ok(None)` means it was found nowhere and the
    /// download path in `execute_lama` fetches it; `Err` is a path the caller named that is not
    /// there, which a download must never hide.
    fn model(s: &Session, p: &Value) -> Result<Option<std::path::PathBuf>> {
        if let Some(given) = p.get("model") {
            let given = given.as_str().ok_or_else(|| bad(CMD, "`model` must be a file path"))?;
            let path = std::path::PathBuf::from(given);
            return if path.exists() { Ok(Some(path)) } else { Err(EngineError::Other(format!("`model` names {given}, which does not exist"))) };
        }
        if let Some(path) = std::env::var_os("PHOTOCRAFT_LAMA_MODEL").filter(|p| !p.is_empty()) {
            let path = std::path::PathBuf::from(path);
            return if path.is_file() {
                Ok(Some(path))
            } else {
                Err(EngineError::Other(format!("PHOTOCRAFT_LAMA_MODEL points at {}, which is not a file", path.display())))
            };
        }
        let folders = Self::folders(s);
        let mut searched: Vec<std::path::PathBuf> = Vec::new();
        for folder in &folders {
            searched.push(folder.join("lama-cleaner").join(MODELS).join(MODEL_NAME));
            searched.push(folder.join(MODELS).join(MODEL_NAME));
            searched.push(folder.join(MODEL_NAME));
        }
        // The helper's own `models` folder is searched even when no `folders` entry covers it
        // (the helper lives where the params or the environment name it).
        if let Some(dir) = p.get("exe").and_then(|e| e.as_str()).map(std::path::PathBuf::from).and_then(|p| p.parent().map(std::path::Path::to_path_buf)) {
            for candidate in [dir.join(MODELS).join(MODEL_NAME), dir.join(MODEL_NAME)] {
                if !searched.contains(&candidate) {
                    searched.push(candidate);
                }
            }
        // The repository's own `plugins/lama-cleaner/models`, found from the app's ancestors the
        // same way the SAM 3.1 pack is: a development build lives in `target\release`, which is
        // below the checkout's `plugins` folder.
        } else if let Some(dir) = std::env::current_exe().ok().and_then(|e| e.parent().map(std::path::Path::to_path_buf)) {
            for root in dir.ancestors() {
                for candidate in [
                    root.join("plugins").join("lama-cleaner").join(MODELS).join(MODEL_NAME),
                    root.join("plugins").join(MODELS).join(MODEL_NAME),
                    root.join("plugins").join(MODEL_NAME),
                ] {
                    if !searched.contains(&candidate) {
                        searched.push(candidate);
                    }
                }
            }
        }
        if let Some(found) = searched.iter().find(|path| path.is_file()) {
            return Ok(Some(found.clone()));
        }
        Ok(None)
    }

    /// Where a missing model is downloaded to: `<plugins>/lama-cleaner/models`, derived from the
    /// preferences folder or from the helper's own location. No machine's absolute path is
    /// written here.
    #[cfg(test)]
    fn download_folder(s: &Session, helper: &std::path::Path) -> std::path::PathBuf {
        let plug_ins = &s.prefs().plug_ins;
        let chosen = plug_ins.additional_plugins_folder.trim();
        let prefs = (plug_ins.use_additional_plugins_folder && !chosen.is_empty(), chosen.to_string());
        Self::download_folder_for(prefs, helper)
    }

    /// The same folder derived from the preferences values that a job closure carries, so the
    /// download lands where [`Self::model`] looks next time.
    fn download_folder_for(prefs_plugins: (bool, String), helper: &std::path::Path) -> std::path::PathBuf {
        let (use_additional, chosen) = prefs_plugins;
        if use_additional && !chosen.is_empty() {
            return std::path::PathBuf::from(chosen).join("lama-cleaner").join(MODELS);
        }
        let parent = helper.parent().unwrap_or_else(|| std::path::Path::new("."));
        match parent.file_name().and_then(|n| n.to_str()) {
            Some("lama-cleaner") => parent.join(MODELS),
            Some(n) if n == MODELS => parent.to_path_buf(),
            _ => parent.join("lama-cleaner").join(MODELS),
        }
    }

    /// The helper is missing: the error names every place searched, so the fix is in the
    /// message. The model is not part of this check: a missing model is downloaded.
    fn missing_helper_message(s: &Session, p: &Value) -> String {
        match Self::file(s, p, "exe", HELPER) {
            Ok(_) => "the LaMa helper is not available".into(),
            Err(EngineError::Other(message)) => message,
            Err(error) => error.to_string(),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn resolve(s: &Session, p: &Value) -> Result<Self> {
        let exe = Self::file(s, p, "exe", HELPER).ok();
        let model = Self::model(s, p)?;
        Ok(Self { exe, model })
    }
}

/// A resident `lama-cleaner --serve` helper: the model stays loaded between strokes, so only
/// the inference time is paid. One process serves every request of the session; a helper that
/// cannot serve (an older build, an unexpected reply) or a process that died is dropped, and the
/// caller runs the helper once instead.
#[cfg(not(target_arch = "wasm32"))]
mod resident {
    use std::io::{BufRead, BufReader, Write};
    use std::path::{Path, PathBuf};
    use std::process::{Child, Command, Stdio};
    use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
    use std::sync::{Mutex, OnceLock};
    use std::time::Duration;

    /// How long one request may take before the helper counts as hung; the first request also
    /// waits for the model load.
    const REPLY_TIMEOUT: Duration = Duration::from_secs(120);

    #[derive(Debug, PartialEq, Eq)]
    pub(super) enum ResidentError {
        /// The helper could not serve as a resident process (failed to spawn, exited, timed out,
        /// or printed non-JSON prose as an older build does): drop the worker and fall back to a
        /// single run.
        Unavailable(String),
        /// The resident helper is alive and answered with `{"ok": false, "error": "..."}`: keep
        /// the loaded worker and report the error directly without re-running from scratch.
        Failed(String),
    }

    struct Worker {
        child: Child,
        stdin: std::process::ChildStdin,
        replies: Receiver<String>,
        exe: PathBuf,
        model: PathBuf,
    }

    impl Worker {
        fn start(exe: &Path, model: &Path) -> Result<Self, ResidentError> {
            let mut child = Command::new(exe)
                .arg("--serve")
                .arg("--model")
                .arg(model)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .spawn()
                .map_err(|e| ResidentError::Unavailable(format!("could not start {}: {e}", exe.display())))?;
            let stdin = match child.stdin.take() {
                Some(stdin) => stdin,
                None => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(ResidentError::Unavailable("the helper has no stdin".into()));
                }
            };
            let stdout = match child.stdout.take() {
                Some(stdout) => stdout,
                None => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(ResidentError::Unavailable("the helper has no stdout".into()));
                }
            };
            let (send, replies) = mpsc::channel();
            // The reader thread frees the request path from a pipe that a hung helper would block:
            // every request waits with a timeout on the channel instead of on `read_line`.
            std::thread::spawn(move || {
                let mut reader = BufReader::new(stdout);
                loop {
                    let mut line = String::new();
                    match reader.read_line(&mut line) {
                        Ok(0) | Err(_) => break,
                        Ok(_) => {
                            if send.send(line).is_err() {
                                break;
                            }
                        }
                    }
                }
            });
            Ok(Self { child, stdin, replies, exe: exe.to_path_buf(), model: model.to_path_buf() })
        }

        fn matches(&self, exe: &Path, model: &Path) -> bool {
            self.exe == exe && self.model == model
        }

        fn request(&mut self, image: &Path, mask: &Path, output: &Path) -> Result<String, ResidentError> {
            let line = request_line(image, mask, output);
            writeln!(self.stdin, "{line}")
                .and_then(|_| self.stdin.flush())
                .map_err(|e| ResidentError::Unavailable(format!("the LaMa helper stopped reading: {e}")))?;
            let reply = match self.replies.recv_timeout(REPLY_TIMEOUT) {
                Ok(reply) => reply,
                Err(RecvTimeoutError::Timeout) => {
                    return Err(ResidentError::Unavailable(format!("the LaMa helper did not answer within {} s", REPLY_TIMEOUT.as_secs())));
                }
                Err(RecvTimeoutError::Disconnected) => return Err(ResidentError::Unavailable("the LaMa helper exited".into())),
            };
            parse_reply(&reply)
        }
    }

    impl Drop for Worker {
        fn drop(&mut self) {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }

    fn slot() -> &'static Mutex<Option<Worker>> {
        static WORKER: OnceLock<Mutex<Option<Worker>>> = OnceLock::new();
        WORKER.get_or_init(|| Mutex::new(None))
    }

    /// The JSON line one request sends.
    fn request_line(image: &Path, mask: &Path, output: &Path) -> String {
        serde_json::json!({
            "image": image.to_string_lossy(),
            "mask": mask.to_string_lossy(),
            "output": output.to_string_lossy(),
        })
        .to_string()
    }

    /// The provider from a helper reply. Non-JSON output (an older helper printing its self-test,
    /// a truncated line) is [`ResidentError::Unavailable`] so the caller falls back to a one-shot
    /// run; a structured `{"ok": false, "error": "..."}` reply is [`ResidentError::Failed`] so the
    /// resident worker stays loaded and the error is reported directly.
    fn parse_reply(reply: &str) -> Result<String, ResidentError> {
        let value: serde_json::Value = serde_json::from_str(reply.trim())
            .map_err(|_| ResidentError::Unavailable(format!("the LaMa helper answered something that is not JSON: {}", one_line(reply))))?;
        if value.get("ok").and_then(serde_json::Value::as_bool) == Some(true) {
            Ok(value.get("provider").and_then(serde_json::Value::as_str).unwrap_or("unknown").to_string())
        } else {
            Err(ResidentError::Failed(
                value.get("error").and_then(serde_json::Value::as_str).map_or_else(|| "the LaMa helper reported an error".to_string(), |e| e.to_string()),
            ))
        }
    }

    fn one_line(text: &str) -> String {
        text.trim().chars().take(120).collect()
    }

    /// One request through the resident helper, starting or restarting it as needed.
    ///
    /// If an already-running worker died between strokes, it is replaced once with a fresh
    /// resident worker before giving up. A structured [`ResidentError::Failed`] reply keeps the
    /// resident worker alive in its slot.
    pub(super) fn apply(exe: &Path, model: &Path, image: &Path, mask: &Path, output: &Path) -> Result<String, ResidentError> {
        let mut guard = slot().lock().unwrap_or_else(|e| e.into_inner());
        if guard.as_ref().is_some_and(|worker| !worker.matches(exe, model)) {
            *guard = None;
        }
        let reused = guard.is_some();
        if guard.is_none() {
            *guard = Some(Worker::start(exe, model)?);
        }
        let first = guard.as_mut().ok_or_else(|| ResidentError::Unavailable("the resident LaMa helper is gone".into()))?.request(image, mask, output);
        match first {
            Ok(provider) => Ok(provider),
            Err(ResidentError::Failed(msg)) => Err(ResidentError::Failed(msg)),
            Err(ResidentError::Unavailable(reason)) => {
                *guard = None;
                if !reused {
                    return Err(ResidentError::Unavailable(reason));
                }
                *guard = Some(Worker::start(exe, model)?);
                let retry = guard.as_mut().ok_or_else(|| ResidentError::Unavailable("the resident LaMa helper is gone".into()))?.request(image, mask, output);
                if matches!(retry, Err(ResidentError::Unavailable(_))) {
                    *guard = None;
                }
                retry
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn the_request_line_is_one_json_object_with_the_three_paths() {
            let line = request_line(Path::new("in.png"), Path::new("mask.png"), Path::new("out.png"));
            let value: serde_json::Value = serde_json::from_str(&line).unwrap();
            assert_eq!(value["image"], "in.png");
            assert_eq!(value["mask"], "mask.png");
            assert_eq!(value["output"], "out.png");
            assert!(!line.contains('\n'), "one request is one line");
        }

        #[test]
        fn a_reply_names_the_provider() {
            assert_eq!(parse_reply("{\"ok\":true,\"infer_ms\":42.1,\"provider\":\"CUDA\"}\n").unwrap(), "CUDA");
            assert_eq!(parse_reply("{\"ok\":true}").unwrap(), "unknown");
        }

        #[test]
        fn an_error_reply_carries_its_message_without_marking_the_worker_unavailable() {
            let error = parse_reply("{\"ok\":false,\"error\":\"mask is empty\"}").unwrap_err();
            assert_eq!(error, ResidentError::Failed("mask is empty".into()));
            let bare = parse_reply("{\"ok\":false}").unwrap_err();
            assert!(matches!(bare, ResidentError::Failed(ref msg) if msg.contains("reported an error")), "{bare:?}");
        }

        /// An older helper prints its self-test instead of JSON: that is a reason to fall back.
        #[test]
        fn a_helper_that_prints_prose_is_not_a_worker() {
            let error = parse_reply("=== Testing Native Rust LaMa Cleaner ===\n").unwrap_err();
            assert!(matches!(error, ResidentError::Unavailable(ref msg) if msg.contains("not JSON") && msg.contains("=== Testing")), "{error:?}");
        }

        /// A helper that cannot start is an error the caller turns into the one-shot run.
        #[test]
        fn a_missing_helper_cannot_start() {
            let error = apply(Path::new("no-such-lama-helper"), Path::new("m.onnx"), Path::new("a"), Path::new("b"), Path::new("c")).unwrap_err();
            assert!(matches!(error, ResidentError::Unavailable(ref msg) if msg.contains("could not start")), "{error:?}");
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn run_helper_once(exe: &std::path::Path, model: &std::path::Path, image: &std::path::Path, mask: &std::path::Path, output: &std::path::Path) -> Result<()> {
    let result = std::process::Command::new(exe)
        .arg("-i")
        .arg(image)
        .arg("-m")
        .arg(mask)
        .arg("-o")
        .arg(output)
        .arg("--model")
        .arg(model)
        .output()
        .map_err(|e| EngineError::Other(format!("lama-cleaner failed: {e}")))?;
    if !result.status.success() {
        let err_msg = String::from_utf8_lossy(&result.stderr);
        return Err(EngineError::Other(format!("LaMa inpainting failed: {err_msg}")));
    }
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
/// What one LaMa run needs besides the pixels: the helper exe, the resolved model (already
/// downloaded by the time this runs), and the preferences folder used by the download.
struct LamaRun<'a> {
    tool: &'a LamaTool,
    helper: &'a std::path::Path,
    prefs_plugins: (bool, String),
}

#[cfg(not(target_arch = "wasm32"))]
fn execute_lama(w: usize, h: usize, ch: usize, fmt: &PixelFormat, img_data: &[f32], hole: &[bool], run: &LamaRun<'_>) -> Result<Vec<f32>> {
    let tool = run.tool;
    let helper = run.helper;
    let prefs_plugins = (run.prefs_plugins.0, run.prefs_plugins.1.clone());
    // The model must be on this machine before the helper can run. When it is not (and neither
    // `model` nor `PHOTOCRAFT_LAMA_MODEL` named one), download it from the PhotoCraft model
    // collection on Hugging Face so the first stroke works without a manual setup step. The file
    // lands in the `models` folder beside the helper, where the search above finds it next time.
    let model: std::path::PathBuf = (|| {
        if let Some(path) = std::env::var_os("PHOTOCRAFT_LAMA_MODEL").filter(|p| !p.is_empty()) {
            return Ok(std::path::PathBuf::from(path));
        }
        if let Some(model) = tool.model.as_ref().filter(|m| m.is_file()) {
            return Ok(model.clone());
        }
        let folder = LamaTool::download_folder_for(prefs_plugins, helper);
        std::fs::create_dir_all(&folder).map_err(|e| format!("cannot create {}: {e}", folder.display()))?;
        let target = folder.join(MODEL_NAME);
        eprintln!("PhotoCraft: the LaMa Cleaner model was not found locally; downloading from Hugging Face...");
        let status = std::process::Command::new(helper)
            .arg("--download")
            .arg(MODEL_URL)
            .arg("--download-out")
            .arg(&target)
            .stderr(std::process::Stdio::inherit())
            .output()
            .map_err(|e| format!("cannot start {} for the model download: {e}", helper.display()))?;
        if !status.status.success() || !target.is_file() {
            let message = String::from_utf8_lossy(&status.stderr).lines().rev().take(3).collect::<Vec<_>>().into_iter().rev().collect::<Vec<_>>().join(" | ");
            return Err(format!(
                "the LaMa Cleaner model download failed: {message}. Get it manually from {MODEL_URL} and put it in a `models` folder beside the helper."
            ));
        }
        Ok(target)
    })()
    .map_err(EngineError::Other)?;
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

    // The resident helper keeps the model loaded between strokes. A helper that cannot serve (an
    // older build) or a resident process that could not be restarted falls back to the ordinary
    // single run; a structured error from a healthy resident worker is reported directly.
    match resident::apply(helper, &model, &in_p, &mask_p, &out_p) {
        Ok(_) => {}
        Err(resident::ResidentError::Failed(msg)) => {
            return Err(EngineError::Other(format!("LaMa inpainting failed: {msg}")));
        }
        Err(resident::ResidentError::Unavailable(_)) => {
            run_helper_once(helper, &model, &in_p, &mask_p, &out_p)?;
        }
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
    run: &LamaRun<'_>,
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
    let mut data = ctx.stage(0.0, 1.0, label, |_| execute_lama(w, h, img.ch, &fmt, &img.data, &hole, run)).map_err(|_| EngineError::Cancelled)?;
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
    let Some(exe) = tool.exe.clone() else {
        return Err(EngineError::Other(LamaTool::missing_helper_message(s, p)));
    };
    let label = "LaMa Cleaner";
    let d = s.active().ok_or(EngineError::NoDocument)?;
    let doc = d.doc.clone();
    let p = p.clone();
    let id = plan.id;
    let new_layer = plan.new_layer;
    let prefs_plugins = {
        let plug_ins = &s.prefs().plug_ins;
        (plug_ins.use_additional_plugins_folder, plug_ins.additional_plugins_folder.trim().to_string())
    };
    crate::jobs::run(
        s,
        label,
        true,
        {
            let p = p.clone();
            move |ctx| {
                ctx.progress(0.0, label);
                let mut work = (*doc).clone();
                let run = LamaRun { tool: &tool, helper: &exe, prefs_plugins };
                run_lama_cleaner(&mut work, &plan, &p, ctx, label, &run)
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
        assert_eq!((tool.exe, tool.model), (Some(exe), Some(model)));
    }

    /// An explicit model path that is not there is an error, never a silent download: a mistyped
    /// path must say so.
    #[test]
    fn an_explicit_model_path_that_is_absent_is_an_error() {
        let d = tempfile::tempdir().unwrap();
        let missing = d.path().join("absent.onnx");
        let error = LamaTool::resolve(&session(), &serde_json::json!({"model": missing})).unwrap_err().to_string();
        assert!(error.contains("absent.onnx"), "{error}");
    }

    #[test]
    fn a_path_given_in_the_params_that_is_not_there_says_so() {
        let d = tempfile::tempdir().unwrap();
        let exe = d.path().join("absent-helper");
        let error = LamaTool::missing_helper_message(&session(), &serde_json::json!({"exe": exe}));
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
        let error = LamaTool::missing_helper_message(&s, &serde_json::json!({}));
        for folder in LamaTool::folders(&s) {
            assert!(error.contains(&folder.join(HELPER).display().to_string()), "{error}");
        }
    }

    /// The model lives in a `models` subfolder beside the helper (the same layout as the SAM 3.1
    /// pack), and a missing model is downloaded into exactly that folder.
    #[test]
    fn the_models_folder_is_searched_and_is_where_a_download_lands() {
        let d = tempfile::tempdir().unwrap();
        let plugins = d.path().join("plugins");
        let helper_dir = plugins.join("lama-cleaner");
        let models = helper_dir.join(MODELS);
        std::fs::create_dir_all(&models).unwrap();
        let helper = helper_dir.join(HELPER);
        std::fs::write(&helper, b"helper").unwrap();
        let s = session();
        let pack = models.join(MODEL_NAME);
        std::fs::write(&pack, b"model").unwrap();
        let tool = LamaTool::resolve(&s, &serde_json::json!({"exe": helper})).unwrap();
        assert_eq!(tool.model, Some(pack.clone()));
        assert_eq!(LamaTool::download_folder_for((false, String::new()), &helper), models);
        assert_eq!(LamaTool::download_folder(&s, &helper), models);
    }

    /// A missing model resolves to `None` (the download path picks it up) instead of failing the
    /// whole command: the download is what makes the first stroke work.
    #[test]
    fn a_missing_model_resolves_to_none_and_the_download_folder_is_still_derived() {
        let d = tempfile::tempdir().unwrap();
        let plugins = d.path().join("plugins");
        let helper = plugins.join("lama-cleaner").join(HELPER);
        std::fs::create_dir_all(helper.parent().unwrap()).unwrap();
        std::fs::write(&helper, b"helper").unwrap();
        let tool = LamaTool::resolve(&session(), &serde_json::json!({"exe": helper})).unwrap();
        assert!(tool.model.is_none(), "{tool:?}");
        assert_eq!(LamaTool::download_folder_for((false, String::new()), &helper), helper.parent().unwrap().join(MODELS));
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

    /// A selection that selects nothing is no selection, as in Photoshop. Select › All followed by
    /// Inverse leaves one: it draws no ants, and clipping the hole by it made every stroke fail
    /// with "cover nothing" until the document was reopened.
    #[test]
    fn a_selection_that_selects_nothing_does_not_clip_the_stroke() {
        let mut s = session();
        s.execute("file.new", json!({"width": 64, "height": 64})).unwrap();
        let p = json!({"points": [[24.0, 24.0, 1.0], [40.0, 40.0, 1.0]], "size": 12.0});
        let pl = plan(&s, &p).unwrap();
        s.execute("select.all", json!({})).unwrap();
        s.execute("select.inverse", json!({})).unwrap();
        let sel = s.active().unwrap().doc.selection.clone();
        assert!(sel.as_ref().is_some_and(|m| m.content_bounds().is_empty()), "the selection selects nothing and shows no ants");
        assert!(hole(&pl, sel.as_ref()).unwrap().iter().any(|v| *v), "the stroke still covers its own pixels");
    }

    /// A stroke outside a real selection changes nothing there: the hole stays empty so the
    /// command can say why.
    #[test]
    fn a_stroke_outside_the_selection_covers_nothing() {
        let mut s = session();
        s.execute("file.new", json!({"width": 64, "height": 64})).unwrap();
        s.execute("select.rect", json!({"x": 0, "y": 0, "width": 4, "height": 4})).unwrap();
        let p = json!({"points": [[32.0, 32.0, 1.0]], "size": 8.0});
        let pl = plan(&s, &p).unwrap();
        let sel = s.active().unwrap().doc.selection.clone();
        assert!(!hole(&pl, sel.as_ref()).unwrap().iter().any(|v| *v), "the stroke lies outside the selection");
    }
}
