//! `select.sam3`: the SAM 3.1 selection command.
//!
//! The model runs in the standalone `plugins/sam3` helper (`sam3-select`), which never links against
//! ComfyUI or Python. This module only moves pixels: it hands the helper interleaved 8-bit RGB, gets
//! `[N, H, W]` binary masks back, and turns them into a selection through
//! [`combine_region`]. The prompt semantics match `SAM3_Detect`
//! (`comfy_extras/nodes_sam3.py:116-255`): a text prompt detects and refines, a box segments inside
//! the box, a point segments from the click.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

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
/// Where the pack is downloaded from when it is not on this machine.
const PACK_URL: &str = "https://huggingface.co/ussoewwin/SAM3.1-HSWQ-ConvRot-INT8/resolve/main/sam3.1_multiplex_hswq_convrot_int8.safetensors";
/// The image the model is fed.
const SAM_SIZE: f32 = 1008.0;

fn bad(cmd: &str, msg: &str) -> EngineError {
    EngineError::BadParams { cmd: cmd.into(), msg: msg.into() }
}

fn helper_name() -> String {
    format!("sam3-select{}", std::env::consts::EXE_SUFFIX)
}

/// Every place the helper is looked for, in order. A missing binary names all of them.
fn helper_candidates(s: Option<&Session>) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = Vec::new();
    if let Some(session) = s {
        let plug_ins = &session.prefs().plug_ins;
        let chosen = plug_ins.additional_plugins_folder.trim();
        if plug_ins.use_additional_plugins_folder && !chosen.is_empty() {
            let folder = PathBuf::from(chosen);
            out.push(folder.join(helper_name()));
            out.push(folder.join("sam3").join(helper_name()));
        }
    }
    let exe_dir = std::env::current_exe().ok().and_then(|e| e.parent().map(Path::to_path_buf));
    if let Some(dir) = &exe_dir {
        out.push(dir.join("plugins").join(helper_name()));
        out.push(dir.join("plugins").join("sam3").join(helper_name()));
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
        out.push(root.join("plugins").join(helper_name()));
        out.push(root.join("plugins").join("sam3").join(helper_name()));
        out.push(root.join("plugins").join("sam3").join("target").join("release").join(helper_name()));
    }
    let mut deduped: Vec<PathBuf> = Vec::with_capacity(out.len());
    for candidate in out {
        if !deduped.contains(&candidate) {
            deduped.push(candidate);
        }
    }
    deduped
}

/// The helper binary: `CRAFT_SAM3_HELPER`, else the first candidate that exists.
fn resolve_helper(s: Option<&Session>) -> Result<PathBuf> {
    if let Some(path) = std::env::var_os(HELPER_ENV) {
        let path = PathBuf::from(path);
        return if path.is_file() { Ok(path) } else { Err(bad("select.sam3", &format!("{HELPER_ENV} points at {}, which is not a file", path.display()))) };
    }
    let candidates = helper_candidates(s);
    if let Some(found) = candidates.iter().find(|path| path.is_file()) {
        return Ok(found.clone());
    }
    let searched: Vec<String> = candidates.iter().map(|p| p.display().to_string()).collect();
    Err(bad(
        "select.sam3",
        &format!(
            "the SAM 3.1 helper was not found. Build it with `cargo build --release` in plugins/sam3, or set {HELPER_ENV}. Searched: {}",
            searched.join(", ")
        ),
    ))
}

