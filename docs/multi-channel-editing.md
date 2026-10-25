# Editing selected colour channels

Shift-click colour rows in the Channels panel to add or remove them from the editing
set. Only native colour channels can be combined. Selecting every colour channel
returns to the composite. Channel visibility can still be changed independently.

Partial sets work with the existing raster drawing tools (Brush, Pencil, erasers,
stamps, Mixer Brush, retouching tools, Bucket, Gradient and raster Fill/Stroke Path),
destructive adjustments, and the supported native raster filters. Erasing paints
the toolbox background colour into the selected components, preserving transparency.
Copy, Cut, Paste, Paste Into and Paste Outside carry the selected component planes.
Each committed operation records one undo step. Live drawing previews protect the
same components as the committed edit.

Levels and Curves show a neutral combined entry (for example **RB**) followed by the
selected individual components. Colour adjustments that mix components and filters
retain the original native layer context,
including transparency textures, depth inputs and colour-valued settings, and restore
unselected encoded samples. The combined tone operation is applied to each selected
native plane; individual settings can refine those planes. Histograms use
the selected layer's native component values. CMYK channels display tone (the inverse
of stored ink); Lab chroma uses the native neutral encoding at each depth.

RGB, CMYK and Lab are exercised at 8-bit, 16-bit and 32-bit float depths. Unselected
encoded samples, including transparency and float NaN payloads, are preserved byte
for byte. Active pixel selections, including feathering, still constrain editing.

