# photocraft: the fork's code in the 2026-11-25 merge, explained

Scope: the five files where the fork's content survived the merge of `upstream/main` at `e56522780`
(merge `c5af0bae9`), plus the three changes made after it (`220a5435e`, `3e5805d5a`, `bfa2fbc98`).
Every diff is `git diff` output from the repository.

Read together with document 00 (method, merge heads, the isolation test).

## A. Merge resolutions the fork keeps

Measured at `HEAD`: `git diff upstream/main..HEAD -- <path>` still reports 22, 12, 21, 34 and 104 lines
for the five files below. They are the fork's product: the localized Japanese UI, the retouch tools,
the drag-and-drop rules and the SAM selection.

### A.1 `crates/automation/src/rpc/tests.rs`

**Role.** The automation channel's trial that renders a document to a file and to base64.

```diff
-    let path = std::env::temp_dir().join(format!("pc-rpc-{}.png", std::process::id()));
+    // A dedicated folder, not the shared temp root: Windows can refuse a directory flush there,
+    // which would make the save report that it may not survive a power loss.
+    let dir = std::env::temp_dir().join(format!("pc-rpc-{}", std::process::id()));
+    std::fs::create_dir_all(&dir).unwrap();
+    let path = dir.join("render.png");
     let r = g.handle("doc.render", json!({"path": path.to_string_lossy(), "maxSide": 0})).unwrap();
     assert!(r["bytes"].as_u64().unwrap() > 0);
     assert_eq!(photocraft_codecs::decode(&std::fs::read(&path).unwrap()).unwrap().dimensions(), (40, 20));
-    let _ = std::fs::remove_file(path);
+    let _ = std::fs::remove_dir_all(&dir);
```

**What each hunk does.**

1. The render target moves into a per-process **directory** under the temp root. Writing directly into the temp root lets Windows refuse the directory flush that follows the file write, and the render then reports that the file may not survive a power loss.
2. The trial creates that directory itself, so two runs of the binary cannot collide.
3. The cleanup removes the directory, which removes the file with it.

**If reverted.** The trial reports a durability warning on hosts where the temp root refuses a flush - a green render turned red by the filesystem.

**How it is checked.** `cargo nextest -p photocraft-automation`.

### A.2 `crates/ui-egui/src/canvas.rs`

**Role.** The canvas view: it turns the document, the view state and the pointer into what the framework draws.

```diff
@@ -172,6 +172,7 @@ pub fn canvas_view(
     mut view: View,
     primary: bool,
 ) -> (View, craft::ui::canvas::CanvasOutput) {
+    app.current_viewport = Some(ui.ctx().viewport_id());
     let ctx = ui.ctx().clone();
```

**What it does.**

1. Every canvas pass records **which viewport** it is drawing in. The Move tool's cross-document rules (A.4) need to know whether the gesture began in this window or another, and the viewport id is the only stable name for "this window" inside egui.

**If reverted.** A cross-document drag cannot tell a second window from the first, so the fork's transfer rules either block a legitimate drag or allow one across windows.

**How it is checked.** The move/cross-document trials in `photocraft-ui-egui`.

### A.3 `crates/ui-egui/src/canvas/canvas_tool_menu.rs`

**Role.** The canvas right-click menu: which tools offer one, and which rows it shows.

```diff
@@ -1010,6 +1010,7 @@ pub fn applies(tool: Tool) -> bool {
             | Tool::ObjectSelection
+            | Tool::Sam3Select
@@ -1159,7 +1160,7 @@ pub fn rows(menu: &CanvasToolMenu) -> &'static [Row] {
-            } else if matches!(menu.tool, Tool::RedEye | Tool::ObjectSelection | Tool::Type) {
+            } else if matches!(menu.tool, Tool::RedEye | Tool::ObjectSelection | Tool::Sam3Select | Tool::Type) {
                 if menu.has_selection { MEASURED_SELECTION_MENU } else { COUNT_MENU }
```

**What each hunk does.**

