# Changelog

PhotoCraft's release history, newest first. Every release has full notes on GitHub; the last line of
each entry links to them.

## Version 1.4.0

- **Changed**: **the application rides the current official upstream** - `storytold/photocraft` `main`
  at `f0205ae33` (254 commits over `a3915ab2f`, of which 141 are change commits), with PhotoCraft's own
  features kept on top: LaMa Cleaner, SAM 3.1 selection, the quick-selection mode bar in the options
  bar, dragging selected pixels between documents, the Japanese interface and its font names.
- **Changed**: **the shared framework follows upstream's pin** - `storytold/craft-libs` at `5c30761a8`
  (the `next` line) for the framework crates and `249a41bd9` for `craft-develop`, so the application
  builds against the revisions upstream itself pins.
- **Fixed**: **the Move tool's refused press keeps its document** - a press a lock refuses records the
  point, the layer, the document and the window that owned it, so the press cannot become a
  cross-document transfer by accident (#11971).
- **Fixed**: **the smart-filter stage trial runs where its cache is its own** - it moved to the
  global-state trials, which run one at a time, instead of sharing a process with the parallel screen
  trials.
- **Suites**: the workspace runs **15,782 passed / 0 failed / 96 skipped**; `cargo xtask test-corpus`
  reports 670 passed / 0 failed / 11 ignored; check, clippy, format, layers, panel-commands, wasm,
  parity and the panic hunt all exit 0. The release executable builds and starts.
- See [Release Notes v1.4.0](https://github.com/ussoewwin/photocraft/releases/tag/v1.4.0) for details.

## Version 1.3.9

- **Changed**: **the application rides the current official upstream** - `storytold/photocraft` `main`
  at `e22facf45` (108 commits, of which 63 change commits), with PhotoCraft's own features kept on
  top: LaMa Cleaner, SAM 3.1 selection, the quick-selection mode buttons in the options bar,
  dragging selected pixels between documents, and the localized Japanese font names.
- **Changed**: **the shared framework is pinned at `3656e8aaa4`** (`storytold/craft-libs`, the `next`
  line) with `craft-develop` at `249a41bd93` (the `main` line) and egui at `a87d00caa`
  (`storytold/egui`), so one checkout satisfies every revision the application pins.
- **Changed**: **the lock file records both sides of the merge**, so the pinned revisions and the
  upstream additions resolve together.
- **Suites**: the workspace runs **15,460 passed / 0 failed / 96 skipped** (one GPU timing trial
  passed on its retry); `cargo xtask test-corpus` exits 0 with 670 passed / 0 failed / 11 ignored.
- See [Release Notes v1.3.9](https://github.com/ussoewwin/photocraft/releases/tag/v1.3.9) for details.

## Version 1.3.8

- **Changed**: **the application rides `storytold/photocraft` `main` at `e56522780`**, with
  PhotoCraft's own features kept on top: LaMa Cleaner, SAM 3.1 selection, the quick-selection mode
  buttons in the options bar, dragging selected pixels between documents, and the localized
  Japanese font names.
- **Changed**: **the shared framework is pinned at `5c5e0d52c3`** (`storytold/craft-libs`, the
  `next` line) with `craft-develop` at `249a41bd93` (the `main` line), so one checkout satisfies
  every revision the application pins.
- **Fixed**: **a GPU trial's process abort on this Windows driver is retried once**, so a suite run
  reports the trial's own result instead of the driver's exit code.
- **Changed**: **the smart-filter stage trial now lives with the process-wide state trials**, where
  it takes its turn instead of sharing a process with 1,200 parallel trials. Its assertions are
  unchanged.
- **Suites**: the workspace runs **15,362 passed / 0 failed / 96 skipped** (one GPU trial passed on
  its retry); `cargo xtask test-corpus` exits 0 with 670 passed / 0 failed / 11 ignored.
- See [Release Notes v1.3.8](https://github.com/ussoewwin/photocraft/releases/tag/v1.3.8) for details.

## Version 1.3.7

- **Changed**: **the application now rides the current official upstream** - `storytold/photocraft`
  `main` at `dbad7de99a`, bringing the Adjustment Brush tool, the rewritten font picker, the
  floating-document window title bar and the toolchain pin at Rust 1.100. PhotoCraft's own features
  stay on top of it unchanged: LaMa Cleaner, SAM 3.1 selection, the quick-selection mode buttons in
  the options bar, dragging selected pixels between documents, and localized Japanese font names.
  The tools a user can pick now number 72, which is what the README, `Tool::ALL` and the trial over
  both report.
- **Changed**: the shared framework is pinned at
  [`8d436f486a`](https://github.com/storytold/craft-libs/commit/8d436f486a2e890930ffdf10b602935feb492c97)
  (was `aa8a00a3`); the fork's `next` carries both of `storytold/craft-libs`' lines, so one checkout
  satisfies every revision the application pins.
- **Fixed**: **a face that maps its icons into the private-use areas no longer breaks the font
  picker** - a patched face such as a Nerd Font lists those codepoints first, and the sample was
  laid out in the interface face, which has no glyph for them (`No glyph for U+F030 (U+F030) in
  Proportional`). Samples now skip the private-use areas.
- **Fixed**: **the engine's split into `engine-core` no longer leaves the fork's own trials
  broken** - a bare `Session::new()` installs no command there, so the LaMa Cleaner, SAM 3.1
  selection and move-tool trials answered `NotInstalled`; they start from
  `crate::install::session()` now. A compile does not catch this one, only the trials do.
- **Fixed**: **the preferences trials run in the interface language they assert** - upstream's new
  rows are looked up by the English words they paint, and a Japanese interface paints Japanese;
  the shared test harness opens those pages in English, and the application still follows the
  user's own language.
- **Tests**: 14,851 tests pass (`cargo nextest run --workspace`), clippy is clean under
  `-D warnings`, the web build checks 20 crates plus the HEIF encoder, and the PSD/ATN/XCF corpora
  run 667 trials green.
- See [Release Notes v1.3.7](https://github.com/ussoewwin/photocraft/releases/tag/v1.3.7) for details.

## Version 1.3.6

- **Fixed**: **`Edit > Transform > Skew` now reads ゆがみ** - it read 斜体 ("italic"), which is a
  type-style word rather than a transform. The two hints that belong to it follow:
  `Set horizontal skew` and `Set vertical skew` read 水平方向のゆがみを設定 and 垂直方向のゆがみを設定.
- **Fixed**: **`Edit > Transform > Distort` now reads 多方向に伸縮** - it read ゆがみ, the same word
  the `Filter > Distort` submenu already uses, so two different commands read as one tool. The
  submenu keeps ゆがみ: the command is translated by its own command id
  (`edit.transform.distort`), the submenu title by the plain `Distort` row.
- **Changed**: both rows now live in the Japanese catalog this repository carries itself
  (`crates/ui-egui/i18n/fork/ja/photocraft.tsv`), which `crates/ui-egui/build.rs` embeds with or
  without a craft-i18n checkout, and which wins over a checkout for the same locale. The file the
  v1.3 rows went into (`crates/ui-egui/src/i18n/ja.tsv`) is read by no build.
- **Tests**: `i18n::tests::the_fork_settles_the_transform_submenu_in_japanese` pins the three rows
  and runs without a checkout.
- See [Release Notes v1.3.6](https://github.com/ussoewwin/photocraft/releases/tag/v1.3.6) for details.

## Version 1.3.5

- **Changed**: the app is built against the PhotoCraft fork of the shared UI framework
  ([`ussoewwin/craft-libs`](https://github.com/ussoewwin/craft-libs), branch `next`), pinned at
  [`aa8a00a3`](https://github.com/ussoewwin/craft-libs/commit/aa8a00a35f90cbfae071762c21d86523607f09bf)
  (was `18ffc0751`). Nothing else in the application changed; the visible difference comes from the
  framework.
- **Fixed**: **the menu titles in the window's title bar no longer overflow** - the bar reports the
  width its titles actually take (`MenuBar::own_width`, `MenuBarOutput::titles_width`) and the title
  bar declares it (`TitleBar::leading_width`), so `Window` and `Help` stay reachable. Measured: a
  2000 × 1083 window left 439 px for titles that measure 446.8 px.
- **Fixed**: **a declared width of zero is no declaration** - a host that has not measured yet reports
  `0`, and the leading region must not be laid out at zero points for it.
- See [Release Notes v1.3.5](https://github.com/ussoewwin/photocraft/releases/tag/v1.3.5) for details.

## Version 1.3.4

- **Added**: **the LaMa Cleaner downloads its model on first use** - when `big-lama.onnx` is missing,
  `paint.lamaCleaner` fetches it into `plugins/lama-cleaner/models` and the first stroke works with no
  manual step. The download streams through a `.part` file, checks `Content-Length`, reports progress
  every 10 s and lands with a rename; `ureq` 2 with rustls keeps the helper pure Rust.
- **Changed**: every path is derived from the preferences plug-ins folder or the helper's own
  location, so the same build works on any machine, and a shipped helper carries the same download
  path.
- **Fixed**: an explicit `--model` or `PHOTOCRAFT_LAMA_MODEL` that names a missing file stays an
  error; a mistyped path is never quietly replaced by a download.
- **Measured**: with the local copy deleted, the first stroke downloaded 198.4 MiB, loaded it on CUDA
  in 8.75 s and finished the command in 37 s; the steady state stays at 87-95 ms per stroke through
  the resident worker. The model is redistributed byte-identical (SHA-256
  `1faef5301d78db7dda502fe59966957ec4b79dd64e16f03ed96913c7a4eb68d6`) with its full attribution
  chain on Hugging Face.
- See [Release Notes v1.3.4](https://github.com/ussoewwin/photocraft/releases/tag/v1.3.4) for details.

## Version 1.3.3

- **Added**: **SAM Selection downloads its pack on first use** - when the 1.74 GB HSWQ ConvRot INT8
  pack is missing, `select.sam3` fetches it into `plugins/sam3/models`, with the same `.part`
  streaming, `Content-Length` check, progress lines and final rename. `CRAFT_SAM3_PACK` still wins:
  when it is set, no download is attempted.
- **Changed**: a failed download reports the helper's stderr tail plus the manual download URL, so the
  next step is in the error itself.
- **Tests**: a new engine test pins the search order and the folder a download lands in.
- See [Release Notes v1.3.3](https://github.com/ussoewwin/photocraft/releases/tag/v1.3.3) for details.

## Version 1.3.2

- **Added**: **the SAM Selection tool** - SAM 3.1 open-vocabulary detection and interactive
  segmentation, implemented end to end in pure Rust on CUDA from a 1.74 GB HSWQ ConvRot INT8 pack. A
  click, a box drag or a text prompt such as `face:3, person` turns an object into a canvas selection;
  a resident `--serve` worker keeps the pack in VRAM and caches the resized frame.
- **Measured**: masks against the reference oracle cosine 0.9999990-0.9999995, refine IoU 0.999968,
  box-prompt IoU 1.000000, text end-to-end IoU 0.999968 - all inside the f16 noise floor. Speed: pack
  load 15.2 s once per worker, backbone 12.3-13.5 s per frame, 123-204 ms per interactive prompt.
- **Changed**: `localized_families` no longer waits for the font scan on a UI frame, so the type-tool
  font menu stays responsive while faces are enumerated; the SAM labels resolve from the tree rather
  than the catalog pin.
- See [Release Notes v1.3.2](https://github.com/ussoewwin/photocraft/releases/tag/v1.3.2) for details.

## Version 1.3.1

- **Changed**: **the LaMa Cleaner runs on the GPU** - CUDA with an automatic DirectML then CPU
  fallback, and the model stays loaded in a resident helper, so a stroke costs about 90 ms instead of
  paying a 6.7-8.6 s model load every time.
- **Fixed**: CUDA and cuDNN runtimes are found through `CUDA_PATH` / `CUDNN_PATH`, the Program Files
  layouts and `PATH`, with no PATH setup needed, and every DLL in those folders is preloaded - cuDNN
  9.26 ships one that ONNX Runtime's own list omits, which used to fail the first convolution.
- **Changed**: a provider that cannot run the model is replaced by the next one and the same patch is
  re-run, so a stroke completes slower instead of failing.
- See [Release Notes v1.3.1](https://github.com/ussoewwin/photocraft/releases/tag/v1.3.1) for details.

## Version 1.3

- **Fixed**: **the Japanese `Edit > Transform` labels** - `Skew` read 斜体 ("italic") and `Distort`
  read ゆがみ (the word the `Filter > Distort` submenu already used), so two different commands read
  as the same tool. They are now ゆがみ and 多方向に伸縮, and the shared `Distort` row keeps ゆがみ for
  the submenu.
- **Changed**: English and every other locale are untouched; only the Japanese catalog changed.
- See [Release Notes v1.3](https://github.com/ussoewwin/photocraft/releases/tag/v1.3) for details.

## Version 1.2

- **Added**: **native Japanese typography** - family names resolve from the OpenType `name` table
  (platform 3, language `0x0411`, record 16 then record 1), so the UI shows 游ゴシック, メイリオ,
  ＭＳ ゴシック, HG行書体 and their like instead of ASCII fallbacks. `TypePrefs.show_font_names_in_english`
  now defaults to `false`, and a Japanese UI prioritises localized names.
- **Added**: **dynamic Japanese glyph preview** - a font picker whose face covers Hiragana samples
  あいうえお instead of `AaBbCc`, including the 40 pt banner in the font options panel.
- **Added**: **Japanese visual classification** in `crates/text/src/font_class.rs` - Mincho and Song
  faces as serif, Gothic and Meiryo as sans serif, 行書 / 楷書 / 草書 / 隷書 / 勘亭流 as script,
  textbook and pop faces as handwritten.
- **Docs**: the full architecture is in `md/japanese-font-typography-specification.md`.
- See [Release Notes v1.2](https://github.com/ussoewwin/photocraft/releases/tag/v1.2) for details.

## Version 1.1

- **Added**: **the LaMa Cleaner inpainting brush** - paint freely over an object, watermark or
  imperfection, release the button, and the region is inpainted in a single undo/redo step, with a
  translucent yellow stroke trail tracking the cursor.
- **Changed**: the pipeline follows IOPaint's parameters - a 128 px minimum context crop that extends
  to 512 px, reflection padding to multiples of 8, 5 px mask dilation and a 3 px feathered edge -
  with the non-masked pixels preserved exactly. The engine is native Rust with ONNX Runtime: no Python
  and no external daemon.
- **Added**: a 1-300 px brush-size slider in the options bar, and the tool at the bottom of the Brush
  flyout on the left toolbar.
- See [Release Notes v1.1](https://github.com/ussoewwin/photocraft/releases/tag/v1.1) for details.

## Version 1.0

- **Added**: **portable single-instance forwarding** - in portable mode a second launch (file
  association, external shell) detects the running process through the IPC single-instance socket and
  lockfile, forwards its file paths to that window and focuses it instead of opening a duplicate.
- **Fixed**: **saving onto cloud drives and WebDAV mounts** - virtual file systems that reject Windows
  directory synchronisation with `os error 50` / `os error 1` no longer abort a save, which now lands
  atomically without a false durability error.
- **Added**: **Quick Selection mode controls** - New, Add and Subtract buttons in the options bar with
  their own vector glyphs and live modifier badges.
- **Fixed**: **the Move tool across tabs and floating windows** - a dragged selection or floating
  layer follows the active origin viewport and drops into the destination document's coordinates.
- **Synced**: upstream parity to `v0.21.0` - smart object import, vertical text alignment, the brush
  preset angle/roundness picker, and graded alpha falloff for the Background Eraser. Layer separation
  verified across all 31 workspace crates.
- See [Release Notes v1.0](https://github.com/ussoewwin/photocraft/releases/tag/v1.0) for details.
