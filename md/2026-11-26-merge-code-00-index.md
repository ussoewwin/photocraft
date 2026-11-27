# The upstream merges of 2026-11-25 and 2026-11-26

> **Branches as they stand today.** `storytold/craft-libs` and `storytold/photocraft` both have one
> line, `main`; upstream merged the old `next` line into `main` and deleted the branch on 2026-11-27
> (pull requests #3644 and #3645 there), and this fork's own branch was renamed `next` to `main` the
> same day. Where a command or a measurement below names `next`, it names the line as it was at that
> moment and is kept as the record of that round.

## Scope

Both repositories were brought to the current official upstream while upstream kept moving, and the
fork's own features were kept on top.

| repository | branch tip | the fork before | the upstream commits merged, in order |
|---|---|---|---|
| craft-libs | `e629d2315` | `75866f6b1` | `93c2e0825`, `0cb8ed397`, `7635245a9`, `8f1bb4c46`, `e17227177` |
| photocraft | `eaf7ef1e9` | `a3915ab2f` | `dbd191586`, `d827906d6`, `c5aea7e75`, `f0205ae33` |

Each merge is a commit of its own; the repair commits that followed answer what a merge or an
upstream change broke.

## The documents

| document | what it holds |
|---|---|
| [01 - craft-libs' code](https://github.com/ussoewwin/craft-libs/blob/main/md/2026-11-26-merge-code-01-fork-code.md) | every file a merge decided on, with the code |
| [02 - photocraft's code](https://github.com/ussoewwin/photocraft/blob/main/md/2026-11-26-merge-code-02-fork-code.md) | the same for this repository |
| [03 - craft-libs' range and its measurements](https://github.com/ussoewwin/craft-libs/blob/main/md/2026-11-26-merge-code-03-upstream-inventory.md) | every upstream commit, the measurements, and what they do not prove |
| [03 - photocraft's range and its measurements](https://github.com/ussoewwin/photocraft/blob/main/md/2026-11-26-merge-code-03-upstream-inventory.md) | the same |
| [04 - the upstream code behind craft-libs' answers](https://github.com/ussoewwin/craft-libs/blob/main/md/2026-11-26-merge-code-04-upstream-code.md) | each answer's upstream lines, quoted with their numbers |
| [04 - the upstream code behind photocraft's answers](https://github.com/ussoewwin/photocraft/blob/main/md/2026-11-26-merge-code-04-upstream-code.md) | the same |

## What "a merge decided" means

Not every file the fork holds. A file is a decision when its merged content differs from **both**
parents, or when a repair wrote it after a merge.

```powershell
git diff --name-only <first-parent> <merge>
git show <merge>:<path>; git show <first-parent>:<path>; git show <second-parent>:<path>
```

## Re-deriving a claim

- The fork's code: `git diff e17227177 HEAD -- <path>` in craft-libs, `git diff f0205ae33 HEAD -- <path>` in photocraft.
- The upstream code: `git show <upstream>:<path>` with that commit's line numbers.
- The range: `git log --format='%h %s' 75866f6b1..e17227177` and `a3915ab2f..f0205ae33`.
- The measurements: each document 03 lists the commands with their numbers and their limits.
