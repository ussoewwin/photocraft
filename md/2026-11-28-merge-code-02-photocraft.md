# The application's kept code (photocraft)

21 files differ from both parents of merge `581eb807b`: each is code the fork keeps that upstream
does not have in that form. Six more files are the repair commit `359743c5f`. Each file below
carries its role, the diff verbatim, one numbered item per hunk, and the check that catches a
revert.

### `Cargo.toml`

**Role.** the workspace's dependency pins: the framework revision every crate builds against, and the `[patch.crates-io]` entries that redirect the PDF and vector crates to the same tree.

**The diff, verbatim** (`git diff 581eb807b^2 581eb807b -- Cargo.toml`):

```diff
diff --git a/Cargo.toml b/Cargo.toml
index fdbd10ef8..47778e57f 100644
--- a/Cargo.toml
+++ b/Cargo.toml
@@ -12,21 +12,21 @@ rust-version = "1.95"
 [workspace.dependencies]
 # The craft framework through its facade (`craft::ui::…`). The egui `[patch]` below must match
 # `craft::egui_fork::REV` (checked by `cargo xtask ci`).
-craft = { git = "https://github.com/storytold/craft-libs", rev = "3eb5b528b08211844bc5c95c5a4fd169894d52ff", default-features = false }
-craft-tracked = { git = "https://github.com/storytold/craft-libs", rev = "3eb5b528b08211844bc5c95c5a4fd169894d52ff" }
-craft-ui-macros = { git = "https://github.com/storytold/craft-libs", rev = "3eb5b528b08211844bc5c95c5a4fd169894d52ff" }
+craft = { git = "https://github.com/storytold/craft-libs", rev = "51fcc3b0413af193db952df28dffae2321745703", default-features = false }
+craft-tracked = { git = "https://github.com/storytold/craft-libs", rev = "51fcc3b0413af193db952df28dffae2321745703" }
+craft-ui-macros = { git = "https://github.com/storytold/craft-libs", rev = "51fcc3b0413af193db952df28dffae2321745703" }
 # internal
 photocraft-geom = { path = "crates/geom" }
 photocraft-color = { path = "crates/color" }
 photocraft-raster = { path = "crates/raster" }
 photocraft-doc = { path = "crates/doc" }
 photocraft-ops = { path = "crates/ops" }
-craft-plist = { git = "https://github.com/storytold/craft-libs", rev = "3eb5b528b08211844bc5c95c5a4fd169894d52ff" }
-craft-psd = { git = "https://github.com/storytold/craft-libs", rev = "3eb5b528b08211844bc5c95c5a4fd169894d52ff" }
-photocraft-codecs = { package = "craft-image-codecs", git = "https://github.com/storytold/craft-libs", rev = "3eb5b528b08211844bc5c95c5a4fd169894d52ff" }
-photocraft-formats = { package = "craft-file-formats", git = "https://github.com/storytold/craft-libs", rev = "3eb5b528b08211844bc5c95c5a4fd169894d52ff", features = ["raster-codecs", "pdf", "svg", "ora", "kra", "xcf", "pdn", "raw", "psd", "affinity", "pcraft"] }
+craft-plist = { git = "https://github.com/storytold/craft-libs", rev = "51fcc3b0413af193db952df28dffae2321745703" }
+craft-psd = { git = "https://github.com/storytold/craft-libs", rev = "51fcc3b0413af193db952df28dffae2321745703" }
+photocraft-codecs = { package = "craft-image-codecs", git = "https://github.com/storytold/craft-libs", rev = "51fcc3b0413af193db952df28dffae2321745703" }
+photocraft-formats = { package = "craft-file-formats", git = "https://github.com/storytold/craft-libs", rev = "51fcc3b0413af193db952df28dffae2321745703", features = ["raster-codecs", "pdf", "svg", "ora", "kra", "xcf", "pdn", "raw", "psd", "affinity", "pcraft"] }
 photocraft-raw = { path = "crates/raw" }
-craft-affinity = { git = "https://github.com/storytold/craft-libs", rev = "3eb5b528b08211844bc5c95c5a4fd169894d52ff" }
+craft-affinity = { git = "https://github.com/storytold/craft-libs", rev = "51fcc3b0413af193db952df28dffae2321745703" }
 photocraft-compose = { path = "crates/compose" }
 photocraft-gpu = { path = "crates/gpu" }
 photocraft-paint = { path = "crates/paint" }
@@ -48,36 +48,36 @@ photocraft-ipp = { path = "crates/ipp" }
 
 # external
 # ICC colour management: the shared crate (replaces crates/cms; craft-libs#779).
-craft-cms = { git = "https://github.com/storytold/craft-libs", rev = "3eb5b528b08211844bc5c95c5a4fd169894d52ff" }
-craft-print = { git = "https://github.com/storytold/craft-libs", rev = "3eb5b528b08211844bc5c95c5a4fd169894d52ff" }
+craft-cms = { git = "https://github.com/storytold/craft-libs", rev = "51fcc3b0413af193db952df28dffae2321745703" }
+craft-print = { git = "https://github.com/storytold/craft-libs", rev = "51fcc3b0413af193db952df28dffae2321745703" }
 # HEIC/HEIF: the shared decoder (replaces crates/heif; craft-libs#784).
-craft-heif = { git = "https://github.com/storytold/craft-libs", rev = "3eb5b528b08211844bc5c95c5a4fd169894d52ff" }
-craft-pdf = { git = "https://github.com/storytold/craft-libs", rev = "3eb5b528b08211844bc5c95c5a4fd169894d52ff", features = ["raster", "write-raster"] }
-craft-xml = { git = "https://github.com/storytold/craft-libs", rev = "3eb5b528b08211844bc5c95c5a4fd169894d52ff" }
-craft-docpath = { version = "0.1.0", git = "https://github.com/storytold/craft-libs", rev = "3eb5b528b08211844bc5c95c5a4fd169894d52ff" }
-craft-jpeg = { git = "https://github.com/storytold/craft-libs", rev = "3eb5b528b08211844bc5c95c5a4fd169894d52ff" }
-craft-pcraft-format = { git = "https://github.com/storytold/craft-libs", rev = "3eb5b528b08211844bc5c95c5a4fd169894d52ff" }
-craft-pcraft-bundle = { git = "https://github.com/storytold/craft-libs", rev = "3eb5b528b08211844bc5c95c5a4fd169894d52ff" }
-craft-zip = { git = "https://github.com/storytold/craft-libs", rev = "3eb5b528b08211844bc5c95c5a4fd169894d52ff" }
-craft-ora = { git = "https://github.com/storytold/craft-libs", rev = "3eb5b528b08211844bc5c95c5a4fd169894d52ff" }
-craft-kra = { git = "https://github.com/storytold/craft-libs", rev = "3eb5b528b08211844bc5c95c5a4fd169894d52ff" }
-craft-xcf = { git = "https://github.com/storytold/craft-libs", rev = "3eb5b528b08211844bc5c95c5a4fd169894d52ff" }
-craft-pdn = { git = "https://github.com/storytold/craft-libs", rev = "3eb5b528b08211844bc5c95c5a4fd169894d52ff" }
+craft-heif = { git = "https://github.com/storytold/craft-libs", rev = "51fcc3b0413af193db952df28dffae2321745703" }
+craft-pdf = { git = "https://github.com/storytold/craft-libs", rev = "51fcc3b0413af193db952df28dffae2321745703", features = ["raster", "write-raster"] }
+craft-xml = { git = "https://github.com/storytold/craft-libs", rev = "51fcc3b0413af193db952df28dffae2321745703" }
+craft-docpath = { version = "0.1.0", git = "https://github.com/storytold/craft-libs", rev = "51fcc3b0413af193db952df28dffae2321745703" }
+craft-jpeg = { git = "https://github.com/storytold/craft-libs", rev = "51fcc3b0413af193db952df28dffae2321745703" }
+craft-pcraft-format = { git = "https://github.com/storytold/craft-libs", rev = "51fcc3b0413af193db952df28dffae2321745703" }
+craft-pcraft-bundle = { git = "https://github.com/storytold/craft-libs", rev = "51fcc3b0413af193db952df28dffae2321745703" }
+craft-zip = { git = "https://github.com/storytold/craft-libs", rev = "51fcc3b0413af193db952df28dffae2321745703" }
+craft-ora = { git = "https://github.com/storytold/craft-libs", rev = "51fcc3b0413af193db952df28dffae2321745703" }
+craft-kra = { git = "https://github.com/storytold/craft-libs", rev = "51fcc3b0413af193db952df28dffae2321745703" }
+craft-xcf = { git = "https://github.com/storytold/craft-libs", rev = "51fcc3b0413af193db952df28dffae2321745703" }
+craft-pdn = { git = "https://github.com/storytold/craft-libs", rev = "51fcc3b0413af193db952df28dffae2321745703" }
 # Keep the shared embedded-image validation API until the framework revision includes it.
-craft-svg = { git = "https://github.com/storytold/craft-libs", rev = "3eb5b528b08211844bc5c95c5a4fd169894d52ff", features = ["tree-render", "tree-render-system-fonts"] }
-craft-raw = { git = "https://github.com/storytold/craft-libs", rev = "3eb5b528b08211844bc5c95c5a4fd169894d52ff" }
-craft-tiff = { git = "https://github.com/storytold/craft-libs", rev = "3eb5b528b08211844bc5c95c5a4fd169894d52ff" }
+craft-svg = { git = "https://github.com/storytold/craft-libs", rev = "51fcc3b0413af193db952df28dffae2321745703", features = ["tree-render", "tree-render-system-fonts"] }
+craft-raw = { git = "https://github.com/storytold/craft-libs", rev = "51fcc3b0413af193db952df28dffae2321745703" }
+craft-tiff = { git = "https://github.com/storytold/craft-libs", rev = "51fcc3b0413af193db952df28dffae2321745703" }
 # Camera-field EXIF reader/writer, shared prerequisite craft-libs#1563.
-craft-metadata = { git = "https://github.com/storytold/craft-libs", rev = "3eb5b528b08211844bc5c95c5a4fd169894d52ff" }
-craft-crs = { git = "https://github.com/storytold/craft-libs", rev = "3eb5b528b08211844bc5c95c5a4fd169894d52ff" }
-craft-develop = { git = "https://github.com/storytold/craft-libs", rev = "3eb5b528b08211844bc5c95c5a4fd169894d52ff" }
-craft-develop-model = { git = "https://github.com/storytold/craft-libs", rev = "3eb5b528b08211844bc5c95c5a4fd169894d52ff" }
+craft-metadata = { git = "https://github.com/storytold/craft-libs", rev = "51fcc3b0413af193db952df28dffae2321745703" }
+craft-crs = { git = "https://github.com/storytold/craft-libs", rev = "51fcc3b0413af193db952df28dffae2321745703" }
+craft-develop = { git = "https://github.com/storytold/craft-libs", rev = "51fcc3b0413af193db952df28dffae2321745703" }
+craft-develop-model = { git = "https://github.com/storytold/craft-libs", rev = "51fcc3b0413af193db952df28dffae2321745703" }
 # Interface translations: lookup, fallback, CLDR plurals and language names over the craft-i18n
 # catalogs that `crates/ui-egui/build.rs` embeds from `CRAFT_I18N_DIR` (storytold/craft-i18n, at
 # the commit in `craft-i18n.rev`). Already in the tree through craft.
-craft-i18n = { git = "https://github.com/storytold/craft-libs", rev = "3eb5b528b08211844bc5c95c5a4fd169894d52ff" }
+craft-i18n = { git = "https://github.com/storytold/craft-libs", rev = "51fcc3b0413af193db952df28dffae2321745703" }
 # craft-update: app-specific pretend versions apply to startup and manual checks.
-craft-update = { git = "https://github.com/storytold/craft-libs", version = "=0.1.0", rev = "3eb5b528b08211844bc5c95c5a4fd169894d52ff", features = ["ui"] }
+craft-update = { git = "https://github.com/storytold/craft-libs", version = "=0.1.0", rev = "51fcc3b0413af193db952df28dffae2321745703", features = ["ui"] }
 thiserror = "2"
 serde = { version = "1", features = ["derive", "rc"] }
 serde_json = "1"
@@ -350,16 +350,16 @@ opt-level = 3
 # Keep the complete egui family and native event bridge on reviewed fork commits.
 
 [patch.crates-io]
-krilla = { git = "https://github.com/storytold/craft-libs", rev = "3eb5b528b08211844bc5c95c5a4fd169894d52ff" }
-hayro = { git = "https://github.com/storytold/craft-libs", rev = "3eb5b528b08211844bc5c95c5a4fd169894d52ff" }
-hayro-interpret = { git = "https://github.com/storytold/craft-libs", rev = "3eb5b528b08211844bc5c95c5a4fd169894d52ff" }
-hayro-syntax = { git = "https://github.com/storytold/craft-libs", rev = "3eb5b528b08211844bc5c95c5a4fd169894d52ff" }
-hayro-jbig2 = { git = "https://github.com/storytold/craft-libs", rev = "3eb5b528b08211844bc5c95c5a4fd169894d52ff" }
-hayro-cmap = { git = "https://github.com/storytold/craft-libs", rev = "3eb5b528b08211844bc5c95c5a4fd169894d52ff" }
-hayro-postscript = { git = "https://github.com/storytold/craft-libs", rev = "3eb5b528b08211844bc5c95c5a4fd169894d52ff" }
-linesweeper = { git = "https://github.com/storytold/craft-libs", rev = "3eb5b528b08211844bc5c95c5a4fd169894d52ff" }
-vello_common = { git = "https://github.com/storytold/craft-libs", rev = "3eb5b528b08211844bc5c95c5a4fd169894d52ff" }
-vello_cpu = { git = "https://github.com/storytold/craft-libs", rev = "3eb5b528b08211844bc5c95c5a4fd169894d52ff" }
+krilla = { git = "https://github.com/storytold/craft-libs", rev = "51fcc3b0413af193db952df28dffae2321745703" }
+hayro = { git = "https://github.com/storytold/craft-libs", rev = "51fcc3b0413af193db952df28dffae2321745703" }
+hayro-interpret = { git = "https://github.com/storytold/craft-libs", rev = "51fcc3b0413af193db952df28dffae2321745703" }
+hayro-syntax = { git = "https://github.com/storytold/craft-libs", rev = "51fcc3b0413af193db952df28dffae2321745703" }
+hayro-jbig2 = { git = "https://github.com/storytold/craft-libs", rev = "51fcc3b0413af193db952df28dffae2321745703" }
+hayro-cmap = { git = "https://github.com/storytold/craft-libs", rev = "51fcc3b0413af193db952df28dffae2321745703" }
+hayro-postscript = { git = "https://github.com/storytold/craft-libs", rev = "51fcc3b0413af193db952df28dffae2321745703" }
+linesweeper = { git = "https://github.com/storytold/craft-libs", rev = "51fcc3b0413af193db952df28dffae2321745703" }
+vello_common = { git = "https://github.com/storytold/craft-libs", rev = "51fcc3b0413af193db952df28dffae2321745703" }
+vello_cpu = { git = "https://github.com/storytold/craft-libs", rev = "51fcc3b0413af193db952df28dffae2321745703" }
 ecolor = { git = "https://github.com/storytold/egui.git", rev = "932a73b28560acb1a695533307b2cd0d66a42081" }
 emath = { git = "https://github.com/storytold/egui.git", rev = "932a73b28560acb1a695533307b2cd0d66a42081" }
 epaint = { git = "https://github.com/storytold/egui.git", rev = "932a73b28560acb1a695533307b2cd0d66a42081" }
```

