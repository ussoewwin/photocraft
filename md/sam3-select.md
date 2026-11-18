# SAM Selection (SAM 3.1): technical notes

This document describes the whole implementation of the SAM Selection tool (`Tool::Sam3Select`, command `select.sam3`). It runs SAM 3.1 Multiplex open-vocabulary detection and interactive segmentation directly in **pure Rust + CUDA (`candle-core` / `candle-nn`)**. Neither ComfyUI nor Python is used at all at run time. ComfyUI's SAM 3.1 implementation serves only as a numerical reference (a development-time oracle), and `plugins/sam3/data/*_reference.py` is a copy of it (clean room).

- Key commits: `395be2eff` (adds the SAM Selection tool), `a4d67a60c` (restores the Japanese labels inside the tree)
- Key files: `plugins/sam3/src/` (inference helper `sam3-select`), `crates/engine/src/cmd/sam3_select_cmds.rs` (command `select.sam3`), `crates/ui-egui/src/tools/retouch_ui.rs` (options bar and gestures)
- Source of the plan and the verification: `plugins/sam3/PLAN.md` (Revision 5, including the measured records). This document is a summary brought in line with the current main

---

## 1. Overview

- **Model**: `sam3.1_multiplex_hswq_convrot_int8.safetensors` (1,738,165,347 bytes = 1.74 GB). An HSWQ ConvRot INT8 pack of the SAM 3.1 Multiplex checkpoint (`image_model = "SAM31"`, `multiplex = True`). With **no ONNX conversion** it memory-maps the `safetensors` and runs directly on the GPU.
- **Inference backend**: `candle-core` / `candle-nn` 0.11 (`features = ["cuda"]`). It links directly against cublas / curand / cuda_runtime; no cuDNN or onnxruntime DLL is needed.
- **Prompts**: text (comma-separated as in `face:3, person`, with `category:N` setting the detection limit), boxes, points (one point). Two stages: detect → refine.
- **Output**: `[N, H, W]` binary masks → composited as a document-space selection (`doc.selection`).
- **Resident**: in `--serve` mode it keeps the pack (1.74 GB, including the ConvRot inverse rotation) in VRAM and caches image frames too. The second selection onward takes under a second.

## 2. Components and data flow

```text
UI (SAM Selection Tool in the W slot)
  └─ gesture: click → point / drag → box / Detect button → prompt
       └─ engine command select.sam3        crates/engine/src/cmd/sam3_select_cmds.rs:606
            ├─ prompt_from(): document coordinates → 1008 grid (:223)
            ├─ sampling: the target layer (or the sampleAllLayers composite)
            │   to a temporary file as interleaved 8bit RGB (run_helper, :457)
            ├─ resident::apply() → sam3-select --serve (:427)
            │     └─ when unavailable, the one-shot CLI (--verify-weights + --select-*)
            ├─ reads masks.bin (f32 LE) + masks.json (shapes/coverage/boxes/scores)
            └─ HelperMask (trimmed to document space) → Region → combine_region (:163, :568)
```

Because a wasm build cannot launch the helper, the resident path is closed off with `#[cfg(not(target_arch = "wasm32"))]`, and the feature — the one-shot path included — is desktop-only.

## 3. Build configuration

`plugins/sam3/Cargo.toml` (a standalone crate, with an empty `[workspace]` table):

```toml
candle-core = { version = "0.11", features = ["cuda"] }
candle-nn   = { version = "0.11", features = ["cuda"] }
memmap2 = "0.9"; safetensors = "0.8"; half = "2"
image = { version = "0.25", default-features = false, features = ["png", "jpeg"] }
regex = "1"   # CLIP BPE pre-tokenization
```

The release profile is `opt-level = 3`, `lto = "thin"`, `codegen-units = 1`. Under MSVC (`vcvars64.bat`) it links directly against the local CUDA toolkit and produces `plugins/sam3/target/release/sam3-select.exe`.

## 4. The model: HSWQ ConvRot INT8 pack

Measured breakdown of `plugins/sam3/sam3.1_multiplex_hswq_convrot_int8.safetensors` (1,672 tensors in total):

