# PR #11971: resolving the overlap with the official upstream

Scope: `feat/move-tool-selection-drag-transfer` (#11971) against `upstream/main`.
Goal: leave the PR as **the official code plus only the behaviour upstream still lacks** — the PR must
delete no upstream line; every upstream line stays and the fork's behaviour sits on top of it.

This document is the plan and the ledger. Every number in it was measured on 2026-11-18; the commands
that produced them are in §7 so the next run can reproduce them.

---

## 1. Measured starting point

| Item | Measured value |
|---|---|
| `upstream/main` | `7cbc2b85e3` — "Build the craft UI framework below opt-level 2 in test builds (#15565)" (2026-11-18 14:13 +0700) |
| PR branch | `origin/feat/move-tool-selection-drag-transfer` = `7e12ff2dd9` — "ui-egui: restore move box multi-window guards, transfer fallback, and document window cleanup (#11971)" |
| Merge base | `01a5b5a389` (#15449, "PhotoCraft on the craft UI framework") |
| `git diff --stat upstream/main <PR>` | 15 files, **+1464 / −120** |
| Deletions outside `crates/` | 70 lines — `AGENTS.md` (1) and `Cargo.toml` (69): upstream's newest commit, which the PR branch does not have yet |
| Deletions inside `crates/` | **50 lines** — the overlap sites (§6, Appendix A) |
| Working tree `target\wt-pr11971` | clean, head `7e12ff2dd9` |

The port itself already happened: `be2ec83fdb` ("Keep the #11971 layer-transfer rules through the craft
port") and `7e12ff2dd9` carried the fork's rules onto the #15449 framework. What remains is the part
where the PR *replaces* upstream lines instead of extending them.

## 2. What upstream already provides (keep it, do not re-implement)

| Upstream location | What it provides |
|---|---|
| `crates/ui-egui/src/panels/layer_transfer.rs` | the whole transfer UI: `begin`, `arm_move_transfer`, `over_tab`, `finish`, `drop`, `ghost`, `panel_transfer`, `move_drag`, `layer_drag` |
| `crates/engine/src/cmd/layer_copy_cmds.rs:213` | `copy_to_document`, the command the drag dispatches |
| `crates/engine/src/cmd/layer_copy_cmds.rs:240` | `selection_only` — copying the selected pixels of one raster layer |
| `crates/engine/src/cmd/layer_copy_cmds.rs` (params) | `document`, `layers`, `source`, `selectionOnly`, `center`, `at`, `offset`, `name` |
| `crates/engine/src/cmd/float_cmds.rs` | `CutParts` with `is_mask`, `mask`, `place`-style bookkeeping for the floating piece |
| `crates/engine/src/cmd/edit_cmds.rs` | private `lift` and `clear_selected` helpers |
| `crates/engine/src/cmd/pick_cmds.rs` | private `pick_target(doc, x, y, group, move_auto_select)` with the lock-skip rule (`!locks.all && (!move_auto_select \|\| !locks.position)`) |
| `crates/ui-egui/src/tools/move_box.rs` | `LayerBox { start, end, add, move_auto_select, clear_on_empty }` and `move_box::begin` |
| `crates/ui-egui/src/canvas/tools/move_tool.rs` | the `Kind::Refused` gesture and its `prompted` flag (the message is shown once) |

## 3. What only the PR provides (the value to land on upstream's shape)

| PR-only behaviour | Where it lives now | Upstream state |
|---|---|---|
| The refused press is recorded with point, layer, document and window (`record_refused_press`, `move_blocked`, `move_blocked_start`, `move_blocked_moved`, `move_blocked_layer`, `move_blocked_doc`, `move_viewport`) | `crates/ui-egui/src/canvas/tools/move_tool.rs`, `crates/ui-egui/src/app.rs` | `crates/ui-egui/src/app.rs` has **0** hits for `move_blocked` / `move_viewport` |
| A locked layer (position-locked, all-locked, or the Background) under the pointer is the transfer candidate (`layer.pickAt` with `includeLocked: true`, `locked_hit_at`, `locked_layer`) | `crates/ui-egui/src/canvas/tools/move_tool.rs`, `crates/engine/src/cmd/pick_cmds.rs` | `includeLocked`: **0** hits in upstream's `pick_cmds.rs` |
| The lock message waits while the press can still become a transfer (`may_transfer`) | `crates/ui-egui/src/canvas/tools/move_tool.rs` | absent |
| The refusal belongs to the window that began it, and a new press clears a refusal left over from a lost release in another window (`refused_move_ends`, `finish()`'s cleanup) | `crates/ui-egui/src/panels/layer_transfer.rs` | absent |
| Copying **or cutting** the selected pixels or the mask content, with the source's remainder (`selection` and `cut` params, `CutParts::copied`, `piece()`, `place_rest()`, `clear_selected`) | `crates/engine/src/cmd/layer_copy_cmds.rs`, `crates/engine/src/cmd/float_cmds.rs`, `crates/engine/src/cmd/edit_cmds.rs` | upstream has `selectionOnly` (copy only); the `cut` param: **0** hits in upstream's `layer_copy_cmds.rs` |
| The grabbed layer travels with the marquee (`LayerBox` gains the layer) | `crates/ui-egui/src/tools/move_box.rs` | absent |

## 4. Ledger: the 50 crate-side deletions, file by file

Each line below is upstream code the PR currently deletes. The action is what the resolution needs.

| File | Upstream lines the PR deletes | Action |
|---|---|---|
| `crates/engine/src/cmd/edit_cmds.rs` (2) | private `fn lift(...)`, `fn clear_selected(...)` | **CLOSE** — upstream has them; the PR relocated copies into `float_cmds.rs` as `pub(crate)`. Keep upstream's and widen the visibility instead of moving them. |
| `crates/engine/src/cmd/float_cmds.rs` (6) | `CutParts::is_mask`, the `mask` field read, the `Ok(Self { … mask })` constructor line | **CLOSE + PORT** — upstream's `is_mask` stays; add only `copied`, `piece()`, `place_rest()` to upstream's `CutParts`. |
| `crates/engine/src/cmd/layer_copy_cmds.rs` (3) | `let mut l = if selection_only {`, the "source stays the active document" comment, the params doc | **PORT** — extend upstream's branch with the `selection` / `cut` variant; keep upstream's params and add the two new ones to the same doc. |
| `crates/engine/src/cmd/pick_cmds.rs` (7) | `pick_target(...)` and its lock-skip rule, the `select` handling, the params doc | **PORT** — keep upstream's `pick_target` and its lock rule; add `includeLocked` as a third variant that defaults to today's behaviour. The params doc must keep upstream's `list` param. |
| `crates/ui-egui/src/canvas/tools/move_tool.rs` (25) | the `prompted` flag, the 4-argument `marquee` closure, `move_box::begin(app, p, add, auto, clear)`, the inline `locked_hit` check, `Kind::Refused => {}` | **PORT** — keep upstream's `prompted` flag and refusal prompt; extend the marquee closure and `move_box::begin` with the grabbed layer instead of replacing them; add `record_refused_press` / `locked_hit_at` / `may_transfer` beside them. |
| `crates/ui-egui/src/panels/layer_transfer.rs` (5) | the `Transfer { … selection_only }` construction, the three release checks, `p["selectionOnly"] = …` | **PORT** — keep upstream's `Transfer` and release logic; add the refusal cleanup (`refused_move_ends` and the `finish()` guard) on top. |
| `crates/ui-egui/src/tools/move_box.rs` (2) | the `LayerBox` construction and the `start` field line | **PORT** — add the grabbed layer as a field of upstream's `LayerBox`. |

Everything the PR adds without deleting upstream is already in the required shape and stays as it is:
`crates/ui-egui/src/app.rs` (21), `crates/ui-egui/src/canvas.rs` (1),
`crates/ui-egui/src/canvas/document_windows.rs` (7), `crates/ui-egui/src/lib.rs` (10), and the two test
files (`crates/ui-egui/src/panels/layer_transfer/tests.rs` 610,
`crates/engine/src/cmd/layer_copy_cmds/tests.rs` 250).

## 5. Procedure

1. **Take upstream's newest commit.** Merge `upstream/main` into the PR branch. This clears the 70
   `AGENTS.md` / `Cargo.toml` deletions and keeps the conflict count at 0. Verify with
   `git diff --stat upstream/main <PR>`: the `AGENTS.md` and `Cargo.toml` lines disappear.
2. **Work file by file, in the order of §4.** For each file: restore the upstream lines the ledger names,
   then re-apply the PR's behaviour on top of them. One file per commit, each commit a reviewable
   behaviour.
3. **Keep the PR's tests as the specification.** `crates/ui-egui/src/panels/layer_transfer/tests.rs`
   (32 tests) and `crates/engine/src/cmd/layer_copy_cmds/tests.rs` (37 tests) must stay green. Where a
   test asserts the PR's *implementation* rather than its *behaviour* (a helper's location, a parameter
   documentation string), adapt the test to the upstream shape and record the adaptation; do not weaken an
   assert and do not delete a test.
4. **Run the gates of §6 after each file**, not only at the end.
5. **Measure the result.** The PR's diff against upstream must show **0 deletions under `crates/`**. That
   number is the definition of "the overlap is resolved": the PR adds, it does not replace.
6. **Push** the branch only after §6 is green, then re-measure the diff and the PR's status.

## 6. Acceptance gates

| Gate | Expected |
|---|---|
| `cargo test --lib -- layer_transfer` | 32 passed / 0 failed (the file holds 32 `#[test]` functions) |
| `cargo test --lib -- layer_copy_cmds` | 37 passed / 0 failed |
| `cargo test --lib -- layer_transfer move_tool` | 51 passed / 0 failed |
| 6-package suite with `--no-fail-fast` | no failure beyond upstream's own baseline: `photocraft-automation` 1 (`rpc::tests::render_to_file_and_base64`, `os error 5`), `photocraft-engine` 2 (`cmd::analysis_cmds::tests::log_delete_and_csv_export`, `cmd::print_cmds::tests::a_thread_holding_the_spooler_does_not_wait_for_itself`), `photocraft-ui-egui` lib 19 plus the stack overflow in `tools::type_tool::pointer::mask_click_text_layer_tests::a_type_mask_click_on_text_edits_that_layer_as_the_type_tool_does` |
| `rustfmt --edition 2024 --check` on the touched files | clean |
| `cargo clippy -p photocraft-ui-egui -p photocraft-engine -p photocraft -p photocraft-cli -p photocraft-text -p photocraft-automation` (lib targets) | exit 0 |
| `git diff --stat upstream/main <PR>` | 0 deletions under `crates/` |

The suite baseline above was measured with `CRAFT_I18N_REQUIRED=1`, `CRAFT_I18N_DIR` pointing at the
Japanese catalog and `CRAFT_FONTS_DIR` at the font checkout, with a dedicated `CARGO_TARGET_DIR`. Without
those the fork-feature suites fail for lack of glyphs. The 6-package selection is required: a single
`-p photocraft-ui-egui` build fails with 13 feature-resolution errors.

## 7. How the measurements were produced

```powershell
Set-Location D:\USERFILES\GitHub\photocraft
git fetch upstream main
git log -1 --format='%H %ci %s' upstream/main
git log -1 --format='%H %s' origin/feat/move-tool-selection-drag-transfer
git merge-base upstream/main origin/feat/move-tool-selection-drag-transfer
git diff --stat upstream/main origin/feat/move-tool-selection-drag-transfer

# the overlap sites: upstream lines the PR deletes, per file
$cur=''; foreach ($l in (git diff upstream/main origin/feat/move-tool-selection-drag-transfer)) {
  if ($l -match '^\+\+\+ b/(.+)$') { $cur = $Matches[1] }
  elseif ($l -match '^-' -and $l -notmatch '^---' -and $cur -like 'crates/*') { "$cur  $l" }
}

# what upstream already has
git grep -n -E "move_blocked|move_viewport" upstream/main -- crates/ui-egui/src/app.rs
git grep -n "includeLocked" upstream/main -- crates/engine/src/cmd/pick_cmds.rs
git grep -n '"cut"' upstream/main -- crates/engine/src/cmd/layer_copy_cmds.rs
git show upstream/main:crates/ui-egui/src/panels/layer_transfer.rs | Select-String -Pattern 'fn '
```

## 8. Risks and known unknowns

- **The port may move behaviour, not only lines.** Re-inserting the PR's behaviour on upstream's shape can
  change what a test observes. The PR's tests are the gate; an adaptation is recorded in the commit, never
  a weakened assert.
- **`includeLocked` changes the pick command.** It must default to today's behaviour so every existing
  caller is unaffected, and the pick command's own tests must stay green.
- **The cut variant touches the float path.** `CutParts` is shared with `float_cmds`; adding `copied`,
  `piece()` and `place_rest()` must not change the floating piece's behaviour. The engine suite is the gate.
- **`pick_cmds.rs` carries a parameter-doc regression to fix:** the PR's params doc must keep upstream's
  `list` param.
- **Upstream moves.** `upstream/main` is at `7cbc2b85e3` and receives commits; step 1 and step 5 re-measure
  against whatever it is at that moment.

## 9. Decisions

| Decision | Recommendation | Why |
|---|---|---|
| The `selection` / `cut` variant of `layer.copyToDocument` | **Keep it** | Upstream has copy-only (`selectionOnly`); the cut variant with the source's remainder is a capability upstream does not have. Port it onto upstream's branch instead of replacing it. |
| The relocated `lift` / `clear_selected` | **Close** | Upstream has them; the fork's copies are a duplication. Keep upstream's and widen the visibility. |
| The upstream `pick_target` lock rule | **Keep upstream's** | It encodes the Auto-Select lock skip; the PR's `includeLocked` is an additional variant, not a replacement. |

## 10. Definition of done

- `git diff --stat upstream/main <PR>` shows **0 deletions under `crates/`**, and the PR's addition count
  is at or below the current 1464 minus the 70 lines that came from upstream's newer commit.
- `layer_transfer` 32/32, `layer_copy_cmds` 37/37, `layer_transfer move_tool` 51/51, and the 6-package
  suite no worse than upstream's own baseline.
- `rustfmt --check` clean on the touched files, `cargo clippy` (lib targets) exit 0.
- The branch is pushed and the PR's diff and status are re-measured and reported.

## Appendix A: the 50 crate-side deletions

The upstream lines the PR currently deletes, grouped by file. Appendix A of this document is the work
list for step 2.

- `crates/engine/src/cmd/edit_cmds.rs` (2): `fn lift(src: &Surface, sel: Option<&Surface>, canvas: Rect) -> Result<Clip> {`, `fn clear_selected(doc: &mut Document, id: LayerId, background: [f32; 4]) -> Result<()> {`
- `crates/engine/src/cmd/float_cmds.rs` (6): the `is_mask` doc comment, `pub(crate) fn is_mask(&self) -> bool {`, `self.mask`, `}`, the blank line, `Ok(Self { layer, rest, piece, piece_bounds, outline, mask })`
- `crates/engine/src/cmd/layer_copy_cmds.rs` (3): `let mut l = if selection_only {`, the "Nothing changed: the source stays the active document." comment, the params doc string
- `crates/engine/src/cmd/pick_cmds.rs` (7): `pick_target(doc, x, y, group, false)`, `pick_target(doc, x, y, group, true)`, `fn pick_target(doc: &Document, x: i32, y: i32, group: bool, move_auto_select: bool) -> Option<LayerId> {`, `!locks.all && (!move_auto_select || !locks.position)`, the `let Some(target) = pick_target(&doc, …) else { return Ok(json!({ "layer": null })) };` line, `if p.get("select").and_then(Value::as_bool).unwrap_or(true) {`, the params doc string
- `crates/ui-egui/src/canvas/tools/move_tool.rs` (25): the `prompted` doc comment, `prompted: bool,`, the `Run { … prompted: false … }` constructor, the 4-argument `marquee` closure, the `move_box::begin` call and its "PLAN: assumed …" comment, the "locked-only hit can start a marquee (#4058)." comment, the blank line, the `if !mods.shift { … }` block with its `select.deselectLayers` call, the `return marquee(app, mods.shift, auto, auto && !empty);` line, the inline `let locked_hit = !auto && …` block with its `effective_locks` and `is_background` reads, `return marquee(app, mods.shift, false, false);`, the `if !std::mem::replace(&mut run.prompted, true) {` block with `self.scope.with_app(crate::move_lock::prompt);`, its closing brace, `Kind::Refused => {}`
- `crates/ui-egui/src/panels/layer_transfer.rs` (5): the `Transfer { … selection_only }` construction, `if ctx.input(|i| i.pointer.any_released()) {`, `let (pos, released, down) = ctx.input(…)`, `if !ctx.input(|i| i.pointer.any_released()) {`, `p["selectionOnly"] = json!(t.selection_only);`
- `crates/ui-egui/src/tools/move_box.rs` (2): `start: [f64; 2],`, `app.layer_box = Some(LayerBox { start: p, end: p, add, move_auto_select, clear_on_empty });`