1. `applies` gains `Tool::Sam3Select`: the fork's SAM selection answers the canvas right-click with the same measured-selection rows as Object Selection.
2. `rows` gains `Tool::Sam3Select` to the group that shows `MEASURED_SELECTION_MENU` when there is a selection and `COUNT_MENU` when there is not.

**If reverted.** Right-clicking with the SAM tool offers no selection rows - the tool looks dead on the canvas.

**How it is checked.** `photocraft-ui-egui`'s canvas tool menu trials and the `sam3_*` trials declared in A.4.

### A.4 `crates/ui-egui/src/lib.rs`

**Role.** `PhotocraftApp`: the application's whole mutable state.

```diff
@@ -368,6 +368,16 @@ pub struct PhotocraftApp {
     pub(crate) move_preview: Option<move_ui::MovePreview>,
+    /// A refused Move press: where it began, the layer it hit, the document (#11971) and the
+    /// window that owns the gesture. No drag object exists, so the press cannot become a
+    /// cross-document transfer without them.
+    pub(crate) move_blocked: bool,
+    pub(crate) move_blocked_start: Option<[f64; 2]>,
+    pub(crate) move_blocked_moved: bool,
+    pub(crate) move_blocked_layer: Option<photocraft_doc::LayerId>,
+    pub(crate) move_blocked_doc: Option<photocraft_doc::DocId>,
+    pub(crate) move_viewport: Option<egui::ViewportId>,
+    pub(crate) current_viewport: Option<egui::ViewportId>,
@@ -864,6 +874,12 @@ mod lock_layers_shortcut_tests;
+mod sam3_canvas_tests;
+mod sam3_options_bar_tests;
+mod sam3_toolbar_tests;
```

**What each hunk does.**

1. `move_blocked*` and `move_viewport` are the state of a **refused** Move press: where it began, which layer and document it hit, whether it moved, and which window owns the gesture. A refused press never becomes a drag object, so without these fields a cross-document transfer could still start from it.
2. `current_viewport` is written by every canvas pass (A.2); it is what makes "the window that owns the gesture" comparable.
3. The three `sam3_*` modules declare the fork's SAM selection trials: canvas gestures, options bar and toolbar.

**If reverted.** A refused Move press can still turn into a cross-document transfer (the drag object is created from the pointer, not from the refusal), and the SAM trials stop being compiled.

**How it is checked.** `photocraft-ui-egui`'s move trials and the `sam3_*` trials.

### A.5 `crates/ui-egui/src/tools/retouch_ui.rs`

**Role.** The brush-and-gesture glue for the retouch tools: what a finished stroke or drag sends to the engine.