**What each hunk does.**

1. moves the facade and the two macro crates to `51fcc3b04`, this round's framework revision.
2. moves the plist, psd, image-codecs, file-formats and affinity pins to the same revision.
3. moves the remaining framework pins (colour management, print, HEIF, PDF, XML, doc path, JPEG, PCTIP, ZIP, ORA, KRA, XCF, PDN, SVG, raw, TIFF, EXIF, CRS, develop, i18n, update) to that revision.
4. moves the `[patch.crates-io]` redirects for krilla, hayro and the vello crates to that revision.

**If this were reverted / how it is checked.** `cargo xtask craftlibs status` prints the config on the pin and the lock recorded; reverting the pins builds the previous framework and the SAM and selection rows fail to compile.

### `crates/engine-paint/src/cmd/retouch_cmds.rs`

**Role.** the retouch command table: every command the Retouch family registers, its params and its journal flag.

**The diff, verbatim** (`git diff 581eb807b^2 581eb807b -- crates/engine-paint/src/cmd/retouch_cmds.rs`):

```diff
diff --git a/crates/engine-paint/src/cmd/retouch_cmds.rs b/crates/engine-paint/src/cmd/retouch_cmds.rs
index 767f0547d..ffb11bd77 100644
--- a/crates/engine-paint/src/cmd/retouch_cmds.rs
+++ b/crates/engine-paint/src/cmd/retouch_cmds.rs
@@ -34,6 +34,7 @@ mod pattern_stamp;
 pub use pattern_stamp::{
     INDEXED_REFUSAL as PATTERN_STAMP_INDEXED_REFUSAL, live_impressionist as live_pattern_impressionist, press_refusal as pattern_stamp_press_refusal,
 };
+mod lama_cleaner;
 mod periodic;
 mod remove;
 pub use content_aware_move::{ContentAwarePreview, preview as content_aware_move_preview};
@@ -363,6 +364,18 @@ pub fn specs() -> Vec<CommandSpec> {
             run: remove::remove,
             journal: true,
         },
+        CommandSpec {
+            id: "paint.lamaCleaner",
+            label: "LaMa Cleaner",
+            menu: &[],
+            shortcut: None,
+            params: brush_params!(
+                r#","strokes":[[[x,y,pressure?],…]|{"points":[…],"size"?,"hardness"?,"spacing"?},…]?,"closeLoops":bool=true,"sampleAllLayers":bool=false,"newLayer":bool=false,"exe":text,"model":text (the LaMa helper and model, when they are not beside the app) → {"damage","layer"?}"#
+            ),
+            enabled: has_pixel_layer,
+            run: lama_cleaner::lama_cleaner,
+            journal: true,
+        },
         CommandSpec {
             id: "paint.dodge",
             label: "Dodge",
```

**What each hunk does.**

1. keeps the LaMa Cleaner command module registered in the retouch table.
2. keeps the `paint.lamaCleaner` command spec: brush params plus `strokes`, `closeLoops`, `sampleAllLayers`, `newLayer`, enabled on a pixel layer and journalled.

**If this were reverted / how it is checked.** `cargo nextest run -p photocraft-ui-egui -E 'test(<the trials this file holds>)'` and `cargo xtask ci lint` name the loss.

### `crates/ui-egui/src/app.rs`

**Role.** the application object's own state and the small methods that reset it.

**The diff, verbatim** (`git diff 581eb807b^2 581eb807b -- crates/ui-egui/src/app.rs`):

```diff
diff --git a/crates/ui-egui/src/app.rs b/crates/ui-egui/src/app.rs
index 8f0732514..256379e89 100644
--- a/crates/ui-egui/src/app.rs
+++ b/crates/ui-egui/src/app.rs
@@ -74,6 +74,13 @@ impl PhotocraftApp {
             dock_model: Default::default(),
             preset_art: Default::default(),
             move_preview: None,
+            move_blocked: false,
+            move_blocked_start: None,
+            move_blocked_moved: false,
+            move_blocked_layer: None,
+            move_blocked_doc: None,
+            move_viewport: None,
+            current_viewport: None,
             move_hover: None,
             move_distance: None,
             object_finder: None,
@@ -262,6 +269,20 @@ impl PhotocraftApp {
     }
 }
 
+impl PhotocraftApp {
+    /// Clear the state of a refused Move press and its candidate transfer layer.
+    pub(crate) fn clear_move_blocked(&mut self) {
+        self.move_blocked = false;
+        self.move_blocked_start = None;
+        self.move_blocked_moved = false;
+        self.move_blocked_layer = None;
+        self.move_blocked_doc = None;
+        if self.layer_box.is_none() {
+            self.move_viewport = None;
+        }
+    }
+}
+
 impl eframe::App for PhotocraftApp {
     fn on_exit(&mut self) {
         if let Err(err) = tool_memory::save_on_exit(self) {
```

**What each hunk does.**

1. keeps the refused-Move bookkeeping initialised: `move_blocked`, its start point, moved flag, layer, document, and the gesture's viewport.
2. keeps `clear_move_blocked`, which clears the refused press and drops the remembered viewport when no layer box is up.

**If this were reverted / how it is checked.** `cargo nextest run -p photocraft-ui-egui -E 'test(<the trials this file holds>)'` and `cargo xtask ci lint` name the loss.

### `crates/ui-egui/src/canvas.rs`

**Role.** the canvas view: one frame's tool, viewport and paint plumbing.

**The diff, verbatim** (`git diff 581eb807b^2 581eb807b -- crates/ui-egui/src/canvas.rs`):

```diff
diff --git a/crates/ui-egui/src/canvas.rs b/crates/ui-egui/src/canvas.rs
index 2af53fa99..087e79e6e 100644
--- a/crates/ui-egui/src/canvas.rs
+++ b/crates/ui-egui/src/canvas.rs
@@ -176,6 +176,7 @@ pub fn canvas_view(
     mut view: View,
     primary: bool,
 ) -> (View, craft::ui::canvas::CanvasOutput) {
+    app.current_viewport = Some(ui.ctx().viewport_id());
     let ctx = ui.ctx().clone();
     // What the canvas shows follows the pointer (the tool's cursor, the rulers' marks, the Info panel and status bar
     // readouts), so every pointer move over it runs a pass; elsewhere a move that changes nothing runs none (#17148).
```

**What each hunk does.**

1. records the viewport that owns each frame's canvas, so a refused Move press cannot become a cross-window transfer.

**If this were reverted / how it is checked.** `cargo nextest run -p photocraft-ui-egui -E 'test(<the trials this file holds>)'` and `cargo xtask ci lint` name the loss.

### `crates/ui-egui/src/canvas/canvas_tool_menu.rs`

**Role.** the canvas tool menu (the right-click menu of a tool): which tools it applies to and which rows it shows.

**The diff, verbatim** (`git diff 581eb807b^2 581eb807b -- crates/ui-egui/src/canvas/canvas_tool_menu.rs`):

```diff
diff --git a/crates/ui-egui/src/canvas/canvas_tool_menu.rs b/crates/ui-egui/src/canvas/canvas_tool_menu.rs
index 810aec8d1..accf2f9e0 100644
--- a/crates/ui-egui/src/canvas/canvas_tool_menu.rs
+++ b/crates/ui-egui/src/canvas/canvas_tool_menu.rs
@@ -1085,6 +1085,7 @@ pub fn applies(tool: Tool) -> bool {
             | Tool::MagneticLasso
             | Tool::MagicWand
             | Tool::ObjectSelection
+            | Tool::Sam3Select
             // Its brush is set from the options bar's chip; the canvas right-click is the
             // selection menu, as in Photoshop (#7268).
             | Tool::QuickSelection
@@ -1264,7 +1265,7 @@ pub fn rows(menu: &CanvasToolMenu) -> &'static [Row] {
                 // are greyed was not measured, so this tool adds no rule: `entry_enabled` keeps the
                 // menu bar's.
                 if menu.has_selection { MEASURED_SELECTION_MENU } else { COUNT_MENU }
-            } else if matches!(menu.tool, Tool::RedEye | Tool::ObjectSelection | Tool::Type) {
+            } else if matches!(menu.tool, Tool::RedEye | Tool::ObjectSelection | Tool::Sam3Select | Tool::Type) {
                 if menu.has_selection { MEASURED_SELECTION_MENU } else { COUNT_MENU }
             } else if menu.tool == Tool::Eyedropper {
                 EYEDROPPER_MENU
```

