# The fork's code in photocraft after the upstream merge

The fork's own code in this repository after the upstream merge: every file whose content in the
merge commit differs from both of the merge's parents, with the role of the file, the code the
fork holds, what each hunk does, and the command that catches a revert.


Every diff below is `git diff <upstream> <merge> -- <path>` in the repository that owns the
file: it is the fork's own code as the merged tree holds it. The upstream lines that forced
each of these answers are quoted in the upstream-code document of the same repository.

## The isolation test

A file is the fork's own when its content in the merge commit `9ecee7176` differs from **both**
parents: `git show b86838240:<path>` (the fork before the merge), `git show e22facf45:<path>`
(the upstream commit) and `git show 9ecee7176:<path>` (the merged tree). The command that
lists the set is:

```powershell
git -C <repo> diff --name-only b86838240 9ecee7176          # every file the merge touched
git -C <repo> show 9ecee7176:<path>                   # the merged tree's copy
git -C <repo> show b86838240:<path>                      # the fork's copy
git -C <repo> show e22facf45:<path>                # the upstream copy
```

## `Cargo.lock`

**Role.** The workspace lock file: the resolved graph of every crate the workspace builds.

**The fork's code.**

```diff
diff --git a/Cargo.lock b/Cargo.lock
index 48b6a650a..ccfce1461 100644
--- a/Cargo.lock
+++ b/Cargo.lock
@@ -6723,6 +6723,7 @@ name = "photocraft-engine-paint"
 version = "0.23.1"
 dependencies = [
  "craft-cms 0.1.0 (git+https://github.com/storytold/craft-libs?rev=3656e8aaa41cba15ec5d11b7c77045cb72dec163)",
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

**What each hunk does.**

1. The merged tree gives `photocraft-engine-paint` the two dependencies the moved module needs: `craft-image-codecs` (it encodes its crop and mask as PNG) and `tempfile` (it writes a temporary directory).

**If this were reverted / how it is checked.** Superseded: the lock file regenerates, so it is not hand-edited. It is listed because it is part of the merged tree.

## `crates/automation/src/server.rs`

**Role.** The automation server: the MCP commands the application answers over its local socket.

**The fork's code.**

```diff
diff --git a/crates/automation/src/server.rs b/crates/automation/src/server.rs
index 8e13589d9..d86607d88 100644
--- a/crates/automation/src/server.rs
+++ b/crates/automation/src/server.rs
@@ -1531,6 +1531,37 @@ mod tests {
         assert_eq!(docs, 0, "the step after the budget ran out did not run");
     }
 
+    /// rmcp may drop a cancelled request's future without polling it again; its job must still
+    /// stop and leave the document unchanged (#8129).
+    #[tokio::test(flavor = "multi_thread")]
+    async fn dropping_a_waited_command_cancels_its_job() {
+        let mcp = PhotocraftMcp::headless();
+        mcp.headless_op(|h| {
+            h.command_run("file.new", json!({"width": 6000, "height": 6000}))?;
+            h.command_run("filter.noise.addNoise", json!({"amount": 50}))
+        })
+        .await
+        .unwrap()
+        .unwrap();
+        let before = mcp.headless_op(|h| h.inspect(None)).await.unwrap().unwrap();
+        let mut waiter = Waiter::new(None);
+        // Per-pixel noise is seconds of work on any machine, so the job is still running when
+        // the waiter is dropped (a GPU blur can finish inside the 100 ms window).
+        let waited = mcp.headless_wait("filter.noise.addNoise".into(), json!({"amount": 50}), &mut waiter, (0, 1));
+        assert!(tokio::time::timeout(std::time::Duration::from_millis(100), waited).await.is_err(), "the noise ended before it was dropped");
+        let t = std::time::Instant::now();
+        loop {
+            let (running, cancelled) =
+                mcp.headless_op(|h| Ok((h.session.has_jobs(), h.session.jobs_with_recent().iter().any(|j| j.state == "cancelled")))).await.unwrap().unwrap();
+            if !running {
+                assert!(cancelled, "the job ended without being cancelled");
+                break;
+            }
+            assert!(t.elapsed().as_secs() < 30, "the dropped job kept running");
+            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
+        }
+        assert_eq!(mcp.headless_op(|h| h.inspect(None)).await.unwrap().unwrap(), before);
+    }
     /// Dropped while its start still waits for the session lock (held here as a long synchronous
     /// command would): the job that starts afterwards is cancelled too, not left running.
     #[tokio::test(flavor = "multi_thread")]