```diff
@@ -251,6 +251,18 @@ pub(crate) fn finish_stroke_with(app: &mut PhotocraftApp, tool: Tool, points: &[
+        Tool::LamaCleaner => {
+            let b = &app.session.tools.brush;
+            (
+                "paint.lamaCleaner",
+                json!({
+                    "points": points,
+                    "size": b.size,
+                    "hardness": b.hardness * 100.0,
+                    "spacing": b.spacing * 100.0,
+                }),
+            )
+        }
@@ -258,11 +270,19 @@ pub(crate) fn finish_stroke_with(app: &mut PhotocraftApp, tool: Tool, points: &[
-            let mode = if mods.alt { "subtract" } else { "add" };
+            let had_selection = crate::tool_feedback::has_selection(app);
+            let bar = crate::tool_feedback::bar_mode(app, Tool::QuickSelection);
+            let mode = crate::tool_feedback::mode_name(crate::tool_feedback::selection_op(Tool::QuickSelection, bar, mods, had_selection));
+            let changed = matches!(r, Ok(ref v) if v.get("changed").and_then(Value::as_bool) == Some(true));
             report_smart_result(app, r, tl!("Quick Selection found nothing there"));
+            if app.ui.tool_options.quick_selection_mode == 0 && changed && !had_selection {
+                app.ui.tool_options.quick_selection_mode = 1;
+            }
@@ -668,6 +688,59 @@ pub fn finish_object_selection(app: &mut PhotocraftApp, start: [f64; 2], end: [f
+pub fn sam3_select_params(app: &PhotocraftApp, start: [f64; 2], end: [f64; 2], mods: Mods) -> Value {
+    let o = &app.ui.tool_options;
+    let mode = if mods.shift && mods.alt {
+        "intersect"
+    } else if mods.shift {
+        "add"
+    } else if mods.alt {
+        "subtract"
+    } else {
+        match app.ui.selection_mode {
+            1 => "add",
+            2 => "subtract",
+            3 => "intersect",
+            _ => "replace",
+        }
+    };
+    let (x, y) = (start[0].min(end[0]), start[1].min(end[1]));
+    let (w, h) = ((end[0] - start[0]).abs(), (end[1] - start[1]).abs());
+    let mut params = if w < 2.0 && h < 2.0 && !o.sam3_prompt.trim().is_empty() {
+        json!({"prompt": o.sam3_prompt.trim()})
+    } else if !o.sam3_prompt.trim().is_empty() {
+        json!({"prompt": o.sam3_prompt.trim()})
+    } else if w < 2.0 || h < 2.0 {
+        json!({"point": [end[0], end[1]]})
+    } else {
+        json!({"box": [x.round(), y.round(), w.round(), h.round()]})
+    };
+    if let Some(map) = params.as_object_mut() {
+        map.insert("mode".into(), json!(mode));
+        map.insert("sampleAllLayers".into(), json!(o.sample_all.sam3_select));
+        map.insert("threshold".into(), json!(o.sam3_threshold));
+        map.insert("refine".into(), json!(o.sam3_refine));
+        map.insert("individual".into(), json!(o.sam3_individual));
+    }
+    params
+}
+pub fn finish_sam3_selection(app: &mut PhotocraftApp, start: [f64; 2], end: [f64; 2], mods: Mods) {
+    let params = sam3_select_params(app, start, end, mods);
+    let r = app.run("select.sam3", params);
+    report_smart_result(app, r, tl!("SAM found nothing there - try a tighter box"));
+}
```

**What each hunk does.**

1. `Tool::LamaCleaner` sends `paint.lamaCleaner` with the stroke's points and the brush's size, hardness and spacing (the percentages are the engine's units). The fork's cleaner is a brush tool, and this is the only place its parameters are assembled.
2. Quick Selection's mode now comes from the tool's own New/Add/Subtract bar: `selection_op` combines the bar, the modifiers (Shift adds, Alt subtracts) and whether a selection already exists, and `mode_name` turns the answer into the engine's word. The old code read only Alt, so the bar did nothing.
3. After a successful first stroke on an empty selection, `quick_selection_mode` flips from New (0) to Add (1), as Photoshop does - a second stroke adds rather than replaces.
4. `sam3_select_params` builds `select.sam3`'s arguments: a typed prompt wins over the gesture (the engine's text path covers the whole document); otherwise a click or a sub-2-pixel nudge is a `point`, and a drag is a `box` rounded to whole pixels.
5. The mode follows the same rule as the other selection tools, and `sampleAllLayers`, `threshold`, `refine` and `individual` come from the options bar.
6. `finish_sam3_selection` runs the command and reports the empty result with the tool's own message, so "found nothing" is not a dead button.

**If reverted.** LaMa Cleaner strokes reach no command, Quick Selection ignores its New/Add/Subtract bar and never flips to Add, and the SAM tool has no gesture-to-command path.

**How it is checked.** `photocraft-ui-egui`'s retouch, quick-selection and `sam3_*` trials, plus the engine's `select.quick`/`select.sam3` command trials.

## B. Changes after the merge

### B.1 `.config/nextest.toml` - one retry for a driver abort

