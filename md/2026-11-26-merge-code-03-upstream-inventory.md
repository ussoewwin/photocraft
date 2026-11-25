# The upstream range and what was verified in photocraft

Every upstream commit this merge took, the measurements made on the merged
tree, and what those measurements do not prove.

## The upstream range

- The upstream commit this merge took: `e22facf45`.
- The fork's pre-merge head: `b86838240`.
- Commits in the range: **108**, of which **63** are change commits; the rest
  are merges.

### Every commit of the range

- `e22facf45 Merge pull request #16894 from superplay2021/fix/cmyk-vector-shape-paint`
- `a39646d49 Merge branch 'main' into fix/cmyk-vector-shape-paint`
- `8a3ff6c08 Merge pull request #16691 from davidmann4/batch/16665-16598`
- `3e094078e Pen shape-mode tests read the number before the unit next adds to a field's accessible value`
- `2f9362d24 Move craft-libs to storytold next 3656e8aaa4 (dock-gap fixes, storytold/craft-libs#3383, #3360, #3058, #3086, #3212)`
- `ffe41a5dc fix(cmyk): discard invisible ink before dropping alpha`
- `3e3831ab4 Merge remote-tracking branch 'origin/main' into HEAD`
- `b76d2a12b i18n tests: the drawn File title starts with the translation, with the access key after it on next`
- `492cbae90 fix(cmyk): preserve native CMYK in vector shape fills and strokes`
- `ac2e39a06 Merge queue batch #16717: #15767, #15421, #16049, #15092, #16446`
- `2af203280 Review fixes: a held corner widget is no snap target; a pasted CRLF moves the caret by one`
- `39f8ba661 Move craft-libs to storytold next a3174010b7 (font-manager builds for wasm again, storytold/craft-libs#3399)`
- `349af1806 Merge remote-tracking branch 'origin/main' into HEAD`
- `23bf461da Task bars stay clear of the scroll bars; the Magnetic Lasso test takes the arrow cursor`
- `5babb6a3b Merge #16446 into the merge queue batch`
- `8cf4c9951 Merge #15092 into the merge queue batch`
- `713963e39 Merge #16049 into the merge queue batch`
- `0aaf8ecfb Merge #15421 into the merge queue batch`
- `94f968388 Merge #15767 into the merge queue batch`
- `928326271 RAW #587: choose the actual frame budget for shared CFA development (#16699)`
- `dd56106df Merge queue batch #16705: #14467, #16250, #16290, #15965, #16318, #16308`
- `6e5dec786 Drop a stray trailing blank line at the test file's end`
- `76467eab5 Name the dot-parse assumption on field_number: the harness locale is the built-in one, a comma locale would need a pin`
- `399271a8e Merge upstream/main into feat/pen-shape-fill-stroke-12907`
- `928b9ff70 Merge #16308 into the merge queue batch`
- `e67693730 Merge #16318 into the merge queue batch`
- `c80a7a618 Merge #15965 into the merge queue batch`
- `223d159e6 Merge #16290 into the merge queue batch`
- `f1c86f09c Merge #16250 into the merge queue batch`
- `7e8d2959c Merge #14467 into the merge queue batch`
- `32f53c8fb Merge queue batch #16687: #16079, #16122`
- `e86d7a20b test(atn): start the engine guard tests from an installed session`
- `15c6baf62 Drop the now-unused Checkbox and VectorMode imports in shape_options: main moved the shape tools' Align Edges out of this file`
- `31d7313d2 Port the pen bar's Align Edges to main's checkbox, and fit the swatch tests to the new browser`
- `a2a757a99 Move craft-libs to storytold next c6a4484bca and egui to a87d00caa (storytold/craft-libs#3357)`
- `c8a3b0386 Call session presets as a method: main turned the engine's presets field into an accessor`
- `6281da080 Merge remote-tracking branch 'pr/16598' into HEAD`
- `c473b4108 New Document: test that documents created alike stay separate Recent items (#14038)`
- `068a52e83 Tests follow craft-libs next: set copy order, edge click, no desktop pointer`
- `87c83781e Move craft-libs to storytold next a4db799563 (own-window dialogs keep their size, task bar keeps its controls)`
- `f27cd5f0b Merge main into fix/new-doc-recent-items-14038`
- `f5ee04545 fix(engine): Auto-Blend Layers is unavailable with a type layer or smart object selected (#14204)`
- `85c63e6a4 Merge upstream/main into feat/pen-shape-fill-stroke-12907`
- `9d4d331a4 Merge #16122 into the merge queue batch`
- `11f3caafc Merge #16079 into the merge queue batch`
- `f00af0431 Merge remote-tracking branch 'origin/main' into HEAD`
- `db1402989 Merge queue batch #16679: #16655, #13486, #15281`
- `aa84e9e92 Merge #15281 into the merge queue batch`
- `5146eca6d Merge #13486 into the merge queue batch`
- `01459afbd Merge #16655 into the merge queue batch`
- `8e3529512 fix(atn): match the ask's name test to each layer-selector kind`
- `f926cafb9 fix(atn): follow main's session accessor in the engine guard tests`
- `492786e8e fix(atn): follow main's session accessor in the panel tests`
- `17d7b5aa6 ﻿Merge upstream/main into fix/atn-recording-any-document-10769`
- `530eb4191 Warp: remove the added split from its inner anchors`
- `072a7a427 Move egui to storytold/egui next 7520646e5c (VoiceOver reads floating panels, storytold/egui#58)`
- `218d0780f Move craft-libs to storytold next 7762aab37b, which carries every release fix`
- `883daa21a Merge remote-tracking branch 'upstream/main' into rc-bump-next`
- `50b2584dc Move craft-libs to rc-integration 3e5f0cbdbc (adds the memo view fill fix from craft-libs#3319: the title bar spans the window again)`
- `608760bf2 Use the internal command argument for the Warp pick`
- `a4f869fe5 Warp: remove the split line at the picked anchor`
- `0a453eb6f Move craft-libs to rc-integration df7df7c663 (next 3238a3eca8 with the release fixes and craft-libs#3319)`
- `340ad3141 Follow next's deliberate changes in the ui-egui tests; shape box drags skip their own layers as snap targets`
- `074537073 Actions tests: sets keep the order they were made in`
- `135a6b342 Move craft-libs to next`
- `9de7ffac9 WIP: RAW Develop window keeps its size (#16595)`
- `07278de6c Merge main into feat/14136-print-units-points-picas`
- `4a8374298 Dering only the columns read, and read beyond the content as the default pixel`
- `b13e942d0 Merge remote-tracking branch 'origin/main' into a-15092`
- `9abf14768 ui: grey the Count tool's one-layer rows only with several pixel layers selected (#12358)`
- `e6035facf Limit Preserve Details deringing to the measured case and bound its memory`
- `bef461389 Merge main into fix/12358-canvas-layer-menu-plural-labels`
- `a306a440c Merge remote-tracking branch 'origin/main' into tests-condition-waits`
- `a49f739ed Enlarge with Preserve Details without ringing at hard edges`
- `ff80f6869 Smudge: Alt flips Finger Painting only at Strength 100% (CodeRabbit on #16308)`
- `249a1912c Create Ellipse: open with the active ellipse's size, else 100 x 100`
- `cfe88d326 Smudge: Alt turns Finger Painting the other way for the stroke`
- `6eb8a853a Magic Eraser: alerts for a hidden layer, a locked layer and a group`
- `36601b9cb Content-Aware Fill: choosing Custom asks for the Sampling Brush`
- `8c06f525d Freeform Pen: Shift leaves the freehand stroke as drawn`
- `1205c6a1d Assign Profile: ask before assigning on a document with layers`
- `5d8594323 fix(pen): measure the W / H row against the live Freeform Pen trace`
- `c85bbe79d test(pen): pin the degenerate W / H drafts and the Curvature Pen row`
- `7b59c7136 refactor(pen): measure the Shape-mode W / H with the existing path bounds helper`
- `3d6fda134 feat(pen): show the W and H captions in the Shape-mode options bar`
- `6c296cd89 feat(pen): Fill, Stroke, W/H and Align Edges in the Shape-mode options bar`
- `4e4044ffb Merge upstream/main into fix/atn-recording-any-document-10769`
- `1be0226c4 Merge upstream/main into fix/atn-recording-any-document-10769`
- `20a50d657 Merge commit '86c2056d01ca55dd0295e314dd6f526e879155c5' into HEAD`
- `4fbc1d373 Merge remote-tracking branch 'upstream/main' into fix/atn-recording-any-document-10769`
- `0ed9f48cc Merge remote-tracking branch 'upstream/main' into fix/atn-recording-any-document-10769`
- `b72d57b33 docs(atn): say which two cases the missing-layer pin's document covers`
- `6308e802c fix(atn): widen the ask's name match, narrow it to the failed step`
- `bced4247e fix(atn): ask Continue or Stop when a played step names a missing layer`
- `515733b12 Tests: the queued-starts wait matches the session clone count exactly`
- `3878d8fd5 Tests: the batch check waits until the cancelled batch has returned`
- `19a42910c Tests: wait for the other thread's state instead of a fixed sleep (#15379)`
- `ca25ec261 Merge origin/main into fix/summarize-modifier-order-14882`
- `dafd55564 Print: the printer's paper line follows Units; typed lengths at 72 pt/in`
- `654295da1 Print: Units lists Inches, Centimeters, Millimeters, Points and Picas (#14136)`
- `3efed5840 New Document: an empty Recent tab lists only Default PhotoCraft Size (#14038)`
- `33b8ade49 Merge main into fix/new-doc-recent-items-14038`
- `c2ef3cbf8 Merge main into fix/new-doc-recent-items-14038`
- `6228b5cf4 New Document: Recent tab lists Your Recent Items, 4 per row, duplicates kept (#14038)`
- `e3743d0e1 Summarize: order modifiers Alt, Shift, Ctrl off macOS (#14882)`
- `55e9b409b ui: Count tool layer menu greys Blending Options, Color Range and Load Selection with several layers selected (#12358)`
- `f502e675a Merge remote-tracking branch 'upstream/main' into fix/12358-canvas-layer-menu-plural-labels`
- `8947ad041 ui: canvas layer menu reads Duplicate/Delete/Rasterize Layers with several layers selected (#12358)`