Move and pixel transforms, including Content-Aware Scale and mesh warps, on partial
sets remain unavailable. Camera Raw requires
the composite or one colour channel. HDR Toning requires the composite because it flattens
the document. Filters that generate transparency cannot edit
colour-only targets. Options that create a new layer, including Remove’s Create a new layer option, also
require the composite.
Rasterize a non-raster layer before editing its colour planes.
Vector path transforms and moving guides remain available. This work is independent
of [#4648](https://github.com/storytold/photocraft/pull/4648) and
[#6864](https://github.com/storytold/photocraft/pull/6864); it does not include their
isolated-channel geometry or remaining scalar-target tone-dialog changes.

## Automation

The UI uses the existing engine commands. Native component indices follow the
document colour mode: RGB 0–2, CMYK 0–3 and Lab 0–2.

```json
{"command":"channel.target","params":{"colors":[0,2]}}
{"command":"channel.target","params":{"channel":{"color":1},"extend":true}}
{"command":"image.adjustments.curves","params":{"target":{"colors":[0,2]},"points":[[0,0],[128,170],[255,255]]}}
```

`extend` toggles a colour row; from the composite it toggles that row out of the full
set. Alpha channels, masks and Quick Mask cannot join a colour set. An explicit
`target` overrides the panel selection. Without it, applicable pixel commands follow
the selected set. The channels reply exposes `selectedColors` in addition to the
legacy primary `target`. Dialogs capture their target when opened; changing the panel
while a dialog is open does not redirect its operation.

## Validation and performance

`crates/engine/tests/multi_channel_editing.rs` covers the native mode/depth matrix,
selected-plane math, byte preservation, live/committed drawing, selections, clipboard,
explicit layer targets, HDR Curves, selected-default masks, missing destinations,
errors, undo and redo. UI tests cover the selected tone
choices, captured drawing targets and guide/path interaction.

The release benchmark is `crates/engine/examples/multi_channel_bench.rs`. It uses a
6000 × 4000 synthetic RGB16 raster, one warmup and six measured repetitions per case,
with target modes interleaved. Document setup is outside the timed interval; command
execution, pixel edits and history commit are inside. Eraser is the ordinary Brush
eraser. Live-update cases time one 50-pixel extension after stroke creation. Timings
exclude display refresh and do not claim end-to-end stylus latency.
The observed maximum of six samples is reported as p95 in the raw output.

Run with `RAYON_NUM_THREADS=8 cargo +1.99.0 run --release -p photocraft-engine
--example multi_channel_bench -- --subset`. For the unchanged base, use the same
fixture, parameters and timing loop and omit
`--subset`. The base cannot execute the new partial-set target. Add `--region` for the
five imaging cases through a 16 × 16 pixel selection; selection setup stays outside
the timer. Use `--clipboard` for a separate one-pixel paste case: copy and selection
setup stay outside the timer, while pasting edits the existing target layer.

Measured on 2026-10-25 on an AMD Ryzen 7 9700X (8 cores / 16 threads), Rust 1.99.0,
release profile with the workspace's thin LTO defaults and eight Rayon workers. The
base is `6719f1813e84e96129fc31676cc945d51be0e736`; it uses the same fixture,
command parameters and timing loop. The candidate adds untimed setup options for
small selections and a separate one-pixel clipboard fixture.
Retained measurements ran without concurrent Rust compilation or test processes.
Interrupted runs are discarded and repeated once compilation and tests are idle.

| Case (milliseconds) | Base composite | New composite | Base single | New single | New RB (p50 / p95) |
|---|---:|---:|---:|---:|---:|
| Invert | 77.04 | 68.08 | 249.48 | 66.25 | 136.00 / 140.01 |
| Threshold | 93.99 | 76.32 | 275.66 | 87.99 | 178.84 / 186.18 |
| Levels (gamma 1.4) | 110.99 | 92.31 | 297.75 | 111.08 | 227.30 / 260.00 |
| Curves (one midpoint) | 98.55 | 77.52 | 271.28 | 86.82 | 172.33 / 183.65 |
| Gaussian Blur (radius 8) | 211.73 | 204.83 | 323.83 | 339.90 | 313.88 / 325.83 |
| Brush commit (120 px) | 4.67 | 4.90 | 6.71 | 7.16 | 6.69 / 6.98 |
| Eraser commit (120 px) | 4.53 | 4.41 | 6.53 | 7.11 | 6.72 / 6.79 |
| Live brush (50 px extension) | 0.33 | 0.34 | 0.33 | 1.72 | 1.50 / 1.52 |
| Live eraser (50 px extension) | 0.30 | 0.31 | 0.30 | 1.74 | 1.48 / 1.49 |

All columns are p50 except the last column's additional p95. Baseline single-channel
live previews did not protect the unselected samples; the new live timings include
that protection. The full-process peak RSS was 649.8 MiB on the base (two target
modes) and 684.3 MiB on the candidate (three modes); these are process peaks across
the complete harness, not isolated per-operation allocation measurements.

Single-channel Invert/Levels/Curves are faster than the base because native plane
edits run by tile in parallel and avoid a redundant full-layer restoration pass.
Filters retain the original native context and restore unselected encoded samples
after filtering.
Composite timings are controls; these runs establish no new performance budget or
readiness percentage.

| Case, 16 × 16 selection | RB p50 (ms) | RB p95 (ms) |
|---|---:|---:|
| Invert | 1.25 | 1.38 |
| Threshold | 1.25 | 1.40 |
| Levels (gamma 1.4) | 1.84 | 1.93 |
| Curves (one midpoint) | 1.45 | 1.83 |
| Gaussian Blur (radius 8) | 0.98 | 1.10 |

The small-selection cases use the same 24 MP layer. They measure bounded region work,
including the filter halo, with selection creation outside the timer.

The separate one-pixel Paste case measured **0.45 ms** for one channel
and **0.97 ms** for RB (p95 0.45 / 1.00 ms). Copying the pixel and
selection setup are outside the timer; the paste edits the existing 24 MP layer.
Destination plane extraction is bounded to the pasted region.

Raw samples: [base](benchmarks/multi-channel-editing/baseline.json),
[candidate](benchmarks/multi-channel-editing/candidate.json),
[small selection](benchmarks/multi-channel-editing/candidate-region.json),
[one-pixel paste](benchmarks/multi-channel-editing/candidate-clipboard.json),
[environment and binary hashes](benchmarks/multi-channel-editing/environment.json).

Validation: 6,467 engine and UI tests passed (38 ignored), including 50 focused engine
regressions. Strict all-target Clippy, formatting, layering,
wasm, parity generation and the adversarial command panic hunt passed. The complete
`cargo xtask perf --quick` runner was attempted: its layout fixture fails because
it tries to convert an already imported Smart Object, and six GPU bevel rows are
absent with the available adapter setup. These unrelated runner failures are not
reported as a passing full performance gate. The feature-specific benchmark above
completed successfully. Eleven independent review/fix rounds were completed; the
latest review found no actionable issues. Visual captures were rendered offscreen
and inspected.


## Visual evidence

The images below use an original synthetic gradient, rendered offscreen without
opening a window. Their JSON scripts are adjacent to the images. Reproduce them with
`cargo run -p photocraft-ui-egui --example snapshot -- --safe-gpu --size 1440x900
--scale 1 --out <image.png> --script "$(cat <script.json>)"`; add `--click-at 720,242`
for the Curves dropdown capture.

![Red and Blue selected; neutral combined Curves entry](images/multi-channel-editing/curves.png)

![Combined and individual Curves choices](images/multi-channel-editing/curves-choices.png)

![Combined Levels entry and native subset histogram](images/multi-channel-editing/levels.png)

![Drawing changes Red and Blue; Green stays untouched, and Remove keeps colour-only edits on the current layer](images/multi-channel-editing/drawing.png)