/// The HSWQ pack: `CRAFT_SAM3_PACK`, else beside the helper, else `plugins/sam3/` or `plugins/`.
fn resolve_pack(s: Option<&Session>, helper: &Path) -> Result<PathBuf> {
    if let Some(path) = std::env::var_os(PACK_ENV) {
        let path = PathBuf::from(path);
        return if path.is_file() { Ok(path) } else { Err(bad("select.sam3", &format!("{PACK_ENV} points at {}, which is not a file", path.display()))) };
    }
    let mut searched: Vec<PathBuf> = Vec::new();
    if let Some(session) = s {
        let plug_ins = &session.prefs().plug_ins;
        let chosen = plug_ins.additional_plugins_folder.trim();
        if plug_ins.use_additional_plugins_folder && !chosen.is_empty() {
            let folder = PathBuf::from(chosen);
            searched.push(folder.join(PACK_NAME));
            searched.push(folder.join("sam3").join(PACK_NAME));
        }
    }
    if let Some(dir) = helper.parent() {
        searched.push(dir.join(PACK_NAME));
        searched.push(dir.join("sam3").join(PACK_NAME));
    }
    for root in helper.ancestors() {
        if root.file_name().is_some_and(|n| n == "sam3") {
            searched.push(root.join(PACK_NAME));
        }
        searched.push(root.join("plugins").join("sam3").join(PACK_NAME));
        searched.push(root.join("plugins").join(PACK_NAME));
    }
    if let Ok(cwd) = std::env::current_dir() {
        for root in cwd.ancestors() {
            searched.push(root.join("plugins").join("sam3").join(PACK_NAME));
            searched.push(root.join("plugins").join(PACK_NAME));
        }
    }
    let mut deduped: Vec<PathBuf> = Vec::with_capacity(searched.len());
    for candidate in searched {
        if !deduped.contains(&candidate) {
            deduped.push(candidate);
        }
    }
    if let Some(found) = deduped.iter().find(|path| path.is_file()) {
        return Ok(found.clone());
    }
    let listed: Vec<String> = deduped.iter().map(|p| p.display().to_string()).collect();
    Err(bad(
        "select.sam3",
        &format!("{PACK_NAME} was not found. Put it beside the helper or in plugins/sam3, or set {PACK_ENV}. Searched: {}", listed.join(", ")),
    ))
}

fn resolve_data_dir(helper: &Path, pack: &Path) -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Some(dir) = pack.parent() {
        candidates.push(dir.join("data"));
        candidates.push(dir.join("sam3").join("data"));
    }
    if let Some(dir) = helper.parent() {
        candidates.push(dir.join("data"));
        candidates.push(dir.join("sam3").join("data"));
    }
    for root in helper.ancestors() {
        candidates.push(root.join("plugins").join("sam3").join("data"));
    }
    candidates.into_iter().find(|d| d.join("vocab.json").is_file())
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
        return Ok(SamPrompt::Box([
            (x - bounds.x0 as f32) * sx,
            (y - bounds.y0 as f32) * sy,
            (x + bw - bounds.x0 as f32) * sx,
            (y + bh - bounds.y0 as f32) * sy,
        ]));
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

/// A resident `sam3-select --serve` helper: the 1.7 GB pack and its 41 dequantised ConvRot layers
/// stay loaded in VRAM between selections, and full-frame ViTDet features are cached per image.
#[cfg(not(target_arch = "wasm32"))]
mod resident {
    use std::io::{BufRead, BufReader, Write};
    use std::path::{Path, PathBuf};
    use std::process::{Child, Command, Stdio};
    use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
    use std::sync::{Mutex, OnceLock};
    use std::time::Duration;

    use super::SamPrompt;

    /// How long one request may take before the helper counts as hung; the first request also
    /// waits for the pack load + ConvRot dequantisation.
    const REPLY_TIMEOUT: Duration = Duration::from_secs(180);

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