| Kind | Count | Contents |
|---|---|---|
| FP16 | 1,549 | The 32-block ViTDet trunk, the 3-level FPN neck (convs / interactive_convs / propagation_convs), the 24-layer CLIP text encoder, the whole SAM31Tracker |
| INT8 `weight` | 41 | All are 2D `nn.Linear` inside `detector.*` (resizer, dot_prod_scoring, geometry_encoder, segmentation_head.out_proj, decoder/encoder) |
| FP32 `weight_scale` | 41 | The row scale of shape `[out, 1]` (`amax / 127.0`) |
| U8 `comfy_quant` | 41 | A 67-byte JSON `{"format":"int8_tensorwise","convrot":true,"convrot_groupsize":256}`. Zero layers use groupsize 64 and zero Conv2d layers are quantized |

**The ConvRot inverse rotation is deterministic** (`GROUP_SIZE = 256` at `loader.rs:29`): the rotation matrix is H₂₅₆, the 4-fold Kronecker power of the normalized regular Hadamard matrix H₄ (a 4×4 differing only in sign placement) divided by √256, and since H₂₅₆ᵀ = H₂₅₆ and H₂₅₆² = I hold, restoring it at load time takes a single matrix multiply on the GPU:

1. `deq = f32(q) × weight_scale`
2. `W = f16(reshape(deq.view(O, I/256, 256) · H₂₅₆))`

`build_hadamard()` (`loader.rs:450`) builds H₂₅₆ on the GPU, and `Pack::load` (`loader.rs:116`) dequantizes all 41 layers in one go and uploads them as f16. Zero Conv2d layers are quantized, so every convolution runs in the original FP16.

## 5. Key remaps (the four required at load time)

The checkpoint is stored with upstream's key layout, and `remap_name()` (`loader.rs:94`) plus the loader's splitting perform four kinds of remap:

1. **`tracker.model.*` → `tracker.*`**: the file has 457 `tracker.model.*` keys and zero bare `tracker.*` keys. Strip the `.model` segment.
2. **Splitting the CLIP fused QKV** (24 modules, 48 tensors): split `encoder.transformer.resblocks.{0..23}.attn.in_proj_weight` (`[3072, 1024]`) and bias (`[3072]`) into three along dim0, into `.q_proj` / `.k_proj` / `.v_proj` (`[1024, 1024]` and `[1024]`).
3. **Splitting the detector `SplitMHA` fused QKV** (37 modules, 74 tensors): `geometry_encoder.encode.{0..2}.{self_attn, cross_attn_image}` (6), `segmentation_head.cross_attend_prompt` (1), `transformer.decoder.layers.{0..5}.{self_attn, cross_attn, ca_text}` (18), `transformer.encoder.layers.{0..5}.{self_attn, cross_attn_image}` (12). `[768, 256]` → 3 × `[256, 256]`.
4. **Renames in the tracker `TwoWayTransformer`**: for keys containing `sam_mask_decoder.transformer.*`, `.mlp.lin1.` → `.mlp.0.`, `.mlp.lin2.` → `.mlp.2.`, `.norm_final_attn.` → `.norm_final.`.

## 6. Model structure (1008×1008 input)

**Preprocessing**: normalize RGB to `[0.0, 1.0]` and resample bilinearly (`align_corners=False`) to 1008×1008. **No ImageNet mean/std normalization is used** (`ViTDet.patch_embed` takes the image directly through `Conv2d(3, 1024, k=14, s=14, bias=False)`). The helper reorders the interleaved RGB handed over by the engine into CHW f32, resamples it bilinearly to 1008, then casts to f16 (`main.rs:844` `load_raw_rgb`, `main.rs:874` `load_image_1008`).

**ViTDet trunk** (`vitdet.rs`):

- `img_size=1008, patch_size=14, embed_dim=1024, depth=32, num_heads=16, mlp_ratio=4.625, window_size=24, global_att_blocks=(7,15,23,31), use_rope=True`.
- `patch_embed` → `[B, 1024, 72, 72]` → `72×72 = 5,184` tokens. **The cls token does not enter the trunk** (the stored `pos_embed` is `[1, 577, 1024]` = cls + 24×24. Tile the spatial grid 4×4 and crop it to 72×72, use only `pos[:, 1:]`, and discard `cls_pos`). Then `ln_pre`.
- Attention is a 24×24 window in 28 blocks (9 windows × 576 tokens), and global in blocks 7/15/23/31 (5,184 tokens).
- Positional encoding is the 2-axis RoPE of Flux's `EmbedND` (`rope_2d`, head_dim 64): `scale_pos = 24/72` for global, `scale_pos = 1.0` for windows, theta 10000.

