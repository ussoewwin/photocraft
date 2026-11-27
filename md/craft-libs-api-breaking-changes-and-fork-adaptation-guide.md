# craft-libs API breaking changes and the fork's adaptation

This document records the breaking changes that came with the upstream redesign of `storytold/craft-libs` (`main` since 2026-11-27, `next` before that), the changes `photocraft` (`crates/ui-egui`) needed to meet them, and the fork features that had to survive: Space as the Hand tool's own key (the `with_space_activation` rule), the PSD extension blocks, the ZIP 64-bit EOCD header, and the CJK font classification.

Every path below is repository-relative. Code is quoted from the working tree at the time of the repair; the measurements at the end are dated.

---

## 1. What broke

### 1.1 The release build stopped
Linking the shared library (`craft-libs`, then on `next`, now `main`) into `photocraft` with the local patch (`cargo xtask craftlibs on`) produced type errors, trait-bound errors and calls to methods that no longer existed, across `crates/ui-egui`; no Windows binary (`photocraft.exe`, `photocraft-cli.exe`) could be produced from that tree.

### 1.2 The upstream redesign
Upstream introduced memoization for the UI panels, changed how multilingual menus are published, simplified the preset manager's type parameters, and added generation tracking to the rendering caches — in one series of commits. The public signatures the application had been calling changed with them.

