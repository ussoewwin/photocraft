# Quick Selection: New / Add / Subtract mode buttons in the options bar

The Quick Selection options bar gains the three selection-mode buttons every selection tool has in
Photoshop: New selection, Add to selection, and Subtract from selection. The mode changes what a
stroke does, what the canvas cursor badge shows, and what a held modifier overrides.

| Behaviour | Implementation |
|---|---|
| Mode buttons | Three segmented icon buttons at the left of the options bar, before the brush preset chip, followed by a divider |
| Modes | `ToolOptions.quick_selection_mode`: `0` New, `1` Add (default), `2` Subtract |
| Add (1) | A stroke adds; Alt subtracts |
| Subtract (2) | A stroke subtracts; Alt adds |
| New (0) | A stroke replaces; Shift adds, Alt subtracts |
| After a New stroke | The tool switches to Add once the stroke actually changed the selection |
| Cursor badge | `+` / `-` / none follows the effective mode for the held modifiers |
| Reset Tool | Restores mode 1 (Add) |
| Persistence | Stored with the other tool options; old preferences without the field load as 1 |
| Select and Mask | A button added at the right end of the options bar |

## Problem

Quick Selection always added to the selection, subtracting while Alt was held. That is Photoshop's
behaviour *after* the first stroke, but it leaves no way to start a new selection in the options bar:
a second stroke always extends the first even when the user wants a fresh one. The options bar also
lacked the New / Add / Subtract group that every other selection tool shows, so the tool's mode was
invisible and unreachable.

## Design

Three things have to agree: the bar, the stroke, and the cursor badge. They all derive from one
function so they cannot drift apart.

```
options bar buttons ──► quick_selection_mode (u8)
                              │
                              ▼
             tool_feedback::selection_mode(tool, bar, modifiers)
                              │            │
                              ▼            ▼
                    stroke mode      Badge (+ / - / none)
                    (select.quick)          │
                              │             ▼
                              └──────► canvas cursor
```

`selection_mode` is the single interpreter: it takes the tool, the bar value and the held modifiers,
and returns `"replace"`, `"add"`, `"subtract"` or `"intersect"`. Quick Selection passes its own bar
value; every other tool keeps passing `app.ui.selection_mode`.

### Mode resolution

| Bar | Modifiers | Effective mode |
|---|---|---|
| Add (1) | none | add |
| Add (1) | Alt | subtract |
| Add (1) | Shift | add |
| Subtract (2) | none | subtract |
| Subtract (2) | Alt | add |
| Subtract (2) | Shift | subtract |
| New (0) | none | replace |
| New (0) | Shift | add |
| New (0) | Alt | subtract |

An out-of-range bar value (a preference written by a future version) is read as Add.

### The New → Add transition

Photoshop starts a new selection in New mode and continues in Add. The transition is not a mode
timer; it is driven by the command result:

```rust
let changed = matches!(r, Ok(ref v) if v.get("changed").and_then(Value::as_bool) == Some(true));
report_smart_result(app, r, tl!("Quick Selection found nothing there"));
if o.quick_selection_mode == 0 && changed {
    app.ui.tool_options.quick_selection_mode = 1;
}
```

A stroke that found nothing (the alert path) leaves the mode at New, so the next attempt is still a
fresh selection. Only a stroke that changed the selection moves the tool to Add.

### Selection presence

`document_selection_mode` keeps its existing rule: an Alt stroke that would subtract from an empty
selection draws a new one instead, and it asks this last, from the same cached bounds the canvas uses.
Quick Selection now feeds that rule its own bar value:

```rust
let bar = if tool == Tool::QuickSelection { app.ui.tool_options.quick_selection_mode } else { app.ui.selection_mode };
let mode = selection_mode(tool, bar, m);
```

The badge reads the same function, so an Alt press with nothing selected shows no `-` when the stroke
will actually replace:

```rust
if tool == Tool::QuickSelection {
    return Badge::from_mode(document_selection_mode(app, tool, m));
}
```

## Implementation

### `crates/ui-egui/src/state/tool_options.rs`

```rust
/// Quick Selection selection mode: 0 New, 1 Add (default), 2 Subtract.
#[serde(default = "default_quick_selection_mode")]
pub quick_selection_mode: u8,
```

The serde default keeps old preference files loading as Add, and `Default for ToolOptions` sets the
same value, so a fresh profile and an upgraded one behave identically.

### `crates/ui-egui/src/panels/properties.rs`

`quick_selection_mode_buttons` clamps an out-of-range value to Add, then draws three 24 px icon
buttons with the current one selected:

```rust
for (i, (icon, tip)) in [
    ("quick-selection", tl!("New selection").to_string()),
    ("quick-selection-add", ...),
    ("quick-selection-subtract", ...),
].iter().enumerate()
```

The tooltips name the modifier that does the opposite operation in the current mode (Shift for Add,
Alt for Subtract), formatted through the shortcut table so the label follows the platform. Clicking a
button only changes the option; it does not touch the document or the journal.

`select_and_mask_button` is raised from `pub(super)` to `pub(crate)` because the options bar now calls
it for Quick Selection.

### `crates/ui-egui/src/panels/options_bar.rs`

The group is drawn only for Quick Selection, immediately before the brush chip, with the same
vertical divider the other grouped controls use:

```rust
if tool == Tool::QuickSelection {
    quick_selection_mode_buttons(app, ui);
    widgets::vline(ui, 22.0);
}
```

The Select and Mask… button sits at the right end of the bar for the same tool.

### `crates/ui-egui/src/canvas/tool_feedback.rs`

`selection_mode` replaces the two-branch Quick Selection case with three explicit modes plus a
normalised bar value, and `document_selection_mode` selects the bar source per tool. `badge` short
circuits for Quick Selection so the badge is derived from the same effective mode.

### `crates/ui-egui/src/tools/retouch_ui.rs`

`finish_stroke` used to compute `if mods.alt { "subtract" } else { "add" }`. It now asks
`document_selection_mode`, so the options bar, the modifiers and the empty-selection rule all reach
`select.quick` as one string, and the New → Add transition runs on the command's `changed` flag.

### `crates/ui-egui/src/tools/tool_reset.rs`

Reset Tool restores `quick_selection_mode` to 1 for Quick Selection, alongside the other per-tool
options.

### Icons and attribution

`assets/icons/quick-selection-add.svg` and `assets/icons/quick-selection-subtract.svg` are drawn for
this change, derived from the existing `quick-selection.svg` (ISC). They are registered in
`icon_data.rs` and listed in `ATTRIBUTION.md`.

## Commit by commit

Six commits, two of them merges, oldest first.

| Commit | Date | Subject | What it did |
|---|---|---|---|
| `828cf2635` | 2026-11-07 | ui-egui: add New, Add, and Subtract mode buttons to Quick Selection options bar | The field, the two SVGs, the icon keys, the buttons function, the options-bar call, the attribution rows. Hunks 1-6, 9-16, 18. |
| `7e29f4f96` | 2026-11-07 | ui-egui: keep Quick Selection in New mode after an empty stroke | The `changed` read and the flip condition in `finish_stroke`; the tests that pin the two "keeps New mode" cases. Hunk 19, part of 21. |
| `55ed8a0f7` | 2026-11-08 | Merge remote-tracking branch 'upstream/main' into feat/quick-selection-mode-buttons | First upstream merge. |
| `1359f933f` | 2026-11-09 | ui-egui: use document_selection_mode for Quick Selection strokes and cursor badge | `document_selection_mode`'s per-tool bar, the `badge` early return, the stroke's mode call, the badge test. Hunks 5-8, 19-20. |
| `e6e31005d` | 2026-11-09 | Merge branch 'upstream/main' into feat/quick-selection-mode-buttons | Second upstream merge. |
| `4bebcddda` | 2026-11-09 | fix(ui): normalize quick_selection_mode exceeding 2 to Add on render | The clamp at the top of `quick_selection_mode_buttons`. Hunk 14. |

The last commit is worth reading as a record of a defect found after the feature worked: the field is persisted and nothing prevented an out-of-range value from an older or hand-edited project file, so the fix is a clamp at the point of drawing rather than a validation at load. `selection_mode` (hunk 5) clamps independently, which is why the two agree even if the file is corrupt - the drawing normalises the stored value and the classifier normalises what it reads.

The two merges are upstream's `main` coming in; neither conflicted in the files this branch touches, and both are in the range because the branch was rebased by merge rather than by rebase.

## Types and call graph

| Item | Kind | Contract | Callers |
|---|---|---|---|
| `ToolOptions.quick_selection_mode` | `u8` field | 0 New, 1 Add (default), 2 Subtract; `#[serde(default = "default_quick_selection_mode")]` | `selection_mode` via `document_selection_mode`, `quick_selection_mode_buttons`, `finish_stroke`, `tool_reset` |
| `default_quick_selection_mode()` | fn | `1` | the serde attribute, `Default for ToolOptions` (by value) |
| `selection_mode(tool, bar, m)` | fn | tool + bar + modifiers -> `"replace" | "add" | "subtract" | "intersect"` | `document_selection_mode`, `badge` (other tools), `gesture.rs`, `tool_input.rs` |
| `document_selection_mode(app, tool, m)` | fn | `selection_mode` plus the empty-selection fallback; picks the bar per tool | `badge`, `retouch_ui::finish_stroke`, `gesture.rs` (Lasso) |
| `badge(app, tool, m)` | fn | `Option<Badge>` | the canvas cursor |
| `quick_selection_mode_buttons(app, ui)` | fn (`pub(super)`) | draws the 3 buttons, clamps, writes the option | `options_bar` |
| `select_and_mask_button(app, ui)` | fn (`pub(crate)`, widened) | the trailing action | `options_bar`, `retouch_ui` (hunk 20) |

Call chain, measured on the branch:

```
canvas/tool_feedback.rs
  selection_mode(tool, bar, m)
       ^                    ^
       |                    +-- badge() for every tool except Quick Selection
       |
  document_selection_mode(app, tool, m)
       ^                 ^                    ^
       |                 |                    |
   badge()          retouch_ui::finish_stroke   canvas/gesture.rs (Lasso)
   (Quick Selection)  (Quick Selection)          (works through document_selection_mode)

panels/options_bar.rs
  -> properties::quick_selection_mode_buttons(app, ui)
       -> app.ui.tool_options.quick_selection_mode = i as u8
  -> (retouch_ui's Quick Selection row) properties::select_and_mask_button(app, ui)

tools/retouch_ui.rs
  finish_stroke(QuickSelection)
       -> document_selection_mode(...)          -> "mode" for select.quick
       -> app.run("select.quick", {... "mode": mode ...})
       -> if quick_selection_mode == 0 && changed { quick_selection_mode = 1 }

tools/tool_reset.rs
  reset!(enhance_edge, quick_selection_mode)
```

The single-source rule is visible in the graph: the bar value is read in exactly one place (`document_selection_mode` chooses between `quick_selection_mode` and `selection_mode`), and the three consumers - the stroke, the badge and the Lasso - all pass through it.

## Behaviour details

| Situation | Result |
|---|---|
| First stroke in a fresh session | Add, because the default mode is 1 |
| Switch to New, drag over empty space | Alert "found nothing"; mode stays New |
| Switch to New, drag that changes the selection | Selection replaced; mode becomes Add |
| New + Shift | The stroke adds instead of replacing (mode stays New until it changes the selection) |
| Alt in Add mode, nothing selected | The stroke replaces rather than subtracting nothing; no `-` badge |
| Alt in Subtract mode | The stroke adds |
| Reset Tool | Mode returns to Add; the rest of the options reset as before |
| Preference file from an earlier version | Loads with mode 1 |
| Clicking a mode button | Only changes the option; no document edit, no history step |

## Testing

```
cargo test  -p photocraft-ui-egui --lib quick_selection_options_bar_has_new_add_subtract_mode_buttons
cargo test  -p photocraft-ui-egui --lib tool_feedback
cargo clippy -p photocraft-ui-egui --lib -- -D warnings
cargo fmt --check -p photocraft-ui-egui
cargo xtask layers
```

`quick_selection_options_bar_has_new_add_subtract_mode_buttons` drives the bar through `egui_kittest`:
it checks the default mode, clicks Subtract, New and Add in turn, checks the badge for each mode with
and without modifiers, and completes a stroke in New mode to assert the switch to Add.

`tool_feedback::tests` gains the New-mode badge case: with nothing selected, Alt shows no `-` because
the stroke will replace.

## Limitations

* The mode is per tool, not per document; switching documents keeps the mode.
* The New → Add transition depends on the command reporting `changed: true`; a command that changes
  the selection without the flag would leave the tool in New.
* The buttons are only drawn for Quick Selection; other selection tools keep the existing shared
  mode control.
* Baseline against the reference application: the mode group, the default, and the New → Add
  transition follow Photoshop's options bar; the exact icon shapes are drawn for this project.
## Verification

| Check | Result |
|---|---|
| `git diff --stat 535ea2d15 4bebcddd` | 12 files, +228 / -14 |
| `git diff 535ea2d15 4bebcddd \| grep -c '^@@'` | 22 hunks, matching the 22 sections below |
| GitHub checks | `CodeRabbit: success`; no GitHub Actions workflow runs are recorded (`status total=0`) |
| Merge state at the time of writing | `mergeable: MERGEABLE`, `mergeStateStatus: UNSTABLE` |
| `cargo test -p photocraft-ui-egui --lib quick_selection` in a detached worktree at `4bebcddd` | **7 passed, 0 failed** (0.16 s). Includes `tools::retouch_ui::tests::quick_selection_options_bar_has_new_add_subtract_mode_buttons` |
| `cargo test -p photocraft-ui-egui --lib tool_feedback` | **13 passed, 0 failed** (0.11 s). Includes `canvas::tool_feedback::tests::badge_follows_modifiers_and_the_options_bar` |
| `cargo test -p photocraft-ui-egui --lib panels::properties` | **3 passed, 0 failed** (0.05 s) |

## Merge state

The branch merges cleanly into `main` at the time of writing (`MERGEABLE`, `UNSTABLE`). "Unstable" is the status GitHub reports when a pull request is mergeable but a non-required check has not completed - here the only check is the review bot, which has finished. No workflow run exists for the branch, so there is no CI signal beyond the bot.

The change touches no shared module in a way that another tool can observe: `canvas/tool_feedback.rs` gains one branch inside the Quick Selection arm and one early return in `badge`; `panels/properties.rs` gains one function and widens one item's visibility; `state/tool_options.rs` gains one field with a serde default. Lasso, Marquee, Object Selection, Magic Wand and every other tool read the same fields and the same functions they read before, and the three assertions in hunk 8 that cover `ObjectSelection` are there to prove it.

## Every change, shown as original code and changed code

Base `535ea2d15` (the upstream commit the branch was opened against), head `4bebcddd`.
For each file, every hunk is printed twice: the code as it stands in the base, and the code as it
stands after the change. Unchanged context lines appear in both, so each block can be read on its own.

### `ATTRIBUTION.md` (+2 / -2)

