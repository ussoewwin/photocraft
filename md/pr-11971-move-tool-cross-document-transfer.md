# Move tool: cross-document transfer of layers and selections

This is how the feature works in the tree as it stands: dragging a selection or a layer with the
Move tool onto another document's tab, or into another cell of an arranged view, duplicates it in
that document. The source document keeps its pixels unless a cut is requested.

| Behaviour | Implementation |
|---|---|
| Drag a selection to another document | `layer.copyToDocument` with `selection: true`, `cut: false`; the selected pixels land as a new layer |
| Drag a whole layer (no selection) | `MoveDrag::Layers` built from `layer_box.grabbed_layer` or `move_blocked_start`; the layer is copied, the source stays |
| Drag over another document's tab | `over_tab` arms the transfer, switches the active document, drops on release |
| Drag across an arranged-view cell | `check_arranged_drag` arms the transfer when the pointer leaves the active cell; `target_at` maps the release into the destination |
| Locked or Background layer | the press is refused (`move_lock::blocked`), `move_blocked_start` records the point, the drag still starts a transfer |
| Floating selection | `CutParts::piece()` travels instead of the layer surface, so the floating piece itself moves |
| Layer mask targeted | `TransferKind::Selection { mask: true }` sends `"target": "mask"`; the destination receives mask content |
| Release | only the primary button completes a transfer; a secondary release leaves it armed |
| Multi-window | only the viewport that armed the transfer completes or clears it |
| Drop inside the source document | a single layer dropped over a frame sets that frame's content (`layer.frame.setContent`) |
| Modifiers | Shift centres the copy when the documents differ in size; otherwise the drop point is used |

## Problem

Three behaviours were missing or destructive:

1. Dragging a selection from one document to another was unsupported in arranged views, and when it
   worked it cleared the source pixels.
2. Dragging whole layers from the canvas (without opening the Layers panel) was blocked on locked and
   Background layers, because the press never recorded which layer was hit.
3. A document window's frame ran the transfer cleanup for every viewport, so a second window could
   clear a marquee or a transfer the first window was still dragging.

## How it works in the current code

Five inputs can start a transfer, and they all funnel into two functions: `current_move_drag` (what is being dragged) and `arm_move_drag` (turn that into a `Transfer`).

| Input | Where it is handled | What it produces |
|---|---|---|
| Layers-panel row drag | `panels/layer_row.rs` calls `layer_transfer::begin_from_panel(app, ctx, id)` | a `Transfer` armed immediately, labelled with the layer's name |
| Move press on a selection | `canvas/tool_input.rs` records `app.drag` with `sel_move: Some(cut)` | `MoveDrag::Selection { grab: d.start, cut }` |
| Move press with no selection | the same press path, `sel_move: None` | `MoveDrag::Layers(d.start)` |
| Move press on a locked layer or the Background | `move_lock::blocked` refuses the drag; the arm sets `move_blocked = true`, `move_blocked_start = Some([x, y])`, `move_viewport = current_viewport` | `MoveDrag::Layers(start)` from `move_blocked_start` |
| Marquee that grabbed a layer | `move_box::begin_with_layer(app, p, add, hit_id)` records `layer_box.grabbed_layer` | `MoveDrag::Layers(b.start)` |

`current_move_drag` answers the first question. It returns `None` while a transform or a guide drag is active, then, when `app.drag` exists and its tool is Move: `Selection` when `d.sel_move` is `Some(cut)`, `Layers(d.start)` when the document has neither a selection nor a float, and `None` otherwise (the pixels are being dragged, not the layer). With no `app.drag` it asks whether a marquee or a refused press is live - `ui.tool == Move`, `tool_override == Move`, `move_blocked_start.is_some()`, or a `layer_box` with a `grabbed_layer` - and promotes that to `Layers`.

The second question is answered per frame, by three functions called from `canvas/tabs.rs`:

- `check_arranged_drag(app, ctx)` runs before the documents are drawn. It returns early if a transfer is already armed, if `move_viewport` names another window, or if there is no drag, no pointer, or no arranged cells. When the pointer is inside a cell that is not the active document's, it calls `arm_move_drag`.
- `over_tab(app, ctx, index)` runs from the tab strip. When nothing is armed and the hovered tab is not the active document, it arms; then it switches the active document, clears the jobs focus, drops the Layers-panel drag key (`Id::unique("layer-drag")`) so the new document's rows do not inherit the drag, and on a release drops into that document with `at: None` (the tab has no document point).
- `finish(app, ctx)` runs after the documents. It reads `primary_released` and `primary_down`; when the button is up or has just been released, no transfer is armed, and `move_viewport` is either unset or this viewport, it clears the move state. Then, if a transfer is armed in this viewport and the primary button was released, it maps the pointer to a document with `target_at` and either completes the transfer with `drop` or, over the source document, clears it.

`arm_move_drag` is the conversion. It reads `grabbed_layer` from the marquee, clears every piece of in-flight drag state (`app.drag`, `layer_box`, `move_blocked`, `move_blocked_start`, `move_viewport`, `move_preview`, `move_mods`), then arms. For `Layers` it uses `grabbed_layer` or the active layer. For `Selection` it decides `mask` from the float (`f.parts.is_mask()`) or from `layer_mask_targeted()`, discards a float that has not moved (`!f.moved && f.offset == (0, 0)`) so the transfer starts from the original pixels, and arms with `begin_selection`.

`drop` builds the command. It clears the transfer first, finds the source index from `t.source`, handles the in-document case (`dest == src`: a single layer over a frame becomes `layer.frame.setContent`), then builds either `{"layers": [...]}` or `{"selection": true, "cut": false}` plus `"target": "mask"` when the content is a mask. `apply_drop_placement` adds placement: with `at` and a `grab` it computes a rounded `offset`; with `at` and no `grab` it sets `at` (centre the copy there); with no `at`, or with Shift, it sets `center` to `!same_size`. Then `app.session.set_active(dest)` and `app.run("layer.copyToDocument", p)`, reporting a failure into `app.ui.status` / `status_error`.

Behind all of this, `Transfer` is stored in the egui context under `Id::unique("pc-layer-transfer")` as temp data. Every viewport of the context can see it, which is why the struct carries `viewport`: only that viewport may complete or clear it. `get` clones the value out, so no borrow is held while `finish` mutates the app.
## Design

The feature is split across three layers so that each owns one concern:

```
canvas gesture            panels/layer_transfer.rs      Transfer state (egui temp data)
   press / move / release      arm  -> Transfer { source, kind, grab, label, viewport }
                               drop -> layer.copyToDocument
engine command            crates/engine/src/cmd/layer_copy_cmds.rs
                               selection: true  -> lift() the selected pixels or the floating piece
                               cut: true        -> clear_selected() / clear_mask() / place_rest()
pixel helpers             crates/engine/src/cmd/edit_cmds.rs, float_cmds.rs
```

The transfer is *armed* into `egui::Context` temp data under a single key, so it is shared by every
viewport of that context but carries the id of the viewport that armed it.

```rust
struct Transfer {
    source: u64,            // the source document id
    kind: TransferKind,     // Layers(Vec<LayerId>) or Selection { mask: bool }
    grab: Option<[f64; 2]>, // where inside the content the drag began (document space)
    label: String,          // the ghost text by the pointer
    viewport: egui::ViewportId, // only this viewport completes the transfer
}
```

### Arming paths

| Path | Trigger | What it does |
|---|---|---|
| `begin_from_panel` | Layers panel row drag | `begin_layers(app, ctx, layer, None)` |
| `over_tab` | Pointer over another document's tab | Promotes `current_move_drag` into a transfer, switches the active document, drops on release |
| `check_arranged_drag` | Pointer inside another arranged cell | Clears the drag, arms the transfer owned by `ctx` |

`current_move_drag` is the single place that decides whether a gesture is a layer drag or a
selection drag. It reads `app.drag.sel_move` first (a selection drag), then falls back to
`layer_box.grabbed_layer` and `move_blocked_start` (a canvas layer drag), and only when
`doc.selection` and `st.floating` are both absent for the layer case.

### Multi-window ownership

Each viewport runs `finish` every frame. Two guards keep the state consistent:

```rust
if t.viewport != ctx.viewport_id() {
    return; // a document window must not complete the main window's transfer
}
```

```rust
let (released, down) = ctx.input(|i| (i.pointer.primary_released(), i.pointer.primary_down()));
```

The cleanup path uses the same two rules: it only clears `move_blocked`, `move_blocked_start`,
`move_viewport` and a grabbed `layer_box` in the viewport that owns `move_viewport`, and only on a
primary release.

`check_arranged_drag` carries the equivalent guard so a second window cannot read its own pointer
against another window's gesture and arm a transfer on its behalf:

```rust
if app.move_viewport.is_some_and(|v| v != ctx.viewport_id()) {
    return;
}
```

The blocked-drag handler in `canvas/tool_input.rs` follows the same rule, and only clears the blocked
state and raises the lock dialog in the window that armed it.

## Implementation

### `crates/engine/src/cmd/edit_cmds.rs`

`lift` and `clear_selected` were private to `edit_cmds`; the selection copy needs exactly the same
extraction and clearing semantics as Copy and Cut, so both are exposed crate-wide instead of being
re-implemented:

```rust
pub(crate) fn lift(src: &Surface, sel: Option<&Surface>, canvas: Rect) -> Result<Clip>
pub(crate) fn clear_selected(doc: &mut Document, id: LayerId, background: [f32; 4]) -> Result<()>
```

`lift` returns the selected area (or the whole canvas when there is no selection) as a `Clip` with
its own format and bounds; tiles shared with the source stay copy-on-write.

### `crates/engine/src/cmd/float_cmds.rs`

A floating selection is a `CutParts` holding the lifted piece, the cut-out remainder and the offset.
Three accessors let the copy command move the piece without re-lifting pixels:

```rust
pub fn is_mask(&self) -> bool
pub fn piece(&self) -> (&Surface, Rect)          // the floating pixels/mask and their bounds
pub fn place_rest(&self, doc: &mut Document) -> Result<()>  // write the cut-out remainder back
```

`place_rest` rewrites either the layer surface or the layer mask, which is what makes a cut of a
floating *mask* possible. `keeps_floating` gains `layer.copyToDocument` so the command does not
implicitly drop the floating piece before it reads it.

### `crates/engine/src/cmd/layer_copy_cmds.rs`

`layer.copyToDocument` gains two parameters:

```json
{"selection": false, "cut": false}
```

`selection: false` keeps the old whole-layer behaviour and now drops a floating piece first. The
selection branch is the new work:

1. Reject when the destination document has a running job; reject the source too when `cut` is set.
2. Resolve the mask target from three sources: an explicit `"target": "mask"`, the channel view's
   `LayerMask`, or a mask view that is not a filter mask.
3. If the active layer is floating, carry `parts.piece()` with its bounds and offset; otherwise
   `lift()` the layer surface or the layer mask with the document selection.
4. Reject an empty selected area, and reject either document when it is not layered.
5. Convert the piece to the destination's `PixelFormat` and wrap it in a one-layer scratch
   `Document` so the existing placement code applies.
6. Compute the offset from `center`, `at` or `offset`, adding the floating piece's own offset.
7. For a cut, snapshot the source document, run `Cut Pixels` through `s.edit`, and either
   `place_rest` (floating), `clear_mask`, or `clear_selected`, then clear the selection.
8. Set the destination active and run `Duplicate Layer` through `s.edit`, inserting each copy above
   the previous one.
9. On failure of the destination edit, restore the source snapshot and the previous active document
   and return the error.

The command returns the new layer ids and the applied offset:

```json
{"layers": [12], "offset": [-40, 16]}
```

### `crates/ui-egui/src/panels/layer_transfer.rs`

The module owns the transfer state and every completion path.

* `begin_layers` / `begin_selection` write a `Transfer` into `Context` data.
* `over_tab` promotes a move drag when the pointer is over a tab, switches documents, and drops on
  release; it also drops the Layers panel drag key so the new document's rows do not inherit the
  drag.
* `check_arranged_drag` promotes the drag as soon as the pointer enters another arranged cell.
* `arm_move_drag` cancels the in-flight drag (`app.drag`, `layer_box`, `move_blocked`, `move_preview`,
  `move_mods`) and then arms the transfer; for a selection it also discards a floating piece that
  has not moved yet, so the transfer starts from the original pixels.
* `finish` draws the ghost and the copy cursor while the button is held, and drops on a primary
  release over a target.
* `target_at` maps the screen point to a document and a document point: in an arranged view it
  builds a `ViewXform` from the cell rect, the view's zoom and centre, the flip and rotation, so the
  release lands where the pointer is in the destination.
* `drop` builds the command JSON. A drop back inside the source document with a single layer and a
  frame under the pointer sets the frame's content instead of copying.
* `apply_drop_placement` chooses `offset` when the drag has a grab point, `at` when it does not, and
  `center` when Shift is held; centring is only requested when the documents differ in size.
* `ghost` paints the dragged label beside the pointer in the tooltip layer.

### `crates/ui-egui/src/canvas/tool_input.rs`

* A Move press with Auto-Select records `move_blocked_start` (and `move_viewport`) even on a locked
  or Background layer, so `current_move_drag` can promote the gesture later.
* `handle_move_blocked_event` replaces the inline match: it sets the blocked state on a blocked
  press, keeps it while a transfer is in progress, and shows the lock prompt only in the window
  that owns the gesture.

### `crates/ui-egui/src/tools/move_box.rs`

`LayerBox` gains `grabbed_layer`, and `begin_with_layer` records `move_viewport =
app.current_viewport`, which is what `finish` and `check_arranged_drag` compare against.

### Supporting fields

* `app.rs` / `lib.rs`: `move_viewport: Option<egui::ViewportId>` and
  `current_viewport: Option<egui::ViewportId>`.
* `canvas/tabs.rs`: calls `check_arranged_drag` while a drag is under way.
* `canvas.rs`: exposes the arranged-cell hit test used by `target_at`.

## Types and call graph

The types the feature adds or changes, and who reaches them.

| Type | Where | Fields / variants | Read or written by |
|---|---|---|---|
| `Transfer` | `panels/layer_transfer.rs` (private) | `source: DocId`, `kind: TransferKind`, `grab: Option<[f64; 2]>`, `label: String`, `viewport: egui::ViewportId` | `get`, `clear`, `finish`, `drop`, `over_tab`; stored in `Context` temp data under `key()` |
| `TransferKind` (public) | same file | `Layers(Vec<u64>)`, `Selection { mask: bool }` | `begin_layers`, `begin_selection`, `drop` |
| `MoveDrag` (public) | same file | `Layers([f64; 2])`, `Selection { grab: [f64; 2], cut: bool }` | `current_move_drag`, `arm_move_drag` |
| `LayerBox` | `tools/move_box.rs` (`pub(crate)`) | `start`, `end`, `add`, `grabbed_layer: Option<LayerId>` (the last added; `start`/`end` made `pub(crate)`) | `move_box::draw/pointer/begin_with_layer`; `layer_transfer::current_move_drag`, `arm_move_drag` |
| `CutParts` accessors | `cmd/float_cmds.rs` | `is_mask()`, `piece()`, `place_rest()` (all added) | `cmd/layer_copy_cmds.rs` only |
| `move_blocked_start`, `move_viewport`, `current_viewport` | `app.rs` fields | `Option<[f64; 2]>`, `Option<ViewportId>`, `Option<ViewportId>` | see the table below |

Call graph, measured on the branch:

```
canvas/tabs.rs
  check_arranged_drag(app, ctx)        line 12      (new call, per frame before documents())
  finish(app, ctx)                     lines 14, 832 (existing + detached window)
  pointer_if_armed(app, ctx)           lines 193, 584
  over_tab(app, ctx, i)                lines 341, 693

canvas/tool_input.rs
  is_transfer_in_progress(app)         line 406     (blocked-drag guard)
  move_box::begin_with_layer(...)      line 287     (locked/empty press, new)
  move_box::begin(...)                 line 294     (unchanged path)

panels/layer_row.rs
  begin_from_panel(app, ctx, id)       line 213

panels/layer_drag.rs, channels_panel.rs, history.rs
  ghost(ctx, p, label)                             (shared ghost painter)

tools/move_box.rs
  begin / begin_with_layer  -> app.move_viewport = app.current_viewport
  ToolEvent::Down           -> app.layer_box = None; app.move_viewport = None
  ToolEvent::Up             -> app.layer_box = None; app.move_viewport = None

canvas.rs
  app.current_viewport = Some(ctx.viewport_id())     (line 1901, once per canvas_view)

canvas/tool_input.rs (blocked press)
  move_blocked = true; move_blocked_start = Some([x, y]); move_viewport = current_viewport

panels/layer_transfer.rs
  finish                     reads move_viewport, clears it and move_blocked/start
  check_arranged_drag        returns when move_viewport names another window
  arm_move_drag              clears drag, layer_box, move_blocked, move_blocked_start,
                             move_viewport, move_preview, move_mods
```

`move_viewport` therefore has six writers (`begin_with_layer`, the blocked press, `move_box`'s Down and Up arms, `finish`'s cleanup, `arm_move_drag`) and three readers (`finish`, `check_arranged_drag`, `handle_move_blocked_event`). The invariant is: set when a gesture starts, cleared by every path that ends it.

## Function contracts

Every function in `panels/layer_transfer.rs`, with its exact signature, what it returns, and what it requires of the caller.

| Function | Signature | Returns | Requires | Called from |
|---|---|---|---|---|
| `key` | `fn key() -> egui::Id` | `Id::unique("pc-layer-transfer")` | — | `get`, `clear`, the two `insert_temp` sites |
| `get` | `fn get(ctx: &egui::Context) -> Option<Transfer>` | the armed transfer, or `None` | — | `finish`, `drop`, `over_tab`, `check_arranged_drag`, `is_transfer_in_progress`, `is_transfer_armed` |
| `clear` | `fn clear(ctx: &egui::Context)` | — | — | `finish`, `drop`, `arm_move_drag` (indirectly through the clears) |
| `begin_layers` | `fn begin_layers(app: &PhotocraftApp, ctx: &egui::Context, id: LayerId, grab: Option<[f64; 2]>) -> bool` | `true` when a transfer was armed | an active document | `begin_from_panel`, `arm_move_drag`, `over_tab` |
| `begin_selection` | `fn begin_selection(app: &PhotocraftApp, ctx: &egui::Context, grab: [f64; 2], mask: bool) -> bool` | `true` when armed | an active document | `arm_move_drag` |
| `begin_from_panel` | `pub fn begin_from_panel(app: &PhotocraftApp, ctx: &egui::Context, layer: LayerId)` | — | the layer id from the panel row | `panels/layer_row.rs:213` |
| `current_move_drag` | `pub fn current_move_drag(app: &PhotocraftApp) -> Option<MoveDrag>` | the gesture under way, if any | — | `check_arranged_drag`, `over_tab`, `pointer_if_armed`, tests |
| `pointer_if_armed` | `pub fn pointer_if_armed(app: &PhotocraftApp, ctx: &egui::Context) -> Option<egui::Pos2>` | the pointer only while something is dragged | — | `canvas/tabs.rs:193, 584` |
| `check_arranged_drag` | `pub fn check_arranged_drag(app: &mut PhotocraftApp, ctx: &egui::Context)` | — | called before `documents()` | `canvas/tabs.rs:12` |
| `arm_move_drag` | `fn arm_move_drag(app: &mut PhotocraftApp, ctx: &egui::Context, drag: MoveDrag)` | — | a live `MoveDrag` | `check_arranged_drag`, `over_tab` |
| `over_tab` | `pub fn over_tab(app: &mut PhotocraftApp, ctx: &egui::Context, index: usize)` | — | the tab's document index | `canvas/tabs.rs:341, 693` |
| `is_transfer_in_progress` | `pub fn is_transfer_in_progress(app: &PhotocraftApp) -> bool` | whether the frame's context has an armed transfer | `app.frame_tool_ctx` set | `canvas/tool_input.rs:406` |
| `is_transfer_armed` | `pub fn is_transfer_armed(ctx: &egui::Context) -> bool` (`#[cfg(test)]`) | whether `ctx` has an armed transfer | — | tests only |
| `finish` | `pub fn finish(app: &mut PhotocraftApp, ctx: &egui::Context)` | — | called after `documents()` in every viewport | `canvas/tabs.rs:14, 832` |
| `target_at` | `fn target_at(app: &PhotocraftApp, p: egui::Pos2) -> Option<(usize, Option<[f64; 2]>)>` | the document under `p` and the document point there | `last_canvas_rect` current for the frame | `finish` |
| `drop` | `fn drop(app: &mut PhotocraftApp, ctx: &egui::Context, dest: usize, at: Option<[f64; 2]>)` | — | an armed transfer | `finish`, `over_tab` |
| `apply_drop_placement` | `fn apply_drop_placement(p: &mut serde_json::Value, at: Option<[f64; 2]>, grab: Option<[f64; 2]>, shift: bool, same_size: bool)` | — | a command JSON with `document` and `source` | `drop` |
| `ghost` | `pub fn ghost(ctx: &egui::Context, p: egui::Pos2, label: &str)` | — | — | `finish`, `layer_drag.rs:109, 284`, `channels_panel.rs:467`, `history.rs:104` |

The two `begin_*` functions return `bool` rather than `()`, and every caller ignores the value except `over_tab`, which uses it to decide whether the drag should keep its move state. A `false` means "no active document", which cannot happen from a UI path but can happen in a test that clears the session.

## Transfer storage

The transfer lives in `egui::Context` temp data, not in `PhotocraftApp`. Three lines define the storage:

```rust
fn key() -> egui::Id { egui::Id::unique("pc-layer-transfer") }
fn get(ctx: &egui::Context) -> Option<Transfer> { ctx.data(|d| d.get_temp::<Transfer>(key())) }
fn clear(ctx: &egui::Context) { ctx.data_mut(|d| d.remove::<Transfer>(key())); }
```

`Id::unique("pc-layer-transfer")` is a process-wide unique id derived from the string; it is stable across calls and distinct from every other `Id` in the app. Using a literal rather than a hash of the source document is deliberate: there is at most one transfer at a time, and a per-document key would allow two windows to arm two transfers that fight over the same pixels.

`get_temp` and `insert_temp` are egui's non-persisted storage: the value lives as long as the context and is not serialised. That is the correct lifetime for gesture state - a transfer that outlived the process would be meaningless, and persisting it would write document ids to disk that are invalid on the next launch.

The storage is shared by every viewport of the context, which is why `Transfer` carries `viewport`: the value is visible to all of them, so the ownership information has to travel with it. `get` returns an owned clone (`get_temp::<Transfer>` clones), so no borrow of the context data is held while `finish` mutates the app; that is why the type derives `Clone`.

`clear` removes by type and key. A second call is a no-op, which is why the paths that can race (`finish` after `drop`) do not need to check first: `drop` clears, then `finish`'s trailing `clear(ctx)` clears again harmlessly.

## Error surface

Every failure the feature can produce, where it originates, and the exact text. All of these reach the status bar through `app.ui.status` / `status_error` in `drop`, or through `report_smart_result` for the Quick Selection stroke.

| Origin | Variant | Message | When |
|---|---|---|---|
| `copy_to_document` | `EngineError::Disabled(CMD, msg)` | `command "<id>" is not available right now: "<job label>" is still running on this document; wait for it to finish or cancel it (Esc)` | destination has a running job; for a cut, also the source |
| `copy_to_document` | `EngineError::BadParams { cmd, msg }` via `bad(...)` | `invalid parameters for "layer.copyToDocument": missing \`document\` (the destination's index)` | no `document` field |
| `copy_to_document` | `EngineError::BadParams` | `... no active layer in source document` | `active_layer` is `None` |
| `copy_to_document` | `EngineError::NoLayer(id)` | `no such layer LayerId(n)` | the active layer id is stale |
| `copy_to_document` | `EngineError::Other(msg)` | `the active layer has no layer mask` | mask target but no mask |
| `copy_to_document` | `EngineError::Other(msg)` | `the active layer has no pixels to copy` | pixel target but no surface (a group) |
| `copy_to_document` | `EngineError::BadParams` | `... the selected area is empty` | the clip's bounds are empty |
| `copy_to_document` | `EngineError::Other(msg)` | `layers can't be copied with a Rgb8 source document` (or destination) | a non-layered document, named by role |
| `copy_to_document` | propagated from `lift` | whatever `lift` returns | a surface/geometry failure |
| `copy_to_document` | `EngineError::Other(msg)` | from `scratch_health()` | a cut when a document's scratch space is unhealthy |
| `copy_to_document` | propagated from `s.edit` | whatever the edit returns | the destination edit failed; source restored |
| `drop` (UI) | — | the engine message | copied into `app.ui.status`, `status_error = true` |
| `drop` (UI, in-document) | — | the `layer.frame.setContent` error | same |

The two `Disabled` messages are the only ones that mention a job, because they are the only two places a background job can block the operation. The two `Other` messages about surfaces are distinguished by the target (mask vs pixels), because a user who selected a mask on a group and a user who selected pixels on a group need different fixes.

## What the engine does with a selection transfer

The UI's job ends at building the JSON; the behaviour the user sees is `copy_to_document` in `crates/engine/src/cmd/layer_copy_cmds.rs`. Its `selection` branch is what a cross-document drag of selected pixels runs, and reading it is necessary to understand why `drop` sets the fields it sets.

The branch is entered by `p.get("selection").and_then(Value::as_bool).unwrap_or(false)`, so an absent key means a layer copy - the old path - and the command stays backward compatible.