**FPN neck (a SAM 3.1 multiplex-only specification)**: 3 scales `[4.0, 2.0, 1.0]`, `scalp = 0`. This differs from SAM 3 (multiplex=false) with its 4 scales `[4.0, 2.0, 1.0, 0.5]` and `sam2_convs`.

- scale 4.0 (288×288): `dconv_2x2_0` (1024→512, k2s2) → GELU → `dconv_2x2_1` (512→256, k2s2) → `conv_1x1` → `conv_3x3` (pad 1)
- scale 2.0 (144×144): `dconv_2x2` (1024→512, k2s2) → `conv_1x1` → `conv_3x3`
- scale 1.0 (72×72): identity → `conv_1x1` (1024→256) → `conv_3x3`

`convs.{0,1,2}` are the features for detection, `interactive_convs.{0,1,2}` those for the interactive tracker (288/144/72). `propagation_convs` are for video tracking and are not loaded for still-image selection.

## 7. Detection (DETR)

`detector.rs`:

- **CLIP text encoder** (`clip.rs` + `tokenizer.rs`): hidden 1024, 24 layers, 16 heads, `quick_gelu` (`x · sigmoid(1.702x)`), vocab 49,408, max_position 32. The special tokens are start 49406 / end 49407 / **pad 0 (`pad_with_end = false`)**. The attention_mask (1 for a valid token, 0 for the trailing pad) is passed both to CLIP's self-attention (together with the causal mask) and to the detector. `language_backbone.resizer` (1024→256) projects it for the detector.
- **Prompt syntax**: strip parentheses, then comma-separated categories. The `:N` of `face:3` sets `max_detections = max(1, round(N))` (default 1).
- **Positional encoding**: applies `PositionEmbeddingSine(num_pos_feats=256, normalize=True, scale=2π)` at the 72×72 level.
- **GeometryEncoder**: a point prompt goes through `points_direct_project` + bilinear `grid_sample` + `points_pool_project` + sine pos + `label_embed`. A box goes through `boxes_direct_project` + `roi_align(roi_size=7, aligned=False)` + `boxes_pool_project` (`Conv2d 256→256, k=7`). After the three pre-norm EncoderLayers, `_run_geo_layer` runs against `cls_embed` (note that geo uses `k = img + pos` and `v = img`), then `[text_embeddings, geo_prompts, geo_cls]` are concatenated and `text_mask` is extended.
- **TransformerEncoder**: 6 pre-norm layers (`self_attn` with `q = k = norm1(x) + pos, v = norm1(x)`, `cross_attn_image` attending to text_memory, ReLU FFN).
- **TransformerDecoder**: 6 post-norm layers over **201 tokens** (1 `presence_token` + 200 `query_embed`). The LayerNorm order is `norm2 → catext_norm → norm1 → norm3`. `query_pos = ref_point_head(sineembed(ref_points))` (512→256, with zeros prepended for presence). `_compute_box_rpb` passes the log-scaled coordinate difference `sign(8d)·log₂(|8d|+1)/log₂8` through `boxRPB_embed_x/y` (2→256→8) and builds a `[B, 8, 201, 5184]` cross_attn bias. After layers 0..4, iterative box refinement `ref_points = sigmoid(inv_sig(ref_points) + bbox_embed(norm(tgt)))`.
- **DotProductScoring**: `prompt_mlp` (a 2-block residual MLP + LayerNorm), masked mean pooling, `hs_proj` / `prompt_proj`, scale `1/√256`, clamp ±12, filtered by `scores.sigmoid() > threshold`.
- **SegmentationHead**: `cross_attend_prompt` (`cross_attn_norm(memory)` attending to the prompt) → `encoder_visual` onto `seg_features[2]` (72×72) → a three-stage top-down PixelDecoder (`Conv2d 3×3 pad1` + `GroupNorm(8,256)` + ReLU, nearest 2× upsample 72→144→288) → `instance_seg_head` (256→256, k1) → `MaskPredictor` (3-layer MLP `mask_embed` + `einsum("bqc,bchw->bqhw")`), producing coarse mask logits of `[B, 200, 288, 288]`.

## 8. Interactive segmentation (SAM31Tracker)

`tracker.rs`:

