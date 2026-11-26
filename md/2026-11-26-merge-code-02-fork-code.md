# The fork's code in photocraft after the upstream merge

The files the merges decided on. The upstream is `storytold/photocraft` `main` at
`d827906d6`; the fork's `main` is `06dd81e8b`.

## How a file got here

A file is in this document when the merge had to decide something about it: its merged content
differs from **both** parents (`git show <p1>:<path>`, `git show <p2>:<path>`, `git show <merge>:<path>`),
or it was written by the repair commit that followed the merge. A file that one side left alone is
not a decision, and it is not listed.

Every diff below is `git diff <upstream> <head> -- <path>`: the fork's code as the tree holds it now.

## `Cargo.lock`

**Role.** The workspace lock file.

**The code.**

```diff
diff --git a/Cargo.lock b/Cargo.lock
index ad99b1725..00dd11f2b 100644
--- a/Cargo.lock
+++ b/Cargo.lock
@@ -6723,6 +6723,7 @@ name = "photocraft-engine-paint"
 version = "0.23.1"
 dependencies = [
  "craft-cms 0.1.0 (git+https://github.com/storytold/craft-libs?rev=a165ef3a2cbc1e88f1b945966b77dc75a1fdf8e4)",
+ "craft-image-codecs",
  "photocraft-algo",
  "photocraft-color",
  "photocraft-compose",
@@ -6735,6 +6736,7 @@ dependencies = [
  "rayon",
  "serde",
  "serde_json",
+ "tempfile",
 ]
 
 [[package]]
```

**What it does, hunk by hunk.**

1. The merged graph keeps upstream's craft-libs pin `a165ef3a2` for the framework crates and the older `249a41bd9` for `craft-develop`, exactly as upstream's own workspace file asks; `photocraft-engine-paint` keeps `craft-image-codecs` and `tempfile`, which the LaMa Cleaner's module needs.

**Checked by.** `cargo metadata` regenerates it; `cargo check --workspace --all-targets` (exit 0) proves the graph.

## `crates/engine-core/src/cmd.rs`

**Role.** The engine's command modules.

**The code.**

```diff
diff --git a/crates/engine-core/src/cmd.rs b/crates/engine-core/src/cmd.rs
index 764d48d42..3eb51fd69 100644
--- a/crates/engine-core/src/cmd.rs
+++ b/crates/engine-core/src/cmd.rs
@@ -103,6 +103,8 @@ pub mod plugin_cmds;
 pub mod preset_import_cmds;
 pub mod print_cmds;
 pub mod render_cmds;
+pub mod sam3_select_cmds;
+
 #[cfg(test)]
 mod rotate_canvas_bitmap_tests;
 
```

**What it does, hunk by hunk.**

1. The fork's `sam3_select_cmds` module is declared beside upstream's list.

**Checked by.** `cargo check -p photocraft-engine-core --all-targets` (exit 0) and the command-id table check in `cargo xtask panel-commands` (exit 0).

## `crates/engine-core/src/install.rs`

**Role.** The registry of command tables: every module's `specs` function, by name.

**The code.**

```diff
diff --git a/crates/engine-core/src/install.rs b/crates/engine-core/src/install.rs
index 44d6a4f9e..c56159802 100644
--- a/crates/engine-core/src/install.rs
+++ b/crates/engine-core/src/install.rs
@@ -44,6 +44,7 @@ const TABLES: &[Table] = &[
     ("float_cmds", crate::float_cmds::specs),
     ("vector_cmds", crate::vector_cmds::specs),
     ("smartselect_cmds", crate::smartselect_cmds::specs),
+    ("sam3_select_cmds", crate::sam3_select_cmds::specs),
     ("cutout_cmds", crate::cutout_cmds::specs),
     ("symmetry_cmds", crate::symmetry_cmds::specs),
     ("edit_cmds", crate::edit_cmds::specs),
```

**What it does, hunk by hunk.**

1. `sam3_select_cmds` joins `TABLES`, so its commands are installed with the rest and reach the menus and the automation surface.

**Checked by.** `cargo xtask panel-commands` (exit 0 on 2026-11-26).

