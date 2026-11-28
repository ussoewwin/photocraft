# The framework's kept code (craft-libs)

This round the merge itself kept **no** conflict resolution of the fork's: all four
conflicted files took upstream's side, because upstream's `51fcc3b04` carries its own fix of
each one (`#[prop(quantity = ...)]` in place of the fork's `props/point-size` marker, the
save-for-web focus recovery and the selection-click block deleted). Two exemption files the
fork had carried are gone with them
(`design/exceptions/craft-draw/transform_box.toml`,
`design/exceptions/craft-ui-patterns/preferences.toml`), and the other two are upstream's
`config_version = 1` stubs.

What the fork does keep is the **repair**, commit `023528394`: upstream's new
`runtime/egui/text_edit.rs` imports four names from `primitives`, one layer above it, and the
design lint named that fourteen times. The repair moves them to
`runtime/egui/text_ink.rs` and re-exports them from `primitives`, so no caller changes.

### `crates/design-lint/src/workspace.rs`

**Role.** a file this merge keeps.

**The diff, verbatim** (`git diff 02419852d..023528394 -- crates/design-lint/src/workspace.rs`):

```diff
diff --git a/crates/design-lint/src/workspace.rs b/crates/design-lint/src/workspace.rs
index 247454411..4437391c3 100644
--- a/crates/design-lint/src/workspace.rs
+++ b/crates/design-lint/src/workspace.rs
@@ -84,6 +84,7 @@ const CLOSED: &[&str] = &[
     "ToastEnd",         // built or matched in craft-ui-widgets
     "TrackLook",        // built or matched in craft-ui-widgets
     "VariantParts",     // built or matched in craft-ui-widgets
+    "ColorRange",       // built in craft-develop-model with a struct expression
 ];
 
 /// Permission keys whose only allowed location is craft-ui-contract's `runtime::egui`.
```

**What each hunk does.**

1. adds `ColorRange` to the closed list: a higher crate builds it with a struct expression, so `#[non_exhaustive]` is not the rule that fits it.

**If this were reverted / how it is checked.** `cargo nextest run -p photocraft-ui-egui -E 'test(<the trials this file holds>)'` and `cargo xtask ci lint` name the loss.

### `crates/ui-contract/src/primitives/text_input.rs`

**Role.** a file this merge keeps.

**The diff, verbatim** (`git diff 02419852d..023528394 -- crates/ui-contract/src/primitives/text_input.rs`):

```diff
diff --git a/crates/ui-contract/src/primitives/text_input.rs b/crates/ui-contract/src/primitives/text_input.rs
index dda34fb8d..6a3f94c5f 100644
--- a/crates/ui-contract/src/primitives/text_input.rs
+++ b/crates/ui-contract/src/primitives/text_input.rs
@@ -52,49 +52,7 @@ pub enum TabKey {
 /// Whether a pasted text is whole ([`TextOptions::whole_paste`]).
 type WholePaste = dyn Fn(&str) -> bool;
 
-/// The colour ranges of a text ([`TextOptions::color_ranges`]): called with the text each time the part lays it out,
-/// which is every frame the part shows. An app whose ranges are costly to find keeps its own cache.
-pub type ColorRanges = dyn Fn(&str) -> Vec<ColorRange>;
-
-/// The longest text, in bytes, a part colours: a longer one is laid out in its plain ink, and its
-/// [`ColorRanges`] is not called. It bounds the cost of an app's colouring (a highlighter whose time grows with the
-/// square of the length on crafted input) at the size of a long script.
-pub const COLORED_TEXT_LIMIT: usize = 16 * 1024;
-
-/// A theme colour for a stretch of a field's text. Each is the token's colour moved, where it has to be, to the text
-/// contrast minimum on the field's fill, so every ink reads in every theme and contrast.
-#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
-#[non_exhaustive]
-pub enum RangeInk {
-    /// `text.muted`.
-    Muted,
-    /// `accent.text`.
-    Accent,
-    /// `status.good.fg`.
-    Good,
-    /// `status.warn.fg`.
-    Warn,
-    /// `status.bad.fg`.
-    Bad,
-    /// `status.info.fg`.
-    Info,
-}
-
-/// One stretch of a text in an ink of its own: `range` is in bytes of the text the [`ColorRanges`] was given.
-#[derive(Clone, Debug, PartialEq, Eq)]
-pub struct ColorRange {
-    /// The bytes the ink covers.
-    pub range: Range<usize>,
-    /// Their ink.
-    pub ink: RangeInk,
-}
-
-impl ColorRange {
-    /// The bytes `range` in `ink`.
-    pub fn new(range: Range<usize>, ink: RangeInk) -> Self {
-        ColorRange { range, ink }
-    }
-}
+pub use crate::runtime::egui::text_ink::{COLORED_TEXT_LIMIT, ColorRange, ColorRanges, RangeInk};
 
 /// How a text part behaves beyond its codec.
 #[derive(Clone)]
```

**What each hunk does.**

1. replaces the definitions with a re-export of the four names from layer 2 (`runtime::egui::text_ink`), so every existing import path keeps working.

**If this were reverted / how it is checked.** `cargo nextest run -p photocraft-ui-egui -E 'test(<the trials this file holds>)'` and `cargo xtask ci lint` name the loss.

### `crates/ui-contract/src/runtime/egui/mod.rs`

**Role.** a file this merge keeps.

**The diff, verbatim** (`git diff 02419852d..023528394 -- crates/ui-contract/src/runtime/egui/mod.rs`):