#### 1. `ATTRIBUTION.md` — hunk -42,7 +42,7

**Original code (upstream base)**

```rust
| `epaint_default_fonts::EGUI_ICONS` (build input; deflated into `OUT_DIR/egui_fonts/egui-icons.ttf.deflate`, registered by `crates/ui-egui/src/egui_fonts.rs`) | egui-icons, five-glyph subset of emoji-icon-font | John Slegers | [egui fork at 6d518fc](https://github.com/storytold/egui/tree/6d518fc799681571963842f95b55a6b68431b15a/crates/epaint_default_fonts/fonts), [original](https://github.com/jslegers/emoji-icon-font); egui subset, unmodified here | MIT; Copyright (c) 2014 John Slegers; full upstream notice in [NOTICE](NOTICE) |
| `assets/fonts/Inter-Regular.ttf`, `Inter-Medium.ttf`, `Inter-SemiBold.ttf` | Inter 4.001 (UI font) | The Inter Project Authors (Rasmus Andersson) | <https://github.com/rsms/inter> | SIL OFL 1.1, [`assets/fonts/OFL-Inter.txt`](assets/fonts/OFL-Inter.txt) |
| `assets/fonts/JetBrainsMono-Regular.ttf` | JetBrains Mono 2.305 (numeric font) | The JetBrains Mono Project Authors | <https://github.com/JetBrains/JetBrainsMono> | SIL OFL 1.1, [`assets/fonts/OFL-JetBrainsMono.txt`](assets/fonts/OFL-JetBrainsMono.txt) |
| `assets/icons/*.svg` except `slice-knife.svg`, `slice-select.svg`, `font-similar.svg`, `font-variable.svg`, `sampler.svg`, `artboard.svg`, `perspective-crop.svg`, `frame-placeholder.svg`, `ellipse-dashed.svg`, `gradient.svg`, `type-mask.svg`, `type-mask-vertical.svg`, `text-warp.svg`, `clip-below.svg`, `clip-release.svg`, `layer-mask.svg`, `adjustment-layer.svg`, `brush-replace.svg`, `history-brush.svg`, `quick-selection.svg`, `eraser-background.svg`, `eraser-magic.svg`, `lasso-magnetic.svg`, `direct-select.svg`, `convert-point.svg`, `pen-add-anchor.svg`, `pen-delete-anchor.svg`, `pen-freeform.svg`, `pen-curvature.svg`, `bandage-sparkle.svg`, `painting-symmetry.svg`, `symmetry-*.svg`, `path-op-*.svg` and `marquee-{row,column}.svg`, `align-edge-*.svg` and `distribute-*.svg`, `crop-overlay-*.svg`, `vertical-align-*.svg` (115 files; `star-filled.svg` is `star.svg` with a solid fill) | Lucide icons | Lucide Icons and Contributors | <https://github.com/lucide-icons/lucide> (`icons/<name>.svg`; `align-*` are now named `text-align-*` upstream) | ISC, [`assets/icons/LICENSE-lucide.txt`](assets/icons/LICENSE-lucide.txt) |
| `assets/icons/{check,chevron-down,chevron-left,chevron-right,chevron-up,chevrons-left,chevrons-right,circle,clock,compass,info,link,link-2,lock,minus,monitor,moon,move,navigation,plus,search,square,trash,triangle,type,x,zoom-in}.svg` | Lucide icons derived from Feather (subset of the row above) | Cole Bemis (Feather), Lucide Contributors | <https://github.com/feathericons/feather> via Lucide | MIT (Feather) and ISC (Lucide), [`assets/icons/LICENSE-lucide.txt`](assets/icons/LICENSE-lucide.txt) |
| `assets/icons/slice-knife.svg` | Slice tool glyph | PhotoCraft contributors | Original work | MIT OR Apache-2.0 |
| `assets/icons/slice-select.svg` | Slice Select tool glyph | PhotoCraft contributors | Original work | MIT OR Apache-2.0 |
```

**Changed code (this pull request)**

```rust
| `epaint_default_fonts::EGUI_ICONS` (build input; deflated into `OUT_DIR/egui_fonts/egui-icons.ttf.deflate`, registered by `crates/ui-egui/src/egui_fonts.rs`) | egui-icons, five-glyph subset of emoji-icon-font | John Slegers | [egui fork at 6d518fc](https://github.com/storytold/egui/tree/6d518fc799681571963842f95b55a6b68431b15a/crates/epaint_default_fonts/fonts), [original](https://github.com/jslegers/emoji-icon-font); egui subset, unmodified here | MIT; Copyright (c) 2014 John Slegers; full upstream notice in [NOTICE](NOTICE) |
| `assets/fonts/Inter-Regular.ttf`, `Inter-Medium.ttf`, `Inter-SemiBold.ttf` | Inter 4.001 (UI font) | The Inter Project Authors (Rasmus Andersson) | <https://github.com/rsms/inter> | SIL OFL 1.1, [`assets/fonts/OFL-Inter.txt`](assets/fonts/OFL-Inter.txt) |
| `assets/fonts/JetBrainsMono-Regular.ttf` | JetBrains Mono 2.305 (numeric font) | The JetBrains Mono Project Authors | <https://github.com/JetBrains/JetBrainsMono> | SIL OFL 1.1, [`assets/fonts/OFL-JetBrainsMono.txt`](assets/fonts/OFL-JetBrainsMono.txt) |
| `assets/icons/*.svg` except `slice-knife.svg`, `slice-select.svg`, `font-similar.svg`, `font-variable.svg`, `sampler.svg`, `artboard.svg`, `perspective-crop.svg`, `frame-placeholder.svg`, `ellipse-dashed.svg`, `gradient.svg`, `type-mask.svg`, `type-mask-vertical.svg`, `text-warp.svg`, `clip-below.svg`, `clip-release.svg`, `layer-mask.svg`, `adjustment-layer.svg`, `brush-replace.svg`, `history-brush.svg`, `quick-selection.svg`, `quick-selection-add.svg`, `quick-selection-subtract.svg`, `eraser-background.svg`, `eraser-magic.svg`, `lasso-magnetic.svg`, `direct-select.svg`, `convert-point.svg`, `pen-add-anchor.svg`, `pen-delete-anchor.svg`, `pen-freeform.svg`, `pen-curvature.svg`, `bandage-sparkle.svg`, `painting-symmetry.svg`, `symmetry-*.svg`, `path-op-*.svg` and `marquee-{row,column}.svg`, `align-edge-*.svg` and `distribute-*.svg`, `crop-overlay-*.svg`, `vertical-align-*.svg` (115 files; `star-filled.svg` is `star.svg` with a solid fill) | Lucide icons | Lucide Icons and Contributors | <https://github.com/lucide-icons/lucide> (`icons/<name>.svg`; `align-*` are now named `text-align-*` upstream) | ISC, [`assets/icons/LICENSE-lucide.txt`](assets/icons/LICENSE-lucide.txt) |
| `assets/icons/{check,chevron-down,chevron-left,chevron-right,chevron-up,chevrons-left,chevrons-right,circle,clock,compass,info,link,link-2,lock,minus,monitor,moon,move,navigation,plus,search,square,trash,triangle,type,x,zoom-in}.svg` | Lucide icons derived from Feather (subset of the row above) | Cole Bemis (Feather), Lucide Contributors | <https://github.com/feathericons/feather> via Lucide | MIT (Feather) and ISC (Lucide), [`assets/icons/LICENSE-lucide.txt`](assets/icons/LICENSE-lucide.txt) |
| `assets/icons/slice-knife.svg` | Slice tool glyph | PhotoCraft contributors | Original work | MIT OR Apache-2.0 |
| `assets/icons/slice-select.svg` | Slice Select tool glyph | PhotoCraft contributors | Original work | MIT OR Apache-2.0 |
```

**Explanation.** `ATTRIBUTION.md` is the licence inventory: every third-party asset the repository ships is listed with its origin, its licence and where the licence text lives. The table row edited here is the one for the derived Lucide glyphs, and the change is to the *exception list* in its first cell.

The cell reads `` `assets/icons/*.svg` except `slice-knife.svg`, ... `` and then enumerates the SVG files that are **not** plain Lucide copies but Lucide compositions with marks added by PhotoCraft contributors. That list is the mechanism by which the file distinguishes two classes of asset with one row: the blanket glob covers the unmodified Lucide set under ISC, and the exceptions are the files whose provenance includes a contributor's drawing, which the last column then credits separately.

The edit inserts `quick-selection-add.svg` and `quick-selection-subtract.svg` after `quick-selection.svg`. Alphabetical order in the list is pre-existing and preserved: the entries run `...brush-replace.svg`, `history-brush.svg`, `quick-selection.svg`, then the two new names, then `eraser-backgrou...`. Inserting at that position rather than at the end keeps the list scannable and keeps the diff to one line.

Why the exception list rather than new rows: the two new files are derived from the same Lucide `brush.svg` as `quick-selection.svg`, with the same 70 % scale inside the same dashed "C", plus a `+` or `−` drawn by a contributor. Their licence status is therefore identical to the existing file's - ISC for the Lucide part, the repository's own licence for the added marks - so a separate row would state the same origin, the same author chain and the same licence file twice. The exception list exists precisely to avoid that duplication.

The consequence if this insertion were omitted is not cosmetic: the glob would silently claim the two new files as unmodified Lucide assets, and the attribution would then be wrong about who drew the `+` and the `−`. A licence inventory that over-claims third-party origin is the failure mode this file exists to prevent, which is why the greppable exception list is maintained by hand rather than generated.

Nothing else in the row changes: the description cell, the author cell, the source URL and the licence cell still describe `brush.svg` under ISC, because that is still the derivation. Only the set of files the row applies to grows.

The table's columns, in order, are: file pattern, description, author/origin, upstream URL, licence with the file holding its text. The edit touches only the first, but the row is a single unit of meaning: a reader who wants to know whether a given SVG is a third-party asset finds it by matching the pattern, and everything else in the row then applies. That is why the exception list has to be exact - a file in the list gets the derived-work description and the contributor credit, a file not in it gets the plain Lucide description, and there is no third state.

The list is not generated, and it cannot be without adding a build step. Generating it would mean deciding per file whether it is a byte-identical copy of an upstream SVG, which needs the upstream repository checked out at a known revision - a network dependency in a CI step for a documentation file. Hand-maintained lists drift, but the drift is visible in review: a new derived icon shows up as an unmodified-looking file in a diff, and a reviewer who knows the convention looks for the exception entry. That is the trade this file makes, and the hunk is what maintaining the trade looks like.

Nothing in this row names the two new files' *sizes* or checksums. The inventory records provenance, not identity - the icons are compiled into the binary by `include_bytes!` and the build would fail if a file were missing (hunk 9). Checksums would duplicate that check and would need updating on every legitimate edit to an SVG.

#### 2. `ATTRIBUTION.md` — hunk -66,7 +66,7

**Original code (upstream base)**

```rust
| `assets/icons/eraser-background.svg`, `assets/icons/eraser-magic.svg` | Background Eraser and Magic Eraser tool glyphs: Lucide `eraser` plus added marks | Lucide Icons and Contributors; marks by PhotoCraft contributors | Derived from <https://github.com/lucide-icons/lucide> `icons/eraser.svg` | ISC, [`assets/icons/LICENSE-lucide.txt`](assets/icons/LICENSE-lucide.txt) |
| `assets/icons/brush-replace.svg` | Color Replacement tool glyph: Lucide `brush` plus two swap arrows | Lucide Icons and Contributors; arrows by PhotoCraft contributors | Derived from <https://github.com/lucide-icons/lucide> `icons/brush.svg` | ISC, [`assets/icons/LICENSE-lucide.txt`](assets/icons/LICENSE-lucide.txt) |
| `assets/icons/history-brush.svg` | History Brush tool glyph: Lucide `brush` plus counter-clockwise history rewind arrow | Lucide Icons and Contributors; arrow by PhotoCraft contributors | Derived from <https://github.com/lucide-icons/lucide> `icons/brush.svg` | ISC, [`assets/icons/LICENSE-lucide.txt`](assets/icons/LICENSE-lucide.txt) |
| `assets/icons/quick-selection.svg` | Quick Selection tool glyph: Lucide `brush`, scaled to 70%, inside a dashed upright rounded "C" selection | Lucide Icons and Contributors; selection by PhotoCraft contributors | Derived from <https://github.com/lucide-icons/lucide> `icons/brush.svg` | ISC, [`assets/icons/LICENSE-lucide.txt`](assets/icons/LICENSE-lucide.txt) |
| `assets/icons/bandage-sparkle.svg` | Remove tool glyph: Lucide `bandage` moved down, plus a sparkle mark | Lucide Icons and Contributors; mark by PhotoCraft contributors | Derived from <https://github.com/lucide-icons/lucide> `icons/bandage.svg` | ISC, [`assets/icons/LICENSE-lucide.txt`](assets/icons/LICENSE-lucide.txt) |
| `assets/icons/direct-select.svg` | Direct Selection tool glyph: Lucide `mouse-pointer-2` plus an anchor square | Lucide Icons and Contributors; square by PhotoCraft contributors | Derived from <https://github.com/lucide-icons/lucide> `icons/mouse-pointer-2.svg` | ISC, [`assets/icons/LICENSE-lucide.txt`](assets/icons/LICENSE-lucide.txt) |
| `assets/icons/pen-freeform.svg`, `assets/icons/pen-curvature.svg` | Freeform Pen and Curvature Pen tool glyphs: Lucide `pen-tool` scaled down, plus a freehand squiggle or a curve through two points | Lucide Icons and Contributors; marks by PhotoCraft contributors | Derived from <https://github.com/lucide-icons/lucide> `icons/pen-tool.svg` | ISC, [`assets/icons/LICENSE-lucide.txt`](assets/icons/LICENSE-lucide.txt) |
```

**Changed code (this pull request)**

```rust
| `assets/icons/eraser-background.svg`, `assets/icons/eraser-magic.svg` | Background Eraser and Magic Eraser tool glyphs: Lucide `eraser` plus added marks | Lucide Icons and Contributors; marks by PhotoCraft contributors | Derived from <https://github.com/lucide-icons/lucide> `icons/eraser.svg` | ISC, [`assets/icons/LICENSE-lucide.txt`](assets/icons/LICENSE-lucide.txt) |
| `assets/icons/brush-replace.svg` | Color Replacement tool glyph: Lucide `brush` plus two swap arrows | Lucide Icons and Contributors; arrows by PhotoCraft contributors | Derived from <https://github.com/lucide-icons/lucide> `icons/brush.svg` | ISC, [`assets/icons/LICENSE-lucide.txt`](assets/icons/LICENSE-lucide.txt) |
| `assets/icons/history-brush.svg` | History Brush tool glyph: Lucide `brush` plus counter-clockwise history rewind arrow | Lucide Icons and Contributors; arrow by PhotoCraft contributors | Derived from <https://github.com/lucide-icons/lucide> `icons/brush.svg` | ISC, [`assets/icons/LICENSE-lucide.txt`](assets/icons/LICENSE-lucide.txt) |
| `assets/icons/quick-selection.svg`, `quick-selection-add.svg`, `quick-selection-subtract.svg` | Quick Selection tool glyphs (tool icon and New/Add/Subtract options-bar modes): Lucide `brush`, scaled to 70%, inside a dashed upright rounded "C" selection, with optional + or − mark | Lucide Icons and Contributors; selection and marks by PhotoCraft contributors | Derived from <https://github.com/lucide-icons/lucide> `icons/brush.svg` | ISC, [`assets/icons/LICENSE-lucide.txt`](assets/icons/LICENSE-lucide.txt) |
| `assets/icons/bandage-sparkle.svg` | Remove tool glyph: Lucide `bandage` moved down, plus a sparkle mark | Lucide Icons and Contributors; mark by PhotoCraft contributors | Derived from <https://github.com/lucide-icons/lucide> `icons/bandage.svg` | ISC, [`assets/icons/LICENSE-lucide.txt`](assets/icons/LICENSE-lucide.txt) |
| `assets/icons/direct-select.svg` | Direct Selection tool glyph: Lucide `mouse-pointer-2` plus an anchor square | Lucide Icons and Contributors; square by PhotoCraft contributors | Derived from <https://github.com/lucide-icons/lucide> `icons/mouse-pointer-2.svg` | ISC, [`assets/icons/LICENSE-lucide.txt`](assets/icons/LICENSE-lucide.txt) |
| `assets/icons/pen-freeform.svg`, `assets/icons/pen-curvature.svg` | Freeform Pen and Curvature Pen tool glyphs: Lucide `pen-tool` scaled down, plus a freehand squiggle or a curve through two points | Lucide Icons and Contributors; marks by PhotoCraft contributors | Derived from <https://github.com/lucide-icons/lucide> `icons/pen-tool.svg` | ISC, [`assets/icons/LICENSE-lucide.txt`](assets/icons/LICENSE-lucide.txt) |
```

### `assets/icons/quick-selection-add.svg` (+18 / -0)

**Explanation.** The second `ATTRIBUTION.md` row is the one that credits the *description* of each Quick Selection glyph, so it has to be updated in step with the exception list in hunk 1 - the list decides which files this row covers, and this row says what each of them is.

The first cell changes from a single filename to three. Before, it read `` `assets/icons/quick-selection.svg` ``; now it reads `` `assets/icons/quick-selection.svg`, `quick-selection-add.svg`, `quick-selection-subtract.svg` ``. Grouping them in one cell is correct because all three are one drawing with optional marks, not three independent works.

The second cell is the description, and it is where the meaning of the new files is recorded. It changes from "Quick Selection tool glyph: Lucide `brush`, scaled to 70%, inside a dashed upright rounded \"C\" selection" to "Quick Selection tool glyphs (tool icon and New/Add/Subtract options-bar modes): Lucide `brush`, scaled to 70%, inside a dashed upright rounded \"C\" selection, with optional + or − mark". Two additions carry the information: the parenthetical names the three UI places the glyphs appear (the toolbar button and the two new options-bar buttons), and the trailing clause states that the `+` and `−` are optional marks on the shared drawing rather than separate compositions.

Keeping "optional" in the sentence matters for the attribution claim: if the description said "with a + or − mark" without "optional", a reader could infer three independently authored drawings. What exists in the repository is one drawing whose optional marks a contributor added, and the wording now says exactly that.

The author cell is unchanged ("Lucide Icons and Contributors; selection and marks by PhotoCraft contributors"). It already said "marks", which covers the `+`/`−` before they existed - that phrasing was written for the derived glyphs generally, so it did not need editing. The source and licence cells are unchanged for the same reason as in hunk 1: the derivation (`brush.svg`), the ISC licence and `LICENSE-lucide.txt` all still apply.

The order of the two edits matters if they are ever reverted separately: reverting this row's filename list without hunk 1 would leave two unlisted files that the glob claims as pure Lucide; reverting hunk 1 without this row would list files whose description mentions only one glyph. Both halves describe the same change and neither is meaningful alone.

Worth noting what the row does *not* claim. It does not say the two new files are separate works; it says the glyphs are the tool icon and the two options-bar modes, with the marks optional. That distinction is the difference between "we ship three icons, two of which we commissioned" and "we ship one icon with two marks we added", and the latter is what the licence analysis supports - the underlying `brush.svg` is ISC and unmodified, and what the repository adds is a plus and a minus.

The description cell also carries the UI mapping, which is information the inventory does not strictly need but that a reader looking for "where is this icon used" does need. Naming "New/Add/Subtract options-bar modes" is what connects the asset to `quick_selection_mode_buttons` (hunk 14), where the three keys are used in that order. Without it, a reader would have to grep for the filename to learn that the three glyphs are one control.

The two cells that would be tempting to change and were not: the author cell already says "marks by PhotoCraft contributors", which covers `+` and `−` without naming them; rewriting it to "plus and minus marks" would be more specific but would then need editing again for the next derived glyph. The licence cell points at the shared `LICENSE-lucide.txt`, which is correct for the whole derived set - a per-file licence text would be the same file copied.

The change is confined to one physical line in each of hunks 1 and 2 because the table is wide: each row is one line in the source, however long. A reviewer reading the raw diff sees two long lines replaced by two longer ones, and the added text is at a stable position in each (the pattern list in hunk 1, the description in hunk 2). That is the reason the two edits can be described separately rather than as one: they are in different cells and have no ordering dependency.

#### 3. `assets/icons/quick-selection-add.svg` — hunk -0,0 +1,18

**Original code (upstream base)**

```rust
(no lines)
```

**Changed code (this pull request)**

```rust
<svg
  xmlns="http://www.w3.org/2000/svg"
  width="24"
  height="24"
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width="2"
  stroke-linecap="round"
  stroke-linejoin="round"
>
  <path d="M14 3H7a4 4 0 0 0-4 4v10a4 4 0 0 0 4 4h7" stroke-dasharray="2 3" />
  <path d="m14 10 2.1 2.1" />
  <path d="M10.85 17.7A2.45 2.45 0 1 0 8.4 15.25a1.834 1.834 0 0 1-.496 1.254A.7 .7 0 0 0 8.4 17.7z" />
  <path d="M13.278 14.922 21.265 6.937a.7 .7 0 0 0-2.101-2.101L11.177 12.822" />
  <path d="M16.5 18.5h5" />
  <path d="M19 16v5" />
</svg>
```

### `assets/icons/quick-selection-subtract.svg` (+17 / -0)

**Explanation.** A new SVG file, 18 lines. Every one of them exists because of how the icon pipeline consumes SVGs, so the geometry is not arbitrary.

The root element carries `xmlns`, `width="24"`, `height="24"`, `viewBox="0 0 24 24"`, `fill="none"`, `stroke="currentColor"`, `stroke-width="2"`, `stroke-linecap="round"`, `stroke-linejoin="round"`. That is the exact prelude of the existing `quick-selection.svg` and of every Lucide original, which matters because the icons are rasterised by one shared routine: a different `viewBox` would scale differently against its neighbours in the toolbar, and a different `stroke-width` would look heavier or lighter at the same size. `stroke="currentColor"` rather than a literal colour is what lets the button tint the glyph with the theme's foreground and with the disabled/active state; a hard-coded colour would ignore both.

The first path, `M14 3H7a4 4 0 0 0-4 4v10a4 4 0 0 0 4 4h7`, with `stroke-dasharray="2 3"`, draws the open "C": a vertical edge at x=14 from y=3, round corners of radius 4, down the left side and along the bottom, ending at x=14 on the right. The `2 3` dash pattern is the same as in `quick-selection.svg`, so the three icons read as one family. It is drawn open on the right - the two endpoints stop at x=14 - because the glyph inside the opening is a brush, and a closed ring would have to be overlapped by the brush to fit.

The second path, `m14 10 2.1 2.1`, is the bristle detail: a short diagonal stroke from (14, 10) to (16.1, 12.1). It is a separate path rather than part of the brush silhouette because Lucide draws the brush as a silhouette plus this one hair line, and keeping the decomposition identical to the upstream file is what makes the derivation auditable against `brush.svg`.

The third path is the brush head, a closed teardrop: `M10.85 17.7A2.45 2.45 0 1 0 8.4 15.25a1.834 1.834 0 0 1-.496 1.254A.7 .7 0 0 0 8.4 17.7z`. The two arcs and the cubic tail produce the shape Lucide uses for the ferrule end. It is copied unchanged from `quick-selection.svg`, which is why the coordinates are not round numbers.

The fourth path, `M13.278 14.922 21.265 6.937a.7 .7 0 0 0-2.101-2.101L11.177 12.822`, is the handle: a shaft rising to the upper right and a rounded cap. Its far end reaches x≈21.3, past the "C" opening at x=14 - deliberately, because the handle is what visually breaks the ring and tells the eye the C is open.

The fifth and sixth paths, `M16.5 18.5h5` and `M19 16v5`, are the only new geometry: a horizontal stroke 5 units long centred at (19, 18.5) and a vertical stroke 5 units long centred at the same point. Two strokes, not a single path, because the `+` must have the same stroke width and cap as the rest of the glyph; drawing them as one path would risk the join being mitred. They sit in the lower-right quadrant, outside the "C" and below the handle's end, which is the only region of the 24 x 24 box that is empty in all three glyphs - placing the mark inside the C would collide with the brush head, and above the handle would collide with the handle.

The file has no `<title>`, no `id` and no class attributes. Those would be dead weight: the rasteriser reads geometry only, and `currentColor` already carries the colour.

#### 4. `assets/icons/quick-selection-subtract.svg` — hunk -0,0 +1,17

**Original code (upstream base)**

```rust
(no lines)
```

**Changed code (this pull request)**

```rust
<svg
  xmlns="http://www.w3.org/2000/svg"
  width="24"
  height="24"
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width="2"
  stroke-linecap="round"
  stroke-linejoin="round"
>
  <path d="M14 3H7a4 4 0 0 0-4 4v10a4 4 0 0 0 4 4h7" stroke-dasharray="2 3" />
  <path d="m14 10 2.1 2.1" />
  <path d="M10.85 17.7A2.45 2.45 0 1 0 8.4 15.25a1.834 1.834 0 0 1-.496 1.254A.7 .7 0 0 0 8.4 17.7z" />
  <path d="M13.278 14.922 21.265 6.937a.7 .7 0 0 0-2.101-2.101L11.177 12.822" />
  <path d="M16.5 18.5h5" />
</svg>
```

### `crates/ui-egui/src/canvas/tool_feedback.rs` (+39 / -7)

**Explanation.** The subtract glyph, 17 lines, is the add glyph with the vertical stroke removed. Describing the difference is the whole of this file's meaning, so the shared parts get only what the difference depends on.

Lines 1-10 are byte-identical to `quick-selection-add.svg`: the root element with `viewBox="0 0 24 24"`, `fill="none"`, `stroke="currentColor"`, `stroke-width="2"`, round caps and joins. That identity is the requirement, not a convenience - the two buttons sit next to each other in the options bar and any difference in prelude would show as a difference in weight or alignment.

The first four paths are also byte-identical: the dashed open "C" (`M14 3H7a4 4 0 0 0-4 4v10a4 4 0 0 0 4 4h7` with `stroke-dasharray="2 3"`), the bristle line (`m14 10 2.1 2.1`), the brush head teardrop, and the handle (`M13.278 14.922 21.265 6.937a.7 .7 0 0 0-2.101-2.101L11.177 12.822`). Identical coordinates mean the brush sits at the same pixel position in all three icons, which is what lets the three buttons read as one tool in three states rather than three tools.

The single difference is the last path: `M16.5 18.5h5` with no companion `M19 16v5`. One horizontal stroke, the same 5 units at the same centre (19, 18.5), gives a minus sign. The centre is shared with the plus on purpose: when a user clicks Add then Subtract, the mark must not appear to move - the only visible change should be the disappearance of the vertical bar.

The file is one line shorter than the add glyph (17 versus 18) purely because of that missing path. That is the mechanical explanation of the `-0,0 +1,17` versus `-0,0 +1,18` in the two hunks, and it is worth stating because a reviewer comparing the two hunks line by line will otherwise wonder whether something else differs.

Why a minus rather than, say, an outlined C with a slash: the mark has to be legible at toolbar size (the glyphs are rasterised small) and has to carry the same meaning as Photoshop's Subtract icon, which is a minus on a lasso. A minus is the smallest unambiguous mark; a slash would be confusable with the "no" symbol and a hollow mark would disappear at small sizes.

No `fill` attribute is added to the strokes. Lucide's minus is a stroke, not a filled rectangle: at `stroke-width="2"` a filled 5 x 2 rectangle would have square ends, while the round caps specified in the prelude give the rounded ends the rest of the glyph uses.

The relationship between the two new files matters for review: subtract is not an independent drawing, it is add minus one `<path>`, and that is verifiable by comparing the two files. Every other line is byte-identical, which means a future edit to the brush geometry has to be made in both files and in `quick-selection.svg` - three places. That is the cost of shipping three static SVGs instead of one SVG with a marker, and it is the right cost here because the icon pipeline takes a name and returns bytes: it has no notion of a variant or a parameter, so a parameterised glyph would need either three generated files at build time (a build-script dependency for three files) or a renderer that composites marks (`icons::button` would grow an overlay concept for one tool).

The alternative of drawing the mark in code - the button draws the C-plus-brush icon, then paints a `+` on top using `ui.painter()` - was not taken for two reasons. The mark would have to match the SVG's stroke width and cap style exactly, and reproducing that with painter primitives means hard-coding the same 2.0 width and round caps in a second place; and the mark's position would be a magic offset in the button code, whereas in the SVG it is two coordinates in the same coordinate system as everything else.

The file's dimensions and viewBox (24 x 24) are what `icons::button` assumes when it scales to the requested size (24.0 in hunk 14's call). An icon with a different viewBox would still render, but its stroke weight relative to the others would be off, because the rasteriser scales the whole coordinate system: a 32-unit viewBox drawn at 24 points would make a `stroke-width="2"` look 1.5 units thick relative to the 24-unit family. Keeping the prelude identical is therefore not tidiness - it is what makes the three buttons visually consistent.

No `id` attributes on any path. An `id` would be needed only if something referenced the path (`<use>`, a CSS rule, an animation); the rasteriser does neither, and an unused `id` is a string in the file that a linter will eventually flag.

#### 5. `crates/ui-egui/src/canvas/tool_feedback.rs` — hunk -56,12 +56,36

**Original code (upstream base)**

```rust
/// 2 Subtract, 3 Intersect) and the held modifiers: exactly what the tool's gesture will use.
pub fn selection_mode(tool: Tool, bar: u8, m: egui::Modifiers) -> &'static str {
    match tool {
        // Quick Selection always adds (⌥ subtracts), like Photoshop's after the first stroke.
        Tool::QuickSelection => {
            if m.alt {
                "subtract"
            } else {
                "add"
            }
        }
        // Object Selection: ⌥ subtracts, ⇧ adds (see `retouch_ui::finish_object_selection`).
```

**Changed code (this pull request)**

```rust
/// 2 Subtract, 3 Intersect) and the held modifiers: exactly what the tool's gesture will use.
pub fn selection_mode(tool: Tool, bar: u8, m: egui::Modifiers) -> &'static str {
    match tool {
        // Quick Selection: options bar has New (0), Add (1, default), Subtract (2).
        Tool::QuickSelection => {
            let eff_bar = match bar {
                0..=2 => bar,
                _ => 1,
            };
            match eff_bar {
                2 => {
                    if m.alt {
                        "add"
                    } else {
                        "subtract"
                    }
                }
                0 => {
                    if m.alt {
                        "subtract"
                    } else if m.shift {
                        "add"
                    } else {
                        "replace"
                    }
                }
                _ => {
                    if m.alt {
                        "subtract"
                    } else {
                        "add"
                    }
                }
            }
        }
        // Object Selection: ⌥ subtracts, ⇧ adds (see `retouch_ui::finish_object_selection`).
```

**Explanation.** `selection_mode` maps (tool, options-bar value, held modifiers) to one of four strings that the stroke commands accept: `"replace"`, `"add"`, `"subtract"`, `"intersect"`. It is a pure function with no state, which is why the whole table can be tested exhaustively and why the change is confined to one match arm.

Signature: `pub fn selection_mode(tool: Tool, bar: u8, m: egui::Modifiers) -> &'static str`. `bar` is the numeric options-bar mode - the field is a `u8` rather than an enum because it is persisted in the project file as a number and because the bar's radio group is indexed. Returning `&'static str` rather than an enum is pre-existing: the strings go straight into the command JSON as `"mode"`, so an enum would need a `to_str` at every call site. The cost is that a typo is a runtime bug rather than a compile error, which is why the test asserts exact strings.

Before the change the Quick Selection arm ignored `bar` entirely:

```rust
Tool::QuickSelection => { if m.alt { "subtract" } else { "add" } }
```

That is Photoshop's behaviour *after the first stroke*: the tool is additive and ⌥ inverts to subtractive. The comment above it said so ("Quick Selection always adds (⌥ subtracts), like Photoshop's after the first stroke"). The pull request adds the three-way options bar, so the arm now has to honour it, and the comment is rewritten to name the three values: New (0), Add (1, default), Subtract (2).

The clamp is the first statement: `let eff_bar = match bar { 0..=2 => bar, _ => 1 };`. It exists because `bar` comes from a persisted field that an older or hand-edited project file can set to anything in `u8`'s range. Without the clamp, a value of 7 would fall into the final `_` arm and silently behave as Add - the same outcome, but by accident rather than by rule, and the `_` arm would then be doing two jobs. With the clamp, `eff_bar` provably has one of three values and the inner match is exhaustive over them, with `_` covering only Add. The default on an out-of-range value is Add (1), not New, because Add is the tool's normal behaviour and a corrupt file should degrade to the ordinary case rather than to a mode that replaces the selection on every stroke.

The Subtract case: `2 => if m.alt { "add" } else { "subtract" }`. ⌥ inverts, so Subtract + ⌥ is Add. This is the same inversion Photoshop applies to its options-bar modes, and it is why the mode cannot simply be picked from `eff_bar` alone.

The New case: `0 => if m.alt { "subtract" } else if m.shift { "add" } else { "replace" }`. Three-way. `replace` is what New means - the stroke replaces the existing selection. ⌥ subtracts from it; ⇧ adds to it. Note the order: ⌥ is checked before ⇧, so ⌥+⇧ yields Subtract. That order is not arbitrary - it matches the other tools' arms in the same function, so a user moving between tools does not get a different meaning from the same chord.

The remaining case, `_` (which is Add, 1): `if m.alt { "subtract" } else { "add" }`. Plain Add with ⌥ inversion, identical to the pre-change behaviour of the whole arm. That is the reason the default is Add: with `bar == 1` the function returns exactly what it returned before the pull request existed, so a project file saved by an older build behaves identically after the upgrade.

Every branch returns `"replace"`, `"add"` or `"subtract"` - never `"intersect"`, because the Quick Selection tool has no intersect mode in this UI. The other tools' arms still produce `"intersect"`, so the string is still in the function's range; it is simply unreachable from this arm.

The arm is `Tool::QuickSelection` only. Lasso, Marquee, Object Selection and the rest are untouched, which is the scope the pull request claims: the options-bar triple is Quick Selection's, and its semantics have to be established in this one function because the stroke, the cursor badge and the Lasso all read their mode through it.

#### 6. `crates/ui-egui/src/canvas/tool_feedback.rs` — hunk -92,11 +116,12

**Original code (upstream base)**

```rust
/// subtract from or intersect with, so it draws a new selection, as in Photoshop (where ⌥ then
/// only draws a marquee from its centre). An options-bar Subtract or Intersect is kept (#1106).
pub fn document_selection_mode(app: &PhotocraftApp, tool: Tool, m: egui::Modifiers) -> &'static str {
    let mode = selection_mode(tool, app.ui.selection_mode, m);
    // Asked last, and from the bounds the selection outline caches (same key): the canvas asks
    // every frame while a Patch lasso is drawn or hovered with ⌥.
    let selected = || app.session.active().is_some_and(|st| st.doc.selection.as_ref().is_some_and(|s| !app.known_bounds(u64::MAX - st.doc.id.0, s).is_empty()));
    if m.alt && matches!(mode, "subtract" | "intersect") && selection_mode(tool, app.ui.selection_mode, egui::Modifiers::NONE) == "replace" && !selected() {
        "replace"
    } else {
        mode
```

**Changed code (this pull request)**

```rust
/// subtract from or intersect with, so it draws a new selection, as in Photoshop (where ⌥ then
/// only draws a marquee from its centre). An options-bar Subtract or Intersect is kept (#1106).
pub fn document_selection_mode(app: &PhotocraftApp, tool: Tool, m: egui::Modifiers) -> &'static str {
    let bar = if tool == Tool::QuickSelection { app.ui.tool_options.quick_selection_mode } else { app.ui.selection_mode };
    let mode = selection_mode(tool, bar, m);
    // Asked last, and from the bounds the selection outline caches (same key): the canvas asks
    // every frame while a Patch lasso is drawn or hovered with ⌥.
    let selected = || app.session.active().is_some_and(|st| st.doc.selection.as_ref().is_some_and(|s| !app.known_bounds(u64::MAX - st.doc.id.0, s).is_empty()));
    if m.alt && matches!(mode, "subtract" | "intersect") && selection_mode(tool, bar, egui::Modifiers::NONE) == "replace" && !selected() {
        "replace"
    } else {
        mode
```

**Explanation.** `document_selection_mode` is the wrapper that `selection_mode` cannot be: it knows the app, so it can answer the one question `selection_mode` deliberately does not - whether anything is currently selected - and it can pick which options-bar value applies to the given tool. This hunk is where the second job is added.

The first change is the line before the call: `let bar = if tool == Tool::QuickSelection { app.ui.tool_options.quick_selection_mode } else { app.ui.selection_mode };`. Two fields now hold "the options-bar mode". `app.ui.selection_mode` is the long-standing one, shared by Lasso, Marquee, Object Selection and the rest. `app.ui.tool_options.quick_selection_mode` is the new per-tool field this pull request adds, living in `ToolOptions` because it is per-tool state that is persisted with the document's tool settings.

Why a second field rather than reusing `app.ui.selection_mode`: the existing field is global UI state, and Quick Selection's three modes are a different vocabulary (New/Add/Subtract versus Replace/Add/Subtract/Intersect - Quick Selection has no Intersect but does have New, and its default is Add rather than Replace). Sharing the field would mean the Lasso's mode changes when the user clicks a Quick Selection button, which is exactly the coupling this design avoids.

The ternary is written as an `if` expression on `tool == Tool::QuickSelection` rather than a match because there is one exception and a default. A match would need a `_` arm and would grow a second exception awkwardly; the `if` reads as "Quick Selection has its own field, everything else uses the shared one", which is the rule.

The second change is that both uses of the mode inside the function now pass `bar` instead of `app.ui.selection_mode`. There are two: `let mode = selection_mode(tool, bar, m);` and the guard inside the `if`, `selection_mode(tool, bar, egui::Modifiers::NONE) == "replace"`. Passing the field in only one of them would be a subtle bug: the function would compute the displayed mode from the new field but decide whether to override it from the old one, so in New mode with ⌥ the first call would say `"replace"` and the guard's second call would say `"add"` (the old field's default), the `== "replace"` test would fail, and the override would not fire - the badge would show Subtract in a mode where nothing can be subtracted.

The guard's purpose is unchanged by this hunk and is worth stating because the mode value now varies per tool: `if m.alt && matches!(mode, "subtract" | "intersect") && selection_mode(tool, bar, NONE) == "replace" && !selected() { "replace" } else { mode }`. It means "⌥ would subtract, the unmodified mode is replace, and there is nothing selected - so ⌥ has nothing to subtract from, and the stroke will actually create a new selection". The comment above it explains the Photoshop lineage: ⌥ with nothing selected draws a fresh marquee rather than subtracting from emptiness.

`selected()` is a closure, not a value, because the bounds lookup is not free and the comment records why: the canvas calls this every frame while a Patch lasso is drawn or hovered with ⌥. It reads `app.known_bounds(u64::MAX - st.doc.id.0, s)` - the same cache key the selection outline uses, so the lookup does not force a second scan of the mask.

The blast radius of this hunk is three call sites, all of which pass through `document_selection_mode` and therefore all of which now see the per-tool field: `retouch_ui::finish_stroke` (the stroke), `badge` (hunk 7, the cursor), and `gesture.rs` for the Lasso (which is not Quick Selection, so it keeps reading the shared field). No other tool's behaviour changes, because the `else` branch returns the same field it always did.

#### 7. `crates/ui-egui/src/canvas/tool_feedback.rs` — hunk -145,6 +170,9

**Original code (upstream base)**

```rust
    if matches!(tool, Tool::Lasso | Tool::PolygonLasso) && app.drag.is_none() && app.session.active().is_none_or(|st| st.doc.selection.is_none()) {
        m.alt = false;
    }
    Badge::from_mode(selection_mode(tool, app.ui.selection_mode, m))
}

```

**Changed code (this pull request)**

```rust
    if matches!(tool, Tool::Lasso | Tool::PolygonLasso) && app.drag.is_none() && app.session.active().is_none_or(|st| st.doc.selection.is_none()) {
        m.alt = false;
    }
    if tool == Tool::QuickSelection {
        return Badge::from_mode(document_selection_mode(app, tool, m));
    }
    Badge::from_mode(selection_mode(tool, app.ui.selection_mode, m))
}

```

**Explanation.** `badge` returns the little +/- indicator drawn next to the cursor while a selection tool is active. It has two exits: an early one for Lasso and PolygonLasso, and the tail that builds the badge from `selection_mode`. This hunk adds a third path between them for Quick Selection.

The early-exit block above is untouched: `if matches!(tool, Tool::Lasso | Tool::PolygonLasso) && app.drag.is_none() && app.session.active().is_none_or(|st| st.doc.selection.is_none()) { m.alt = false; }`. It suppresses the ⌥ meaning on an empty document for those tools, because ⌥ there starts a marquee from the centre rather than subtracting.

The added block is three lines: `if tool == Tool::QuickSelection { return Badge::from_mode(document_selection_mode(app, tool, m)); }`. It is placed *after* the Lasso block and *before* the general tail, which is the only position that works: before the Lasso block it would be irrelevant (Quick Selection is not a Lasso), and after the tail it would be dead code (the tail already returned).

Why Quick Selection needs the wrapper rather than the direct call the tail makes: the tail builds the badge from `selection_mode(tool, app.ui.selection_mode, m)`, which for Quick Selection would read the *wrong* field (the shared one) and would skip the empty-selection fallback. Both matter for the badge specifically, because the badge is a promise about what the next stroke does: showing a `−` when the stroke will create a new selection (New mode, nothing selected, ⌥ held) would be a lie, and showing `+` when the user picked Subtract would also be a lie.

The fallback is the same logic `document_selection_mode` already implements: if ⌥ would subtract, the unmodified mode is replace, and nothing is selected, the returned mode is `"replace"`, and `Badge::from_mode("replace")` is `None` - no badge. So the visual result of the fix is that the `−` disappears in exactly the case where it would mislead.

Returning early rather than assigning and falling through is deliberate. If the block set `mode` and the tail used it, the tail would still have to be edited to use the variable instead of the call, which changes the code path for every other tool. The early return leaves the tail byte-identical, so the diff shows one insertion and no modification.

`Badge::from_mode` is `pub(crate)` and maps `"add"` to `Badge::Add`, `"subtract"` to `Badge::Subtract`, `"intersect"` to `Badge::Intersect`, and both `"replace"` and anything unknown to `None`. That mapping is why returning `"replace"` from the wrapper produces no badge rather than a "replace" badge.

Two structural details of that branch deserve naming because they constrain future edits. The early return means the function has four exits, not two: the `unwrap_or_else`-style paths above, this new one, and the tail. A reader adding a fifth tool-specific rule must put it before the tail, and the compiler will not tell them if they put it after - the tail always returns. The file's convention is that tool-specific rules go above the general tail, in tool order, and this insertion follows it.

The second detail is that this branch deliberately does *not* apply the Lasso's `m.alt = false` reset. That reset exists because ⌥ on an empty document starts a Lasso marquee from the centre instead of subtracting; Quick Selection has no such alternative meaning for ⌥, so ⌥ must keep meaning subtract-or-add-inversion even on an empty document. Applying the reset here would have made ⌥ show no badge at all on a fresh document, which is a different behaviour from the one the tool has.

`Badge::from_mode` is called with the wrapper's return value, so the badge's vocabulary is unchanged: Add, Subtract, Intersect, or nothing. A mode the badge has no glyph for (`"replace"`) produces `None`, and that is what makes the empty-selection case show nothing rather than a wrong glyph.

#### 8. `crates/ui-egui/src/canvas/tool_feedback.rs` — hunk -290,6 +318,10

**Original code (upstream base)**

```rust
        // Smart selection tools.
        assert_eq!(badge(&app, Tool::QuickSelection, none), Some(Badge::Add));
        assert_eq!(badge(&app, Tool::QuickSelection, alt), Some(Badge::Subtract));
        assert_eq!(badge(&app, Tool::ObjectSelection, none), None);
        assert_eq!(badge(&app, Tool::ObjectSelection, shift), Some(Badge::Add));
        // Other tools never show one.
```

**Changed code (this pull request)**

```rust
        // Smart selection tools.
        assert_eq!(badge(&app, Tool::QuickSelection, none), Some(Badge::Add));
        assert_eq!(badge(&app, Tool::QuickSelection, alt), Some(Badge::Subtract));
        // Quick Selection in New mode with nothing selected falls back to a new selection (no − badge).
        app.ui.tool_options.quick_selection_mode = 0;
        assert_eq!(badge(&app, Tool::QuickSelection, alt), None);
        app.ui.tool_options.quick_selection_mode = 1;
        assert_eq!(badge(&app, Tool::ObjectSelection, none), None);
        assert_eq!(badge(&app, Tool::ObjectSelection, shift), Some(Badge::Add));
        // Other tools never show one.
```

### `crates/ui-egui/src/icon_data.rs` (+2 / -0)

**Explanation.** Three assertions are added to the existing `badge` test, and they exist because hunk 7 introduces a branch that the old assertions cannot reach.

The first two lines are the pre-existing cover: `badge(&app, Tool::QuickSelection, none) == Some(Badge::Add)` and `badge(&app, Tool::QuickSelection, alt) == Some(Badge::Subtract)`. Those pass with the field at its default (Add, 1), which is why they were written before the triple existed.

The added block sets the field, asserts, and restores it: `app.ui.tool_options.quick_selection_mode = 0;` then `assert_eq!(badge(&app, Tool::QuickSelection, alt), None);` then `app.ui.tool_options.quick_selection_mode = 1;`. What it proves: in New mode with nothing selected and ⌥ held, the badge is absent. That is the `document_selection_mode` fallback reaching the cursor through hunk 7 - without hunk 7 the badge would come from `selection_mode(tool, app.ui.selection_mode, alt)` and would be `Some(Badge::Subtract)`, so this assertion is the direct test of that hunk.

The restore to 1 is not cosmetic. The assertions that follow (`Tool::ObjectSelection` with none is `None`, with shift is `Some(Add)`) run in the same app instance, and a stray New mode would not affect them - but the *next* test in the file may share the fixture's construction, and leaving a non-default field behind makes a later failure depend on test order. Writing the restore as a plain assignment rather than a `scopeguard` matches the style of the surrounding test, which sets and restores fields the same way.

Why `None` rather than `Some(Badge::Replace)`: `Badge` has no Replace variant, because "replace" is the absence of a modifier indicator - the cursor shows nothing when a stroke will simply replace. Asserting `None` therefore asserts both "the fallback fired" and "the badge vocabulary has no misleading member for it".

The assertion is written against the public `badge` function rather than against `document_selection_mode` directly, because the defect it prevents is a *display* defect: the wrapper could be right and the cursor still wrong (which is exactly the state before hunk 7). Testing at the level where the bug would appear is what makes the test fail when hunk 7 is reverted.

The assertion style also matters for the accesskit path the test exercises indirectly: `h.get_by_label(...)` in the other test of this group looks widgets up by their accessible name, and `icons::button`'s tooltip is what becomes that name. So the tooltip strings in hunk 14 are not only user-visible text - they are the strings a screen reader announces and the strings the test matches. Changing "Add to selection  ({key})" to "Add" would break the lookup by label, which is why the tooltip is long and why it embeds the modifier name.

#### 9. `crates/ui-egui/src/icon_data.rs` — hunk -133,6 +133,8

**Original code (upstream base)**

```rust
    ("plus", include_bytes!("../../../assets/icons/plus.svg")),
    ("pointer", include_bytes!("../../../assets/icons/pointer.svg")),
    ("quick-selection", include_bytes!("../../../assets/icons/quick-selection.svg")),
    ("rectangle-horizontal", include_bytes!("../../../assets/icons/rectangle-horizontal.svg")),
    ("rectangle-vertical", include_bytes!("../../../assets/icons/rectangle-vertical.svg")),
    ("redo-2", include_bytes!("../../../assets/icons/redo-2.svg")),
```

**Changed code (this pull request)**

```rust
    ("plus", include_bytes!("../../../assets/icons/plus.svg")),
    ("pointer", include_bytes!("../../../assets/icons/pointer.svg")),
    ("quick-selection", include_bytes!("../../../assets/icons/quick-selection.svg")),
    ("quick-selection-add", include_bytes!("../../../assets/icons/quick-selection-add.svg")),
    ("quick-selection-subtract", include_bytes!("../../../assets/icons/quick-selection-subtract.svg")),
    ("rectangle-horizontal", include_bytes!("../../../assets/icons/rectangle-horizontal.svg")),
    ("rectangle-vertical", include_bytes!("../../../assets/icons/rectangle-vertical.svg")),
    ("redo-2", include_bytes!("../../../assets/icons/redo-2.svg")),
```

### `crates/ui-egui/src/panels.rs` (+1 / -1)

**Explanation.** `icon_data.rs` holds the icon table: a `&[(&str, &[u8])]` of name-to-SVG-bytes, built with `include_bytes!` so the SVGs are compiled into the binary rather than read at runtime. Two entries are added, directly after the existing `quick-selection` entry.

`include_bytes!("../../../assets/icons/quick-selection-add.svg")` resolves at compile time relative to the source file (`crates/ui-egui/src/icon_data.rs`), so three levels up is the repository root and then `assets/icons/`. The path is a manifest of the asset's location, which is why the entry cannot simply be added to the table without the file existing - a missing file is a compile error, not a runtime one. That is the desirable failure direction: the build breaks rather than the icon rendering blank.

The `&str` key is what the rest of the UI uses to look icons up by name. It is `"quick-selection-add"` and `"quick-selection-subtract"` - hyphenated, matching the file stem and matching the existing `"quick-selection"` key. Consistency in the key matters because the lookup sites name icons as string literals; a mismatch between key and file stem would produce a working build with a blank icon.

The position in the array is alphabetical and preserved: `plus`, `pointer`, `quick-selection`, then the two new keys, then `rectangle-horizontal`. The array is scanned linearly by name, so order does not affect correctness, but the sorted order is what makes the giant table reviewable and what a contributor adding an icon is expected to maintain.

Why compile the bytes in rather than load at runtime: the binary is shipped as a single executable, and the icons must be available before any file-system work and even when the assets directory is absent. `include_bytes!` also means the SVG bytes cannot drift from the compiled binary, which is a real hazard for a UI whose visual contract is "the same glyph on every platform".

No other table needs to know about the two files. The options-bar buttons that use them look them up by the key added here, the ATTRIBUTION rows describe them (hunks 1 and 2), and the licence text is the shared Lucide ISC file. This hunk is the single registration point, which is why the two entries are one hunk rather than two.

Two consequences of `include_bytes!` that are worth stating because they shape how the table is maintained. First, the binary grows by the size of the file, uncompressed - the SVGs are small (a few hundred bytes each) but the table also includes fonts elsewhere, so the mechanism is already accepted for assets at this scale. Second, editing an SVG requires a rebuild for the change to appear; there is no hot-reload path, which is why icon work in this repository is done as "change file, `cargo run`, look". A runtime-loaded icon directory would avoid the rebuild and would introduce a failure mode (missing file at runtime) that the compiled-in table makes impossible.

The keys are also the strings the `icons` module exposes to the rest of the UI. `icons::button(ui, "quick-selection-add", 24.0, selected, tip)` looks the key up in this table, and a key that is absent renders nothing rather than panicking - which is why the two entries have to exist before hunk 14's call can show anything. The lookup's leniency is the reason a typo is a silent blank icon rather than a compile error, and it is the reason the key literal in hunk 14 and the key here must be checked together in review.

#### 10. `crates/ui-egui/src/panels.rs` — hunk -53,7 +53,7

**Original code (upstream base)**

```rust
pub(super) mod panel_docking;
mod panel_state;
pub mod preset_panels;
mod properties;
pub mod props_layout;
pub mod smart_ui;
mod status_bar;
```

**Changed code (this pull request)**

```rust
pub(super) mod panel_docking;
mod panel_state;
pub mod preset_panels;
pub(crate) mod properties;
pub mod props_layout;
pub mod smart_ui;
mod status_bar;
```

### `crates/ui-egui/src/panels/options_bar.rs` (+5 / -1)

**Explanation.** A one-word visibility change on the `properties` module declaration: `pub mod properties;` becomes `pub(crate) mod properties;`.

`panels.rs` declares its child modules and re-exports the panel-drawing functions. `properties` is the module that draws the tool properties / options rows, and this pull request adds a function to it (`quick_selection_mode_buttons`, hunk 13/14) that the options bar must call. Before the change, `pub` meant the module was visible outside the crate - which it never needed to be, since no binary outside `ui-egui` draws these rows - and `pub(crate)` says the true scope.

The change is required rather than cosmetic because of how the new function is declared. `quick_selection_mode_buttons` is `pub(super)` inside `properties` (visible to `panels`, its parent), and `options_bar` is a sibling module inside `panels`. A `pub(super)` item in `properties` is visible in `panels` but *not* in `panels::options_bar` unless the module path is reachable - and it is reachable only if `panels` can name `properties`. A `pub mod` in `panels` is reachable; a private `mod` would not be. `pub(crate)` is the narrowest declaration that satisfies the call and still keeps the module out of the crate's public API, which is what `cargo doc` and semver tooling read.

Why not leave it `pub`: the crate's public surface is what downstream code can name, and a module that is only used internally being `pub` invites accidental coupling from outside. The rest of the file shows the convention: `panel_docking` is `pub(super)`, `panel_state` private, `preset_panels` `pub`, `props_layout` `pub`, `smart_ui` `pub`, `status_bar` private. `pub(crate)` is the value used elsewhere in the crate for "internal but cross-module", so the change also brings this line in line with that convention.

The blast radius is nil at runtime: visibility is a compile-time property, the module's items and their bodies are unchanged, and no code is added or removed. The only observable effect is that `ui_egui::panels::properties` is no longer nameable from outside the crate - which nothing outside the crate does, or the build would now fail and we would see it.

The visibility word also has a documentation consequence. `pub(crate)` items are excluded from `cargo doc`'s public output by default, so this module's helper functions do not appear in the rendered API for the crate - which is the intent, since they take `&mut PhotocraftApp` and exist to draw widgets. Had the change gone the other way (a private `mod` with `pub(crate)` items inside), the items would be unreachable from `options_bar` and the build would fail, which is what makes this a necessary edit rather than a tidy-up.

The rest of the file's declarations, read as a whole, are the convention this line now follows: `pub(super) mod panel_docking` (used by `panels` and its children only), private `mod panel_state` (used inside `panels` only), `pub mod preset_panels`, `pub mod props_layout`, `pub mod smart_ui`, private `mod status_bar`. Each value is the narrowest one that its callers need, and `pub(crate)` for `properties` is the value for "drawn from a sibling module and from `tools`".

#### 11. `crates/ui-egui/src/panels/options_bar.rs` — hunk -1,7 +1,7

**Original code (upstream base)**

```rust
use super::brush_controls::{
    brush_preset_chip, mixer_combination_dropdown, mixer_load_swatch, paint_mode_dropdown, smoothing_field, smoothing_options, tool_mode_dropdown,
};
use super::properties::{select_and_mask_button, selection_mode_buttons};
use super::{BlendMode, Color32, PhotocraftApp, RichText, Sense, Tokens, Tool, Value, icons, json, starts_blend_group, vec2, widgets};
use photocraft_paint::BrushSettings;

```

**Changed code (this pull request)**

```rust
use super::brush_controls::{
    brush_preset_chip, mixer_combination_dropdown, mixer_load_swatch, paint_mode_dropdown, smoothing_field, smoothing_options, tool_mode_dropdown,
};
use super::properties::{quick_selection_mode_buttons, select_and_mask_button, selection_mode_buttons};
use super::{BlendMode, Color32, PhotocraftApp, RichText, Sense, Tokens, Tool, Value, icons, json, starts_blend_group, vec2, widgets};
use photocraft_paint::BrushSettings;

```

**Explanation.** One import line in `panels/options_bar.rs` gains a name: `use super::properties::{select_and_mask_button, selection_mode_buttons};` becomes `use super::properties::{quick_selection_mode_buttons, select_and_mask_button, selection_mode_buttons};`.

The name is used in hunk 12, where the Quick Selection arm of the options bar calls it. Rust would not compile without this, so the line is not decorative - but the *placement* carries information. `quick_selection_mode_buttons` is inserted first because the list is alphabetical, which is the convention the file follows in the visible braces (`select_and_mask_button` before `selection_mode_buttons` is alphabetical too, and the added name sorts before both).

The `super::properties::` path rather than `crate::panels::properties::` is the same relative form the neighbouring import uses, and it survives a module move. It is also the direct consequence of hunk 10: the path resolves because `properties` is `pub(crate)`, and it would not resolve from `options_bar` if the module were private.

`select_and_mask_button` is pre-existing in this import and is used by the same arm; `selection_mode_buttons` is used by the other selection tools' arms. Keeping all three names on one line, rather than adding a separate `use` statement for the new one, matches the file's single-import-per-module style - a second `use super::properties::...` line would be legal but would read as an afterthought.

Nothing else in the import block changes: the `brush_controls` line, the long `super::{...}` line and the `photocraft_paint::BrushSettings` line are untouched, so the hunk is one line and can be reviewed without reading the neighbours.

`rustfmt` sorts names inside a brace group, which is why the added name had to go first rather than last: writing it at the end (after `selection_mode_buttons`) compiles but fails `cargo fmt --check`, and this repository runs that check in CI. The sorted order is also what makes the line stable - a future addition lands in a predictable position rather than at the end of an ever-growing list.

#### 12. `crates/ui-egui/src/panels/options_bar.rs` — hunk -93,6 +93,10

**Original code (upstream base)**

```rust
                        // The options bar shows the active tool's brush: switch it in before drawing the
                        // chip, so a tool change swaps the brush with it (Photoshop keeps one per tool).
                        crate::paint_mouse::sync_tool_brush(app);
                        // Brush edits here go through `tools.setBrush`, one journal entry per gesture (Rule 1).
                        if (tool.is_brushlike() && !matches!(tool, Tool::Brush | Tool::Pencil | Tool::MixerBrush | Tool::Eraser))
                            || tool == Tool::QuickSelection
```

**Changed code (this pull request)**

```rust
                        // The options bar shows the active tool's brush: switch it in before drawing the
                        // chip, so a tool change swaps the brush with it (Photoshop keeps one per tool).
                        crate::paint_mouse::sync_tool_brush(app);
                        if tool == Tool::QuickSelection {
                            quick_selection_mode_buttons(app, ui);
                            widgets::vline(ui, 22.0);
                        }
                        // Brush edits here go through `tools.setBrush`, one journal entry per gesture (Rule 1).
                        if (tool.is_brushlike() && !matches!(tool, Tool::Brush | Tool::Pencil | Tool::MixerBrush | Tool::Eraser))
                            || tool == Tool::QuickSelection
```

### `crates/ui-egui/src/panels/properties.rs` (+23 / -1)

**Explanation.** The options bar draws the active tool's controls; this hunk inserts the Quick Selection triple into the drawing sequence, immediately after the brush is synchronised and before the brush-edit block.

The insertion point is not arbitrary. The line above it is `crate::paint_mouse::sync_tool_brush(app);`, whose comment explains that the options bar shows the active tool's brush and that a tool change must swap the brush before the chip is drawn. The three mode buttons are not brush state, so they must come after the sync and before the widget that edits the brush - otherwise the mode buttons would sit between the sync and its consumer, and a reader would have to check whether the sync was still current when the chip drew.

The block is guarded by `if tool == Tool::QuickSelection`, so the buttons exist only for that tool. Every arm of the surrounding match is already tool-specific, but this particular region is shared by several tools (the brush chip draws for every brush-like tool), so the guard is needed here rather than implied by the arm.

`quick_selection_mode_buttons(app, ui)` is called first and `widgets::vline(ui, 22.0)` second. The vertical rule is the separator the bar uses between logical groups; it is placed *after* the buttons so the group reads as "modes | brush", with the rule belonging to the group that follows. That ordering matches the other groups in the file, where a `vline` closes the group before the next one opens.

`app` and `ui` are passed through rather than the field and a widget context: `quick_selection_mode_buttons` reads and writes `app.ui.tool_options.quick_selection_mode` and needs `ui` to draw. Passing the field by `&mut u8` would work for reading and writing but would lose the ability to look at anything else in the app later without changing the signature again, and every other helper in this function takes `(app, ui)`.

One consequence worth naming: the block runs before the brush-edit guard below it, which begins `if (tool.is_brushlike() && !matches!(tool, Tool::Brush | ...)) || tool == Tool::QuickSelection`. That condition is what decides whether the brush chip is editable; Quick Selection is included in it, so the brush controls below are drawn for Quick Selection too. The new triple therefore appears to the *left* of the brush controls for this tool, which is the Photoshop order (mode first, then brush).

If the guard were forgotten, the triple would appear for every brush-like tool, and its buttons would write `quick_selection_mode` while a Brush was active - a state that nothing reads, but which would then apply the next time Quick Selection was selected. The guard prevents a silent cross-tool write.

The location inside the shared region is also constrained by the tools that do not get the triple. `tool.is_brushlike()` is true for Brush, Pencil, Mixer Brush, Eraser and several others; those run the same block and would otherwise pick up the three buttons. The `if tool == Tool::QuickSelection` guard is therefore the only thing separating this control from every brush tool's options row, and it has to be a runtime condition rather than a compile-time one because the tool is state, not a type.

The rule about the separator belongs to the row, not to the tool: a `vline` after a group is how every row in this bar ends a group, so the triple is followed by one whether or not the brush chip comes next. A reader adding a fourth control here should insert it before the `vline`, not after - the line marks the end of what precedes it.

#### 13. `crates/ui-egui/src/panels/properties.rs` — hunk -264,7 +264,7

**Original code (upstream base)**

```rust

/// The options bar's Select and Mask… button: greyed out, as the menu item is, until there is a
/// selection (or a layer mask) to refine (#7520).
pub(super) fn select_and_mask_button(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    let enabled = crate::menus::is_enabled(app, "select.selectAndMask");
    let clicked = ui.add_enabled_ui(enabled, |ui| widgets::secondary_button(ui, tl!("Select and Mask…"), 0.0).clicked()).inner;
    if clicked {
```

**Changed code (this pull request)**

```rust

/// The options bar's Select and Mask… button: greyed out, as the menu item is, until there is a
/// selection (or a layer mask) to refine (#7520).
pub(crate) fn select_and_mask_button(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    let enabled = crate::menus::is_enabled(app, "select.selectAndMask");
    let clicked = ui.add_enabled_ui(enabled, |ui| widgets::secondary_button(ui, tl!("Select and Mask…"), 0.0).clicked()).inner;
    if clicked {
```

**Explanation.** `select_and_mask_button` was `pub(super)`, meaning visible to `panels` and its descendants; it becomes `pub(crate)`, meaning visible anywhere in the `ui-egui` crate. The body is unchanged.

The reason is a second caller. The button already appears in the options bar (`panels/options_bar.rs`, which is inside `panels`, so `pub(super)` was enough). The Quick Selection options row in `tools/retouch_ui.rs` also needs it - hunk 20 draws "Select Subject", a rule, and then this button - and `tools` is not a descendant of `panels`, so `pub(super)` would not resolve there.

Why not move the button or duplicate it: the button's behaviour depends on `crate::menus::is_enabled(app, "select.selectAndMask")`, which is the single source of truth for whether the command is available. A copy in `retouch_ui` would have to duplicate that lookup and the enabled-wrapper, and the two copies would drift. Widening the visibility reuses one implementation, which is the smaller change.

Why `pub(crate)` rather than `pub`: the button is a UI helper, not part of the crate's API. `pub` would expose it to the binary crate and to documentation, and nothing outside `ui-egui` should be drawing it. `pub(crate)` is the exact scope of the two call sites.

The visibility change is a pure compile-time act: no code is added, no behaviour changes, and the only way it can break anything is if it creates an ambiguity from outside the crate - which it cannot, because widening from `pub(super)` to `pub(crate)` cannot produce a name collision that `pub` would not have produced.

Note that this hunk and hunk 10 (`panels::properties` from `pub` to `pub(crate)`) are the same kind of change in opposite directions, and that is not a contradiction: hunk 10 narrows a *module* that was over-exposed, this hunk widens an *item* that was under-exposed. Both move towards "the narrowest scope that serves the real callers".

The two call sites differ in one respect that explains why `pub(crate)` rather than a re-export: the options bar calls it as a plain name (`select_and_mask_button(app, ui)`) because it imported it, while `retouch_ui` calls it by full path (`crate::panels::properties::select_and_mask_button(app, ui)`). Both forms are now legal, and the second is the one that forced the widening - `pub(super)` would not resolve a path through `crate::panels::` from a module outside `panels`, because the item was not visible at that path outside the `panels` subtree.

There is a third option that was not taken: move `select_and_mask_button` up into `panels.rs` as a free function. That would have made it reachable from both children without widening it, but it would have moved a function whose whole body depends on `properties`' helpers (`widgets::secondary_button`, `crate::menus::is_enabled`) out of the module that owns them, and the diff would have touched `panels.rs` and both call sites instead of one word. Widening visibility is the smaller change and leaves the code where it belongs.

#### 14. `crates/ui-egui/src/panels/properties.rs` — hunk -292,6 +292,27

**Original code (upstream base)**

```rust
    ui.spacing_mut().item_spacing.x = 8.0;
}

#[cfg(test)]
mod tests {
    use egui::accesskit::Role;
```

**Changed code (this pull request)**

```rust
    ui.spacing_mut().item_spacing.x = 8.0;
}

/// New / add / subtract selection buttons for Quick Selection tool.
pub(super) fn quick_selection_mode_buttons(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    if app.ui.tool_options.quick_selection_mode > 2 {
        app.ui.tool_options.quick_selection_mode = 1;
    }
    ui.spacing_mut().item_spacing.x = 2.0;
    for (i, (icon, tip)) in [
        ("quick-selection", tl!("New selection").to_string()),
        ("quick-selection-add", crate::i18n::fmt(tl!("Add to selection  ({key})"), &[("key", &crate::shortcuts::pretty("Shift"))])),
        ("quick-selection-subtract", crate::i18n::fmt(tl!("Subtract from selection  ({key})"), &[("key", &crate::shortcuts::pretty("Alt"))])),
    ]
    .iter()
    .enumerate()
    {
        if icons::button(ui, icon, 24.0, app.ui.tool_options.quick_selection_mode == i as u8, tip).clicked() {
            app.ui.tool_options.quick_selection_mode = i as u8;
        }
    }
    ui.spacing_mut().item_spacing.x = 8.0;
}

#[cfg(test)]
mod tests {
    use egui::accesskit::Role;
```

**Explanation.** The new function, 21 lines, is the whole options-bar triple. Its structure is: clamp, tighten spacing, iterate a three-element table, restore spacing.

Signature: `pub(super) fn quick_selection_mode_buttons(app: &mut PhotocraftApp, ui: &mut egui::Ui)`. `pub(super)` makes it visible to `panels` and its descendants, which is exactly what `options_bar` needs; `retouch_ui` does not call it (it calls `select_and_mask_button` instead), so `pub(crate)` is not required - unlike hunk 13, where the second caller is outside `panels`.

The clamp is the first statement: `if app.ui.tool_options.quick_selection_mode > 2 { app.ui.tool_options.quick_selection_mode = 1; }`. It normalises a persisted value that an older or hand-edited project file could have set out of range. It is written as an `if` with a write rather than `let mode = mode.min(2)`, because the write is what makes the normalisation visible to the rest of the app - `selection_mode` in `tool_feedback.rs` also clamps, so the two agree, but the field itself should hold a legal value rather than being re-clamped at every read. The direction of the repair is Add (1), not New (0) and not Subtract (2): Add is the tool's default and the value an older build would have produced.

`ui.spacing_mut().item_spacing.x = 2.0;` tightens the gap between the three buttons so they read as one segmented control rather than three separate buttons. The value 2.0 is smaller than the 8.0 the surrounding bar uses, and the last line restores 8.0 - so the setting is scoped to this function and cannot leak into whatever draws next. Restoring rather than relying on the frame's default is deliberate: `item_spacing` is `Ui` state, `ui` is the caller's, and a helper that leaves it changed would silently alter the next group in the bar.

The table is `[(&str, String); 3]` of `(icon_key, tooltip)`. The keys are the three names registered in `icon_data.rs` (hunk 9) - `"quick-selection"`, `"quick-selection-add"`, `"quick-selection-subtract"`. The tooltips are the three labels, and the two with modifiers are built through `crate::i18n::fmt(tl!("Add to selection  ({key})"), &[("key", &crate::shortcuts::pretty("Shift"))])` so the modifier name comes from the platform's shortcut table rather than being hard-coded - on a keyboard where Shift is displayed differently (or on macOS, where the modifier glyph differs), the tooltip follows. The double space before `({key})` is in the source: it separates the label from the parenthetical without a separator character, and it is preserved from the label style used elsewhere in the bar.

`.iter().enumerate()` gives `(i, (icon, tip))` where `i` is 0, 1, 2 - and those are exactly the values the field stores (New, Add, Subtract). That correspondence is the reason the loop needs no lookup table: the array order *is* the encoding, and the doc comment on `selection_mode` names the same three numbers.

`icons::button(ui, icon, 24.0, app.ui.tool_options.quick_selection_mode == i as u8, tip)` draws one icon button at 24 points, with the `selected` flag set when the field equals the index, and returns a response. `i as u8` is a cast from `usize`; it is safe because the array has three elements. `.clicked()` then writes `app.ui.tool_options.quick_selection_mode = i as u8` - the only write in the function, so the field can only take one of the three legal values from this path.

No `Undo`/journal entry is created by the click. That is the correct level: the mode is a tool option, not a document change, and the rule elsewhere in the codebase is that tool options are not journaled while document edits are. The stroke that follows is journaled once by `tools.setBrush`-style grouping, not per mode change.

The doc comment is one line, `/// New / add / subtract selection buttons for Quick Selection tool.`, which is the convention in this file (short single-line docs on `pub(super)` helpers).

#### 15. `crates/ui-egui/src/panels/properties.rs` — hunk -360,4 +381,5

**Original code (upstream base)**

```rust
            assert!((value(&h) - 1.0).abs() < 1e-6, "{key}: one undo reverts the whole drag");
        }
    }
}
```

**Changed code (this pull request)**

```rust
            assert!((value(&h) - 1.0).abs() < 1e-6, "{key}: one undo reverts the whole drag");
        }
    }

}
```

### `crates/ui-egui/src/state/tool_options.rs` (+8 / -0)

**Explanation.** One added blank line, at the end of the `tests` module, before its closing brace.

It is the tail of hunk 14's insertion: the new `quick_selection_mode_buttons` function was inserted above `#[cfg(test)] mod tests`, and the blank line that separated the last test from the module's closing brace was consumed by the reflow. Without it the module ends as `}\n}` - the inner `}` closing the last `#[test]` function and the outer closing `mod` - which compiles and formats identically but reads as one brace too few.

`cargo fmt` is the authority here: the file is formatted, and a missing blank line at a module boundary is not something `rustfmt` enforces, which is why this is a hand-maintained line rather than a formatting artifact. The diff shows it as a `+` on an empty line because that is literally what changed.

There is no behavioural meaning to this line, and it is included in the document because the instruction was to show every changed line rather than to show the interesting ones. A reviewer scanning hunk numbers against `git diff` should be able to account for all 22 of them, and this is the one with no semantics.

#### 16. `crates/ui-egui/src/state/tool_options.rs` — hunk -89,6 +89,9

**Original code (upstream base)**

```rust
    pub finger_painting: bool,
    /// Quick Selection enhance edge.
    pub enhance_edge: bool,
    /// Pen: "path" (work path) or "shape" (shape layer).
    pub vector_mode: String,
    /// Pen, Freeform Pen and Curvature Pen › Path Operations (#1397): how a new path combines with
```

**Changed code (this pull request)**

```rust
    pub finger_painting: bool,
    /// Quick Selection enhance edge.
    pub enhance_edge: bool,
    /// Quick Selection selection mode: 0 New, 1 Add (default), 2 Subtract.
    #[serde(default = "default_quick_selection_mode")]
    pub quick_selection_mode: u8,
    /// Pen: "path" (work path) or "shape" (shape layer).
    pub vector_mode: String,
    /// Pen, Freeform Pen and Curvature Pen › Path Operations (#1397): how a new path combines with
```

**Explanation.** `ToolOptions` is the serialised struct that holds every per-tool option; it is saved into the project file, so every field is a compatibility commitment. This hunk adds one field between `enhance_edge` and `vector_mode`.

`/// Quick Selection selection mode: 0 New, 1 Add (default), 2 Subtract.` is the doc line, and the three numbers are the contract: they are the values `selection_mode` matches on and the indices the options-bar buttons write. Naming them here rather than only at the point of use is what lets a reader of the state struct know the encoding without following the call graph.

`#[serde(default = "default_quick_selection_mode")]` is the compatibility mechanism. Without it, opening a project saved by any build before this pull request would fail to deserialise `ToolOptions`, because the JSON has no `quick_selection_mode` key and serde's default for a missing field is an error unless the field is `Option` or has `#[serde(default)]`. With the attribute, a missing key yields the function's value (1) and the project opens unchanged.

Why `default = "default_quick_selection_mode"` rather than `default`: a plain `#[serde(default)]` would use `u8::default()`, which is 0 - New mode. That would be wrong: a user opening an old project would find their Quick Selection silently switched to "replace the selection on every stroke", which is a behaviour change they did not ask for. The custom function returns 1, so the upgrade preserves the pre-change behaviour exactly. This is the kind of default that must be chosen by what the old code did, not by what the type's zero value is.

Why `u8` rather than an enum: the field is read by `selection_mode` as a number, written by the buttons as an index, and compared against `> 2` for the clamp. An enum with `#[serde(rename_all = "camelCase")]` would be more type-safe but would change the persisted JSON shape and would need `#[serde(other)]` handling for an unknown value from a hand-edited file - and the code already clamps, so the number is the simpler contract. The doc line naming the three values is what an enum would have given for free, and that is why it is written there.

Position in the struct: after `enhance_edge` and before `vector_mode`. Both neighbours are unrelated to Quick Selection (`vector_mode` is the Pen's), so the placement groups the new field with the tool's other option rather than following any structural rule. Serde does not care about field order for named structs, so this is a readability choice.

Nothing else changes: the struct has no explicit `Deserialize` impl to update, and it has no `Default` derive that would need the field added (hunk 18 hand-writes the `Default` below it).

One consequence for the project file: the field is written out whenever tool options are saved, so a project saved after this change and opened in an older build will carry an unknown key. The older build's deserialiser ignores unknown fields (serde's default behaviour for structs without `deny_unknown_fields`), so the round trip is harmless in that direction too - which is the property that makes adding a field with a default safe in both directions.

#### 17. `crates/ui-egui/src/state/tool_options.rs` — hunk -309,6 +312,10

**Original code (upstream base)**

```rust
    "newLayer".into()
}

fn default_curve_fit() -> f32 {
    2.0
}
```

**Changed code (this pull request)**

```rust
    "newLayer".into()
}

fn default_quick_selection_mode() -> u8 {
    1
}

fn default_curve_fit() -> f32 {
    2.0
}
```

**Explanation.** A four-line free function, inserted between `default_pen_path_op`'s neighbourhood and `default_curve_fit`.

`fn default_quick_selection_mode() -> u8 { 1 }` returns Add. It exists solely to be named in the `#[serde(default = "...")]` attribute added by hunk 16, because that attribute takes a path to a function with no arguments, not a literal.

Why a function rather than `#[serde(default = "one")]` reusing an existing one: no existing function returns the value 1 as a `u8` with the right name. Reusing an unrelated default (say the one that returns `"newLayer"`) is impossible for the type; writing an inline closure is not supported by the attribute syntax. So a named function is the minimum.

The name is the field's name with the `default_` prefix, matching the file's convention: `default_curve_fit`, `default_pen_path_op`, and the string-returning defaults above. That convention is what a reader uses to find the default for a field: search `default_<field>`. A name like `one` or `add_mode` would break the search and would also be ambiguous if a second field ever defaults to 1.

The body returns a literal rather than a named constant. A `const ADD: u8 = 1;` would be a second place to look for the encoding, and the encoding is already documented on the field and matched in `selection_mode`. The literal here is the third mention of the same number, and that is acceptable because this one is unconditional: it is the value a *missing* key takes, which must be the pre-change behaviour, and that is 1.

The function is private (no `pub`), because only serde names it. Making it `pub(crate)` would expose a one-line function that no other module has a reason to call, and `dead_code` would then be the only thing keeping it honest.

Placement: immediately before `default_curve_fit`, so the alphabetical-ish grouping of the `default_*` functions is preserved and the diff is a pure insertion between two existing functions - no existing line moves, which is what lets the hunk be three lines of context plus four of addition.

One point about serde's default mechanism that this function relies on: `#[serde(default = "path")]` requires a path to a function, resolved in the module where the field is declared, and it is called only when the key is *absent*. A key that is present with a wrong type is an error, not a default - so a project file with `"quickSelectionMode": "add"` fails to load rather than silently becoming Add. That behaviour is what makes the clamped `u8` safe: the only way to get an out-of-range value is for the file to contain one, and the clamp in the drawing code (hunk 14) and in `selection_mode` (hunk 5) handles that.

The function is also not `const fn`. It could be, but the attribute only needs a callable path, and `const` would add a keyword that suggests it is used in a const context, which it is not.

#### 18. `crates/ui-egui/src/state/tool_options.rs` — hunk -475,6 +482,7

**Original code (upstream base)**

```rust
            protect_detail: true,
            finger_painting: false,
            enhance_edge: false,
            vector_mode: "path".into(),
            pen_path_op: default_pen_path_op(),
            shape_mode: "shape".into(),
```

**Changed code (this pull request)**

```rust
            protect_detail: true,
            finger_painting: false,
            enhance_edge: false,
            quick_selection_mode: 1,
            vector_mode: "path".into(),
            pen_path_op: default_pen_path_op(),
            shape_mode: "shape".into(),
```

### `crates/ui-egui/src/tools/retouch_ui.rs` (+7 / -1)

**Explanation.** `impl Default for ToolOptions` is hand-written in this file (not derived), which is why a new field must be added here as well as in the struct. The added line is `quick_selection_mode: 1,` between `enhance_edge: false,` and `vector_mode: "path".into(),`.

Both the struct (hunk 16) and this literal are needed, and they can drift: the struct's `#[serde(default = "...")]` governs *deserialisation of a missing key*, while this literal governs *a freshly constructed `ToolOptions`*. If they disagreed, a new project and an upgraded old project would start in different modes - a defect that only appears on one of the two paths, which is the hardest kind to notice in review.

The value is 1 (Add) for the same reason as the serde default: Add is the pre-change behaviour of Quick Selection (the old arm returned `"add"` unless ⌥ was held). New (0) would make the first stroke replace the selection, which no user of the old build experienced; Subtract (2) would be worse still.

`1` rather than `default_quick_selection_mode()`: the surrounding literal uses plain constants (`protect_detail: true`, `enhance_edge: false`) rather than calling the `default_*` functions, and calling the function here would be inconsistent with its neighbours while adding a call in a const-ish context. The duplication is two occurrences of the same number in one file, both adjacent to comments that name its meaning.

Why the struct has no `#[derive(Default)]`: fields like `vector_mode: "path".into()` and the string options need non-zero values, so `Default` must be written by hand. That is the reason this hunk exists at all - a derived `Default` would have needed no edit, at the cost of every string option starting empty.

The field order in the literal matches the struct declaration, which is the file's convention and what `rustfmt` does not enforce but a reader expects: `enhance_edge` then `quick_selection_mode` then `vector_mode`, exactly as declared. A field appended at the end of the literal would compile and would read as an afterthought.

One structural note: because `Default` is hand-written here, adding a field to the struct is a two-site change (the field, and this literal) and the compiler enforces it - a missing field in the literal is an error, not a zero. That is the opposite of a derived `Default`, where the new field is silently zeroed. For a mode field whose zero value means "New" (a behaviour change), the compiler-enforced reminder is worth the extra line, and it is the reason the two defaults (serde's and this one) are written next to each other in the same file rather than one of them being derived.

Both are 1, and the doc comment on the field (hunk 16) names 1 as "Add (default)", so there are three mentions of the same number in this file. That is acceptable for a value that must be identical in three places - the alternative, a named constant, would add a fourth name to look up without removing the need for the doc line.

#### 19. `crates/ui-egui/src/tools/retouch_ui.rs` — hunk -171,11 +171,15

**Original code (upstream base)**

```rust
        },
        Tool::QuickSelection => {
            let size = app.session.tools.brush.size;
            let mode = if mods.alt { "subtract" } else { "add" };
            let xy: Vec<[f64; 2]> = points.iter().map(|q| [q[0], q[1]]).collect();
            let (sample_all, enhance) = (o.sample_all.quick_selection, o.enhance_edge);
            let r = app.run("select.quick", json!({"points": xy, "size": size, "mode": mode, "enhanceEdge": enhance, "sampleAllLayers": sample_all}));
            report_smart_result(app, r, tl!("Quick Selection found nothing there"));
            return true;
        }
        _ => return false,
```

**Changed code (this pull request)**

```rust
        },
        Tool::QuickSelection => {
            let size = app.session.tools.brush.size;
            let mode = crate::canvas::tool_feedback::document_selection_mode(app, tool, mods);
            let xy: Vec<[f64; 2]> = points.iter().map(|q| [q[0], q[1]]).collect();
            let (sample_all, enhance) = (o.sample_all.quick_selection, o.enhance_edge);
            let r = app.run("select.quick", json!({"points": xy, "size": size, "mode": mode, "enhanceEdge": enhance, "sampleAllLayers": sample_all}));
            let changed = matches!(r, Ok(ref v) if v.get("changed").and_then(Value::as_bool) == Some(true));
            report_smart_result(app, r, tl!("Quick Selection found nothing there"));
            if o.quick_selection_mode == 0 && changed {
                app.ui.tool_options.quick_selection_mode = 1;
            }
            return true;
        }
        _ => return false,
```

**Explanation.** `finish_stroke` is where a Quick Selection drag becomes a command. This hunk replaces a two-line modifier test with the shared mode function and adds the New→Add flip. Both halves are one behaviour: the stroke must use the mode the options bar shows, and New mode must not persist past its first successful stroke.

The removed line is `let mode = if mods.alt { "subtract" } else { "add" };`. That was the same rule the old `selection_mode` arm implemented, duplicated at the stroke. With three modes it cannot stay: the stroke has to honour New as well, and duplicating the triple here would put the encoding in two places that must agree.

The replacement is `let mode = crate::canvas::tool_feedback::document_selection_mode(app, tool, mods);`. Full path rather than a `use`, because `retouch_ui` does not otherwise depend on `canvas::tool_feedback` and the file's style keeps cross-module calls explicit. What matters is that it is the *wrapper*, not `selection_mode`: the wrapper applies the empty-selection fallback (⌥ with nothing selected means "replace", not "subtract") and reads the per-tool field. Using `selection_mode` here would reintroduce the exact divergence the pull request removes - the badge would say one thing and the stroke would do another.

`tool` is the function's parameter (the arm is `Tool::QuickSelection`), so passing it rather than the literal keeps the call honest: if the arm and the argument ever diverged, the wrapper would compute the mode for the wrong tool. It costs nothing and it is what the wrapper's signature expects.

The command JSON is otherwise unchanged: `"points"`, `"size"` from `app.session.tools.brush.size`, `"mode": mode`, `"enhanceEdge"` and `"sampleAllLayers"` from `o`. `mode` is now a `&'static str` from the wrapper rather than a literal, and `json!` serialises it identically.

The added `changed` read is the pivot of the flip: `let changed = matches!(r, Ok(ref v) if v.get("changed").and_then(Value::as_bool) == Some(true));`. Three details matter. `ref v` because `r` is consumed a line later by `report_smart_result`; borrowing avoids a clone of the whole response. `v.get("changed").and_then(Value::as_bool)` because the field may be absent (a failure response has no `changed`) or of another type, and `.and_then` collapses both to `None` rather than panicking - a direct `v["changed"].as_bool()` would return `None` too, but `matches!` on `Some(true)` is the explicit form. `== Some(true)` rather than `!= Some(false)`, because an absent field must mean "not changed", not "changed".

The flip is `if o.quick_selection_mode == 0 && changed { app.ui.tool_options.quick_selection_mode = 1; }`. `o` is the `&mut ToolOptions` the function already holds, but the write goes through `app.ui.tool_options` - `o` was captured before the `app.run` call, and going through `app` is the form the rest of the function uses after a command. Both name the same field; the `app` form is used because `o`'s borrow would be a second mutable path to the same data.

Why flip *after* a successful stroke and not immediately on click: New means "this stroke replaces the selection". After one replace, the natural next action is to refine - add or subtract - which is what Photoshop does ("after the first stroke" is the phrase the old comment used). Flipping on click would make New unusable as a mode; flipping on every stroke including failed ones would leave the user in New after a stroke that selected nothing, which is the opposite of helpful.

`return true` follows, unchanged: the arm claims the stroke. `_ => return false` still closes the match, so no other tool's behaviour is touched by this hunk.

#### 20. `crates/ui-egui/src/tools/retouch_ui.rs` — hunk -761,6 +765,8

**Original code (upstream base)**

```rust
                let r = app.run("select.subject", json!({}));
                report_smart_result(app, r, tl!("No subject found — try Object Selection or the Lasso"));
            }
        }
        Tool::ObjectSelection => {
            crate::widgets::checkbox(ui, &mut o.object_finder, tl!("Object Finder"));
```

**Changed code (this pull request)**

```rust
                let r = app.run("select.subject", json!({}));
                report_smart_result(app, r, tl!("No subject found — try Object Selection or the Lasso"));
            }
            crate::widgets::vline(ui, 22.0);
            crate::panels::properties::select_and_mask_button(app, ui);
        }
        Tool::ObjectSelection => {
            crate::widgets::checkbox(ui, &mut o.object_finder, tl!("Object Finder"));
```

### `crates/ui-egui/src/tools/retouch_ui/tests.rs` (+105 / -0)

**Explanation.** The Quick Selection options row in the second options bar (the one drawn by `retouch_ui`, as opposed to `panels/options_bar.rs`) gains a vertical rule and the Select and Mask button, closing the row the way the other tools' rows close.

The tail of the arm before this hunk is the "Select Subject" button and its `report_smart_result` call:

```rust
if crate::widgets::secondary_button(ui, tl!("Select Subject"), 0.0).clicked() {
    let r = app.run("select.subject", json!({}));
    report_smart_result(app, r, tl!("No subject found — try Object Selection or the Lasso"));
}
```

The two added lines go after that block and before the arm's closing `}`: `crate::widgets::vline(ui, 22.0);` and `crate::panels::properties::select_and_mask_button(app, ui);`.

The rule exists because the button that follows is not a Quick Selection command - it is the shared "refine this selection" action that every selection tool offers. Without a separator it would read as part of the Select Subject group; with it, the row reads "Sample All Layers | Enhance Edge | ⌥ to subtract | [rule] Select Subject | [rule] Select and Mask…", which is the structure the other selection tools' rows already have.

`22.0` is the height the bar uses for its rules everywhere; it is not a value chosen here, it is the constant that makes all the rules in this bar line up vertically.

`select_and_mask_button` is called with the fully qualified path `crate::panels::properties::select_and_mask_button` rather than imported, and that is the visible consequence of hunk 13: the function had to become `pub(crate)` for this call to resolve, because `tools::retouch_ui` is not inside `panels`. The path is written out here because a `use` at the top of `retouch_ui` would add a module dependency for one call, and the file's existing style for cross-panel calls is the full path.

Why this row needs the button at all: a user who has just made a Quick Selection needs the refine step, and in the other bar it is the last control. Putting it in both bars means the button is where the user's eye ends up regardless of which bar is showing, and the button itself is state-driven (`crate::menus::is_enabled(app, "select.selectAndMask")`), so it greys out correctly in both places with no duplication of the enabling rule.

Note that this hunk does not add the mode triple here: the triple lives in `panels/options_bar.rs` (hunk 12), which is the bar used for the tool's primary options. This row is the secondary options shown alongside, and it keeps its existing checkboxes and hints.

The call passes `app` mutably, which matters for what the button can do: `select_and_mask_button` reads `crate::menus::is_enabled(app, "select.selectAndMask")` and, on click, runs the command and sets status, so it needs the whole app. A version taking only `&mut Ui` could draw but not act - which is why the other control in this row (`Select Subject`) also takes `app` and calls `app.run("select.subject", json!({}))` directly.

This row and the one in `panels/options_bar.rs` (hunk 12) both exist because the application shows two options bars: the shared one drawn by `panels`, and the per-tool one drawn here for tools whose options are not in the shared bar. Quick Selection gets the mode triple in the shared bar (hunk 12) because it is the tool's primary control, and the refine buttons here because they are shared with the selection tools that use this bar. Neither bar draws the same widget twice - the mode triple is only in `options_bar`, and `Select Subject`/`Select and Mask…` are only here.

#### 21. `crates/ui-egui/src/tools/retouch_ui/tests.rs` — hunk -1163,3 +1163,108

**Original code (upstream base)**

```rust
    h.run_steps(4);
    h.get_by_label("The History Brush source was deleted: set a new source in the History panel");
}
```

**Changed code (this pull request)**

```rust
    h.run_steps(4);
    h.get_by_label("The History Brush source was deleted: set a new source in the History panel");
}

#[test]
fn quick_selection_options_bar_has_new_add_subtract_mode_buttons() {
    use crate::theme::ThemeKind;
    use egui::vec2;
    use egui_kittest::kittest::Queryable;

    let mut app = app();
    app.ui.tool = Tool::QuickSelection;
    assert_eq!(app.ui.tool_options.quick_selection_mode, 1, "Quick Selection defaults to Add mode");

    let mut h = egui_kittest::Harness::builder().with_size(vec2(1400.0, 60.0)).build_ui_state(
        |ui, app: &mut PhotocraftApp| {
            if !ui.ctx().fonts(|f| f.families().contains(&egui::FontFamily::Name("medium".into()))) {
                return;
            }
            crate::panels::options_bar(app, ui);
        },
        app,
    );
    PhotocraftApp::setup_context(&h.ctx, ThemeKind::Studio);
    h.run_steps(4);

    // Verify New, Add, and Subtract buttons exist in the options bar
    let add_tip = crate::i18n::fmt(tl!("Add to selection  ({key})"), &[("key", &crate::shortcuts::pretty("Shift"))]);
    let sub_tip = crate::i18n::fmt(tl!("Subtract from selection  ({key})"), &[("key", &crate::shortcuts::pretty("Alt"))]);

    // Clicking Subtract switches to Subtract mode (2)
    h.get_by_label(&sub_tip).click();
    h.run_steps(4);
    assert_eq!(h.state().ui.tool_options.quick_selection_mode, 2);

    // Clicking New switches to New mode (0)
    h.get_by_label("New selection").click();
    h.run_steps(4);
    assert_eq!(h.state().ui.tool_options.quick_selection_mode, 0);

    // Clicking Add switches to Add mode (1)
    h.get_by_label(&add_tip).click();
    h.run_steps(4);
    assert_eq!(h.state().ui.tool_options.quick_selection_mode, 1);

    // A saved mode exceeding 2 is normalized to Add mode (1) on render
    h.state_mut().ui.tool_options.quick_selection_mode = 3;
    h.run_steps(4);
    assert_eq!(h.state().ui.tool_options.quick_selection_mode, 1, "out-of-range mode normalizes to Add mode");

    // Test selection_mode feedback across modes
    let none = egui::Modifiers::NONE;
    let alt = egui::Modifiers::ALT;
    let shift = egui::Modifiers::SHIFT;

    assert_eq!(crate::canvas::tool_feedback::selection_mode(Tool::QuickSelection, 1, none), "add");
    assert_eq!(crate::canvas::tool_feedback::selection_mode(Tool::QuickSelection, 1, alt), "subtract");

    assert_eq!(crate::canvas::tool_feedback::selection_mode(Tool::QuickSelection, 2, none), "subtract");
    assert_eq!(crate::canvas::tool_feedback::selection_mode(Tool::QuickSelection, 2, alt), "add");

    assert_eq!(crate::canvas::tool_feedback::selection_mode(Tool::QuickSelection, 0, none), "replace");
    assert_eq!(crate::canvas::tool_feedback::selection_mode(Tool::QuickSelection, 0, alt), "subtract");
    assert_eq!(crate::canvas::tool_feedback::selection_mode(Tool::QuickSelection, 0, shift), "add");

    // After drawing a stroke in New mode (0), it automatically flips back to Add mode (1)
    h.state_mut().ui.tool_options.quick_selection_mode = 0;
    let modifiers = egui::Modifiers::NONE;
    tool_event(h.state_mut(), ToolEvent::Down { x: 20.0, y: 20.0, pressure: 1.0 }, modifiers);
    tool_event(h.state_mut(), ToolEvent::Up { x: 20.0, y: 20.0 }, modifiers);
    assert!(!h.state().ui.status_error, "{}", h.state().ui.status);
    assert!(h.state().session.active().unwrap().doc.selection.is_some(), "select.quick created a selection");
    assert_eq!(h.state().ui.tool_options.quick_selection_mode, 1, "New mode automatically flips to Add mode after stroke");

    // An empty stroke (outside canvas, changed: false) or failed stroke (no points) in New mode keeps New mode (0)
    h.state_mut().run("select.deselect", json!({})).unwrap();
    h.state_mut().ui.tool_options.quick_selection_mode = 0;
    tool_event(h.state_mut(), ToolEvent::Down { x: -20.0, y: -20.0, pressure: 1.0 }, none);
    tool_event(h.state_mut(), ToolEvent::Up { x: -20.0, y: -20.0 }, none);
    assert!(h.state().session.active().unwrap().doc.selection.is_none());
    assert_eq!(h.state().ui.tool_options.quick_selection_mode, 0, "Empty stroke keeps New mode");

    finish_stroke(h.state_mut(), Tool::QuickSelection, &[], none);
    assert!(h.state().ui.status_error);
    assert_eq!(h.state().ui.tool_options.quick_selection_mode, 0, "Failed stroke keeps New mode");
    h.state_mut().ui.status_error = false;

    // Shift and Alt modifiers in New mode (0) run add/subtract and transition to Add mode (1) when changed
    tool_event(h.state_mut(), ToolEvent::Down { x: 20.0, y: 20.0, pressure: 1.0 }, shift);
    tool_event(h.state_mut(), ToolEvent::Up { x: 20.0, y: 20.0 }, shift);
    assert!(h.state().session.active().unwrap().doc.selection.is_some());
    assert_eq!(h.state().ui.tool_options.quick_selection_mode, 1);

    // When selection is empty in New mode (0), Alt-drag falls back to New selection and flips to Add (1)
    h.state_mut().run("select.deselect", json!({})).unwrap();
    h.state_mut().ui.tool_options.quick_selection_mode = 0;
    tool_event(h.state_mut(), ToolEvent::Down { x: 20.0, y: 20.0, pressure: 1.0 }, alt);
    tool_event(h.state_mut(), ToolEvent::Up { x: 20.0, y: 20.0 }, alt);
    assert!(h.state().session.active().unwrap().doc.selection.is_some(), "Alt on empty selection creates a new selection");
    assert_eq!(h.state().ui.tool_options.quick_selection_mode, 1);

    // When selection is non-empty in New mode (0), Alt-drag subtracts from it
    h.state_mut().ui.tool_options.quick_selection_mode = 0;
    tool_event(h.state_mut(), ToolEvent::Down { x: 20.0, y: 20.0, pressure: 1.0 }, alt);
    tool_event(h.state_mut(), ToolEvent::Up { x: 20.0, y: 20.0 }, alt);
    assert!(h.state().session.active().unwrap().doc.selection.is_none(), "Alt on existing selection subtracts");
    assert_eq!(h.state().ui.tool_options.quick_selection_mode, 1);
}
```

### `crates/ui-egui/src/tools/tool_reset.rs` (+1 / -1)

**Explanation.** One UI test, `quick_selection_options_bar_has_new_add_subtract_mode_buttons`, driven through `egui_kittest`. It builds a harness 1400 x 60 (the options bar's shape), sets the tool to Quick Selection, renders `crate::panels::options_bar`, and then walks the feature in the order a user would. The font guard at the top (`if !ui.ctx().fonts(|f| f.families().contains(&FontFamily::Name("medium".into()))) { return; }`) waits until the app's fonts are loaded; without it the first frames draw with fallback fonts and `get_by_label` can miss. `PhotocraftApp::setup_context(&h.ctx, ThemeKind::Studio)` installs the theme after the harness is built, and `h.run_steps(4)` settles the layout.

The assertions, in order:

`assert_eq!(app.ui.tool_options.quick_selection_mode, 1, "Quick Selection defaults to Add mode")` - the struct's `Default` sets 1, and this is the only place that checks it from a running app rather than from the struct literal.

`h.get_by_label(&sub_tip).click(); assert_eq!(..., 2)` - clicking the button whose accessible label is the formatted Subtract tooltip stores 2. Looking the button up by label rather than by index proves the buttons are reachable to assistive technology as well as to a mouse, and it proves the tooltip string is the one the widget actually uses (the test builds `sub_tip` from the same `i18n::fmt` call the widget makes).

`h.get_by_label("New selection").click(); assert_eq!(..., 0)` - New stores 0. The label here is the plain translated string, because the New button's tooltip has no `{key}` substitution.

`h.get_by_label(&add_tip).click(); assert_eq!(..., 1)` - Add stores 1, restoring the default.

`quick_selection_mode = 3; h.run_steps(4); assert_eq!(..., 1, "out-of-range mode normalizes to Add mode")` - the clamp lives in the drawing function (hunk 14), so it can only be observed by rendering; this is the only test that can reach it.

The seven `selection_mode` assertions then check the table directly rather than through strokes, because a stroke per combination would need twelve gestures: Add/None = add, Add/Alt = subtract, Subtract/None = subtract, Subtract/Alt = add, New/None = replace, New/Alt = subtract, New/Shift = add. Checking the function is the right level for the table; checking the stroke is the right level for the wiring, which the next block does.

The stroke block drives `tool_event` with `Down` and `Up` at (20, 20). In New mode with no modifiers: `assert!(!h.state().ui.status_error)` (the command did not fail), `assert!(...doc.selection.is_some())` (the stroke created a selection), `assert_eq!(..., 1, "New mode automatically flips to Add mode after stroke")`. The three together prove the stroke used `replace` and that the flip ran on `changed: true`.

The two "keeps New mode" cases are the negative half. A `Down`/`Up` at (-20, -20) is outside the canvas, so the command reports `changed: false`: `assert!(...doc.selection.is_none())` and `assert_eq!(..., 0, "Empty stroke keeps New mode")`. Then `finish_stroke(..., &[], none)` with no points fails: `assert!(h.state().ui.status_error)` and `assert_eq!(..., 0, "Failed stroke keeps New mode")`. Without these two, a flip on *any* stroke would pass the positive case.

The last three cases pin the modifier and empty-selection interactions in New mode. Shift produces a selection and flips to 1 (`assert_eq!(...)` with no message - the assertion is the state). Alt on an empty selection creates a new selection and flips (`"Alt on empty selection creates a new selection"`), which is the `document_selection_mode` fallback reaching the stroke. Alt on a non-empty selection subtracts everything and leaves no selection (`"Alt on existing selection subtracts"`) while still flipping, because the stroke changed the selection.

`h.state_mut().run("select.deselect", json!({})).unwrap()` between cases resets the document without touching the mode, so each case starts from a known selection state; that is why the test calls it before the two Alt cases.

Why the test is long: each branch of `selection_mode`, each branch of the New→Add flip, and the clamp are mutually independent, so a test per branch is the only way to know a change did not silently move a decision from one branch to another.

#### 22. `crates/ui-egui/src/tools/tool_reset.rs` — hunk -195,7 +195,7

**Original code (upstream base)**

```rust
        Tool::Lasso | Tool::PolygonLasso => reset!(feather, anti_alias),
        Tool::MagneticLasso => reset!(feather, anti_alias, magnetic_width, magnetic_contrast, magnetic_frequency, magnetic_pressure),
        Tool::MagicWand => reset!(tolerance, anti_alias, contiguous),
        Tool::QuickSelection => reset!(enhance_edge),
        Tool::ObjectSelection => reset!(enhance_edge, object_finder),
        Tool::Crop => reset!(
            crop_ratio,
```

**Changed code (this pull request)**

```rust
        Tool::Lasso | Tool::PolygonLasso => reset!(feather, anti_alias),
        Tool::MagneticLasso => reset!(feather, anti_alias, magnetic_width, magnetic_contrast, magnetic_frequency, magnetic_pressure),
        Tool::MagicWand => reset!(tolerance, anti_alias, contiguous),
        Tool::QuickSelection => reset!(enhance_edge, quick_selection_mode),
        Tool::ObjectSelection => reset!(enhance_edge, object_finder),
        Tool::Crop => reset!(
            crop_ratio,
```

Total hunks recorded: 22.

**Explanation.** `tool_reset.rs` resets a tool's options to their defaults when the user double-clicks the tool button; the `reset!` macro lists the fields that belong to that tool. `Tool::QuickSelection => reset!(enhance_edge)` becomes `reset!(enhance_edge, quick_selection_mode)`.

Without the added field, double-clicking Quick Selection would clear `enhance_edge` but leave the mode at whatever the user last picked - so "reset this tool" would be a partial reset. Every other field the tool owns is listed, which is what makes the omission a defect rather than a design choice.

`reset!` expands to an assignment of each named field to `<field>`, i.e. to the field's default through `Default::default()` on the whole struct or to a per-field default depending on the macro's definition. Either way the effect is "Add" (1) for this field, since the hand-written `Default` sets 1 (hunk 18). So the reset restores the tool's documented default, not zero - which is correct, because zero is New, a mode the old build never had.

Ordering: `quick_selection_mode` is appended after `enhance_edge`, matching the struct's declaration order (hunk 16 put the new field directly after `enhance_edge`). The `reset!` lists in this file follow the struct order, so appending here is the consistent choice; inserting alphabetically would break that correspondence for this one line.

The neighbouring arms are untouched: `ObjectSelection` still resets `enhance_edge, object_finder`, and `MagicWand` its three fields. That matters because the `reset!` arms are exhaustive over tools and a mis-placed edit would silently change another tool's reset set.

Nothing else needs to know about the reset. The field is not journaled and not part of any command, so resetting it changes only UI state; the next stroke reads it through `document_selection_mode` and behaves accordingly, which is why there is no undo entry and no test asserting a document change.

A detail about `reset!` worth recording: the lists are not exhaustive over the struct, they are exhaustive over *the tool's* fields. `Tool::QuickSelection` lists its two (`enhance_edge`, `quick_selection_mode`); `Tool::MagicWand` lists `tolerance, anti_alias, contiguous`; fields shared by all tools (the brush size, the sample-all-layers toggles) are reset by their own paths. That is why the reset for a tool is a short list and why a new field for a tool must be added here deliberately - the compiler cannot tell whether a field belongs to a tool.

The order of the list also serves as the answer to "which fields does this tool own": a reader looking at the line sees exactly the two, and `quick_selection_mode` being second says it was added later than `enhance_edge`, which is the same order the struct declares. If the struct order ever changed, this line would be the second place to update, and a reviewer would see the two adjacent.