### 1.3 The risk in the automation
The local sync script used `--theirs` (take upstream's file) on some paths, and it listed `crates/zip` among the upstream-only prefixes; the fork's ZIP 64-bit EOCD support could have been overwritten by a sync. The Space rule (`activate_on_space`, `with_space_activation`) could have been cancelled the same way.

---

## 2. The causes

1. **`MemoProps` on `PresetSource`.** `PresetSource::new<I: MemoProps>` refuses the old `PresetRow<K>`, whose `preview` is an `Arc<dyn Fn...>`: it cannot derive `PartialEq` or `Hash`. Every place that passed a `PresetRow` array stopped compiling.
2. **`HostedPresetManager<K, G>` became `HostedPresetManager<K>`,** and `.rows(...)` became `.names(...)`; grouping moved to another layer.
3. **The English menu API was removed.** `MenuBarUi::english`, `publish_english` and the helpers (`english_names`, `english_label`, `english_title`, `collect_english`) are gone.
4. **`&mut Serials` became a constructor argument** for `PreviewTextures::new`, `CoverageMask::new` and `HoverMask::sampled`.
5. **The egui pin diverged.** craft-libs pins `storytold/egui` `8a3c5ecb29` for `Context::with_accesskit_parents`; the application root still pinned `6525c87923`, so `cargo test` failed to link (`E0599`) once the `testing` feature was on.
6. **`Send + Sync` at the thumbnail boundary.** `photocraft_engine::Session` is `!Sync` (it owns the job channel), and the thumbnail closure requires `'static + Send + Sync`; a captured `Session` reference cannot satisfy both.

---

## 3. The files that changed

Twenty-two files in `photocraft`, and the local sync script (outside this repository).

| # | path | change |
|---|---|---|
| 1 | `Cargo.toml` | the pinned `storytold/egui` revision moves to `8a3c5ecb29` |
| 2 | `Cargo.lock` | the lock file follows |
| 3 | `.cargo/config.toml` | the local patch of the 231 `craft-libs` crates |
| 4 | `crates/ui-egui/src/menus/bar.rs` | `MenuBarUi::english` is gone; `MenuModel` carries the cache generation |
| 5 | `crates/ui-egui/src/menus/items.rs` | `publish_english` and the English collection are gone |
| 6 | `crates/ui-egui/src/prefs/prefs_ui/presets.rs` | `HostedPresetManager<String>`, `manager.names(...)` |
| 7 | `crates/ui-egui/src/panels/brush_panel/brushes.rs` | `BrushKey` implements `MemoProps`; `preset_source` |
| 8 | `crates/ui-egui/src/panels/brush_sections.rs` | `PaintValues` initialisation |
| 9 | `crates/ui-egui/src/panels/options_bar/paint.rs` | the removed `size` and `hardness` assignments |
| 10 | `crates/ui-egui/src/panels/preset_browser.rs` | `PresetKey` implements `MemoProps`; `PresetRowsInput`, `preset_source`, `preset_source_with_thumb` |
| 11 | `crates/ui-egui/src/dialogs/pattern_fill_ui.rs` | `.thumbnails(...)` is gone; `PatternSwatch::with_picture` |
| 12 | `crates/ui-egui/src/dialogs/fill_ui.rs` | the pattern list through `preset_source` |
| 13 | `crates/ui-egui/src/dialogs/layer_style/contour.rs` | the closure type becomes `fn(&[[f32; 2]]) -> Vec<f32>` |
| 14 | `crates/ui-egui/src/dialogs/layer_style.rs` | `StyleInputs` implements `MemoProps`; the thumbnail pattern is cloned apart |
| 15 | `crates/ui-egui/src/tools/brush_picker.rs` | `PickerInput` implements `MemoProps`; the source is passed by reference |
| 16 | `crates/ui-egui/src/tools/cas_transform.rs` | `&mut Serials` reaches `PreviewTextures::new` |
| 17 | `crates/ui-egui/src/tools/perspective_ui.rs` | the same |
| 18 | `crates/ui-egui/src/tools/puppet_ui.rs` | the same |
| 19 | `crates/ui-egui/src/tools/transform_tool.rs` | the same |
| 20 | `crates/ui-egui/src/tools/transform_tool/begin.rs` | `&mut Serials` at all seven `PreviewTextures::new` calls |
| 21 | `crates/ui-egui/src/tools/stroke_trail.rs` | `&mut Serials` reaches `CoverageMask::new` |
| 22 | `crates/ui-egui/src/tools/object_finder.rs` | `&mut Serials` reaches `HoverMask::sampled` |

---

## 4. The code

### 4-1 `Cargo.toml` — the egui revision follows craft-libs

craft-libs needs `Context::with_accesskit_parents`, so its pin moved to `8a3c5ecb29`; the application root pinned the older `6525c87923` and failed to link once `craft-ui-contract`'s `testing` feature turned on (`E0599`). The four `storytold/egui` entries move together.

```toml
ecolor = { git = "https://github.com/storytold/egui.git", rev = "8a3c5ecb29ab8679d57035f2cb51d7ec8a5c080f" }
emath = { git = "https://github.com/storytold/egui.git", rev = "8a3c5ecb29ab8679d57035f2cb51d7ec8a5c080f" }
epaint = { git = "https://github.com/storytold/egui.git", rev = "8a3c5ecb29ab8679d57035f2cb51d7ec8a5c080f" }
epaint_default_fonts = { git = "https://github.com/storytold/egui.git", rev = "8a3c5ecb29ab8679d57035f2cb51d7ec8a5c080f" }
egui = { git = "https://github.com/storytold/egui.git", rev = "8a3c5ecb29ab8679d57035f2cb51d7ec8a5c080f" }
egui-winit = { git = "https://github.com/storytold/egui.git", rev = "8a3c5ecb29ab8679d57035f2cb51d7ec8a5c080f" }
egui-wgpu = { git = "https://github.com/storytold/egui.git", rev = "8a3c5ecb29ab8679d57035f2cb51d7ec8a5c080f" }
egui_glow = { git = "https://github.com/storytold/egui.git", rev = "8a3c5ecb29ab8679d57035f2cb51d7ec8a5c080f" }
egui_extras = { git = "https://github.com/storytold/egui.git", rev = "8a3c5ecb29ab8679d57035f2cb51d7ec8a5c080f" }
egui_kittest = { git = "https://github.com/storytold/egui.git", rev = "8a3c5ecb29ab8679d57035f2cb51d7ec8a5c080f" }
egui_system_font_provider = { git = "https://github.com/storytold/egui.git", rev = "8a3c5ecb29ab8679d57035f2cb51d7ec8a5c080f" }
eframe = { git = "https://github.com/storytold/egui.git", rev = "8a3c5ecb29ab8679d57035f2cb51d7ec8a5c080f" }
egui_demo_lib = { git = "https://github.com/storytold/egui.git", rev = "8a3c5ecb29ab8679d57035f2cb51d7ec8a5c080f" }
egui_inspection = { git = "https://github.com/storytold/egui.git", rev = "8a3c5ecb29ab8679d57035f2cb51d7ec8a5c080f" }
```

### 4-2 `crates/ui-egui/src/menus/bar.rs` — the English menu API is gone

`MenuBarUi::english` and the helpers behind it were removed upstream; the model keeps its cache generation instead.

```rust
#[derive(Clone, Default)]
pub struct MenuModel {
    pub(crate) state: u64,
}

impl MenuModel {
    pub fn new() -> Self {
        Self::default()
    }
}

pub fn menu_bar(
    ui: &mut egui::Ui,
    app: &mut crate::PhotocraftApp,
    state: &mut MenuModel,
) -> Option<CommandId> {
    let mut bar = MenuBarUi::new();
    let current_state = app.session.state_serial();
    let dirty = state.state != current_state;
    state.state = current_state;

    let mut invoked = None;
    bar.show(ui, |menu| {
        let cmd = show_menus(menu, app, dirty);
        if cmd.is_some() {
            invoked = cmd;
        }
    });
    invoked
}
```

### 4-3 `crates/ui-egui/src/menus/items.rs` — the English collection is gone

`publish_english` and the collection around it were removed upstream; the menu items are built once, in the language the catalog supplies.

```rust
pub(crate) fn show_help_menu(
    menu: &mut MenuUi,
    app: &mut crate::PhotocraftApp,
) -> Option<CommandId> {
    let entries = help_entries(app, false);
    for entry in entries {
        if menu.button(&entry.label).clicked() {
            return Some(entry.id);
        }
    }
    None
}
```

### 4-4 `crates/ui-egui/src/prefs/prefs_ui/presets.rs` — one type parameter

`HostedPresetManager<K, G>` became `HostedPresetManager<K>`, and `.rows(...)` became `.names(...)`; grouping now belongs to another layer.

```rust
#[derive(Clone, Debug, Default)]
pub(crate) struct PresetsUi {
    manager: HostedPresetManager<String>,
}

impl PresetsUi {
    pub(crate) fn show(&mut self, ui: &mut egui::Ui, app: &mut PhotocraftApp) {
        let kind = self.current_kind;
        let names = load_names(app, kind);
        self.manager.names(names);
        self.manager.show(ui, |action| match action {
            PresetAction::Select(name) => apply_preset(app, kind, &name),
            PresetAction::Delete(name) => delete_preset(app, kind, &name),
            PresetAction::Rename(old, new) => rename_preset(app, kind, &old, &new),
        });
    }
}
```

### 4-5 `crates/ui-egui/src/panels/brush_panel/brushes.rs` — `MemoProps` on the key

`PresetSource::new` requires `I: MemoProps` so a panel is not rebuilt every frame; `BrushKey` derives it and the source is built from the brush list.

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, MemoProps)]
pub(crate) struct BrushKey(pub(crate) usize);

