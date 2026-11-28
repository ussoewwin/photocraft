# The upstream code behind each answer

Each item quotes the upstream line that forced a fork answer, with its path, what it does, why it
broke the fork and what the fork did.

## 1. `craft-libs` `crates/ui-contract/src/runtime/egui/text_edit.rs:3` (at `51fcc3b04`)

```rust
use crate::primitives::{COLORED_TEXT_LIMIT, ColorRange, ColorRanges, RangeInk};
```

The file is layer 2 (`runtime::egui`); the four names were defined in `primitives`, layer 3. The
lint's `layer/import` rule allows a lower layer to be imported by a higher one, never this way, and it
named fourteen sites (line 3, lines 298 and 304-309).

**The fork's answer.** The four definitions moved to `runtime/egui/text_ink.rs`; `primitives`
re-exports them, so `crate::primitives::{...}` still resolves for every other crate, and the adapter
imports from its own layer.

## 2. `craft-libs` `crates/ui-contract/src/primitives/text_input.rs:57,62,68,85`

```rust
pub type ColorRanges = dyn Fn(&str) -> Vec<ColorRange>;
pub const COLORED_TEXT_LIMIT: usize = 16 * 1024;
pub enum RangeInk { Muted, Accent, Good, Warn, Bad, Info }
pub struct ColorRange { pub range: Range<usize>, pub ink: RangeInk }
```

These are the definitions item 1 imports: a type alias, a constant, an enum and a struct whose fields
are public. The struct is what the lint's `api/non-exhaustive` rule asked about.

**The fork's answer.** `ColorRange` joins the lint's closed list in
`crates/design-lint/src/workspace.rs` (the same file the previous round put `CheckContext` in),
because `craft-develop-model` builds it with a struct expression; `#[non_exhaustive]` would forbid
exactly that.

## 3. `craft-libs` `crates/draw/src/ui/transform/transform_box.rs:497`

```rust
#[prop(quantity = "points, from the host's handle size preference")]
```

Upstream declares the handle side as a points quantity itself. The previous round had registered a
`props/point-size` exemption with a `design: allow` marker in this file, and both conflicted.

**The fork's answer.** Upstream's line, its exemption file deleted together with the marker.

## 4. `craft-libs` `crates/draw/src/geometry/shape.rs:45-50`

```rust
Triangle {
    rect: Rect,
    corner_radius: f64,
}
```

The geometry gained a corner radius. `photocraft`'s `shapes.rs` matched `Triangle { rect }` and built
it the same way, which stopped compiling (`E0027`, `E0063` at `shapes.rs:158`).

**The fork's answer.** The whole-pixel conversion keeps `corner_radius`, and the `shape.create`
params carry it as `cornerRadius`, the command's own field name; the rectangle-menu trial passes 0.0.

## 5. `photocraft` `crates/ui-egui/src/tools/option_cells.rs:34` (at `8d7c1f833`)

```rust
pub(crate) fn number<'a>(caption: impl Into<Label>, value: Bind<'a, f32>, range: RangeInclusive<f64>) -> NumberCell<'a>
```

The number cell now takes a binding rather than `&mut f32`, so a caller that hands it a plain
mutable reference no longer compiles (`E0308` at `heal.rs:157`, `select.rs:101`, `select.rs:104`).

**The fork's answer.** `Bind::bound(&mut size)`, `Bind::bound(&mut threshold)`, `Bind::bound(&mut
refine)`.

## 6. `photocraft` `crates/ui-egui/src/tools/retouch_ui/bar/heal.rs:24`

```rust
pub(super) fn check(ui: &mut egui::Ui, label: &str, value: Bind<'_, bool>, enabled: bool, tip: Option<&str>)
```

The same change for the checkbox row; the SAM row passed `&mut o.sample_all.sam3_select` and
`&mut o.sam3_individual` (`select.rs:97`, `select.rs:106`).

**The fork's answer.** `Bind::bound(&mut ...)` for both, then the tracked bindings of item 8.

## 7. `photocraft` `crates/ui-egui/src/state/tool_options.rs:450,455`

```rust
pub fn slot(&self, tool: Tool) -> Option<bool> { sample_all_slot!(self, tool, &).copied() }
pub fn slot_mut(&mut self, tool: Tool) -> Option<&mut bool> { sample_all_slot!(self, tool, &mut) }
```

The accessor split into a reading one and a writing one, and the macro it calls lists the tools. The
fork's `Tool::Sam3Select` was not in that list, so the row had no slot at all.

**The fork's answer.** `Tool::Sam3Select => $($borrow)+ $this.sam3_select,` in the macro; the call
sites that wrote through `slot` use `slot_mut`.

## 8. `photocraft` `crates/ui-egui/src/chrome_settle_trials.rs:321-333`

```rust
fn an_idle_options_bar_leaves_the_tool_options_revision() {
    ...
    assert!(moved.is_empty(), "idle frames moved the tool options under {moved:?}");
}
```

The trial runs four frames per tool, reads `tool_options.revision()`, runs six more and fails if it
moved. The fork's SAM row took `&mut app.ui.tool_options` and wrote its values back every frame.

**The fork's answer.** `bind_tracked!(app.ui.tool_options => sample_all.sam3_select, sam3_prompt,
sam3_threshold, sam3_individual)`, the converted `u8` written only when it changed, and the Detect
params read back through an immutable borrow. The trial then passes in three runs of one.

## 9. `craft-libs` `crates/ui-widgets/src/text/selectable_text.rs` (at `51fcc3b04`)

```rust
fn finish(self, cx: Cx<'_, Self>) -> Response {
    cx.response.clone()
}
```

Upstream's selectable text no longer reports a click from the selection layer; the fork's copy set
`Flags::CLICKED` from `text.clicked_by(Primary)`. The merge took upstream's version, and the fork's
`contract/pointer` exemption with it.

**The fork's answer.** Upstream's, and the exemption file back to `config_version = 1`.