## `crates/engine/tests/global_state/main.rs`

**Role.** The trials that assert on process-wide state, one at a time.

**The code.**

```diff
diff --git a/crates/engine/tests/global_state/main.rs b/crates/engine/tests/global_state/main.rs
index 741a80a11..cdd2eb38d 100644
--- a/crates/engine/tests/global_state/main.rs
+++ b/crates/engine/tests/global_state/main.rs
@@ -12,6 +12,7 @@ mod scratch_disk;
 mod scratch_pause;
 mod scratch_release;
 mod shape_kernel_budget;
+mod smart_filter_stages;
 mod smart_source_cache;
 mod smart_source_peak;
 mod spot_dot_gain;
```

**What it does, hunk by hunk.**

1. `smart_filter_stages` is listed here: the trial asserts on a cache the whole process shares, and it took its turn here instead of sharing a process with the parallel screen trials.

**Checked by.** `cargo xtask test-corpus` and the workspace run (green on 2026-11-26).

## `crates/engine/tests/it/main.rs`

**Role.** The engine's screen-trial module list.

**The code.**

```diff
diff --git a/crates/engine/tests/it/main.rs b/crates/engine/tests/it/main.rs
index 69b4cb084..d38266586 100644
--- a/crates/engine/tests/it/main.rs
+++ b/crates/engine/tests/it/main.rs
@@ -182,7 +182,6 @@ mod shortcuts_headless;
 mod smart_clipboard_sources;
 mod smart_duplicates;
 mod smart_filter_canvas_geometry;
-mod smart_filter_stages;
 mod smart_object_tones;
 mod smart_psd;
 mod smart_rerender_fallback;
```

**What it does, hunk by hunk.**

1. `smart_filter_stages` is no longer listed here, because the trial moved to the global-state binary above; leaving both would run it twice, once where it cannot pass.

**Checked by.** `cargo nextest run -p photocraft-engine` (green on 2026-11-26).

## `crates/ui-egui/src/canvas.rs`

**Role.** The canvas view: one document's canvas in the window.

**The code.**

```diff
diff --git a/crates/ui-egui/src/canvas.rs b/crates/ui-egui/src/canvas.rs
index f2c04a720..51b96350a 100644
--- a/crates/ui-egui/src/canvas.rs
+++ b/crates/ui-egui/src/canvas.rs
@@ -176,6 +176,7 @@ pub fn canvas_view(
     mut view: View,
     primary: bool,
 ) -> (View, craft::ui::canvas::CanvasOutput) {
+    app.current_viewport = Some(ui.ctx().viewport_id());
     let ctx = ui.ctx().clone();
     let idx = host.idx;
     // The primary canvas keeps its view in `app.ui.views`: the pass starts from the stored view and stores the one it
```

**What it does, hunk by hunk.**