**What each hunk does.**

1. adds `Tool::Sam3Select` to the tools the canvas menu applies to.
2. gives `Sam3Select` the selection rows the Red Eye and Object Selection rows use.

**If this were reverted / how it is checked.** `cargo nextest run -p photocraft-ui-egui -E 'test(<the trials this file holds>)'` and `cargo xtask ci lint` name the loss.

### `crates/ui-egui/src/canvas/cursor_policy.rs`

**Role.** the cursor a tool shows over the canvas.

**The diff, verbatim** (`git diff 581eb807b^2 581eb807b -- crates/ui-egui/src/canvas/cursor_policy.rs`):

```diff
diff --git a/crates/ui-egui/src/canvas/cursor_policy.rs b/crates/ui-egui/src/canvas/cursor_policy.rs
index cbc336f91..4f02b1937 100644
--- a/crates/ui-egui/src/canvas/cursor_policy.rs
+++ b/crates/ui-egui/src/canvas/cursor_policy.rs
@@ -2,7 +2,7 @@
 //! which let ⌥ sample or load paint, the Pencil's tip shape, and the pointers of the tools whose
 //! cursor is no brush tip.
 
-use craft::behavior::{Cursor, CursorBadge, PaintedCursor, PointerGlyph};
+use craft::behavior::{Cursor, PaintedCursor, PointerGlyph};
 use craft::keymap::Mods;
 use photocraft_engine::prefs::PaintingCursor as PaintingPref;
 
@@ -35,14 +35,6 @@ pub(crate) fn brush_tip_centre(tool: Tool, option: bool, show_crosshair: bool) -
     show_crosshair || matches!(tool, Tool::BackgroundEraser | Tool::ColorReplacement | Tool::Remove)
 }
 
-/// The mark at the centre of Quick Selection's ring with `mods` held: a "−" while ⌥ would subtract
-/// from a selection (#16822). ⇧ alone and no selection keep the plain ring, as in the reference;
-/// ⇧⌥, Control+⌥ (the brush-resize drag) and the options bar's mode were not measured and keep it
-/// too. `None` for any other tool.
-pub(crate) fn quick_selection_badge(app: &PhotocraftApp, mods: Mods) -> Option<CursorBadge> {
-    (app.ui.tool == Tool::QuickSelection && mods.alt && !mods.shift && !mods.ctrl && super::tool_feedback::has_selection(app)).then_some(CursorBadge::Subtract)
-}
-
 /// Whether the brush cursor is the reference's fixed sample target (a small circle with a cross
 /// through the hotspot) at any tip size: the Healing Brush's ⌥, the source point, with Painting
 /// Cursors = Normal Brush Tip (#12113). The other Painting Cursors settings were not measured and
```

**What each hunk does.**

1. drops `CursorBadge` from the import: the subtraction badge moved into the outline-drag cursor.
2. removes `quick_selection_badge`, whose job the drag cursor in `paint_tool.rs` now does (mode badge plus selection-move cursor).

**If this were reverted / how it is checked.** `cargo nextest run -p photocraft-ui-egui -E 'test(<the trials this file holds>)'` and `cargo xtask ci lint` name the loss.

### `crates/ui-egui/src/canvas/paint_tool.rs`

**Role.** the canvas tool that paints brush strokes and, since #11837, drags a selection's outline with the Quick Selection.

**The diff, verbatim** (`git diff 581eb807b^2 581eb807b -- crates/ui-egui/src/canvas/paint_tool.rs`):

```diff
diff --git a/crates/ui-egui/src/canvas/paint_tool.rs b/crates/ui-egui/src/canvas/paint_tool.rs
index c3be4b491..b90635ae7 100644
--- a/crates/ui-egui/src/canvas/paint_tool.rs
+++ b/crates/ui-egui/src/canvas/paint_tool.rs
@@ -9,9 +9,13 @@
 //! at once. `PaintResize` lives in `app.brush_resize`: each call that needs the app too takes it out
 //! with `std::mem::take` first.
 
+use std::cell::RefCell;
 use std::time::Duration;
 
+use craft::behavior::geometry::SelectionOperation;
+use craft::behavior::kurbo::Point;
 use craft::behavior::{CancelReason, Cursor, CursorIcon, PaintedCursor, PointerButton};
+use craft::imaging::ui::select::{DragKind, PressRule, SelectionProbe, drag_kind};
 use craft::keymap::{Chord, Key, Mods};
 use craft::ui::canvas::{CanvasTool, ToolAction, ToolClaims, ToolContext, ToolHostResponse, ToolKey, ToolPath, ToolPointer};
 use craft::ui::primitives::Draw;
@@ -25,6 +29,7 @@ use super::strokes::{
     HOLD_REPEAT_MS, begin_live_stroke, feed_live_stroke, live_stroke, live_stroke_busy, paint_target, snap_aliased_tip, stroke_params, stroke_wants_time,
     strokes_live, tap_at_peak_pressure,
 };
+use super::tools::selection::{InsideSelectionDrag, PcSelection, drag_delta};
 use super::tools::{FamilyTool, ToolScope};
 use crate::state::Tool;
 use serde_json::json;
@@ -78,21 +83,51 @@ impl PaintStroke {
 }
 
 /// The paint family's state, in `CanvasTools::paint`.
-#[derive(Debug, Default)]
+#[derive(Default)]
 pub(crate) struct PaintState {
     /// The stroke being drawn, from the press to the release.
     pub(crate) stroke: Option<PaintStroke>,
+    /// The Quick Selection's New-mode press inside its selection (#11837): a click deselects and a
+    /// drag moves the outline, as with the other selection tools (#11782).
+    pub(crate) drag: InsideSelectionDrag,
+}
+
+impl PaintState {
+    /// The whole-pixel offset of the Quick Selection's outline drag in progress, for `selection::shown_offset`.
+    pub(crate) fn drag_offset(&self) -> Option<(i32, i32)> {
+        drag_delta(self.drag.phase())
+    }
 }
 
 /// The paint tool for one pass.
 pub(crate) struct PaintTool<'a> {
     scope: &'a ToolScope<'a>,
-    state: &'a mut PaintState,
+    state: RefCell<&'a mut PaintState>,
+    tool: Tool,
+    probe: PcSelection,
+    bar: SelectionOperation,
+}
+
+impl PaintTool<'_> {
+    /// In New mode, a Quick Selection press inside a non-floating selection acts on the selection
+    /// (#11837): a click deselects and a drag moves the outline, as the other selection tools do
+    /// (#11782). Shift adds and Alt subtracts.
+    fn drags_outline(&self, at: [f64; 2], mods: Mods) -> bool {
+        self.tool == Tool::QuickSelection
+            && !self.probe.floating()
+            && drag_kind(&self.probe, Point::new(at[0], at[1]), mods, self.bar, PressRule::Drags) == Some(DragKind::Outline)
+    }
 }
 
 /// The paint family's tool for this pass.
 pub(crate) fn tool<'a>(scope: &'a ToolScope<'a>, state: &'a mut PaintState, _host: CanvasHost) -> FamilyTool<'a> {
-    Box::new(PaintTool { scope, state })
+    let (tool, probe, bar) = scope.with_app(|app| {
+        let tool = app.ui.tool;
+        let bar = crate::tool_feedback::selection_op(tool, crate::tool_feedback::bar_mode(app, tool), Mods::NONE, true);
+        (tool, PcSelection::of(app), bar)
+    });
+    state.drag.operation(bar);
+    Box::new(PaintTool { scope, state: RefCell::new(state), tool, probe, bar })
 }
 
 /// The stroke being drawn, for readers outside the host's run (tests, `retouch_ui`, the clone
@@ -105,6 +140,7 @@ pub(crate) fn stroking(app: &PhotocraftApp) -> Option<&PaintStroke> {
 /// so the stroke shows them in the same pass. Only the points not yet fed are fed, so a second
 /// pass of the same frame (which sees only its own moves) never repeats the first one's.
 pub(crate) fn drain(app: &mut PhotocraftApp, state: &mut PaintState, _response: &ToolHostResponse) {
+    state.drag.drain(app, Tool::QuickSelection);
     let Some(stroke) = state.stroke.as_ref() else { return };
     feed_live_stroke(app, stroke);
     let ctx = app.ctx().clone();
@@ -345,9 +381,10 @@ impl CanvasTool for PaintTool<'_> {
     /// stroke whose document or revision moved on is dropped (its uncommitted paint belongs to the
     /// old picture).
     fn each_pass(&mut self, cx: &mut ToolContext<'_>) {
-        let Self { scope, state } = self;
+        let mut state = self.state.borrow_mut();
+        state.drag.each_pass(&self.probe, cx);
         let now_ms = cx.time * 1000.0;
-        scope.with_app(|app| {
+        self.scope.with_app(|app| {
             crate::brush_cursor::sync_tip(app);
             // The readout request lasts one pass: a pass without pointer input repeats it.
             app.brush_resize.hud(cx);
@@ -355,7 +392,7 @@ impl CanvasTool for PaintTool<'_> {
             let stale_mixer =
                 stroke.tool == Tool::MixerBrush && app.live_stroke.is_some() && app.session.active_index().is_none_or(|idx| live_stroke(app, idx).is_none());
             if stale_mixer {
-                drop_stroke(app, state);
+                drop_stroke(app, &mut state);
                 return;
             }
             if hold_due(app, now_ms) {
@@ -369,13 +406,20 @@ impl CanvasTool for PaintTool<'_> {
 
     fn press(&mut self, pointer: &ToolPointer, cx: &mut ToolContext<'_>) -> ToolAction {
         let Some(at) = doc_point(cx, pointer.pos) else { return ToolAction::Ignored };
-        let Self { scope, state } = self;
-        scope.with_app(|app| begin(app, state, pointer, at, cx))
+        if pointer.button == PointerButton::Primary && self.drags_outline(at, pointer.mods) {
+            return self.state.borrow_mut().drag.press(&self.probe, pointer, cx);
+        }
+        let mut state = self.state.borrow_mut();
+        self.scope.with_app(|app| begin(app, &mut state, pointer, at, cx))
     }
 
     fn drag(&mut self, pointer: &ToolPointer, cx: &mut ToolContext<'_>) {
-        let Self { scope, state } = self;
-        scope.with_app(|app| {
+        let mut state = self.state.borrow_mut();
+        if state.drag.active() {
+            state.drag.drag(&self.probe, pointer, cx);
+            return;
+        }
+        self.scope.with_app(|app| {
             if with_resize(app, |r, app| {
                 let active = r.active();
                 if active {
@@ -407,9 +451,13 @@ impl CanvasTool for PaintTool<'_> {
     }
 
     fn release(&mut self, pointer: &ToolPointer, cx: &mut ToolContext<'_>) {
+        let mut state = self.state.borrow_mut();
+        if state.drag.active() {
+            state.drag.release(&self.probe, pointer, cx);
+            return;
+        }
         let at = doc_point(cx, pointer.pos);
-        let Self { scope, state } = self;
-        scope.with_app(|app| {
+        self.scope.with_app(|app| {
             if with_resize(app, |r, app| {
                 let active = r.active();
                 if active {
@@ -446,10 +494,11 @@ impl CanvasTool for PaintTool<'_> {
         });
     }
 
-    fn cancel(&mut self, _reason: CancelReason, cx: &mut ToolContext<'_>) {
-        let Self { scope, state } = self;
-        scope.with_app(|app| {
-            drop_stroke(app, state);
+    fn cancel(&mut self, reason: CancelReason, cx: &mut ToolContext<'_>) {
+        let mut state = self.state.borrow_mut();
+        state.drag.cancel(&self.probe, reason, cx);
+        self.scope.with_app(|app| {
+            drop_stroke(app, &mut state);
             with_resize(app, |r, app| r.cancel(app, cx));
             // A cancelled right-press (Escape) ends the HUD whoever opened it; the held button's release is then no gesture.
             crate::color_hud::close(&mut app.ui);
@@ -484,6 +533,13 @@ impl CanvasTool for PaintTool<'_> {
     fn paint(&self, _draw: &Draw<'_>, _cx: &ToolContext<'_>) {}
 
     fn cursor(&self, cx: &ToolContext<'_>) -> Cursor {
+        {
+            let mut state = self.state.borrow_mut();
+            let over = state.stroke.is_none() && cx.pointer.as_ref().and_then(|p| doc_point(cx, p.pos)).is_some_and(|at| self.drags_outline(at, cx.mods));
+            if state.drag.active() || over {
+                return state.drag.cursor(&self.probe, cx);
+            }
+        }
         self.scope.with_app(|app| {
             if app.brush_resize.active() {
                 return Cursor::Painted(PaintedCursor::Hidden);
```

