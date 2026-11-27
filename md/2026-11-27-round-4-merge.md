# The fourth round of 2026-11-27 in photocraft

## What was merged

`storytold/photocraft` `702da0278` — 19 commits over `984c304b2` — merged into `main` as
`049a1d8e4` with no conflict, behind 0 afterwards. The upstream range carries the merge-queue batch
#17136 (#14621, #15962, #17117) and the toolbar footer memo (#17143), besides the color-settings
text gamma work and the channel-calculation trials.

## The pin, and why it moved twice

The framework crates are pinned to `storytold/craft-libs` by revision. This round merged craft-libs'
`c0977ad61` first, so the pin follows it: 41 entries in `Cargo.toml` moved from `a46af76c7` — where
upstream's own `Cargo.toml` still sat — to `c0977ad61`, and `Cargo.lock` was regenerated and
committed with them (`b5afaaa1b`). `git merge-base --is-ancestor c0977ad61 HEAD` in the craft-libs
checkout exits 0, which is what `cargo xtask craftlibs on` needs to patch the crates from that tree.

## Measured

| what | result |
| craftlibs status | config on the pin, lock safe to commit |
| format, clippy | 0, 0 |
| layers, panel-commands, wasm, parity | 0, 0, 0, 0 |
| panic hunt (ignored trials) | 1 passed, 0 failed |
| workspace suite | 15,898 run, 15,898 passed, 96 skipped (1,150.6 s) |
| release executable | 333,083,648 bytes, window titled `PhotoCraft`, no process left behind |
| merge check | 0 dropped upstream files, 0 lost fork files |
