# The upstream merge of 2026-11-25 and 2026-11-26

## Scope

Two repositories were brought up to the current official upstream, and the fork's own features were
kept on top of it.

| repository | merge commit | the fork before the merge | the upstream commit the merge took |
|---|---|---|---|
| craft-libs | `698b1a4db` | `89ed9dfd7` | `6fcf61c81` (the `next` line) |
| photocraft | `9ecee7176` | `b86838240` | `e22facf45` (the `main` line) |

The upstream ranges are 213 commits in craft-libs (175 of them change commits) and 108 in
photocraft (63 change commits). Every one of them is listed in the inventory documents.

## The documents

| document | repository | what it holds |
|---|---|---|
| [01 - the fork's code in craft-libs](https://github.com/ussoewwin/craft-libs/blob/next/md/2026-11-26-merge-code-01-fork-code.md) | craft-libs | every file whose content in the merge differs from both parents: the role of the file, the code the fork holds, what each hunk does, and the command that catches a revert |
| [02 - the fork's code in photocraft](https://github.com/ussoewwin/photocraft/blob/main/md/2026-11-26-merge-code-02-fork-code.md) | photocraft | the same for this repository |
| [03 - the upstream range and the verification, craft-libs](https://github.com/ussoewwin/craft-libs/blob/next/md/2026-11-26-merge-code-03-upstream-inventory.md) | craft-libs | every commit of the range, the measurements, and what they do not prove |
| [03 - the upstream range and the verification, photocraft](https://github.com/ussoewwin/photocraft/blob/main/md/2026-11-26-merge-code-03-upstream-inventory.md) | photocraft | the same for this repository |
| [04 - the upstream code behind the fork's answers, craft-libs](https://github.com/ussoewwin/craft-libs/blob/next/md/2026-11-26-merge-code-04-upstream-code.md) | craft-libs | for each adaptation: the upstream lines that forced it, what they do, why they broke the fork, and the fork's answer |
| [04 - the upstream code behind the fork's answers, photocraft](https://github.com/ussoewwin/photocraft/blob/main/md/2026-11-26-merge-code-04-upstream-code.md) | photocraft | the same for this repository |

## The isolation test

A file is the fork's own when its content in the merge commit differs from **both** of the merge's
parents. The test is mechanical, and it is the only definition used in these documents:

```powershell
# every file the merge touched
git diff --name-only <first-parent> <merge-commit>
# the three copies that decide the question
git show <merge-commit>:<path>     # the merged tree
git show <first-parent>:<path>     # the fork before the merge
git show <second-parent>:<path>    # the upstream commit
```

## How to re-derive each claim

- **The fork's code.** Every diff in documents 01 and 02 is
  `git diff <second-parent> <merge-commit> -- <path>`, taken from the repository that owns the file.
- **The upstream code.** Every quotation in document 04 is
  `git show <second-parent>:<path>`, with the line numbers of that commit.
- **The upstream range.** Every commit in document 03 is
  `git log --format='%h %s' <first-parent>..<second-parent>`; the change commits alone are
  `git log --no-merges --format='%h %s'` over the same range.
- **The measurements.** The commands and their results are in the "What was measured" part of each
  document 03, and the numbers are the ones the run produced.
- **What is not proven.** Each document 03 ends with the claims the measurements do not support.

## The measurements of this run

- craft-libs: 18,485 trials passed, 0 failed, 63 skipped; the workspace clippy run and the format
  check exit 0; the design lint's workspace check passes.
- photocraft: 15,460 trials passed, 0 failed, 96 skipped; the corpus run passes 670 of 670; the
  workspace check, the clippy run and the format check exit 0; the layering, panel-command, wasm,
  parity and panic-hunt gates pass; the release executable builds and starts.

The numbers are stated with their commands in the two inventory documents, and what they do not
prove is listed there as well.
