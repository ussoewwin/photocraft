# Essential integration plan: rebuild the fork on the official #15449 base

Created: 2026-11-18
Repository: `D:\USERFILES\GitHub\photocraft` (`origin` = ussoewwin/photocraft, `upstream` = storytold/photocraft)
Readers: every AI that continues this work (the goal is that this document alone is enough to execute it)

---

## 0. Absolute conditions (break one and the rebuild has failed)

1. **Never erase a conflict by overwriting an official file.** Do not settle a conflict by picking one side; on top of the official new implementation (#15449 and later), rewrite the fork's intent with the new APIs.
2. **Do not break fork features.** After each port, confirm the feature with its tests and `cargo check`.
3. **Do not delete the three PR branches.** `feat/localized-japanese-fonts`, `feat/quick-selection-mode-buttons`, `feat/move-tool-selection-drag-transfer`.
4. **Do not change `craft-libs`.** Do not change the git pin in `Cargo.toml` either. The fork completes itself within the photocraft repository alone.
5. **#11971 (`feat/move-tool-selection-drag-transfer`) is unfinished.** Another AI is repairing it (14 of the tests are red). Do not merge it into `main` until it is complete and passes the audit.
6. `cargo` on Windows needs the MSVC environment loaded (see §7). Do not use the global Python.
7. Before pushing to `main`, always get the user's approval. Do not update `origin/main` without asking.

## 1. What is going on (root cause)

- In the official #15449 "PhotoCraft on the craft UI framework" (`ed14e1c413`), the UI was replaced from Photocraft's own implementation with the framework of the external library `storytold/craft-libs`. `canvas.rs`, `icons.rs`, `options_bar.rs`, `font_preview.rs` (deleted), `type_tool/fonts.rs` and others changed heavily.
- The fork's `main` is built on official code from before #15449 (around `ea6ced03b1`). Its core is the integration commit `8da2be3707` (113 files), which bundles two kinds of work into one.
  - **Content that came from PRs**: #12452, #11837, #11971, #12339, etc.
  - **Fork-only features**: LaMa Cleaner, SAM 3.1, Japanese i18n, `plugins/`, `md/`, `start-photocraft.cmd`, etc.
- Merging the latest official code directly into `main` therefore conflicts in about 31 files such as `canvas.rs` (measured with `git merge-tree`). The conflicts are a collision between "fork changes written on the old UI implementation" and "the new UI framework", so a mechanical resolution breaks.
- The three PR branches, on the other hand, already include #15449 and have **0 conflicts** against `upstream/main` (3 commits behind upstream).

Measured values (as of 2026-11-18)

| Subject | Conflicts against `upstream/main` | Behind |
|---|---|---|
| `origin/feat/localized-japanese-fonts` (#12452) | 0 | 3 commits |
| `origin/feat/quick-selection-mode-buttons` (#11837) | 0 | 3 commits |
| `origin/feat/move-tool-selection-drag-transfer` (#11971) | 0 | 3 commits |
| old `main` | 31 | 16 commits |

## 2. Policy

**Do not merge the official code into the old `main`.** Rebuild `main` in this order.

1. Use `upstream/main` (which includes #15449) as the base.
2. Merge the conflict-free PRs one at a time (#12452 → #11837).
3. Port the fork-only features back one feature at a time, on the premise of the official new UI framework.
4. Take #11971 in last, once it is complete and has passed the audit.

## 3. Where the work happens

- Working worktree: `D:\USERFILES\GitHub\photocraft\target\wt-main` (branch `main-rebuild`, created from `upstream/main`)
- Existing PR worktrees (do not touch):
  - `target\wt-pr12452` (`pr12452-sync`)
  - `target\wt-pr11837` (`pr11837-sync`, warmed build at `D:\pf_target_pr11837`)
  - `target\wt-pr11971` (`pr11971-sync`, warmed build at `D:\pf_target_pr11971`)
- If a worktree is in a half-done state, it may be thrown away and recreated: `git worktree remove --force target\wt-main`, `git branch -D main-rebuild`, `git worktree add -b main-rebuild target\wt-main upstream/main`.
- If an `index.lock` is left behind, confirm no git process is running before deleting it (`.git\worktrees\wt-main\index.lock`).

## 4. Procedure

### Step 1: the base and the conflict-free PRs (skip if already done)

```powershell
Set-Location D:\USERFILES\GitHub\photocraft
git fetch upstream origin
git worktree add -b main-rebuild target\wt-main upstream/main
Set-Location target\wt-main
git merge --no-edit origin/feat/localized-japanese-fonts      # #12452
git merge --no-edit origin/feat/quick-selection-mode-buttons  # #11837
```

Expected: no conflicts in either. `git status -sb` shows `main-rebuild...upstream/main [ahead N]` and there are no uncommitted changes.

### Step 2: sorting out the fork-only features (verified as fact)

The diff baseline is `git merge-base origin/main upstream/main` = `2163bd45af` (`$base` below).

The "fork-only" files contained in none of the three PRs (from `git diff --name-only $base origin/main`, with the PR-side files removed):

- Engine: `crates/engine/src/cmd/retouch_cmds.rs`, `retouch_cmds/lama_cleaner.rs`, `retouch_cmds/remove.rs`, `cmd/sam3_select_cmds.rs`, `cmd/analysis_cmds/tests.rs`, `cmd/print_cmds/tests.rs`
- Format: `crates/format/src/atomic.rs`, `atomic/tests.rs` (#12339 Windows directory flush)
- Automation: `crates/automation/src/rpc/tests.rs`
- UI: `crates/ui-egui/src/i18n/ja.tsv`, `sam3_canvas_tests.rs`, `sam3_options_bar_tests.rs`, `sam3_toolbar_tests.rs`, `state/tool.rs`
- Others: `plugins/` (LaMa, SAM3 and their binaries), `md/`, `assets/icons/quick-selection-*.svg`, `.gitignore`, `README.md`, `start-photocraft.cmd`

Some of the PR files also carry changes made for SAM/LaMa. Files whose added lines contain `SAM3` or `lama` are the following (the number is the count of added lines):

`crates/engine/src/cmd.rs` (2), `engine/src/lib.rs` (1), `engine/src/prefs.rs` (2), `prefs/tests.rs` (2), `shell_commands.rs` (2), `ui-egui/src/app/commands.rs` (1), `canvas.rs` (1), `canvas/gesture.rs` (2), `canvas/paint.rs` (3), `canvas/tool_feedback.rs` (1), `icons.rs` (2), `input/hold_keys.rs` (1), `input/tool_key_default_tests.rs` (1), `lib.rs` (3), `panels.rs` (1), `panels/toolbar.rs` (2), `panels/type_flyout_tests.rs` (1), `pencil_tests.rs` (3), `smart_objects_menu_tests.rs` (1), `state/tool_options.rs` (17), `tool_shift_cycle_order_tests.rs` (3), `toolbar_group_order_tests.rs` (2), `tools/retouch_ui.rs` (37), `tools/retouch_ui/tests.rs` (21), `tools/tool_reset.rs` (3), `dialogs/file_ui/tests.rs` (5), `dialogs/discard_ui/tests/jpeg_tests.rs` (1)

This list is reproduced by:

```powershell
$base='2163bd45af'
git diff --name-only $base origin/main | Where-Object { $_ -notmatch '^(plugins/|md/)' } | ForEach-Object {
  $n = (git diff $base origin/main -- $_ | Select-String -Pattern '^\+.*(?i)(sam3|lama|SamSelect|flush)').Count
  if ($n -gt 0) { "$n  $_" }
}
```

### Step 3: port the parts that do not depend on the UI first (phase A)

Targets: engine, format, automation, `plugins/`, `md/`, `assets/icons`, `.gitignore`, `README.md`, `start-photocraft.cmd`, `ja.tsv`. They do not depend on the UI framework, so they should apply cleanly.

```powershell
Set-Location D:\USERFILES\GitHub\photocraft\target\wt-main
$base='2163bd45af'
# List the engine/format/automation files above plus plugins md assets .gitignore README start-photocraft.cmd ja.tsv
git diff --binary $base origin/main -- <paths...> > ..\fork_a.patch
git apply --3way --index ..\fork_a.patch
```

- For files that conflict, keep the official side (#15449 and later) and hand-add only the fork's additions.
- Caution: the `crates/engine` diff includes files that came from #11971 (`float_cmds.rs`, `layer_copy_cmds.rs`, `pick_cmds.rs`). **Do not take them in phase A.** When enumerating target paths for the command above, exclude them.
- The `plugins/` binaries (`*.exe`, `*.dll`) are large (about 130 MB in total). Do not add or remove them without the user's confirmation.
- `crates/automation/src/server.rs` has a history of conflicting (the official side changed it). Keep the official side and add only the fork's additions.
- Verify: `cargo check -p photocraft-engine -p photocraft-format -p photocraft-automation` and each crate's relevant tests.

### Step 4: port the UI parts back on the #15449 APIs (phase B, the essential part)

The targets are the UI files listed in step 2. Work one feature at a time, with this method:

1. Confirm the "intent" of the feature (SAM Selection, LaMa Cleaner, SAM's tool switching, toolbar, options bar, key bindings and icons, the added `tool_options` fields, and the added parts of `retouch_ui`) in `md/sam3-select.md` and `md/lama-cleaner-cuda.md`.
2. Read the old implementation (the files of `origin/main`) and understand what it did.
3. In the official new structure (the craft-libs framework, `canvas/tools/*`, `panels/options_bar/*`, `tools/retouch_ui/bar/*`, etc.), find the place that expresses the same feature.
4. On top of the official code, write the fork's addition with the new APIs. Do not delete the official code that is already there.
5. Port that feature's tests (`sam3_*_tests.rs` and the like) to the new APIs and make them green.
6. For each feature run `cargo test --lib -- <filter>`, confirm green, then commit.

Keep phase B commits split per feature (for example: SAM tool registration, SAM options bar, LaMa's retouch wiring, the Japanese i18n wiring).

### Step 5: the overall check

```powershell
cargo fmt --check
cargo clippy -p photocraft-ui-egui -p photocraft-engine --all-targets -- -D warnings
cargo test -p photocraft-ui-egui --lib
cargo test -p photocraft-engine --lib
cargo xtask layers
cargo xtask panel-commands
cargo xtask wasm
```

- For failures, separate whether the cause is a fork change or a pre-existing failure on the official side, by checking `upstream/main` alone (do not blame the fork for upstream's failures, and the reverse).
- Also confirm: `SAM3`/`LaMa` are reachable as tools (toolbar, options bar, menus); #12452's font classification and the `あいうえお` preview; #11837's badge, button layout and inside-selection drag.

### Step 6: updating `main` (after the user's approval)

1. Reflect `main-rebuild` into `main`. Because `origin/main` becomes a rebuilt history, always confirm with the user.
2. For `photocraft_sync_and_build` (the `git_automation` MCP) **always** specify `repository_path` (without it, it fails with `Invalid repository path`). Example: `repository_path = D:\USERFILES\GitHub\photocraft`.
3. Integrate one PR at a time, not in one batch. For the PR branch sync (`sync_pr_branches`), set it to `false` or state `pr_branches` explicitly so that the branch another AI is working on (#11971) is not touched.

### Step 7: integrating #11971 (last)

1. Wait until the other AI has fixed the 14 red tests and `cargo test --lib -- layer_transfer` is green.
2. Audit it. Follow `md/pr-11971-move-tool-cross-document-transfer.md` and measure the drop onto an arranged tile, the drop onto a tab, spring-loading, the copy of a mask selection, and the drag state while blocked.
3. Merge `origin/feat/move-tool-selection-drag-transfer` into `main-rebuild`. It should have no conflicts.
4. Re-run step 5.

## 5. Current progress (as of 2026-11-18 13:30)

- #12452: audited, no problems.
- #11837: the 4 missing pieces (the brush cursor badge, live following of an inside-selection drag and its 45°, the layout of the mode buttons, `Select and Mask…`) were fixed and pushed to `origin/feat/quick-selection-mode-buttons` (`7427d24271`). The 53 related tests are green.
- #11971: another AI is repairing it. Waiting for it to be complete.
- `main-rebuild` (`target\wt-main`): step 1 is done (#12452 and #11837 merged). Step 3's patch application was run partway; `crates/automation/src/server.rs` is in a conflicted state and the rest is in the index. An AI continuing from here confirms the state with `git status` and, if needed, redoes step 3.
- The local `main` has an unpushed commit (`4d4570e52c`) that merged #12452. If it becomes unnecessary after the rebuild, discard it. `origin/main` has not been touched.

## 6. What not to do (prevention)

- Merge `upstream/main` directly into the old `main` and erase conflicts by picking a side.
- Overwrite official files such as `canvas.rs`, `icons.rs`, `options_bar.rs` with the fork's old versions.
- Restore files the official side deleted (`text/font_preview.rs`, `canvas/gesture.rs`, `canvas/paint.rs`, `canvas/tool_input.rs`, `burn_bar_tooltips_tests.rs`) for the sake of conflict resolution.
- Delete a test because it fails, or weaken an assert.
- Put #11971 into `main` while it is unfinished.
- Update `origin/main` without the user's approval, or delete a PR branch.
- Change `craft-libs`, or change the pin in `Cargo.toml`.

## 7. Environment notes (Windows)

- Load the MSVC environment before running `cargo`. Without it, `aws-lc-sys` cannot find `stdint.h` and fails.

```powershell
$env:CARGO_TARGET_DIR = "D:\pf_target_main"   # use a dedicated target per job (about 10 GB each)
Set-Location D:\USERFILES\GitHub\photocraft\target\wt-main
cmd /c "call ""C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat"" >nul && cargo test --lib -- <filter>"
```

- When using a warmed target, run `cargo test --lib -- <filter>` without `-p` (so that the feature unification matches the existing cache).
- If Python is needed, call `D:\USERFILES\ComfyUI\python_embeded\python.exe` by its full path. Do not use the global Python.
- Code conventions: never use `unwrap()`, `expect()` or `panic!()` in production code. Never put an AI's perspective or a signature into commits or documents.

## 8. Definition of done

- `main-rebuild` has `upstream/main` as an ancestor (`git merge-base --is-ancestor upstream/main main-rebuild` succeeds).
- The content of #12452 and #11837 is all in (`git merge-base --is-ancestor` succeeds for each PR branch).
- The fork-only features (SAM 3.1, LaMa Cleaner, Japanese i18n, the #12339 flush fix) work and their tests are green.
- All the commands of step 5 pass.
- After the user's approval, `main` is updated, and #11971 is integrated separately once it is complete.