**What each hunk does.**

1. imports `RefCell`, the selection operation, the point type and the framework's selection probe helpers.
2. imports the fork's `InsideSelectionDrag`, `PcSelection` and `drag_delta`.
3. gives `PaintState` a `drag` field and `drag_offset()` for the live marching ants; `Debug` goes because the drag holds no printable state.
4. makes `PaintTool` hold `state` behind a `RefCell` and carry `tool`, `probe` and `bar`, with `drags_outline` deciding whether a press acts on the selection.
5. drains a finished outline drag when a stroke starts, so a new stroke never inherits the drag's state.
6. borrows the state each pass, lets the drag advance, and runs the frame through the app scope.
7. passes the borrowed state to `drop_stroke`.
8. routes a primary press inside the selection in New mode to the drag, and a move to the active drag, before any stroke begins.
9. releases an active drag and returns, so the release records a selection move rather than a stroke.
10. cancels the drag with the same reason as the stroke, then drops the stroke.
11. shows the drag's own cursor while dragging or hovering inside the selection in New mode.

**If this were reverted / how it is checked.** `cargo nextest run -p photocraft-ui-egui -E 'test(quick_selection) or test(the_quick_selection)'` fails: a click inside the selection paints instead of deselecting, and the cursor stays the brush tip.

### `crates/ui-egui/src/canvas/tests.rs`

**Role.** the canvas's GPU-free and GPU trials.

**The diff, verbatim** (`git diff 581eb807b^2 581eb807b -- crates/ui-egui/src/canvas/tests.rs`):

```diff
diff --git a/crates/ui-egui/src/canvas/tests.rs b/crates/ui-egui/src/canvas/tests.rs
index 9c928ab23..fea05ef58 100644
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

1. adds `tile_all_vertically_shows_every_document_in_its_own_pane`: two documents, one pane each, side by side or stacked, and no floating window.

**If this were reverted / how it is checked.** `cargo nextest run -p photocraft-ui-egui -E 'test(<the trials this file holds>)'` and `cargo xtask ci lint` name the loss.

### `crates/ui-egui/src/canvas/tools/mod.rs`

**Role.** the tool families: which family a tool belongs to (Retouch, Selection, Vector, ...).

**The diff, verbatim** (`git diff 581eb807b^2 581eb807b -- crates/ui-egui/src/canvas/tools/mod.rs`):

```diff
diff --git a/crates/ui-egui/src/canvas/tools/mod.rs b/crates/ui-egui/src/canvas/tools/mod.rs
index 7d8c6ba2e..44d095f19 100644
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

**What each hunk does.**

1. puts `Sam3Select` in `Family::Retouch`, so the Retouch bar, cursors and Reset Tool treat it as one of theirs.

**If this were reverted / how it is checked.** `cargo nextest run -p photocraft-ui-egui -E 'test(<the trials this file holds>)'` and `cargo xtask ci lint` name the loss.

### `crates/ui-egui/src/lib.rs`

**Role.** the application crate root: `PhotocraftApp`'s fields and the test modules it registers.

**The diff, verbatim** (`git diff 581eb807b^2 581eb807b -- crates/ui-egui/src/lib.rs`):

```diff
diff --git a/crates/ui-egui/src/lib.rs b/crates/ui-egui/src/lib.rs
index d159f8f21..ca5dad36f 100644
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
@@ -875,6 +885,12 @@ mod lock_layers_shortcut_tests;
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

1. declares the refused-Move fields on `PhotocraftApp`.
2. registers the three SAM trial modules.

**If this were reverted / how it is checked.** `cargo nextest run -p photocraft-ui-egui -E 'test(<the trials this file holds>)'` and `cargo xtask ci lint` name the loss.

### `crates/ui-egui/src/panels/options_bar.rs`

**Role.** the options bar: which rows a tool shows above the canvas.

**The diff, verbatim** (`git diff 581eb807b^2 581eb807b -- crates/ui-egui/src/panels/options_bar.rs`):

```diff
diff --git a/crates/ui-egui/src/panels/options_bar.rs b/crates/ui-egui/src/panels/options_bar.rs
index 14c00e228..dab09719d 100644
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

**What each hunk does.**

1. gives the Quick Selection its own New / Add / Subtract segments in the options bar.

**If this were reverted / how it is checked.** `cargo nextest run -p photocraft-ui-egui -E 'test(<the trials this file holds>)'` and `cargo xtask ci lint` name the loss.

### `crates/ui-egui/src/prefs/prefs_ui/tests.rs`

**Role.** the preferences window's trials.

**The diff, verbatim** (`git diff 581eb807b^2 581eb807b -- crates/ui-egui/src/prefs/prefs_ui/tests.rs`):

```diff
diff --git a/crates/ui-egui/src/prefs/prefs_ui/tests.rs b/crates/ui-egui/src/prefs/prefs_ui/tests.rs
index 2c9ecb184..29936a01d 100644
--- a/crates/ui-egui/src/prefs/prefs_ui/tests.rs
+++ b/crates/ui-egui/src/prefs/prefs_ui/tests.rs
@@ -168,6 +168,10 @@ fn prefs_harness(
     let mut h = egui_kittest::Harness::builder().with_size(size).build_eframe(move |cc| {
         PhotocraftApp::setup_context(&cc.egui_ctx, Default::default());
         let mut app = PhotocraftApp::with_context(cc.egui_ctx.clone(), photocraft_engine::Session::new(), services_on(&files));
+        // The trials assert the words upstream writes, so the harness runs the interface in English:
+        // the app's default follows the system's language, and a Japanese machine would paint
+        // Japanese. The shipped app still follows the user's preferences.
+        let _ = app.run("prefs.set", json!({"values": {"interface.language": "en"}}));
         setup(&mut app);
         open_prefs(&mut app, section);
         app
@@ -491,7 +495,9 @@ fn preferences_interface_offers_and_applies_sync_with_system() {
 #[test]
 fn theme_star_saves_favorite_and_menu_groups_it() {
     use egui_kittest::kittest::Queryable;
-    let (_, files) = app_with_store();
+    // The framework's star and footer read the window's language: pin English, so a Japanese
+    // system locale does not rename them under the test.
+    let (_, files) = app_with_json(&json!({"interface": {"language": "en"}}));
     let mut h = prefs_harness(&files, "interface", vec2(1280.0, 800.0), |_| {});
     h.get_by_label("Classic").hover();
     h.run_steps(2);
@@ -1763,6 +1769,9 @@ fn scratch_disk_browse_sets_only_its_draft_entry_and_cancel_keeps_it() {
     use egui_kittest::kittest::Queryable;
     let scripted = Arc::new(ScriptedFiles::answering([Picked::Paths(vec!["/tmp/scratch".into()]), Picked::Cancelled]));
     let (store, picks) = (Arc::new(MemoryFiles::new()), scripted.clone());
+    // Browse… comes from the framework's words: pin English, so a Japanese system locale does not
+    // rename it under the test.
+    seed(&store, &json!({"interface": {"language": "en"}}), Some(<prefs::SavedPrefs as Settings>::SCHEMA_VERSION));
     let mut h = egui_kittest::Harness::builder().with_size(vec2(1280.0, 800.0)).build_eframe(move |cc| {
         PhotocraftApp::setup_context(&cc.egui_ctx, Default::default());
         let services = crate::Services { files: Some(picks), ..services_on(&store) };
```

**What each hunk does.**

1. pins `interface.language` to English in the preferences harness, so the trials assert the words they name on any system locale.
2. builds the sync trial's app from JSON with English pinned, for the framework's star and footer words.
3. seeds English before the Browse… trial, whose label is a framework word.

**If this were reverted / how it is checked.** `cargo nextest run -p photocraft-ui-egui -E 'test(<the trials this file holds>)'` and `cargo xtask ci lint` name the loss.

### `crates/ui-egui/src/state.rs`

**Role.** the tool list and the per-tool state (`Tool::ALL`).

**The diff, verbatim** (`git diff 581eb807b^2 581eb807b -- crates/ui-egui/src/state.rs`):