```diff
diff --git a/crates/ui-contract/src/runtime/egui/mod.rs b/crates/ui-contract/src/runtime/egui/mod.rs
index 340b21a69..1b2fc913a 100644
--- a/crates/ui-contract/src/runtime/egui/mod.rs
+++ b/crates/ui-contract/src/runtime/egui/mod.rs
@@ -30,6 +30,7 @@ pub mod style;
 #[cfg(feature = "testing")]
 pub(crate) mod surface;
 pub mod text_edit;
+pub mod text_ink;
 pub mod text_layout;
 pub mod textures;
 pub mod viewports;
```

**What each hunk does.**

1. registers the new module beside `text_edit`.

**If this were reverted / how it is checked.** `cargo nextest run -p photocraft-ui-egui -E 'test(<the trials this file holds>)'` and `cargo xtask ci lint` name the loss.

### `crates/ui-contract/src/runtime/egui/text_edit.rs`

**Role.** a file this merge keeps.

**The diff, verbatim** (`git diff 02419852d..023528394 -- crates/ui-contract/src/runtime/egui/text_edit.rs`):

```diff
diff --git a/crates/ui-contract/src/runtime/egui/text_edit.rs b/crates/ui-contract/src/runtime/egui/text_edit.rs
index 2b4fe7288..967c1db91 100644
--- a/crates/ui-contract/src/runtime/egui/text_edit.rs
+++ b/crates/ui-contract/src/runtime/egui/text_edit.rs
@@ -1,6 +1,6 @@
 //! Building `TextEdit` and its state: the one caller of egui's text editing.
 
-use crate::primitives::{COLORED_TEXT_LIMIT, ColorRange, ColorRanges, RangeInk};
+use crate::runtime::egui::text_ink::{COLORED_TEXT_LIMIT, ColorRange, ColorRanges, RangeInk};
 use crate::runtime::env::Env;
 use craft_theme::TypeRole;
 use egui::text::{CCursor, CCursorRange, LayoutJob, TextFormat};
```

**What each hunk does.**

1. imports the four names from its own layer instead of from `primitives`, which is what the lint flagged fourteen times.

**If this were reverted / how it is checked.** `cargo nextest run -p photocraft-ui-egui -E 'test(<the trials this file holds>)'` and `cargo xtask ci lint` name the loss.

### `crates/ui-contract/src/runtime/egui/text_ink.rs`

**Role.** a file this merge keeps.

**The diff, verbatim** (`git diff 02419852d..023528394 -- crates/ui-contract/src/runtime/egui/text_ink.rs`):

```diff
diff --git a/crates/ui-contract/src/runtime/egui/text_ink.rs b/crates/ui-contract/src/runtime/egui/text_ink.rs
new file mode 100644
index 000000000..613b645b1
--- /dev/null
+++ b/crates/ui-contract/src/runtime/egui/text_ink.rs
@@ -0,0 +1,47 @@
+//! The ink a coloured text range names, and the most text a part colours. Layer 2, so `primitives`
+//! (layer 3) re-exports them rather than this adapter importing upward.
+
+/// The longest text, in bytes, a part colours: a longer one is laid out in its plain ink, and its
+/// [`ColorRanges`](crate::primitives::ColorRanges) is not called. It bounds the cost of an app's colouring (a
+/// highlighter whose time grows with the square of the length on crafted input) at the size of a long script.
+pub const COLORED_TEXT_LIMIT: usize = 16 * 1024;
+
+/// A theme colour for a stretch of a field's text. Each is the token's colour moved, where it has to be, to the text
+/// contrast minimum on the field's fill, so every ink reads in every theme and contrast.
+#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
+#[non_exhaustive]
+pub enum RangeInk {
+    /// `text.muted`.
+    Muted,
+    /// `accent.text`.
+    Accent,
+    /// `status.good.fg`.
+    Good,
+    /// `status.warn.fg`.
+    Warn,
+    /// `status.bad.fg`.
+    Bad,
+    /// `status.info.fg`.
+    Info,
+}
+
+/// The colour ranges of a text ([`TextOptions::color_ranges`](crate::primitives::TextOptions::color_ranges)): called
+/// with the text each time the part lays it out, which is every frame the part shows. An app whose ranges are costly
+/// to find keeps its own cache.
+pub type ColorRanges = dyn Fn(&str) -> Vec<ColorRange>;
+
+/// One stretch of a text in an ink of its own: `range` is in bytes of the text the [`ColorRanges`] was given.
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct ColorRange {
+    /// The bytes the ink covers.
+    pub range: std::ops::Range<usize>,
+    /// Their ink.
+    pub ink: RangeInk,
+}
+
+impl ColorRange {
+    /// The bytes `range` in `ink`.
+    pub fn new(range: std::ops::Range<usize>, ink: RangeInk) -> Self {
+        ColorRange { range, ink }
+    }
+}
```

**What each hunk does.**

1. the new home of `COLORED_TEXT_LIMIT`, `RangeInk`, `ColorRange` and `ColorRanges`, with the documentation they had, at layer 2.

**If this were reverted / how it is checked.** `cargo nextest run -p photocraft-ui-egui -E 'test(<the trials this file holds>)'` and `cargo xtask ci lint` name the loss.

The isolated pins stay as they are: `crates/engine-core`'s `craft-pcraft-format` and
`crates/ipp`'s `craft-ipp` name `3eb5b528b` in their own `Cargo.toml`, commented as isolated
pins, so `Cargo.lock` holds three references to that revision.