- `forward_segment`: builds `high_res = [288, 144]` and `backbone_feat = feat_72 + interactivity_no_mem_embed` (`[1,1,256]`) from `interactive_convs.{0,1,2}`.
- **Prompt encoder**: points (only those with a valid label get `point_embeddings`; when there is no box, one `not_a_point_embed` with `label = -1` is padded in), boxes (`point_embeddings[2]` and `[3]`), mask logits (bilinear `1008→288` with **`antialias=True`**, then to 72×72 through `mask_downscaling`'s stride-2 2×2 conv + `LayerNorm2d`; `no_mask_embed` when there is no mask). Positions come from `pe_layer`'s `positional_encoding_gaussian_matrix` (`[2, 128]`).
- **Mask decoder**: `SAMMaskDecoder` (single object, `num_mask_tokens = 4`). `SAMTwoWayTransformer` (depth 2) → `output_upscaling` + `conv_s1(feat_144)` + `conv_s0(feat_288)` up to 288×288 → `output_hypernetworks_mlps.{0..3}` → gated by `object_score_logits > 0` (`-1024.0` when negative). With `multimask_output = false` it takes mask index 0; with multimask and `0 < num_pts ≤ 1` it takes `argmax(ious)` over indices 1..3. Finally it interpolates bilinearly to 1008×1008.

**Two behaviours pinned down by measurement** (guarded by the unit tests in `nn.rs`):

1. **A token with `label = -1` zeroes its coordinate encoding before `not_a_point_embed` is added**. Adding it to the raw coordinate encoding shifted object_score from 5.26 → 0.92 (the mask-prompt case).
2. **The `1008→288` mask-prompt resize uses `antialias=True`**. The kernel is a separable triangle: `scale = in/out`, `support = max(scale, 1)`, `center = (o + 0.5)·scale − 0.5` (clamped at 0), normalizing the weight `1 − |i − center| / support` over the taps inside the input. `nn::resample_bilinear_antialias` matches `F.interpolate(antialias=True)` to 3.2e-07 across 10 size pairs. Without it object_score becomes 4.54.

## 9. Selection pipeline (refinement and mask assembly)

`select.rs` (`PAD_FRAC = 0.1`, `SIZE = 1008`, `refine_mask()` `:28`, `union_masks()` `:112`) and `selection.rs` (`DEFAULT_THRESHOLD = 0.5`, `DEFAULT_REFINE = 2`, `SelectionStack` `:24`):

1. Inflate each edge of the detection box by 10%, clamp to `[0..W, 0..H]`, and crop from the source image and the coarse mask.
2. Resize the crop to 1008×1008 and run **the trunk + `interactive_convs` only once per crop**.
3. Loop `interactive_sam_prompt_encoder` + `interactive_sam_mask_decoder` `refine_iterations` times (they are cheap). On the full-screen path for box/point prompts, run the trunk once, cache the GPU tensors and iterate only the decoder (`segment_prompt_cached` / `detect_and_refine_cached`, `selection.rs:90` / `:144`).
4. Combine the coarse and refined masks with `coarse | refined`. With `individual` set, emit each mask separately (`Selection::finish`).
5. Boxes are output in original pixel space (`orig_size` scales both the boxes and the coarse mask). `select.rs` makes the model dtype explicit, resizing the image crop in f32 before casting to f16 (the same order as the reference's `common_upscale(...).to(dtype)`).

## 10. Helper modes and CLI

All modes of `sam3-select.exe` (`usage()`, `main.rs:15`):

| Mode | Arguments | Purpose |
|---|---|---|
| Token inspection | `--tokens <text>` | Prints the 32 token IDs and the attention mask |
| Prompt parsing | `--parse <text>` | Prints the categories and the detection limits |
| Weight verification | `--verify-weights` | Opens the pack, dequantizes the ConvRot layers and reports (statistics JSON with `--dump-dequant <out>`) |
| Detection | `--detect-image <png> --detect-prompt <text> --detect-out <bin>` | DETR detection only |
| Segment | `--segment-image <png> --segment-box x1,y1,x2,y2 / --segment-point x,y / --segment-mask <f32> --segment-out <bin>` | Interactive segmentation (multimask enabled for points) |
| Select | `--select-prompt <text> / --select-box <4 values> / --select-point <2 values>` + `--select-raw <path> --select-raw-size <w>x<h>` + `--select-threshold / --select-refine / --select-individual / --select-out <path>` | The single entry point the engine uses. `--select-raw` is not a PNG but interleaved 8bit RGB (removing the PNG codec on both sides) |
| Resident | `--serve [--model <safetensors>] [--data <dir>]` | A JSON-lines worker on stdin/stdout |

Common: `--model <pack>` (defaults to the `PACK_FILENAME` next to the executable), `--data <dir>` (the directory holding `vocab.json` and `merges.txt`, default `<exe folder>/data`). For the device, `default_device()` (`loader.rs:483`) picks `Device::new_cuda(0)` and falls back to CPU when there is no GPU.

**Output format**: `masks.bin` is the concatenation of the per-mask f32 (LE), and `masks.json` is `{"masks": [shapes...], "coverage": [...], "boxes": [...], "scores": [...], "threshold", "refine_iterations", "individual"}`. The engine reads the shapes and slices the f32 stream into `[H, W]` blocks.

## 11. Resident worker (--serve)

- **Request** (`ServeRequest`, `main.rs:916`): `{"warmup":true}` (responds immediately without doing anything) or `{"raw":"<path>","width":W,"height":H,"prompt":"..."|"box_xyxy":[4]|"point_xy":[2],"threshold":0.5,"refine":2,"individual":false,"out":"<path>"}`. `raw` is `W×H×3` bytes of interleaved RGB; a length mismatch is an error.
- **Frame cache**: `fnv1a_hash()` (`main.rs:942`) builds a 64-bit FNV-1a digest of the raw bytes + dimensions, and for the same image it reuses `CachedFrame` (`native` f32 CHW and `frame` 1008 f16) (`serve_one`, `main.rs:963`). A different image is rebuilt.
- **Response**: one line of JSON on stdout. stderr is diagnostics only.
- **Engine side** (the `resident` module at `crates/engine/src/cmd/sam3_select_cmds.rs:261`): the slot is `OnceLock<Mutex<Option<Worker>>>` (`:380`). `matches()` requires the exe, pack and data_dir to agree. `REPLY_TIMEOUT = 180 seconds` (`:273`, including the pack load + ConvRot dequantization on the first run). A dedicated thread streams stdout into an mpsc, and the requester waits with `recv_timeout`. `Unavailable` (failed start, exit, timeout, non-JSON prose) discards the worker: if an existing worker died it restarts it once and retries, while a failure right after start gives up immediately. `Failed` (a healthy worker's `{"ok":false,"error":...}`) keeps the worker and reports the error directly. Once it lands on `Unavailable` it falls back to the one-shot CLI.
- The engine does not currently send `{"warmup":true}` on its own. The first selection pays the pack load (15.2 seconds measured).

## 12. Engine integration (`select.sam3`)

Command spec (`sam3_select_cmds.rs:606`):

```json
{"prompt":"text" | "box":[x,y,width,height] | "point":[x,y],
 "threshold":0..1=0.5, "refine":0..5=2, "individual":bool=false,
 "sampleAllLayers":bool=false, "mode":"replace|add|subtract|intersect"="replace"}
```

- **Prompt scaling** (`prompt_from`, `:223`): subtract the bounds from the document coordinates and convert them to the model's 1008 grid with `(x − bounds.x0) × 1008/W` (for a box, to its two xyxy corners). Text wins: if `prompt` is present it detects over the whole document and box/point are ignored (the same rule holds in the UI).
- **Sampling**: `with_doc_sampler` (reused from smartselect_cmds) samples the target layer, or the composite when `sampleAllLayers` is set, in RGBA, reduces it to 8bit RGB and writes `image.rgb` in a temporary directory. After inference it reads `masks.bin` / `masks.json` and deletes the temporary directory once done.
- **Compositing into the selection** (`HelperMask`, `:163`; `sam3_selection`, `:568`): each `[H, W]` mask is trimmed to its content bbox in document space, becomes a `Region` (GRAY8 coverage), and is replaced/added/subtracted/intersected by `combine_region(doc.selection, region, mode)`. The output JSON is `{"selected","changed","objects","bounds":[x,y,w,h]}`. When the mask is empty the result is `changed: false, objects: 0`.
- **Resolving helper/pack/data** (`:22`–`:159`): the `CRAFT_SAM3_HELPER` / `CRAFT_SAM3_PACK` environment variables win (a path that does not exist is an explicit error). The helper is looked for in the Preferences › Plug-ins additional folder (when enabled, directly inside it and directly inside `sam3/`) → `plugins` and `plugins\sam3` next to the executable → directly next to the executable → `plugins(\/sam3)(\/target\/release)` in each ancestor of the executable and of the CWD. The pack is looked for next to the helper, in `sam3/`, in an ancestor's `plugins/sam3`, and in an ancestor of the CWD. The data directory (where `vocab.json` lives) is inferred from the pack's/helper's parent. **A not-found error lists every candidate that was searched**, and no machine-specific path is written into the source.
- **one-shot fallback** (`run_helper`, `:457`): when the resident worker is `Unavailable`, it runs `--select-*` with `--verify-weights` (pack verification) in a single process. The last 4 lines of stderr go into the error message.

## 13. UI integration

| Place | Content |
|---|---|
| `state/tool.rs:83` | `Tool::Sam3Select` (added at the end of `Tool::ALL`, so the order of existing tools does not move) |
| `shell_commands.rs:181` | `("tools.select.sam3Select", "SAM Selection Tool", Some("W"))` |
| `prefs.rs:1353` | The selection column of `DEFAULT_TOOLBAR`, `[ObjectSelection, QuickSelection, MagicWand, Sam3Select]` |
| `panels/toolbar.rs:26` | The same 4 items in the W slot's flyout (SAM last) |
| `icons.rs` | The `scan` glyph |
| `state/tool_options.rs:106-115` | `sam3_prompt: String`, `sam3_threshold: f32` (0.5), `sam3_refine: u8` (2), `sam3_individual: bool` + `SampleAllLayers::sam3_select` |
| `tools/retouch_ui.rs:689` | `sam3_select_params`: click (nudge < 2px) → `point`, drag → `box` (document pixels). **If a prompt has been entered, it sends the prompt rather than point/box** (because the text path covers the whole document and box is ignored). Modifiers are ⇧⌥ for intersect, ⇧ for add, ⌥ for subtract, otherwise the options bar's mode |
| `tools/retouch_ui.rs:720` | `finish_sam3_selection`: runs `select.sam3` on release |
| `tools/retouch_ui.rs:1315, 1394` | The options bar (`sam3_select_options_bar`): 4 mode buttons, Sample All Layers, Prompt (placeholder `e.g. face`), Detect, Threshold, Refine, Individual Masks, a hint line |
| `canvas/gesture.rs:110` | Commits on release. Modifiers are judged by whether they were held at press time (`alt: d.modifiers.alt && !d.released.alt`) — so an early release still keeps the mode |
| `canvas/paint.rs` | The marching-ants rectangle while dragging |
| `canvas/tool_feedback.rs` | The mode badge (+ / − / ×) |
| `canvas.rs` / `input/hold_keys.rs` | Capturing the modifiers at press time, re-placing the rectangle with Space while dragging |
| `app/commands.rs` | Shows View › Extras once a selection exists (like the other `select.*`) |

**Two defects found and fixed by measurement**: (1) releasing Alt before the mouse button dropped the mode and turned the gesture into a plain selection → fixed by basing it on press time. (2) an entered prompt was ignored by click/drag → fixed by letting the prompt win.

**Japanese labels**: rows such as `SAM Selection Tool` → `SAM選択ツール` are **held inside the tree in `JA_TEXT_OVERRIDES` of `crates/ui-egui/src/i18n.rs`** (`tr()` reads them before the catalogue). The pinned craft-i18n revision does not have these rows, so this guarantees the Japanese display independently of the catalogue. The translation test is `retouch_ui::tests::sam_selection_strings_are_translated_in_japanese`.

## 14. Measured performance and accuracy

Measurement environment: RTX 5060 Ti, `plugins/sam3/data/test_face.png` (a 640×640 synthetic face scaled to 1008×1008), the HSWQ ConvRot INT8 pack.

**Speed**:

| Item | Value |
|---|---|
| Pack load (stack construction, including dequantization into VRAM) | 15.2 seconds (one-off) |
| ViTDet backbone (one pass over the 1008 frame) | 12.3–13.5 seconds |
| Interactive segmenter (one prompt) | 123–204 ms |
| Text selection end-to-end (detection + CLIP + 2 refinements) | 12.9 seconds |
| Engine e2e (point, `--ignored` test) | 19.0 seconds |
| UI e2e (box, real helper, `--ignored` test) | 34.9 seconds |

**Accuracy** (the masks are the final 1008×1008 logits, IoU is `logit > 0`):

| Stage | Comparison | Result |
|---|---|---|
| Interactive segmenter (box / point / mask prompts) | Rust vs `tracker_reference.py` | mask cosine 0.9999990–0.9999995, logit max abs 0.08–0.23, IoU (>0) 0.9998–0.99997 |
| `_refine_mask` (from the oracle's coarse mask) | binary mask | IoU 0.999968 (61,756 / 61,758 px) |
| box prompt (refine = 1) | Rust vs the M2d mask resized to 640 | IoU 1.000000 (61,568 px on both sides) |
| text `"face"` end-to-end | Rust vs the oracle + refine | IoU 0.999968, coverage 0.150771 on both sides, score 0.981934 / 0.981453 |
| `--select-raw` (interleaved RGB) vs the PNG path | the same image | IoU 0.999968 (2 px difference) |

Every difference stays within the f16 noise floor.

## 15. Tests

| Place | Count and content |
|---|---|
| tests in `crates/engine/src/cmd/sam3_select_cmds.rs` | 8 (7 pass + 1 ignored). Mask bbox trimming, empty masks, Region coverage, prompt validation, 1008 scaling, helper candidates, parsing of resident request lines/responses (including the rejection of prose), and an end-to-end with the real helper + real pack (`a_point_selects_the_disc_under_it_end_to_end`, ignored and requiring `CRAFT_SAM3_HELPER` / `CRAFT_SAM3_PACK`) |
| `crates/ui-egui/src/sam3_toolbar_tests.rs` | 4. The 4 items in the W slot and their order, the flyout's anchor, clicking a row, the Edit › Keyboard Shortcuts row |
| `crates/ui-egui/src/sam3_options_bar_tests.rs` | 3. In a real egui harness, the 4 mode buttons + Sample All Layers + Prompt + Threshold + Refine + Individual Masks + Detect are reachable by name, and the other tools have no bar |
| `crates/ui-egui/src/sam3_canvas_tests.rs` | The real canvas path (raw egui events): a click selects the disc, ⇧ still adds on an early release, the mode badge follows |
| `retouch_ui::tests::sam_selection_strings_are_translated_in_japanese` | The Japanese labels (from `JA_TEXT_OVERRIDES` in the tree) |

M4 record: the UI-wiring regression set `canvas:: hold_keys:: tool_feedback marquee retouch_ui tool_reset sam_selection` gave **1258 passed / 0 failed**.

## 16. Deployment

1. `cargo build --release` in `plugins/sam3` → `plugins/sam3/target/release/sam3-select.exe`.
2. Put the helper where it can be found (the search order in §12). While developing, the fastest way is to point `CRAFT_SAM3_HELPER` and `CRAFT_SAM3_PACK` at it explicitly.
3. The 1.74 GB pack goes next to the helper or in `plugins/sam3/`. `vocab.json` / `merges.txt` go in `plugins/sam3/data/` (`data/` next to the pack or the helper also works).
4. Because it links against the CUDA toolkit (cublas / curand / cuda_runtime), an MSVC environment is required. **cuDNN is not needed** (candle calls them directly). A machine without a GPU falls back to CPU (speed not measured).
5. When using the Preferences › Plug-ins additional folder, the settings keys are shared with LaMa (`"plugIns": {"useAdditionalPluginsFolder", "additionalPluginsFolder"}`).

## 17. Limitations and record of design decisions

- **Clean room**: no ComfyUI code is used at run time. `data/*_reference.py` are oracles for numerical verification only.
- **Still images only**: `propagation_convs` (for video tracking) are not loaded.
- **The input is fixed at 1008×1008**: a high-resolution canvas is analysed by scaling the whole document (partial-region analysis is a planned extension, not implemented).
- **The first selection pays the 15.2 second pack load**: the engine does not send `{"warmup":true}` on its own. Starting it ahead of time while idle is possible (the resident protocol already supports it).
- **VRAM usage is not measured**: the 1.74 GB pack plus the f16 trunk and the dequantized 41 layers on top.
- **One slot**: the resident worker is one per process. A mismatch in exe / pack / data_dir rebuilds it immediately.
- **Timeout**: 180 seconds (`REPLY_TIMEOUT`). It includes the first pack load.
- The ONNX + `ort` configuration (up to PLAN Revision 4) was retired: it eliminated the difficulty of dynamic control flow, the roughly 2.5 GB intermediate `.onnx`, and the version dependency on the external cuDNN / provider DLL (the Revision 5 decision).