    pub(super) struct RequestArgs<'a> {
        pub raw_in: &'a Path,
        pub width: usize,
        pub height: usize,
        pub prompt: &'a SamPrompt,
        pub threshold: f64,
        pub refine: u64,
        pub individual: bool,
        pub out: &'a Path,
    }

    struct Worker {
        child: Child,
        stdin: std::process::ChildStdin,
        replies: Receiver<String>,
        exe: PathBuf,
        pack: PathBuf,
        data_dir: Option<PathBuf>,
    }

    impl Worker {
        fn start(exe: &Path, pack: &Path, data_dir: Option<&Path>) -> Result<Self, ResidentError> {
            let mut cmd = Command::new(exe);
            cmd.arg("--serve").arg("--model").arg(pack);
            if let Some(data) = data_dir {
                cmd.arg("--data").arg(data);
            }
            let mut child = cmd
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
            Ok(Self { child, stdin, replies, exe: exe.to_path_buf(), pack: pack.to_path_buf(), data_dir: data_dir.map(Path::to_path_buf) })
        }

        fn matches(&self, exe: &Path, pack: &Path, data_dir: Option<&Path>) -> bool {
            self.exe == exe && self.pack == pack && self.data_dir.as_deref() == data_dir
        }

        fn request(&mut self, args: &RequestArgs<'_>) -> Result<(), ResidentError> {
            let line = request_line(args);
            writeln!(self.stdin, "{line}")
                .and_then(|_| self.stdin.flush())
                .map_err(|e| ResidentError::Unavailable(format!("the SAM 3.1 helper stopped reading: {e}")))?;
            let reply = match self.replies.recv_timeout(REPLY_TIMEOUT) {
                Ok(reply) => reply,
                Err(RecvTimeoutError::Timeout) => {
                    return Err(ResidentError::Unavailable(format!("the SAM 3.1 helper did not answer within {} s", REPLY_TIMEOUT.as_secs())));
                }
                Err(RecvTimeoutError::Disconnected) => return Err(ResidentError::Unavailable("the SAM 3.1 helper exited".into())),
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

    pub(super) fn request_line(args: &RequestArgs<'_>) -> String {
        let mut obj = serde_json::json!({
            "raw": args.raw_in.to_string_lossy(),
            "width": args.width,
            "height": args.height,
            "threshold": args.threshold,
            "refine": args.refine,
            "individual": args.individual,
            "out": args.out.to_string_lossy(),
        });
        if let Some(map) = obj.as_object_mut() {
            match args.prompt {
                SamPrompt::Text(text) => {
                    map.insert("prompt".into(), serde_json::json!(text));
                }
                SamPrompt::Box(corners) => {
                    map.insert("box_xyxy".into(), serde_json::json!(corners));
                }
                SamPrompt::Point(point) => {
                    map.insert("point_xy".into(), serde_json::json!(point));
                }
            }
        }
        obj.to_string()
    }

    pub(super) fn parse_reply(reply: &str) -> Result<(), ResidentError> {
        let value: serde_json::Value = serde_json::from_str(reply.trim())
            .map_err(|_| ResidentError::Unavailable(format!("the SAM 3.1 helper answered something that is not JSON: {}", one_line(reply))))?;
        if value.get("ok").and_then(serde_json::Value::as_bool) == Some(true) {
            Ok(())
        } else {
            Err(ResidentError::Failed(
                value.get("error").and_then(serde_json::Value::as_str).map_or_else(|| "the SAM 3.1 helper reported an error".to_string(), |e| e.to_string()),
            ))
        }
    }

    fn one_line(text: &str) -> String {
        text.trim().chars().take(120).collect()
    }

    pub(super) fn apply(exe: &Path, pack: &Path, data_dir: Option<&Path>, args: &RequestArgs<'_>) -> Result<(), ResidentError> {
        let mut guard = slot().lock().unwrap_or_else(|e| e.into_inner());
        if guard.as_ref().is_some_and(|worker| !worker.matches(exe, pack, data_dir)) {
            *guard = None;
        }
        let reused = guard.is_some();
        if guard.is_none() {
            *guard = Some(Worker::start(exe, pack, data_dir)?);
        }
        let first = guard.as_mut().ok_or_else(|| ResidentError::Unavailable("the resident SAM 3.1 helper is gone".into()))?.request(args);
        match first {
            Ok(()) => Ok(()),
            Err(ResidentError::Failed(msg)) => Err(ResidentError::Failed(msg)),
            Err(ResidentError::Unavailable(reason)) => {
                *guard = None;
                if !reused {
                    return Err(ResidentError::Unavailable(reason));
                }
                *guard = Some(Worker::start(exe, pack, data_dir)?);
                let retry = guard.as_mut().ok_or_else(|| ResidentError::Unavailable("the resident SAM 3.1 helper is gone".into()))?.request(args);
                if matches!(retry, Err(ResidentError::Unavailable(_))) {
                    *guard = None;
                }
                retry
            }
        }
    }
}

/// Runs the helper and returns the masks it wrote.
fn run_helper(s: &Session, p: &Value, bounds: Rect, pixels: &[[f32; 4]], prompt: &SamPrompt) -> Result<Vec<HelperMask>> {
    let helper = resolve_helper(Some(s))?;
    let pack = match resolve_pack(Some(s), &helper) {
        Ok(pack) => pack,
        Err(_) if std::env::var_os(PACK_ENV).is_none() => {
            // Not on this machine: download from the release page so the first selection
            // works without a manual setup step. The pack lands beside the helper, where
            // `resolve_pack` finds it next time.
            let target = helper.parent().unwrap_or(Path::new(".")).join(PACK_NAME);
            eprintln!("PhotoCraft: the SAM 3.1 pack was not found locally; downloading from Hugging Face...");
            let status = Command::new(&helper)
                .arg("--download")
                .arg(PACK_URL)
                .arg("--download-out")
                .arg(&target)
                .stderr(Stdio::inherit())
                .output()
                .map_err(|e| bad("select.sam3", &format!("cannot start {} for the pack download: {e}", helper.display())))?;
            if !status.status.success() || !target.is_file() {
                let message =
                    String::from_utf8_lossy(&status.stderr).lines().rev().take(3).collect::<Vec<_>>().into_iter().rev().collect::<Vec<_>>().join(" | ");
                return Err(bad(
                    "select.sam3",
                    &format!("the SAM 3.1 pack download failed: {message}. Get it manually from {PACK_URL} and put it beside the helper."),
                ));
            }
            target
        }
        Err(error) => return Err(error),
    };
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
        for &c in px.iter().take(3) {
            rgb.push((c.clamp(0.0, 1.0) * 255.0).round() as u8);
        }
    }
    std::fs::write(&raw_in, &rgb).map_err(|e| bad("select.sam3", &format!("cannot write {}: {e}", raw_in.display())))?;

    let threshold = p.get("threshold").and_then(Value::as_f64).unwrap_or(0.5).clamp(0.0, 1.0);
    let refine = p.get("refine").and_then(Value::as_u64).unwrap_or(2).min(5);
    let individual = p.get("individual").and_then(Value::as_bool).unwrap_or(false);
    let data_dir = resolve_data_dir(&helper, &pack);

    #[cfg(not(target_arch = "wasm32"))]
    let served = {
        let req_args = resident::RequestArgs { raw_in: &raw_in, width: w, height: h, prompt, threshold, refine, individual, out: &out };
        match resident::apply(&helper, &pack, data_dir.as_deref(), &req_args) {
            Ok(()) => true,
            Err(resident::ResidentError::Failed(message)) => {
                let _ = std::fs::remove_dir_all(&dir);
                return Err(bad("select.sam3", &format!("the SAM 3.1 helper failed: {message}")));
            }
            Err(resident::ResidentError::Unavailable(_)) => false,
        }
    };
    #[cfg(target_arch = "wasm32")]
    let served = false;

    if !served {
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
    }

    let meta: Value = std::fs::read(out.with_extension("json"))
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .ok_or_else(|| bad("select.sam3", "the SAM 3.1 helper wrote no result"))?;
    let raw = std::fs::read(&out).map_err(|e| bad("select.sam3", &format!("cannot read {}: {e}", out.display())))?;
    let values: Vec<f32> = raw.as_chunks::<4>().0.iter().map(|c| f32::from_le_bytes(*c)).collect();
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
    let masks = run_helper(s, p, bounds, &pixels, &prompt)?;
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
        assert!(HelperMask::from_mask(&[0.0f32; 12], 4, 3).is_none());
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

    #[test]
    fn helper_candidates_include_plugins_and_sam3_subfolders() {
        let s = Session::new();
        let list = helper_candidates(Some(&s));
        assert!(!list.is_empty());
        assert!(list.iter().any(|p| p.ends_with(Path::new("plugins").join(helper_name()))));
        assert!(list.iter().any(|p| p.ends_with(Path::new("plugins").join("sam3").join(helper_name()))));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn resident_request_line_and_reply_parsing_cover_success_failure_and_prose() {
        let prompt = SamPrompt::Point([504.0, 504.0]);
        let args = resident::RequestArgs {
            raw_in: Path::new("image.rgb"),
            width: 256,
            height: 128,
            prompt: &prompt,
            threshold: 0.5,
            refine: 2,
            individual: false,
            out: Path::new("masks.bin"),
        };
        let line = resident::request_line(&args);
        assert!(!line.contains('\n'));
        let parsed: Value = serde_json::from_str(&line).expect("json");
        assert_eq!(parsed["width"], 256);
        assert_eq!(parsed["height"], 128);
        assert_eq!(parsed["point_xy"], json!([504.0, 504.0]));

        assert!(resident::parse_reply("{\"ok\":true,\"objects\":1,\"infer_ms\":120.5}\n").is_ok());
        assert_eq!(resident::parse_reply("{\"ok\":false,\"error\":\"pack missing\"}").unwrap_err(), resident::ResidentError::Failed("pack missing".into()));
        assert!(matches!(resident::parse_reply("sam3-select: --serve is not implemented yet\n").unwrap_err(), resident::ResidentError::Unavailable(_)));
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