1. `canvas_view` records the viewport it draws in, `app.current_viewport`, so a refused Move press can name the window that owns the gesture (#11971).

**Checked by.** `cargo nextest run -p photocraft-ui-egui` (green on 2026-11-26).

## `crates/ui-egui/src/canvas/canvas_tool_menu.rs`

**Role.** The canvas right-click menu.

**The code.**

```diff
diff --git a/crates/ui-egui/src/canvas/canvas_tool_menu.rs b/crates/ui-egui/src/canvas/canvas_tool_menu.rs
index 98d1dbf39..17fbfe871 100644
--- a/crates/ui-egui/src/canvas/canvas_tool_menu.rs
+++ b/crates/ui-egui/src/canvas/canvas_tool_menu.rs
@@ -1032,6 +1032,7 @@ pub fn applies(tool: Tool) -> bool {
             | Tool::MagneticLasso
             | Tool::MagicWand
             | Tool::ObjectSelection
+            | Tool::Sam3Select
             // Its brush is set from the options bar's chip; the canvas right-click is the
             // selection menu, as in Photoshop (#7268).
             | Tool::QuickSelection
@@ -1197,7 +1198,7 @@ pub fn rows(menu: &CanvasToolMenu) -> &'static [Row] {
                 SAMPLER_MENU
             } else if menu.tool == Tool::Count {
                 COUNT_MENU
-            } else if matches!(menu.tool, Tool::RedEye | Tool::ObjectSelection | Tool::Type) {
+            } else if matches!(menu.tool, Tool::RedEye | Tool::ObjectSelection | Tool::Sam3Select | Tool::Type) {
                 if menu.has_selection { MEASURED_SELECTION_MENU } else { COUNT_MENU }
             } else if menu.tool == Tool::Eyedropper {
                 EYEDROPPER_MENU
```

**What it does, hunk by hunk.**

1. `Tool::Sam3Select` joins the tools whose right-click opens the selection menu.
2. With a selection it shows the measured-selection menu, and without one the count menu, as the other selection tools do.

**Checked by.** `cargo nextest run -p photocraft-ui-egui` (green on 2026-11-26); `sam3_canvas_tests` covers the tool.

## `crates/ui-egui/src/canvas/tools/mod.rs`

**Role.** The tool families: which family a tool belongs to.

**The code.**

```diff
diff --git a/crates/ui-egui/src/canvas/tools/mod.rs b/crates/ui-egui/src/canvas/tools/mod.rs
index 2aa3f2726..d495bd14c 100644
--- a/crates/ui-egui/src/canvas/tools/mod.rs
+++ b/crates/ui-egui/src/canvas/tools/mod.rs
@@ -208,7 +208,7 @@ pub(crate) fn family(tool: Tool) -> Family {
         Tool::Artboard => Family::Artboard,
         Tool::Frame => Family::Frame,
         Tool::Slice | Tool::SliceSelect => Family::Slice,
-        Tool::Patch | Tool::ContentAwareMove | Tool::RedEye | Tool::ObjectSelection => Family::Retouch,
+        Tool::Patch | Tool::ContentAwareMove | Tool::RedEye | Tool::ObjectSelection | Tool::Sam3Select => Family::Retouch,
         Tool::Pen
         | Tool::FreeformPen
         | Tool::CurvaturePen
```

**What it does, hunk by hunk.**

1. `Tool::Sam3Select` joins `Family::Retouch` beside Object Selection and Red Eye, so its options bar and canvas behaviour follow the retouch tools.

**Checked by.** `cargo nextest run -p photocraft-ui-egui` (green on 2026-11-26).

## `crates/ui-egui/src/canvas/tools/move_tool.rs`

**Role.** The Move tool: the press, the refusals, and the cross-document transfer.

**The code.**

```diff
diff --git a/crates/ui-egui/src/canvas/tools/move_tool.rs b/crates/ui-egui/src/canvas/tools/move_tool.rs
index c2823c064..4e3c7915c 100644
--- a/crates/ui-egui/src/canvas/tools/move_tool.rs
+++ b/crates/ui-egui/src/canvas/tools/move_tool.rs
@@ -141,6 +141,35 @@ fn rect4(r: photocraft_geom::Rect) -> [f64; 4] {
     [f64::from(r.x0), f64::from(r.y0), f64::from(r.x1), f64::from(r.y1)]
 }
 
+/// A layer a Move drag can't move in place: locked all over, locked in position, or the Background.
+/// Auto-Select skips it; a drag toward another document transfers it (#11971).
+fn locked_layer(app: &PhotocraftApp, id: photocraft_doc::LayerId) -> bool {
+    app.session.active().is_some_and(|st| {
+        let locks = st.doc.effective_locks(id);
+        locks.position || locks.all || st.doc.is_background(id)
+    })
+}
+
+/// The topmost layer at `p` a drag toward another document would transfer: the lock a pick skips
+/// is exactly what travels (#11971).
+fn locked_hit_at(app: &mut PhotocraftApp, p: [f64; 2]) -> Option<photocraft_doc::LayerId> {
+    let target = app.ui.tool_options.move_target.clone();
+    let hit = app
+        .run("layer.pickAt", json!({"x": p[0], "y": p[1], "target": target, "select": false, "includeLocked": true}))
+        .ok()
+        .and_then(|v| v["layer"].as_u64())
+        .map(photocraft_doc::LayerId);
+    hit.filter(|id| locked_layer(app, *id))
+}
+
+/// A refused Move press can still become a cross-document transfer (#11971): while that is
+/// possible the lock message waits, so the drag can reach another document's tab or tile.
+fn may_transfer(app: &PhotocraftApp) -> bool {
+    app.session.documents().len() > 1
+        && app.ui.transform.is_none()
+        && (app.move_blocked_layer.is_some() || app.session.active().is_some_and(|st| st.doc.selection.is_some()))
+}
+
 /// The box of what a drag moves, where it is at the press: the selected pixels' outline (the float's
 /// piece once cut), else the union of the frames of what moves: a group carries its layers and linked layers go
 /// along.
@@ -161,6 +190,26 @@ fn moving_box(app: &PhotocraftApp, pixels: bool) -> Option<[f64; 4]> {
 }
 
 /// The press: Auto-Select's pick, the refusals, then the kind of drag it starts.
+/// A refused Move press: the point, the layer under it, the document it began in and the window
+/// that owns the gesture (#11971). No drag object exists, so without them the press can neither
+/// show the lock message nor become a cross-document transfer.
+fn record_refused_press(app: &mut PhotocraftApp, p: [f64; 2]) {
+    app.move_blocked = true;
+    app.move_blocked_start = Some(p);
+    app.move_blocked_moved = false;
+    app.move_blocked_layer = if app.session.active().is_some_and(|st| st.doc.selection.is_none()) {
+        let target = app.ui.tool_options.move_target.clone();
+        app.run("layer.pickAt", json!({"x": p[0], "y": p[1], "target": target, "select": false, "includeLocked": true}))
+            .ok()
+            .and_then(|v| v["layer"].as_u64())
+            .map(photocraft_doc::LayerId)
+    } else {
+        None
+    };
+    app.move_blocked_doc = app.session.active().map(|st| st.doc.id);
+    app.move_viewport = app.current_viewport;
+}
+
 fn press_at(app: &mut PhotocraftApp, p: [f64; 2], mods: Mods, style: ConstrainStyle) -> (ToolAction, Option<Run>) {
     // The Cmd stand-in for a painting tool, or the Move tool itself.
     let standing_in = app.ui.tool != Tool::Move;
@@ -206,6 +255,7 @@ fn press_at(app: &mut PhotocraftApp, p: [f64; 2], mods: Mods, style: ConstrainSt
                     // locked-only hit can start a marquee (#4058).
 
                     if inside_selection(app, p) && crate::move_lock::blocked(app, standing_in) {
+                        record_refused_press(app, p);
                         return (ToolAction::Claimed, Some(Run::plain(Kind::Refused, p)));
                     }
                     // Shift preserves the base selection for its additive marquee.
@@ -213,29 +263,35 @@ fn press_at(app: &mut PhotocraftApp, p: [f64; 2], mods: Mods, style: ConstrainSt
                         let _ = app.run("select.deselectLayers", json!({}));
                     }
                 }
-                return marquee(app, mods.shift, auto, auto && !empty);
+                let locked = if empty { None } else { locked_hit_at(app, p) };
+                let res = marquee(app, mods.shift, auto, auto && !empty);
+                if let (Some(layer), Some(b)) = (locked, app.layer_box.as_mut()) {
+                    b.grabbed_layer = Some(layer);
+                }
+                return res;
             }
         } else {
-            let locked_hit = !auto
-                && probe.as_ref().ok().and_then(|v| v["layer"].as_u64()).is_some_and(|id| {
-                    app.session.active().is_some_and(|st| {
-                        let id = photocraft_doc::LayerId(id);
-                        let locks = st.doc.effective_locks(id);
-                        locks.position || locks.all || st.doc.is_background(id)
-                    })
-                });
+            let hit_layer = probe.as_ref().ok().and_then(|v| v["layer"].as_u64()).map(photocraft_doc::LayerId);
+            let locked_hit = !auto && hit_layer.is_some_and(|id| locked_layer(app, id));
+            let transfer_candidate = if app.session.documents().len() > 1 { locked_hit_at(app, p) } else { None };
+            let locked_hit = locked_hit || transfer_candidate.is_some();
             let active_before = app.session.active().and_then(|st| st.active_layer);
             let revealed = crate::layer_reveal::mark(app);
             let _ = app.run("layer.pickAt", json!({"x": p[0], "y": p[1], "target": target, "mode": mode, "moveAutoSelect": auto}));
             note_move_pick(app, active_before);
             crate::layer_reveal::file(app, revealed);
             if locked_hit {
-                return marquee(app, mods.shift, false, false);
+                let res = marquee(app, mods.shift, false, false);
+                if let Some(b) = app.layer_box.as_mut() {
+                    b.grabbed_layer = transfer_candidate.or(hit_layer);
+                }
+                return res;
             }
         }
     }
     // A locked layer: no drag, and the message once the pointer moves.
     if crate::move_lock::blocked(app, standing_in) {
+        record_refused_press(app, p);
         return (ToolAction::Claimed, Some(Run::plain(Kind::Refused, p)));
     }
     let gesture = MoveGesture::press(&style, Point::new(p[0], p[1]), mods, Directions::Eight);
@@ -298,6 +354,9 @@ impl CanvasTool for MoveRun<'_> {
     fn press(&mut self, pointer: &ToolPointer, cx: &mut ToolContext<'_>) -> ToolAction {
         let Some(space) = cx.space() else { return ToolAction::Ignored };
         let at = space.view.to_doc(pointer.pos);
+        // A new press starts a fresh gesture: blocked state left over from a lost release in another
+        // window is cleared here rather than consuming this press's events (#11971).
+        self.scope.with_app(|app| app.clear_move_blocked());
         let style = ConstrainStyle::new(f64::from(cx.slop), f64::from(cx.slop));
         // A double-click on type edits it, as in Photoshop (#1132): only the selected Move tool, not a ⌘ stand-in for
         // another tool, and not while a transform box is open (its double-click commits).
@@ -328,9 +387,18 @@ impl CanvasTool for MoveRun<'_> {
                 self.scope.with_app(|app| crate::move_box::drag_to(app, [at.x, at.y]));
             }
             Kind::Refused => {
-                if !std::mem::replace(&mut run.prompted, true) {
-                    self.scope.with_app(crate::move_lock::prompt);
-                }
+                self.scope.with_app(|app| {
+                    if crate::panels::layer_transfer::armed(app) {
+                        return;
+                    }
+                    if app.move_blocked_start.is_some_and(|s| s != [at.x, at.y]) {
+                        app.move_blocked_moved = true;
+                    }
+                    if !may_transfer(app) && !std::mem::replace(&mut run.prompted, true) {
+                        app.clear_move_blocked();
+                        crate::move_lock::prompt(app);
+                    }
+                });
             }
             Kind::Layers | Kind::Pixels => self.scope.with_app(|app| follow(run, app, pointer, cx, true)),
         }
@@ -344,7 +412,12 @@ impl CanvasTool for MoveRun<'_> {
             Kind::Marquee => {
                 crate::move_box::release(app, [at.x, at.y]);
             }
-            Kind::Refused => {}
+            Kind::Refused => {
+                let show_lock = app.move_blocked_moved || app.move_blocked_start.is_some_and(|s| s != [at.x, at.y]);
+                if show_lock && !crate::panels::layer_transfer::armed(app) && !run.prompted {
+                    crate::move_lock::prompt(app);
+                }
+            }
             Kind::Layers | Kind::Pixels => {
                 // A click (released without leaving its press) never moves: `follow` snaps nothing inside the press.
                 follow(&mut run, app, pointer, cx, false);
```

**What it does, hunk by hunk.**

1. `locked_layer` names what a Move drag cannot move in place: a layer locked all over, locked in position, or the Background.
2. `locked_hit_at` finds the topmost layer under the pointer that a drag toward another document would transfer, and no other: the lock a pick skips is exactly what travels (#11971).
3. `may_transfer` says when a refused press can still become a transfer - more than one document open, no transform in flight, and a refused layer or a selection - so the lock message waits while the drag can still reach another document's tab or tile.

**Checked by.** `cargo nextest run -p photocraft-ui-egui` (green on 2026-11-26); the move-tool trials cover the refusal and the transfer.

## `crates/ui-egui/src/lib.rs`

**Role.** The application state the framework keeps between frames.

**The code.**

```diff
diff --git a/crates/ui-egui/src/lib.rs b/crates/ui-egui/src/lib.rs
index 20d0f0d1c..35a47646a 100644
--- a/crates/ui-egui/src/lib.rs
+++ b/crates/ui-egui/src/lib.rs
@@ -368,6 +368,16 @@ pub struct PhotocraftApp {
     pub(crate) preset_art: panels::preset_browser::PresetArt,
     /// Move tool drag shown live (`move_ui`).
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
     /// Move tool › Show Highlight on Rollover: the last hover answer (`move_hover`).
     pub(crate) move_hover: Option<move_hover::Hover>,
     /// Smart Guides distances shown with ⌘ held over the canvas: the last answer (`move_distance`).
@@ -870,6 +880,12 @@ mod lock_layers_shortcut_tests;
 #[cfg(test)]
 mod pencil_tests;
 #[cfg(test)]
+mod sam3_canvas_tests;
+#[cfg(test)]
+mod sam3_options_bar_tests;
+#[cfg(test)]
+mod sam3_toolbar_tests;
+#[cfg(test)]
 mod smart_objects_menu_tests;
 #[cfg(test)]
 mod tool_shift_cycle_order_tests;
```

**What it does, hunk by hunk.**

1. The refused Move press is recorded: whether one happened, where it began, whether it moved, the layer it hit, the document, the window that owns the gesture and the viewport drawing this pass. No drag object exists for a refused press, so the transfer cannot happen without them (#11971).
2. The SAM Selection trials are declared as modules of the crate.

**Checked by.** `cargo nextest run -p photocraft-ui-egui` (green on 2026-11-26).

## `crates/ui-egui/src/panels/options_bar.rs`

**Role.** The options bar of the active tool.

**The code.**

```diff
diff --git a/crates/ui-egui/src/panels/options_bar.rs b/crates/ui-egui/src/panels/options_bar.rs
index 961b2318e..667e6d271 100644
--- a/crates/ui-egui/src/panels/options_bar.rs
+++ b/crates/ui-egui/src/panels/options_bar.rs
@@ -157,6 +157,10 @@ fn body(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
     // The bar shows the active tool's brush: switch it in before drawing, so a tool change swaps the brush with it
     // (Photoshop keeps one per tool).
     crate::paint_mouse::sync_tool_brush(app);
+    if tool == Tool::QuickSelection {
+        selection::quick_selection_segments(app, ui);
+        Divider::vertical().show(ui);
+    }
     if (tool.is_brushlike() && !matches!(tool, Tool::Brush | Tool::Pencil | Tool::MixerBrush | Tool::Eraser | Tool::AdjustmentBrush))
         || tool == Tool::QuickSelection
     {
```

**What it does, hunk by hunk.**

1. The Quick Selection bar draws its own New / Add / Subtract segments before the brush options, with a divider, so the mode sits where Photoshop puts it (#11837).

**Checked by.** `cargo nextest run -p photocraft-ui-egui` (green on 2026-11-26); `quick_selection_mode_bar_tests` covers it.

## `crates/ui-egui/src/state/tool_options.rs`

**Role.** The tool options, persisted with the document.

**The code.**

```diff
diff --git a/crates/ui-egui/src/state/tool_options.rs b/crates/ui-egui/src/state/tool_options.rs
index 7c699804a..d10e37dbe 100644
--- a/crates/ui-egui/src/state/tool_options.rs
+++ b/crates/ui-egui/src/state/tool_options.rs
@@ -127,6 +127,22 @@ pub struct ToolOptions {
     pub smudge_mode: photocraft_color::BlendMode,
     /// Quick Selection enhance edge.
     pub enhance_edge: bool,
+    /// Quick Selection's options-bar mode: 0 New (the default), 1 Add, 2 Subtract. Its own mode,
+    /// as in Photoshop: the other selection tools share `app.ui.selection_mode` (#11837).
+    #[serde(default = "default_quick_selection_mode")]
+    pub quick_selection_mode: u8,
+    /// SAM Selection: the open-vocabulary prompt (empty = click or box instead).
+    #[serde(default)]
+    pub sam3_prompt: String,
+    /// SAM Selection: the detector's score threshold, 0-1.
+    #[serde(default = "default_sam3_threshold")]
+    pub sam3_threshold: f32,
+    /// SAM Selection: refinement passes, 0-5.
+    #[serde(default = "default_sam3_refine")]
+    pub sam3_refine: u8,
+    /// SAM Selection: keep one mask per detected object instead of their union.
+    #[serde(default)]
+    pub sam3_individual: bool,
     /// Pen: a work path or a shape layer.
     pub vector_mode: VectorMode,
     /// Pen, Freeform Pen and Curvature Pen › Path Operations (#1397): how a new path combines with
@@ -336,6 +352,20 @@ fn default_move_target() -> String {
     "layer".into()
 }
 
+fn default_quick_selection_mode() -> u8 {
+    0
+}
+
+/// SAM Selection's threshold (`comfy_extras/nodes_sam3.py:105`).
+fn default_sam3_threshold() -> f32 {
+    0.5
+}
+
+/// SAM Selection's refinement passes (`comfy_extras/nodes_sam3.py:106`).
+fn default_sam3_refine() -> u8 {
+    2
+}
+
 fn default_pen_path_op() -> String {
     "newLayer".into()
 }
@@ -383,6 +413,7 @@ pub struct SampleAllLayers {
     pub magic_wand: bool,
     pub quick_selection: bool,
     pub object_selection: bool,
+    pub sam3_select: bool,
     pub paint_bucket: bool,
     pub magic_eraser: bool,
     pub spot_healing: bool,
@@ -400,6 +431,7 @@ impl Default for SampleAllLayers {
             magic_wand: false,
             quick_selection: false,
             object_selection: false,
+            sam3_select: false,
             paint_bucket: false,
             magic_eraser: false,
             spot_healing: false,
@@ -425,6 +457,7 @@ impl SampleAllLayers {
             Tool::MagicWand => &mut self.magic_wand,
             Tool::QuickSelection => &mut self.quick_selection,
             Tool::ObjectSelection => &mut self.object_selection,
+            Tool::Sam3Select => &mut self.sam3_select,
             Tool::PaintBucket => &mut self.paint_bucket,
             Tool::MagicEraser => &mut self.magic_eraser,
             Tool::SpotHealing => &mut self.spot_healing,
@@ -499,6 +532,11 @@ impl Default for ToolOptions {
             finger_painting: false,
             smudge_mode: photocraft_color::BlendMode::Normal,
             enhance_edge: false,
+            quick_selection_mode: 0,
+            sam3_prompt: String::new(),
+            sam3_threshold: default_sam3_threshold(),
+            sam3_refine: default_sam3_refine(),
+            sam3_individual: false,
             vector_mode: VectorMode::Path,
             pen_path_op: default_pen_path_op(),
             shape_path_op: default_shape_path_op(),
```

**What it does, hunk by hunk.**

1. `quick_selection_mode` (0 New, 1 Add, 2 Subtract) is its own mode, as in Photoshop, while the other selection tools keep sharing `selection_mode` (#11837).
2. The SAM Selection options: `sam3_prompt` (empty means click or box), `sam3_threshold` (default 0.5), `sam3_refine` (default 2) and `sam3_individual` (one mask per object instead of their union).

**Checked by.** `cargo nextest run -p photocraft-ui-egui` (green on 2026-11-26).

## `crates/ui-egui/src/tools.rs`

**Role.** The tool modules and their trials.

**The code.**

```diff
diff --git a/crates/ui-egui/src/tools.rs b/crates/ui-egui/src/tools.rs
index dd163613d..4c2cdcea7 100644
--- a/crates/ui-egui/src/tools.rs
+++ b/crates/ui-egui/src/tools.rs
@@ -81,6 +81,8 @@ mod object_selection_mode_tests;
 mod path_selection_ctrl_direct_tests;
 #[cfg(test)]
 mod path_selection_shape_over_saved_path_tests;
+#[cfg(test)]
+mod quick_selection_mode_bar_tests;
 
 #[cfg(test)]
 mod art_history_reset_tip_tests;
```

**What it does, hunk by hunk.**

1. `quick_selection_mode_bar_tests` is declared as a module of the crate.

**Checked by.** `cargo nextest run -p photocraft-ui-egui` (green on 2026-11-26).

## `crates/ui-egui/src/tools/retouch_ui/bar/heal.rs`

**Role.** The options of the healing tools.

**The code.**

```diff
diff --git a/crates/ui-egui/src/tools/retouch_ui/bar/heal.rs b/crates/ui-egui/src/tools/retouch_ui/bar/heal.rs
index 3ba2fc96c..9a0807e65 100644
--- a/crates/ui-egui/src/tools/retouch_ui/bar/heal.rs
+++ b/crates/ui-egui/src/tools/retouch_ui/bar/heal.rs
@@ -159,5 +159,13 @@ pub(super) fn clone(ui: &mut egui::Ui, app: &mut PhotocraftApp) -> Option<RangeA
     None
 }
 
+/// The options of LaMa Cleaner.
+pub(super) fn lama_cleaner(ui: &mut egui::Ui, app: &mut PhotocraftApp) -> Option<RangeAlert> {
+    let mut size = app.session.tools.brush.size.max(1.0);
+    let alert = number(tl!("Size:"), &mut size, 1.0..=500.0).unit("px").whole().popup_slider().show(ui);
+    app.session.tools.brush.size = size;
+    alert.out_of_range
+}
+
 #[allow(dead_code)]
 fn _json_is_not_used(_: Json) {}
```

**What it does, hunk by hunk.**

1. `lama_cleaner` draws the LaMa Cleaner's one option, the brush size, as a number field with a slider and a unit, and reports a value out of range as the other bars do.

**Checked by.** `cargo nextest run -p photocraft-ui-egui` (green on 2026-11-26).

## `crates/ui-egui/src/tools/tool_reset.rs`

**Role.** What a tool's option reset puts back.

**The code.**

```diff
diff --git a/crates/ui-egui/src/tools/tool_reset.rs b/crates/ui-egui/src/tools/tool_reset.rs
index f324663a3..4ca6e3b39 100644
--- a/crates/ui-egui/src/tools/tool_reset.rs
+++ b/crates/ui-egui/src/tools/tool_reset.rs
@@ -213,8 +213,9 @@ pub fn option_keys(tool: Tool) -> &'static [&'static str] {
         Tool::Lasso | Tool::PolygonLasso => reset!(feather, anti_alias),
         Tool::MagneticLasso => reset!(feather, anti_alias, magnetic),
         Tool::MagicWand => reset!(tolerance, anti_alias, contiguous),
-        Tool::QuickSelection => reset!(enhance_edge),
+        Tool::QuickSelection => reset!(enhance_edge, quick_selection_mode),
         Tool::ObjectSelection => reset!(enhance_edge, object_finder, object_subtract, object_finder_mode, object_hard_edge),
+        Tool::Sam3Select => reset!(sam3_prompt, sam3_threshold, sam3_refine, sam3_individual),
         Tool::Crop => reset!(crop),
         Tool::PerspectiveCrop => reset!(perspective_crop, perspective_crop_grid),
         Tool::Eyedropper => reset!(eyedropper),
@@ -311,6 +312,7 @@ pub fn option_keys(tool: Tool) -> &'static [&'static str] {
         | Tool::Count
         | Tool::RotateView
         | Tool::HistoryBrush
+        | Tool::LamaCleaner
         | Tool::PathSelection
         | Tool::DirectSelection
         | Tool::SliceSelect
```

**What it does, hunk by hunk.**

1. Quick Selection's reset now includes `quick_selection_mode`, since the bar's mode is an option of the tool.
2. SAM Selection resets its prompt, threshold, refinement passes and per-object flag.
3. `Tool::LamaCleaner` joins the tools whose options are reset with the brush's own, so a reset leaves its size with the brush it uses.

**Checked by.** `cargo nextest run -p photocraft-ui-egui` (green on 2026-11-26).
