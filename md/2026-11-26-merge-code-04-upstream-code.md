# The upstream code behind the answers in photocraft

The upstream lines each answer had to meet.

Quotations are `git show f0205ae33:<path>` with that commit's numbers.

## 1. The framework's tile hands every document to a window of its own

**The upstream code.**

`crates/ui-egui/src/menus/view_cmds/arrange.rs`, lines 70-71:

```rust
   70 |     let arranged = run_arrange(&mut app.ui.document_windows, command, &space);
   71 |     crate::canvas::document_area::apply_arranged(app, arranged);
```

**What it does.** Upstream hands the arrangement to the framework's `run_arrange`, which tiles into floating windows.

**What the merge had to answer.** A floating document window is a window of its own here, so the tile left the document area empty.

**The answer.** The two tile commands are answered in the application: one pane per document, and a trial asserts both documents show.

## 2. The tool list counts seventy tools

**The upstream code.**

`crates/ui-egui/src/state.rs`, lines 632-632:

```rust
  632 |         assert_eq!(Tool::ALL.len(), 70);
```

**What it does.** Upstream's trial pins the tool count.

**What the merge had to answer.** The fork's tools sit in the same array.

**The answer.** The trial expects 72.

## 3. A tool command carries no title

**The upstream code.**

`crates/ui-egui/src/app/framework.rs`, lines 652-654:

```rust
  652 | fn build_tool_states(app: &PhotocraftApp) -> SourceStates {
  653 |     Tool::ALL.iter().map(|&tool| (CommandRef::from(crate::panels::tool_commands::command_of(tool)), CommandState::new().checked(app.ui.tool == tool))).collect()
  654 | }
```

**What it does.** Upstream's tool states carry the checked flag only.

**What the merge had to answer.** In a Japanese interface the row would be English.

**The answer.** The title is the tool's label in the interface language, and the derived key carries the language.

## 4. The quick selection takes its mode from the modifiers alone

**The upstream code.**

`crates/ui-egui/src/tools/retouch_ui.rs`, lines 274-279:

```rust
  274 |             let mode = if mods.alt { "subtract" } else { "add" };
  275 |             let xy: Vec<[f64; 2]> = points.iter().map(|q| [q[0], q[1]]).collect();
  276 |             let (sample_all, enhance) = (o.sample_all.quick_selection, o.enhance_edge);
  277 |             let r = app.run("select.quick", json!({"points": xy, "size": size, "mode": mode, "enhanceEdge": enhance, "sampleAllLayers": sample_all}));
  278 |             report_smart_result(app, r, tl!("Quick Selection found nothing there"));
  279 |             return true;
```

**What it does.** Upstream derives Add and Subtract from Alt.

**What the merge had to answer.** The tool has its own New / Add / Subtract bar.

**The answer.** The mode comes from the bar with the modifiers as its shortcut, and New flips to Add after the first stroke on an empty selection.

## 5. The retouch commands moved to another crate

**The upstream code.**

`crates/engine-paint/Cargo.toml`, lines 12-25:

```rust
   12 | [dependencies]
   13 | photocraft-engine-core = { workspace = true }
   14 | craft-cms = { workspace = true }
   15 | photocraft-algo = { workspace = true }
   16 | photocraft-color = { workspace = true }
   17 | photocraft-compose = { workspace = true }
   18 | photocraft-doc = { workspace = true }
   19 | photocraft-geom = { workspace = true }
   20 | photocraft-io = { workspace = true }
   21 | photocraft-paint = { workspace = true }
   22 | photocraft-raster = { workspace = true }
   23 | rayon = { workspace = true }
   24 | serde = { workspace = true }
   25 | serde_json = { workspace = true }
```

**What it does.** Upstream moved the retouch commands out of `engine-core` into `engine-paint`.

**What the merge had to answer.** The module needs the codec crate and a temporary-directory crate, and `engine-core` must stop declaring it.

**The answer.** The fork's LaMa Cleaner already sat in the same directory with those dependencies.

## 6. The canvas right-click menu does not know the SAM tool

**The upstream code.**

`crates/ui-egui/src/canvas/canvas_tool_menu.rs`, lines 1200-1202:

```rust
 1200 |             } else if matches!(menu.tool, Tool::RedEye | Tool::ObjectSelection | Tool::Type) {
 1201 |                 if menu.has_selection { MEASURED_SELECTION_MENU } else { COUNT_MENU }
 1202 |             } else if menu.tool == Tool::Eyedropper {
```

**What it does.** Upstream opens the measured-selection menu for three tools.

**What the merge had to answer.** The fork's SAM Selection needs the same menu.

**The answer.** `Tool::Sam3Select` joins both lists.