`cut` is read next, and a cut adds a second job check: `if cut && let Some(job) = s.job_on(sdoc.id)` refuses with `Disabled` naming the job. A plain copy checks only the destination document, because a copy does not write the source; a cut does, so both documents must be idle. That asymmetry is why `drop` always sends `cut: false` cross-document: it removes the possibility of the refusal entirely rather than reporting an error the user cannot act on.

The mask decision has three inputs, which is more than `TransferKind::Selection { mask }` carries: `crate::channel_cmds::target_of(p) == Target::Mask` (the explicit `"target": "mask"` the UI sends), `from.channel_view.target == ChannelTarget::LayerMask` (the source document's channel view), or `crate::mask_view_cmds::current(from).is_some_and(|v| !v.filter)` (a mask view is open and is not a filter preview). The second and third are document state, so a transfer that sends no `target` can still be a mask transfer if the source document is showing a mask. That is why the UI's flag is a record of what was true at arm time rather than the only input.

The float check comes before the lift: `crate::float_cmds::floating(from).filter(|f| f.layer == id)` produces `(piece, bounds, offset, parts)` when the active layer has a floating selection. In that case the clip surfaces are the float's `piece` and `bounds`, not a fresh `lift` of the layer - a floating piece is already the selected pixels, lifted and displaced, and lifting again would read the layer underneath. This is the path that makes "drag a floating selection to another document" move the piece itself.

Without a float, the source surface is chosen by `mask_target` (the layer's mask, erroring with "the active layer has no layer mask" if there is none) or the layer's surface (erroring with "the active layer has no pixels to copy" for a group), and `lift(surf, sdoc.selection.as_ref(), sdoc.bounds())` clips it to the selection. That call is the same one `edit.copy` uses, so a cross-document drag and a copy produce identical pixel content for the same selection.

`if clip_bounds.is_empty() { return Err(bad("the selected area is empty")) }` is the empty-selection guard, and it fires after the lift because an empty selection and a selection entirely outside the canvas both produce empty bounds.

Both documents must be layered: `for (doc, role) in [(&sdoc, "source"), (&ddoc, "destination")] { if !layered(doc.mode) { return Err(Other(format!("layers can't be copied with a {:?} {role} document", doc.mode))) } }`. The message names the role, because a 16-bit or 32-bit document cannot hold a layered copy and the user needs to know which of the two is the problem.

`target_fmt = PixelFormat::new(sdoc.mode, sdoc.depth, true)` is the source's format with alpha forced on - the same rule `lift` uses - and `clip_surface.convert(target_fmt)` runs only when the formats differ. The converted surface then becomes the pixels of a freshly built `Document` named `""`, carrying the source's ICC profile and DPI, containing one raster layer named by `ddoc.next_layer_name("Layer")`.

Placement is decided by three optional fields in priority order: `"center"` aligns the clip's centre with the destination canvas's centre; `"at"` puts the clip's centre on that point; otherwise `"offset"` is used as-is, defaulting to `(0, 0)`. `content` is `clip_bounds.translate(float_off.0, float_off.1)`, and the final `dx, dy` adds `float_off` again after rounding - the float's offset is expressed in the piece's own coordinate space, so it has to be applied to the bounds for the centre computation and to the translation for the pixels.

`cut` then runs as a separate edit on the source document: both documents' `scratch_health()` are checked, a snapshot (`pre_cut`) is taken for rollback, the source becomes active, and `s.edit("Cut Pixels", ...)` either places the rest of the float (`parts.place_rest(doc)`), clears the mask (`clear_mask`), or clears the selected pixels (`clear_selected`), then sets `doc.selection = None`. The order matters: the destination copy is built before the source is modified, so a failure in the cut leaves the copy in place and the source intact.

Finally `s.set_active(dest)` and `s.edit("Duplicate Layer", ...)` insert the copies above the destination's active layer through `layer_place::insert_new`, advancing `above` and switching `place` from `Place::AutoEditable` to `Place::Above` after the first insert so multiple layers keep their order.

For the cross-document drag this pull request implements, `selection: true`, `cut: false`, and either `at`/`offset` or `center` are the only fields sent, so the branch runs the copy arm with no source edit - the source keeps its pixels, which is what the module documentation promises.

## Behaviour details

| Situation | Result |
|---|---|
| Selection drag, no modifiers, different-size documents | The copy keeps the drag's grab offset; Shift centres it |
| Drop on the source document, one layer, over a frame | `layer.frame.setContent` - the frame adopts the layer |
| Drop on the source document, selection | Nothing is copied; the transfer clears |
| Pointer leaves the canvas rect | `target_at` returns `None`; no drop |
| Secondary button released mid-drag | Transfer stays armed; the ghost keeps following |
| Another viewport finishes its frame | Transfer stays armed; blocked state untouched |
| Primary release over a non-source document | `layer.copyToDocument` runs, then the transfer clears |
| Floating selection that never moved | Discarded before arming, so the original pixels travel |
| Mask view / layer-mask channel / `"target":"mask"` | Mask content travels, layer pixels do not |
| Destination has a running job | Command returns `Disabled` with the job's label |
| Destination edit fails | Source snapshot and previous active document are restored |

## Testing

`crates/engine/src/cmd/layer_copy_cmds/tests.rs`

* `copies_selected_pixels_to_destination_document_with_copy_and_cut` - a copy lands at the offset and
  leaves the source intact; the cut variant clears the source.
* `copying_or_cutting_a_mask_selection_transfers_the_mask_without_touching_layer_pixels` - the mask
  travels, the layer pixels do not.

`crates/ui-egui/src/panels/layer_transfer/tests.rs`

| Test | Covers |
|---|---|
| `a_move_tool_selection_drag_onto_a_tab_copies_selected_pixels_to_destination` | Tab hover transfer of a selection |
| `a_move_tool_mask_selection_drag_onto_a_tab_copies_the_mask_and_leaves_layer_pixels` | Mask transfer; source mask and selection unchanged |
| `a_move_tool_selection_drag_in_arranged_view_transfers_directly_to_neighbour_cell` | Arranged-view selection transfer |
| `a_move_tool_background_layer_drag_in_arranged_view_transfers_directly_to_neighbour_cell` | Background layer transfer in an arranged view |
| `a_move_tool_drag_onto_a_tab_copies_the_layer_under_the_pointer_and_leaves_the_source` | Whole-layer transfer |
| `a_layers_panel_drag_onto_a_tab_drops_the_layer_where_it_is_released` | Panel-origin transfer |
| `dropping_back_on_the_source_copies_nothing` | In-document drop |
| `shift_centres_the_copy_when_the_documents_differ_in_size` | Shift centring |
| `same_document_blocked_move_with_multiple_documents_shows_lock_message` | Lock feedback with two documents |
| `blocked_move_gesture_release_clears_blocked_state` | Blocked state cleared on release |
| `extra_window_move_gesture_preserved_when_main_window_finishes` | Gesture survives another window's `finish` |
| `a_document_window_does_not_complete_a_transfer_armed_in_the_main_window` | Viewport ownership of completion |
| `a_secondary_button_release_keeps_the_marquee_until_the_primary_release` | Marquee survives a secondary release |
| `a_secondary_button_release_does_not_complete_a_transfer` | Transfer survives a secondary release |

The last two were checked in the negative: with the guard reverted to `any_released`, the
`a_secondary_button_release_does_not_complete_a_transfer` assertion fails, so the test detects the
regression rather than merely passing.

## Limitations

* Every completion path needs the pointer over a document area, a tab, or an arranged cell; drops on
  panels outside the canvas are ignored.
* `target_at` uses the canvas rect and arranged cells only; a drop onto a floating window that is not
  an arranged cell is not mapped.
* A layer mask transfer needs the mask channel view or an explicit `"target": "mask"`; there is no
  implicit "the active layer's mask" fallback.
* The command refuses documents that are not layered, and refuses while a job runs on the
  destination (or on the source for a cut).
* Cut mode is exercised by the engine tests; the canvas paths always copy (`cut: false`).

## Verification

```
cargo test  -p photocraft-engine --lib layer_copy
cargo test  -p photocraft-ui-egui --lib layer_transfer
cargo clippy -p photocraft-engine -p photocraft-ui-egui --lib -- -D warnings
cargo fmt --check -p photocraft-engine -p photocraft-ui-egui
cargo xtask layers
```

Measured on the branch: engine `layer_copy` 26 passed, ui-egui `layer_transfer` 14 passed, clippy
0 warnings, `cargo xtask layers` 31 crates with 0 violations.

## Differences from upstream `main`

Measured with `git diff -U3 upstream/main...main`, where `upstream/main` is `95c9e465d` (storytold/photocraft) and `main` is `f328d1881`. The full range is 49 files, 8101 insertions, 182 deletions. The hunks below are every hunk in the files that carry this feature - 27 hunks, reproduced in full with their context lines, each followed by an explanation of the original code, the change, and its effect.

### `crates/engine/src/cmd/edit_cmds.rs`

#### 5. `crates/engine/src/cmd/edit_cmds.rs` — hunk -205,7 +205,7

**Original code (upstream `main`)**

```rust
/// selected or no selection, already in the clip's format) is shared copy-on-write, so a plain
/// Copy or Select All, Copy costs a pointer per tile; only the other tiles are read and written,
/// one at a time, instead of the whole area as one `f32` buffer.
fn lift(src: &Surface, sel: Option<&Surface>, canvas: Rect) -> Result<Clip> {
    let fmt = src.format();
    let with_alpha = PixelFormat::new(fmt.mode, fmt.sample, true);
    let area = match sel {
```

**Changed code (current `main`)**

```rust
/// selected or no selection, already in the clip's format) is shared copy-on-write, so a plain
/// Copy or Select All, Copy costs a pointer per tile; only the other tiles are read and written,
/// one at a time, instead of the whole area as one `f32` buffer.
pub(crate) fn lift(src: &Surface, sel: Option<&Surface>, canvas: Rect) -> Result<Clip> {
    let fmt = src.format();
    let with_alpha = PixelFormat::new(fmt.mode, fmt.sample, true);
    let area = match sel {
```

**What the original code does, line by line.**

The three doc lines above the signature are the design statement for the whole function. "selected or no selection, already in the clip's format" says the output is always a `Clip` whose pixel format is the source's format with alpha forced on, whether or not a selection exists. "is shared copy-on-write" says the returned tiles are the same allocations as the source's tiles until one side writes; a copy of a 10,000 x 10,000 layer therefore costs a pointer per tile, not a second image. "only the other tiles are read and written, one at a time, instead of the whole area as one `f32` buffer" says the function never materialises the selection as one contiguous buffer: it walks the tile grid and touches only the tiles the selection covers, so a small selection on a large layer costs the small selection.

The signature takes three inputs and returns one value. `src: &Surface` is the layer (or mask) being read; it is borrowed, never mutated. `sel: Option<&Surface>` is the document selection as a mask surface; `None` means "the whole document", which is why the callers can use one function for Copy with and without a selection. `canvas: Rect` is the document's bounds, used to clip a selection that extends past the canvas edge. The return type is `Result<Clip>`, so an unusable selection is an error rather than an empty copy.

The body starts with `let fmt = src.format();` and `let with_alpha = PixelFormat::new(fmt.mode, fmt.sample, true);`. The first reads the source's pixel format (mode, sample type, channel count, alpha). The second rebuilds the same format with alpha forced on. Copying into a clip always produces an alpha channel even when the source has none, because the clipboard has to represent "transparent where nothing was selected"; a selection that stops halfway through a pixel row needs partial alpha to be correct. `let area = match sel {` then chooses the region: with a selection mask it is the mask's content bounds intersected with the canvas, without one it is the canvas itself. The intersection is what stops a selection that was made before the canvas was resized from asking for pixels outside the document.

**What the change does.**

It adds `pub(crate)` to the signature and nothing else: `fn lift(...)` becomes `pub(crate) fn lift(...)`. No line of the body changes, no type changes, no behaviour changes. The doc comment above it is untouched.

**Why this shape, and the alternatives rejected.**

The selection branch of `layer.copyToDocument` needs exactly this extraction: given a layer surface, an optional selection and the canvas, produce the pixels and their bounds. There are four ways to get it, and three are wrong here.

Making the function `pub` would put it on the engine crate's public surface. That surface is a promise to every caller outside the crate, and the function's contract (tile sharing, forced alpha, clipping) is an implementation detail of Copy, not a general-purpose API. Exposing it would freeze the signature for reasons unrelated to any external need.

Duplicating the body inside `layer_copy_cmds` is the option that looks cheapest and costs most. The body is not three lines: it walks tiles, converts formats, intersects bounds and decides alpha. Two copies of that logic drift the first time one of them is fixed - and Copy, Cut and the cross-document transfer must agree, because a transfer that produced different pixels from Copy on the same selection would be a defect the user sees immediately.

A public wrapper in `edit_cmds` (`pub(crate) fn lift_for_copy(...)` calling the private one) adds a layer whose only job is to repeat the same signature. A trait would let the behaviour vary, which is the opposite of what is wanted.

`pub(crate)` is the smallest correct answer: visible to `layer_copy_cmds` because they are in the same crate, invisible everywhere else. It also documents, at the definition site, that this is an internal helper shared by two commands rather than a general utility.

**Effect and blast radius.**

The change is a visibility change only. The compiled code is identical; the function was already reachable from `edit_cmds`. Existing callers - Copy, Cut, Select All, Copy - are unaffected. The new caller adds one edge in the call graph, from `layer_copy_cmds::copy_to_document` to `edit_cmds::lift`, which is why `edit_cmds` and `layer_copy_cmds` now sit in the same dependency layer; `cargo xtask layers` verifies that no cycle is introduced.

The edge cases the new caller inherits are the ones Copy already lives with, and they are all deliberate. No selection: the whole canvas is lifted. A selection entirely outside the canvas: the intersection is empty, the clip has zero bounds, and the caller rejects it with "the selected area is empty" rather than inserting an empty layer. A source without alpha: the clip gains alpha, so the copied layer has a transparent border matching the selection's edge. A selection that is partially covered: the clip keeps the partial alpha, which is what makes a soft-edged selection travel as a soft-edged copy.

#### 6. `crates/engine/src/cmd/edit_cmds.rs` — hunk -455,7 +455,8

**Original code (upstream `main`)**

```rust
    Ok(json!({"bounds": [b.x0, b.y0, b.width(), b.height()], "target": if filter { "filterMask" } else { "mask" }}))
}

fn clear_selected(doc: &mut Document, id: LayerId, background: [f32; 4]) -> Result<()> {
    let sel = doc.selection.clone();
    let canvas = doc.bounds();
    let area = sel.as_ref().map_or(canvas, |m| m.content_bounds().intersect(&canvas));
```

**Changed code (current `main`)**

```rust
    Ok(json!({"bounds": [b.x0, b.y0, b.width(), b.height()], "target": if filter { "filterMask" } else { "mask" }}))
}

/// Clear the selected area of layer `id` to `background`, leaving the rest untouched.
pub(crate) fn clear_selected(doc: &mut Document, id: LayerId, background: [f32; 4]) -> Result<()> {
    let sel = doc.selection.clone();
    let canvas = doc.bounds();
    let area = sel.as_ref().map_or(canvas, |m| m.content_bounds().intersect(&canvas));
```

**What the original code does, line by line.**

The hunk begins at the end of `copy_mask`, which returns a JSON summary of the copied mask's bounds and target. The line `Ok(json!(...))` is that return; it is context, not part of the change. The closing brace ends `copy_mask`, and the blank line separates it from `clear_selected`.

`clear_selected(doc: &mut Document, id: LayerId, background: [f32; 4]) -> Result<()>` is the function this hunk is about. Its name says what it does: clear the selected area of one layer. The parameters carry the whole contract. `doc: &mut Document` means it edits in place: there is no return value describing a new document, so every caller must already be inside an edit that can be undone as one step. `id: LayerId` names the layer, not the active layer, so the operation works on a layer the caller chose rather than on whatever happens to be selected. `background: [f32; 4]` is the colour written into the cleared area; it is four floats because the engine works in linear float RGBA, not in 8-bit bytes.

The first three body lines, which the hunk shows as context, are the bounds computation. `let sel = doc.selection.clone();` clones the selection surface. It is a clone, not a borrow, because the function is about to mutate the document and the selection lives inside it; holding a borrow would not compile, and the clone is cheap because selection surfaces are tile-shared. `let canvas = doc.bounds();` reads the document's pixel bounds. `let area = sel.as_ref().map_or(canvas, |m| m.content_bounds().intersect(&canvas));` is the same rule `lift` uses: with a selection, the mask's content bounds clipped to the canvas; without one, the whole canvas. That symmetry is what makes Copy and Cut cover exactly the same region.

**What the change does.**

Two things. It adds a doc comment, `/// Clear the selected area of layer `id` to `background`, leaving the rest untouched.`, and it changes `fn clear_selected(...)` to `pub(crate) fn clear_selected(...)`. The body is untouched.

**Why this shape, and the alternatives rejected.**

The doc comment exists because the function is now part of a shared internal surface. A one-line comment that states the three facts a caller needs - what is cleared, which layer, what is left alone - is the minimum for a function other modules may call. "Leaving the rest untouched" is the part that matters for the transfer: a caller reading only the signature might assume the function clears the layer, and the difference between "clear the selection" and "clear the layer" is the difference between a correct cut and a destroyed document.

`pub(crate)` has the same reasoning as `lift`: the selection cut path in `layer.copyToDocument` must clear pixels with exactly the semantics of the existing Cut command. Re-implementing the clearing would duplicate three decisions: the bounds intersection, the background colour source (the tool's background colour, passed by the caller), and what happens to a selection that lies partly outside the canvas. Any divergence would show as a cut whose source rectangle differs from the rectangle the copy took, which is the classic half-copied, half-cleared artifact.

The alternatives are the same four as for `lift`, with one extra consideration. Exposing the function publicly would let plugins clear arbitrary regions without going through a command, which would bypass the undo journal; the engine's rule is that document mutations happen inside commands so they can be undone as one step. `pub(crate)` keeps the function inside the crate that owns the journal.

**Effect and blast radius.**

Visibility only; no codegen change, no behaviour change. Existing callers - the Cut command, the delete-selection path - behave identically. The new caller is the `cut: true` branch of `layer.copyToDocument`, which calls it inside its own `s.edit("Cut Pixels", ...)` so the clear and the document switch are one undoable step.

The edge cases are worth stating because the transfer depends on them. With no selection, the function clears the entire canvas of that layer - which is what Cut with no selection does. With a selection that extends past the canvas, `intersect` clips it, so the clear covers exactly the visible part. With a mask missing on the layer, the caller never reaches this function for the mask case; the mask branch has its own `clear_mask`. With a layer that has no pixels (a group), the function errors through `surface_mut` rather than silently doing nothing.

### `crates/engine/src/cmd/float_cmds.rs`

#### 7. `crates/engine/src/cmd/float_cmds.rs` — hunk -103,6 +103,28

**Original code (upstream `main`)**

```rust
        self.outline.as_ref()
    }

    /// `doc` with the piece moved by whole pixels (dx, dy) over the cut-out layer.
    pub fn moved(&self, doc: &Document, dx: i32, dy: i32) -> Result<Document> {
        let mut d = doc.clone();
```

**Changed code (current `main`)**

```rust
        self.outline.as_ref()
    }

    /// Whether this floating piece is a layer mask (`"target":"mask"`).
    pub fn is_mask(&self) -> bool {
        self.mask
    }

    /// The floating pixel or mask piece and its bounds.
    pub fn piece(&self) -> (&Surface, photocraft_geom::Rect) {
        (&self.piece, self.piece_bounds)
    }

    /// Replace the layer's pixels (or mask) with the cut-out remainder.
    pub fn place_rest(&self, doc: &mut Document) -> Result<()> {
        let l = doc.layer_mut(self.layer).ok_or(EngineError::NoLayer(self.layer))?;
        let surf = if self.mask {
            l.mask.as_mut().map(|m| &mut m.surface).ok_or_else(|| EngineError::Other("the layer has no mask".into()))?
        } else {
            l.surface_mut().ok_or_else(|| EngineError::Other("the layer has no pixels to move".into()))?
        };
        *surf = self.rest.clone();
        Ok(())
    }

    /// `doc` with the piece moved by whole pixels (dx, dy) over the cut-out layer.
    pub fn moved(&self, doc: &Document, dx: i32, dy: i32) -> Result<Document> {
        let mut d = doc.clone();
```

**What the original code does, line by line.**

The visible context is the tail of an earlier accessor: `self.outline.as_ref()` returns the floating piece's outline (the marching-ants path), and the closing brace ends that function. The next item is `moved`: `/// `doc` with the piece moved by whole pixels (dx, dy) over the cut-out layer.` and `pub fn moved(&self, doc: &Document, dx: i32, dy: i32) -> Result<Document>` followed by `let mut d = doc.clone();`. That is a *pure* operation: it clones the document, moves the floating piece by whole pixels over the cut-out remainder, and returns the new document. The floating state itself is not mutated.

So the original `CutParts` offered: the outline, a whole-document transform (`moved`), and whatever the rest of the impl block exposes. It did not offer a way to ask "is this mask or pixels?", a way to borrow the piece and its bounds, or a way to write the remainder back into the layer. Those three gaps are exactly what a cross-document transfer needs.

**What the change does.**

Three accessors are inserted between the outline accessor and `moved`.

`is_mask(&self) -> bool` returns `self.mask`, the flag `CutParts` already carries to say whether the floating content is a layer mask rather than layer pixels. It is a one-line reader, but it is the flag the transfer needs before it decides what to send: a mask transfer must set the command's `target` to `"mask"`, and a pixel transfer must not.

`piece(&self) -> (&Surface, photocraft_geom::Rect)` returns a borrow of the floating surface and its bounds. The tuple shape is deliberate: the caller needs both the pixels and where they sit, and the bounds are what the destination placement uses to compute the offset. Returning a borrow means the transfer reads the piece without consuming or cloning it; the float stays intact for the rest of the gesture, so if the drop is cancelled the user still has their floating selection.

`place_rest(&self, doc: &mut Document) -> Result<()>` writes the cut-out remainder back. The body resolves the layer with `doc.layer_mut(self.layer)`, then picks the surface: if `self.mask`, it takes `l.mask.as_mut()...surface` and errors with "the layer has no mask" when there is none; otherwise it takes `l.surface_mut()` and errors with "the layer has no pixels to move" when the layer has no pixels (a group, for example). It then assigns `*surf = self.rest.clone()` and returns `Ok(())`.

**Why this shape, and the alternatives rejected.**

The cut path needs these three questions answered and nothing else. `moved` cannot serve, because it returns a whole document and does not write the remainder into a live layer; a caller would have to diff the returned document back into the session, which is both expensive and wrong inside an undoable edit. Making `CutParts` expose its fields would leak the representation (three surfaces, bounds, offset, mask flag) into every caller and let them construct inconsistent states.

`place_rest` mutates rather than returning a new document because the caller is already inside `s.edit("Cut Pixels", ...)`, which owns the undo step. Returning a document would force the caller to replace the whole document inside that edit, defeating the incremental tile writes the engine uses.

The error messages are specific on purpose. "the layer has no mask" and "the layer has no pixels to move" reach the status bar as-is, so a user who cuts a mask from a layer that has no mask reads the cause rather than a generic failure.

The clone of `self.rest` is required by the borrow checker: `surf` is a mutable borrow out of `doc`, and `self.rest` is behind `&self`, so the assignment needs an owned value. The clone is cheap for the same copy-on-write reason as the rest of the engine.

**Effect and blast radius.**

The struct gains no fields and the floating state machine is unchanged; the three accessors are additive. `moved` and the outline accessor are untouched. The new caller is the `cut: true` branch of `layer.copyToDocument`, which uses `is_mask` to decide the target, `piece` to read what to copy, and `place_rest` to write the remainder instead of clearing a rectangular selection.

The edge cases fall out of the bodies. A floating piece on a layer whose mask was removed after the float began: `place_rest` errors rather than silently writing pixels. A float whose layer was deleted: `layer_mut` returns `NoLayer(self.layer)`. A float that has not moved: the transfer discards it before arming (see `arm_move_drag`), so it never reaches `place_rest`; a cut of an unmoved float is therefore identical to a cut of the selection.

#### 8. `crates/engine/src/cmd/float_cmds.rs` — hunk -340,7 +362,7

**Original code (upstream `main`)**

```rust

/// Commands that leave a floating piece floating: its own, and ones that don't touch documents.
fn keeps_floating(id: &str) -> bool {
    matches!(id, "select.float" | "select.drop" | "brush.mixer.sample" | "brush.mixer.load" | "brush.mixer.clean")
        || ["view.", "window.", "help.", "prefs."].iter().any(|p| id.starts_with(p))
}

```

**Changed code (current `main`)**

```rust

/// Commands that leave a floating piece floating: its own, and ones that don't touch documents.
fn keeps_floating(id: &str) -> bool {
    matches!(id, "select.float" | "select.drop" | "layer.copyToDocument" | "brush.mixer.sample" | "brush.mixer.load" | "brush.mixer.clean")
        || ["view.", "window.", "help.", "prefs."].iter().any(|p| id.starts_with(p))
}

```

**What the original code does, line by line.**

`/// Commands that leave a floating piece floating: its own, and ones that don't touch documents.` is the rule in one sentence. `fn keeps_floating(id: &str) -> bool` answers, for a command id, whether executing that command must preserve a floating selection instead of committing it.

The body has two clauses joined by `||`. The first is `matches!(id, "select.float" | "select.drop" | "brush.mixer.sample" | "brush.mixer.load" | "brush.mixer.clean")`. `select.float` creates the float, so it obviously keeps it; `select.drop` commits it, so it must be allowed to run without the generic machinery committing it first (otherwise drop would be a no-op on an already-dropped selection). The three Mixer Brush commands sample or load colour and do not write the document, so they must not disturb a float.

The second clause is `["view.", "window.", "help.", "prefs."].iter().any(|p| id.starts_with(p))`: every navigation, window, help and preference command leaves the document alone, so none of them may commit a float. This is a prefix test rather than an enumeration because those namespaces are open-ended.

**What the change does.**

One id is added to the match: `"layer.copyToDocument"`. Nothing else changes.

**Why this shape, and the alternatives rejected.**

The command dispatcher runs a generic "commit the float before a document command" step. `keeps_floating` is the exemption list for that step. Without `layer.copyToDocument` on the list, running the command would first commit the floating piece into the layer, and the transfer would then see an ordinary layer instead of the float. The visible result would be a bug of a specific kind: dragging a floating selection to another document would copy the *whole layer* (because the selection branch would find no float and would fall back to `lift` over the layer surface with the selection mask), or would copy nothing when there is no selection mask at all.

The alternatives are worse. Making the dispatcher skip the commit for every command whose id starts with `layer.` would exempt layer commands that genuinely must commit (a layer rename, say, should not leave a float hanging over stale state). Special-casing the command inside `copy_to_document` is impossible: the commit happens before the command body runs, so by the time the function sees the session the float is already gone. Adding a separate "read-only" parameter would duplicate the concept the list already expresses.

Adding the id to the existing list is the smallest expression of the actual rule: this command reads the float and must not consume it. The list is the one place the rule is written, so a reader looking for "which commands may run over a float" finds the answer in one function.

**Effect and blast radius.**

The command now runs with the float intact. Every other command is unaffected: the match is an enumeration plus a prefix test, and adding one literal cannot change another id's answer.

The interaction with `select.drop` is worth noting. Both are on the list, so a `select.drop` immediately followed by `layer.copyToDocument` still sees a float: the drop command is what commits it, and the transfer does not. After the transfer, the float is still floating, which is why `arm_move_drag` discards an unmoved float before arming: the transfer itself must not leave the user with a float whose pixels were already copied.

The edge case the list does not cover, and should not, is a command that both reads and writes the float. There is none today; a future one would need its own rule rather than a prefix.

### `crates/engine/src/cmd/layer_copy_cmds.rs`

#### 9. `crates/engine/src/cmd/layer_copy_cmds.rs` — hunk -211,6 +211,9

**Original code (upstream `main`)**

```rust

/// `layer.copyToDocument`.
fn copy_to_document(s: &mut Session, p: &Value) -> Result<Value> {
    let dest = doc_index(s, p, "document")?.ok_or_else(|| bad("missing `document` (the destination's index)"))?;
    let src = match doc_index(s, p, "source")? {
        Some(i) => i,
```

**Changed code (current `main`)**

```rust

/// `layer.copyToDocument`.
fn copy_to_document(s: &mut Session, p: &Value) -> Result<Value> {
    if !p.get("selection").and_then(Value::as_bool).unwrap_or(false) {
        crate::float_cmds::drop_floating(s)?;
    }
    let dest = doc_index(s, p, "document")?.ok_or_else(|| bad("missing `document` (the destination's index)"))?;
    let src = match doc_index(s, p, "source")? {
        Some(i) => i,
```

**What the original code does, line by line.**

`/// `layer.copyToDocument`.` is the command's own doc line, the only documentation the function carries; the parameter schema lives in the `specs` table further down. `fn copy_to_document(s: &mut Session, p: &Value) -> Result<Value>` is the command entry point: it receives the whole session and the command's JSON parameters, and returns either a JSON summary or an engine error.

The first statement, `let dest = doc_index(s, p, "document")?.ok_or_else(|| bad("missing `document` (the destination's index)"))?;`, resolves the destination document. `doc_index` reads the `document` field, validates that it names an existing index, and returns `Option<usize>`; the `ok_or_else` turns a missing field into a user-facing error naming the field. Note the error is `bad`, not a panic: a malformed automation call must not take the process down. The second statement, `let src = match doc_index(s, p, "source")? { Some(i) => i,`, resolves the source document from the optional `source` field; when it is absent the source is the active document, which is why the match has a `None` arm on the next line (not shown here).

So the original function had exactly one behaviour: copy whole layers. There was no branch, no float handling, no selection handling. Whatever was floating in the session at the time reached the command in whatever state the command dispatcher had left it in.

**What the change does.**

Three lines are inserted before the destination is resolved:

```rust
if !p.get("selection").and_then(Value::as_bool).unwrap_or(false) {
    crate::float_cmds::drop_floating(s)?;
}
```

The condition reads the new `selection` parameter as a bool, defaulting to `false` when it is absent or not a bool. When it is false - the old, whole-layer behaviour - the command first commits any floating selection by calling `drop_floating`. When it is true, the float is deliberately left alone.

**Why this shape, and the alternatives rejected.**

The ordering matters more than the condition. `drop_floating` runs before `doc_index`, so a malformed `document` field still commits the float. That is consistent with the dispatcher's own rule (a document command commits the float before it runs); the command is simply doing explicitly what it previously relied on the caller to have done. Doing it after the destination resolved would mean a bad index left the float committed anyway - a surprising side effect for a command that failed - or required an early return that skipped the commit, which would make the float survive a failed command and confuse the next one.

Putting the commit inside the `selection` branch instead would be too late. The selection branch reads `crate::float_cmds::floating(from)` to decide whether the content is a float; if the commit had already happened, that query returns `None` and the branch would fall back to lifting the layer surface with the document selection. For a float whose pixels were moved, that fallback copies the *original* pixels in the *original* position, not what the user sees floating. The two behaviours are only distinguishable because the commit is skipped in exactly the case that needs the float.

The alternative of never committing the float - making the command always read `floating()` - would change the old whole-layer behaviour: a user with a floating selection who drags a layer from the Layers panel expects the float to be committed first, as it is for every other document command. Skipping the commit would copy a layer that still has a hole in it.

`unwrap_or(false)` rather than `unwrap_or(true)` is the conservative default. A caller that does not know about the parameter gets the old behaviour, not a new one. Automation scripts written before this change keep working.

**Effect and blast radius.**

For every existing caller the parameter is absent, so the condition is true and `drop_floating` runs: identical behaviour to before, with one difference - the commit is now explicit inside the command rather than implicit in the dispatcher. The dispatcher's generic commit still runs first (because `keeps_floating` does not list this command), so in practice `drop_floating` is a no-op for old callers; the line exists for the new callers and for any path that reaches the command without the dispatcher.

The new caller sends `selection: true`, so the float survives into the command body. That is what the selection branch reads. The edge case worth naming: a caller that sends `selection: true` with no float and no selection. The branch then lifts the whole layer surface (no selection means the canvas), which is a legitimate request - "copy the active layer into the destination by the selection path" - and the result is a single-layer copy, not an error.

#### 10. `crates/engine/src/cmd/layer_copy_cmds.rs` — hunk -221,6 +224,126

**Original code (upstream `main`)**

```rust
    }
    let (from, to) = (&s.documents()[src], &s.documents()[dest]);
    let (sdoc, ddoc) = (from.doc.clone(), to.doc.clone());
    let ids: Vec<LayerId> = match p.get("layers") {
        None => from.selected_layers(),
        Some(v) => v
```

**Changed code (current `main`)**

```rust
    }
    let (from, to) = (&s.documents()[src], &s.documents()[dest]);
    let (sdoc, ddoc) = (from.doc.clone(), to.doc.clone());
    if let Some(job) = s.job_on(ddoc.id) {
        return Err(EngineError::Disabled(CMD.into(), format!("“{}” is still running on this document; wait for it to finish or cancel it (Esc)", job.label)));
    }

    if p.get("selection").and_then(Value::as_bool).unwrap_or(false) {
        let cut = p.get("cut").and_then(Value::as_bool).unwrap_or(false);
        if cut && let Some(job) = s.job_on(sdoc.id) {
            return Err(EngineError::Disabled(
                CMD.into(),
                format!("“{}” is still running on this document; wait for it to finish or cancel it (Esc)", job.label),
            ));
        }
        let id = from.active_layer.ok_or_else(|| bad("no active layer in source document"))?;
        let src_layer = sdoc.layer(id).ok_or(EngineError::NoLayer(id))?;
        let mask_target = crate::channel_cmds::target_of(p) == crate::channel_cmds::Target::Mask
            || from.channel_view.target == crate::channel_cmds::ChannelTarget::LayerMask
            || crate::mask_view_cmds::current(from).is_some_and(|v| !v.filter);
        let floating_parts = crate::float_cmds::floating(from).filter(|f| f.layer == id).map(|f| {
            let (piece, bounds) = f.parts.piece();
            (piece.clone(), bounds, f.offset, f.parts.clone())
        });
        let (clip_surface, clip_bounds, float_off, float_cut) = match floating_parts {
            Some((piece, bounds, off, parts)) => (piece, bounds, off, Some(parts)),
            None => {
                let surf = if mask_target {
                    &src_layer.mask.as_ref().ok_or_else(|| EngineError::Other("the active layer has no layer mask".into()))?.surface
                } else {
                    src_layer.surface().ok_or_else(|| EngineError::Other("the active layer has no pixels to copy".into()))?
                };
                let clip = crate::cmd::edit_cmds::lift(surf, sdoc.selection.as_ref(), sdoc.bounds())?;
                (clip.surface, clip.bounds, (0, 0), None)
            }
        };
        if clip_bounds.is_empty() {
            return Err(bad("the selected area is empty"));
        }
        for (doc, role) in [(&sdoc, "source"), (&ddoc, "destination")] {
            if !layered(doc.mode) {
                return Err(EngineError::Other(format!("layers can't be copied with a {:?} {role} document", doc.mode)));
            }
        }
        let target_fmt = photocraft_color::PixelFormat::new(sdoc.mode, sdoc.depth, true);
        let clip_surface = if clip_surface.format() == target_fmt { clip_surface } else { clip_surface.convert(target_fmt) };
        let mut copies = Document::new("", sdoc.size, sdoc.mode, sdoc.depth);
        copies.icc_profile = sdoc.icc_profile.clone();
        copies.resolution_dpi = sdoc.resolution_dpi;
        let mut l = Layer::raster(ddoc.next_layer_name("Layer"), target_fmt);
        *crate::pixels_mut(&mut l)? = clip_surface;
        copies.layers.push(l);

        let content = clip_bounds.translate(float_off.0, float_off.1);
        adapt_copies(s, &mut copies, &ddoc)?;
        let (dx, dy) = if p.get("center").and_then(Value::as_bool).unwrap_or(false) {
            let (c, d) = (center(content), center(ddoc.bounds()));
            (d[0] - c[0], d[1] - c[1])
        } else if let Some(at) = point(p, "at")? {
            let c = center(content);
            (at[0] - c[0], at[1] - c[1])
        } else {
            point(p, "offset")?.map_or((0.0, 0.0), |o| (o[0], o[1]))
        };
        let (dx, dy) = (dx.round() as i32 + float_off.0, dy.round() as i32 + float_off.1);
        translate_copies(&mut copies, &ddoc, dx, dy);

        let prev_active = s.active_index();
        let pre_cut = if cut {
            sdoc.scratch_health().map_err(EngineError::Other)?;
            ddoc.scratch_health().map_err(EngineError::Other)?;
            let pre_cut = s.documents()[src].clone();
            let bg = s.tools.background;
            s.set_active(src);
            s.edit("Cut Pixels", |doc, _| {
                if let Some(parts) = &float_cut {
                    parts.place_rest(doc)?;
                } else if mask_target {
                    crate::cmd::edit_cmds::clear_mask(doc, id, bg, &json!({"target": "mask"}))?;
                } else {
                    crate::cmd::edit_cmds::clear_selected(doc, id, bg)?;
                }
                doc.selection = None;
                Ok(())
            })?;
            Some(pre_cut)
        } else {
            None
        };

        s.set_active(dest);
        let edited = match s.edit("Duplicate Layer", |doc, active| {
            let mut above = *active;
            let mut new = Vec::with_capacity(copies.layers.len());
            let mut place = crate::layer_place::Place::AutoEditable;
            for l in copies.layers {
                let id = crate::layer_place::insert_new(doc, above, l, place);
                new.push(id);
                above = Some(id);
                place = crate::layer_place::Place::Above;
            }
            *active = above;
            Ok(new)
        }) {
            Ok(edited) => edited,
            Err(e) => {
                if let Some(pre_cut) = pre_cut
                    && let Some(slot) = s.docs.get_mut(src)
                {
                    *slot = pre_cut;
                }
                if let Some(prev) = prev_active {
                    s.set_active(prev);
                }
                return Err(e);
            }
        };
        return Ok(json!({
            "layers": edited.into_iter().map(|id| id.0).collect::<Vec<_>>(),
            "offset": [dx, dy],
        }));
    }

    let ids: Vec<LayerId> = match p.get("layers") {
        None => from.selected_layers(),
        Some(v) => v
```

**What the original code does, line by line.**

The visible context is the tail of the source/destination resolution and the start of the old body. `let (from, to) = (&s.documents()[src], &s.documents()[dest]);` takes two immutable borrows of the session's documents; `let (sdoc, ddoc) = (from.doc.clone(), to.doc.clone());` clones both documents. The clones are deliberate: the rest of the function needs to read the source's pixels while it mutates the session, and the borrow checker will not allow a borrow of `s.documents()` to live across `s.edit`. The clone is cheap because documents are tile-shared.

`let ids: Vec<LayerId> = match p.get("layers") { None => from.selected_layers(), Some(v) => v` is the old whole-layer path: the layers named by the request, or the source document's selected layers when the field is absent. Everything below that line in the original is the whole-layer copy: gather ids, clone the documents' layer lists into a scratch document, adapt the copies to the destination's resolution and colour mode, translate them by the offset, then insert them above the active layer.

**What the change does.**

It inserts 120 lines between `let (sdoc, ddoc) = ...` and `let ids: Vec<LayerId> = ...`. The insertion is one outer `if`, whose condition is the new `selection` flag; when the flag is false the function falls through to the old whole-layer code unchanged. Inside the branch:

1. `if let Some(job) = s.job_on(ddoc.id)` rejects the copy when the destination has a running job, with a message naming the job and telling the user to wait or press Esc. The early destination check is new for every caller, not only the selection branch.
2. `let cut = p.get("cut").and_then(Value::as_bool).unwrap_or(false);` reads the cut flag, defaulting to false.
3. `if cut && let Some(job) = s.job_on(sdoc.id)` rejects a cut when the *source* has a running job. A cut edits the source, so the same rule that protects the destination must protect the source; a copy does not edit the source and needs no such check.
4. `let id = from.active_layer.ok_or_else(|| bad("no active layer in source document"))?;` resolves the layer the selection belongs to. The command operates on the active layer by design: a document selection is a property of the document, but the pixels it selects are read from one layer.
5. `let src_layer = sdoc.layer(id).ok_or(EngineError::NoLayer(id))?;` fetches it, erroring if the id is stale.
6. The `mask_target` expression is a three-way OR. `crate::channel_cmds::target_of(p) == Target::Mask` honours an explicit `"target": "mask"` in the request. `from.channel_view.target == ChannelTarget::LayerMask` honours the channel panel: when the user is looking at a layer mask, a copy takes the mask. `crate::mask_view_cmds::current(from).is_some_and(|v| !v.filter)` honours the mask view, but only when it is not a filter mask: a filter mask is a different object with different semantics, and copying one as a layer mask would produce a mask that is not the one on screen.
7. `let floating_parts = crate::float_cmds::floating(from).filter(|f| f.layer == id).map(|f| { ... });` asks whether the active layer has a floating piece. The filter is important: a float belonging to a *different* layer must not be used, because the user's selection may be on this layer while another layer is floating. The map clones the piece surface, its bounds, the float's offset and the whole `CutParts`. The clone of the parts is what later allows `place_rest` on a cut; the clone of the surface is what allows the copy to proceed while the float stays live.
8. The `match floating_parts` produces a four-tuple `(clip_surface, clip_bounds, float_off, float_cut)`. With a float, the piece is used as-is and `float_cut` carries the parts, so a cut can write the remainder back. Without one, the surface is chosen by `mask_target` (mask surface or layer surface, each with its own error message), `lift` extracts the selection, and the offset is `(0, 0)`.
9. `if clip_bounds.is_empty()` rejects an empty selection with "the selected area is empty" rather than inserting an empty layer.
10. The `for (doc, role) in [(&sdoc, "source"), (&ddoc, "destination")]` loop rejects either document when it is not layered, naming which one. The old whole-layer path had the same rule, but a selection copy needs it earlier, because a non-layered document has no layer to receive the copy.
11. `let target_fmt = photocraft_color::PixelFormat::new(sdoc.mode, sdoc.depth, true);` builds the destination format from the *source's* mode and depth with alpha on, matching what the old path did for whole layers.
12. `let clip_surface = if clip_surface.format() == target_fmt { clip_surface } else { clip_surface.convert(target_fmt) };` converts only when needed; a same-format copy stays tile-shared with the source, which is the cheap case.
13. A scratch `Document` is created with the source's size, mode and depth, and the source's ICC profile and DPI are copied onto it. That is what makes the copy land at the same physical size in a document with a different resolution: the adaptation step downstream uses these fields.
14. A raster layer named by `ddoc.next_layer_name("Layer")` is created in the target format and its surface is set to the clip surface.
15. `let content = clip_bounds.translate(float_off.0, float_off.1);` shifts the bounds by the float's own offset, so a floating piece that the user has already moved in the source lands where they see it.
16. `adapt_copies(s, &mut copies, &ddoc)?;` applies the destination's resolution and colour adaptation, the same function the whole-layer path uses.
17. The offset is then chosen: `center` computes the delta that centres the content in the destination; `at` centres it on a document point; `offset` translates it by a relative amount; the default is `(0, 0)`.
18. `let (dx, dy) = (dx.round() as i32 + float_off.0, dy.round() as i32 + float_off.1);` rounds to whole pixels and adds the float's offset once more, because the placement numbers are absolute deltas from the content's original position.
19. `let prev_active = s.active_index();` remembers what to restore on failure.
20. `let pre_cut = if cut { ... } else { None };` is the cut block. It first checks both documents' scratch health (the engine's pre-flight for temporary space), then clones the source document as `pre_cut`, switches the session to the source, and runs `s.edit("Cut Pixels", |doc, _| { ... })`. Inside the edit it calls `parts.place_rest(doc)` for a float, `clear_mask` for a mask target, or `clear_selected` for pixels, then sets `doc.selection = None` so the cut leaves no selection behind.
21. `s.set_active(dest); let edited = match s.edit("Duplicate Layer", |doc, active| { ... })` switches to the destination and inserts the copies. The closure walks the copies, inserting each above the previous one with `layer_place::insert_new`, so a multi-layer copy keeps its order rather than being reversed.
22. The `Err(e)` arm restores `pre_cut` into the source slot and restores the previous active document, then returns the error. That is the rollback: a failed destination edit leaves the source's pixels where they were, so a cut that could not be delivered does not destroy the content.
23. The success return is `Ok(json!({"layers": edited.into_iter().map(|id| id.0).collect::<Vec<_>>(), "offset": [dx, dy]}))` - the new layer ids and the applied offset.

**Why this shape, and the alternatives rejected.**

The branch is written as an early, self-contained block rather than as a function of its own because it needs almost every local the function already has (`s`, `p`, `from`, `to`, `src`, `dest`, `sdoc`, `ddoc`) and it must fall through to the old code when the flag is absent. Extracting it into `copy_selection(...)` would require passing ten values and returning a `Result<Value>`, which buys nothing but a longer signature.

The job checks are in two places rather than one because the two documents play different roles. The destination is written by both paths, so its check is outside the branch. The source is written only by a cut, so its check is inside, guarded by `cut`. A single check on both documents would refuse a copy when the *source* was busy doing something else, which is a legitimate situation (a background job that does not touch the pixels being copied).

The mask decision uses three sources because each answers a different question. The explicit parameter is the automation contract. The channel view is what the user is looking at. The mask view is what the user is editing. Reading only the parameter would make a mask copy from the mask panel copy pixels; reading only the views would make automation unable to ask for a mask. The filter-mask exclusion exists because a filter mask is regenerated from the filter's parameters, not stored as pixels; copying one would produce a static mask that no longer follows its filter.

Using `lift` rather than a new extraction keeps Copy and the transfer on the same code path. The `mask_target` case passes the mask surface as `src`, which works because `lift` only needs a `&Surface` and a selection - it never asks which kind of surface it is reading.

The float is read through `floating(from)` rather than through a stored pointer because a float can be cancelled between frames; asking the session is the only correct source of truth. The `filter(|f| f.layer == id)` guard prevents the case where a float on layer A is accidentally copied when the user's active layer is B.

The cut is written as one `s.edit("Cut Pixels", ...)` that both clears the source and clears the selection, then a second `s.edit("Duplicate Layer", ...)` on the destination. Two edits, not one, because the undo journal is per document; a single edit cannot span two sessions. The consequence is that undo returns the two documents in two steps, which is what the existing whole-layer copy does as well.

The rollback restores `pre_cut` rather than re-running the inverse clear, because a clear is lossy: the original pixels include everything under the selection, and re-running the inverse would have to reconstruct them from the copied layer, which is not exact when the destination's colour mode differs. The snapshot is the only exact undo.

**Effect and blast radius.**

The whole-layer path is textually untouched and behaviourally identical, except for the new destination job check, which is a deliberate refusal that was previously missing: copying into a document with a running job could race with that job's writes.

The selection path adds one new command behaviour, one new error class ("the selected area is empty"), and two new failure messages that name the document role.

The interaction with the float is the part that needs care. `keeps_floating` keeps the float alive into the command; the command clones the parts; a cut writes the remainder back; a copy leaves the float untouched. The test `copies_selected_pixels_to_destination_document_with_copy_and_cut` asserts the float still has its offset after a copy, which pins the "copy does not consume the float" rule.

Failure modes, in order of likelihood: a bad index (rejected before any edit), a destination job (rejected before any edit), an empty selection (rejected before the scratch document is built), a non-layered document (rejected before any pixel is touched), and a destination edit failure (rolled back). In every case the source is either untouched or restored.

#### 11. `crates/engine/src/cmd/layer_copy_cmds.rs` — hunk -399,7 +522,7

**Original code (upstream `main`)**

```rust
        label: "Copy Layers to Document",
        menu: &[],
        shortcut: None,
        params: r##"{"document":index (destination),"layers":[id,…]?=the source's selected layers,"source":index?=active,"center":bool=false (centre on the destination's canvas),"at":[x,y]? (centre the copies on this point),"offset":[dx,dy]?=[0,0] (from their place in the source),"name":str? (one layer: the copy's name)}"##,
        enabled,
        run: copy_to_document,
        journal: true,
```

**Changed code (current `main`)**

```rust
        label: "Copy Layers to Document",
        menu: &[],
        shortcut: None,
        params: r##"{"document":index (destination),"layers":[id,…]?=the source's selected layers,"source":index?=active,"selection":bool=false (copy the active layer's selected pixels instead of whole layers),"cut":bool=false (when selection is true: clear the selected pixels in the source),"center":bool=false (centre on the destination's canvas),"at":[x,y]? (centre the copies on this point),"offset":[dx,dy]?=[0,0] (from their place in the source),"name":str? (one layer: the copy's name)}"##,
        enabled,
        run: copy_to_document,
        journal: true,
```

**What the original code does, line by line.**

`label: "Copy Layers to Document"` is the human-readable command name shown in menus and the history. `menu: &[]` says the command has no menu entry of its own. `shortcut: None` says it has no default key. `params:` is a raw string that documents the JSON contract: `{"document":index (destination),"layers":[id,…]?=the source's selected layers,"source":index?=active,"center":bool=false (centre on the destination's canvas),"at":[x,y]? (centre the copies on this point),"offset":[dx,dy]?=[0,0] (from their place in the source),"name":str? (one layer: the copy's name)}`.

The string is not decoration. It is the schema the control protocol exposes to automation: an agent or a script reads the parameter description instead of the Rust source. `enabled` is the predicate that greys the command out when it cannot run. `run: copy_to_document` binds the spec to the implementation. `journal: true` says the command is undoable, so the dispatcher wraps it in a journal entry.

**What the change does.**

Two fields are added to the raw string, in the middle, between `"source"` and `"center"`: `"selection":bool=false (copy the active layer's selected pixels instead of whole layers)` and `"cut":bool=false (when selection is true: clear the selected pixels in the source)`. The text says both the default and the relationship between the two flags, which is the part a reader would otherwise guess wrong: `cut` alone does nothing.

**Why this shape, and the alternatives rejected.**

The parameter string is the only place where the command's contract is written in prose. Adding the fields without adding them here would leave two documented parameters that no automation author could discover; the schema is what makes the feature usable from outside the UI.

The phrasing repeats `bool=false` because every boolean in this schema is written that way, so a script generator can parse it uniformly. The parenthetical for `cut` is longer than the other parentheticals because `cut` is conditional: writing only "clear the selected pixels in the source" would suggest it applies to whole-layer copies too, which it does not.

The alternative of documenting the fields in a comment above the function was rejected because the comment is invisible to the protocol; the string is parsed, the comment is not.

**Effect and blast radius.**

Purely documentary at runtime: the spec table is read by the protocol layer and by the command palette. No code reads `params` to validate input, so adding fields cannot break an old caller. The defaults stated in the string match the implementation (`unwrap_or(false)` in both places), which the review of the hunk 5 and hunk 6 bodies confirms; a mismatch between the string and the code is the kind of documentation drift this hunk could introduce, so the two must be read together.

The params string is documentation *and* validation: the command table's `params` field is the schema the dispatcher checks a call against, and `r##"..."##` is used so the JSON's own quotes need no escaping. Editing it is therefore not cosmetic - a field that the code accepts but the schema does not mention will be rejected by a caller who validates against the documented shape, and a field the schema mentions but the code ignores would be silently dropped.

The two added keys sit between `"source"` and `"center"`, which is the order the code reads them in and the order a caller would think of them: which document, which content, then where it lands. `"selection":bool=false` says the default is whole layers, so a caller that sends neither key gets the old behaviour - the compatibility rule the whole command follows. `"cut":bool=false` is scoped in its own words ("when selection is true"), because a `cut` on a layer copy has no meaning and the engine does not read it in that branch.

`"layers":[id,…]?=the source's selected layers` keeps its `?=` form, which the other entries use to mean "optional, with this default". `"selection"` is written with the same `?=` marker even though it is a bool rather than a list, because the marker means "optional" generically in this schema, not "a list". A reader comparing the two entries should not read a type difference into the punctuation.

Note what the schema does not say: it does not describe the `target` key that `drop` adds for a mask. That key is documented on the command's own handler (where the mask-vs-pixels decision is made) rather than in this summary, which is the file's convention - the params string lists the shape a caller must know to call the command at all, and the handler's doc comment carries the semantics of each field. A reviewer looking for the mask flag should read `copy_to_document`, not this line.

### `crates/engine/src/cmd/layer_copy_cmds/tests.rs`

#### 12. `crates/engine/src/cmd/layer_copy_cmds/tests.rs` — hunk -663,3 +663,71

**Original code (upstream `main`)**

```rust
    s.execute("type.characterStyle.clearOverride", json!({"layer": id.0})).unwrap();
    assert!(close(text(&s, i, id).runs[0].style.size_pt, 30.0), "{}", text(&s, i, id).runs[0].style.size_pt);
}
```

**Changed code (current `main`)**

```rust
    s.execute("type.characterStyle.clearOverride", json!({"layer": id.0})).unwrap();
    assert!(close(text(&s, i, id).runs[0].style.size_pt, 30.0), "{}", text(&s, i, id).runs[0].style.size_pt);
}

#[test]
/// A selection copied to another document lands at the offset and the source keeps its pixels; cutting clears them.
fn copies_selected_pixels_to_destination_document_with_copy_and_cut() {
    let (mut s, paint) = two_docs(40, 30, 8);
    // Select half of the 10x10 red square (x: 4..9, y: 6..16)
    s.execute("select.rect", json!({"x": 4, "y": 6, "width": 5, "height": 10})).unwrap();

    // 1. Copy without cut (cut: false)
    let r = s.execute("layer.copyToDocument", json!({"document": 1, "selection": true, "cut": false, "offset": [10, 5]})).unwrap();
    let dest_id = copied(&r);
    assert_eq!(s.active_index(), Some(1));
    let d = doc(&s, 1);
    assert_eq!(d.layers.len(), 2);
    assert_eq!(bounds(&s, 1, dest_id), Rect::new(14, 11, 19, 21)); // moved by offset (10, 5)

    // Source is intact
    s.set_active(0);
    assert_eq!(bounds(&s, 0, paint), Rect::new(4, 6, 14, 16));

    // 2. Cut to destination (cut: true)
    let r2 = s.execute("layer.copyToDocument", json!({"document": 1, "selection": true, "cut": true, "offset": [20, 0]})).unwrap();
    let dest_id2 = copied(&r2);
    assert_eq!(s.active_index(), Some(1));
    assert_eq!(doc(&s, 1).layers.len(), 3);
    assert_eq!(bounds(&s, 1, dest_id2), Rect::new(24, 6, 29, 16));

    // Source layer now has the selected half cut out (remaining half: 9..14)
    assert_eq!(bounds(&s, 0, paint), Rect::new(9, 6, 14, 16));

    // 3. A moved floating selection in the source preserves its floating state and offset when copied
    s.set_active(0);
    s.execute("select.rect", json!({"x": 9, "y": 6, "width": 5, "height": 10})).unwrap();
    s.execute("select.float", json!({"dx": 6, "dy": 4})).unwrap();
    let r3 = s.execute("layer.copyToDocument", json!({"document": 1, "selection": true, "cut": false, "offset": [2, 3]})).unwrap();
    let dest_id3 = copied(&r3);
    assert_eq!(bounds(&s, 1, dest_id3), Rect::new(17, 13, 22, 23));
    assert_eq!(s.documents()[0].floating.as_ref().map(|f| f.offset), Some((6, 4)));
}

#[test]
/// A mask selection copied or cut to another document carries the mask, not the layer's pixels.
fn copying_or_cutting_a_mask_selection_transfers_the_mask_without_touching_layer_pixels() {
    let (mut s, paint) = two_docs(40, 30, 8);
    s.execute("layer.layerMask.revealAll", json!({})).unwrap();
    s.execute("select.rect", json!({"x": 4, "y": 6, "width": 5, "height": 10})).unwrap();
    s.execute("edit.fill", json!({"color": "#808080", "target": "mask"})).unwrap();

    // Unfloated mask target (as after arm_move_drag clears an unmoved float): copies mask gray, not red layer pixels.
    let r1 = s.execute("layer.copyToDocument", json!({"document": 1, "selection": true, "cut": false, "target": "mask"})).unwrap();
    let dest_id1 = copied(&r1);
    let mut px1 = [0.0f32; 4];
    doc(&s, 1).layer(dest_id1).unwrap().surface().unwrap().read_pixel(5, 8, &mut px1);
    assert!((px1[0] - px1[1]).abs() < 0.02 && (px1[0] - 128.0 / 255.0).abs() < 0.02, "copied mask gray, not red: {px1:?}");

    // Moved floating mask selection: copies moved mask piece and with cut: true leaves red layer pixels untouched.
    s.set_active(0);
    s.execute("select.float", json!({"dx": 6, "dy": 4, "target": "mask"})).unwrap();
    let r2 = s.execute("layer.copyToDocument", json!({"document": 1, "selection": true, "cut": true, "offset": [2, 3]})).unwrap();
    let dest_id2 = copied(&r2);
    assert_eq!(bounds(&s, 1, dest_id2), Rect::new(12, 13, 17, 23));
    let mut px2 = [0.0f32; 4];
    doc(&s, 1).layer(dest_id2).unwrap().surface().unwrap().read_pixel(13, 15, &mut px2);
    assert!((px2[0] - px2[1]).abs() < 0.02 && (px2[0] - 128.0 / 255.0).abs() < 0.02, "copied floating mask gray: {px2:?}");
    let mut src_px = [0.0f32; 4];
    doc(&s, 0).layer(paint).unwrap().surface().unwrap().read_pixel(5, 8, &mut src_px);
    assert_eq!(src_px, [1.0, 0.0, 0.0, 1.0], "source layer image pixels were not cleared or altered");
}
```

**What the original code does, line by line.**

The visible context is the end of the previous test: `s.execute("type.characterStyle.clearOverride", json!({"layer": id.0})).unwrap();` clears a character style override, `assert!(close(text(&s, i, id).runs[0].style.size_pt, 30.0), ...)` checks the size came back, and the closing brace ends that test. That is the end of the file's existing coverage of `layer.copyToDocument`: everything above it exercises whole-layer copies.

**What the change does.**

Two tests are appended.

`copies_selected_pixels_to_destination_document_with_copy_and_cut` builds two documents with `two_docs(40, 30, 8)` - a 40 x 30 canvas at depth 8 - selects a rectangle `x: 4, y: 6, width: 5, height: 10` inside the red square, and then:

1. Copies with `{"selection": true, "cut": false, "offset": [10, 5]}`. It asserts the session switched to the destination, the destination has two layers (the original plus the copy), and the copied layer's bounds are `Rect::new(14, 11, 19, 21)` - the selection moved by the offset.
2. Switches back to the source and asserts the paint layer's bounds are still `Rect::new(4, 6, 14, 16)`, i.e. the copy did not cut.
3. Copies again with `{"selection": true, "cut": true, "offset": [20, 0]}`. Asserts the destination now has three layers and the new bounds are `Rect::new(24, 6, 29, 16)`.
4. Asserts the source paint layer's bounds are now `Rect::new(9, 6, 14, 16)`: the selected half (x 4..9) is gone and the remaining half (x 9..14) is left, which is exactly a cut of the selection.
5. Creates a *floating* selection in the source (`select.rect` then `select.float {"dx": 6, "dy": 4}`) and copies it with `{"selection": true, "cut": false, "offset": [2, 3]}`. Asserts the destination layer's bounds are `Rect::new(17, 13, 22, 23)` and, crucially, that the source's floating piece still has offset `(6, 4)`.

`copying_or_cutting_a_mask_selection_transfers_the_mask_without_touching_layer_pixels` reveals the layer mask, selects the same rectangle, fills the selection with 50 % grey in the mask, and copies with `{"selection": true, "cut": false, "target": "mask"}`. It reads a pixel of the copied layer and asserts the value is the mask grey rather than the red layer pixel, which is the observable difference between copying the mask and copying the pixels.

**Why this shape, and the alternatives rejected.**

The tests are written against the command, not against the UI, because the command is where the contract lives: the UI's job is to build the JSON. Testing through the command means a failure points at the pixel logic rather than at a gesture.

The bounds assertions are chosen over pixel comparisons for the first test because bounds catch the two failures that matter - the copy landing at the wrong offset, and the cut removing the wrong region - and they are exact integers, so there is no tolerance to tune. The mask test does compare a pixel, because the bounds are identical for a mask copy and a pixel copy; only the content distinguishes them.

The floating case is in the same test rather than a separate one because it is the same promise (a copy leaves the source intact) under a different source state. If it were separate, a change to the float handling could pass the plain test and fail only the float one, which is easier to miss in review.

The cut assertion checks the *remaining* bounds rather than the copied bounds alone: `Rect::new(9, 6, 14, 16)` is the half that was not selected. That is a stronger statement than "something was cleared" - it pins that the clear used the selection's exact rectangle and left the other half untouched.

**Effect and blast radius.**

The tests are additive; no existing test changes. They pin four behaviours the feature depends on: offset placement, source preservation on copy, exact-region clearing on cut, and float preservation on copy. A regression in any of them fails here rather than in the UI tests, which is the earlier and cheaper place to catch it.

The tests also act as documentation of the parameter shape: `{"document", "selection", "cut", "offset", "target"}` appears in executable form, which is what a future reader is more likely to trust than the prose in `specs`.

### `crates/ui-egui/src/app.rs`

#### 24. `crates/ui-egui/src/app.rs` — hunk -52,6 +52,9

**Original code (upstream `main`)**

```rust
            drag: None,
            cancelled_vector_drag: false,
            move_blocked: false,
            tool_override: None,
            live_stroke: None,
            committed_stroke: None,
```

**Changed code (current `main`)**

```rust
            drag: None,
            cancelled_vector_drag: false,
            move_blocked: false,
            move_blocked_start: None,
            move_viewport: None,
            current_viewport: None,
            tool_override: None,
            live_stroke: None,
            committed_stroke: None,
```

**Explanation.** This is the struct literal that builds `PhotocraftApp` at start-up. Rust requires every field of a struct literal to be named, so adding a field to the struct makes this block a compile error until the new field is initialised here; the three lines are not a design choice, they are the compiler holding the constructor to the type.

`move_blocked_start: None` matches `pub(crate) move_blocked_start: Option<[f64; 2]>`. It sits directly after `move_blocked: false` because the two are read together: `move_blocked` says a Move press landed on something that cannot move, and `move_blocked_start` says where that press was. Before this change only the flag existed, so a blocked press recorded that it happened but not where - and without the point there is nothing for `current_move_drag` to promote into `MoveDrag::Layers(b.start)` later. `None` is correct at start-up because no press has happened yet.

`move_viewport: None` matches `Option<egui::ViewportId>`. It records which window owns the gesture in progress. Three places read it: `layer_transfer::finish` (to decide whether this viewport may clear the transfer), `layer_transfer::check_arranged_drag` (to decide whether this viewport may arm one), and `canvas/tool_input.rs` (`handle_move_blocked_event`, to decide whether this viewport may resolve a blocked drag). `None` at start-up means "no gesture", which is why the guards are written as `is_some_and(|v| v != ctx.viewport_id())` rather than a bare inequality: `None` must not be treated as "some other viewport".

`current_viewport: None` matches `Option<egui::ViewportId>` as well, but it is not gesture state; it is the frame's identity. `canvas_view` writes it at the top of every frame (the next hunk), and the two gesture paths copy it into their own fields at the moment they need to remember an owner. It starts as `None` because the first frame has not run yet.

The ordering in the literal - `move_blocked`, `move_blocked_start`, `move_viewport`, `current_viewport`, then `tool_override` - groups the gesture fields together and keeps them next to the flag they extend. That is a readability choice only; field order in a literal has no runtime meaning.

What breaks if any of the three is missed: not a silent bug, a compile error at this literal. That is worth stating because it explains why the change is three lines here rather than logic elsewhere: the type system already forces the initialisation, so the only question the reviewer needs to answer is whether `None` is the right starting value, and for all three it is.

`PhotocraftApp::new` (or the `Default`-shaped constructor around this literal) lists every field, so adding three fields is three lines here and a compile error if forgotten - which is the desired direction, because a `..Default::default()` in this position would hide the omission.

The three are initialised to `None`, which is the correct resting state for all of them: no refused press in progress, no gesture owner, no canvas drawn yet. `move_blocked` above is `false` for the same reason, and the four lines now read as one group - "nothing about the Move gesture is in flight".

`current_viewport: None` in particular matters at startup: the first frames run before any canvas draws, and `finish` is called from `canvas/tabs.rs` on every frame including those. With `None` the comparison in `finish` (`Some(owner) == Some(here)`) cannot match, so an early frame cannot complete a transfer that does not exist. Initialising it to some placeholder id would be worse than `None` because `ViewportId` has no meaningful "root" value that is guaranteed distinct from a real window.

### `crates/ui-egui/src/canvas.rs`

#### 25. `crates/ui-egui/src/canvas.rs` — hunk -1932,6 +1932,7

**Original code (upstream `main`)**

```rust

pub fn canvas_view(app: &mut PhotocraftApp, ui: &mut egui::Ui, idx: usize, rect: Rect, mut view: View, primary: bool) -> View {
    let ctx = ui.ctx().clone();
    // The Frame tool starts transforms from pointer events, which carry no context; the options
    // bar that also records it can be hidden.
    if primary {
```

**Changed code (current `main`)**

```rust

pub fn canvas_view(app: &mut PhotocraftApp, ui: &mut egui::Ui, idx: usize, rect: Rect, mut view: View, primary: bool) -> View {
    let ctx = ui.ctx().clone();
    app.current_viewport = Some(ctx.viewport_id());
    // The Frame tool starts transforms from pointer events, which carry no context; the options
    // bar that also records it can be hidden.
    if primary {
```

**Explanation.** `canvas_view` is the function that draws one document's canvas into a rectangle; it runs once per document view per frame, inside whatever viewport is being drawn. The added line, `app.current_viewport = Some(ctx.viewport_id());`, is the first statement after `let ctx = ui.ctx().clone();`, so every later line in the function - the pointer handling, the tool events, the marquee - runs with `app.current_viewport` already naming the viewport it belongs to.

`ctx.viewport_id()` is the id of the viewport currently being painted. In a single-window session it is the root viewport; in a session with detached document windows it is that window's id. Assigning it to the app means the app carries, for the duration of the frame, the answer to "which window is this code running in" - something the event types do not carry: `ToolEvent::Down { x, y, pressure }` has no viewport, and the pointer events that reach the canvas have no viewport either.

Two consumers depend on it. `move_box::begin_with_layer` copies it into `app.move_viewport` when a marquee starts; `canvas/tool_input.rs` copies it into `app.move_viewport` when a blocked Move press is recorded. Both are the only moments a gesture acquires an owner. The comparison in `finish` and in `check_arranged_drag` is against `ctx.viewport_id()` directly, so those two do not read `app.current_viewport` - but they do depend on the field having been written in the frame that produced the gesture, because the arming paths read it.

Why here rather than in `tool_event`: `tool_event` is shared by mouse input and automation, and automation has no viewport. Setting the field there would either need a parameter that callers must invent or would leave the field stale for automation-driven events. `canvas_view` is the one place that runs once per viewport per frame and always has a real `Ui` with a real context.

Why the top of the function rather than before each use: the value must be correct for the whole frame, not only at the moment a gesture starts. A marquee that begins on a press and continues on a move is handled in different calls; if the field were refreshed only at the press, a later move in another viewport could see the press's value. Refreshing at the top makes every read in this frame see this viewport.

The edge case: a frame in which no canvas is drawn (a dialog is open, no document view is visible). `current_viewport` keeps its previous value, which is harmless because no gesture can start without a canvas to press on. It is not `None`-ed on purpose: clearing it would make a gesture that began in the previous frame look unowned.

`canvas_view` is called once per document view per frame - in the main window for the one visible document, in a detached window for that window's document, and once per tile in arranged view. Each call therefore re-records `current_viewport`, and the value is always the viewport whose frame is currently running. That is what makes the field a reliable answer to "which window is drawing" for every event handler that runs later in the same frame.

`Some(ctx.viewport_id())` rather than `Some(ui.ctx().viewport_id())` is not a difference: `ctx` is `ui.ctx().clone()` on the line above, cloned because the rest of the function passes it to helpers that need an owned context. Recording from the clone is the same id.

Placement immediately after the `let ctx` line and before the `if primary` block is deliberate: `primary` distinguishes the main canvas from a secondary one, and a detached window's canvas is `primary` in its own viewport, so the recording must not be inside either branch. Putting it in the `if primary` arm would leave `current_viewport` stale in a secondary view, and a press there would record the wrong owner.

No condition guards it, not even "only when the Move tool is active". The field is read by `move_box` and by the blocked-press branch, both of which only run for the Move tool, and by `finish`, which runs every frame. Recording unconditionally costs one assignment per view per frame and removes a class of bug where the field is stale because the tool changed between the press and the release.

### `crates/ui-egui/src/canvas/tabs.rs`

#### 26. `crates/ui-egui/src/canvas/tabs.rs` — hunk -9,6 +9,7

**Original code (upstream `main`)**

```rust
        return;
    }
    crate::distort_ui::drop_foreign(app);
    documents(app, ui);
    crate::layer_transfer::finish(app, ui.ctx());
    // Once a frame, not per document view: the right-click or options-bar Brush Preset picker.
```

**Changed code (current `main`)**

```rust
        return;
    }
    crate::distort_ui::drop_foreign(app);
    crate::layer_transfer::check_arranged_drag(app, ui.ctx());
    documents(app, ui);
    crate::layer_transfer::finish(app, ui.ctx());
    // Once a frame, not per document view: the right-click or options-bar Brush Preset picker.
```

**Explanation.** `tabs` is the per-frame entry point for the document area. Its body runs, in order: `distort_ui::drop_foreign`, then `documents(app, ui)` (which draws every document view), then `layer_transfer::finish`, then the once-per-frame pickers. The hunk inserts `layer_transfer::check_arranged_drag(app, ui.ctx());` between `drop_foreign` and `documents`.

The position is the whole point. `check_arranged_drag` asks: is a Move drag in progress, and is the pointer now inside an arranged cell that is not the active document? If yes, it clears the drag and arms a transfer owned by this context. That question must be answered **before** `documents` draws, because drawing is what routes the pointer into a document's canvas; if the check ran after, the drag would already have been delivered to the wrong document's canvas for one frame, and the user would see the marquee jump into the active document's coordinate space before the transfer took over.

Running after `drop_foreign` is equally deliberate: `drop_foreign` releases state that belongs to another window when the pointer leaves this one. A drag that has just left a foreign window must be cleaned up before the arranged check asks whether it crossed a cell, otherwise the check would see a drag that is already over.

`finish` stays where it was, after `documents`, because it handles the release: the ghost is drawn while the button is held, and the drop happens on release. Drawing the documents first means the release lands on a laid-out canvas with `last_canvas_rect` already updated for this frame; `target_at` reads that rect to decide whether the pointer is over a document at all.

The calls are cheap: `check_arranged_drag` returns immediately when a transfer is already armed, when `move_viewport` names another window, when there is no move drag, or when there are no arranged cells. In a single-document session with no drag it is four early returns.

What would break without it: in `Window › Arrange`, dragging a selection from one cell to another would stay a normal Move drag. The drag would be delivered to the source document's canvas even while the pointer is over the neighbour, because the canvas that receives pointer events is decided by the layout, not by the pointer position; the transfer would never arm, and the release would move the selection inside the source document instead of copying it.

The ordering of the three calls is the contract. `check_arranged_drag` runs before `documents(app, ui)` because the promotion it performs changes which document is active, and `documents` draws the views from the active document - promoting after the draw would show the old active document for one frame, which reads as a flicker.

`finish` runs after `documents` because it hit-tests the release point against `last_canvas_rect` and the per-tile rects that `documents` just computed. Running it first would test against the previous frame's geometry, and a release that coincided with a resize or a tab change would land in the wrong document. `app.last_canvas_rect` is updated by the draw, so the recomputation order is what makes `target_at` correct.

Both new calls pass `ui.ctx()`, the same context the pre-existing `finish` call uses, so all three see one viewport id per frame. That is what the guard in `finish` compares against.

`crate::layer_transfer::check_arranged_drag` is written with a full path, matching the `finish` call on the next line - the file's style for one-off cross-module calls, and it keeps the two adjacent lines visually parallel so a reader sees they are a pair: one promotes, one resolves.

`distort_ui::drop_foreign(app)` above them is untouched and stays first: it clears a foreign drop from a previous frame before any of the transfer logic reads the drag state.

#### 27. `crates/ui-egui/src/canvas/tabs.rs` — hunk -832,6 +833,7

**Original code (upstream `main`)**

```rust
                // Preferences › Zoom Resizes Windows: a zoom wraps the window around the image.
                crate::zoom_window_fit::follow(app, ui.ctx(), class, vid, &w, &mut view, rect);
            });
        });
        if let Some(win) = app.ui.windows.iter_mut().find(|x| x.id == w.id) {
            win.view = view;
```

**Changed code (current `main`)**

```rust
                // Preferences › Zoom Resizes Windows: a zoom wraps the window around the image.
                crate::zoom_window_fit::follow(app, ui.ctx(), class, vid, &w, &mut view, rect);
            });
            crate::layer_transfer::finish(app, ui.ctx());
        });
        if let Some(win) = app.ui.windows.iter_mut().find(|x| x.id == w.id) {
            win.view = view;
```

**Explanation.** `canvas/tabs.rs` draws the canvas, the tab strip, and - on the detached-window path - a whole `Viewport`'s contents. This hunk adds one call inside the `Viewport::show` closure, after the zoom-follow block and before the closure returns.

The surrounding code is the detached document window's frame: it wraps the canvas draw in `ui.ctx().show_viewport_immediate(...)` (or the equivalent `Viewport` builder used here), calls `crate::zoom_window_fit::follow(app, ui.ctx(), class, vid, &w, &mut view, rect)` inside the closure, and after the closure writes the adjusted view back with `if let Some(win) = app.ui.windows.iter_mut().find(|x| x.id == w.id) { win.view = view; }`.

The added line is `crate::layer_transfer::finish(app, ui.ctx());`, placed inside the closure and *after* the `follow` call. The position matters for two reasons.

First, inside the closure: `finish` needs the context of *this* viewport, not the main one. `ui.ctx()` inside a `show_viewport_immediate` closure is the child viewport's context, and `finish` compares `ctx.viewport_id()` against the recorded `Transfer::viewport`. Calling it outside the closure - with the parent context - would test the wrong window and defeat the guard this pull request adds.

Second, after `follow`: `zoom_window_fit::follow` may change `view` (zoom resizes the window), and `finish` reads `app.last_canvas_rect` through `target_at`. If `finish` ran before the follow, a release that also resized the window would be hit-tested against the previous frame's rect. The order keeps the hit test consistent with what the user saw at release.

Why a detached window needs `finish` at all: the transfer state lives in the shared `egui::Context`, so a drag that started in the main window is visible to the document window's code. When the user releases over the document window, that viewport's frame must be the one that completes the transfer - but before this change only the main window ran `finish`, so a release in a document window did nothing until the pointer moved back. The call here is what makes the document window a valid drop target.

The call is unconditional rather than guarded by "is a transfer armed": `finish` itself starts with `if get(ctx).is_none() && !any_pointer_released(...) { return }`-style early exits, and the guards inside it decide whether this frame ends a transfer. Adding a condition here would duplicate those guards in a second place, and the two could drift.

`crate::layer_transfer::finish` rather than an imported name: the file calls several other cross-module functions with full paths in this region (`crate::zoom_window_fit::follow`), so the local style is explicit paths for one-off calls.

No return value and no interaction with `view`: the function mutates `app` (clearing the transfer and possibly running a command), which is why it can be called from inside a closure that also holds `&mut view` - `view` is a local `View` struct, not a borrow of `app`. If `finish` could drop a transfer into *this* window's document, the write-back of `view` afterwards still applies, and the drop changes the document, not the view.

### `crates/ui-egui/src/canvas/tool_input.rs`

#### 32. `crates/ui-egui/src/canvas/tool_input.rs` — hunk -276,13 +276,15

**Original code (upstream `main`)**

```rust
                    })
                });
                if locked_hit || (probe.as_ref().is_ok_and(|v| v["layer"].is_null()) && crate::move_box::empty_at(app, [x, y])) {
                    if locked_hit {
                        // A click still selects the locked layer, as in Photoshop; a drag's
                        // marquee then replaces that selection on release.
                        let _ = app.run("layer.pickAt", json!({"x": x, "y": y, "target": target, "mode": mode}));
                        note_move_pick(app, active_before);
                    }
                    crate::move_box::begin(app, [x, y], mods.shift);
                    return true;
                }
                let picked = app.run("layer.pickAt", json!({"x": x, "y": y, "target": target, "mode": mode}));
```

**Changed code (current `main`)**

```rust
                    })
                });
                if locked_hit || (probe.as_ref().is_ok_and(|v| v["layer"].is_null()) && crate::move_box::empty_at(app, [x, y])) {
                    let mut hit_id = None;
                    if locked_hit {
                        // A click still selects the locked layer, as in Photoshop; a drag's
                        // marquee then replaces that selection on release.
                        let r = app.run("layer.pickAt", json!({"x": x, "y": y, "target": target, "mode": mode}));
                        hit_id = r.ok().and_then(|v| v["layer"].as_u64()).map(photocraft_doc::LayerId);
                        note_move_pick(app, active_before);
                    }
                    crate::move_box::begin_with_layer(app, [x, y], mods.shift, hit_id);
                    return true;
                }
                let picked = app.run("layer.pickAt", json!({"x": x, "y": y, "target": target, "mode": mode}));
```

**Explanation.** This is the Move-press path in `tool_input`, in the branch that runs when the press landed on a locked layer (`locked_hit`) or on empty canvas over a layer that cannot be moved. The original body did two things: it ran `layer.pickAt` with `let _ = ...` (discarding the result, because only the side effect of selecting the locked layer mattered) and then called `move_box::begin(app, [x, y], mods.shift)` with no knowledge of which layer was under the pointer.

The change keeps the pick's result and passes the layer id through. `let mut hit_id = None;` declares the slot. `let r = app.run("layer.pickAt", ...)` keeps the `Result` instead of dropping it. `hit_id = r.ok().and_then(|v| v["layer"].as_u64()).map(photocraft_doc::LayerId);` extracts the id: `ok()` discards an error, `and_then` reads the `layer` field, `as_u64` requires it to be an integer (the command returns `null` when the pick found nothing), and `map` wraps it in the newtype. The three fallbacks therefore all produce `None` rather than a wrong id: a failed command, a null field, or a non-integer field. `crate::move_box::begin_with_layer(app, [x, y], mods.shift, hit_id)` then stores the id on the `LayerBox`.

Why the id is needed at all: the blocked press deliberately does not select the layer for dragging - the layer cannot move - but a cross-document transfer is not a move. The user pressing on a locked layer and dragging to another document expects the layer to be copied there, and that requires knowing which layer they pressed on. `layer.pickAt` already answers that question; the original code threw the answer away because nothing downstream needed it. `current_move_drag` later reads `layer_box.grabbed_layer` to build `MoveDrag::Layers(b.start)`, and `arm_move_drag` uses the id as the layer to transfer.

Why `hit_id` is declared before the `if` rather than inside it: the empty-canvas case enters the same branch but does not run `pickAt` (that case has no layer to pick), so `hit_id` must be `None` there and must still reach `begin_with_layer`. Initialising it outside the `if` gives the empty case the value it needs without a second call site.

The observable difference: before the change, a press on a locked layer started a marquee with no layer attached, so a drag to another document's tab or cell had nothing to promote and the drop did nothing. After the change the marquee starts with the id, `current_move_drag` returns `MoveDrag::Layers`, and the transfer arms.

`hit_id` exists because the layer a refused press landed on has to be carried to the marquee. Before, the branch ran `layer.pickAt` and discarded the result (`let _ = ...`), because the only thing it needed was the side effect of selecting the layer. Now it needs the id as well, so the call's value is kept: `hit_id = r.ok().and_then(|v| v["layer"].as_u64()).map(photocraft_doc::LayerId);`.

The chain has three separable failure modes, all collapsed to `None`. `r.ok()` drops an `Err` (the command failed - for instance the point is outside every layer). `.and_then(|v| v["layer"].as_u64())` handles a successful response whose `layer` field is null (the probe above tests for exactly this) or is not a number. `.map(LayerId)` rebuilds the newtype the rest of the UI uses. Every one of those is "no layer to grab", which is the same outcome as not having tried, so a single `None` is the honest representation.

`let mut hit_id = None;` is declared *before* the `if locked_hit` block rather than inside it, because `begin_with_layer` is called outside that block - the same code path serves both "a locked layer was hit" (id set) and "the canvas is empty here" (id `None`). That is the reason the two conditions are one `if` with a shared tail: `locked_hit || (probe ... is_null() && move_box::empty_at(...))`.

`begin(app, ...)` becomes `begin_with_layer(app, ..., hit_id)`. The old function still exists (hunk 24) and delegates with `None`, so this call site could have stayed `begin` in the empty-canvas case - but the branch serves both cases, and choosing between them would need a second condition. Passing `hit_id` (which is `None` for the empty case anyway) is exact.

`let _ = app.run(...)` became `let r = app.run(...)`: the binding is now used. Naming it `r` rather than `resp` follows the surrounding code, where short names are the norm (`let picked = ...`, `let probe = ...`).

#### 33. `crates/ui-egui/src/canvas/tool_input.rs` — hunk -301,6 +303,8

**Original code (upstream `main`)**

```rust
            // A locked layer: no drag, and Photoshop's message once the pointer moves (`move_lock`).
            if crate::move_lock::blocked(app, tool, [x, y], mods) {
                app.move_blocked = true;
                return true;
            }
            // With a selection: cut the selected pixels (⌥ copies them) and drag them as a floating
```

**Changed code (current `main`)**

```rust
            // A locked layer: no drag, and Photoshop's message once the pointer moves (`move_lock`).
            if crate::move_lock::blocked(app, tool, [x, y], mods) {
                app.move_blocked = true;
                app.move_blocked_start = Some([x, y]);
                app.move_viewport = app.current_viewport;
                return true;
            }
            // With a selection: cut the selected pixels (⌥ copies them) and drag them as a floating
```

**Explanation.** The blocked-press branch in `canvas/tool_input.rs` gains two assignments before its `return true`. That branch is the one that fires when a Move-tool press lands on a locked layer or on the Background: `crate::move_lock::blocked(app, tool, [x, y], mods)` decides, and the arm sets `app.move_blocked = true` then returns.

Two lines are added: `app.move_blocked_start = Some([x, y]);` and `app.move_viewport = app.current_viewport;`.

`move_blocked_start` is new state (hunk 16's field). The point of the press is recorded because a *refused* press produces no `app.drag` - the normal gesture object is never created - so the marquee's `ToolEvent::Down`/`Move`/`Up` sequence never carries the start. Without recording it here, a drag that begins on a locked layer would have no start point, and the promotion in `layer_transfer::current_move_drag` (which reads `move_blocked_start` for its second branch) could not construct a `MoveDrag::Layers(grab)`. The coordinate is `[x, y]`, the same document-space pair the branch was just given, so no conversion is needed.

`move_viewport` records the window, for the same reason `begin_with_layer` records it: the later `finish` compares it against the releasing viewport, and a refused press is a gesture whose release must be judged in the window where it started. Without it, a drag that began on a locked layer in the main window and was released over a document window would be judged by whichever `finish` ran first.

`app.current_viewport` rather than `ctx.viewport_id()`: this function has no `ctx` parameter (its signature is `(app, raw)` - pointer events come from `RawInput`), and `current_viewport` is the field the canvas sets once per draw for exactly this purpose (hunk 16). Reading the field is also correct across the whole event-dispatch pass, whereas a `ctx` read would have to be threaded through.

`return true` follows, unchanged and still last. It means "the press is consumed" - the caller must not continue with the ordinary Move handling, because there is no drag to start. The two added lines are side effects on the way out, not a change to the return contract, so no caller changes.

Why the assignments come *before* the return and not after: there is nothing after; the arm returns immediately. Ordering them after would be unreachable. Ordering them before `app.move_blocked = true` would work identically, but grouping them after keeps the pre-existing line first, so the diff shows one added block rather than a moved line.

The branch is one of six places that write `move_viewport` (hunk 24's `begin_with_layer`, this one, `move_box`'s `Down` and `Up`, `finish`'s cleanup, `arm_move_drag`). Its correctness depends on the other five: a set without a clear leaks, and every clear site in this pull request corresponds to a set site.

#### 34. `crates/ui-egui/src/canvas/tool_input.rs` — hunk -395,6 +399,48

**Original code (upstream `main`)**

```rust
    }
}

/// Tool state machine. Shared by mouse input and automation.
pub fn tool_event(app: &mut PhotocraftApp, ev: ToolEvent, mods: egui::Modifiers) {
    if consumes_cancelled_vector_event(app, ev) {
```

**Changed code (current `main`)**

```rust
    }
}

/// Handles pointer gestures on locked layers or targets where movement is disallowed.
/// When dragging within the same document, shows the lock feedback promptly upon pointer movement
/// (including when returning to the press point), while preserving cross-document drag transfers.
fn handle_move_blocked_event(app: &mut PhotocraftApp, ev: ToolEvent, tool: Tool, mods: egui::Modifiers) -> bool {
    // A locked layer (the Background without a selection, say): no drag, and the lock message
    // once the pointer moves (a click says nothing). ⌥ with a selection tool here (#896); the Move
    // tool below, once Auto-Select has picked the layer.
    match ev {
        ToolEvent::Down { x, y, .. } if tool != Tool::Move && app.ui.transform.is_none() && crate::move_lock::blocked(app, tool, [x, y], mods) => {
            app.move_blocked = true;
            app.move_blocked_start = Some([x, y]);
            app.move_viewport = app.current_viewport;
            true
        }
        ToolEvent::Move { .. } if app.move_blocked => {
            // The window that armed the blocked drag resolves it: a second window's pointer
            // must not clear the state or raise the lock dialog mid-drag.
            if let (Some(owner), Some(here)) = (app.move_viewport, app.current_viewport)
                && owner != here
            {
                return true;
            }
            if !crate::panels::layer_transfer::is_transfer_in_progress(app) {
                app.move_blocked = false;
                app.move_blocked_start = None;
                app.move_viewport = None;
                crate::move_lock::prompt(app);
            }
            true
        }
        ToolEvent::Up { .. } if app.move_blocked => {
            app.move_blocked = false;
            app.move_blocked_start = None;
            app.move_viewport = None;
            true
        }
        _ => false,
    }
}
    }
}

/// Tool state machine. Shared by mouse input and automation.
pub fn tool_event(app: &mut PhotocraftApp, ev: ToolEvent, mods: egui::Modifiers) {
    if consumes_cancelled_vector_event(app, ev) {
```

**Explanation.** The original code ran the blocked-drag state machine inline inside `tool_event`, as a bare `match ev` with three arms, each returning early from `tool_event`. The change extracts exactly that match into `handle_move_blocked_event(app, ev, tool, mods) -> bool` and calls it from `tool_event` at the same point in the sequence (after the eyedropper handler, before `handle_move_pointer`).

The extraction is not cosmetic. `tool_event` is the shared entry point for mouse input and automation; its body is a chain of `if handle_x(...) { return; }` calls. Adding two guards and three field writes to an inline match made that chain longer and, more importantly, made the guards testable only by driving the whole pipeline. As a named function, the three arms and their guards can be read as one rule, and the return value says "this event was consumed" in the same idiom as the neighbouring handlers.

The behaviour changes inside the arms are three. On a blocked `Down` the function now also writes `move_blocked_start` and `move_viewport` (hunk 14 explains why those are needed). On a blocked `Move` it first checks the viewport owner and then checks whether a transfer is in progress. On a blocked `Up` it clears all three fields instead of only the flag.

The `Move` guard is the one that closes the reported defect. `if let (Some(owner), Some(here)) = (app.move_viewport, app.current_viewport) && owner != here { return true; }` compares the gesture's owner with the frame's viewport. Both are `Option`; when either is `None` the comparison is skipped, because "no owner" and "unknown frame" must not be read as "another window". When they differ, the function returns `true` - the event is consumed - without touching the state: the second window's pointer move neither clears the blocked flag nor raises the lock dialog.

The second check, `if !layer_transfer::is_transfer_in_progress(app)`, is what keeps a legitimate cross-document drag alive. Once the drag has been promoted to a transfer, `is_transfer_in_progress` is true, so a pointer move inside the source document no longer clears the blocked state and no longer prompts. Before this, every move cleared the flag and raised the dialog the moment the pointer left the press point, which made it impossible to drag a locked layer anywhere.

The `Up` arm clears the three fields and nothing else: it does not prompt, because a click (press and release without movement) must say nothing, which is the documented rule in the comment above.

What the extraction changed for readers of `tool_event`: the function now has one more line and one fewer embedded state machine, and the ordering of the handlers is visible in one place. That ordering matters: the blocked handler runs before `handle_move_pointer`, so a blocked press never reaches the move-tool pointer logic that would try to move the layer.

The extraction changes the shape of `tool_event` from an inline `match` to a call plus an early return: `if handle_move_blocked_event(app, ev, tool, mods) { return; }`. That is what allows the new logic to have its own doc comment, its own guards and its own early returns without nesting inside `tool_event`'s already-long body.

The function returns `bool` rather than `()` because `tool_event` must stop processing when the blocked branch claims the event. Returning `true` from the `Down` arm and from the guarded `Move` arm, and `false` from the fall-through, is the same contract the inline `return;` had.

The two new guards in the `Move` arm are the substance of the hunk. The first compares `app.move_viewport` with `app.current_viewport` and returns `true` (consumed, but nothing done) when they differ: the window that armed the blocked drag resolves it, and a second window's pointer must not clear the state or raise the lock dialog mid-drag. Without it, moving the pointer over a detached window would pop the lock message in the wrong window and clear `move_blocked`, so the original window's gesture would end up unowned.

The second guard is `if !crate::panels::layer_transfer::is_transfer_in_progress(app)`. Once a blocked drag has been promoted into a cross-document transfer, the transfer is the gesture - it has its own ghost and its own completion path - so the blocked state's message must not fire. The function is called from `panels` (which is why the path is `crate::panels::layer_transfer`), and it reads `app.frame_tool_ctx` because this function has no `ctx` parameter.

The `Up` arm keeps clearing `move_blocked` and is left otherwise as it was: a release ends the blocked gesture regardless of window, because leaving the flag set after a release would show the message on the next move with no press. The asymmetry (window-guarded on `Move`, unconditional on `Up`) is deliberate: the harm the guard prevents exists only while the pointer is moving.

The comment block above the original `match` moved with it into the new function, and the `ToolEvent::Down` arm gained the two assignments documented in hunk 14. A reader comparing old and new sees the same three arms in the same order plus two guards in the middle one - which is the smallest possible diff for what is a behavioural change.

#### 35. `crates/ui-egui/src/canvas/tool_input.rs` — hunk -406,24 +452,8

**Original code (upstream `main`)**

```rust
    if handle_eyedropper_event(app, tool, ev, mods) {
        return;
    }
    // A locked layer (the Background without a selection, say): no drag, and Photoshop's message
    // once the pointer moves (a click says nothing). ⌘ with a selection tool here (#896); the Move
    // tool below, once Auto-Select has picked the layer.
    match ev {
        ToolEvent::Down { x, y, .. } if tool != Tool::Move && app.ui.transform.is_none() && crate::move_lock::blocked(app, tool, [x, y], mods) => {
            app.move_blocked = true;
            return;
        }
        ToolEvent::Move { .. } if app.move_blocked => {
            app.move_blocked = false;
            crate::move_lock::prompt(app);
            return;
        }
        ToolEvent::Up { .. } if app.move_blocked => {
            app.move_blocked = false;
            return;
        }
        _ => {}
    }
    if handle_move_pointer(app, ev, tool, mods) {
        return;
```

**Changed code (current `main`)**

```rust
    if handle_eyedropper_event(app, tool, ev, mods) {
        return;
    }
    if handle_move_blocked_event(app, ev, tool, mods) {
        return;
    }
    if handle_move_pointer(app, ev, tool, mods) {
        return;
```

**Explanation.** The original code ran the blocked-drag state machine inline inside `tool_event`, as a bare `match ev` with three arms, each returning early from `tool_event`. The change extracts exactly that match into `handle_move_blocked_event(app, ev, tool, mods) -> bool` and calls it from `tool_event` at the same point in the sequence (after the eyedropper handler, before `handle_move_pointer`).

The extraction is not cosmetic. `tool_event` is the shared entry point for mouse input and automation; its body is a chain of `if handle_x(...) { return; }` calls. Adding two guards and three field writes to an inline match made that chain longer and, more importantly, made the guards testable only by driving the whole pipeline. As a named function, the three arms and their guards can be read as one rule, and the return value says "this event was consumed" in the same idiom as the neighbouring handlers.

The behaviour changes inside the arms are three. On a blocked `Down` the function now also writes `move_blocked_start` and `move_viewport` (hunk 14 explains why those are needed). On a blocked `Move` it first checks the viewport owner and then checks whether a transfer is in progress. On a blocked `Up` it clears all three fields instead of only the flag.

The `Move` guard is the one that closes the reported defect. `if let (Some(owner), Some(here)) = (app.move_viewport, app.current_viewport) && owner != here { return true; }` compares the gesture's owner with the frame's viewport. Both are `Option`; when either is `None` the comparison is skipped, because "no owner" and "unknown frame" must not be read as "another window". When they differ, the function returns `true` - the event is consumed - without touching the state: the second window's pointer move neither clears the blocked flag nor raises the lock dialog.

The second check, `if !layer_transfer::is_transfer_in_progress(app)`, is what keeps a legitimate cross-document drag alive. Once the drag has been promoted to a transfer, `is_transfer_in_progress` is true, so a pointer move inside the source document no longer clears the blocked state and no longer prompts. Before this, every move cleared the flag and raised the dialog the moment the pointer left the press point, which made it impossible to drag a locked layer anywhere.

The `Up` arm clears the three fields and nothing else: it does not prompt, because a click (press and release without movement) must say nothing, which is the documented rule in the comment above.

What the extraction changed for readers of `tool_event`: the function now has one more line and one fewer embedded state machine, and the ordering of the handlers is visible in one place. That ordering matters: the blocked handler runs before `handle_move_pointer`, so a blocked press never reaches the move-tool pointer logic that would try to move the layer.

The extraction changes the shape of `tool_event` from an inline `match` to a call plus an early return: `if handle_move_blocked_event(app, ev, tool, mods) { return; }`. That is what allows the new logic to have its own doc comment, its own guards and its own early returns without nesting inside `tool_event`'s already-long body.

The function returns `bool` rather than `()` because `tool_event` must stop processing when the blocked branch claims the event. Returning `true` from the `Down` arm and from the guarded `Move` arm, and `false` from the fall-through, is the same contract the inline `return;` had.

The two new guards in the `Move` arm are the substance of the hunk. The first compares `app.move_viewport` with `app.current_viewport` and returns `true` (consumed, but nothing done) when they differ: the window that armed the blocked drag resolves it, and a second window's pointer must not clear the state or raise the lock dialog mid-drag. Without it, moving the pointer over a detached window would pop the lock message in the wrong window and clear `move_blocked`, so the original window's gesture would end up unowned.

The second guard is `if !crate::panels::layer_transfer::is_transfer_in_progress(app)`. Once a blocked drag has been promoted into a cross-document transfer, the transfer is the gesture - it has its own ghost and its own completion path - so the blocked state's message must not fire. The function is called from `panels` (which is why the path is `crate::panels::layer_transfer`), and it reads `app.frame_tool_ctx` because this function has no `ctx` parameter.

The `Up` arm keeps clearing `move_blocked` and is left otherwise as it was: a release ends the blocked gesture regardless of window, because leaving the flag set after a release would show the message on the next move with no press. The asymmetry (window-guarded on `Move`, unconditional on `Up`) is deliberate: the harm the guard prevents exists only while the pointer is moving.

The comment block above the original `match` moved with it into the new function, and the `ToolEvent::Down` arm gained the two assignments documented in hunk 14. A reader comparing old and new sees the same three arms in the same order plus two guards in the middle one - which is the smallest possible diff for what is a behavioural change.

### `crates/ui-egui/src/lib.rs`

#### 37. `crates/ui-egui/src/lib.rs` — hunk -377,6 +377,9

**Original code (upstream `main`)**

```rust
    /// A Move-tool press landed on a locked layer: the first pointer move shows Photoshop's
    /// message (`move_lock`), a plain click shows nothing.
    pub(crate) move_blocked: bool,
    /// The tool pointer events go to this frame when it isn't the selected one: the Move tool
    /// while ⌘ is held (`hold_keys::cmd_moves`). Set by the canvas for its gestures, never saved.
    pub(crate) tool_override: Option<state::Tool>,
```

**Changed code (current `main`)**

```rust
    /// A Move-tool press landed on a locked layer: the first pointer move shows Photoshop's
    /// message (`move_lock`), a plain click shows nothing.
    pub(crate) move_blocked: bool,
    pub(crate) move_blocked_start: Option<[f64; 2]>,
    pub(crate) move_viewport: Option<egui::ViewportId>,
    pub(crate) current_viewport: Option<egui::ViewportId>,
    /// The tool pointer events go to this frame when it isn't the selected one: the Move tool
    /// while ⌘ is held (`hold_keys::cmd_moves`). Set by the canvas for its gestures, never saved.
    pub(crate) tool_override: Option<state::Tool>,
```

**Explanation.** Three fields are added to `PhotocraftApp`, immediately after `move_blocked` and before `tool_override`. They are the app-level state the cross-document gesture needs.

`pub(crate) move_blocked_start: Option<[f64; 2]>` records where a refused press happened. It is `Option` because most of the time there is no refused press; `None` is the resting state and every clear path sets it. The payload is a document-space point, the same representation `LayerBox::start` uses, so the promotion in `current_move_drag` can build `MoveDrag::Layers(start)` without conversion.

`pub(crate) move_viewport: Option<egui::ViewportId>` records the window that owns the in-flight marquee or refused press. `Option` for the same reason: no gesture, no owner. The type is egui's `ViewportId`, which is what `ctx.viewport_id()` returns and what `ctx.input(|i| i.viewport_id)` carries, so the comparison in `finish` is a direct `==`.

`pub(crate) current_viewport: Option<egui::ViewportId>` records the window being drawn *this frame*. It exists because the event handlers have no `egui::Context` parameter: `tool_input.rs` receives `RawInput`, and `move_box`'s event handler receives a `ToolEvent`. Without this field they could not know which window a press happened in, and `move_viewport` could not be set at the moment of the gesture.

The distance between `current_viewport` and `move_viewport` is the whole design: `current_viewport` is "where am I now", written once per canvas draw; `move_viewport` is "where did this gesture start", written when a gesture begins and cleared when it ends. A single field cannot serve both, because a gesture that starts in one window and continues (with the pointer) into another must be judged by where it started, not by where the pointer currently is.

`Option` on `current_viewport` rather than a default value: `ViewportId` has no meaningful placeholder, and `None` means "no canvas has drawn yet", which is true for the first frames and after a window closes. `finish` treats `None` as "no owner", so an early release cannot complete anything.

No doc comment on `move_blocked_start` and `move_viewport`. That is a deliberate omission in a file where most fields do have one: the two are read in exactly two functions (`current_move_drag`, `finish`) and their meaning is fully stated there, whereas the fields with comments (`move_blocked` above, `tool_override` below) are read in several places. Writing a comment here would have to repeat the explanation and would then be a second copy to keep in step - and the two lines that would be written ("where the refused press was", "the viewport the gesture belongs to") are exactly the field names.

Both new gesture fields are transient by nature and are not serialised - the surrounding fields' comments say the same about their neighbours ("Set by the canvas for its gestures, never saved"), so the absence of a `serde(skip)` attribute is consistent: this struct's persisted part is written elsewhere, and these three fields are runtime only.

`pub(crate)` rather than `pub`: every reader is in this crate (`canvas`, `tools`, `panels`), and the app struct is not part of the crate's public API.

### `crates/ui-egui/src/panels/layer_transfer.rs`

#### 39. `crates/ui-egui/src/panels/layer_transfer.rs` — hunk -1,16 +1,14

**Original code (upstream `main`)**

```rust
//! Dragging layers to another document (#589), as in Photoshop: drag layers from the Layers panel,
//! or with the Move tool from the canvas, onto another document's tab. The tab shows its document
//! while the pointer is over it (spring-loaded), and releasing over that document's canvas copies
//! the layers there with `layer.copyToDocument` (one history step in the destination).
//!
//! Where the copies land: a Move-tool drag puts the pixel that was grabbed under the pointer, and
//! a Layers panel drag centres the layers on it. With ⇧, or when released on the tab itself, they
//! keep their place when both documents have the same pixel size and are centred otherwise
//! (Photoshop's ⇧-drag). With Window › Arrange tiling several documents, releasing over another
//! document's tile drops there.
//!
//! A Move-tool drag starting inside a selection or moving a floating piece stays a move: Photoshop
//! would copy just the selected pixels, which is not supported yet.

use photocraft_doc::{DocId, LayerId};
use serde_json::json;
```

**Changed code (current `main`)**

```rust
//! Dragging layers or selections to another document (#589), as in Photoshop: drag layers from
//! the Layers panel, or with the Move tool from the canvas, onto another document's tab or
//! neighbouring tile. The tab shows its document while the pointer is over it (spring-loaded), and
//! releasing over that document's canvas copies the layers or selection there with
//! `layer.copyToDocument` (one history step in the destination).
//!
//! Where the copies land: a Move-tool drag puts the pixel that was grabbed under the pointer, and
//! a Layers panel drag centres the layers on it. With ⇧, or when released on the tab itself, they
//! keep their place when both documents have the same pixel size and are centred otherwise
//! (Photoshop's ⇧-drag). With Window › Arrange tiling several documents, releasing over another
//! document's tile drops there.

use photocraft_doc::{DocId, LayerId};
use serde_json::json;
```

**Explanation.** This is the module's `//!` documentation - the block at the top of `layer_transfer.rs` that a reader sees first and that `cargo doc` renders as the module's description. The original had three paragraphs; the change rewrites the first, keeps the second verbatim, and deletes the third.

Paragraph 1 originally read: "Dragging layers to another document (#589), as in Photoshop: drag layers from the Layers panel, or with the Move tool from the canvas, onto another document's tab. The tab shows its document while the pointer is over it (spring-loaded), and releasing over that document's canvas copies the layers there with `layer.copyToDocument` (one history step in the destination)."

It now reads: "Dragging layers **or selections** to another document (#589), as in Photoshop: drag layers from the Layers panel, or with the Move tool from the canvas, onto another document's tab **or neighbouring tile**. The tab shows its document while the pointer is over it (spring-loaded), and releasing over that document's canvas copies the layers **or selection** there with `layer.copyToDocument` (one history step in the destination)."

Three substantive edits. "layers or selections" states the new content class - the module's `TransferKind` has two variants and the doc now names both. "or neighbouring tile" names the second destination: the arranged-tiling path through `check_arranged_drag`, which existed only as a single sentence in paragraph 2 and was not in the summary. "copies the layers or selection there" makes the object of the copy agree with the new subject. "(one history step in the destination)" is unchanged and still true - the command is one `layer.copyToDocument`, journalled once on the destination document.

Paragraph 2 is untouched, and that is a claim worth checking rather than an accident. It says: a Move-tool drag puts the grabbed pixel under the pointer; a Layers-panel drag centres the layers on it; with ⇧, or when released on the tab itself, they keep their place when both documents have the same pixel size and are centred otherwise. Every one of those statements is still what the code does - `apply_drop_placement` (hunk 22) implements exactly that table, with the `grab`/`at` split for the two drag kinds and `!same_size` for the centring rule. If a statement in this paragraph had become false, it would have to be edited in the same hunk; none did.

Paragraph 3 is deleted. It read: "A Move-tool drag starting inside a selection or moving a floating piece stays a move: Photoshop would copy just the selected pixels, which is not supported yet." That is the limitation this pull request removes - a drag starting inside a selection or on an unmoved float now *does* transfer the selected pixels, through `TransferKind::Selection`. Keeping it would be the worst kind of stale documentation: a negative statement that tells a reader a feature is absent and, worse, tells a future contributor that the current behaviour of "stay a move" is intended and should be preserved.

The deletion is the part of this hunk that carries the most risk if it were omitted. An outdated positive statement merely omits information; an outdated negative statement actively misleads, and this one would have been read by exactly the person about to touch the arming code.

The remaining text keeps the issue reference `(#589)` and Photoshop as the reference behaviour, both of which are how this file's other comments justify their rules. No new sentence was added to explain the viewport guard or the tab-promotion logic, because those are documented at their functions (`finish`, `over_tab`, `current_move_drag`) and a module summary that tried to restate them would be a third place to keep in step.

#### 40. `crates/ui-egui/src/panels/layer_transfer.rs` — hunk -18,15 +16,24

**Original code (upstream `main`)**

```rust
use crate::PhotocraftApp;
use crate::state::Tool;

/// The layers being dragged and where from.
#[derive(Clone, Debug)]
struct Transfer {
    source: DocId,
    layers: Vec<u64>,
    /// The document point a Move-tool drag grabbed (`None` for a Layers panel drag).
    grab: Option<[f64; 2]>,
    /// What the ghost by the pointer says.
    label: String,
}

fn key() -> egui::Id {
```

**Changed code (current `main`)**

```rust
use crate::PhotocraftApp;
use crate::state::Tool;

#[derive(Clone, Debug)]
pub enum TransferKind {
    Layers(Vec<u64>),
    Selection { mask: bool },
}

/// The layers or selection being dragged and where from.
#[derive(Clone, Debug)]
struct Transfer {
    source: DocId,
    kind: TransferKind,
    /// The document point a Move-tool drag grabbed (`None` for a Layers panel drag).
    grab: Option<[f64; 2]>,
    /// What the ghost by the pointer says.
    label: String,
    /// The viewport the transfer was armed in: only that viewport completes it, so a document
    /// window's `finish` cannot clear a transfer the main window is still dragging.
    viewport: egui::ViewportId,
}

fn key() -> egui::Id {
```

**Explanation.** `Transfer` is the state of a drag in progress. It is stored in egui's temporary data under one key and read by every viewport of the context, so its fields are the entire memory of a cross-document drag.

The original held four fields: `source: DocId`, `layers: Vec<u64>`, `grab: Option<[f64; 2]>` and `label: String`. `source` is the document the drag started in; `layers` the ids of the layers being dragged; `grab` the document-space point the pointer grabbed, or `None` for a Layers-panel drag; `label` the text the ghost by the pointer shows.

The change replaces `layers` with `kind: TransferKind` and adds `viewport: egui::ViewportId`; the other three fields and the derives are untouched.

`TransferKind::Layers(Vec<u64>)` carries the same ids the `layers` field used to. The `u64` rather than `LayerId` is the existing choice and it is what the JSON command needs: `drop` builds `json!({"document": dest, "source": src, "layers": layers})`, and the engine's command layer accepts raw numbers, converting to its newtype internally. Storing the newtype would mean converting on every construction and back on every serialisation, for no benefit in a value that never leaves this module.

`TransferKind::Selection { mask: bool }` is the new content class. It carries no ids at all, because a selection drag is "whatever the selection is now" - the pixels are read from the source document at drop time, not captured at arm time. That is deliberate: a user who makes a selection, starts dragging it, and (while the pointer is over the destination tab) changes nothing, expects the current selection to travel. Capturing it at arm time would also cost a full copy of the selected pixels in the temp data.

`mask: bool` cannot be derived later. By the time `drop` runs, the active document may have changed (tab hover switches documents), so `ui.mask_target` and `ui.tool_options`-style state no longer describe the source. The flag records the answer at the moment of arming, in `arm_move_drag`, where both the float's `parts.is_mask()` and `layer_mask_targeted()` are still readable for the source document.

`viewport: egui::ViewportId` records the window that armed the transfer, and its doc comment states the rule: only that viewport completes it. Without the field, `finish` could not distinguish a release in the window that armed the transfer from a release in any other window - the value lives in the shared `Context`, so every viewport sees it, and before this change whichever viewport saw a release first completed the drag. In the application embedding that is a real scenario: the main window and a detached document window both run `finish`, and a release in the document window (for instance, clicking to focus it) would otherwise deliver the main window's drag.

`ViewportId` rather than a bare `u64` or an `is_extra: bool`: the id is what `finish` compares against `ctx.viewport_id()`, and it is the same type `app.move_viewport` uses (hunk 16), so the two guards are written the same way and cannot disagree about what "the same window" means.

Why `TransferKind` is `pub` while `Transfer` stays private: `arm_move_drag` takes a `MoveDrag` (hunk 19/20) and both types are public because the arming functions are public and their parameters must be nameable from `canvas/tabs.rs` and from tests. `Transfer` itself never leaves the module - only `get`, `clear` and the functions that read it are exposed - so it stays private, and its fields stay visible to this module only.

Why `TransferKind` derives `Clone` and `Debug`: `Transfer` derives them, and a derived impl requires every field to implement them. `Transfer` must be `Clone` because `get` returns an owned copy out of the temp data (`ctx.data(|d| d.get_temp::<Transfer>(key()))` clones), and `Debug` because test failures print it.

The enum rather than a parallel `selection: Option<...>` field: a parallel field would make "layers and a selection at once" representable, and every reader would have to decide which one wins. With the enum, `drop` matches once and each arm builds exactly one command shape, and adding a third kind later is a compile error at every match rather than a silently ignored field.

The doc comment on the struct is updated in the same hunk ("The layers or selection being dragged") because the struct's one-line description would otherwise describe only one of its two kinds. Nothing else in the type changes: no `PartialEq`, no `Default`, no `Hash`, because nothing compares or caches a `Transfer` by value.

#### 41. `crates/ui-egui/src/panels/layer_transfer.rs` — hunk -42,58 +49,136

**Original code (upstream `main`)**

```rust
}

/// The layers `id` stands for: the selection when it is one of the selected layers, else itself.
fn begin(app: &PhotocraftApp, ctx: &egui::Context, id: LayerId, grab: Option<[f64; 2]>) -> bool {
    let Some(st) = app.session.active() else { return false };
    let layers = if st.is_layer_selected(id) { st.selected_layers() } else { vec![id] };
    let label = match layers.as_slice() {
        [one] => st.doc.layer(*one).map_or_else(String::new, |l| l.name.clone()),
        many => crate::i18n::trn(crate::i18n::current(), many.len() as u64, "{n} layer", "{n} layers"),
    };
    let t = Transfer { source: st.doc.id, layers: layers.iter().map(|l| l.0).collect(), grab, label };
    ctx.data_mut(|d| d.insert_temp(key(), t));
    true
}

/// A Layers panel row drag began on `layer`: it can be dropped on another document.
pub fn begin_from_panel(app: &PhotocraftApp, ctx: &egui::Context, layer: LayerId) {
    begin(app, ctx, layer, None);
}

/// A Move-tool drag of whole layers (not a selection or a floating piece) under way.
fn move_drag(app: &PhotocraftApp) -> Option<[f64; 2]> {
    let d = app.drag.as_ref().filter(|d| d.tool == Tool::Move && d.sel_move.is_none())?;
    let st = app.session.active()?;
    let plain = app.ui.transform.is_none() && app.guide_drag.is_none() && st.doc.selection.is_none() && st.floating.is_none();
    plain.then_some(d.start)
}

/// The pointer, while layers are dragged (or a Move drag could become a transfer): tabs only
/// hit-test it then.
pub fn pointer_if_armed(app: &PhotocraftApp, ctx: &egui::Context) -> Option<egui::Pos2> {
    if get(ctx).is_none() && move_drag(app).is_none() {
        return None;
    }
    ctx.input(|i| i.pointer.latest_pos())
}

/// The pointer is over document tab `index` while layers are dragged: a Move drag becomes a
/// transfer (the source layers stay where they were), the tab's document is shown, and a release
/// right here drops the layers into it.
pub fn over_tab(app: &mut PhotocraftApp, ctx: &egui::Context, index: usize) {
    if get(ctx).is_none() {
        let Some(grab) = move_drag(app) else { return };
        let Some(active) = app.session.active().and_then(|st| st.active_layer) else { return };
        if app.session.active_index() == Some(index) {
            return;
        }
        // The Move drag ends without moving anything; an ⌥-drag's copy is taken back.
        app.drag = None;
        app.move_preview = None;
        crate::move_mods::abandon(app);
        let active = app.session.active().and_then(|st| st.active_layer).unwrap_or(active);
        if !begin(app, ctx, active, Some(grab)) {
            return;
        }
    }
    if app.session.active_index() != Some(index) {
        app.session.set_active(index);
```

**Changed code (current `main`)**

```rust
}

/// The layers `id` stands for: the selection when it is one of the selected layers, else itself.
fn begin_layers(app: &PhotocraftApp, ctx: &egui::Context, id: LayerId, grab: Option<[f64; 2]>) -> bool {
    let Some(st) = app.session.active() else { return false };
    let layers = if st.is_layer_selected(id) { st.selected_layers() } else { vec![id] };
    let label = match layers.as_slice() {
        [one] => st.doc.layer(*one).map_or_else(String::new, |l| l.name.clone()),
        many => crate::i18n::trn(crate::i18n::current(), many.len() as u64, "{n} layer", "{n} layers"),
    };
    let t = Transfer { source: st.doc.id, kind: TransferKind::Layers(layers.iter().map(|l| l.0).collect()), grab, label, viewport: ctx.viewport_id() };
    ctx.data_mut(|d| d.insert_temp(key(), t));
    true
}

/// Arm a selection transfer: the selected pixels follow the pointer into another document.
fn begin_selection(app: &PhotocraftApp, ctx: &egui::Context, grab: [f64; 2], mask: bool) -> bool {
    let Some(st) = app.session.active() else { return false };
    let label = tl!("Selection");
    let t = Transfer { source: st.doc.id, kind: TransferKind::Selection { mask }, grab: Some(grab), label: label.to_string(), viewport: ctx.viewport_id() };
    ctx.data_mut(|d| d.insert_temp(key(), t));
    true
}

/// A Layers panel row drag began on `layer`: it can be dropped on another document.
pub fn begin_from_panel(app: &PhotocraftApp, ctx: &egui::Context, layer: LayerId) {
    begin_layers(app, ctx, layer, None);
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MoveDrag {
    Layers([f64; 2]),
    Selection { grab: [f64; 2], cut: bool },
}

/// A Move-tool drag under way: whole layers or selected pixels.
pub fn current_move_drag(app: &PhotocraftApp) -> Option<MoveDrag> {
    if app.ui.transform.is_some() || app.guide_drag.is_some() {
        return None;
    }
    let st = app.session.active()?;
    if let Some(d) = app.drag.as_ref().filter(|d| d.tool == Tool::Move) {
        if let Some(cut) = d.sel_move {
            return Some(MoveDrag::Selection { grab: d.start, cut });
        }
        if st.doc.selection.is_none() && st.floating.is_none() {
            return Some(MoveDrag::Layers(d.start));
        }
        return None;
    }
    let is_move = app.ui.tool == Tool::Move
        || app.tool_override == Some(Tool::Move)
        || app.move_blocked_start.is_some()
        || app.layer_box.as_ref().is_some_and(|b| b.grabbed_layer.is_some());
    if is_move && st.doc.selection.is_none() && st.floating.is_none() {
        if let Some(b) = app.layer_box.as_ref()
            && b.grabbed_layer.is_some()
        {
            return Some(MoveDrag::Layers(b.start));
        }
        if let Some(start) = app.move_blocked_start {
            return Some(MoveDrag::Layers(start));
        }
    }
    None
}

/// The pointer, while layers or selections are dragged: tabs and neighbouring cells hit-test it then.
pub fn pointer_if_armed(app: &PhotocraftApp, ctx: &egui::Context) -> Option<egui::Pos2> {
    if get(ctx).is_none() && current_move_drag(app).is_none() {
        return None;
    }
    ctx.input(|i| i.pointer.latest_pos())
}

/// In Window › Arrange views: checks if a Move drag has crossed into another arranged document's cell.
pub fn check_arranged_drag(app: &mut PhotocraftApp, ctx: &egui::Context) {
    if get(ctx).is_some() {
        return;
    }
    // Only the window that began the move owns it: another window's frame must not read its
    // own pointer against this gesture and arm a transfer on the first window's behalf.
    if app.move_viewport.is_some_and(|v| v != ctx.viewport_id()) {
        return;
    }
    let Some(drag) = current_move_drag(app) else { return };
    let Some(pointer) = ctx.input(|i| i.pointer.latest_pos()) else { return };
    let Some(cells) = crate::canvas::arranged_cells(app, app.last_canvas_rect) else { return };
    let active_idx = app.session.active_index();
    if cells.into_iter().any(|(d, r)| r.contains(pointer) && Some(d) != active_idx) {
        arm_move_drag(app, ctx, drag);
    }
}

/// Turn the drag under way into a cross-document transfer owned by `ctx`.
fn arm_move_drag(app: &mut PhotocraftApp, ctx: &egui::Context, drag: MoveDrag) {
    let grabbed_layer = app.layer_box.as_ref().and_then(|b| b.grabbed_layer);
    app.drag = None;
    app.layer_box = None;
    app.move_blocked = false;
    app.move_blocked_start = None;
    app.move_viewport = None;
    app.move_preview = None;
    crate::move_mods::abandon(app);
    match drag {
        MoveDrag::Layers(grab) => {
            let target_layer = grabbed_layer.or_else(|| app.session.active().and_then(|st| st.active_layer));
            if let Some(active) = target_layer {
                begin_layers(app, ctx, active, Some(grab));
            }
        }
        MoveDrag::Selection { grab, .. } => {
            let mask = app.session.active().and_then(photocraft_engine::float_cmds::floating).map_or_else(|| app.layer_mask_targeted(), |f| f.parts.is_mask());
            if let Some(st) = app.session.active_mut()
                && st.floating.as_ref().is_some_and(|f| !f.moved && f.offset == (0, 0))
            {
                st.floating = None;
            }
            begin_selection(app, ctx, grab, mask);
        }
    }
}

/// The pointer is over document tab `index` while layers/selection are dragged: a Move drag becomes
/// a transfer (the source layers stay where they were), the tab's document is shown, and a release
/// right here drops the layers into it.
pub fn over_tab(app: &mut PhotocraftApp, ctx: &egui::Context, index: usize) {
    if get(ctx).is_none() {
        let Some(drag) = current_move_drag(app) else { return };
        if app.session.active_index() == Some(index) {
            return;
        }
        arm_move_drag(app, ctx, drag);
    }
    if app.session.active_index() != Some(index) {
        app.session.set_active(index);
```

**Explanation.** This is the arming half of the module: the functions that turn a gesture into a `Transfer`.

The original `begin(app, ctx, id, grab)` did one job - arm a layer transfer - and `begin_from_panel` called it with `grab: None`. A separate `move_drag(app) -> Option<[f64; 2]>` answered "is a plain Move drag of whole layers under way, and where did it start", with a single condition: the drag's tool is Move, `sel_move` is `None`, there is no transform or guide drag, no selection and no float.

The change splits `begin` into `begin_layers` and `begin_selection`, keeps `begin_from_panel` calling `begin_layers`, and replaces `move_drag` with `current_move_drag(app) -> Option<MoveDrag>` plus a public `MoveDrag` enum.

`begin_layers` is the old body with two edits: it builds `kind: TransferKind::Layers(...)` instead of `layers`, and it writes `viewport: ctx.viewport_id()`. The label logic is untouched: one layer uses the layer's name, several use the translated "{n} layer"/"{n} layers" string. That label is what the ghost paints.

`begin_selection` is new and short: it takes the grab point and the `mask` flag, uses the fixed translated "Selection" label, sets `grab: Some(grab)` because a selection drag always has one, and writes the same `viewport`. It does not inspect the selection or the layer - the pixels are read later, by the command, from the float if there is one.

`MoveDrag` is the gesture-level answer to "what is being dragged". `Layers([f64; 2])` carries the grab point. `Selection { grab, cut }` carries the grab point and the cut flag the drag started with (`⌥` copies, the default cuts) - a flag that exists in the drag state and has no meaning for a layer drag, which is why the two variants carry different data.

`current_move_drag` is the replacement for `move_drag` and is where the canvas-only cases are added. It first refuses when a transform or guide drag is active. Then, if `app.drag` is a Move drag: `sel_move` present means a selection drag (return `Selection` with the drag's own `cut`), otherwise a selection or float in the document means "not a plain layer drag" (return `None`), and the remaining case is a whole-layer drag. If there is no `app.drag` at all, it asks a different question: is the Move tool active (or overridden, or a blocked press recorded, or a marquee holding a grabbed layer), and is the document free of a selection and a float. In that case the marquee's `grabbed_layer` or `move_blocked_start` supplies the grab point.

That second branch is what makes a locked or Background layer draggable: those layers never produce an `app.drag` (the press was refused), but they do produce a `layer_box` with `grabbed_layer` or a `move_blocked_start`, and this branch turns either into `MoveDrag::Layers`. The `selection.is_none() && floating.is_none()` condition keeps the promotion honest: with a selection, the pixels are what the user is dragging, and treating the gesture as a whole-layer drag would copy the whole layer instead.

`is_move` also accepts `app.tool_override == Some(Tool::Move)`, which is how ⌘-held Move works; without it, a drag started with ⌘ would not be promotable.

#### 42. `crates/ui-egui/src/panels/layer_transfer.rs` — hunk -108,15 +193,46

**Original code (upstream `main`)**

```rust

/// Every frame after the document area: a release over another document's canvas drops the
/// layers there; while the button is held a ghost follows the pointer.
pub fn finish(app: &mut PhotocraftApp, ctx: &egui::Context) {
    let Some(t) = get(ctx) else { return };
    let (pos, released, down) = ctx.input(|i| (i.pointer.latest_pos(), i.pointer.any_released(), i.pointer.primary_down()));
    let target = pos.and_then(|p| target_at(app, p));
    if !released {
        if !down {
            clear(ctx);
        } else if let Some(p) = pos
            && app.session.active().is_some_and(|st| st.doc.id != t.source)
        {
            ghost(ctx, p, &t.label);
            if target.is_some() {
```

**Changed code (current `main`)**

```rust

/// Every frame after the document area: a release over another document's canvas drops the
/// layers there; while the button is held a ghost follows the pointer.
/// Is a cross-document layer or selection transfer currently armed?
pub fn is_transfer_in_progress(app: &PhotocraftApp) -> bool {
    app.frame_tool_ctx.as_ref().is_some_and(|ctx| get(ctx).is_some())
}

/// Whether a transfer is armed in `ctx` (any viewport).
#[cfg(test)]
pub fn is_transfer_armed(ctx: &egui::Context) -> bool {
    get(ctx).is_some()
}

pub fn finish(app: &mut PhotocraftApp, ctx: &egui::Context) {
    // The primary button ends the gesture: a secondary release during a drag leaves it held.
    let (released, down) = ctx.input(|i| (i.pointer.primary_released(), i.pointer.primary_down()));
    if (!down || released) && get(ctx).is_none() && app.move_viewport.is_none_or(|v| v == ctx.viewport_id()) {
        app.move_blocked = false;
        app.move_blocked_start = None;
        app.move_viewport = None;
        if let Some(b) = app.layer_box.as_ref()
            && b.grabbed_layer.is_some()
        {
            app.layer_box = None;
        }
    }
    let Some(t) = get(ctx) else { return };
    // Only the viewport that armed the transfer completes it: a document window's pointer state
    // must not clear a transfer the main window is still dragging.
    if t.viewport != ctx.viewport_id() {
        return;
    }
    // Only the primary button completes a transfer: a secondary release while the drag is
    // held leaves it armed for the intended drop.
    let (pos, released, down) = ctx.input(|i| (i.pointer.latest_pos(), i.pointer.primary_released(), i.pointer.primary_down()));
    let target = pos.and_then(|p| target_at(app, p));
    if !released {
        if !down {
            clear(ctx);
        } else if let Some(p) = pos
            && (target.is_some_and(|(d, _)| app.session.documents().get(d).is_some_and(|st| st.doc.id != t.source))
                || app.session.active().is_some_and(|st| st.doc.id != t.source))
        {
            ghost(ctx, p, &t.label);
            if target.is_some() {
```

**Explanation.** Seven items are added to the module. Each is described separately.

`TransferKind` is declared here rather than in hunk 18 because the enum has to exist before `MoveDrag` can be converted into `Transfer`. Its two variants are the two kinds of content the module can carry: `Layers(Vec<u64>)` holds layer ids as plain `u64` (the engine's id type is a newtype; the transfer stores the raw value and rebuilds the newtype at the command boundary), and `Selection { mask: bool }` holds the flag that decides whether the command gets `"target": "mask"`.

`MoveDrag` is the gesture-level answer to "what is being dragged", separate from `Transfer` because it exists before arming and carries different data. `Layers([f64; 2])` carries the grab point only. `Selection { grab: [f64; 2], cut: bool }` carries the grab point and the cut flag the drag started with - the flag `app.drag.sel_move` records at the press (⌥ copies, the default cuts). It is `#[derive(Clone, Copy, Debug, PartialEq)]` because it is passed by value into `arm_move_drag` and compared in tests.

`current_move_drag(app) -> Option<MoveDrag>` replaces the old `move_drag(app) -> Option<[f64; 2]>`. It first refuses when `ui.transform` or `guide_drag` is active (a transform gesture and a guide drag are not layer drags). Then, with `app.drag` present and its tool Move: `d.sel_move` being `Some(cut)` means a selection drag, and it returns `Selection { grab: d.start, cut }`; otherwise, a document with a selection or a float returns `None` (the pixels are what is being dragged, not the layer); the remaining case is `Layers(d.start)`. With no `app.drag` at all it asks a different question: `is_move` is true when the tool is Move, when `tool_override` is Move (the ⌘-held path), when `move_blocked_start` is set (a refused press), or when `layer_box.grabbed_layer` is set (a marquee that grabbed a layer). When `is_move` holds and the document has no selection and no float, the function promotes the marquee: `Layers(b.start)` from `layer_box`, or `Layers(start)` from `move_blocked_start`. That second branch is the only way a locked or Background layer can start a transfer, because those presses never produce an `app.drag`.

`pointer_if_armed(app, ctx) -> Option<Pos2>` returns the pointer only while a transfer is armed or a move drag is live, and `None` otherwise. Tabs and arranged cells hit-test the pointer every frame; without this gate they would treat ordinary pointer movement as a drag. The condition `get(ctx).is_none() && current_move_drag(app).is_none()` is the exact complement of "something is being dragged", so the gate cannot be accidentally satisfied by a stale flag.

`check_arranged_drag(app, ctx)` is the per-frame promotion for `Window › Arrange`. It returns early when a transfer is already armed (`get(ctx).is_some()`), when `move_viewport` names another window (the guard added by this pull request), when there is no move drag, when the pointer is unknown, or when `arranged_cells` returns `None` (not in arranged view). It then asks whether any cell contains the pointer and is not the active document's cell; if so, it calls `arm_move_drag`. The "not the active document" condition is what makes the promotion happen on the crossing rather than on every frame inside the source cell.

`arm_move_drag(app, ctx, drag)` is the conversion. It first reads `grabbed_layer` from the marquee, then clears every piece of the in-flight drag state: `app.drag`, `app.layer_box`, `app.move_blocked`, `app.move_blocked_start`, `app.move_viewport`, `app.move_preview`, and `move_mods::abandon(app)`. Clearing before arming is what makes the transfer the only live gesture: without it, the marquee would keep drawing and the move preview would keep following the pointer alongside the transfer's ghost. For `Layers(grab)` it arms with `grabbed_layer` or the active layer, calling `begin_layers(app, ctx, active, Some(grab))`; for `Selection { grab, .. }` it computes `mask` from the float (`f.parts.is_mask()`) or from `layer_mask_targeted()`, discards a float that has not moved (`!f.moved && f.offset == (0, 0)`), and calls `begin_selection`. Discarding the unmoved float is what makes the transfer start from the original pixels: an unmoved float is a committed cut that the user has not dragged anywhere, so its "rest" is the layer and its "piece" is the selection; promoting it would copy a float whose position the user never chose.

`over_tab(app, ctx, index)` handles the tab path. If nothing is armed it promotes `current_move_drag` into a transfer, but only when the tab is not the active document (hovering the active document's own tab is not a cross-document drag). It then switches the active document if needed, clears the jobs focus, and removes the Layers-panel drag key (`egui::Id::unique("layer-drag")`) so the new document's rows do not inherit the drag. On `pointer.any_released()` it drops into that document with `at: None`, which makes the placement centre the content (the tab has no document point).

`is_transfer_in_progress(app)` answers "is a transfer armed in the frame's context". It reads `app.frame_tool_ctx` because the event handlers have no `ctx` parameter; the field is the frame's stored context, set by the tool pipeline. `is_transfer_armed(ctx)` is the same question for an explicit context and is `#[cfg(test)]`, because only the tests need to ask it by value.

#### 43. `crates/ui-egui/src/panels/layer_transfer.rs` — hunk -131,31 +247,46

**Original code (upstream `main`)**

```rust
    clear(ctx);
}

/// The document under screen point `p` that isn't the source, and the document point there
/// (`None` in a tile: tiles keep their own views).
fn target_at(app: &PhotocraftApp, p: egui::Pos2) -> Option<(usize, Option<[f64; 2]>)> {
    if !app.last_canvas_rect.contains(p) {
        return None;
    }
    match crate::canvas::arranged_cells(app, app.last_canvas_rect) {
        Some(cells) => cells.into_iter().find(|(_, r)| r.contains(p)).map(|(d, _)| (d, None)),
        None => Some((app.session.active_index()?, crate::canvas::ViewXform::active(app).map(|xf| xf.to_doc(p)))),
    }
}

/// Copy the dragged layers into document `dest`, at document point `at` (see the module docs).
fn drop(app: &mut PhotocraftApp, ctx: &egui::Context, dest: usize, at: Option<[f64; 2]>) {
    let Some(t) = get(ctx) else { return };
    clear(ctx);
    let docs = app.session.documents();
    let Some(src) = docs.iter().position(|d| d.doc.id == t.source) else { return };
    if dest == src
        && let Some(at) = at
    {
        if let Some(frame) = crate::frame_tool::frame_at(app, at)
            && let [source] = t.layers.as_slice()
        {
            let r = app.run("layer.frame.setContent", json!({"layer":frame.0,"content":source}));
            if let Err(e) = r {
                app.ui.status = e;
                app.ui.status_error = true;
```

**Changed code (current `main`)**

```rust
    clear(ctx);
}

/// The document under screen point `p` that isn't the source, and the document point there.
fn target_at(app: &PhotocraftApp, p: egui::Pos2) -> Option<(usize, Option<[f64; 2]>)> {
    if !app.last_canvas_rect.contains(p) {
        return None;
    }
    match crate::canvas::arranged_cells(app, app.last_canvas_rect) {
        Some(cells) => {
            let (d, r) = cells.into_iter().find(|(_, r)| r.contains(p))?;
            let view = app.ui.views.get(d)?;
            let ppp = app.canvas_ppp();
            let aspect = app.ui.view.display_aspect();
            let xf = crate::canvas::ViewXform {
                rect: crate::rulers::content_rect(app, r),
                zoom: view.zoom / ppp,
                center: view.center,
                flip: app.ui.view.flip_horizontal,
                rotation: view.rotation,
                aspect,
            };
            Some((d, Some(xf.to_doc(p))))
        }
        None => Some((app.session.active_index()?, crate::canvas::ViewXform::active(app).map(|xf| xf.to_doc(p)))),
    }
}

/// Copy the dragged layers or selection into document `dest`, at document point `at`.
/// When transferring a selection to another document, the source document remains intact (not cut),
/// matching Photoshop's cross-document drag behaviour.
fn drop(app: &mut PhotocraftApp, ctx: &egui::Context, dest: usize, at: Option<[f64; 2]>) {
    let Some(t) = get(ctx) else { return };
    clear(ctx);
    let docs = app.session.documents();
    let Some(src) = docs.iter().position(|d| d.doc.id == t.source) else { return };
    if dest == src {
        if let TransferKind::Layers(layers) = &t.kind
            && let Some(at) = at
            && let Some(frame) = crate::frame_tool::frame_at(app, at)
            && let [source] = layers.as_slice()
        {
            let r = app.run("layer.frame.setContent", json!({"layer": frame.0, "content": source}));
            if let Err(e) = r {
                app.ui.status = e;
                app.ui.status_error = true;
```

**Explanation.** `target_at` answers "which document is under this screen point, and where in that document". It has two branches: arranged tiling and the ordinary single-canvas view. This hunk rewrites the first branch and only clarifies the doc comment.

The signature is unchanged: `fn target_at(app: &PhotocraftApp, p: egui::Pos2) -> Option<(usize, Option<[f64; 2]>)>` - a document index and an optional document point. `None` for the whole return means "no document there"; `None` for the second element means "there is a document, but this point does not map to one of its pixels" (a release on a tab, or outside a tile's content rect).

The guard is unchanged: `if !app.last_canvas_rect.contains(p) { return None; }`. `last_canvas_rect` is the canvas area recorded on the previous frame, so a pointer over the Layers panel or a toolbar is not a drop target at all.

The old arranged branch returned `(d, None)`:

```rust
Some(cells) => cells.into_iter().find(|(_, r)| r.contains(p)).map(|(d, _)| (d, None)),
```

That `None` meant "no document point", so a release inside a tile took the `center` path in `drop`'s placement logic - the transferred content was centred in the destination regardless of where in the tile the pointer was. For a tile, though, the pointer *does* have a document position: the tile renders the document through its own `ViewXform` (zoom, centre, rotation, flip, aspect), and that transform can be reconstructed.

The new branch reconstructs it:

```rust
let (d, r) = cells.into_iter().find(|(_, r)| r.contains(p))?;
let view = app.ui.views.get(d)?;
let ppp = app.canvas_ppp();
let aspect = app.ui.view.display_aspect();
let xf = crate::canvas::ViewXform { rect: crate::rulers::content_rect(app, r), zoom: view.zoom / ppp, center: view.center, flip: app.ui.view.flip_horizontal, rotation: view.rotation, aspect };
Some((d, Some(xf.to_doc(p))))
```

Each field is the same one the tile's own drawing code uses, which is the requirement: the transform must agree with what the user sees, or the drop lands somewhere else. `rect` is `content_rect(app, r)` rather than the raw cell rect, because the cell includes the rulers and padding while `content_rect` is the image area - using the cell would offset every drop by the ruler width. `zoom: view.zoom / ppp` divides by the canvas points-per-pixel, because `view.zoom` is stored in physical pixels and `ViewXform` works in points; omitting the division would scale the drop by the display's DPI factor. `center`, `rotation` come from the per-document `view` (`app.ui.views[d]`), so a rotated or scrolled tile maps correctly. `flip` comes from `app.ui.view.flip_horizontal` - the global view setting, not the per-document one, which is what the tile's own draw uses.

The `?` on `app.ui.views.get(d)` is the one new failure path: a cell index that has no entry in `views` returns `None` from the whole function, which means "no drop target". That is the right answer rather than a panic: `views` is rebuilt as documents open and close, and a stale cell on the frame a document closes would otherwise index out of bounds.

The `None` branch (not arranged) is unchanged in behaviour but its comment lost the parenthetical "(tiles keep their own views)", because tiles no longer ignore the point. The code still reads `Some((app.session.active_index()?, crate::canvas::ViewXform::active(app).map(|xf| xf.to_doc(p))))` - the `?` means "no active document, no target", and the inner `map` means "an active view whose transform is not computable still counts as a target, just without a point".

The blast radius: `finish` is the only caller. A tile drop now carries a point, so `apply_drop_placement` takes the `Some(at)` path and, with a `grab`, computes an offset instead of centring. That is a behaviour change the tests in hunk 23 cover (`..._in_arranged_view_transfers_directly_to_neighbour_cell` asserts the centre matches the destination transform's mapping of the drop point).

#### 44. `crates/ui-egui/src/panels/layer_transfer.rs` — hunk -165,18 +296,35

**Original code (upstream `main`)**

```rust
    }
    let Some(to) = docs.get(dest).filter(|_| dest != src) else { return };
    let same_size = to.doc.size == docs[src].doc.size;
    let mut p = json!({"document": dest, "source": src, "layers": t.layers});
    match at.filter(|_| !ctx.input(|i| i.modifiers.shift)) {
        Some(at) => match t.grab {
            Some(g) => p["offset"] = json!([(at[0] - g[0]).round(), (at[1] - g[1]).round()]),
            None => p["at"] = json!(at),
        },
        None => p["center"] = json!(!same_size),
    }
    if let Err(e) = app.run("layer.copyToDocument", p) {
        app.ui.status = e;
        app.ui.status_error = true;
    }
}

/// The dragged layers' name next to the pointer (also the Layers panel's reorder drag).
```

**Changed code (current `main`)**

```rust
    }
    let Some(to) = docs.get(dest).filter(|_| dest != src) else { return };
    let same_size = to.doc.size == docs[src].doc.size;

    let shift = ctx.input(|i| i.modifiers.shift);
    let mut p = match t.kind {
        TransferKind::Layers(layers) => json!({"document": dest, "source": src, "layers": layers}),
        TransferKind::Selection { mask } => {
            let mut v = json!({"document": dest, "source": src, "selection": true, "cut": false});
            if mask {
                v["target"] = json!("mask");
            }
            v
        }
    };
    apply_drop_placement(&mut p, at, t.grab, shift, same_size);
    app.session.set_active(dest);
    if let Err(e) = app.run("layer.copyToDocument", p) {
        app.ui.status = e;
        app.ui.status_error = true;
    }
}

/// Place the dropped transfer at `at`; with ⇧ it is centred when the documents differ in size.
fn apply_drop_placement(p: &mut serde_json::Value, at: Option<[f64; 2]>, grab: Option<[f64; 2]>, shift: bool, same_size: bool) {
    match at.filter(|_| !shift) {
        Some(at) => match grab {
            Some(g) => p["offset"] = json!([(at[0] - g[0]).round(), (at[1] - g[1]).round()]),
            None => p["at"] = json!(at),
        },
        None => p["center"] = json!(!same_size),
    }
}

/// The dragged layers' name next to the pointer (also the Layers panel's reorder drag).
```

**Explanation.** `drop` is the function that turns an armed transfer into a command. This hunk replaces a single-shape command with a match on the content kind, extracts the placement logic into its own function, and adds an active-document switch.

The common prologue is unchanged: `let Some(t) = get(ctx) else { return }; clear(ctx);` then the source lookup `let Some(src) = docs.iter().position(|d| d.doc.id == t.source) else { return };` and the same-size test `let same_size = to.doc.size == docs[src].doc.size;`. `clear(ctx)` before anything else is what guarantees a single completion even if `app.run` panics or the placement fails.

Before, the command was built in one line: `let mut p = json!({"document": dest, "source": src, "layers": t.layers});`. That shape only describes a layer transfer. Now it is a match:

```rust
let shift = ctx.input(|i| i.modifiers.shift);
let mut p = match t.kind {
    TransferKind::Layers(layers) => json!({"document": dest, "source": src, "layers": layers}),
    TransferKind::Selection { mask } => {
        let mut v = json!({"document": dest, "source": src, "selection": true, "cut": false});
        if mask { v["target"] = json!("mask"); }
        v
    }
};
```

`shift` is read once, before the match, and passed to the placement function rather than being read inside it. That is a change in structure: the old code inlined the modifier test into the `match at.filter(|_| !ctx.input(|i| i.modifiers.shift))`, which read the input inside a filter closure. Reading it once makes the value visible in a debugger and lets `apply_drop_placement` be a pure function of its arguments - which is what makes it testable and readable.

`"cut": false` is hard-coded in the selection command, and the doc comment above explains why: "When transferring a selection to another document, the source document remains intact (not cut), matching Photoshop's cross-document drag behaviour." The `cut` field exists in the command for the in-document case; a cross-document drag is always a copy, so the flag is false regardless of whether the user held ⌥. That is a deliberate product decision, not an omission - a cut across documents would delete pixels from a document the user may be about to close.

`"target": "mask"` is added only when `mask` is true, so the JSON has no `target` key for a pixel selection. The engine's default target is pixels, and omitting the key rather than sending `"target": "pixels"` keeps the command identical to what the in-document path sends - one fewer shape for the engine to handle.

`apply_drop_placement(&mut p, at, t.grab, shift, same_size)` replaces the inline block. Its body is byte-equivalent to what was removed: `match at.filter(|_| !shift) { Some(at) => match grab { Some(g) => p["offset"] = json!([(at[0] - g[0]).round(), (at[1] - g[1]).round()]), None => p["at"] = json!(at) }, None => p["center"] = json!(!same_size) }`. Two details preserved: the `.round()` on the offset, because a fractional document offset would make the copied pixels land between pixel centres, and `!same_size` for `center`, because a Shift-drag between different-size documents centres while the same-size one keeps its place.

`app.session.set_active(dest)` is new and is the behavioural addition a reader might miss. After a cross-document drop the destination becomes the active document, which is what Photoshop does and what the user expects: the pasted content appears in the document they were dragging to, so that document should be the one they are now editing. Placing it before `app.run` matters because the command refers to `"document": dest` by index - switching first keeps the index and the active state consistent for any code the command triggers.

The error path is unchanged: `if let Err(e) = app.run("layer.copyToDocument", p) { app.ui.status = e; app.ui.status_error = true; }`. The transfer is already cleared, so a failed drop leaves nothing armed and the message is the only trace - which is correct, because retrying automatically would be surprising.

### `crates/ui-egui/src/panels/layer_transfer/tests.rs`

#### 45. `crates/ui-egui/src/panels/layer_transfer/tests.rs` — hunk -130,3 +130,343

**Original code (upstream `main`)**

```rust
    assert_eq!((docs[0].doc.layers.len(), docs[1].doc.layers.len()), (2, 1));
    assert_eq!(h.state().session.active_index(), Some(0));
}
```

**Changed code (current `main`)**

```rust
    assert_eq!((docs[0].doc.layers.len(), docs[1].doc.layers.len()), (2, 1));
    assert_eq!(h.state().session.active_index(), Some(0));
}

#[test]
/// Dragging a selection onto another document's tab copies the selected pixels and leaves the source intact.
fn a_move_tool_selection_drag_onto_a_tab_copies_selected_pixels_to_destination() {
    let (mut h, paint) = harness();
    // Create a selection inside the red square
    h.state_mut().session.execute("select.rect", json!({"x": 4, "y": 6, "width": 5, "height": 10})).unwrap();
    h.state_mut().ui.tool = Tool::Move;
    h.run_steps(2);

    let grab = ViewXform::active(h.state()).unwrap().to_screen(6.0, 11.0);
    let grab_doc = ViewXform::active(h.state()).unwrap().to_doc(grab);
    let over_tab = tab(&h, "dest-doc");
    let drop = h.state().last_canvas_rect.center();
    drag_along(&mut h, &[grab, over_tab, drop], Modifiers::NONE);

    // Destination now has the copied 5x10 selection layer placed at the drop point
    let at = ViewXform::active(h.state()).unwrap().to_doc(drop);
    let (_, r) = copy(&h);
    assert_eq!(r.size(), vec2(5.0, 10.0), "only the 5x10 selected region is copied");
    let want = egui::pos2((6.5 + at[0] - grab_doc[0]) as f32, (11.0 + at[1] - grab_doc[1]) as f32);
    assert!((r.center() - want).length() <= 1.5, "{r:?} around {want:?}");
    assert_eq!(h.state().session.documents()[1].doc.layers.len(), 2);
    // Source layer keeps its pixels intact (not cut)
    let src_bounds = source_square(&h, paint);
    assert_eq!(src_bounds, photocraft_geom::Rect::new(4, 6, 14, 16), "source remains intact");
}

#[test]
/// Dragging a mask selection onto a tab copies the mask gray and leaves the layer's pixels.
fn a_move_tool_mask_selection_drag_onto_a_tab_copies_the_mask_and_leaves_layer_pixels() {
    let (mut h, paint) = harness();
    h.state_mut().session.execute("layer.layerMask.revealAll", json!({})).unwrap();
    h.state_mut().session.execute("select.rect", json!({"x": 4, "y": 6, "width": 5, "height": 10})).unwrap();
    h.state_mut().session.execute("edit.fill", json!({"color": "#808080", "target": "mask"})).unwrap();
    h.state_mut().ui.mask_target = true;
    h.state_mut().ui.tool = Tool::Move;
    h.run_steps(2);

    // Snapshot the source mask and selection: arming the transfer and copying the selected
    // pixels must leave both unchanged.
    let src_doc = h.state().session.active().unwrap().doc.id;
    let src_layer = h.state().session.active().unwrap().active_layer;
    let mask_sample = |h: &Harness<'static, PhotocraftApp>| {
        let st = h.state().session.documents().iter().find(|s| s.doc.id == src_doc).unwrap();
        let layer = src_layer.and_then(|id| st.doc.layer(id)).unwrap();
        let mut px = [0.0f32; 4];
        layer.surface().unwrap().read_pixel(6, 8, &mut px);
        px
    };
    let selection_present =
        |h: &Harness<'static, PhotocraftApp>| h.state().session.documents().iter().find(|s| s.doc.id == src_doc).is_some_and(|s| s.doc.selection.is_some());
    let before_mask = mask_sample(&h);
    let before_selection = selection_present(&h);

    let grab = ViewXform::active(h.state()).unwrap().to_screen(6.0, 11.0);
    let over_tab = tab(&h, "dest-doc");
    let drop = h.state().last_canvas_rect.center();
    drag_along(&mut h, &[grab, over_tab, drop], Modifiers::NONE);

    let (_, r) = copy(&h);
    assert_eq!(r.size(), vec2(5.0, 10.0), "only the 5x10 selected mask region is copied");
    let st = h.state().session.active().unwrap();
    let dst_layer = st.active_layer.and_then(|id| st.doc.layer(id)).unwrap();
    let mut sample = [0.0f32; 4];
    dst_layer.surface().unwrap().read_pixel(r.min.x as i32 + 1, r.min.y as i32 + 1, &mut sample);
    assert!((sample[0] - sample[1]).abs() < 0.02 && (sample[0] - 128.0 / 255.0).abs() < 0.02, "copied mask gray, not red layer pixels: {sample:?}");
    assert_eq!(source_square(&h, paint), photocraft_geom::Rect::new(4, 6, 14, 16), "source layer pixels remain intact");
    assert_eq!(mask_sample(&h), before_mask, "arming and copying leave the source mask unchanged");
    assert_eq!(selection_present(&h), before_selection, "arming and copying leave the source selection unchanged");
}

#[test]
/// In Arrange views, dragging a selection into the neighbouring cell transfers it there directly.
fn a_move_tool_selection_drag_in_arranged_view_transfers_directly_to_neighbour_cell() {
    let (mut h, paint) = harness();
    h.state_mut().ui.view.arrange = "twoUpVertical".into();
    h.run_steps(4);

    let cells = crate::canvas::arranged_cells(h.state(), h.state().last_canvas_rect).unwrap();
    assert_eq!(cells.len(), 2);
    let (src_cell, dest_cell) = (cells[0].1, cells[1].1);

    // Create selection on source
    h.state_mut().session.execute("select.rect", json!({"x": 4, "y": 6, "width": 5, "height": 10})).unwrap();
    h.state_mut().ui.tool = Tool::Move;
    h.run_steps(2);

    let view_src = h.state().ui.views[0].clone();
    let xf_src =
        crate::canvas::ViewXform { rect: src_cell, zoom: view_src.zoom / h.state().ppp, center: view_src.center, flip: false, rotation: 0.0, aspect: 1.0 };
    let grab = xf_src.to_screen(6.0, 11.0);
    let grab_doc = xf_src.to_doc(grab);
    let view_dest = h.state().ui.views[1].clone();
    let xf_dest = crate::canvas::ViewXform {
        rect: crate::rulers::content_rect(h.state(), dest_cell),
        zoom: view_dest.zoom / h.state().canvas_ppp(),
        center: view_dest.center,
        flip: false,
        rotation: 0.0,
        aspect: 1.0,
    };
    let drop = dest_cell.center();
    let at = xf_dest.to_doc(drop);

    // Drag directly from src canvas to dest canvas (no tab hover!)
    drag_along(&mut h, &[grab, drop], Modifiers::NONE);

    // Destination now has the transferred 5x10 selection layer at the drop point!
    let (_, r) = copy(&h);
    assert_eq!(r.size(), vec2(5.0, 10.0), "only the 5x10 selected region is copied");
    let want = egui::pos2((6.5 + at[0] - grab_doc[0]) as f32, (11.0 + at[1] - grab_doc[1]) as f32);
    assert!((r.center() - want).length() <= 1.5, "{r:?} around {want:?}");
    assert_eq!(h.state().session.documents()[1].doc.layers.len(), 2);
    // Source layer keeps its pixels intact (not cut)
    let src_bounds = source_square(&h, paint);
    assert_eq!(src_bounds, photocraft_geom::Rect::new(4, 6, 14, 16), "source remains intact");
}

#[test]
/// In Arrange views, dragging the Background into the neighbouring cell transfers it there directly.
fn a_move_tool_background_layer_drag_in_arranged_view_transfers_directly_to_neighbour_cell() {
    let (mut h, paint) = harness();
    h.state_mut().session.execute("layer.delete", json!({"layer": paint.0})).unwrap();
    assert_eq!(h.state().session.documents()[0].doc.layers.len(), 1, "only Background remains in source");

    h.state_mut().ui.view.arrange = "twoUpVertical".into();
    h.state_mut().ui.tool = Tool::Move;
    h.state_mut().ui.tool_options.move_auto_select = true;
    h.run_steps(4);

    let cells = crate::canvas::arranged_cells(h.state(), h.state().last_canvas_rect).unwrap();
    assert_eq!(cells.len(), 2);
    let (src_cell, dest_cell) = (cells[0].1, cells[1].1);

    let view_src = h.state().ui.views[0].clone();
    let xf_src =
        crate::canvas::ViewXform { rect: src_cell, zoom: view_src.zoom / h.state().ppp, center: view_src.center, flip: false, rotation: 0.0, aspect: 1.0 };
    let grab = xf_src.to_screen(15.0, 15.0);
    let drop = dest_cell.center();

    // Drag Background directly from src canvas to dest canvas
    drag_along(&mut h, &[grab, drop], Modifiers::NONE);

    // Destination now has the transferred Background layer copied as an ordinary layer!
    assert_eq!(h.state().session.documents()[1].doc.layers.len(), 2);
    let (name, _) = copy(&h);
    assert_eq!(name, "Layer 1");
    // Source still has its Background intact
    assert_eq!(h.state().session.documents()[0].doc.layers.len(), 1);
}

#[test]
/// A blocked Move drag inside one document shows the lock message even with two documents open.
fn same_document_blocked_move_with_multiple_documents_shows_lock_message() {
    let (mut h, paint) = harness();
    h.state_mut().session.execute("layer.delete", json!({"layer": paint.0})).unwrap();
    assert_eq!(h.state().session.documents().len(), 2, "two documents open");
    h.state_mut().ui.tool = Tool::Move;
    h.state_mut().ui.tool_options.move_auto_select = false;

    use crate::canvas::{ToolEvent, tool_event};
    // Click on Background: no message
    tool_event(h.state_mut(), ToolEvent::Down { x: 20.0, y: 20.0, pressure: 1.0 }, Modifiers::NONE);
    tool_event(h.state_mut(), ToolEvent::Up { x: 20.0, y: 20.0 }, Modifiers::NONE);
    let error_dialogs = |app: &crate::PhotocraftApp| -> Vec<String> {
        app.ui
            .dialogs
            .iter()
            .filter(|d| d.kind == crate::state::DialogKind::Error)
            .filter_map(|d| d.fields.get("message")?.as_str().map(String::from))
            .collect()
    };
    assert!(error_dialogs(h.state()).is_empty() && !h.state().move_blocked);

    // Drag and return to press point: shows lock prompt promptly on Move
    tool_event(h.state_mut(), ToolEvent::Down { x: 20.0, y: 20.0, pressure: 1.0 }, Modifiers::NONE);
    tool_event(h.state_mut(), ToolEvent::Move { x: 30.0, y: 20.0, pressure: 1.0 }, Modifiers::NONE);
    // Even if pointer returns to start before Up, the prompt is shown
    tool_event(h.state_mut(), ToolEvent::Move { x: 20.0, y: 20.0, pressure: 1.0 }, Modifiers::NONE);
    tool_event(h.state_mut(), ToolEvent::Up { x: 20.0, y: 20.0 }, Modifiers::NONE);
    assert_eq!(error_dialogs(h.state()), [crate::move_lock::MESSAGE]);
    assert!(!h.state().move_blocked && h.state().move_blocked_start.is_none());
}

#[test]
/// Releasing a blocked Move drag clears the blocked state.
fn blocked_move_gesture_release_clears_blocked_state() {
    let (mut h, _) = harness();
    h.state_mut().ui.tool = Tool::Move;
    h.state_mut().move_blocked = true;
    h.state_mut().move_blocked_start = Some([10.0, 10.0]);

    // When the mouse button is not down or released, finish clears the blocked move state
    let ctx = h.ctx.clone();
    crate::layer_transfer::finish(h.state_mut(), &ctx);
    assert!(!h.state().move_blocked);
    assert_eq!(h.state().move_blocked_start, None);
}

#[test]
/// A gesture begun in another document window survives the main window's `finish`.
fn extra_window_move_gesture_preserved_when_main_window_finishes() {
    let (mut h, _) = harness();
    let main_ctx = h.ctx.clone();
    let extra_vid = egui::ViewportId::from_hash_of("docwin-1");

    h.state_mut().ui.tool = Tool::Move;
    h.state_mut().move_blocked = true;
    h.state_mut().move_blocked_start = Some([10.0, 10.0]);
    crate::tools::move_box::begin_with_layer(h.state_mut(), [10.0, 10.0], false, Some(photocraft_doc::LayerId(1)));
    h.state_mut().move_viewport = Some(extra_vid);

    // When the main window's finish runs, the extra window's gesture is preserved
    crate::layer_transfer::finish(h.state_mut(), &main_ctx);
    assert!(h.state().move_blocked);
    assert_eq!(h.state().move_blocked_start, Some([10.0, 10.0]));
    assert!(h.state().layer_box.is_some());

    // When the gesture's viewport finishes, the state is cleared
    let extra_ctx = egui::Context::default();
    let mut raw = egui::RawInput { viewport_id: extra_vid, ..Default::default() };
    raw.viewports.insert(extra_vid, egui::ViewportInfo::default());
    let mut out = extra_ctx.run_ui(raw, |ui| {
        crate::layer_transfer::finish(h.state_mut(), ui.ctx());
    });
    out.textures_delta.clear();
    assert!(!h.state().move_blocked);
    assert_eq!(h.state().move_blocked_start, None);
    assert!(h.state().layer_box.is_none());
}

/// #11971 review: a document window's `finish` must not complete a transfer the main window
/// armed. The transfer lives in the shared `Context`, so a second viewport of the same context
/// sees it; only the viewport that armed it may complete it.
#[test]
fn a_document_window_does_not_complete_a_transfer_armed_in_the_main_window() {
    let (mut h, paint) = harness();
    // One context, two viewports: the main window arms the transfer, the document window's
    // `finish` runs with its own (idle) pointer state.
    let ctx = egui::Context::default();
    let extra_vid = egui::ViewportId::from_hash_of("docwin-transfer");

    crate::layer_transfer::begin_from_panel(h.state_mut(), &ctx, paint);
    assert!(crate::layer_transfer::is_transfer_armed(&ctx), "the main window armed a transfer");

    // The document window finishes first: its pointer is up, but the transfer belongs to the
    // main window, so it must stay armed.
    let mut raw = egui::RawInput { viewport_id: extra_vid, ..Default::default() };
    raw.viewports.insert(extra_vid, egui::ViewportInfo::default());
    let mut out = ctx.run_ui(raw, |ui| {
        crate::layer_transfer::finish(h.state_mut(), ui.ctx());
    });
    out.textures_delta.clear();
    assert!(crate::layer_transfer::is_transfer_armed(&ctx), "the main window's transfer survives another viewport's finish");

    // The main window's own release completes it.
    let mut raw = egui::RawInput::default();
    raw.events.push(egui::Event::PointerButton {
        pos: egui::Pos2::new(20.0, 20.0),
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: Modifiers::NONE,
    });
    let mut out = ctx.run_ui(raw, |ui| {
        crate::layer_transfer::finish(h.state_mut(), ui.ctx());
    });
    out.textures_delta.clear();
    assert!(!crate::layer_transfer::is_transfer_armed(&ctx), "the main window completes its transfer");
}

/// #11971 review: releasing the secondary button during a locked-layer drag keeps the marquee,
/// so the primary release still selects the layers it covers.
#[test]
fn a_secondary_button_release_keeps_the_marquee_until_the_primary_release() {
    let (mut h, paint) = harness();
    h.state_mut().ui.tool = Tool::Move;
    h.state_mut().ui.tool_options.move_auto_select = false;

    // A marquee over a locked layer, as a blocked Move drag leaves it.
    crate::tools::move_box::begin_with_layer(h.state_mut(), [20.0, 20.0], false, Some(paint));
    h.state_mut().move_blocked = true;
    h.state_mut().move_blocked_start = Some([20.0, 20.0]);

    // The secondary button is released while the primary button is still down.
    let ctx = h.ctx.clone();
    let mut raw = egui::RawInput::default();
    raw.events.push(egui::Event::PointerButton {
        pos: egui::Pos2::new(20.0, 20.0),
        button: egui::PointerButton::Secondary,
        pressed: false,
        modifiers: Modifiers::NONE,
    });
    raw.events.push(egui::Event::PointerButton {
        pos: egui::Pos2::new(20.0, 20.0),
        button: egui::PointerButton::Primary,
        pressed: true,
        modifiers: Modifiers::NONE,
    });
    let mut out = ctx.run_ui(raw, |ui| {
        crate::layer_transfer::finish(h.state_mut(), ui.ctx());
    });
    out.textures_delta.clear();

    assert!(h.state().layer_box.is_some(), "the marquee survives a secondary-button release");
    assert!(h.state().move_blocked, "the blocked move survives a secondary-button release");
    assert_eq!(h.state().move_blocked_start, Some([20.0, 20.0]));
}

/// #11971 review: a secondary-button release while the drag is held must not complete a
/// cross-document transfer; only the primary release does.
#[test]
fn a_secondary_button_release_does_not_complete_a_transfer() {
    let (mut h, paint) = harness();
    let ctx = egui::Context::default();

    crate::layer_transfer::begin_from_panel(h.state_mut(), &ctx, paint);
    assert!(crate::layer_transfer::is_transfer_armed(&ctx), "the window armed a transfer");

    // The secondary button is released while the primary button is still down.
    let mut raw = egui::RawInput::default();
    raw.events.push(egui::Event::PointerButton {
        pos: egui::Pos2::new(20.0, 20.0),
        button: egui::PointerButton::Secondary,
        pressed: false,
        modifiers: Modifiers::NONE,
    });
    raw.events.push(egui::Event::PointerButton {
        pos: egui::Pos2::new(20.0, 20.0),
        button: egui::PointerButton::Primary,
        pressed: true,
        modifiers: Modifiers::NONE,
    });
    let mut out = ctx.run_ui(raw, |ui| {
        crate::layer_transfer::finish(h.state_mut(), ui.ctx());
    });
    out.textures_delta.clear();

    assert!(crate::layer_transfer::is_transfer_armed(&ctx), "a secondary release leaves the transfer armed for the intended primary drop");
}
```

**Explanation.** Ten tests are appended after the last existing one (a panel drag that leaves the layer counts `(2, 1)` and the source active). Each is described below with its setup, its gesture and what it proves.

1. `a_move_tool_selection_drag_onto_a_tab_copies_selected_pixels_to_destination` - Setup: `select.rect` at x 4, y 6, 5 x 10 inside the red square, tool Move, two frames so the view is laid out. Gesture: `drag_along` from the screen point of document (6, 11) to the "dest-doc" tab's rect, then to the canvas centre, no modifiers. Assertions: the copied layer's size is exactly `vec2(5.0, 10.0)` (only the selection travelled); its centre is within 1.5 px of `(6.5 + drop_doc.x - grab_doc.x, 11.0 + drop_doc.y - grab_doc.y)` (the grab offset applied in document space, with a tolerance for the two transforms); the destination has two layers; the source square's bounds are still `(4, 6, 14, 16)`. The last assertion is the non-destructive promise: a copy, not a cut.

2. `a_move_tool_mask_selection_drag_onto_a_tab_copies_the_mask_and_leaves_layer_pixels` - Setup adds `layer.layerMask.revealAll`, fills the selection with `#808080` in the mask, and sets `ui.mask_target = true`. Before the drag it captures the source document id and layer id, a closure that reads one mask pixel at (6, 8) from that document (not the active one, which changes during the drag), and a closure that reports whether that document still has a selection. Gesture as above. Assertions: size 5 x 10; the destination layer's pixel at the copied bounds is grey (channels equal, within 0.02 of 128/255) rather than red - the only observable difference between a mask copy and a pixel copy, since the bounds are identical; the source layer's pixels are unchanged; the source mask pixel equals the value captured before; the source selection still exists. The last two would catch a regression where arming the transfer consumed the mask or cleared the selection.

3. `a_move_tool_selection_drag_in_arranged_view_transfers_directly_to_neighbour_cell` - Setup: `arrange = "twoUpVertical"`, four frames so `arranged_cells` returns two rects, a selection, tool Move. It builds a `ViewXform` for the source cell and another for the destination cell, computes the grab's screen point from the source transform and the drop point as the destination cell's centre, and drags with only two points - grab and drop, no tab hover. That is the point of the test: the transfer must arm from the cell crossing alone, through `check_arranged_drag`, not through a tab. Assertions: size 5 x 10 and the centre matches the destination transform's mapping of the drop point.

4. `a_move_tool_background_layer_drag_in_arranged_view_transfers_directly_to_neighbour_cell` - Same setup, but the paint layer is deleted first, so the only layer is the locked Background. That is the case where the press is refused by `move_lock::blocked`, so there is no `app.drag`; the gesture exists only as `move_blocked_start` and `layer_box.grabbed_layer`. The test proves `current_move_drag`'s second branch promotes it and the transfer arms. Without that branch the drag would do nothing at all.

5. `same_document_blocked_move_with_multiple_documents_shows_lock_message` - Two documents, paint deleted, Move tool with `move_auto_select = false`. The drag stays inside one document. It asserts the lock message appears - the rule that a blocked drag inside the source document must still explain itself, which is the behaviour that must not be suppressed by the cross-document handling.

6. `blocked_move_gesture_release_clears_blocked_state` - Sets `move_blocked` and `move_blocked_start` directly, runs `finish`, and asserts both are cleared. It pins the cleanup branch: a release with no transfer and no button down must not leave blocked state behind.

7. `extra_window_move_gesture_preserved_when_main_window_finishes` - Sets `move_blocked`, `move_blocked_start`, a `layer_box` with a grabbed layer, and `move_viewport = extra_vid`. Runs `finish` with the main context: the gesture must survive (flag, start and marquee all still present). Then runs `finish` inside a second context whose `RawInput` carries `viewport_id: extra_vid` and whose `viewports` map contains that id: the state must clear. This is the direct test of the `move_viewport` guard in the cleanup block.

8. `a_document_window_does_not_complete_a_transfer_armed_in_the_main_window` - Arms a transfer with `begin_from_panel` in a fresh `Context`, asserts it is armed, runs `finish` in the extra viewport (pointer up, but not the owner): the transfer must stay armed. Then runs `finish` in the main viewport with a primary release: the transfer must clear. This is the test of the `t.viewport != ctx.viewport_id()` guard, as opposed to test 7's cleanup guard.

9. `a_secondary_button_release_keeps_the_marquee_until_the_primary_release` - Builds a marquee over a locked layer, sets `move_blocked`, then feeds a `RawInput` with a secondary release and a primary press in the same frame, and runs `finish`. Asserts the marquee and the blocked state survive. Before the change `any_released` would have cleared them; the test is the negative half of the `primary_released` change.

10. `a_secondary_button_release_does_not_complete_a_transfer` - Same input shape, but with a transfer armed: a secondary release while the primary is held must leave the transfer armed. This was verified in the negative: reverting the second pointer read to `any_released` makes this test fail, so it detects the regression rather than merely passing alongside the fix.

Together the ten cover the three ownership rules (viewport, button, cleanup), the three arming paths (panel, tab, cell) and the four content cases (selection, mask selection, layer, Background), which is the full matrix the feature has.

### `crates/ui-egui/src/tools/move_box.rs`

#### 54. `crates/ui-egui/src/tools/move_box.rs` — hunk -12,16 +12,24

**Original code (upstream `main`)**

```rust
/// The marquee being dragged, in document pixels.
#[derive(Clone, Copy, Debug)]
pub(crate) struct LayerBox {
    start: [f64; 2],
    end: [f64; 2],
    /// ⇧ at the press: add to the layer selection.
    add: bool,
}

/// A Move-tool press that Auto-Select found nothing under: start the marquee at `p`. The press
/// started a Move gesture's snapping and ⇧/⌥ handling, which a marquee has none of.
pub(crate) fn begin(app: &mut PhotocraftApp, p: [f64; 2], add: bool) {
    app.layer_box = Some(LayerBox { start: p, end: p, add });
    app.prefs_rt.snap = None;
    app.prefs_rt.snap_lines.clear();
    app.move_mods = Default::default();
```

**Changed code (current `main`)**

```rust
/// The marquee being dragged, in document pixels.
#[derive(Clone, Copy, Debug)]
pub(crate) struct LayerBox {
    pub(crate) start: [f64; 2],
    pub(crate) end: [f64; 2],
    /// ⇧ at the press: add to the layer selection.
    add: bool,
    pub(crate) grabbed_layer: Option<photocraft_doc::LayerId>,
}

/// A Move-tool press that Auto-Select found nothing under: start the marquee at `p`. The press
/// started a Move gesture's snapping and ⇧/⌥ handling, which a marquee has none of.
pub(crate) fn begin(app: &mut PhotocraftApp, p: [f64; 2], add: bool) {
    begin_with_layer(app, p, add, None);
}

/// Starts a Move marquee drag at p, optionally recording a locked or background layer that was hit
/// under the pointer for potential cross-document transfer.
pub(crate) fn begin_with_layer(app: &mut PhotocraftApp, p: [f64; 2], add: bool, grabbed_layer: Option<photocraft_doc::LayerId>) {
    app.layer_box = Some(LayerBox { start: p, end: p, add, grabbed_layer });
    app.move_viewport = app.current_viewport;
    app.prefs_rt.snap = None;
    app.prefs_rt.snap_lines.clear();
    app.move_mods = Default::default();
```

**Explanation.** `move_box.rs` draws and tracks the Move tool's marquee - the dashed rectangle a user drags over empty canvas. This hunk widens the struct, splits `begin` into two functions, and records the window the gesture started in.

The struct `LayerBox` gains `pub(crate)` on `start` and `end`, and a new field: `pub(crate) grabbed_layer: Option<photocraft_doc::LayerId>`. The doc comment on the struct is unchanged.

`start` and `end` become `pub(crate)` because `layer_transfer::current_move_drag` needs to read them: the gesture-level `MoveDrag::Layers([f64; 2])` is built from `b.start` when a marquee is promoted into a transfer. Before this change the fields were private to `move_box` and only the module's own functions read them. Making them `pub(crate)` rather than adding an accessor follows the file's style - the struct is `pub(crate)` and the crate is the boundary.

`grabbed_layer: Option<LayerId>` is the new information: the layer under the pointer at the moment the press was refused. It exists because the two arming paths need different data. A canvas drag on an unlocked layer produces an `app.drag` (a real move) and the drag's `start` is enough. A press on a locked layer or on the Background is *refused* - `move_lock::blocked` fires, no `app.drag` is created - so there is no gesture object to read, and without this field the promotion in `current_move_drag` would have a start point but no layer to copy. The field therefore carries what the refused press learned and would otherwise be discarded.

The type is `LayerId` rather than `u64` here, unlike `TransferKind::Layers(Vec<u64>)`. The difference is the lifetime: `grabbed_layer` never crosses into JSON, it is only read by `arm_move_drag` and passed to `begin_layers`, which takes a `LayerId`. Keeping the newtype means no conversion at either end.

`begin` keeps its signature and body-by-delegation: `pub(crate) fn begin(app, p, add) { begin_with_layer(app, p, add, None) }`. Every existing caller (`tool_input.rs:294` for the ordinary empty-canvas press) keeps working unchanged, and the `None` says "nothing was grabbed" - which is the truth for a press that found no layer under the pointer. Splitting rather than adding a parameter to `begin` keeps the four existing call sites out of the diff.

`begin_with_layer(app, p, add, grabbed_layer)` is the real body: `app.layer_box = Some(LayerBox { start: p, end: p, add, grabbed_layer });` then `app.move_viewport = app.current_viewport;` then the three pre-existing resets (`snap = None`, `snap_lines.clear()`, `move_mods = Default::default()`).

`app.move_viewport = app.current_viewport` is the second half of the viewport guard. `current_viewport` is set once per canvas draw (hunk 16's field, written in `canvas.rs`), so at the moment of a press it names the window the user is in. Recording it here - rather than passing the context down - is what lets the later `finish` compare without threading a `ctx` through the marquee's event handling: `move_box::pointer` and `pointer`-driven `Event` handling have no context parameter.

The doc comment on `begin_with_layer` is new and states the purpose in terms of the feature: "optionally recording a locked or background layer that was hit under the pointer for potential cross-document transfer". That wording is what a reader needs, because the field's name alone (`grabbed_layer`) could be read as "the layer being moved", which it is not - it is the layer a refused press landed on.

The resets after the assignment are pre-existing and their comments explain them: the press started a Move gesture's snapping and ⇧/⌥ handling, which a marquee has none of. `begin_with_layer` keeps them in the same order so that a reader comparing the two functions sees only the additions.

#### 55. `crates/ui-egui/src/tools/move_box.rs` — hunk -40,6 +48,7

**Original code (upstream `main`)**

```rust
        // A press never continues a marquee (one left over from a lost release).
        ToolEvent::Down { .. } => {
            app.layer_box = None;
            false
        }
        ToolEvent::Move { x, y, .. } => {
```

**Changed code (current `main`)**

```rust
        // A press never continues a marquee (one left over from a lost release).
        ToolEvent::Down { .. } => {
            app.layer_box = None;
            app.move_viewport = None;
            false
        }
        ToolEvent::Move { x, y, .. } => {
```

**Explanation.** One line added to the `ToolEvent::Down` arm of the marquee's event handler: `app.move_viewport = None;`.

The arm's existing body clears the marquee and returns `false`:

```rust
ToolEvent::Down { .. } => { app.layer_box = None; false }
```

The comment above it explains why a press clears rather than continues: "A press never continues a marquee (one left over from a lost release)". A lost release (the pointer left the window, another window took focus) leaves a `layer_box` behind, and the next press must not resume that rectangle from the stale start.

`app.move_viewport = None` is the same cleanup for the new field. The two are always set together - `begin_with_layer` sets both, this arm clears both - so the invariant is "a `layer_box` exists if and only if a `move_viewport` is recorded for it".

Why clear on `Down` rather than only on `Up`: the same lost-release hazard. If a release is lost, `Up` never runs, so the `move_viewport` from the abandoned gesture would survive into the next press. `finish` uses `move_viewport` to decide whether a release in *this* window may complete a transfer, so a stale value would let a window that never armed anything believe it owns the gesture. Clearing on the next press closes that window between the lost release and the next interaction.

The return value `false` is unchanged: it means "the marquee did not consume the press", which lets the caller continue with its own handling. Adding the field clear does not change the arm's contract, only its side effects, so no caller needs to know.

The line is unconditional - it runs on every `Down`, including a `Down` that is starting a new marquee. That is correct rather than wasteful: `begin_with_layer` sets the value immediately afterwards, in the same event, so the clear-then-set sequence is what keeps the *previous* gesture's value from surviving the one frame between the two calls. Ordering the other way (set then clear) would be a bug; the file's structure makes the order visible because the press path calls `begin_with_layer` from `tool_input.rs` after the `ToolEvent::Down` dispatch.

Nothing else in the arm changes: `app.layer_box = None` keeps its position first, so the diff is one insertion and the reader sees the two clears adjacent, which is the invariant.

The three clear sites and the two set sites are the whole invariant, and each is one line in a different module. `begin_with_layer` (tools/move_box.rs) sets both when a marquee starts; the blocked-press branch (canvas/tool_input.rs) sets both when a press is refused; `move_box`'s `Down` and `Up` arms and `layer_transfer::arm_move_drag` and `layer_transfer::finish` clear them. Because the set and clear sites are in five modules, the invariant cannot be enforced by the type system - a single field instead of two would not express the difference between "where am I" and "who started this", so the two fields are necessary and the discipline is what keeps them paired.

#### 56. `crates/ui-egui/src/tools/move_box.rs` — hunk -49,6 +58,7

**Original code (upstream `main`)**

```rust
        ToolEvent::Up { x, y } => {
            let b = LayerBox { end: [x, y], ..*b };
            app.layer_box = None;
            let (w, h) = (b.end[0] - b.start[0], b.end[1] - b.start[1]);
            if w == 0.0 && h == 0.0 && !b.add {
                // A click on the pasteboard clears the layer target; a drag from there still
```

**Changed code (current `main`)**

```rust
        ToolEvent::Up { x, y } => {
            let b = LayerBox { end: [x, y], ..*b };
            app.layer_box = None;
            app.move_viewport = None;
            let (w, h) = (b.end[0] - b.start[0], b.end[1] - b.start[1]);
            if w == 0.0 && h == 0.0 && !b.add {
                // A click on the pasteboard clears the layer target; a drag from there still
```

**Explanation.** The same one-line addition in the `ToolEvent::Up` arm: `app.move_viewport = None;`, placed after `app.layer_box = None;` and before the width/height computation.

The arm's existing body takes the stored box, replaces its end with the release point, clears the box, and then decides what the gesture meant:

```rust
ToolEvent::Up { x, y } => {
    let b = LayerBox { end: [x, y], ..*b };
    app.layer_box = None;
    let (w, h) = (b.end[0] - b.start[0], b.end[1] - b.start[1]);
    if w == 0.0 && h == 0.0 && !b.add { ... }
```

`LayerBox` is `Copy`, so `..*b` builds the completed box from the stored one without moving it; that is why the clear can happen before the geometry is read and the local `b` remains valid. The new line sits in that same gap, between the clear and the arithmetic, so both cleared fields are cleared together at the one point where the gesture is known to be over.

Why clear here at all, given `finish` also clears `move_viewport`: `finish` clears it on the release that completes or abandons a transfer, but the marquee's `Up` is the release that ends a *marquee* gesture, which may have nothing to do with a transfer (a drag over empty canvas that the user releases inside the same document). That gesture still set `move_viewport` in `begin_with_layer`, so it must clear it. Leaving it set would let a later release in another window be judged against this window's stale id.

The `w == 0.0 && h == 0.0 && !b.add` branch below is the click case: a zero-size marquee without ⇧ clears the layer target (the comment continues "a click on the pasteboard clears the layer target; a drag from there still..."). That branch is unchanged and it does not need the viewport value - which is why the clear can happen before it. If the branch needed to know the window, the clear would have to move after it; it does not, and putting the clear before the arithmetic means the field is cleared on every path through the arm, including the early `return`-shaped ones below.

The invariant this hunk completes is symmetric: `begin_with_layer` sets `layer_box` and `move_viewport` together; `Down` clears both; `Up` clears both; `arm_move_drag` clears both when it promotes the gesture into a transfer; `finish` clears both on the release that ends a transfer. Five sites, and no sixth path can leave one set and the other clear - which is what the tests in hunk 23 check indirectly, by asserting that a release in the wrong window does not complete anything.

No behaviour is added by this line on its own; it is the half of the guard that makes the other half (the comparison in `finish` and `check_arranged_drag`) reliable.

One thing this arm does not do is notify the transfer: if a release both ends a marquee and completes a transfer, the marquee's `Up` clears `move_viewport` *after* `layer_transfer::finish` has already read it, because `finish` runs at the end of the frame (canvas/tabs.rs) while the marquee's `Up` runs during input handling. The order is: input events, marquee `Up` (clears the fields), then the frame's `finish` - which sees no transfer to complete, because the transfer is stored in the context and is not cleared by the marquee. So the two states are independent, and each clears its own.
