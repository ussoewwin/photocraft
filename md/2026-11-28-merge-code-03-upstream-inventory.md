# Upstream inventory and verification

Two ranges: `craft-libs` `a7ff12f4d..51fcc3b04` and `photocraft` `904374406..8d7c1f833`.

| range | commits | of which non-merge | listed below |
|---|---|---|---|
| craft-libs | 328 | 198 | 198 |
| photocraft | 207 | 123 | 123 |

The merge commits themselves are counted, not listed: `git log --oneline --no-merges` lists change
commits only, and every count here is measured with the command beside it.

## craft-libs: 328 commits over `a7ff12f4d`, 198 non-merge

```text
120dd52dd ui-contract: the hand-forward trial's cell id derives from its parent's scope id
670487588 ui-contract: the hand-forward trial's cell id derives from its parent (Id::new is deprecated)
6cfc7513a ui-contract: a memoized view's sizing replay hands forward again what its sizing run handed forward
e4084d666 ui-contract: a memoized view keeps its sizing measures per layout context, and a taken-back draft keeps the ones it found
edd8d1266 LayersOutput::requests_taken: the panel's real run reports its one-time requests applied
7dff06d72 proof: Indic pack patterns reach words with joiners
588110ab5 ui-contract: a lazy child meets the view by its shown extent, whichever kind of run decides
70286ac3e proof: one copy of the script checks for Gujarati and Hindi; Hindi akshara hyphenation
b42509ee6 proof: keep zero-width joiners inside Indic conjuncts
076b3e337 Trials follow the replay and measure rules: a story's open dialog replays, a panel edit measures afresh
3123577f6 proof: indic docs link no private item
a77142912 proof: no doc link from the public Hindi grammar module to a private item
7d9544226 Clippy over the merged branch: needless borrows, a copied DockCommands, named tuple types
a6c4f46bf font-manager: one revision read per glyphs pass
401d95a5a proof: the mixed-spans proptest draws every language
4ecc5e3ed proof: rustfmt the indic module
5b5de9fc1 proof: rustfmt the Hindi module
5b571e525 Write history on for app-made links and cut settings merge back to w: compat cases
36b53c4bc proof: Hindi (hi, hi-IN) spelling, suggestions and grammar
d14a23d33 proof: hyphenate Gujarati between aksharas
323db9809 font-manager: an open glyph menu lets go of a glyph id when the source replaces its faces
c2d8c9e2e font-manager: an open glyph menu lets go of a glyph id when the source changes
b8bda06c2 Pin egui at storytold/egui#65's merge commit 932a73b2 (photocraft#17148)
1d487be7e fix(draw): preserve Alt when ending Space translation
93ae5a874 style: format Bezier Space regression
079da658b Keep vendor markup on deduplicated switches and drop bare compat blocks and all relationship attributes
0f525a505 test(update): cover the signed-catalog source and signature failure paths
a885cd5f3 ui-contract: the pointer-move gate compares each root's hover and press winners
338882944 Bound inherited drawing targets without changing legacy mark text
9b4e1134d platform: the inherit-only test checks the Users entry apart from the runner's
2573a9193 font-manager: an open glyph menu lets go of its cell when the face changes
2476cd834 proof: the mixed-spans proptest draws every language again
b7e66acd5 Format the paragraph-mark picture-link regression
1eb6a92ad Clear the design-lint findings on main
72ddff0c1 Revert "Merge pull request #3738 from dotCooCoo/fix/selectable-text-click-response"
b0370f229 Revert "Merge pull request #3855 from dotCooCoo/fix/web-export-web-shift-state"
33fb95c71 fcproj-format: schema v19 for FilmCraft Script items (#3782)
49cfba3fa Preserve paragraph-mark links on single Flow drawings
df8ff1496 A radio group names Tab's arrival from the members its run registers (FW-365)
e187b3e8c Replay audit skips a pass with a pending layout discard; a radio group's first run asks for its arrival pass (FW-365)
7f4311f96 A memoized view's key is its content's version: bodies inside measure afresh in a run with a new key (FW-365)
86bb64075 fix(zip): index virtual names from the central directory and require UTF-8
6e11c7efd ui-shell trial: the status slot's hover fade ends before its version changes (FW-365 F3)
076c0b38f ui-shell test: holds takes the shared command list (follows 8401672c4c)
518ea0a97 A bar draws its rules at the close of its line (FW-365 integration item 3)
b1e2db503 A body's measure keys on the view room its rows shrink to (FW-365 integration item 3)
9793060fe fix(draw): retain translated Bezier handles across Space transitions
533615890 craft-palette: a chunk after data that runs past the end is refused
0ff22c950 A replay compares what pointer sensing gives the view's own components (FW-365 integration item 2)
6e13c41e8 The app's windows are the ones the framework showed this frame (FW-365 integration item 1)
c417cd305 feat(ui-shell): shell slots take the host's version and replay as memoized views (FW-365 F3)
cc9612fa2 fix(ui-contract): a memoized view kept with AccessKit off runs once AccessKit is on
b402ab745 The tool options' types are PlainData (FW-365 F6b)
408de2e17 test(update): register signed catalog unit tests with #[trial]
b3663dae2 feat(update): default update checks to signed getartcraft.com catalog with GitHub fallback
14317bbe7 fix(ui-panels, ui-patterns): a split area's body lays out at the frames of its own run
7b1b63664 feat(ui-contract): pointer::press_state, a press-state read that a pointer move leaves alone
eda8a4073 fix(ui-patterns): the dock reports what it drew from the real run only
1cf70ccfa fix(ui-contract): a layer of known size starts its Area at that size
432c21889 fix(ui-contract): a memoized view in a row sits on the line from its first pass
c07dc92f2 ui-contract: collection rows name the row under the pointer through hover-only parts
8f6558607 ui-contract: an unwrapped row reports its line in every alignment
fdfcecf20 ui-contract: the open-record count is the thread's own
4fe31f5dc ui-contract: a settled view under the pointer replays while what sensing gives it is unchanged
27e8a7714 ui-contract: one tracked hover-position read; cursor requests and pointer facts journaled
84be0cbf2 perf(ui-patterns): a settled dock replays its columns and its docked tree
a9adb36c6 ui-contract: a run reads its place from the env its parent installed
830521eb0 Match DOCX settings reader semantics and retain disabled switch extensions
be002ddcd fix(opc): keep legacy leading-slash part names readable under zip hardening
7fa53a4c2 test(proof): restore Gujarati NOSUGGEST collector coverage
c73a94fa2 StatusReadout binds the zoom (f64) and reports zoom edits; the view stays the host's (FW-365 F6)
8401672c4 perf(ui-contract): menu titles share their held commands, and facts are read in place
090a789c3 perf(ui-contract): shown facts share their command, and facts keep their buffers across passes
66391f53e ui-contract: a run reaches its pass state and its pass facts through one binding
de45e61fa craft: egui_fork::REV at the pinned egui (storytold/egui#65's head)
cbcc2ffbd ui-contract: a repaint request's cause names its caller
bd2e08423 build: egui at storytold/egui#65's head (move-only passes ask the gate)
b093185a3 perf(ui-contract): a pointer move that changes nothing under the pointer runs no pass (photocraft#17148)
e7f4e17fa WIP: perf(ui-contract): a pointer move that changes nothing runs no frame (photocraft#17148)
56b86f192 Assert macOS detach fixture setup before using its directory
4a1eb8f5c Format the empty linked group boundary regression
d49088d6c Split out of #3358: this PR keeps the editor focus across app switches only; hit-area bounds and the Rating colour/size move to PRs of their own
07a01d257 Format the current contribution source
925e79ebb Use fixed-size row chunks in the predictor overhead fixture
8088ba0bb Identify unsupported DeviceN fixture routing in the test assertion
d0139810f Format the serialized-overhead fixture
036612061 Strengthen PDF row-prediction review regressions
011956be9 Share continuation flag resolution within the affix modules
f03e10310 Format the retained JBIG2 boundary assertion
edeb9f6bf Keep the ZIP16 tail with a smaller EXR fixture
e48954d56 Separate NaN-aware effects check from remaining clip state equality
89793592a Compare automatic motion effects consistently with round-trip controls
0f05cc091 Format split source-selection control
ef9a0ee6d Verify split FCPXML source selections without inventing internal links
dcfcca989 Keep the production constructor limit in the full node-budget test
f47d41114 Apply package formatting to FCPXML review repairs
a73767ac6 Fix deferred audio replay diagnostics, pairing and test registration
e40155d42 draw: PaintBar's Opacity and Flow hold 1 to 100 %, so a typed 0 reports the range alert (photocraft#16809)
5ce4e3251 Curve graph: a right-click deselects on the empty graph and removes a point again
2cae30e5b Curves point keys: the graph applies its claimed keys in the order they were pressed
66c1d5fed craft-psd: tbr::write refuses a preset tbr::read would refuse
a3a06160a font-manager: an open glyph menu keeps the cell that was clicked
316382bc0 craft-palette: a RIFF palette entry flagged PC_EXPLICIT is refused
37031ecda font-manager: a right-click on a glyph cell names that cell
95f922dbb Editor trial: pin the window focus the pass reports, so a has_focus regression fails the switching-apps trial
f9b1c6014 Small additions review: a replay moves a hit area's bounds with its rect
87fb1818d craft-palette: read RIFF palettes (.pal) (photocraft#14372)
7e906e7df craft-psd: read and write toolbar presets (.tbr)
18d3d383c style(docx): format settings_merge.rs
c0024b06a fix(docx): preserve compat block namespaces and handle whitespace in on-off (#3308, #3305)
7f16b4221 fix(docx): address CodeRabbit review feedback on settings merge and ruby links
a12bea6ad fix(docx): preserve absent compatibilityMode and hyperlink history (#3308, #3305)
097c0bb15 ui-contract, ui-widgets: TextField::color_ranges colours stretches of a field's text in theme inks
8c688fa68 rustfmt
123716adc Rating: an app colour for the lit stars, and a star size
87ab3c740 Editors: switching to another app is no focus move
78cd3e98a Inspector: a hit area's original bounds before minimum-size expansion
b1c553c20 fix(sec): filter traversal components in names iterator and check has_traversal_component in penpot
177c15b4c fix(sec): sanitize zip entry paths and reject traversal vectors
876b934e0 latex: check deterministic work for repeated labels
f58690eb2 craft-zip: check raw entry-name bytes in has_traversal_component
8c5c90eba craft-zip: reject zip-slip path traversal vectors during entry enumeration
f5c5668e6 Layer Style: a Satin press without travel leaves a sparse instance's parameters alone
eb40a69e3 test(proof): replace wall-clock timing assertion with deterministic bounded suggestions check
ec7b3877c docs(audio): clarify other Opus families retain stream order
437e51576 Keep the fixed-window move trial outside docking behavior
b631ecb83 Assert floating window movement after the drag is committed
7998d30a7 test(docx): build attribute limit XML in one buffer
6c5fbd789 Avoid repeated symbol chains in the Fig depth test
6f6c889ec cms: use smaller grids for the Oklab table test
00c76d129 Separate encounter-order fixture assertions for strict lint
d8c444c45 Use comparable explicit coordinates in the budget fixture
b2b23ac01 Plan and budget deferred audio placement before mutation
6787d6429 Cover replay-budget refusal before parent mutation
421b9be73 Remove the superseded deferred audio preference field
9f11a55fe Preserve audio encounter-order lanes after deferred recovery
adb134031 Cover deferred and ordinary audio encounter-order placement
2893e322e Keep connected audio controls within the XML nesting limit
018ea0a6f fix(timeline): restore deferred nested FCPXML audio
240080c5a test(timeline): expose deferred audio after nested rebuilds
adab8bcbd test(flow): use the inline object marker from para
a0d811902 fix(docx): close conversion budgets and deepest group lifecycle
4a1dcba53 fix(docx): bound drawing links and preserve nested group content
e2686340d fix(docx): resolve hyperlinks from grouped VML templates
8c10bdb37 refactor(docx): separate hyperlink fixture geometry checks
23b2328fe fix(docx): import hyperlink relationship constants
6a46b4444 fix(docx): preserve object and nested group hyperlinks
0af55e7f9 test: reproduce VML object hyperlink loss with original input
6438ae78d Exercise momentary Hand routing in the Bezier interaction tests
c5e325d83 Format the Bezier Space interaction regressions
d86f715fe Let Space move new Bezier anchors during handle drags
3e22efd35 test: group feature-sensitive image and text cases
4216f97b7 svg: normalize CSS value identifier casing
7128c0fde svg: cover CSS value identifier casing
b08e77e71 test(ps): reuse number samples and retain clamp boundary
eca8c55c3 exr: trim oversized estimate test fixtures
47316b0d8 pdf: predict sampled image rows when smaller
a9ca8e42d Format the streamed PNG fixture construction
a2e0d965e Keep distinct row runs in the streamed PNG fixture
910500386 pdf: add sampled image prediction regression
9d7714dea Keep only independent streamed and large PNG fixture reductions
f7797e32b New Document: Your Recent Items keeps four cards a row on next's card grid
2b42b6591 image-codecs: trim redundant PNG encoder test passes
c29d4b748 Proofing: handle Gujarati digits in grammar/spelling and check pack patterns fit
42958efaa Proofing: add Gujarati (gu_IN) spelling, suggestion dispatch and grammar checks
e85c1d64c cms: format measurement transform pairs
de3374b2a cms: reuse measurement transforms in colour tests
f5a0a1458 proof: reuse alias keys and final continuation set IDs
33702a095 proof: compare aliased and literal continuation output
4a691d889 proof: format continuation alias compatibility checks
c6a6a1908 proof: share continuation alias parsing during setup
f6efa4760 proof: format continuation alias compatibility checks
610b60199 proof: cover continuation alias compatibility
a6d8a43d0 Curves point keys: the trials move to tests/it/curve_point_keys.rs
eccaa1593 doc: reduce the Zstd tile limit test fixture
a5ddcff18 latex: reduce linear-label timing fixture inputs
c63da6685 proto: remove obsolete command-limit import
c65361ba4 proto: exercise IMAP budgets with small test limits
b47954762 style(docx): format Flat OPC cap fixture
db1030835 docx: exercise Flat OPC element limits with small inputs
7bbf32add style(stream-codecs): format small-cap controls
6cde42277 stream-codecs: test symbol caps with small inputs
999c8df18 style(prepress): apply crate formatting
ec17698c1 perf(prepress): exercise plate caps with small inputs
27379f911 Reduce DrawingML chart fixture cloning
759cf08a7 perf(xml): shrink the aggregate-attribute test fixture
568824405 perf(pdf): shrink the node-budget test fixture
c804f37c6 Test PDF decoded limits with small Flate streams
e090330e5 Layer Style: a drag on the image moves the Satin
18e5188fb Let floating panels opt out of edge resizing
65dccc7f0 Curves: optional Ctrl+Tab point stepping, and Delete on an end point deselects
2be116237 New Document: optional Your Recent Items layout for the Recent tab
b78899df5 New Document: Photo, Print and Art & Illustration list the blank document presets
c430dff1b Shapes: the capped triangle test meets halfway along the slanted edges; allow field reassignment on the non-exhaustive settings
d1d1667c9 Shapes: import ParamCurve for the triangle arc test
e4c1dc599 Shapes: the Triangle tool's own corner radius in the options bar, its drag and Create Triangle
f97614d97 craft-audio: README says Opus family 0 is mono or stereo
b84bd0613 craft-audio: README says opus::wav_order is for mapping family 1 only
```

