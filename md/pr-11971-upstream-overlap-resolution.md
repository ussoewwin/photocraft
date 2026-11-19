# PR #11971: resolving the overlap with the official upstream

Scope: `feat/move-tool-selection-drag-transfer` (#11971) against `upstream/main`.
Goal: leave the PR as **the official code plus only the behaviour upstream still lacks** — eliminate all
avoidable relocation churn and duplicate helpers while preserving every behavioural change required by the
PR's test suite.

This document is the audited plan and ledger. Every number in it was measured and verified against the live
tree on 2026-11-18; the commands that produced them are in §7 so the next run can reproduce them.

---

## 1. Measured starting point

| Item | Measured value |
|---|---|
| `upstream/main` | `7cbc2b85e3` — "Build the craft UI framework below opt-level 2 in test builds (#15565)" (2026-11-18 14:13 +0700) |
| PR branch | `origin/feat/move-tool-selection-drag-transfer` = `7e12ff2dd9` — "ui-egui: restore move box multi-window guards, transfer fallback, and document window cleanup (#11971)" |
| Merge base | `01a5b5a389` (#15449, "PhotoCraft on the craft UI framework") |
| `git diff --stat upstream/main <PR>` (two-dot) | 15 files, **+1464 / −120** |
| `git diff --stat upstream/main...<PR>` (three-dot) | 13 files, **+1464 / −50** |
| Deletions outside `crates/` (two-dot only) | 70 lines — `AGENTS.md` (1) and `Cargo.toml` (69): upstream's newest commit (`7cbc2b85e3`), which the PR branch has not merged yet |
| Deletions inside `crates/` | **50 lines** — 36 lines of avoidable overlap/relocation churn (§4.1) + 14 lines of required in-place extensions (§4.2) |
| Working tree `target\wt-pr11971` | clean, head `7e12ff2dd9` |

The port itself already happened: `be2ec83fdb` ("Keep the #11971 layer-transfer rules through the craft
port"), `220519bcb1`, and `7e12ff2dd9` carried the fork's rules and multi-window guards onto the #15449
framework. What remains is merging upstream's latest commit and eliminating the 36 lines where the PR
unnecessarily moved or rewrote upstream code instead of extending it in place.

---

## 2. What upstream already provides (keep it, do not re-implement or relocate)

| Upstream location | What it provides |
|---|---|
| `crates/ui-egui/src/panels/layer_transfer.rs` | the core transfer UI: `begin`, `arm_move_transfer`, `over_tab`, `finish`, `drop`, `ghost`, `panel_transfer`, `move_drag`, `layer_drag` |
| `crates/engine/src/cmd/layer_copy_cmds.rs:213` | `copy_to_document`, the command the drag dispatches |
| `crates/engine/src/cmd/layer_copy_cmds.rs:240` | `selection_only` — copying the selected pixels of one raster layer |
| `crates/engine/src/cmd/layer_copy_cmds.rs` (params) | `document`, `layers`, `source`, `selectionOnly`, `center`, `at`, `offset`, `name` |
| `crates/engine/src/cmd/float_cmds.rs:48` | `CutParts` with `selected_pixels`, `is_mask`, and `mask` bookkeeping for the floating piece |
| `crates/engine/src/cmd/edit_cmds.rs:208, 458` | `lift` and `clear_selected` helpers |
| `crates/engine/src/cmd/pick_cmds.rs:89-115` | `auto_select_target`, `move_auto_select_target`, and private `pick_target(doc, x, y, group, move_auto_select)` with the lock-skip rule (`!locks.all && (!move_auto_select \|\| !locks.position)`) |
| `crates/ui-egui/src/tools/move_box.rs` | `LayerBox { start, end, add, move_auto_select, clear_on_empty }` and `move_box::begin` |
| `crates/ui-egui/src/canvas/tools/move_tool.rs` | the `Kind::Refused` gesture, `Run::prompted` flag (so the message is shown once), the 4-argument `marquee` closure, and the inline `locked_hit` check |

---

## 3. What only the PR provides (the value to keep on top of upstream's shape)

| PR-only behaviour | Where it lives now | Upstream state |
|---|---|---|
| The refused press is recorded with point, layer, document and window (`record_refused_press`, `move_blocked`, `move_blocked_start`, `move_blocked_moved`, `move_blocked_layer`, `move_blocked_doc`, `move_viewport`) | `crates/ui-egui/src/canvas/tools/move_tool.rs`, `crates/ui-egui/src/app.rs` | `crates/ui-egui/src/app.rs` has **0** hits for `move_blocked` / `move_viewport` |
| A locked layer (position-locked, all-locked, or the Background) under the pointer is the transfer candidate (`layer.pickAt` with `includeLocked: true`, `locked_hit_at`, `locked_layer`) | `crates/ui-egui/src/canvas/tools/move_tool.rs`, `crates/engine/src/cmd/pick_cmds.rs` | `includeLocked`: **0** hits in upstream's `pick_cmds.rs` |
| The lock message waits while the press can still become a transfer (`may_transfer`), and shows on release if the drag stayed in the same document | `crates/ui-egui/src/canvas/tools/move_tool.rs` | absent (upstream prompts immediately on the first drag sample and does nothing on release) |
| Multi-window gesture ownership: the refusal and marquee belong to the window that began them (`move_viewport`), a new press clears stale blocked state from another window, and secondary-button releases do not abort or complete primary drags (`primary_released()`) | `crates/ui-egui/src/panels/layer_transfer.rs`, `crates/ui-egui/src/tools/move_box.rs`, `crates/ui-egui/src/canvas/document_windows.rs` | absent (upstream uses `any_released()` and has no `move_viewport` guard) |
| Copying **or cutting** the selected pixels or the **layer mask** content, with the source's remainder (`selection`, `cut`, and `target: "mask"` params, `CutParts::copied`, `piece()`, `place_rest()`, `clear_selected`) | `crates/engine/src/cmd/layer_copy_cmds.rs`, `crates/engine/src/cmd/float_cmds.rs`, `crates/engine/src/cmd/edit_cmds.rs`, `crates/ui-egui/src/panels/layer_transfer.rs` | upstream has `selectionOnly` (copy raster pixels only; rejects layer masks); `"cut"` param: **0** hits in upstream's `layer_copy_cmds.rs` |
| The grabbed locked/Background layer and source document travel with the marquee (`LayerBox::grabbed_layer`, `LayerBox::doc`) | `crates/ui-egui/src/tools/move_box.rs` | absent |

---

## 4. Audited ledger of the 50 crate-side deletions

In Git's unified diff, modifying an existing line in place (such as widening `fn` to `pub(crate) fn`, adding a
field to a struct constructor literal `Ok(Self { ... })`, or appending parameter docs to a single-line raw
string literal `params: r##"..."##`) always records 1 deletion (`-`) and 1 addition (`+`). Therefore, the 50
crate-side deletions split into two distinct categories:

1. **§4.1 Avoidable overlap & relocation churn (36 lines)** — upstream lines the PR moved, reordered, or
   rewrote unnecessarily. These **must be restored** so the diff only adds on top of upstream.
2. **§4.2 Required in-place extensions & behavioural replacements (14 lines)** — single-line visibility
   widenings, struct literal initializers, `CommandSpec.params` strings, and the `any_released()` →
   `primary_released()` / `selection` mask-transfer fixes. These **must be kept**; reverting them either fails
   to compile or breaks the PR's test suite.

### 4.1 Avoidable overlap & relocation churn to eliminate (36 lines → 0)

| File | Avoidable deletions | Resolution |
|---|---|---|
| `crates/engine/src/cmd/float_cmds.rs` (**4** of 6) | Deleted `CutParts::is_mask` doc comment, body (`self.mask`), closing brace, and blank line at lines 48–52, only to re-add `pub fn is_mask` at line 70; also added a redundant `#[allow(dead_code)]` on `selected_pixels`. | Keep `is_mask` at its original upstream position (lines 48–52), widening only its signature line in place from `pub(crate) fn is_mask(&self) -> bool {` to `pub fn is_mask(&self) -> bool {` (required because `ui-egui`'s `layer_transfer.rs` calls `f.parts.is_mask()`). Remove the duplicate `is_mask` at line 70 and remove `#[allow(dead_code)]` on `selected_pixels`. |
| `crates/engine/src/cmd/layer_copy_cmds.rs` (**2** of 3) | Replaced `let mut l = if selection_only {` by putting `if selection {` first, and replaced the `// Nothing changed: the source stays the active document.` comment above `if edited.is_err() {`. | Keep `let mut l = if selection_only {` first and chain `} else if selection {` after upstream's `selection_only` block. Keep upstream's `// Nothing changed: the source stays the active document.` comment intact above `if edited.is_err() {` and place the pre-cut restoration comment inside the `if edited.is_err()` block. |
| `crates/engine/src/cmd/pick_cmds.rs` (**4** of 7) | Added a 6th parameter `include_locked: bool` directly to `pick_target`, modifying `auto_select_target`, `move_auto_select_target`, `fn pick_target(...)`, and its `!locks.all && ...` line. | Keep upstream's 5-argument `pick_target`, `auto_select_target`, and `move_auto_select_target` 100% untouched. Add a sibling helper `pick_target_with_locked(doc, x, y, group, move_auto_select, include_locked)` (delegating to `pick_target` when `!include_locked`) and call it from `pick()`. |
| `crates/ui-egui/src/canvas/tools/move_tool.rs` (**25** of 25) | Deleted `prompted: bool` and its doc comment from `Run` and `Run::plain` (3); changed the 4-argument `marquee` closure and its `PLAN:` comment (3); moved `if !mods.shift { select.deselectLayers }` outside `if !empty` (6); replaced the inline `let locked_hit = !auto && ...` block (9); replaced `if !std::mem::replace(&mut run.prompted, true)` in `drag` and `Kind::Refused => {}` in `release` (4). | 1. Keep `prompted: bool` on `Run` and `prompted: false` in `Run::plain`, and use `run.prompted` to guard `move_lock::prompt` when `!may_transfer(app)` in `drag` and when releasing a refused press in `release`.<br>2. Keep the 4-argument `marquee` closure and its `PLAN:` comment intact; set `b.grabbed_layer` via a small helper or right after `marquee(...)` creates `app.layer_box`.<br>3. Keep `if !mods.shift { select.deselectLayers }` inside `if !empty` in its exact upstream lines.<br>4. Keep upstream's `let locked_hit = !auto && ...` block intact; compute `transfer_candidate` alongside it and attach `grabbed_layer` when `locked_hit \|\| transfer_candidate.is_some()`.<br>5. In `drag` (`Kind::Refused`), gate upstream's `if !std::mem::replace(&mut run.prompted, true)` on `!may_transfer(app)` while recording `move_blocked_moved`, and in `release` (`Kind::Refused`) run the deferred prompt when `!run.prompted`. |
| `crates/ui-egui/src/panels/layer_transfer.rs` (**1** of 5 + 1 orphan comment) | Expanded the single-line `let t = Transfer { ... };` constructor in `begin()` into 9 lines; also left a dangling half-sentence doc comment at line 320 (`/// The refused Move press ended in this window: no button is held, and the window recorded for`) from the removed `refused_move_ends` helper. | Format `let t = Transfer { ... };` cleanly and delete the orphaned 1-line comment at line 320. |

### 4.2 Required in-place extensions & behavioural fixes to keep (14 lines)

| File | Lines | Why this `-`/`+` modification is required and must NOT be reverted |
|---|---|---|
| `crates/engine/src/cmd/edit_cmds.rs` (**2**) | `-fn lift(...)` → `+pub(crate) fn lift(...)`<br>`-fn clear_selected(...)` → `+pub(crate) fn clear_selected(...)` | `layer_copy_cmds.rs` calls both helpers across modules in `photocraft-engine`. They are already in place in `edit_cmds.rs` (not relocated); widening `fn` to `pub(crate) fn` is the minimal 2-line change. |
| `crates/engine/src/cmd/float_cmds.rs` (**2**) | `-pub(crate) fn is_mask(&self) -> bool {` → `+pub fn is_mask(&self) -> bool {`<br>`-Ok(Self { layer, rest, piece, piece_bounds, outline, mask })` → `+Ok(Self { ..., mask, copied: copy })` | `ui-egui` calls `f.parts.is_mask()` across crates (requiring `pub fn`), and Rust requires `copied` to be initialized in the `CutParts` struct literal. |
| `crates/engine/src/cmd/layer_copy_cmds.rs` (**1**) | `-params: r##"..."##,` → `+params: r##"..."##,` | Adds `"selection"`, `"cut"`, and `"target"` to the single-line `CommandSpec.params` raw string literal while keeping all existing upstream parameter docs intact. |
| `crates/engine/src/cmd/pick_cmds.rs` (**3**) | `-let Some(target) = pick_target(...)`<br>`-if p.get("select")...`<br>`-params: r##"..."##,` | Wires `include_locked` into `pick()`, prevents `includeLocked` probes from selecting `locks.all` layers, and adds `"includeLocked"` to `CommandSpec.params` (which already preserves upstream's `"list":bool=false`). |
| `crates/ui-egui/src/panels/layer_transfer.rs` (**4**) | 3 lines of `i.pointer.any_released()` → `i.pointer.primary_released()` in `over_tab`, `finish`, and `prepare`;<br>1 line `p["selectionOnly"] = json!(t.selection_only);` → `if t.selection_only { p["selection"] = json!(true); ... }` in `drop()` (plus the `Transfer` struct literal in `begin()`). | Reverting `primary_released()` to `any_released()` breaks `a_secondary_button_release_does_not_complete_a_transfer` and `a_secondary_button_release_keeps_the_marquee_until_the_primary_release`. Reverting `drop()`'s `selection`/`target: "mask"` dispatch to `selectionOnly` breaks `a_move_tool_mask_selection_drag_onto_a_tab_copies_the_mask_and_leaves_layer_pixels` because upstream's `selectionOnly` rejects layer masks. |
| `crates/ui-egui/src/tools/move_box.rs` (**2**) | `-start: [f64; 2],` → `+pub(crate) start: [f64; 2],`<br>`-app.layer_box = Some(LayerBox { ... });` → `+app.layer_box = Some(LayerBox { ..., grabbed_layer: None, doc: ... });` | `layer_transfer.rs` reads `b.start`, and Rust requires all fields of `LayerBox` to be initialized in `move_box::begin`. |

Everything the PR adds without deleting upstream lines is already in the required shape and stays as it is:
`crates/ui-egui/src/app.rs` (+21), `crates/ui-egui/src/canvas.rs` (+1),
`crates/ui-egui/src/canvas/document_windows.rs` (+7), `crates/ui-egui/src/lib.rs` (+10), and the two test
files (`crates/ui-egui/src/panels/layer_transfer/tests.rs` +610,
`crates/engine/src/cmd/layer_copy_cmds/tests.rs` +250).

---

## 5. Procedure

1. **Take upstream's newest commit.** Merge `upstream/main` (`7cbc2b85e3`) into the PR branch in
   `target\wt-pr11971`. This clears the 70 `AGENTS.md` / `Cargo.toml` deletions in the two-dot diff and
   keeps the conflict count at 0. Verify with `git diff --stat upstream/main <PR>`: the `AGENTS.md` and
   `Cargo.toml` lines disappear.
2. **Apply the §4.1 cleanups file by file:**
   - `crates/engine/src/cmd/float_cmds.rs`: keep `pub fn is_mask` in its original upstream slot (line 48),
     remove the relocated copy at line 70, and remove `#[allow(dead_code)]` on `selected_pixels`.
   - `crates/engine/src/cmd/layer_copy_cmds.rs`: place `if selection_only` first and chain
     `} else if selection {`, and restore upstream's `// Nothing changed: the source stays the active document.`
     comment above `if edited.is_err()`.
   - `crates/engine/src/cmd/pick_cmds.rs`: restore upstream's 5-argument `pick_target`, `auto_select_target`,
     and `move_auto_select_target` verbatim; add `pick_target_with_locked` for `pick()`.
   - `crates/ui-egui/src/canvas/tools/move_tool.rs`: restore `prompted: bool` on `Run`, restore the
     4-argument `marquee` closure and inner `if !mods.shift` block, and restore upstream's `let locked_hit = ...`
     block while keeping `record_refused_press`, `locked_hit_at`, `may_transfer`, and the deferred refusal
     prompt on release.
   - `crates/ui-egui/src/panels/layer_transfer.rs`: remove the orphaned doc comment at line 320 while keeping
     `primary_released()` and the `selection` / `target: "mask"` dispatch in `drop()`.
3. **Keep the PR's tests as the non-negotiable specification.** Every test in `layer_transfer::tests` (32),
   `move_box::tests` (7), `layer_copy_cmds::tests` (36), `pick_cmds::tests` (13), and `move_tool` (19) must
   pass with zero weakened assertions and zero deleted tests.
4. **Run the gates of §6** before committing and pushing.
5. **Propagate to `main`.** Per the Two-Track Iron Rule (`upstream-merge-procedure.md` §10), merge the
   updated PR branch into `main` so `main` and the PR branch remain 100% content-identical on all touched
   files.

---

## 6. Acceptance gates

| Gate | Expected |
|---|---|
| `cargo test --lib -- layer_transfer move_box` | **39 passed / 0 failed** (`layer_transfer::tests` 32 + `move_box::tests` 7) |
| `cargo test --lib -- layer_copy_cmds` | **36 passed / 0 failed** (`layer_copy_cmds::tests` holds 36 `#[test]` functions) |
| `cargo test --lib -- pick_cmds` | **13 passed / 0 failed** (includes both `includeLocked` tests) |
| `cargo test --lib -- layer_transfer move_tool` | **51 passed / 0 failed** (`layer_transfer` 32 + `move_tool` 19) |
| `cargo xtask layers` | **OK: 26 crates, no layering violations** |
| `cargo fmt --all -- --check` | **exit 0** (clean) |
| `git diff --stat upstream/main <PR>` | 13 files changed (no `AGENTS.md` or `Cargo.toml`), crate-side deletions reduced from 50 to the ~14 required in-place modification lines (§4.2) |

All test commands on Windows run with Visual Studio 2022 `vcvars64.bat`, `CRAFT_I18N_REQUIRED=1`,
`CRAFT_I18N_DIR=D:\USERFILES\GitHub\craft-i18n`, and `CRAFT_FONTS_DIR=D:\USERFILES\GitHub\photocraft\craft-fonts`.

---

## 7. How the measurements were produced

```powershell
Set-Location D:\USERFILES\GitHub\photocraft
git fetch upstream main
git log -1 --format='%H %ci %s' upstream/main
git log -1 --format='%H %s' origin/feat/move-tool-selection-drag-transfer
git merge-base upstream/main origin/feat/move-tool-selection-drag-transfer
git diff --stat upstream/main origin/feat/move-tool-selection-drag-transfer
git diff --stat upstream/main...origin/feat/move-tool-selection-drag-transfer

# Inspect all crate-side `-` lines against their corresponding `+` lines:
git diff upstream/main...origin/feat/move-tool-selection-drag-transfer -- crates/

# Verify exact test counts in each module:
(Select-String -Path crates/engine/src/cmd/layer_copy_cmds/tests.rs -Pattern '#\[test\]').Count
(Select-String -Path crates/ui-egui/src/panels/layer_transfer/tests.rs -Pattern '#\[test\]').Count
(Select-String -Path crates/ui-egui/src/tools/move_box.rs -Pattern '#\[test\]').Count
```

---

## 8. Risks and safeguards

- **Do not revert `primary_released()` to `any_released()` in `layer_transfer.rs`.** Upstream's
  `any_released()` fires when a secondary mouse button is released while the primary button is still held,
  which prematurely drops or cancels the transfer and fails 2 tests.
- **Do not revert `drop()`'s `selection` / `target: "mask"` payload to `selectionOnly` in `layer_transfer.rs`.**
  Upstream's `selectionOnly` only copies raster layer pixels and rejects layer masks, which fails the mask
  selection transfer test.
- **Do not prompt unconditionally on the first drag frame of `Kind::Refused` in `move_tool.rs`.** While
  `may_transfer(app)` is true (multiple documents open and a transferable locked layer or selection), the
  lock refusal dialog must wait until `release()` so dragging onto another tab transfers cleanly without
  popping an error dialog mid-drag.
- **`CutParts::is_mask` must be `pub fn`, not `pub(crate) fn`.** `crates/ui-egui/src/panels/layer_transfer.rs`
  calls `f.parts.is_mask()` across crate boundaries; keeping `is_mask` at its upstream line position in
  `float_cmds.rs` still requires widening `pub(crate)` to `pub`.

---

## 9. Decisions

| Decision | Resolution | Why |
|---|---|---|
| The `selection` / `cut` / `target: "mask"` variant of `layer.copyToDocument` | **Keep it, chained after upstream's `if selection_only`** | Upstream's `selectionOnly` is copy-only for raster pixels; the `selection` variant adds mask transfers and `cut` with source remainder restoration. |
| `lift` and `clear_selected` in `edit_cmds.rs` | **Keep in place as `pub(crate) fn`** | Already in `edit_cmds.rs` (never relocated); the 2 `-`/`+` lines are simply the `pub(crate)` visibility widening. |
| `CutParts::is_mask` in `float_cmds.rs` | **Keep at upstream line 48 as `pub fn`** | Eliminates the 4-line deletion/relocation while allowing cross-crate call from `ui-egui`. |
| Upstream's 5-argument `pick_target` in `pick_cmds.rs` | **Keep verbatim; add `pick_target_with_locked`** | Leaves `auto_select_target`, `move_auto_select_target`, and `pick_target` completely untouched. |
| `Run::prompted`, `marquee` closure, and `locked_hit` in `move_tool.rs` | **Keep upstream structure; layer PR hooks alongside** | Eliminates ~20 lines of structural diff while preserving deferred lock prompts and locked-layer transfer candidates. |

---

## 10. Definition of done

- `upstream/main` (`7cbc2b85e3`) is merged into `feat/move-tool-selection-drag-transfer`, eliminating the 70
  non-crate deletions (`AGENTS.md`, `Cargo.toml`).
- All 36 lines of avoidable relocation/structural churn in `float_cmds.rs`, `layer_copy_cmds.rs`,
  `pick_cmds.rs`, `move_tool.rs`, and `layer_transfer.rs` (§4.1) are eliminated, leaving only the minimal
  in-place extensions and behavioural fixes (§4.2).
- `layer_transfer` + `move_box` **39/39**, `layer_copy_cmds` **36/36**, `pick_cmds` **13/13**,
  `layer_transfer` + `move_tool` **51/51** all pass.
- `cargo xtask layers` (26 crates, 0 violations) and `cargo fmt --all -- --check` exit 0.
- The PR branch is pushed to `origin/feat/move-tool-selection-drag-transfer`, merged into `main`, and pushed
  to `origin/main`.
