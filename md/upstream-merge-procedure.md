# Merging Upstream Latest Into This Fork — Complete Procedure

This document is the full, repeatable procedure for merging `upstream/main` into this fork's `main`
while (a) preserving every fork-only feature, (b) leaving zero failing tests, and (c) never trusting
an unverified claim: every step below ends in a measured result. It was written from a completed
merge of `6d99175b1` (675 files changed, +52,228 / −151,198, including the i18n catalog externaliza-
tion and the HEIF crate swap) and reflects the conflicts and repairs that merge actually required.
Every identifier, path, command and number in this document was checked against the tree it
describes.

Placeholders used in the commands below: `<repo>` — this checkout; `<drive>` — a large data drive;
`<craft-i18n checkout>` and `<craft-fonts checkout>` — the external checkouts described in §7.1 and
§2.5; `<merge-base>` — the output of `git merge-base main upstream/main`; `<pre-merge main>` — the
commit `main` pointed at before the merge; `<branch>` — an open pull-request branch; `<crate-path>`
— a workspace member path; `<target>` — the value of `CARGO_TARGET_DIR`.

Two invariants govern everything:

1. **Fork features survive.** Each fork feature is listed in §1 together with the test that proves
   it. A merge is not done until all of those tests pass.
2. **No failures left behind.** A red test is never explained away ("environment", "upstream's
   fault") without a measurement that proves it. Find the root cause, fix it in code, and show the
   green run. Never weaken an assertion or a tolerance to make a test pass.

---

## 1. Fork feature inventory and its guarding tests

Before merging, know exactly what must survive. Each feature maps to at least one test; after the
merge these tests are the definition of "not broken".

| Fork feature | Lives in | Guarding tests |
|---|---|---|
| LaMa Cleaner tool | `crates/engine/src/cmd/retouch_cmds/lama_cleaner.rs`, `crates/ui-egui/src/tools/retouch_ui.rs`, `crates/engine/src/cmd/retouch_cmds.rs` | `retouch_ui` (90 tests), engine lib suite |
| Japanese typography / CJK font engine | `crates/text/src/cjk.rs`, `crates/text/src/fonts.rs`, `crates/text/src/font_class.rs`, `crates/ui-egui/src/text/font_preview.rs` | `cjk_fonts` (55 tests) |
| Japanese transform-menu labels (`Skew` / `Distort` wording) | `crates/ui-egui/src/i18n.rs` (`JA_TEXT_OVERRIDES` at line 433, `JA_ID_OVERRIDES` at line 434, applied in `tr()` / `tr_id()`) | `i18n::tests::japanese_transform_labels_keep_skew_and_distinct` |
| Move tool cross-document transfer (incl. background-layer drag) | `crates/ui-egui/src/panels/layer_transfer.rs` (`move_drag()` line 74, `arm_move_transfer()` line 112), `crates/ui-egui/src/tools/move_box.rs`, `crates/ui-egui/src/canvas/tool_input.rs` (`record_move_blocked()` line 403) | `layer_transfer` (26 tests) |
| Windows directory-flush fix (dir sync: 1 / 50 / 120 are non-fatal) | `crates/format/src/atomic.rs` | `crates/format/src/atomic/tests.rs` |
| HEIF/HEIC support | upstream-owned since the `craft-heif` (craft-libs) swap; the fork carries no private copy | `crates/codecs` HEIF tests |
| Craft-fonts script coverage at runtime | `crates/text/src/craft_fonts.rs`, `crates/text/build.rs` (compiled in only when `CRAFT_FONTS_DIR` is set) | `cjk_fonts`, `text` suite |

Rule: when upstream refactors an area a fork feature lives in, the feature is **ported**, not
sacrificed, and the port is proven by the tests in this table.

Note on scope: some fork work still lives only on an open pull-request branch (for example the
`pickAt` `includeLocked` probe support is **not** in `main` — `git grep includeLocked -- crates`
returns nothing on `main`). Treat such items as branch-local: check the branch before assuming a
symbol exists in `main`, and never copy a whole PR-branch file into `main` (§7.3).

---

## 2. Pre-merge checks

Run all of these before touching anything:

```powershell
cd <repo>
git switch main
git fetch origin
git fetch upstream

# 2.1 Working tree must be clean, and no other session may be operating in this tree.
git status --short                       # must print nothing
if (Test-Path .git\MERGE_HEAD) { 'merge in progress' }
git diff --name-only --diff-filter=U     # must print nothing (no leftover conflicts)

# 2.2 How far behind is main, and how big is the merge?
git log --oneline main..upstream/main | Measure-Object -Line
git diff main...upstream/main --stat | Select-Object -Last 3

# 2.3 Is upstream itself healthy? A broken upstream tip (e.g. duplicate definitions) fails at
#     compile time and its CI shows it. Check before merging so an upstream break is never
#     mistaken for a merge mistake.
gh run list --repo storytold/photocraft --branch main --workflow ci.yml --limit 3 `
  --json databaseId,conclusion,headSha --jq '.[] | "\(.headSha[0:8]) \(.conclusion) \(.databaseId)"'
# If a run is "failure", open the failed jobs and their logs now and record the root cause.
# A null conclusion means the run has not finished yet: take the newest run whose conclusion is
# set, and read its jobs with `gh api repos/storytold/photocraft/actions/runs/<id>/jobs`.

# 2.4 Build environment
$env:CARGO_TARGET_DIR   = '<drive>:\pf_target'    # large data drive only, never the system drive
$env:CARGO_INCREMENTAL  = '0'                     # keeps the target directory from bloating
$env:CRAFT_I18N_DIR     = '<craft-i18n checkout>'  # catalogs; see §7.1 — required for translated runs
$env:CRAFT_FONTS_DIR    = '<craft-fonts checkout>' # optional craft-fonts faces; see §2.5
```

### 2.5 What each environment variable actually does (verified)

| Variable | Effect when set | Effect when unset |
|---|---|---|
| `CARGO_TARGET_DIR` | all build artifacts land on that drive | the system drive fills up |
| `CARGO_INCREMENTAL` | `0` disables incremental artifacts | the target directory grows without bound |
| `CRAFT_I18N_DIR` | `crates/ui-egui/build.rs` embeds the craft-i18n catalogs: every language works, and `needs_catalogs()` (`crates/ui-egui/src/i18n.rs`, `pub(crate)`) returns true | English-only build; language-iterating tests silently cover English only, so translation regressions stay invisible |
| `CRAFT_FONTS_DIR` | `crates/text/build.rs` compiles in every face listed in the craft-fonts `fonts/manifest.txt` (Japanese, Arabic, Thai, …) | the app falls back to system fonts only; the test suites still pass (verified: `cjk_fonts` 55/0 both ways) but the shipped script coverage is reduced — set it for the fork's release |

`craft-fonts` is **not** tracked in this repository (`git ls-files craft-fonts` → 0 files): it is an
external checkout, exactly like craft-i18n. A `CRAFT_FONTS_DIR` pointing inside the working tree is
a local convenience, not a repository path.

---

## 3. Divergence and overlap analysis

Run this **before** the merge: afterwards, `main..upstream/main` is empty by construction and the
overlap count is 0, which says nothing about the conflicts the merge had.

Predict the conflicts before merging so each one meets a prepared decision:

```powershell
# 3.1 Every file upstream changes
git diff main...upstream/main --name-only > $env:TEMP\up_files.txt

# 3.2 Every file the fork changed since the merge base
$mb = git merge-base main upstream/main
git diff "$mb..main" --name-only > $env:TEMP\fork_files.txt

# 3.3 The overlap = where git will report hard conflicts (or silently mis-merge)
$up = Get-Content $env:TEMP\up_files.txt
$fk = Get-Content $env:TEMP\fork_files.txt
$fk | Where-Object { $up -contains $_ }
```

For every overlap file, decide in advance: **ours** (fork behavior wins), **theirs** (upstream
wins), or **synthesis** (both, merged by hand). §5 lists the standard decisions.

Also check whether any fork-only fix still lives only on a pull-request branch and is needed on
`main` (§7.3); plan to port it in the same merge session.

---

## 4. The merge

```powershell
$env:GIT_AUTHOR_NAME    = 'ussoewwin'
$env:GIT_AUTHOR_EMAIL   = '136552381+ussoewwin@users.noreply.github.com'
$env:GIT_COMMITTER_NAME = 'ussoewwin'
$env:GIT_COMMITTER_EMAIL= '136552381+ussoewwin@users.noreply.github.com'

git merge upstream/main -m "Merge upstream/main into main"
git diff --name-only --diff-filter=U     # the hard-conflict list
```

Never use `git add -A`; stage only files you actually resolved, checking `git status --short`
entry by entry. The same discipline applies to every later commit in this procedure.

---

## 5. Conflict-resolution policy (hard conflicts)

| Conflict kind | Decision | Rationale |
|---|---|---|
| `crates/format/src/atomic.rs` (doc comment + `dir_sync_unsupported` body) | **ours** | The fork classifies Windows dir-sync codes 1 / 50 / **120** as non-fatal; upstream treats only 1 / 50 as such. Fold upstream's informational notes (SMB, issue references) into our doc comment. The merged `crates/format/src/atomic/tests.rs` must still assert 120 → success; if it does not, the wrong side won. |
| `crates/ui-egui/src/i18n/ja.tsv` (modify/delete after the catalog externalization) | **ours, keep the file** | Preserve the fork's translation data as a record. The live translations come from craft-i18n plus the in-repo override table (§7.1); the file itself is data, not code, and deleting it loses the fork's wording record. |
| `crates/ui-egui/src/canvas/canvas_tool_menu.rs` (label call style) | **theirs** | Upstream moved menu labels to the new `tl!` lookup; the fork's old `tr_id(current(), …)` call no longer matches the merged i18n module. Take upstream's style, then `cargo check`. |
| `Cargo.lock` | resolve by hand, then **let cargo finalize** | Keep fork-only workspace entries that upstream lacks, and drop entries for crates upstream removed — verify each against the workspace manifests (example: a crate upstream replaced with a craft-libs dependency must leave the lock). After the conflicts are staged, a build pass rewrites the lock to match the merged manifests; stage that rewrite. |

Anything not in this table: read both sides, pick the side that keeps the fork's documented
behavior, and state the reason in the eventual fix commit's message.

---

## 6. Semantic conflicts — what auto-merge gets wrong

Git reports these as success. Every one of them has actually happened in this fork; check all of
them after every merge:

1. **Duplicated definitions (E0428).** When both sides add the same constant or function in
   different places, auto-merge can keep both copies. Upstream's own tip has shipped exactly this
   bug (`TAG_JPEG_TABLES` / `TAG_PREDICTOR` in `crates/codecs/src/codecs/tiff/decode.rs`); its
   `ci.yml` run for that tip fails with the same E0428. After the merge, run a full
   `cargo check --workspace --all-targets` early; on E0428 keep the copy that carries the doc
   comment and delete the bare duplicate — then re-check upstream's CI conclusion so you know you
   fixed *their* bug and did not introduce one.
2. **Removed test helpers.** Upstream may delete a constructor the fork's tests still use (e.g. a
   `JA()` language constructor replaced by `Lang::from_code("ja")`). Port the test to the new API
   and gate it on catalog availability (`if !needs_catalogs() { return; }`, `needs_catalogs()` is
   `pub(crate)` in `crates/ui-egui/src/i18n.rs`) so it still runs when catalogs are embedded and
   skips cleanly when they are not.
3. **Message wording changed upstream.** Tests that assert refusal/alert strings break when
   upstream re-words the message or re-routes which tool gets which string. Align the test with the
   merged implementation's routing (measured tools get their own wording via
   `brush_refusal::tool_name` in `crates/ui-egui/src/tools/brush_refusal.rs`, everything else gets
   the generic template) — build the expectation with the same `i18n::fmt(tl!(…))` call the
   production code uses, never with a stale literal.
4. **Signature drift on shared functions.** If a fork branch added a parameter to a function
   upstream also evolved (`begin_with_layer`, a probe command's options, …), the auto-merge keeps
   one signature while callers on the other branch expect another. `cargo check --all-targets`
   catches it; resolve by synthesizing both parameters (see §7.3 for the transfer example).
5. **Fork tests calling removed modules.** Grep the fork's test names from §1 after the merge; if a
   helper or module disappeared, port the test — do not delete it.

---

## 7. Restoring fork behavior inside upstream-refactored areas

### 7.1 i18n: the catalog moved to craft-i18n

Upstream replaced the in-repo `crates/ui-egui/src/i18n/*.tsv` catalogs with the external
`storytold/craft-i18n` crate, embedded at build time from `CRAFT_I18N_DIR` at the commit pinned in
`craft-i18n.rev`. Consequences and the required steps:

```powershell
# One-time setup of the catalog checkout (repeat whenever craft-i18n.rev changes):
git clone https://github.com/storytold/craft-i18n <craft-i18n-checkout>
cd <craft-i18n-checkout>
git checkout (Get-Content <repo>\craft-i18n.rev)
```

- **Never edit the catalog checkout to carry fork wording.** A fix that lives only in a local
  checkout is not in the repository: any other build loses it. Fork-specific rows belong in the
  in-repo override table instead (next bullet).
- Fork wording that upstream's catalog gets wrong or lacks belongs in
  `crates/ui-egui/src/i18n.rs`: the `JA_TEXT_OVERRIDES` (English source → Japanese) and
  `JA_ID_OVERRIDES` (command id → Japanese) tables, applied only when `lang.code() == "ja"` inside
  `tr()` and `tr_id()`. The current rows cover `Skew`, `Set horizontal skew`, `Set vertical skew`
  (text) and `edit.transform.skew`, `edit.transform.distort` (ids). The guarding test
  `japanese_transform_labels_keep_skew_and_distinct` must pass with a **pristine** catalog
  checkout — that is the proof the feature no longer depends on the local machine.
- Keep the old `ja.tsv` in the tree as the fork's data record (§5).

### 7.2 Crate swaps that replace fork-touched code

Upstream sometimes deletes a crate the fork carried (example: `crates/heif` replaced by
`craft-heif` from craft-libs, pinned in the workspace `Cargo.toml`). Before accepting the
deletion, prove the fork had no private changes, comparing the merge base against the fork's `main`
**as it was before the merge** (afterwards the path is gone from `main` — verified: `crates/heif`
has one entry at the pre-merge commit and none at `HEAD`):
`git diff <merge-base>..<pre-merge main> -- <crate-path>` — empty means pure upstream history, so
the swap is a pure follow and the fork loses nothing. Non-empty means the fork customized it; then
port the customization first. After the swap, the corresponding tests (here: the codecs HEIF
tests) are the proof the feature still works.

### 7.3 Porting fork fixes that only exist on pull-request branches

`main` already contains merged PR features; upstream changes can silently break them (git sees no
conflict because both sides edited different lines). Symptom: a fork-feature test fails although no
hard conflict exists anywhere. Procedure:

1. Run the guarding test from §1 and read the exact assertion.
2. Diff `main` against the branch that carries the fix: `git diff main <branch> -- <file>`. The
   missing hunk is the port. Example (transfer): `move_drag()`'s condition must accept a recorded
   blocked press, and `arm_move_transfer()` must treat the active layer as optional; without that
   hunk a background-layer drag no longer transfers.
3. Port only the relevant hunks — never copy the whole branch file, which carries changes that
   belong to other pull requests.
4. If the hunk depends on engine-side support that branch added (for example an extra option on a
   probe command), port that dependency too, or verify the merged code path already provides the
   equivalent behavior. The guarding tests decide.

---

## 8. Fixing failing tests the right way

- Read the assertion and the production code it exercises. Decide which side is stale. Then fix
  the stale side. Never "fix" by deleting a test or widening a tolerance.
- **Replacing `Ui::label` with custom painting** (needed for pixel-exact fixes) silently drops the
  widget's accessibility node, and every label-driven test fails. Always re-add
  `resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Label, true, text.to_owned()));`
- **Pixel fixes must be conditional, never unconditional.** An unconditional inset moves every
  dialog and breaks the reference-layout tests. When a glyph mesh reaches past the pen position
  (script ink, antialiasing fringe), inset by `galley.mesh_bounds` only when the overhang exceeds
  0.5 px, rounded up to whole pixels — text whose mesh starts at the pen keeps its exact position.
- **Debug probes are never committed.** A temporary `eprintln!` used to measure a galley must be
  removed in the same session, and the final fix must stand on the measurement, not the probe.
- **Environment claims need proof.** Before calling a failure "environment", change one variable at
  a time (stash one file, unset one env var) and rerun. If the failure survives every local
  variable change, compare against upstream CI evidence (which jobs ran, what failed there) before
  writing a single sentence about "upstream".

---

## 9. Verification gates (all must be green; measure, never assume)

```powershell
$env:CARGO_TARGET_DIR   = '<drive>:\pf_target'
$env:CARGO_INCREMENTAL  = '0'
$env:CRAFT_I18N_DIR     = '<craft-i18n checkout>'   # catalogs embedded, or the run is blind
$env:CRAFT_FONTS_DIR    = '<craft-fonts checkout>'  # optional, see §2.5

cargo fmt --check
cargo clippy -p photocraft-engine -p photocraft-ui-egui -p photocraft-format -p photocraft-text `
             -p photocraft-codecs --all-targets -- -D warnings
cargo check --workspace --all-targets          # catches E0428-style breakage everywhere

cargo test -p photocraft-ui-egui --lib         # ~6.7k tests, includes cjk_fonts / i18n /
                                               # layer_transfer / retouch_ui / label_column
cargo test -p photocraft-engine --lib
cargo test -p photocraft-codecs
```

Expected baseline (measured on the `6d99175b1` merge): ui-egui 6734 passed / 0 failed,
engine 3561 / 0, codecs 1053 / 0, clippy exit 0, fmt exit 0, workspace check exit 0.

Run the fork-feature suites explicitly after fmt touches anything (`i18n::tests`,
`label_column_tests`, `cjk_fonts`, `layer_transfer`, `retouch_ui`) — formatting must not move a
test result.

---

## 10. Commit discipline

- The merge itself is one commit: `Merge upstream/main into main`, author/committer
  `ussoewwin <136552381+ussoewwin@users.noreply.github.com>` (set via `GIT_AUTHOR_*` /
  `GIT_COMMITTER_*`; the environment may carry different values).
- Everything the merge required beyond conflict resolution — ported branch hunks, test API
  migrations, upstream-bug fixes (duplicate definitions), fork-behavior restorations — is a
  **separate follow-up commit** with a subject naming the change, e.g.
  `Keep the fork's Japanese transform labels in the tree and section-label ink inside the dialog`.
  A merge commit that silently mixes repairs cannot be reviewed or reverted.
- Stage files individually (`git add <file>`), never `-A`.
- Push only on explicit instruction. After pushing, verify `git rev-parse HEAD` equals
  `origin/main` and read the commit back from the API.

### Two-track upstream synchronization

`main` is not the only branch that tracks upstream. While a fork pull request is still open, that
branch needs its own upstream merge:

1. **Redundancy check first.** After upstream absorbs a feature the fork also carries, the fork's
   pull request becomes redundant. Compare the feature's files against upstream:
   `git log upstream/main --oneline -- <feature files>`; if upstream now contains the same change,
   closing the fork's pull request is better than maintaining a duplicate.
2. **Merge, never rebase.** Bring upstream into the branch with
   `git merge upstream/main -m "Merge upstream/main into <branch>"` and keep the branch's history
   replayable; no force-push over a published branch.
3. **Same conflict policy.** §5 and §6 apply unchanged, and the branch's guarding tests (§1) must
   be green after the merge — the transfer and selection-mode suites are the ones that break.
4. **Order of operations.** Merge upstream into `main` first; then update the open branches against
   the new `main` so their diffs stay reviewable.

---

## 11. Fork release and tag discipline (only on explicit instruction)

- A release follows a merge only when it is asked for. The tag object is the current `HEAD` of the
  fork's `main` — fetch first and state the object (`git rev-parse HEAD`) before tagging.
- The release-note and changelog format is copied byte-for-byte from the previous fork release
  (language-switch table, section order, link form, bullet kinds). No invented elements, no new
  bullet kinds, no new link targets.
- After publishing, read the published body back through the API and compare it with the local
  source before reporting success; a self-check that only inspects local bytes proves nothing.
- Tag and release operations are outward-facing: never create, move or delete a tag, publish a
  release, or dispatch a workflow without an explicit instruction.

---

## 12. Release build and cleanup

```powershell
$env:CARGO_TARGET_DIR  = '<drive>:\pf_target'
$env:CARGO_INCREMENTAL = '0'
$env:CRAFT_I18N_DIR    = '<craft-i18n checkout>'   # a release without it ships English-only
$env:CRAFT_FONTS_DIR   = '<craft-fonts checkout>'  # optional; see §2.5

cargo build --release -p photocraft -p photocraft-cli
```

Verify the artifacts exist with fresh timestamps, and smoke-test the CLI
(`<target>\release\photocraft-cli.exe --version`). A GUI executable is not launched as a check.

Cleanup after the build: delete every scratch log and helper script created during the session
(logs, temporary patch scripts, backup directories of already-committed files). Keep only the
build outputs (`<target>\release`), the test cache (`<target>\debug`), and the craft-i18n and
craft-fonts checkouts (required by future catalog- and font-embedded builds). Measure the freed
space and list what remains and why.

---

## 13. Troubleshooting table

| Symptom | Cause | Fix |
|---|---|---|
| `E0428: the name … is defined multiple times` in a file neither side "changed" | both sides added the same item; auto-merge kept both | keep the documented copy, delete the bare duplicate; compare with upstream CI |
| Test calls a removed constructor (`JA()`, old catalog API) | upstream refactored the i18n API | port to `Lang::from_code("…")` plus the `needs_catalogs()` guard |
| Group-refusal test expects the old generic wording | upstream re-routed which tools get which message | build the expectation via `brush_refusal::tool_name` + `i18n::fmt(tl!(…))` like the production code |
| Transfer test: neighbour cell receives nothing | upstream's Auto-Select refactor dropped the fork's blocked-press / grabbed-layer path in `move_drag` / `arm_move_transfer` | port the branch's hunks for that file (§7.3) |
| Japanese labels show the catalog's wording | the craft-i18n catalog's Japanese row for `Skew` is a mistranslation (it reads as "italic") | extend `JA_TEXT_OVERRIDES` / `JA_ID_OVERRIDES` in `crates/ui-egui/src/i18n.rs`; never edit the catalog checkout |
| Label-driven tests fail after a paint change | custom painting dropped the accessibility node | re-add `WidgetInfo::labeled(egui::Role::Label, …)` |
| Reference-layout tests fail after a pixel fix | unconditional inset moved every dialog | make the inset conditional on `galley.mesh_bounds` overhang > 0.5 px |
| Catalog tests pass locally, translation regressions invisible | the build ran without `CRAFT_I18N_DIR` (English-only) | always set it for verification runs |
| Catalog-dependent tests fail everywhere | `CRAFT_I18N_DIR` unset, or the checkout is not at the commit in `craft-i18n.rev` | clone/checkout the pinned commit and set the variable |
| `cargo check` fine but a test count differs from the baseline | a suite ran filtered or with a stale environment | rerun unfiltered with the §9 environment block |

---

## 14. Final checklist

- [ ] `git status --short` clean; `origin/main` == local `main` after the instructed push
- [ ] All §1 fork-feature suites green
- [ ] fmt / clippy (`-D warnings`) / workspace `cargo check --all-targets` exit 0
- [ ] ui-egui, engine, codecs: 0 failed, counts at or above the §9 baseline
- [ ] Fork wording lives in the repository (override table), not in a local catalog checkout
- [ ] Open pull-request branches updated or closed per the two-track section of §10
- [ ] No temporary logs, probes, or backup directories left anywhere
- [ ] Merge commit plus separate repair commits, correct author identity, nothing pushed or
      published without instruction
