# Merge documentation: the fifth round, 2026-11-28

Two repositories, one round: `craft-libs` took `upstream/main` `51fcc3b04` (328 commits over
`a7ff12f4d`, merged as `02419852d`) and `photocraft` took `upstream/main` `8d7c1f833` (207 commits
over `904374406`, merged as `581eb807b`). A repair commit follows each merge: craft-libs
`023528394` and photocraft `359743c5f`.

## Scope

- `craft-libs`: the merge, then the five-hunk repair that moved the coloured-text primitives to
  layer 2 and put `ColorRange` on the lint's closed list.
- `photocraft`: the merge, then the six-file repair the framework's new APIs forced (`Bind` cells,
  `ShapeGeometry::Triangle`'s corner radius, the SAM row's tracked bindings, one brace).

## The isolation test

A file in a merge is the fork's own answer when its merged content differs from **both** parents:

`git show <merge>^1:<path>`, `git show <merge>^2:<path>` and `git show <merge>:<path>` compared by
blob hash. Measured this round:

- craft-libs: 348 files touched by the merge; **1** differs from both parents (`Cargo.lock`, which
  regenerates), and the four conflicted files took upstream's side, because upstream had fixed what
  the fork had registered as exemptions the round before.
- photocraft: 235 files touched; **21** differ from both parents, listed in document 02.

Files the fork changed **after** the merge are answers too: craft-libs 5 (`023528394`), photocraft
6 (`359743c5f`), listed after the merge files in documents 01 and 02.

## Documents

- [01](2026-11-28-merge-code-01-craft-libs.md) - the framework's kept code and its repair.
- [02](2026-11-28-merge-code-02-photocraft.md) - the application's kept code and its repair.
- [03](2026-11-28-merge-code-03-upstream-inventory.md) - both ranges with every non-merge commit listed (198 of 328, 123 of 207), the measured
  numbers, and what they do not prove.
- [04](2026-11-28-merge-code-04-upstream-code.md) - the upstream lines that forced each answer,
  quoted with their path and line.

## How to re-derive each claim

```text
git -C craft-libs rev-list --count a7ff12f4d..51fcc3b04            # 328
git -C craft-libs rev-list --count --no-merges a7ff12f4d..51fcc3b04 # 198
git -C photocraft rev-list --count 904374406..8d7c1f833            # 207
git -C photocraft rev-list --count --no-merges 904374406..8d7c1f833 # 123
git -C <repo> diff 581eb807b^2 581eb807b -- <path>                  # the diffs below
```

## What is not proven here

- Upstream's own code is not reproduced whole: 535 commits would be a copy of `storytold`'s
  repositories. Document 03 lists every non-merge commit and counts the merges; document 04 quotes
  upstream only where it intersects the fork.
- The suites ran on the working trees the two merge trees record, at the times document 03
  states; a run is evidence for the tree it ran on, not for later commits.