```

**What each hunk does.**

1. A trial drops a waited command and asserts that its job is cancelled and that the document is unchanged. The server may drop a cancelled request's future without polling it again, so the job has to stop on its own.

**If this were reverted / how it is checked.** `cargo nextest run -p photocraft-automation` (green as part of the 15,460-trial workspace run of 2026-11-26).

## `crates/engine/tests/it/main.rs`

**Role.** The engine's screen-trial module list.

**The fork's code.**

```diff
diff --git a/crates/engine/tests/it/main.rs b/crates/engine/tests/it/main.rs
index 3feb68561..c50a8c30c 100644
--- a/crates/engine/tests/it/main.rs
+++ b/crates/engine/tests/it/main.rs
@@ -181,7 +181,6 @@ mod shortcuts_headless;
 mod smart_clipboard_sources;
 mod smart_duplicates;
 mod smart_filter_canvas_geometry;
-mod smart_filter_stages;
 mod smart_object_tones;
 mod smart_psd;
 mod smart_rerender_fallback;
```

**What each hunk does.**

1. `mod smart_filter_stages` is not listed here any more: the trial module moved to `crates/engine/tests/global_state`, where a trial that asserts on process-wide state takes its turn instead of sharing a process with 1,200 parallel trials.

**If this were reverted / how it is checked.** `cargo xtask test-corpus` (670 passed / 0 failed / 11 ignored, exit 0 on 2026-11-26) is the run that fails when this module sits in `tests/it`.

## `crates/ui-egui/src/app/framework.rs`

**Role.** The framework's held states: the checked state of every command the menus show.

**The fork's code.**

```diff
diff --git a/crates/ui-egui/src/app/framework.rs b/crates/ui-egui/src/app/framework.rs
index 8b8489f90..4ff6fc517 100644
--- a/crates/ui-egui/src/app/framework.rs
+++ b/crates/ui-egui/src/app/framework.rs
@@ -627,12 +627,24 @@ pub(crate) struct HeldStates {
 
 /// The tool commands: the current tool's row is checked.
 fn build_tool_states(app: &PhotocraftApp) -> SourceStates {
-    Tool::ALL.iter().map(|&tool| (CommandRef::from(crate::panels::tool_commands::command_of(tool)), CommandState::new().checked(app.ui.tool == tool))).collect()
+    let lang = crate::i18n::current();
+    Tool::ALL
+        .iter()
+        .map(|&tool| {
+            let mut state = CommandState::new().checked(app.ui.tool == tool);
+            if lang != crate::i18n::Lang::EN {
+                state = state.title(crate::i18n::tr(lang, tool.label()));
+            }
+            (CommandRef::from(crate::panels::tool_commands::command_of(tool)), state)
+        })
+        .collect()
 }
 
-/// [`build_tool_states`], derived from the tool.
+/// [`build_tool_states`], derived from the tool and the UI language.
 fn tool_states(app: &PhotocraftApp, name: &'static str) -> Arc<SourceStates> {
-    derived_states(app, name, &(app.ui.tool as u64), || build_tool_states(app))
+    let lang = crate::i18n::current().code();
+    let lang_idx = crate::i18n::LANGUAGES.iter().position(|l| l.code == lang).unwrap_or(0) as u64;
+    derived_states(app, name, &(app.ui.tool as u64, lang_idx), || build_tool_states(app))
 }
 
 /// Brings the held states up to date for the coming frame. A source whose states are the `Arc` it last gave costs a
```

**What each hunk does.**

1. A tool command's title is the tool's label in the interface language when the interface is not English, so the menu row is written in the language the application is in.
2. The derived-state key carries the language index as well as the tool, so a change of language re-derives the states.

**If this were reverted / how it is checked.** `cargo nextest run -p photocraft-ui-egui` (green as part of the 15,460-trial workspace run of 2026-11-26).

## `crates/ui-egui/src/canvas/canvas_tool_menu.rs`

**Role.** The canvas right-click menu: the rows a tool shows beside the selection.

**The fork's code.**

```diff
diff --git a/crates/ui-egui/src/canvas/canvas_tool_menu.rs b/crates/ui-egui/src/canvas/canvas_tool_menu.rs
index 80255432b..4d869dd7e 100644
--- a/crates/ui-egui/src/canvas/canvas_tool_menu.rs
+++ b/crates/ui-egui/src/canvas/canvas_tool_menu.rs
@@ -1033,6 +1033,7 @@ pub fn applies(tool: Tool) -> bool {
             | Tool::MagneticLasso
             | Tool::MagicWand
             | Tool::ObjectSelection
+            | Tool::Sam3Select
             // Its brush is set from the options bar's chip; the canvas right-click is the
             // selection menu, as in Photoshop (#7268).
             | Tool::QuickSelection
@@ -1182,7 +1183,7 @@ pub fn rows(menu: &CanvasToolMenu) -> &'static [Row] {
                 SAMPLER_MENU
             } else if menu.tool == Tool::Count {
                 COUNT_MENU
-            } else if matches!(menu.tool, Tool::RedEye | Tool::ObjectSelection | Tool::Type) {
+            } else if matches!(menu.tool, Tool::RedEye | Tool::ObjectSelection | Tool::Sam3Select | Tool::Type) {
                 if menu.has_selection { MEASURED_SELECTION_MENU } else { COUNT_MENU }
             } else if menu.tool == Tool::Eyedropper {
                 EYEDROPPER_MENU
```

**What each hunk does.**

1. `Tool::Sam3Select` joins the tools whose right-click opens the selection menu.
2. The measured-selection menu is the one SAM Selection shows when there is a selection, and the count menu when there is none.

**If this were reverted / how it is checked.** `cargo nextest run -p photocraft-ui-egui` (green as part of the 15,460-trial workspace run of 2026-11-26).

## `crates/ui-egui/src/canvas/tests.rs`

**Role.** The canvas's GPU trials.

**The fork's code.**

```diff
diff --git a/crates/ui-egui/src/canvas/tests.rs b/crates/ui-egui/src/canvas/tests.rs
index e2acefdd9..5638849b1 100644
--- a/crates/ui-egui/src/canvas/tests.rs
+++ b/crates/ui-egui/src/canvas/tests.rs
@@ -1190,3 +1190,31 @@ mod gpu_tests {
         assert_eq!(texture_key(canvas_display(&app, &doc, Some(1)).0.as_deref(), &doc), texture_key(canvas_display(&app, &doc, Some(4)).0.as_deref(), &doc));
     }
 }
+
+/// Window › Arrange › Tile All Vertically with two documents open as tabs: the document area splits into one pane per
+/// document and every document shows. The framework's tile hands each document to a floating window instead, and a
+/// floating document window is a window of its own here: the document area went empty and windows covered the screen.
+#[test]
+fn tile_all_vertically_shows_every_document_in_its_own_pane() {
+    for layout in ["tileAllVertically", "tileAllHorizontally"] {
+        let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), crate::Services::default());
+        for name in ["one", "two"] {
+            app.run("file.new", json!({"width": 400, "height": 300, "name": name})).unwrap();
+        }
+        app.sync_views();
+        let app_ctx = app.ctx().clone();
+        crate::menus::invoke(&mut app, &app_ctx, &format!("window.arrange.{layout}"), json!({})).unwrap();
+        assert_eq!(app.ui.document_windows.docked.len(), 2, "{layout}: one pane per document");
+        assert!(app.ui.document_windows.floating.is_empty(), "{layout}: no document goes to a window of its own");
+        let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(900.0, 600.0));
+        let cells = super::document_area::arranged_cells(&app, rect).unwrap_or_else(|| panic!("{layout}: two panes"));
+        assert_eq!(cells.len(), 2, "{layout}: both documents show");
+        let shown: Vec<usize> = cells.iter().map(|(document, _)| *document).collect();
+        assert_eq!(shown, vec![0, 1], "{layout}: each pane shows its own document");
+        if layout == "tileAllVertically" {
+            assert!(cells[0].1.max.x <= cells[1].1.min.x, "{layout}: side by side, {:?}", cells);
+        } else {
+            assert!(cells[0].1.max.y <= cells[1].1.min.y, "{layout}: stacked, {:?}", cells);
+        }
+    }
+}
```

**What each hunk does.**

1. A trial opens two documents and asks for Window > Arrange > Tile All Vertically and Tile All Horizontally, then asserts that each document has a pane of its own, that no document goes to a window of its own, and that the panes lie side by side or stacked. The framework's tile hands every document to a floating window, which leaves the document area empty and covers the screen with windows.

**If this were reverted / how it is checked.** `cargo nextest run -p photocraft-ui-egui` (green as part of the 15,460-trial workspace run of 2026-11-26).

## `crates/ui-egui/src/dialogs/file_ui/print.rs`

**Role.** The print dialog: the printer, the paper and the settings it opens with.

**The fork's code.**

```diff
diff --git a/crates/ui-egui/src/dialogs/file_ui/print.rs b/crates/ui-egui/src/dialogs/file_ui/print.rs
index 613fdb9f9..361fa88b9 100644
--- a/crates/ui-egui/src/dialogs/file_ui/print.rs
+++ b/crates/ui-egui/src/dialogs/file_ui/print.rs
@@ -145,6 +145,8 @@ pub fn open_print(app: &mut PhotocraftApp) -> DialogId {
             fields.set(k, v);
         }
     }