```diff
diff --git a/crates/ui-egui/src/state.rs b/crates/ui-egui/src/state.rs
index 203bd924e..ddff120b2 100644
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

1. raises the tool count to 72: Quick Selection's own mode additions and the SAM and LaMa tools.

**If this were reverted / how it is checked.** `cargo nextest run -p photocraft-ui-egui -E 'test(<the trials this file holds>)'` and `cargo xtask ci lint` name the loss.

### `crates/ui-egui/src/state/tool_options.rs`

**Role.** the options bar's persisted values, one field per tool option, behind the tracked model.

**The diff, verbatim** (`git diff 581eb807b^2 581eb807b -- crates/ui-egui/src/state/tool_options.rs`):

```diff
diff --git a/crates/ui-egui/src/state/tool_options.rs b/crates/ui-egui/src/state/tool_options.rs
index 463ed44db..16109be09 100644
--- a/crates/ui-egui/src/state/tool_options.rs
+++ b/crates/ui-egui/src/state/tool_options.rs
@@ -133,6 +133,22 @@ pub struct ToolOptions {
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
@@ -342,6 +358,20 @@ fn default_move_target() -> String {
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
@@ -389,6 +419,7 @@ pub struct SampleAllLayers {
     pub magic_wand: bool,
     pub quick_selection: bool,
     pub object_selection: bool,
+    pub sam3_select: bool,
     pub paint_bucket: bool,
     pub magic_eraser: bool,
     pub spot_healing: bool,
@@ -406,6 +437,7 @@ impl Default for SampleAllLayers {
             magic_wand: false,
             quick_selection: false,
             object_selection: false,
+            sam3_select: false,
             paint_bucket: false,
             magic_eraser: false,
             spot_healing: false,
@@ -426,6 +458,7 @@ macro_rules! sample_all_slot {
             Tool::MagicWand => $($borrow)+ $this.magic_wand,
             Tool::QuickSelection => $($borrow)+ $this.quick_selection,
             Tool::ObjectSelection => $($borrow)+ $this.object_selection,
+            Tool::Sam3Select => $($borrow)+ $this.sam3_select,
             Tool::PaintBucket => $($borrow)+ $this.paint_bucket,
             Tool::MagicEraser => $($borrow)+ $this.magic_eraser,
             Tool::SpotHealing => $($borrow)+ $this.spot_healing,
@@ -518,6 +551,11 @@ impl Default for ToolOptions {
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

**What each hunk does.**

1. keeps the Quick Selection mode field and the four SAM fields, each with a serde default so old settings load.
2. keeps the defaults: New, threshold 0.5 and two refinement passes.
3. keeps `sam3_select` in `SampleAllLayers`.
4. keeps its default, false.
5. keeps `Tool::Sam3Select` in `sample_all_slot!`, which is what makes the field reachable through `slot`/`slot_mut`.
6. initialises the new fields in `ToolOptions::default`.

**If this were reverted / how it is checked.** `cargo nextest run -p photocraft-ui-egui -E 'test(tool_options) or test(prefs::prefs_ui)'` fails: the fields stop persisting and the sample-all slot returns `None` for the SAM tool.

### `crates/ui-egui/src/tools/retouch_ui.rs`

**Role.** the Retouch tools' gesture handlers: what a press, drag and release send to the engine.

**The diff, verbatim** (`git diff 581eb807b^2 581eb807b -- crates/ui-egui/src/tools/retouch_ui.rs`):

```diff
diff --git a/crates/ui-egui/src/tools/retouch_ui.rs b/crates/ui-egui/src/tools/retouch_ui.rs
index d0a5e5e7f..32f656f82 100644
--- a/crates/ui-egui/src/tools/retouch_ui.rs
+++ b/crates/ui-egui/src/tools/retouch_ui.rs
@@ -272,6 +272,18 @@ pub(crate) fn finish_stroke_with(app: &mut PhotocraftApp, tool: Tool, points: &[
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
@@ -279,11 +291,19 @@ pub(crate) fn finish_stroke_with(app: &mut PhotocraftApp, tool: Tool, points: &[
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
@@ -735,6 +755,59 @@ pub fn finish_object_selection(app: &mut PhotocraftApp, start: [f64; 2], end: [f
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

1. keeps LaMa Cleaner's stroke handler: it sends `paint.lamaCleaner` with the brush's size, hardness and spacing.
2. keeps the Quick Selection's own mode in its stroke: the bar's mode plus the modifiers decide add or subtract, and New flips to Add after the first stroke on an empty selection.
3. keeps `sam3_select_params` and `finish_sam3_selection`: a typed prompt wins over the gesture, a click sends a point, a drag sends a box, and the mode comes from the modifiers or the bar.

**If this were reverted / how it is checked.** `cargo nextest run -p photocraft-ui-egui -E 'test(<the trials this file holds>)'` and `cargo xtask ci lint` name the loss.

### `crates/ui-egui/src/tools/retouch_ui/bar/heal.rs`

**Role.** the Retouch options bar's healing family (Clone, Heal, Patch, LaMa Cleaner).

**The diff, verbatim** (`git diff 581eb807b^2 581eb807b -- crates/ui-egui/src/tools/retouch_ui/bar/heal.rs`):

```diff
diff --git a/crates/ui-egui/src/tools/retouch_ui/bar/heal.rs b/crates/ui-egui/src/tools/retouch_ui/bar/heal.rs
index fc49e3027..52fdec71c 100644
--- a/crates/ui-egui/src/tools/retouch_ui/bar/heal.rs
+++ b/crates/ui-egui/src/tools/retouch_ui/bar/heal.rs
@@ -151,5 +151,13 @@ pub(super) fn clone(ui: &mut egui::Ui, app: &mut PhotocraftApp) -> Option<RangeA
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

**What each hunk does.**

1. keeps the LaMa Cleaner options row (its Size cell, 1..500 px).

**If this were reverted / how it is checked.** `cargo nextest run -p photocraft-ui-egui -E 'test(<the trials this file holds>)'` and `cargo xtask ci lint` name the loss.

### `crates/ui-egui/src/tools/retouch_ui/bar/select.rs`

**Role.** the Retouch options bar's selection family (Quick Selection, Object Selection, SAM Selection).

**The diff, verbatim** (`git diff 581eb807b^2 581eb807b -- crates/ui-egui/src/tools/retouch_ui/bar/select.rs`):

```diff
diff --git a/crates/ui-egui/src/tools/retouch_ui/bar/select.rs b/crates/ui-egui/src/tools/retouch_ui/bar/select.rs
index e0ab4bd36..47c5a0d67 100644
--- a/crates/ui-egui/src/tools/retouch_ui/bar/select.rs
+++ b/crates/ui-egui/src/tools/retouch_ui/bar/select.rs
@@ -2,17 +2,18 @@
 
 use craft::behavior::IconId;
 use craft::ui::buttons::{Button, IconButton};
-use craft::ui::contract::{Entry, Props as _, Show as _, Trigger, bind_tracked, clicked};
+use craft::ui::contract::{Bind, Entry, Props as _, Show as _, Trigger, bind_tracked, clicked};
 use craft::ui::display::Divider;
 use craft::ui::menu::Menu;
 use craft::ui::runtime::access::PopupKind;
-use craft::ui::text::RangeAlert;
+use craft::ui::runtime::env::Label;
+use craft::ui::text::{RangeAlert, TextField};
 use serde_json::json;
 
 use super::heal::check;
 use crate::PhotocraftApp;
-use crate::option_cells::hint;
-use crate::state::ObjectFinderMode;
+use crate::option_cells::{hint, number};
+use crate::state::{ObjectFinderMode, Tool};
 
 /// The Select Subject button both selection tools carry; `empty` is the message when nothing is found.
 fn select_subject(ui: &mut egui::Ui, app: &mut PhotocraftApp, empty: &str) {
@@ -30,6 +31,8 @@ pub(super) fn quick_selection(ui: &mut egui::Ui, app: &mut PhotocraftApp) -> Opt
     hint(ui, crate::i18n::fmt(tl!("{key} to subtract"), &[("key", &crate::shortcuts::pretty("Alt"))]));
     Divider::vertical().show(ui);
     select_subject(ui, app, tl!("No subject found — try Object Selection or the Lasso"));
+    Divider::vertical().show(ui);
+    crate::panels::options_bar::selection::select_and_mask(ui);
     None
 }
 
@@ -84,3 +87,48 @@ pub(super) fn object_selection(ui: &mut egui::Ui, app: &mut PhotocraftApp) -> Op
     select_subject(ui, app, tl!("No subject found — try a rectangle around the object"));
     None
 }
+
+/// SAM 3.1 Selection's options: operation segments, Sample All Layers, Prompt, Threshold, Refine,
+/// Individual Masks, and the Detect button.
+pub(super) fn sam3_select(ui: &mut egui::Ui, app: &mut PhotocraftApp) -> Option<RangeAlert> {
+    crate::panels::options_bar::selection::operation_segments(app, ui, true);
+    Divider::vertical().show(ui);
+    let o = &mut app.ui.tool_options;
+    check(ui, tl!("Sample All Layers"), &mut o.sample_all.sam3_select, true, None);
+    Divider::vertical().show(ui);
+    TextField::new(Label::from(tl!("Prompt")), Bind::bound(&mut o.sam3_prompt)).show(ui);
+    let mut threshold = o.sam3_threshold;
+    let alert1 = number(tl!("Threshold"), &mut threshold, 0.0..=1.0).show(ui);
+    o.sam3_threshold = threshold;
+    let mut refine = o.sam3_refine as f32;
+    let alert2 = number(tl!("Refine"), &mut refine, 0.0..=5.0).whole().show(ui);
+    o.sam3_refine = refine.round().clamp(0.0, 5.0) as u8;
+    check(ui, tl!("Individual Masks"), &mut o.sam3_individual, true, None);
+    Divider::vertical().show(ui);
+    let can_detect = !o.sam3_prompt.trim().is_empty();
+    if clicked(&Button::new(tl!("Detect")).enabled(can_detect).show(ui)) {
+        let prompt = o.sam3_prompt.clone();
+        let threshold = o.sam3_threshold;
+        let refine = o.sam3_refine;
+        let individual = o.sam3_individual;
+        let sample_all = o.sample_all.sam3_select;
+        let mode = crate::canvas::tool_feedback::mode_name(crate::canvas::tool_feedback::selection_op(
+            Tool::Sam3Select,
+            app.ui.selection_mode,
+            craft::keymap::Mods::NONE,
+            crate::canvas::tool_feedback::has_selection(app),
+        ));
+        let params = json!({
+            "prompt": prompt,
+            "threshold": threshold,
+            "refine": refine,
+            "individual": individual,
+            "sampleAllLayers": sample_all,
+            "mode": mode,
+        });
+        let r = app.run("select.sam3", params);
+        crate::retouch_ui::report_smart_result(app, r, tl!("Nothing matched the prompt"));
+    }
+    hint(ui, tl!("or click, or drag a box"));
+    alert1.out_of_range.or(alert2.out_of_range)
+}
```

**What each hunk does.**

1. imports `Bind`, `Label`, `TextField`, `number` and `Tool`, which the SAM row needs.
2. gives the Quick Selection its Select and Mask entry beside the operation segments.
3. keeps the SAM options row: Sample All Layers, Prompt, Threshold, Refine, Individual Masks and Detect.

**If this were reverted / how it is checked.** `cargo nextest run -p photocraft-ui-egui -E 'test(an_idle_options_bar_leaves_the_tool_options_revision)'` reports `idle frames moved the tool options under [Sam3Select]`.

### `crates/ui-egui/src/tools/retouch_ui/tests.rs`

**Role.** the Retouch tools' trials: gestures, modes and the params they send.

**The diff, verbatim** (`git diff 581eb807b^2 581eb807b -- crates/ui-egui/src/tools/retouch_ui/tests.rs`):

```diff
diff --git a/crates/ui-egui/src/tools/retouch_ui/tests.rs b/crates/ui-egui/src/tools/retouch_ui/tests.rs
index 6b8e32d9c..9fd5b5a46 100644
--- a/crates/ui-egui/src/tools/retouch_ui/tests.rs
+++ b/crates/ui-egui/src/tools/retouch_ui/tests.rs
@@ -822,6 +822,176 @@ fn history_source_caption_names_the_source() {
     assert!(matches!(source_caption(&app), SourceCaption::Deleted), "discarded redo state");
 }
 
+/// The reference application starts the Quick Selection in New and Reset Tool puts it back to
+/// New, so the tool's mode is New unless the user picks another one.
+#[test]
+fn the_quick_selection_starts_in_new_and_reset_returns_to_it() {
+    let mut app = app();
+    app.ui.tool = Tool::QuickSelection;
+    assert_eq!(app.ui.tool_options.quick_selection_mode, 0, "the tool starts in New mode");
+    app.ui.tool_options.quick_selection_mode = 1;
+    crate::menus::invoke(&mut app, &egui::Context::default(), crate::tool_reset::RESET, json!({})).unwrap();
+    assert_eq!(app.ui.tool_options.quick_selection_mode, 0, "Reset Tool returns to New mode");
+}
+
+/// Shift always adds and Alt (also Shift+Alt) always subtracts, whatever the Quick Selection's
+/// own mode is (#11837).
+#[test]
+fn the_quick_selection_modifiers_follow_the_mode() {
+    use crate::canvas::tool_feedback::{mode_name, selection_op};
+    const SHIFT: Mods = Mods { shift: true, ..Mods::NONE };
+    const ALT: Mods = Mods { alt: true, ..Mods::NONE };
+    const BOTH: Mods = Mods { shift: true, alt: true, ..Mods::NONE };
+    let m = |bar: u8, mods: Mods| mode_name(selection_op(Tool::QuickSelection, bar, mods, true));
+    assert_eq!(m(1, Mods::NONE), "add");
+    assert_eq!(m(1, ALT), "subtract");
+    assert_eq!(m(1, SHIFT), "add");
+    assert_eq!(m(2, Mods::NONE), "subtract");
+    assert_eq!(m(2, SHIFT), "add", "Shift always adds");
+    assert_eq!(m(2, ALT), "subtract", "Alt always subtracts");
+    assert_eq!(m(2, BOTH), "subtract", "Alt subtracts even with Shift");
+    assert_eq!(m(0, Mods::NONE), "replace");
+    assert_eq!(m(0, ALT), "subtract");
+    assert_eq!(m(0, SHIFT), "add");
+}
+
+/// New flips to Add after its first stroke on an empty selection; with a selection there, a
+/// Shift stroke adds and keeps New, and a plain stroke replaces it and keeps New too.
+#[test]
+fn a_new_mode_stroke_flips_to_add_only_on_the_first_one() {
+    use crate::canvas::{ToolEvent, tool_event};
+    use egui::Modifiers;
+    let mut app = app();
+    app.ui.tool = Tool::QuickSelection;
+    // A filled patch gives the Quick Selection something to segment.
+    app.run("layer.new.layer", json!({"name": "Patch"})).unwrap();
+    app.run("select.rect", json!({"x": 60, "y": 20, "width": 20, "height": 20})).unwrap();
+    app.run("edit.fill", json!({"color": "#ff0000"})).unwrap();
+    app.run("select.deselect", json!({})).unwrap();
+
+    // The first stroke on an empty selection switches New to Add.
+    app.ui.tool_options.quick_selection_mode = 0;
+    tool_event(&mut app, ToolEvent::Down { x: 70.0, y: 30.0, pressure: 1.0 }, Modifiers::NONE);
+    tool_event(&mut app, ToolEvent::Up { x: 70.0, y: 30.0 }, Modifiers::NONE);
+    assert_eq!(app.ui.tool_options.quick_selection_mode, 1, "the first stroke on an empty selection switches to Add");
+
+    // With a selection there, a plain stroke replaces it and keeps New.
+    app.run("select.rect", json!({"x": 10, "y": 10, "width": 30, "height": 30})).unwrap();
+    app.ui.tool_options.quick_selection_mode = 0;
+    tool_event(&mut app, ToolEvent::Down { x: 70.0, y: 30.0, pressure: 1.0 }, Modifiers::NONE);
+    tool_event(&mut app, ToolEvent::Up { x: 70.0, y: 30.0 }, Modifiers::NONE);
+    assert_eq!(app.ui.tool_options.quick_selection_mode, 0, "a stroke with a selection there keeps New");
+}
+
+#[test]
+/// An empty selection (Select All followed by Inverse) is no selection for the New-to-Add
+/// transition: the stroke that makes one flips New to Add instead of keeping New.
+fn an_empty_selection_counts_as_none_for_the_new_to_add_transition() {
+    use crate::canvas::{ToolEvent, tool_event};
+    use egui::Modifiers;
+    let mut app = app();
+    app.ui.tool = Tool::QuickSelection;
+    app.run("layer.new.layer", json!({"name": "Patch"})).unwrap();
+    app.run("select.rect", json!({"x": 60, "y": 20, "width": 20, "height": 20})).unwrap();
+    app.run("edit.fill", json!({"color": "#ff0000"})).unwrap();
+    // Select All followed by Inverse leaves `Some(empty)` when the layer fills the canvas.
+    app.run("select.all", json!({})).unwrap();
+    app.run("select.inverse", json!({})).unwrap();
+    let sel_bounds = app.session.active().unwrap().doc.selection.as_ref().map(|s| s.content_bounds());
+    assert_eq!(sel_bounds, Some(photocraft_geom::Rect::EMPTY), "the inverted selection is empty");
+    assert!(app.session.active().unwrap().doc.selection.is_some(), "an empty selection is still Some");
+
+    // The stroke that makes a selection out of the empty one flips New to Add.
+    app.ui.tool_options.quick_selection_mode = 0;
+    tool_event(&mut app, ToolEvent::Down { x: 70.0, y: 30.0, pressure: 1.0 }, Modifiers::NONE);
+    tool_event(&mut app, ToolEvent::Up { x: 70.0, y: 30.0 }, Modifiers::NONE);
+    assert_eq!(app.ui.tool_options.quick_selection_mode, 1, "the stroke out of the empty selection switches to Add");
+}
+
+/// In New mode a press inside the selection acts on it, as the other selection tools do
+/// (#11782): a click deselects it, one Deselect step, and a drag moves the outline by the
+/// pointer's delta, one Move Selection step. The mode stays New in both cases, and no pixels
+/// are painted.
+#[test]
+fn a_click_or_drag_inside_the_selection_in_new_mode_acts_on_the_selection() {
+    use crate::canvas::{ToolEvent, tool_event};
+    use egui::Modifiers;
+    let mut app = app();
+    app.ui.tool = Tool::QuickSelection;
+    app.run("select.rect", json!({"x": 10, "y": 10, "width": 30, "height": 30})).unwrap();
+    app.ui.tool_options.quick_selection_mode = 0;
+    let bounds = |app: &PhotocraftApp| app.session.active().unwrap().doc.selection.as_ref().map(|s| s.content_bounds());
+
+    // A click inside the selection deselects it.
+    let steps = app.session.active().unwrap().history.past_len();
+    tool_event(&mut app, ToolEvent::Down { x: 20.0, y: 20.0, pressure: 1.0 }, Modifiers::NONE);
+    tool_event(&mut app, ToolEvent::Up { x: 20.0, y: 20.0 }, Modifiers::NONE);
+    assert!(bounds(&app).is_none(), "a click inside the selection deselects it");
+    assert_eq!(app.ui.tool_options.quick_selection_mode, 0, "the mode stays New");
+    let labels: Vec<String> = app.session.active().unwrap().history.entries().into_iter().skip(steps + 1).map(|e| e.to_string()).collect();
+    assert_eq!(labels.first().map(String::as_str), Some("Deselect"), "{labels:?}");
+
+    // A drag inside the selection moves its outline, leaving the pixels where they are.
+    app.ui.extras.snap = false;
+    app.run("select.rect", json!({"x": 10, "y": 10, "width": 30, "height": 30})).unwrap();
+    app.ui.tool_options.quick_selection_mode = 0;
+    let steps = app.session.active().unwrap().history.past_len();
+    tool_event(&mut app, ToolEvent::Down { x: 20.0, y: 20.0, pressure: 1.0 }, Modifiers::NONE);
+    tool_event(&mut app, ToolEvent::Move { x: 30.0, y: 20.0, pressure: 1.0 }, Modifiers::NONE);
+    assert_eq!(crate::canvas::tools::selection::shown_offset(&app), Some((10, 0)), "the marching ants follow the drag live");
+    tool_event(&mut app, ToolEvent::Up { x: 30.0, y: 20.0 }, Modifiers::NONE);
+    let moved = bounds(&app).expect("the selection is still there");
+    assert_eq!((moved.width(), moved.height()), (30, 30), "the outline kept its size");
+    assert_eq!((moved.x0, moved.y0), (20, 10), "the outline moved with the pointer");
+    assert_eq!(app.ui.tool_options.quick_selection_mode, 0, "the mode stays New");
+    let labels: Vec<String> = app.session.active().unwrap().history.entries().into_iter().skip(steps + 1).map(|e| e.to_string()).collect();
+    assert_eq!(labels.first().map(String::as_str), Some("Move Selection"), "{labels:?}");
+}
+
+/// The Quick Selection's brush cursor carries its effective selection-mode badge (+ or −), and
+/// shows the selection-move cursor when hovering or dragging inside the selection in New mode.
+#[test]
+fn the_quick_selection_cursor_shows_its_mode_badge_and_outline_drag_cursor() {
+    use crate::canvas::{ToolEvent, tool_event, tool_event_response};
+    use craft::behavior::{Cursor, CursorBadge, PaintedCursor};
+    use egui::Modifiers;
+
+    let mut app = app();
+    app.ui.tool = Tool::QuickSelection;
+    app.run("paint.pencil", json!({"points": [[70, 30]], "size": 20, "color": "#ff0000"})).unwrap();
+    let cursor_at = |app: &mut PhotocraftApp, x: f64, y: f64, mods: Modifiers| tool_event_response(app, ToolEvent::Move { x, y, pressure: 1.0 }, mods).cursor;
+    let badge_of = |c: Option<Cursor>| match c {
+        Some(Cursor::Painted(PaintedCursor::Circle { badge, .. } | PaintedCursor::Outline { badge, .. })) => badge,
+        Some(Cursor::Painted(PaintedCursor::Badged { badge, .. })) => Some(badge),
+        _ => None,
+    };
+
+    // New mode with nothing selected: no badge, and Alt shows no − badge either.
+    app.ui.tool_options.quick_selection_mode = 0;
+    assert_eq!(badge_of(cursor_at(&mut app, 70.0, 30.0, Modifiers::NONE)), None);
+    assert_eq!(badge_of(cursor_at(&mut app, 70.0, 30.0, Modifiers::ALT)), None, "nothing selected to subtract from");
+    assert_eq!(badge_of(cursor_at(&mut app, 70.0, 30.0, Modifiers::SHIFT)), Some(CursorBadge::Add));
+
+    // Add mode: + badge, − with Alt.
+    app.ui.tool_options.quick_selection_mode = 1;
+    assert_eq!(badge_of(cursor_at(&mut app, 70.0, 30.0, Modifiers::NONE)), Some(CursorBadge::Add));
+    assert_eq!(badge_of(cursor_at(&mut app, 70.0, 30.0, Modifiers::ALT)), Some(CursorBadge::Subtract));
+
+    // Subtract mode: − badge, + with Shift.
+    app.ui.tool_options.quick_selection_mode = 2;
+    assert_eq!(badge_of(cursor_at(&mut app, 70.0, 30.0, Modifiers::NONE)), Some(CursorBadge::Subtract));
+    assert_eq!(badge_of(cursor_at(&mut app, 70.0, 30.0, Modifiers::SHIFT)), Some(CursorBadge::Add));
+
+    // With a selection in New mode, hovering inside the selection shows the selection-move cursor, not the brush tip.
+    app.run("select.rect", json!({"x": 10, "y": 10, "width": 30, "height": 30})).unwrap();
+    app.ui.tool_options.quick_selection_mode = 0;
+    let inside = cursor_at(&mut app, 20.0, 20.0, Modifiers::NONE);
+    let outside = cursor_at(&mut app, 70.0, 30.0, Modifiers::NONE);
+    assert_ne!(inside, outside, "hovering inside the selection in New mode switches to the selection-move cursor");
+    // Ensure a clean release state.
+    let _ = tool_event;
+}
+
 /// An empty Clone Source slot refuses to paint, as with no source at all: the ⌥-click point left
 /// in `ui.clone_source` carries no document, so it must not be read from another document.
 #[test]
@@ -835,6 +1005,119 @@ fn an_empty_clone_source_slot_refuses_to_paint() {
     assert!(crate::retouch_ui::clone_params(&app).is_none());
 }
 
+#[test]
+fn sam_selection_sends_a_point_for_a_click_and_a_box_for_a_drag() {
+    let mut app = app();
+    app.ui.tool = Tool::Sam3Select;
+    app.ui.tool_options.sam3_threshold = 0.35;
+    app.ui.tool_options.sam3_refine = 4;
+    app.ui.tool_options.sam3_individual = true;
+    let m = craft::keymap::Mods::NONE;
+    let click = sam3_select_params(&app, [10.0, 20.0], [11.0, 20.0], m);
+    assert_eq!(click["point"], json!([11.0, 20.0]), "{click}");
+    assert!(click.get("box").is_none(), "{click}");
+    assert_eq!(click["threshold"], json!(0.35_f32), "{click}");
+    assert_eq!(click["refine"], json!(4_u8), "{click}");
+    assert_eq!(click["individual"], json!(true), "{click}");
+    let drag = sam3_select_params(&app, [60.0, 55.0], [5.0, 5.0], m);
+    assert_eq!(drag["box"], json!([5.0, 5.0, 55.0, 50.0]), "{drag}");
+    assert!(drag.get("point").is_none(), "{drag}");
+    // ⇧⌥ intersects whatever the options-bar mode, exactly as the other selection tools do.
+    let both = sam3_select_params(&app, [0.0, 0.0], [40.0, 40.0], craft::keymap::Mods { shift: true, alt: true, ..craft::keymap::Mods::NONE });
+    assert_eq!(both["mode"], json!("intersect"), "{both}");
+}
+
+/// The tool end to end through the canvas gesture: a box around the disc runs `select.sam3` and
+/// leaves a selection on the canvas. Needs the built helper and its pack
+/// (`CRAFT_SAM3_HELPER` / `CRAFT_SAM3_PACK`).
+#[test]
+#[ignore = "needs the built SAM 3.1 helper and its HSWQ pack (CRAFT_SAM3_HELPER / CRAFT_SAM3_PACK)"]
+fn sam_selection_box_selects_the_disc_end_to_end() {
+    let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), crate::Services::default());
+    app.session.execute("file.new", json!({"width": 256, "height": 256, "depth": 8})).unwrap();
+    app.sync_views();
+    app.session
+        .edit("paint", |doc, _| {
+            let bg = doc.layers[0].surface_mut().unwrap();
+            let n = bg.channels();
+            let mut data = Vec::with_capacity(256 * 256 * n);
+            for y in 0..256i32 {
+                for x in 0..256i32 {
+                    let inside = (x - 128).pow(2) + (y - 128).pow(2) <= 40 * 40;
+                    let c = if inside { [0.1f32, 0.7, 0.2] } else { [1.0, 1.0, 1.0] };
+                    data.extend_from_slice(&c[..n.min(3)]);
+                    if n == 4 {
+                        data.push(1.0);
+                    }
+                }
+            }
+            bg.write_region(photocraft_geom::Rect::new(0, 0, 256, 256), &data);
+            Ok(())
+        })
+        .unwrap();
+    app.ui.tool = Tool::Sam3Select;
+    app.ui.tool_options.sam3_prompt.clear();
+    app.ui.tool_options.sam3_refine = 1;
+    let m = egui::Modifiers::NONE;
+    tool_event(&mut app, ToolEvent::Down { x: 128.0, y: 128.0, pressure: 1.0 }, m);
+    tool_event(&mut app, ToolEvent::Up { x: 128.0, y: 128.0 }, m);
+    assert!(!app.ui.status_error, "{}", app.ui.status);
+    assert_eq!(app.session.journal.last().map(|(id, _)| id.as_str()), Some("select.sam3"));
+    let bounds = app.session.active().expect("document").doc.selection.as_ref().expect("a selection").content_bounds();
+    assert!(bounds.width() > 40 && bounds.width() < 160, "bounds {bounds:?}");
+    // A drag is the box prompt: it runs too (the box path's region is the helper's, not the UI's).
+    tool_event(&mut app, ToolEvent::Down { x: 70.0, y: 70.0, pressure: 1.0 }, m);
+    tool_event(&mut app, ToolEvent::Move { x: 185.0, y: 185.0, pressure: 1.0 }, m);
+    tool_event(&mut app, ToolEvent::Up { x: 185.0, y: 185.0 }, m);
+    assert!(!app.ui.status_error, "{}", app.ui.status);
+    assert_eq!(app.session.journal.last().map(|(id, _)| id.as_str()), Some("select.sam3"));
+}
+
+/// The SAM Selection strings resolve in Japanese once craft-i18n has the rows: the label, the
+/// options bar and both status messages.
+#[test]
+#[ignore = "needs SAM Selection rows in craft-i18n checkout"]
+fn sam_selection_strings_are_translated_in_japanese() {
+    use crate::i18n::{Lang, tr};
+    // Nothing to look up without a craft-i18n checkout; with one the catalog must be embedded.
+    if std::env::var_os("CRAFT_I18N_DIR").is_none_or(|d| d.is_empty()) {
+        return;
+    }
+    assert!(crate::i18n::catalogs_embedded(), "craft-i18n is configured but no catalog is embedded");
+    let ja = Lang::from_code("ja").expect("a Japanese catalog");
+    for (en, want) in [
+        ("SAM Selection Tool", "SAM選択ツール"),
+        ("Prompt", "プロンプト"),
+        ("Detect", "検出"),
+        ("Individual Masks", "個別マスク"),
+        ("SAM found nothing there — try a tighter box", "SAM はそこに何も見つけられませんでした — より狭いボックスで試してください"),
+        // Shared words come from craft-i18n's `common.tsv`, not this app's catalog.
+        ("Threshold", "しきい値"),
+        ("Refine", "調整"),
+    ] {
+        assert_eq!(tr(ja, en), want, "{en}");
+    }
+}
+
+/// A text prompt typed in the options bar wins over the gesture: the engine's text path covers
+/// the whole document, so a click does not send a point prompt beside it, and neither does a drag
+/// send a box the engine would ignore.
+#[test]
+fn a_typed_prompt_wins_over_the_gesture() {
+    let mut app = app();
+    app.ui.tool = Tool::Sam3Select;
+    app.ui.tool_options.sam3_prompt = "face".into();
+    let m = craft::keymap::Mods::NONE;
+    let click = sam3_select_params(&app, [10.0, 20.0], [10.0, 20.0], m);
+    assert_eq!(click["prompt"], json!("face"), "{click}");
+    assert!(click.get("point").is_none(), "{click}");
+    let drag = sam3_select_params(&app, [5.0, 5.0], [60.0, 55.0], m);
+    assert_eq!(drag["prompt"], json!("face"), "{drag}");
+    assert!(drag.get("box").is_none(), "{drag}");
+    // Clearing the prompt gives the gestures back.
+    app.ui.tool_options.sam3_prompt.clear();
+    let click = sam3_select_params(&app, [10.0, 20.0], [10.0, 20.0], m);
+    assert_eq!(click["point"], json!([10.0, 20.0]), "{click}");
 /// #13096: a Red Eye press outside the selection does nothing and says nothing, as in Photoshop:
 /// no status message, no error, no History state and no pixel change.
 #[test]
```

**What each hunk does.**

1. keeps the Quick Selection trials: New at start, Reset Tool back to New, the modifier table, New-to-Add, the empty selection, click and drag inside, and the mode badge.
2. keeps the SAM trials: point and box params, the end-to-end box trial, the Japanese strings and the typed prompt winning.

**If this were reverted / how it is checked.** `cargo nextest run -p photocraft-ui-egui -E 'test(<the trials this file holds>)'` and `cargo xtask ci lint` name the loss.

### `crates/ui-egui/src/tools/tool_reset.rs`

**Role.** Reset Tool: which option keys each tool resets.

**The diff, verbatim** (`git diff 581eb807b^2 581eb807b -- crates/ui-egui/src/tools/tool_reset.rs`):

```diff
diff --git a/crates/ui-egui/src/tools/tool_reset.rs b/crates/ui-egui/src/tools/tool_reset.rs
index 9b2f5bb21..ed65461ea 100644
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

**What each hunk does.**

1. keeps `quick_selection_mode` and the four SAM keys in Reset Tool, so Reset returns the rows to their defaults.
2. keeps LaMa Cleaner in the tools whose brush options Reset Tool reaches.

**If this were reverted / how it is checked.** `cargo nextest run -p photocraft-ui-egui -E 'test(<the trials this file holds>)'` and `cargo xtask ci lint` name the loss.

### `crates/ui-egui/src/tools/type_tool.rs`

**Role.** the Type tool's module list.

**The diff, verbatim** (`git diff 581eb807b^2 581eb807b -- crates/ui-egui/src/tools/type_tool.rs`):

```diff
diff --git a/crates/ui-egui/src/tools/type_tool.rs b/crates/ui-egui/src/tools/type_tool.rs
index 49c925323..3ed9f91fd 100644
--- a/crates/ui-egui/src/tools/type_tool.rs
+++ b/crates/ui-egui/src/tools/type_tool.rs
@@ -139,5 +139,8 @@ mod mask_escape_tests;
 #[cfg(test)]
 mod unchanged_edit_history_tests;
 
+#[cfg(test)]
+mod font_names_tests;
+
 #[cfg(test)]
 mod vertical_mask_alt_drag_tests;
```

**What each hunk does.**

1. registers `font_names_tests`.

**If this were reverted / how it is checked.** `cargo nextest run -p photocraft-ui-egui -E 'test(<the trials this file holds>)'` and `cargo xtask ci lint` name the loss.

### `crates/ui-egui/src/tools/type_tool/fonts.rs`

**Role.** the Type tool's font choices: which families the picker lists and by which names.

**The diff, verbatim** (`git diff 581eb807b^2 581eb807b -- crates/ui-egui/src/tools/type_tool/fonts.rs`):

```diff
diff --git a/crates/ui-egui/src/tools/type_tool/fonts.rs b/crates/ui-egui/src/tools/type_tool/fonts.rs
index 034d72a3f..555abe9ba 100644
--- a/crates/ui-egui/src/tools/type_tool/fonts.rs
+++ b/crates/ui-egui/src/tools/type_tool/fonts.rs
@@ -95,6 +95,13 @@ pub(super) fn preview_text(app: &PhotocraftApp) -> Option<String> {
     (!line.is_empty()).then_some(line)
 }
 
+/// Show the families by their English names. This is the one place the fork differs from the upstream
+/// default: the preference decides, except in a Japanese interface, which lists the families by their
+/// Japanese names (ＭＳ Ｐ明朝, not MS PMincho) as Photoshop does, whatever the preference says.
+pub(super) fn english_font_names(preference: bool, language: &str) -> bool {
+    preference && !language.starts_with("ja")
+}
+
 /// The font picker's most recent families the settings may hold, whatever the preference says.
 const MAX_RECENT_FONTS: usize = 50;
 
@@ -127,7 +134,7 @@ pub(crate) fn font_state(app: &PhotocraftApp) -> FontSourceState {
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

1. keeps the fork's one rule for family names: the preference decides except in a Japanese interface, which lists Japanese names.
2. routes `font_state` through that rule with the current language's code.

**If this were reverted / how it is checked.** `cargo nextest run -p photocraft-ui-egui -E 'test(<the trials this file holds>)'` and `cargo xtask ci lint` name the loss.

## The repair commit `359743c5f`

### `Cargo.lock`

**Role.** the resolved dependency graph, generated from `Cargo.toml`.

**The diff, verbatim** (`git diff 581eb807b..359743c5f -- Cargo.lock`) is 91 hunks of one
kind: every `source = "git+https://github.com/storytold/craft-libs?rev=..."` line moves to
`51fcc3b04`, one package entry splits so the two isolated pins keep their own revision, and the
dependency lists that name `craft-pcraft-format` gain their disambiguating source. The file is
generated; `cargo fetch` writes it and `cargo build --release` accepted it.

```text
Cargo.lock | 203 ++++++++++++++++++++++++++++++++-----------------------------
 1 file changed, 108 insertions(+), 95 deletions(-)
```

**What each hunk does.** They are the pin rewrite above; the one structural change is the
second `craft-pcraft-format` entry at `51fcc3b04`, needed because `crates/engine-core` and the
workspace pin the crate at two revisions.

**If this were reverted / how it is checked.** `cargo build --release -p photocraft` fails
with a lock whose sources disagree with `Cargo.toml`; `cargo fetch` rewrites it.

### `crates/ui-egui/src/canvas/canvas_tool_menu/rectangle_menu_tests.rs`

**Role.** the rectangle menu's trials, which build a shape geometry directly.

**The diff, verbatim** (`git diff 581eb807b..359743c5f -- crates/ui-egui/src/canvas/canvas_tool_menu/rectangle_menu_tests.rs`):

```diff
diff --git a/crates/ui-egui/src/canvas/canvas_tool_menu/rectangle_menu_tests.rs b/crates/ui-egui/src/canvas/canvas_tool_menu/rectangle_menu_tests.rs
index 1b97c92f6..0074f4883 100644
--- a/crates/ui-egui/src/canvas/canvas_tool_menu/rectangle_menu_tests.rs
+++ b/crates/ui-egui/src/canvas/canvas_tool_menu/rectangle_menu_tests.rs
@@ -77,7 +77,7 @@ fn shape_layer_filled(tool: Tool, filled: bool) -> Box<PhotocraftApp> {
     app.ui.tool_options.shape_fill = filled;
     app.ui.tool_options.stroke_width = 2.0;
     let rect = craft::behavior::kurbo::Rect::new(40.0, 40.0, 120.0, 120.0);
-    let shape = if tool == Tool::Triangle { ShapeGeometry::Triangle { rect } } else { ShapeGeometry::Rect { rect, radii: [0.0; 4] } };
+    let shape = if tool == Tool::Triangle { ShapeGeometry::Triangle { rect, corner_radius: 0.0 } } else { ShapeGeometry::Rect { rect, radii: [0.0; 4] } };
     crate::vector_ui::create_geometry(&mut app, &shape, crate::vector_ui::kind_of(tool), None).unwrap();
     let st = app.session.active().unwrap();
     assert_eq!(st.doc.layer(st.active_layer.unwrap()).unwrap().content.kind_name(), "Shape", "{tool:?} made a shape layer");
```

**What each hunk does.**

1. the trial builds `ShapeGeometry::Triangle` directly, and the geometry now carries `corner_radius`; it passes 0.0, the sharp triangle the trial draws.

**If this were reverted / how it is checked.** `cargo nextest run -p photocraft-ui-egui -E 'test(<the trials this file holds>)'` and `cargo xtask ci lint` name the loss.

### `crates/ui-egui/src/tools/retouch_ui/bar/heal.rs`

**Role.** the Retouch options bar's healing family (Clone, Heal, Patch, LaMa Cleaner).

**The diff, verbatim** (`git diff 581eb807b..359743c5f -- crates/ui-egui/src/tools/retouch_ui/bar/heal.rs`):

```diff
diff --git a/crates/ui-egui/src/tools/retouch_ui/bar/heal.rs b/crates/ui-egui/src/tools/retouch_ui/bar/heal.rs
index 52fdec71c..dedb05590 100644
--- a/crates/ui-egui/src/tools/retouch_ui/bar/heal.rs
+++ b/crates/ui-egui/src/tools/retouch_ui/bar/heal.rs
@@ -154,7 +154,7 @@ pub(super) fn clone(ui: &mut egui::Ui, app: &mut PhotocraftApp) -> Option<RangeA
 /// The options of LaMa Cleaner.
 pub(super) fn lama_cleaner(ui: &mut egui::Ui, app: &mut PhotocraftApp) -> Option<RangeAlert> {
     let mut size = app.session.tools.brush.size.max(1.0);
-    let alert = number(tl!("Size:"), &mut size, 1.0..=500.0).unit("px").whole().popup_slider().show(ui);
+    let alert = number(tl!("Size:"), Bind::bound(&mut size), 1.0..=500.0).unit("px").whole().popup_slider().show(ui);
     app.session.tools.brush.size = size;
     alert.out_of_range
 }
```

**What each hunk does.**

1. `number` takes `Bind<'_, f32>` instead of `&mut f32`, so the LaMa Cleaner Size row hands it `Bind::bound(&mut size)`.

**If this were reverted / how it is checked.** `cargo nextest run -p photocraft-ui-egui -E 'test(<the trials this file holds>)'` and `cargo xtask ci lint` name the loss.

### `crates/ui-egui/src/tools/retouch_ui/bar/select.rs`

**Role.** the Retouch options bar's selection family (Quick Selection, Object Selection, SAM Selection).

**The diff, verbatim** (`git diff 581eb807b..359743c5f -- crates/ui-egui/src/tools/retouch_ui/bar/select.rs`):

```diff
diff --git a/crates/ui-egui/src/tools/retouch_ui/bar/select.rs b/crates/ui-egui/src/tools/retouch_ui/bar/select.rs
index 47c5a0d67..5c40911f7 100644
--- a/crates/ui-egui/src/tools/retouch_ui/bar/select.rs
+++ b/crates/ui-egui/src/tools/retouch_ui/bar/select.rs
@@ -93,25 +93,28 @@ pub(super) fn object_selection(ui: &mut egui::Ui, app: &mut PhotocraftApp) -> Op
 pub(super) fn sam3_select(ui: &mut egui::Ui, app: &mut PhotocraftApp) -> Option<RangeAlert> {
     crate::panels::options_bar::selection::operation_segments(app, ui, true);
     Divider::vertical().show(ui);
-    let o = &mut app.ui.tool_options;
-    check(ui, tl!("Sample All Layers"), &mut o.sample_all.sam3_select, true, None);
+    let refine_before = app.ui.tool_options.sam3_refine as f32;
+    let (sample_all, prompt, threshold, individual) = bind_tracked!(
+        app.ui.tool_options => sample_all.sam3_select, sam3_prompt, sam3_threshold, sam3_individual
+    );
+    check(ui, tl!("Sample All Layers"), sample_all, true, None);
     Divider::vertical().show(ui);
-    TextField::new(Label::from(tl!("Prompt")), Bind::bound(&mut o.sam3_prompt)).show(ui);
-    let mut threshold = o.sam3_threshold;
-    let alert1 = number(tl!("Threshold"), &mut threshold, 0.0..=1.0).show(ui);
-    o.sam3_threshold = threshold;
-    let mut refine = o.sam3_refine as f32;
-    let alert2 = number(tl!("Refine"), &mut refine, 0.0..=5.0).whole().show(ui);
-    o.sam3_refine = refine.round().clamp(0.0, 5.0) as u8;
-    check(ui, tl!("Individual Masks"), &mut o.sam3_individual, true, None);
+    TextField::new(Label::from(tl!("Prompt")), prompt).show(ui);
+    let alert1 = number(tl!("Threshold"), threshold, 0.0..=1.0).show(ui);
+    let mut refine = refine_before;
+    let alert2 = number(tl!("Refine"), Bind::bound(&mut refine), 0.0..=5.0).whole().show(ui);
+    check(ui, tl!("Individual Masks"), individual, true, None);
     Divider::vertical().show(ui);
-    let can_detect = !o.sam3_prompt.trim().is_empty();
+    let rounded = refine.round().clamp(0.0, 5.0) as u8;
+    if app.ui.tool_options.sam3_refine != rounded {
+        app.ui.tool_options.sam3_refine = rounded;
+    }
+    let (prompt, threshold, refine, individual, sample_all) = {
+        let o = &app.ui.tool_options;
+        (o.sam3_prompt.clone(), o.sam3_threshold, o.sam3_refine, o.sam3_individual, o.sample_all.sam3_select)
+    };
+    let can_detect = !prompt.trim().is_empty();
     if clicked(&Button::new(tl!("Detect")).enabled(can_detect).show(ui)) {
-        let prompt = o.sam3_prompt.clone();
-        let threshold = o.sam3_threshold;
-        let refine = o.sam3_refine;
-        let individual = o.sam3_individual;
-        let sample_all = o.sample_all.sam3_select;
         let mode = crate::canvas::tool_feedback::mode_name(crate::canvas::tool_feedback::selection_op(
             Tool::Sam3Select,
             app.ui.selection_mode,
```

**What each hunk does.**

1. the SAM row binds every field through `bind_tracked!` and reads them back through an immutable borrow: an idle frame writes nothing, so the options revision does not move.

**If this were reverted / how it is checked.** `cargo nextest run -p photocraft-ui-egui -E 'test(an_idle_options_bar_leaves_the_tool_options_revision)'` reports `idle frames moved the tool options under [Sam3Select]`.

### `crates/ui-egui/src/tools/retouch_ui/tests.rs`

**Role.** the Retouch tools' trials: gestures, modes and the params they send.

**The diff, verbatim** (`git diff 581eb807b..359743c5f -- crates/ui-egui/src/tools/retouch_ui/tests.rs`):

```diff
diff --git a/crates/ui-egui/src/tools/retouch_ui/tests.rs b/crates/ui-egui/src/tools/retouch_ui/tests.rs
index 9fd5b5a46..a4b734eb9 100644
--- a/crates/ui-egui/src/tools/retouch_ui/tests.rs
+++ b/crates/ui-egui/src/tools/retouch_ui/tests.rs
@@ -1118,6 +1118,8 @@ fn a_typed_prompt_wins_over_the_gesture() {
     app.ui.tool_options.sam3_prompt.clear();
     let click = sam3_select_params(&app, [10.0, 20.0], [10.0, 20.0], m);
     assert_eq!(click["point"], json!([10.0, 20.0]), "{click}");
+}
+
 /// #13096: a Red Eye press outside the selection does nothing and says nothing, as in Photoshop:
 /// no status message, no error, no History state and no pixel change.
 #[test]
```

**What each hunk does.**

1. the merge kept both sides' trials; this brace closes the last one of the first side, which the shared trailer had left open.

**If this were reverted / how it is checked.** `cargo nextest run -p photocraft-ui-egui -E 'test(<the trials this file holds>)'` and `cargo xtask ci lint` name the loss.

### `crates/ui-egui/src/tools/vector_ui/shapes.rs`

**Role.** the vector tools' conversion of a live gesture into the framework's `ShapeGeometry` and into `shape.create` params.

**The diff, verbatim** (`git diff 581eb807b..359743c5f -- crates/ui-egui/src/tools/vector_ui/shapes.rs`):

```diff
diff --git a/crates/ui-egui/src/tools/vector_ui/shapes.rs b/crates/ui-egui/src/tools/vector_ui/shapes.rs
index 8379ac8ac..e06bad7a6 100644
--- a/crates/ui-egui/src/tools/vector_ui/shapes.rs
+++ b/crates/ui-egui/src/tools/vector_ui/shapes.rs
@@ -155,7 +155,7 @@ fn snapped(geometry: &ShapeGeometry, keep_angle: bool, whole: bool) -> Option<Sh
     Some(match geometry {
         ShapeGeometry::Rect { rect, radii } => ShapeGeometry::Rect { rect: whole_box(rect, whole)?, radii: *radii },
         ShapeGeometry::Ellipse { rect } => ShapeGeometry::Ellipse { rect: whole_box(rect, whole)? },
-        ShapeGeometry::Triangle { rect } => ShapeGeometry::Triangle { rect: whole_box(rect, whole)? },
+        ShapeGeometry::Triangle { rect, corner_radius } => ShapeGeometry::Triangle { rect: whole_box(rect, whole)?, corner_radius: *corner_radius },
         ShapeGeometry::Polygon { rect, sides, star_ratio } => ShapeGeometry::Polygon { rect: whole_box(rect, whole)?, sides: *sides, star_ratio: *star_ratio },
         ShapeGeometry::Custom { rect, keep_proportions } => ShapeGeometry::Custom { rect: whole_box(rect, whole)?, keep_proportions: *keep_proportions },
         ShapeGeometry::Line { from, to, weight } => {
@@ -210,7 +210,13 @@ fn geometry_params(app: &PhotocraftApp, geometry: &ShapeGeometry, kind: ShapeKin
         ShapeGeometry::Rect { rect, .. } => json!({"kind": "rect", "rect": box_of(rect)}),
         ShapeGeometry::Ellipse { rect } => json!({"kind": "ellipse", "rect": box_of(rect)}),
         // The Triangle tool's triangle fills its box (#13350).
-        ShapeGeometry::Triangle { rect } => json!({"kind": "triangle", "rect": box_of(rect)}),
+        ShapeGeometry::Triangle { rect, corner_radius } => {
+            let mut p = json!({"kind": "triangle", "rect": box_of(rect)});
+            if *corner_radius > 0.0 {
+                p["cornerRadius"] = json!(corner_radius);
+            }
+            p
+        }
         ShapeGeometry::Polygon { rect, sides, star_ratio } => {
             let mut p = json!({"kind": if kind == ShapeKind::Star { "star" } else { "polygon" }, "rect": box_of(rect), "sides": sides});
             if kind == ShapeKind::Star || *star_ratio < 1.0 {
```

**What each hunk does.**

1. the whole-pixel conversion keeps `corner_radius` with the rounded box.
2. the `shape.create` params carry `cornerRadius` when it is above zero, the command's own field name.

**If this were reverted / how it is checked.** `cargo nextest run -p photocraft-ui-egui -E 'test(vector_ui) or test(rectangle_menu)'` fails on the rounded triangle, and `shape.create` loses `cornerRadius`.