pub(crate) fn brush_source<'a>(rows: Vec<PresetRow<BrushKey>>) -> PresetSource<'a, BrushKey> {
    crate::panels::preset_browser::preset_source(rows)
}
```

### 4-6 `crates/ui-egui/src/panels/preset_browser.rs` — the memo wrapper and the sources

`PresetRow` holds `Arc<dyn Fn...>` in `preview`, so it cannot derive `PartialEq`/`Hash`; the input is wrapped in `PresetRowsInput`, which compares what the row needs, and `PresetKey` carries `MemoProps`. The thumbnail source clones what the `'static + Send + Sync` closure needs.

```rust
use std::borrow::Cow;
use craft::ui::widgets::preset_browser::{PresetRow, PresetSource};
use craft::ui::contract::memo::MemoProps;

#[derive(Clone, Debug, PartialEq, Eq, Hash, MemoProps)]
pub struct PresetKey(pub usize);

#[derive(Clone)]
pub struct PresetRowsInput<'a, K: Clone + PartialEq + 'static> {
    rows: Cow<'a, [PresetRow<K>]>,
}

impl<'a, K: Clone + PartialEq + 'static> PresetRowsInput<'a, K> {
    pub fn new(rows: impl Into<Cow<'a, [PresetRow<K>]>>) -> Self {
        Self { rows: rows.into() }
    }
}

impl<'a, K: Clone + PartialEq + 'static> PartialEq for PresetRowsInput<'a, K> {
    fn eq(&self, other: &Self) -> bool {
        if self.rows.len() != other.rows.len() {
            return false;
        }
        self.rows.iter().zip(other.rows.iter()).all(|(a, b)| {
            a.key == b.key && a.label == b.label && a.depth == b.depth
        })
    }
}

impl<'a, K: Clone + PartialEq + 'static> std::hash::Hash for PresetRowsInput<'a, K> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.rows.len().hash(state);
        for row in self.rows.iter() {
            row.label.hash(state);
            row.depth.hash(state);
        }
    }
}

impl<'a, K: Clone + PartialEq + 'static> MemoProps for PresetRowsInput<'a, K> {}

pub fn preset_source<'a, K: Clone + PartialEq + 'static>(
    rows: Vec<PresetRow<K>>,
) -> PresetSource<'a, K> {
    let input = PresetRowsInput::new(rows);
    PresetSource::new(input, |inp| inp.rows.clone())
}

pub fn preset_source_with_thumb<'a, K: Clone + PartialEq + 'static>(
    rows: Vec<PresetRow<K>>,
    thumb: impl Fn(&K, &mut egui::Ui) + Send + Sync + 'static,
) -> PresetSource<'a, K> {
    let input = PresetRowsInput::new(rows);
    PresetSource::with_thumb(input, |inp| inp.rows.clone(), thumb)
}
```

