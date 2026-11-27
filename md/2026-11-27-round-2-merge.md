# The second round of 2026-11-27: the merge, and the seven review findings

## What was merged

| repository | branch | the upstream commits merged | merge commit |
|---|---|---|---|
| craft-libs | `next` | 13, over `ea14416aa`, at `a46af76c7` | `b3e81eb46` |
| photocraft | `main` | 53, over `eff5ee712`, at `984c304b2` | `efcbecc20` |

craft-libs merged with no conflict. photocraft had one, in
`crates/ui-egui/src/tools/brush_cursor.rs`: upstream rebuilt the cursor builder and the fork adds a
Quick Selection badge to it.

## The three-way audit

Against the merge base, no file of either side was dropped: 0 of 36 upstream files and 0 of 190 fork
files for photocraft, 0 of 9 and 0 of 52 for craft-libs. Every file both sides touched keeps the lines
both added, read line by line; the only difference on purpose is the fork's badge line.

## The seven findings of pull request #2239, and what each became

| # | finding | what the tree does now |
|---|---|---|
| 1 | `\{\i0}` renders as tag text in libass | measured six encodings through ffmpeg 7.1: `A\{\i0}NB` comes back as `A\NB` — backslash kept, tag invisible; the fullwidth alternative drops two round-trip trials. No change. |
| 2 | a failed CUPS listing is reported as an empty system | `output_within_full` keeps the exit status and stderr; `printers_via_checked` returns `None` on a failed exit unless it is the driverless-only answer, and a successful empty listing stays empty. |
| 3 | a Space press and its release defer each other on every pass | the input hook no longer defers a Space press for its release. |
| 4 | a Space press emits `Intent::Activate` at once | `PressRoute::Space` holds the stop; the pass that carries the release activates the held record once and clears it. Enter stays immediate. |
| 5 | two documents describe a hold the arm did not have | both carry a correction quoting the code as it stands. |
| 6 | the printer-retention example states the wrong behaviour | section 3.3 quotes `State::poll`: a failed listing keeps the destination, a successful empty listing selects none, and the field is `settings.destination`. |
| 7 | the crate commands read as a workspace run | the eight crate commands are labelled a focused check, and the workspace command is listed on its own. |

## The Quick Selection badge, and the rule that decides it

The framework gives `BrushCursor::badge` on whichever pointer the Painting Cursors setting draws —
the ring, an icon, the precise crosshair. The fork hands it the selection read the options bar shows
(`tool_feedback::selection_op`), so New adds with ⇧ and subtracts with ⌥. Upstream marks the ring a
second time from a narrower rule of its own; two marks on one ring fail two of upstream's own trials.
The fork's rule is the one kept, the narrower rule is removed, and the two trials now state the rule the
pointer follows: ⇧ puts a `+` on the ring, ⌥ with a selection a `−` on whichever form is drawn, and ⌥
with nothing selected changes nothing.

## Measured on the merged trees

| what | craft-libs | photocraft |
|---|---|---|
| workspace suite | 18,729 run, 18,728 passed, 63 skipped, 1 failed | 15,869 run, 15,869 passed, 96 skipped |
| format, clippy | 0, 0 | 0, 0 |
| layers, panel commands | - | 0, 0 |
| wasm, parity | - | 0, 0 |
| design lint | 28 passed | - |

The single failure is upstream's own `craft-xlsx` stack test
(`xlsb::formula::tests::a_long_chain_of_strings_stays_within_the_depth_bound`), which builds a 256 KiB
thread for 4,096 chained strings; the fork holds no change to `crates/xlsx`, and upstream's own runs on
`next` are red as well. The release executable is 333,062,144 bytes (sha256 `31B2D8C1…`), starts, and
shows its window; the space the build took is released afterwards (photocraft's `target` 120.35 GB to
0.93 GB, craft-libs' 237.62 GB to nothing, 366.6 GB free on the drive).
