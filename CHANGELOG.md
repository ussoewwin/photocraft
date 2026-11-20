# Changelog

PhotoCraft's release history, newest first. Every release has full notes on GitHub; the last line of
each entry links to them.

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
