# The 2026-11-25 merge: every line of the fork's code, explained

> **Branches as they stand today.** `storytold/craft-libs` and `storytold/photocraft` both have one
> line, `main`; upstream merged the old `next` line into `main` and deleted the branch on 2026-11-27
> (pull requests #3644 and #3645 there), and this fork's own branch was renamed `next` to `main` the
> same day. Where a command or a measurement below names `next`, it names the line as it was at that
> moment and is kept as the record of that round.

## What this set is

Four documents that together account for **every line the fork changed** around the upstream merges of
2026-11-25, and for **every upstream commit** those merges brought in.

| # | document | contents |
|---|---|---|
| 00 | this file | scope, method, how to verify each claim |
| 01 | `...-01-craft-libs-code.md` | craft-libs: the merge resolutions and the four repairs, file by file, with the diff verbatim and each hunk explained |
| 02 | `...-02-photocraft-code.md` | photocraft: the merge resolutions, the retry setting, the moved trial, file by file |
| 03 | `...-03-upstream-and-verification.md` | both upstream ranges commit by commit, the measurements that prove the result, and what is not proven |
| 04 | `...-04-upstream-code-behind-each-change.md` | the upstream code that forced each adaptation, quoted with its path, and what it does |

## Scope, stated exactly

- **Upstream's own code is not reproduced.** The merges brought 629 commits into photocraft and 1,279
  into craft-libs; reproducing them would be a copy of `storytold`'s repositories, not an explanation.
  Document 03 lists every one of those commits by subject, so nothing is hidden, and names the areas a
  reader has to know about.
- **What is reproduced in full is the fork's side**: every file where the fork's content survives at
  `HEAD` and differs from upstream, plus every file the fork repaired after the merge. Their diffs are
  taken from `git` at the moment this document was generated, not retyped.
- **Line numbers**: each diff carries the real hunks (`@@ -a,b +c,d @@`), so any line can be found in
  the file with `git blame` or an editor.

## Method (reproduce any claim)

```powershell
# the fork's side of a merge, file by file
git -C <repo> diff <merge>^2..<merge> -- <path>      # what the fork has where upstream had its own
# the repairs after the merge
git -C <repo> diff <last-merge>..<head> -- <path>
# does the fork still differ from upstream at HEAD?
git -C <repo> diff upstream/<line>..HEAD -- <path>
# every upstream commit in the range
git -C <repo> log --oneline <old-upstream>..<new-upstream>
```

The two merges and their heads:

| repository | merge commit | upstream parent | fork head at the merge |
|---|---|---|---|
| craft-libs | `65902c049` | `9f2961402` (`upstream/next`) | `d8e30b0a6` then `87ff4927b` |
| photocraft | `c5af0bae9` | `e56522780` (`upstream/main`) | `220a5435e` then `bfa2fbc98` |

## How the fork's side was isolated

A file in a merge is the fork's own work when its merged content differs from **both** parents. That
test, run over every file the merge touched, gives the complete set:

- craft-libs, 238 files touched, **7 differ from both parents**: `crates/ui-contract/src/runtime/keys/route.rs`,
  `.../keys/input.rs`, `.../focus/policy.rs`, `crates/font-manager/src/picker.rs`,
  `.github/workflows/workspace.yml`, `scripts/test-workspace-tests.py`, `Cargo.lock`.
- photocraft, 37 files touched, **5 differ from both parents**: `crates/automation/src/rpc/tests.rs`,
  `crates/ui-egui/src/canvas.rs`, `.../canvas/canvas_tool_menu.rs`, `crates/ui-egui/src/lib.rs`,
  `.../tools/retouch_ui.rs`.

Two of those twelve are not code the fork keeps: `Cargo.lock` is regenerated, and
`scripts/test-workspace-tests.py` was later replaced by upstream's copy whole (document 01, section C,
says why). The rest are explained line by line.