## photocraft: 207 commits over `904374406`, 123 non-merge

```text
86931c85e Pin craft-libs at the merge of craft-libs#3917 (requests taken, per-context sizing measures)
44542cad4 fix(ui): Window › Arrange layouts keep each document's zoom (#16885)
a2502b2ab Test harnesses run the document area's per-pass work before the area, as the window's body does
6573a2eb8 Patch tool: canvas right-click menu inside and outside the selection (#12122)
ead3cd487 Layers panel requests are cleared only once the panel reports them taken
d2a35bf20 Color Balance tone kept by the Properties host from the editor's output; Remove tests wait on the job itself
142921778 Show the not-allowed cursor on blocked painting targets (#16815)
7d92b709f Document area and dock versions cover what changes with no input; per-pass canvas work runs outside the dock
50c872106 Pin craft-libs at storytold/craft-libs#3801's merge commit c7c404fe (#17148)
ea1676263 Artboards to PDF ZIP encoding tests: paint the boards' layers directly, since edit.fill moved to the paint crate
82243345b Puppet Warp pin menu: test that Enter picks the row, not the commit (#16734)
b54d5f2fe Re-run CI (unrelated Remove tool background-job timing failure)
5e6f5252c Test Artboards to PDF ZIP encoding
a33c8ef08 Artboards to PDF: ZIP encoding
35b60e934 build: pin craft-libs at storytold/craft-libs#3900's head 7d9544226d
ebc40ce3c Clippy: a named type for the preview request, needless borrows of the context
e7767a86c build: pin craft-libs at storytold/craft-libs#3900's head 47ba4dfd38
611a386f7 Puppet Warp: right-click pin menus (#16734)
1a835fe96 Layers: a second click on the targeted vector mask un-targets it (#16783)
0f42496ea Move the winit pin to the merged Alt-code fix (#10863)
b88866441 ci: notify artcraft-services on release state changes (#17154)
be49e0834 Pin egui at storytold/egui#65's merge commit and craft-libs at storytold/craft-libs#3801's head (#17148)
591ef2eca Star tool: sample the foreground and Fill with Alt (#16875)
8303d38c6 fix: preserve unsaved Duplicate Image copies and their initial history (#14680)
da571544e Drop channel view imports main no longer uses
350c2638c Layer Style: remember Outer Glow settings for the session (#14945)
f0c11b4c1 Red Eye test: build the session with crate::session() after the engine split
c18f5102b test(ui): pin the child bytes behind the bucket group alerts (#12537)
0a031b939 fix(ui): the Paint Bucket's layer refusals open alerts (#12537)
b187c73c7 test(ui): expect plain unrecognized alerts for Smart Object opens
fd3afcbfc test(ui): assert modal alerts for failed window drops
7127634bc test(ui): target maintained recent-file accessibility names
c25832842 SampleAllLayers::get takes self without mut (FW-365 F6b warning)
21afd93eb Title, options and status bars as the shell's versioned slots (FW-365 F3)
afbb82c62 Separate memory-approval regression scenarios from their assertions
8a216ae14 Use current control replies in unreadable-file regressions
ab20a4d05 text build script reads the manifest dir at run time, so worktrees sharing a target dir find their own fonts
dd80e7099 Canvas: press-state reads that a pointer move leaves alone (FW-365 F4)
d29315ea7 Drain per-file alert acknowledgements by instance-aware dialog type
56bf471c4 Options-bar and tool writes bump the tool options' revision only when they change a value (FW-365 F6b)
da8339acf Document area: a memoized view keyed by the canvas version, with its per-pass work before it (FW-365 F4a)
2b336ac9e Format unreadable-file alert integration
02c84a6f9 Tool options carry a revision: the options bar's version reads it, serialized() is gone (FW-365 F6b)
9de809297 perf(ui): the dock keys its columns by the panels' versions
7c937ffa4 Panel layout and Color panel options are remembered without moving the settings revision (FW-365 F6)
a5e476b93 Status bar binds the zoom alone; what the app remembers in preferences moves no region's version (FW-365 F6)
bd89c1147 Options bar keyed by the view fields its tool shows; status bar keyed by zoom alone (FW-365 F6)
a438a9667 ﻿test(retouch,ui): pin the Replace rim, widen its exact disc, and test the dropdown pick (#12101)
b8304c6d5 build: craft-libs at storytold/craft-libs#3801's head (egui_fork::REV matches the egui pin)
951cfadee perf(ui): a pointer move that changes nothing under the pointer runs no frame (#17148)
1f486fbc1 style: rustfmt on the new #12101 test files
5cb23ca42 feat(ui): the Healing Brush Mode menus gain Replace and the reference's rules (#12101)
de75f4764 feat(retouch): paint.healingBrush accepts mode "replace" as a straight source copy (#12101)
752d83c07 test(retouch): the Healing Brush Replace stroke copies its source exactly (#12101)
9f81f17b2 Use document layer lookup in the cache regression
d2dec04f0 Invalidate group effect maps when a child changes type
86d3f11b2 Alt codes type their character on Windows (#10863)
b59300ba2 Window > Arrange: match measured order and active document label (#16888)
20d5bd0ea Pasteboard menu: Hide/Show Contextual Task Bar heads the Move tool's menu (#16757)
40491cd8d Actions: the Custom-area message of an empty Content-Aware Fill step is translatable (#14254)
9e8827aa4 Actions: an empty Content-Aware Fill step after a Custom fill stops and says why (#14254)
13a3c3aca Leave the profile out of an Indexed PSD when Include ICC Profile is off
2a80a6c1f Apply Image: send a mask alpha named "none" or with no name by its index
807de92bb Apply Image: keep mask alphas and layers named like built-ins apart
6acc98ee7 Give the Layers to Files ICC tests a normal layer
8f73a6432 Keep a document's profile for its colours when Include ICC Profile is off
7965a12a5 Add Include ICC Profile to Layers to Files
ce3e49718 Apply Image: list mask sources in opening order after tabs move
26ee2cafc Apply Image: Mask checkbox opens its Source, Layer, Channel and Invert
1ce53d594 Content-Aware Fill action tests: paint the square instead of Edit > Fill (#14254)
06e8b35ae Content-Aware Fill: carry an earlier version's workspace settings to empty recorded steps (#14254)
b5c7f1f2b Content-Aware Fill action tests: build the session with crate::session() (#14254)
88b6efba2 Burn: test light above white when a 32-bit colour's luma isn't positive
32ae912c3 Dodge: keep light above white when a 32-bit colour's luma isn't positive
67b724031 Artboard from Group: a hidden empty group keeps the canvas beside a board (#14558)
f2789277f Burn: test today's curve at unmeasured 32-bit exposures
d355ed831 Dodge: keep today's curve at unmeasured 32-bit exposures
0b4a20f88 Restore the channel view only when undo undoes the channel step
c64cab2e1 Burn: tone a 32-bit document's linear values with the reference's curve
4dd105c42 Artboard from Layers: clamp Width and Height to 4..32000 px
d35d05dac Dodge: tone a 32-bit document's linear values with the reference's curve
6944b4619 Undo of New Channel and Calculations restores the channel target
fdfb0f1e7 Auto-Align: space the projection columns apart
bf4fca723 Paint Bucket: alert when a typed Tolerance fraction is rounded
800701d7b Give Auto-Align Layers the reference dialog layout
750a67a96 Paint Bucket: whole-number Opacity and Tolerance with alerts, tooltips, no Fill label
1342fb73b Start a new Gradient Overlay at grey 215 to white, Smooth
c018f0a9f Make a Red Eye press outside the selection silent
f858820da Tests: the Custom Shape tool's Make row in Path mode
b846e831f Duplicate Layer: groups keep their naming; a typed As: name survives destination changes (CodeRabbit on #16090)
f9fda28c3 Duplicate Layer: propose the layer's own name for another or a new document
98b08b013 Artboard from Group: an empty group goes right of the topmost board (#14558)
8d9a5dd40 Content-Aware Fill: refuse to save a native step without sampleAllLayers (#14254)
b7fc38d22 Content-Aware Fill: keep the settings an empty recorded step reads across a restart (#14254)
ec37ad574 Paint solid Normal inside strokes without a temporary buffer
174d230ae Check stroke alpha over a transparent canvas
5299ae0fa Cover translucent raster interiors with Inside and Center strokes
b2cefc400 Content-Aware Fill: a background fill remembers its settings when its job is applied (#14254)
fb5842785 Content-Aware Fill test: lock the workspace's own layer for the refused fill
f398e8798 Content-Aware Fill: a refused fill keeps the last remembered settings (#14254)
811eddf4e test: apply bounded CI formatting repair for PR15372
ee4cfc0a0 Format open fixture source argument calls
94219e012 Update open completion fixtures for optional source metadata
eb76800ac Format byte-template memory prompt source retention
d8c1efec8 Preserve byte-template source through memory approval
edc838a57 Format the complete PSD approval fixture
85e5ec2b9 Correct PSD alert test oracle and approved-open header fixture
9db3e586d Format template and memory approval alert regressions
3e768fe6e Preserve unreadable alert behavior for templates and approved opens
ddb59bf5c Folder Batch: an empty Content-Aware Fill step uses the remembered settings
16d9a0dc1 Preserve read sources in unreadable-file alerts
c10e7f277 Update Open failure tests for modal unreadable-file alerts
13049cf36 Keep Smudge pressure control logic within the options bar complexity limit
065abdffc Configure real imports and fonts in unreadable-alert tests
29350152b Add Smudge pressure-for-size control
6352916c3 Show unreadable-file failures in modal alerts
abb5c2371 Actions: load every recorded Content-Aware Fill value and empty steps (#14254)
7cb7d48c6 fix(paint): premultiplied mix weights alpha separately and keeps colour at tiny alpha
d003380fe fix(paint): premultiplied mix keeps the source exactly at full weight
1680566db Refuse Group from Layers when the Background is selected (#14709)
4c586aba1 fix(paint): History Brush restores transparency instead of painting over (#14701)
838c7260d ui-egui: run the bucket pasteboard gesture before the rasterize guard (#12534)
22467aeb3 ui-egui: Paint Bucket Shift+click on the pasteboard sets its colour (#12534)
```

