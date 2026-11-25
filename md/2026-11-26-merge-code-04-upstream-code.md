# The upstream code behind the fork's answers in photocraft

For each adaptation the merge had to make: the upstream lines that forced
it, what they do, why they broke the fork, and the fork's answer.

Each entry names the upstream file and the lines the merge had to answer. The lines are
quoted from `e22facf45` as it was merged, with their line numbers.

## 1. The framework's tile hands every document to a window of its own

**The upstream code.**

`crates/ui-egui/src/menus/view_cmds/arrange.rs`, lines 70-70:

```rust
   70 |     let arranged = run_arrange(&mut app.ui.document_windows, command, &space);
```

**What it does.** Upstream's arrange command hands the whole command to the framework's `run_arrange`, which tiles the documents into floating windows.

**Why it broke the fork.** In this application a floating document window is a window of its own, so the framework's tile leaves the document area empty and covers the screen with windows.

**The fork's answer.** The two tile commands are answered in the application: the documents are consolidated into the window's document area and split into one pane per document, and the arrangement is recorded. A trial checks that both documents show, side by side or stacked.

## 2. The tool list counts seventy tools

**The upstream code.**

`crates/ui-egui/src/state.rs`, lines 632-632:

```rust
  632 |         assert_eq!(Tool::ALL.len(), 70);
```

**What it does.** Upstream's trial counts the tools in `Tool::ALL` as seventy.

**Why it broke the fork.** The fork's own tools are in the same array, so the array and the trial have to count the same tools.

**The fork's answer.** The trial expects 72, the number of tools a user can pick, and the array holds them.

## 3. The tool commands carry no title

**The upstream code.**

`crates/ui-egui/src/app/framework.rs`, lines 629-634:

```rust
  629 | fn build_tool_states(app: &PhotocraftApp) -> SourceStates {
  630 |     Tool::ALL.iter().map(|&tool| (CommandRef::from(crate::panels::tool_commands::command_of(tool)), CommandState::new().checked(app.ui.tool == tool))).collect()
  631 | }
  632 | 
  633 | /// [`build_tool_states`], derived from the tool.
  634 | fn tool_states(app: &PhotocraftApp, name: &'static str) -> Arc<SourceStates> {
```

**What it does.** Upstream's tool states carry the checked flag only, so a menu row is written by the framework's own name for the command.

**Why it broke the fork.** In a Japanese interface the row would show the English name, where the interface writes its own.

**The fork's answer.** A tool command's title is the tool's label in the interface language when the interface is not English, and the derived-state key carries the language so a change of language re-derives the states.

## 4. The font list follows the preference alone

**The upstream code.**

`crates/ui-egui/src/tools/type_tool/fonts.rs`, lines 124-124:

```rust
  124 |     settings.names_in_english = prefs.type_.show_font_names_in_english;
```

**What it does.** Upstream takes the English-names preference as it stands.

**Why it broke the fork.** A Japanese interface lists the families by their Japanese names, as the reference application does, whatever the preference says.

**The fork's answer.** A helper decides between the preference and the interface language: Japanese always lists the Japanese names, every other language follows the preference.

## 5. The quick selection takes its mode from the modifiers alone

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

**Why it broke the fork.** The tool has its own New / Add / Subtract bar; the bar is where the mode is shown and changed, and Shift adds where the bar adds.

**The fork's answer.** The mode comes from the bar, with the modifiers as its shortcut, and the bar flips from New to Add after the first stroke on an empty selection, which needs the command's `changed` answer.

## 6. The engine's retouch commands moved to another crate

**The upstream code.**

`crates/engine-paint/Cargo.toml`, lines 12-31:

```text
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
   26 | 
   27 | # Core's allocation failpoints, which the brush and fill tests arm.
   28 | [dev-dependencies]
   29 | photocraft-engine-core = { workspace = true, features = ["failpoints"] }
   30 | 
```

**What it does.** Upstream moved the retouch commands out of `engine-core` into `engine-paint`, and the crate that hosts them now needs the codec crate (the LaMa Cleaner encodes its crop and mask as PNG) and a temporary-directory crate.

**Why it broke the fork.** The moved module would not build in the crate that hosts it without those two dependencies, and `engine-core` must stop declaring the module.

**The fork's answer.** The fork had already placed its own LaMa Cleaner in the same target directory with the two dependencies declared, so upstream's move landed on it without a change; `engine-core` declares the module no more, on both sides.

## 7. The smart-filter stage trial runs with the other screen trials

**The upstream code.**

`crates/engine/tests/it/main.rs`, lines 184-184:

```rust
  184 | mod smart_filter_stages;
```

**What it does.** Upstream's screen-trial module list includes `smart_filter_stages`.

**Why it broke the fork.** The trial asserts on a cache the whole process shares, and the screen trials run in parallel threads, so it reads another trial's entry and fails on a line that is right.

**The fork's answer.** The trial moved to the global-state trials, where it takes its turn; its assertions are unchanged, and the corpus run exits 0.

## 8. The canvas right-click menu does not know the SAM tool

**The upstream code.**

`crates/ui-egui/src/canvas/canvas_tool_menu.rs`, lines 1185-1187:

```rust
 1185 |             } else if matches!(menu.tool, Tool::RedEye | Tool::ObjectSelection | Tool::Type) {
 1186 |                 if menu.has_selection { MEASURED_SELECTION_MENU } else { COUNT_MENU }
 1187 |             } else if menu.tool == Tool::Eyedropper {
```

**What it does.** Upstream's menu opens the measured-selection menu for the red-eye, object-selection and type tools.

**Why it broke the fork.** The fork's SAM Selection tool is a selection tool and needs the same right-click menu as the other selection tools.

**The fork's answer.** `Tool::Sam3Select` joins that list, and the tool is in the list of tools whose right-click opens the selection menu.

