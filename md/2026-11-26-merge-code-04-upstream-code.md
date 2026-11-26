# The upstream code behind the answers in photocraft

The upstream lines each answer had to meet.

Every quotation is `git show d827906d6:<path>`, with that commit's line numbers.

## 1. The framework's tile hands every document to a window of its own

**The upstream code.**

`crates/ui-egui/src/menus/view_cmds/arrange.rs`, lines 70-71:

```rust
   70 |     let arranged = run_arrange(&mut app.ui.document_windows, command, &space);
   71 |     crate::canvas::document_area::apply_arranged(app, arranged);
```

**What it does.** Upstream hands the whole arrangement to the framework's `run_arrange`, which tiles the documents into floating windows.

**What the merge had to answer.** In this application a floating document window is a window of its own, so the framework's tile leaves the document area empty and covers the screen with windows.

**The answer.** The two tile commands are answered in the application: the documents are consolidated into the window's document area and split into one pane per document, and a trial opens two documents and asserts one pane each.

## 2. The tool list counts seventy tools

**The upstream code.**

`crates/ui-egui/src/state.rs`, lines 632-632:

```rust
  632 |         assert_eq!(Tool::ALL.len(), 70);
```

**What it does.** Upstream's trial pins the number of tools the application offers.

**What the merge had to answer.** The fork's own tools - SAM Selection and the LaMa Cleaner among them - are in the same array, so the count is not upstream's.

**The answer.** The trial expects 72, the number of tools a user can pick.

## 3. A tool command carries no title

**The upstream code.**

`crates/ui-egui/src/app/framework.rs`, lines 629-631:

```rust
  629 | fn build_tool_states(app: &PhotocraftApp) -> SourceStates {
  630 |     Tool::ALL.iter().map(|&tool| (CommandRef::from(crate::panels::tool_commands::command_of(tool)), CommandState::new().checked(app.ui.tool == tool))).collect()
  631 | }
```

**What it does.** Upstream's tool states carry the checked flag only, so a menu row shows the framework's own name for the command.

**What the merge had to answer.** In a Japanese interface that row would be English while the rest of the menu is not.

**The answer.** A tool command's title is the tool's label in the interface language when the interface is not English, and the derived-state key carries the language so a change of language re-derives the states.

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

**What it does.** Upstream's quick selection derives Add and Subtract from the Alt modifier.

**What the merge had to answer.** The tool has its own New / Add / Subtract bar; the bar is where the mode is shown and changed, and Shift adds where the bar adds.

**The answer.** The mode comes from the bar with the modifiers as its shortcut, and the bar flips from New to Add after the first stroke on an empty selection, which needs the command's `changed` answer.

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

**What it does.** Upstream moved the retouch commands out of `engine-core` into `engine-paint`, and that crate now hosts them.

**What the merge had to answer.** The moved module needs the codec crate (the LaMa Cleaner encodes its crop and mask as PNG) and a temporary-directory crate, and `engine-core` must stop declaring the module.

**The answer.** The fork's LaMa Cleaner already sat in the same target directory with those dependencies declared, so upstream's move landed on it; `engine-core` declares the module no more on both sides.

## 6. The canvas right-click menu does not know the SAM tool

**The upstream code.**

`crates/ui-egui/src/canvas/canvas_tool_menu.rs`, lines 1200-1202:

```rust
 1200 |             } else if matches!(menu.tool, Tool::RedEye | Tool::ObjectSelection | Tool::Type) {
 1201 |                 if menu.has_selection { MEASURED_SELECTION_MENU } else { COUNT_MENU }
 1202 |             } else if menu.tool == Tool::Eyedropper {
```

**What it does.** Upstream's menu opens the measured-selection menu for the red-eye, object-selection and type tools.

**What the merge had to answer.** The fork's SAM Selection is a selection tool and needs the same right-click menu.

**The answer.** `Tool::Sam3Select` joins that list and the list of tools whose right-click opens the selection menu.