+    // The panel's own settings come from the same remembered print: the loop above skips them, so the
+    // dialog would open with no printer at all.
     write_settings(&mut fields, &state.settings);
     let sections = sections(&fields);
     app.framework.dialogs.open(HostedPrint::new(state, fields, sections).json_form(|fields, listing| {
```

**What each hunk does.**

1. A comment records that the panel's own settings come from the same remembered print: the loop above skips them, so the dialog would open with no printer at all.

**If this were reverted / how it is checked.** `cargo nextest run -p photocraft-ui-egui` (green as part of the 15,460-trial workspace run of 2026-11-26).

## `crates/ui-egui/src/lib.rs`

**Role.** The application state: what the framework keeps between frames.

**The fork's code.**

```diff
diff --git a/crates/ui-egui/src/lib.rs b/crates/ui-egui/src/lib.rs
index 8171a5400..90190c63b 100644
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
@@ -866,6 +876,12 @@ mod lock_layers_shortcut_tests;
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

**What each hunk does.**

1. The Move tool's refused press is recorded: where it began, the layer it hit, the document and the window that owns the gesture. No drag object exists for a refused press, so the press cannot become a cross-document transfer without them.
2. The SAM Selection trials are declared as modules of the crate.

**If this were reverted / how it is checked.** `cargo nextest run -p photocraft-ui-egui` (green as part of the 15,460-trial workspace run of 2026-11-26).

## `crates/ui-egui/src/menus/view_cmds/arrange.rs`

**Role.** The Window > Arrange commands: how the document area is split and how documents are tiled.

**The fork's code.**

```diff
diff --git a/crates/ui-egui/src/menus/view_cmds/arrange.rs b/crates/ui-egui/src/menus/view_cmds/arrange.rs
index 6b34351e4..b669cee42 100644
--- a/crates/ui-egui/src/menus/view_cmds/arrange.rs
+++ b/crates/ui-egui/src/menus/view_cmds/arrange.rs
@@ -67,7 +67,22 @@ pub(super) fn arrange(app: &mut PhotocraftApp, k: &str, p: &Value) -> Result<Val
         return float_in_window(app, p);
     }
     let space = space(app);
-    let arranged = run_arrange(&mut app.ui.document_windows, command, &space);
+    let arranged = match command {
+        // A document here is a tab of the window's document area, and a floating document window is a window of its
+        // own: the framework's tile hands every document to a floating window, which leaves the document area empty
+        // and covers the screen with windows. Tiling therefore splits the area into one pane per document, as the
+        // n-up layouts do, so every document shows side by side (Vertically) or stacked (Horizontally).
+        ArrangeCommands::TileAllVertically | ArrangeCommands::TileAllHorizontally => {
+            let active = app.session.active().map(|st| st.doc.id);
+            let windows = &mut app.ui.document_windows;
+            windows.consolidate(active.as_ref());
+            let documents = windows.docked.first().map_or(0, |group| group.documents.len());
+            windows.split_into(documents);
+            windows.arrangement = Some(command.arrangement());
+            Arranged::Windows
+        }
+        _ => run_arrange(&mut app.ui.document_windows, command, &space),
+    };
     crate::canvas::document_area::apply_arranged(app, arranged);
     if matches!(command, ArrangeCommands::NewWindowForDocument) {
         keep_own_view(app);
```

**What each hunk does.**

1. Tile All Vertically and Tile All Horizontically are handled here: the documents are consolidated into the window's document area and split into one pane per document, and the arrangement is recorded. The framework's tile hands every document to a floating window, which leaves the document area empty and covers the screen with windows; the n-up layouts split the area instead, so every document shows.

**If this were reverted / how it is checked.** `cargo nextest run -p photocraft-ui-egui -E 'test(tile_all_vertically)'` and `cargo nextest run -p photocraft-ui-egui` (green as part of the 15,460-trial workspace run of 2026-11-26).

## `crates/ui-egui/src/state.rs`

**Role.** The tool list and the state built from it.

**The fork's code.**

```diff
diff --git a/crates/ui-egui/src/state.rs b/crates/ui-egui/src/state.rs
index 78f2ae472..c75811c8c 100644
--- a/crates/ui-egui/src/state.rs
+++ b/crates/ui-egui/src/state.rs
@@ -629,7 +629,7 @@ mod tests {
         assert!(!Tool::RedEye.is_brushlike());
         assert_eq!(Tool::from_name("patternStamp"), Some(Tool::PatternStamp));
         assert_eq!(Tool::from_name("Pattern Stamp Tool"), Some(Tool::PatternStamp));
-        assert_eq!(Tool::ALL.len(), 70);
+        assert_eq!(Tool::ALL.len(), 72);
     }
 
     fn vector_tool_names_parse() {
```

**What each hunk does.**

1. The trial that counts the tools expects 72: the fork's tools and upstream's tools are counted together, and the array and the trial have to agree.

**If this were reverted / how it is checked.** `cargo nextest run -p photocraft-ui-egui -E 'test(tool)'` (green as part of the 15,460-trial workspace run of 2026-11-26).

## `crates/ui-egui/src/tools.rs`

**Role.** The tool modules and their trials.

**The fork's code.**

```diff
diff --git a/crates/ui-egui/src/tools.rs b/crates/ui-egui/src/tools.rs
index 4bae235d4..21943d64f 100644
--- a/crates/ui-egui/src/tools.rs
+++ b/crates/ui-egui/src/tools.rs
@@ -79,6 +79,8 @@ mod object_selection_mode_tests;
 mod path_selection_ctrl_direct_tests;
 #[cfg(test)]
 mod path_selection_shape_over_saved_path_tests;
+#[cfg(test)]
+mod quick_selection_mode_bar_tests;
 
 #[cfg(test)]
 mod art_history_reset_tip_tests;
```

**What each hunk does.**

1. The quick-selection mode bar's trials are declared as a module of the crate.

**If this were reverted / how it is checked.** `cargo nextest run -p photocraft-ui-egui` (green as part of the 15,460-trial workspace run of 2026-11-26).

## `crates/ui-egui/src/tools/retouch_ui.rs`

**Role.** The retouch tools: what a stroke sends to the engine (quick selection, object selection, SAM Selection, the LaMa Cleaner).

**The fork's code.**

```diff
diff --git a/crates/ui-egui/src/tools/retouch_ui.rs b/crates/ui-egui/src/tools/retouch_ui.rs
index f472df2bb..c158aac1f 100644
--- a/crates/ui-egui/src/tools/retouch_ui.rs
+++ b/crates/ui-egui/src/tools/retouch_ui.rs
@@ -264,6 +264,18 @@ pub(crate) fn finish_stroke_with(app: &mut PhotocraftApp, tool: Tool, points: &[
             Some(cp) => cp,
             None => return false,
         },
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
         Tool::QuickSelection => {
             // Indexed Color and Bitmap: the reference application's alert with OK (#12934), and nothing runs.
             if let Some(why) = app.session.active().and_then(|st| photocraft_engine::smartselect_cmds::quick_mode_refusal(st.doc.mode)) {
@@ -271,11 +283,19 @@ pub(crate) fn finish_stroke_with(app: &mut PhotocraftApp, tool: Tool, points: &[
                 return true;
             }
             let size = app.session.tools.brush.size;
-            let mode = if mods.alt { "subtract" } else { "add" };
+            // Its own New / Add / Subtract bar, with ⇧ adding and ⌥ subtracting (#11837).
+            let had_selection = crate::tool_feedback::has_selection(app);
+            let bar = crate::tool_feedback::bar_mode(app, Tool::QuickSelection);
+            let mode = crate::tool_feedback::mode_name(crate::tool_feedback::selection_op(Tool::QuickSelection, bar, mods, had_selection));
             let xy: Vec<[f64; 2]> = points.iter().map(|q| [q[0], q[1]]).collect();
             let (sample_all, enhance) = (o.sample_all.quick_selection, o.enhance_edge);
             let r = app.run("select.quick", json!({"points": xy, "size": size, "mode": mode, "enhanceEdge": enhance, "sampleAllLayers": sample_all}));
+            let changed = matches!(r, Ok(ref v) if v.get("changed").and_then(Value::as_bool) == Some(true));
             report_smart_result(app, r, tl!("Quick Selection found nothing there"));
+            // New flips to Add after its first stroke on an empty selection, as in Photoshop.
+            if app.ui.tool_options.quick_selection_mode == 0 && changed && !had_selection {
+                app.ui.tool_options.quick_selection_mode = 1;
+            }
             return true;
         }
         _ => return false,
@@ -682,6 +702,59 @@ pub fn finish_object_selection(app: &mut PhotocraftApp, start: [f64; 2], end: [f
 /// The "Don't show again" of the alert for an Object Selection rectangle that finds nothing.
 const OBJECT_NOT_FOUND_PREF: &str = "objectSelection.notFound";
 
+/// The `select.sam3` params for the gesture from `start` to `end`. With a text prompt typed the
+/// prompt wins (the engine's text path covers the whole document, as the Detect button's does);
+/// otherwise a click (or a sub-2 px nudge) is a point prompt and a drag a box prompt.
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
+        // A press that never moved with a prompt typed: the text prompt, as Detect sends it.
+        json!({"prompt": o.sam3_prompt.trim()})
+    } else if !o.sam3_prompt.trim().is_empty() {
+        // A drag with a prompt typed is the text prompt; the engine's text path covers the whole
+        // document, so the box is not sent beside it (it would be ignored).
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
+
+/// SAM Selection: a click segments the object under the pointer, a drag segments inside the box
+/// (`SAM3_Detect`'s point and box prompts). The options bar holds the text prompt instead; the
+/// preview is the same marching-ants rectangle Object Selection drags. Modes follow the other
+/// selection tools (⇧⌥ intersects, ⇧ adds, ⌥ subtracts, otherwise the options-bar mode).
+pub fn finish_sam3_selection(app: &mut PhotocraftApp, start: [f64; 2], end: [f64; 2], mods: Mods) {
+    let params = sam3_select_params(app, start, end, mods);
+    let r = app.run("select.sam3", params);
+    report_smart_result(app, r, tl!("SAM found nothing there — try a tighter box"));
+}
+
 /// Reports a smart-selection command result in the status bar. Engine errors are already
 /// reported by [`PhotocraftApp::run`], but an empty result (`changed: false`) is a silent
 /// success that looks like a dead button (Select Subject finding nothing on a busy photo).
```

**What each hunk does.**

1. The LaMa Cleaner sends its own command: the stroke's points and the brush's size, hardness and spacing (hardness and spacing as percentages).
2. The quick selection takes its mode from the tool's own New / Add / Subtract bar, with Shift adding and Alt subtracting, instead of a fixed mode from the modifiers alone.
3. After its first stroke on an empty selection the quick selection flips from New to Add, as the reference application does; the flip needs the command's `changed` answer and that there was no selection before the stroke.
4. `sam3_select_params` builds the `select.sam3` parameters: a typed prompt wins (the engine's text path covers the whole document), otherwise a click is a point prompt and a drag a box prompt, and the mode follows the modifiers (Shift+Alt intersects, Shift adds, Alt subtracts) or the options bar.
5. `finish_sam3_selection` runs the command and reports an empty result in the status bar, as the other selection tools do.

**If this were reverted / how it is checked.** `cargo nextest run -p photocraft-ui-egui` (green as part of the 15,460-trial workspace run of 2026-11-26); the SAM trials are `sam3_canvas_tests`, `sam3_options_bar_tests` and `sam3_toolbar_tests`.

## `crates/ui-egui/src/tools/type_tool/fonts.rs`

**Role.** The type tool's font list: which families are shown and under which name.

**The fork's code.**

```diff
diff --git a/crates/ui-egui/src/tools/type_tool/fonts.rs b/crates/ui-egui/src/tools/type_tool/fonts.rs
index 4bc3d3890..3ee33eeb0 100644
--- a/crates/ui-egui/src/tools/type_tool/fonts.rs
+++ b/crates/ui-egui/src/tools/type_tool/fonts.rs
@@ -89,6 +89,13 @@ pub(super) fn preview_text(app: &PhotocraftApp) -> Option<String> {
     (!line.is_empty()).then_some(line)
 }
 
+/// Show the families by their English names: the preference decides, except in a Japanese
+/// interface, which lists them by their Japanese names (ＭＳ Ｐ明朝, not MS PMincho) as Photoshop
+/// does.
+pub(super) fn english_font_names(preference: bool, language: &str) -> bool {
+    preference && !language.starts_with("ja")
+}
+
 /// The font picker's most recent families the settings may hold, whatever the preference says.
 const MAX_RECENT_FONTS: usize = 50;
 
@@ -121,7 +128,7 @@ pub(crate) fn font_state(app: &PhotocraftApp) -> FontSourceState {
         FontPreview::ExtraLarge => FontPreviewSize::ExtraLarge,
         FontPreview::Huge => FontPreviewSize::Huge,
     };
-    settings.names_in_english = prefs.type_.show_font_names_in_english;
+    settings.names_in_english = english_font_names(prefs.type_.show_font_names_in_english, crate::i18n::current().code());
     let catalog = photocraft_text::fonts::catalog();
     let served = photocraft_text::served::families().into_iter().filter(|f| catalog.family(f).is_none()).collect();
     FontSourceState { before: settings.clone(), settings, filters: app.ui.font_filters.clone(), preview_text: preview_text(app), served }
```

**What each hunk does.**

1. `english_font_names` decides between the preference and the interface language: a Japanese interface lists the families by their Japanese names (ＭＳ Ｐ明朝, not MS PMincho), as the reference application does, and every other language follows the preference.

**If this were reverted / how it is checked.** `cargo nextest run -p photocraft-ui-egui` (green as part of the 15,460-trial workspace run of 2026-11-26).