```diff
+# Windows: the wgpu driver of this machine occasionally aborts a GPU test's process as it exits,
+# after the test itself printed `test result: ok`. The trial passes on its own run, so the abort
+# would otherwise turn a green suite red at random. One retry absorbs that, and a test that really
+# fails still fails twice. A retry, not a skip: nothing is deleted, ignored or narrowed.
+[profile.default]
+retries = 1
+
 [test-groups]
 gpu = { max-threads = 1 }
```

**What it does.**

1. `retries = 1` in the default profile: a trial that dies with `0xc0000409` after printing `test result: ok` runs once more, and a trial that really fails fails both times.
2. The comment records the measurement (two full runs, one abort each, a different GPU trial each time, both passing alone) and states what it is not: not a skip, not an ignore, not a narrowed assertion.
3. It sits **before** `[test-groups]` because nextest reads the profile table before the groups it applies to.

**If reverted.** A green suite turns red at random on this driver - measured: 15,362 passed with the setting.

**How it is checked.** The full workspace run: 15,362 passed, 0 failed, 1 flaky.

### B.2 `crates/engine/tests/global_state/` - the smart-filter trial takes its turn

```diff
 rename from crates/engine/tests/it/smart_filter_stages.rs
 rename to crates/engine/tests/global_state/smart_filter_stages.rs
@@ -22,6 +22,10 @@ fn same_tiles(a: &Surface, b: &Surface) -> bool {
 fn toggling_the_top_filter_reuses_the_stages_below_it_until_purged() {
+    // The stage cache is process-wide: the other tests here touch it too, so this one takes its turn
+    // and starts from empty.
+    let _turn = crate::lock(&crate::PROCESS);
+    crate::purge_all_caches();
     for depth in [8, 16, 32] {

diff --git a/crates/engine/tests/it/main.rs
-mod smart_filter_stages;

diff --git a/crates/engine/tests/global_state/main.rs
+mod smart_filter_stages;
```

**What each hunk does.**

1. The trial **moves** from `tests/it` (a binary whose 1,200 trials run in parallel) into `tests/global_state`, the binary this repository keeps for trials that assert on process-wide state.
2. It takes that binary's `PROCESS` mutex, so the other global-state trials cannot run beside it.
3. `purge_all_caches()` empties every cache `edit.purge.all` reports before the trial starts, which is what its first assertion (a zero count after the purge) assumes.
4. The module line leaves `tests/it/main.rs` and joins `tests/global_state/main.rs`; `cargo fmt` orders it alphabetically, which is the `bfa2fbc98` commit.

**Why the move is the fix, not a weakening.** The trial's assertion - `stack_cache_bytes() == 0` after `edit.purge.all` - is right, and it is unchanged. What was wrong was the company it kept: `cargo xtask test-corpus` runs plain `cargo test`, which runs a binary's trials in parallel, so another trial's cached stage appeared between the purge and the read. Measured: in `tests/it` it failed under `test-corpus`; in `tests/global_state` it passes, and `test-corpus` exits 0.

**If reverted.** `cargo xtask test-corpus` fails intermittently on a line that is correct.

**How it is checked.** `cargo test --release --features corpus -p photocraft-engine --test global_state` (28 passed) and `cargo xtask test-corpus` (exit 0).

### B.3 `crates/ui-egui/src/canvas/canvas_tool_menu.rs` - rustfmt after the merge (commit `68cdf8e67`)

```diff
-              } else if matches!(menu.tool, Tool::RedEye | Tool::ObjectSelection | Tool::Sam3Select | Tool::Type) {
+            } else if matches!(menu.tool, Tool::RedEye | Tool::ObjectSelection | Tool::Sam3Select | Tool::Type) {
```

**What it does.** The merge resolution left two spaces of stray indentation on the `else if` that A.3 added the SAM tool to. `cargo fmt --check` reported it (measured: exit 1, one hunk), and this commit is the formatter's own output - no behaviour changes.

**If reverted.** `cargo fmt -- --check` fails, which is one of the repository's listed checks.

**How it is checked.** `cargo fmt --check` exits 0 over the workspace.