### 4-7 `crates/ui-egui/src/dialogs/pattern_fill_ui.rs` — the picture binds directly

`.thumbnails(...)` is gone; `PatternSwatch::with_picture` takes the image.

```rust
let swatches: Vec<PatternSwatch> = patterns
    .iter()
    .map(|pat| {
        let mut swatch = PatternSwatch::new(pat.name.clone());
        if let Some(tex) = textures.get(&pat.id) {
            swatch = swatch.with_picture(tex.clone());
        }
        swatch
    })
    .collect();
```

### 4-8 `crates/ui-egui/src/dialogs/fill_ui.rs` — the pattern list through a source

The pattern rows come from `preset_source`, as the other panels do.

```rust
let source = crate::panels::preset_browser::preset_source(patterns.clone());
dialog.show_patterns(ui, &source);
```

### 4-9 `crates/ui-egui/src/dialogs/layer_style/contour.rs` — a function pointer

The contour sampler is a plain `fn(&[[f32; 2]]) -> Vec<f32>`, which is what the new bound accepts.

```rust
fn evaluate_contour_samples(points: &[[f32; 2]]) -> Vec<f32> {
    let mut lut = vec![0.0; 256];
    if points.is_empty() {
        return lut;
    }
    for (i, val) in lut.iter_mut().enumerate() {
        let t = i as f32 / 255.0;
        *val = sample_curve_at(points, t);
    }
    lut
}

impl ContourEditorDialog {
    pub fn sampler() -> fn(&[[f32; 2]]) -> Vec<f32> {
        evaluate_contour_samples
    }
}
```

### 4-10 `crates/ui-egui/src/dialogs/layer_style.rs` — the style source and `Send + Sync`

`StyleInputs` implements `MemoProps`; the thumbnail closure receives a clone of the pattern it draws, because `Session` is `!Sync` and cannot be captured.

```rust
#[derive(Clone, PartialEq, Hash, MemoProps)]
struct StyleInputs {
    category: usize,
    count: usize,
}

fn style_source<'a>(app: &PhotocraftApp) -> PresetSource<'a, String> {
    let patterns = app.session.patterns.clone();
    let inputs = StyleInputs {
        category: app.styles.current_category,
        count: app.styles.count(),
    };
    PresetSource::with_thumb(
        inputs,
        move |_| load_style_rows(&patterns),
        move |key, ui| {
            render_style_preview(key, ui, &patterns);
        },
    )
}
```

### 4-11 `crates/ui-egui/src/panels/options_bar/paint.rs` — the removed fields

`PaintValues` no longer carries `size` and `hardness`; the assignments are deleted.

```rust
// the references to the removed fields are gone
pub fn apply_paint_values(app: &mut PhotocraftApp, values: &PaintValues) {
    let tool = app.session.tools.current_paint_mut();
    tool.opacity = values.opacity;
    tool.flow = values.flow;
    // values.size and values.hardness now belong to their own slider handling
}
```

### 4-12 `crates/ui-egui/src/panels/brush_sections.rs` — the initialiser follows

The same removal reaches the initialiser; no code change beyond the field list.

### 4-13 `crates/ui-egui/src/tools/brush_picker.rs` — the picker's source

`PickerInput` implements `MemoProps` and the source is passed by reference.

```rust
#[derive(Clone, PartialEq, Eq, Hash, MemoProps)]
pub struct PickerInput {
    pub keys: Vec<BrushKey>,
}

impl BrushPickerTool {
    pub fn show(&mut self, ui: &mut egui::Ui) {
        let input = PickerInput { keys: self.keys.clone() };
        let source = PresetSource::new(input, |inp| load_brush_rows(&inp.keys));
        BrushPresetPicker::new(&source).show(ui);
    }
}
```

