# The upstream merge of 2026-11-25 and 2026-11-26

## Scope

Both repositories were brought to the current official upstream, and every fork feature was kept on
top. The merges are separate because the two repositories are: craft-libs is a library the other
repository pins by revision.

| repository | merge commits | the fork before | the upstream commits taken |
|---|---|---|---|
| craft-libs (`next`) | `0507d5acb` | `75866f6b1` | `93c2e0825` (190 commits, 117 change commits) |
| photocraft (`main`) | `214e45a35`, `06dd81e8b` | `a3915ab2f` | `dbd191586` then `d827906d6` (225 commits, 128 change commits) |

A repair commit followed on craft-libs: `9f430f19b` (`Space` routing complexity, and Space on a
focused icon button).

## The documents

| document | repository | what it holds |
|---|---|---|
| [01 - the fork's code in craft-libs](https://github.com/ussoewwin/craft-libs/blob/next/md/2026-11-26-merge-code-01-fork-code.md) | craft-libs | every file the merge decided on, with the role, the code and what each hunk does |
| [02 - the fork's code in photocraft](https://github.com/ussoewwin/photocraft/blob/main/md/2026-11-26-merge-code-02-fork-code.md) | photocraft | the same for this repository |
| [03 - the upstream range and the verification, craft-libs](https://github.com/ussoewwin/craft-libs/blob/next/md/2026-11-26-merge-code-03-upstream-inventory.md) | craft-libs | every upstream commit, the measurements, and what they do not prove |
| [03 - the upstream range and the verification, photocraft](https://github.com/ussoewwin/photocraft/blob/main/md/2026-11-26-merge-code-03-upstream-inventory.md) | photocraft | the same for this repository |
| [04 - the upstream code behind the answers, craft-libs](https://github.com/ussoewwin/craft-libs/blob/next/md/2026-11-26-merge-code-04-upstream-code.md) | craft-libs | each adaptation's upstream lines, quoted with their numbers |
| [04 - the upstream code behind the answers, photocraft](https://github.com/ussoewwin/photocraft/blob/main/md/2026-11-26-merge-code-04-upstream-code.md) | photocraft | the same for this repository |

## What "the merge decided" means

Not every file the fork holds. A file is a decision when its merged content differs from **both**
parents, or when a repair commit wrote it after the merge. Everything else came from one side
unchanged, and is not listed - the fork's own identity (its crates, its documents, its plugins) is a
different question and is not what a merge decides.

```powershell
git diff --name-only <first-parent> <merge>      # what the merge touched
git show <merge>:<path>                          # the merged copy
git show <first-parent>:<path>                   # the fork's copy
git show <second-parent>:<path>                  # the upstream copy
```

## Re-deriving a claim

- The fork's code: `git diff d827906d6 HEAD -- <path>` in photocraft, `git diff 93c2e0825 HEAD -- <path>` in craft-libs.
- The upstream code: `git show <upstream>:<path>`, with that commit's line numbers.
- The range: `git log --format='%h %s' a3915ab2f..d827906d6` in photocraft, `75866f6b1..93c2e0825` in craft-libs.
- The measurements: the commands and their numbers are in each document 03, with the limits stated.