## What was measured

- `cargo nextest run --workspace --no-fail-fast --test-threads 4`: **15,460 passed / 0 failed / 96 skipped**, exit 0 (1,795 s; one GPU timing trial passed on its retry).
- `cargo xtask test-corpus`: 670 passed / 0 failed / 11 ignored, exit 0.
- `cargo check --workspace --all-targets --keep-going`: exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0.
- `cargo fmt -- --check`: exit 0.
- `cargo xtask layers`: OK, 30 crates, no layering violations.
- `cargo xtask panel-commands`: OK, every click handler routes through a command id.
- `cargo xtask wasm`: ok, 22 crates.
- `cargo xtask parity`: exit 0.
- `cargo test -p photocraft-engine --test panic_hunt -- --ignored`: 1 passed.
- `cargo build --release -p photocraft`: exit 0 (66 min); the executable starts and shows its window.

## What is not proven

- The macOS and Linux legs were not run; this is a Windows machine.
- The corpus is fetched and verified on this machine; the archive layout behind the fetch is not re-checked.
- The GPU timing trial measures the wall clock, so its result depends on the load of the machine; the fork's configuration runs it one at a time.
- The release executable was started and its window shown; the behaviour of the interface itself was not judged by eye against the reference application.
- Upstream's own CI was not reproduced.