## Measured, this round

- craft-libs: `cargo check --workspace --all-targets` 0 errors; the fork's twelve crates **3,447
  passed**; the workspace **19,885 passed / 66 skipped / 0 failed** (1,084.5 s); `cargo fmt --check`
  0; `cargo clippy --workspace --all-targets -- -D warnings` 0; the design lint **28 passed**.
- photocraft: `cargo check --workspace --all-targets` 0 errors; `cargo xtask ci lint` **9 of 9
  steps**; `cargo xtask parity` 0 with no change to `docs/parity-checklist.md`; the panic hunt 1
  passed; the workspace **16,132 passed / 96 skipped / 0 failed** (1,329.2 s); `cargo xtask
  test-corpus` **744** and **670** passed with 0 failed (0.28 GB of corpora fetched and verified).
- The release executable is 333,945,856 bytes and shows its `PhotoCraft` window; the CLI is
  136,600,576 bytes; the PDBs are 23,212,032 and 11,120,640 bytes.

## Not proven

- The suite runs happened on the working trees the two merge trees record, before the changelog and
  documentation commits that follow this record; those commits touch `CHANGELOG.md`, `md/` and nothing
  under `crates/` or `apps/`.
- Upstream's own trials are upstream's evidence; this round ran the fork's suites, not their CI.
- The corpus fetch needed one repair of a cached archive (a member the platform tar refuses); the
  files themselves verify against `xtask/psd-corpus.sha256`, 170 of 170.