### 4-14 `cas_transform.rs`, `perspective_ui.rs`, `puppet_ui.rs`, `transform_tool.rs`, `transform_tool/begin.rs` — `&mut Serials`

`PreviewTextures::new` takes `&mut Serials` as its first argument; the seven call sites in `begin.rs` and the four tools pass it through.

```rust
// before:
let textures = PreviewTextures::new(width, height);

// after:
let mut serials = craft::ui::primitives::Serials::default();
let textures = PreviewTextures::new(&mut serials, width, height);
```

### 4-15 `crates/ui-egui/src/tools/stroke_trail.rs` — `CoverageMask::new`

The mask constructor takes `&mut Serials`.

```rust
// before:
let mask = CoverageMask::new(bounds);

// after:
let mut serials = craft::ui::primitives::Serials::default();
let mask = CoverageMask::new(&mut serials, bounds);
```

### 4-16 `crates/ui-egui/src/tools/object_finder.rs` — `HoverMask::sampled`

The sampled mask takes `&mut Serials`.

```rust
// before:
let hover = HoverMask::sampled(data, stride);

// after:
let mut serials = craft::ui::primitives::Serials::default();
let hover = HoverMask::sampled(&mut serials, data, stride);
```

### 4-17 The local sync script — fork protection and the tests it owes

The script that copied upstream files into the working tree no longer lists `crates/zip` among the upstream-only prefixes, keeps the fork's Space, PSD, ZIP and CJK rules on a conflict, and runs the fork's tests (`craft-ui-canvas`, `craft-ui-widgets`, `craft-ui-panels`, `craft-psd`, `craft-zip`) after a merge. It is not part of this repository.

```python
# excerpt from tools/photocraft_sync_mcp_server.py

# 1. crates/zip is out of upstream_only_prefixes entirely (the fork's own implementation is kept)
upstream_only_prefixes = [
    # "crates/zip", <-- removed: the fork's EOCD64 support stays
    "vendor/",
]

# 2. on a merge conflict the fork's rules are kept
FORK_PROTECTED_TOKENS = {
    "activate_on_space": "crates/ui-canvas/src/canvas/gesture/space_activation.rs",
    "with_space_activation": "crates/ui-panels/src/panels/tools/keybindings.rs",
    "scan_offsets": "crates/zip/src/reader/eocd64.rs",
    "Layr": "crates/psd/src/layer_records.rs",
    "Mt16": "crates/psd/src/layer_records.rs",
    "cjk_fallback": "crates/font-manager/src/cjk.rs",
}

# 3. after a merge the fork's tests run (x64)
VERIFICATION_CRATES = [
    "craft-ui-canvas",
    "craft-ui-widgets",
    "craft-ui-panels",
    "craft-psd",
    "craft-zip",
]
```

---

## 5. Why it is shaped this way

1. **Memoization and the frame budget.** The wrapper types (`PresetRowsInput`, `PickerInput`, `StyleInputs`) compare what a row draws, not the closures inside it, so the panels keep the new memoization without re-deriving `PartialEq` for `Arc<dyn Fn...>`.
2. **The `Send + Sync` boundary stays where it was.** The engine stays on the UI thread; the closures receive owned data (a cloned pattern, plain numbers) instead of a `Session`.
3. **The fork's features are protected by rule, not by luck.** The sync script no longer treats `crates/zip` as upstream's, keeps the Space, PSD, ZIP and CJK rules on a conflict, and runs the fork's tests after every merge.

---

## 6. Status, 2026-11-27

- The upstream lines merged since this document was written are in `md/2026-11-26-merge-code-03-upstream-inventory.md` (craft-libs) and `md/2026-11-26-merge-code-03-upstream-inventory.md` (photocraft); the measurements of the last round are in the same files.
- The newest breaking item is `crates/menu/src/mac.rs`: upstream's AppKit rebuild retry keeps `std::time::Instant` behind a `contract/motion` marker and registers it in `design/exceptions/craft-menu/crate.toml`. The fork's own marker and `design/exceptions/craft-menu/mac.toml` were a duplicate of that entry and were removed; `design-lint` reports 28/28 with one entry.
- `Prompt::layer_role` is `#[prop(vis = "pub(super)")]` upstream; the fork's plain `#[prop]` is superseded by it, and the hand-written setter stays gone.

