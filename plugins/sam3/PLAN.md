# SAM3.1 Selection Tool — Implementation Plan

Status: plan (no code written yet). Development folder: `plugins/sam3/` (this folder).

Revision 5 (2026-11-15): switched the inference backend from ONNX export + `ort` to **Pure Rust +
CUDA (`candle-core` / `candle-nn` with `features = ["cuda"]`) loading
`sam3.1_multiplex_hswq_convrot_int8.safetensors` directly onto the GPU** (Method 2), eliminating
`sam3-export`, multi-GB intermediate `.onnx` files, ONNX dynamic control-flow complexity, and
external cuDNN/ORT provider DLL dependencies, while preserving all Revision 4 measurements, key
remaps, and UI/Engine integration specifications. See "Corrections from revision 4" at the end.

## Goal

Add a **SAM選択ツール (`SAM Selection Tool`, `Tool::Sam3Select`)** to the selection-tool flyout,
directly below 自動選択ツール (`Magic Wand Tool`, `Tool::MagicWand`), so a click, box-drag, or
prompt trigger runs SAM 3.1 open-vocabulary detection and interactive segmentation on the active
image on CUDA GPU (text prompt such as `face:3, person`, optional box / positive / negative point
prompts) and commits the resulting mask as a PhotoCraft selection.

The feature is **fully independent of ComfyUI**: no ComfyUI code, no ComfyUI runtime, no Python at
run time. The ComfyUI SAM 3.1 implementation (`comfy_extras/nodes_sam3.py`, `comfy/ldm/sam3/*`,
`comfy/text_encoders/sam3_clip.py`, `comfy/supported_models.py`, and the HSWQ converter/loader in
`clip_convert/convert_sam31_convrot_int8.py` and `nodes/hswq_sam3_detect.py`) serves as the
**mathematical reference and development-time oracle** only — the same role the upstream LaMa model
played for `plugins/lama-cleaner`.

## Measured facts this plan is based on

### Helper-plugin pattern (measured in this repo)

- `plugins/lama-cleaner/Cargo.toml` is a standalone crate (`[workspace]` empty table) outside the main
  Cargo workspace, building a standalone helper binary in `plugins/`.
- The app resolves the helper executable and model file through Preferences › Plug-ins › Additional
  Plug-ins Folder → `plugins/` beside the running executable → `plugins/sam3/` → the executable's own
  folder (`crates/engine/src/cmd/retouch_cmds/lama_cleaner.rs:174-224`), with no hardcoded machine
  paths in engine or helper code.

### The weight file available in `plugins/sam3/` (measured)

`plugins/sam3/sam3.1_multiplex_hswq_convrot_int8.safetensors` (**1,738,165,347 B**, the HSWQ ConvRot
INT8 pack of the **SAM 3.1 Multiplex** checkpoint, `image_model = "SAM31"`, `multiplex = True`):

- **1,672 tensors total**:
  - **1,549 FP16 (`torch.float16`) tensors**: the entire 32-block ViTDet trunk
    (`detector.backbone.vision_backbone.trunk.*`), all three FPN necks (`convs`, `interactive_convs`,
    `propagation_convs`), the entire 24-layer CLIP text encoder
    (`detector.backbone.language_backbone.encoder.*`), and the entire `SAM31Tracker`
    (`tracker.model.*`) are already stored in pristine FP16.
  - **41 INT8 (`torch.int8`) `weight` tensors**: all 41 are 2D `nn.Linear` layers inside `detector.*`
    (`detector.backbone.language_backbone.resizer`, `detector.dot_prod_scoring.*`,
    `detector.geometry_encoder.*`, `detector.segmentation_head.cross_attend_prompt.out_proj`,
    `detector.transformer.decoder.*`, and `detector.transformer.encoder.*`).
  - **41 FP32 (`torch.float32`) `weight_scale` tensors**, each shaped `[out, 1]` (row-wise scale:
    `amax / 127.0`).
  - **41 U8 (`torch.uint8`) `comfy_quant` blobs**: all 41 decode to the exact 67-byte JSON string
    `{"format":"int8_tensorwise","convrot":true,"convrot_groupsize":256}` (there are **0**
    `groupsize: 64` tensors and **0** quantized `Conv2d` tensors in this file).
- **Deterministic ConvRot rotation rule (measured from
  `clip_convert/convert_sam31_convrot_int8.py:86-105` and `nodes/hswq_sam3_detect.py:144-165`)**:
  - No random seed exists or is needed. The rotation matrix $H_{256}$ is the normalized 4th
    Kronecker power of the 4×4 regular Hadamard matrix:
    $$H_4 = \begin{pmatrix} 1 & 1 & 1 & -1 \\ 1 & 1 & -1 & 1 \\ 1 & -1 & 1 & 1 \\ -1 & 1 & 1 & 1 \end{pmatrix}, \quad H_{256} = \frac{1}{\sqrt{256}}\,(H_4 \otimes H_4 \otimes H_4 \otimes H_4)$$
  - Because $H_4$ is symmetric and orthogonal, $H_{256}^T = H_{256}$ and $H_{256}^2 = I_{256}$.
    The converter quantized $W_{\text{rot}} = \text{reshape}(W\text{.view}(O, I/256, 256) \cdot H_{256}^T,\ [O, I])$.
    Therefore, exact dequantization + un-rotation back to FP16 on load for each of the 41 quantized
    layers is a single GPU matrix multiply:
    $$W_{\text{deq}} = \text{f32}(q) \cdot \text{weight\_scale}, \quad W_{\text{fp16}} = \text{f16}\!\left(\text{reshape}\!\left(W_{\text{deq}}\text{.view}(O, I/256, 256) \cdot H_{256},\ [O, I]\right)\right)$$

### Weight keys that must be remapped on load (measured)

The checkpoint is stored in upstream key layout. Four in-memory remaps are mandatory when loading
tensors from `sam3.1_multiplex_hswq_convrot_int8.safetensors`
(`comfy/supported_models.py:2569-2600`):

1. **`tracker.model.*` → `tracker.*`**: the file holds **457** `tracker.model.*` keys and **0**
   plain `tracker.*` keys. Strip the `.model` segment so `tracker.model.interactive_sam_mask_decoder.*`
   maps to `tracker.interactive_sam_mask_decoder.*`, `tracker.model.interactivity_no_mem_embed`
   maps to `tracker.interactivity_no_mem_embed`, etc.
2. **Fused CLIP QKV → separate `q_proj` / `k_proj` / `v_proj` (24 modules, 48 tensors)**:
   `detector.backbone.language_backbone.encoder.transformer.resblocks.{0..23}.attn.in_proj_weight`
   (`[3072, 1024]`) and `.in_proj_bias` (`[3072]`) split along dim 0 into three `[1024, 1024]` weights
   and three `[1024]` biases (`.q_proj`, `.k_proj`, `.v_proj`).
3. **Fused Detector `SplitMHA` QKV → separate `q_proj` / `k_proj` / `v_proj` (37 modules, 74 tensors)**:
   In addition to CLIP, **37 attention modules in `detector.*`** store fused `.in_proj_weight`
   (`[768, 256]`) and `.in_proj_bias` (`[768]`):
   - `detector.geometry_encoder.encode.{0..2}.{self_attn, cross_attn_image}` (6 modules)
   - `detector.segmentation_head.cross_attend_prompt` (1 module)
   - `detector.transformer.decoder.layers.{0..5}.{self_attn, cross_attn, ca_text}` (18 modules)
   - `detector.transformer.encoder.layers.{0..5}.{self_attn, cross_attn_image}` (12 modules)
   Each splits along dim 0 into `.q_proj`, `.k_proj`, `.v_proj` (`[256, 256]` and `[256]` each).
4. **Tracker `TwoWayTransformer` key rename**: for every key containing
   `sam_mask_decoder.transformer.` (which covers both `sam_mask_decoder.transformer.*` and
   `interactive_sam_mask_decoder.transformer.*`), apply:
   - `.mlp.lin1.` → `.mlp.0.`
   - `.mlp.lin2.` → `.mlp.2.`
   - `.norm_final_attn.` → `.norm_final.`

### Vision trunk, FPN necks, Detector, and Interactive Tracker (`comfy/ldm/sam3/*`)

- **Image preprocessing (`nodes_sam3.py:118`, `sam.py:276`)**:
  - Input RGB image is scaled to `[0.0, 1.0]` (`u8 as f32 / 255.0`) and resized to `1008×1008` with
    `bilinear` interpolation (`align_corners = False`).
  - **No ImageNet mean/std normalization is used**: `ViTDet.patch_embed`
    (`Conv2d(3, 1024, k=14, s=14, bias=False)`) takes `[0.0, 1.0]` RGB directly.
- **ViTDet trunk (`sam.py:223-299`)**:
  - `ViTDet(img_size=1008, patch_size=14, embed_dim=1024, depth=32, num_heads=16, mlp_ratio=4.625,
    qkv_bias=True, window_size=24, global_att_blocks=(7,15,23,31), use_rope=True,
    pretrain_img_size=336)`.
  - `patch_embed` produces `[B, 1024, 72, 72]`, permuted/reshaped to `[B, 5184, 1024]`.
  - `pos_embed` is stored as `[1, 577, 1024]` (`1 cls + 24×24`). `_get_pos_embed(5185)` tiles the
    `24×24` spatial grid (`pos[:, 1:]`) by `4×4`, crops to `72×72` (`[1, 5184, 1024]`), and **line 282
    slices `pos[:, 1:5185]`**, discarding `cls_pos`. Thus **no `cls` token ever enters the trunk
    blocks**: the trunk runs strictly on the **`72×72 = 5,184` spatial grid** (`[B, 72, 72, 1024]`),
    followed by `ln_pre` (`LayerNorm(1024)`).
  - Attention is **windowed (`24×24` windows = 9 windows of 576 tokens)** in 28 blocks and **global
    (`72×72 = 5,184` tokens)** in blocks `7, 15, 23, 31`.
  - `freqs_cis` (global `72×72`, `head_dim = 64`, `scale_pos = 24.0 / 72.0 = 1/3`) and
    `freqs_cis_window` (window `24×24`, `head_dim = 64`, `scale_pos = 1.0`) are generated by
    `rope_2d` (Flux `EmbedND` 2D axial RoPE) and applied to `q, k` inside `Attention`.
- **SAM 3.1 (`multiplex=True`) FPN necks (`sam.py:301-425`, `detector.py:394-396`)**:
  - Unlike SAM 3 (`multiplex=False`, 4 scales `[4.0, 2.0, 1.0, 0.5]`, `scalp=1`, `sam2_convs`),
    **SAM 3.1 (`multiplex=True`) has 3 scales `[4.0, 2.0, 1.0]` (`scalp = 0`) and NO `0.5` / `36×36`
    level and NO `sam2_convs`**:
    - Scale `4.0` (index `0`, output `288×288`): `dconv_2x2_0` (`ConvTranspose2d(1024→512, k=2, s=2)`) →
      `GELU` → `dconv_2x2_1` (`ConvTranspose2d(512→256, k=2, s=2)`) → `conv_1x1(256→256)` →
      `conv_3x3(256→256, pad=1)`.
    - Scale `2.0` (index `1`, output `144×144`): `dconv_2x2` (`ConvTranspose2d(1024→512, k=2, s=2)`) →
      `conv_1x1(512→256)` → `conv_3x3(256→256, pad=1)`.
    - Scale `1.0` (index `2`, output `72×72`): identity → `conv_1x1(1024→256)` →
      `conv_3x3(256→256, pad=1)`.
  - **`detector.backbone.vision_backbone.convs.{0,1,2}`** produces the 3 detector feature maps
    (`[288×288, 144×144, 72×72]`).
  - **`detector.backbone.vision_backbone.interactive_convs.{0,1,2}`** produces the 3 interactive
    tracker feature maps (`[288×288, 144×144, 72×72]`) consumed by `forward_segment` (mask refinement
    and point/box segmentation). (`propagation_convs` is for video tracking and is not loaded in still-image
    selection.)
- **DETR Detector (`detector.py:84-510`)**:
  - `PositionEmbeddingSine(num_pos_feats=256, normalize=True, scale=2π)` on the `72×72` level (`enc_feat`,
    `enc_pos`).
  - `language_backbone.resizer` (`Linear(1024→256)`) projects the CLIP token embeddings.
  - `GeometryEncoder`: encodes optional point prompts (`points_direct_project` + bilinear `grid_sample` +
    `points_pool_project` + sine pos + `label_embed`) and box prompts (`boxes_direct_project` +
    `roi_align(img_feat_2d, boxes_scaled, roi_size=7, aligned=False)` + `boxes_pool_project`
    `Conv2d(256, 256, k=7)` + `encode_boxes` + `label_embed`), runs 3 pre-norm `EncoderLayer`s on
    `geo`, and runs `_run_geo_layer` on `cls_embed` (`geo_cls` attending to `k = img_flat + pos_flat`,
    `v = img_flat`), then concatenates `[text_embeddings, geo_prompts, geo_cls]` and extends
    `text_mask` with `True` for the geometry tokens.
  - `TransformerEncoder`: 6 pre-norm `EncoderLayer`s (`self_attn` with `q = k = norm1(x) + pos, v = norm1(x)`,
    `cross_attn_image` to `text_memory` with `text_mask`, FFN with ReLU).
  - `TransformerDecoder`: 6 post-norm `DecoderLayer`s over **`201` tokens** (`presence_token` [1]
    prepended to `query_embed` [200]):
    - LayerNorm order inside `DecoderLayer`: `norm2` (after `self_attn`) → `catext_norm` (after
      `ca_text`) → `norm1` (after `cross_attn`) → `norm3` (after FFN).
    - `query_pos = ref_point_head(gen_sineembed_for_position(ref_points, 256))` (`512→256`), with a
      zero vector prepended for `presence_token`.
    - `_compute_box_rpb(ref_points, 72, 72)`: computes log-scaled coordinate deltas
      $\text{sign}(8d)\log_2(|8d|+1)/\log_2 8$, runs `boxRPB_embed_x` and `boxRPB_embed_y` (`2→256→8`),
      prepends a zero row for `presence_token`, and passes the resulting `[B, 8, 201, 5184]` bias as
      `cross_attn` mask.
    - Iterative box refinement after decoder layers `0..4`:
      `ref_points = sigmoid(inverse_sigmoid(ref_points) + bbox_embed(norm(tgt)))`.
  - `DotProductScoring`: runs `prompt_mlp` (2-layer residual MLP + output `LayerNorm`), masked average
    pools over `prompt_mask`, projects queries (`hs_proj`) and pooled prompt (`prompt_proj`), scales by
    $1/\sqrt{256}$, clamps to `[-12.0, 12.0]`, and filters by `scores.sigmoid() > threshold`.
  - `SegmentationHead`: `cross_attend_prompt` (`SplitMHA` on `cross_attn_norm(memory)` attending to
    `prompt`), writes `encoder_visual` into `seg_features[2]` (`72×72`), runs the 3-stage top-down
    `PixelDecoder` (`Conv2d(3×3, pad=1) + GroupNorm(8, 256) + ReLU` with `nearest` 2× upsampling from
    `72×72` → `144×144` → `288×288`), `instance_seg_head` (`Conv2d(256→256, k=1)`), and `MaskPredictor`
    (`mask_embed` 3-layer MLP + `einsum("bqc,bchw->bqhw")` producing `[B, 200, 288, 288]` coarse mask
    logits).
- **Interactive SAM Mask Refinement (`detector.py:524-565`, `tracker.py:276-330, 1149-1210`)**:
  - In `SAM31Tracker` (`multiplex=True`), `forward_segment` runs `interactive_convs.{0,1,2}` to obtain
    `high_res = [feat_288, feat_144]` and `backbone_feat = feat_72 + tracker.interactivity_no_mem_embed`.
  - `tracker.interactive_sam_prompt_encoder` encodes points (padded with a `label = -1`
    `not_a_point_embed` token when `boxes is None`), boxes (`point_embeddings[2]` and `[3]`), and
    mask logits (`F.interpolate` to `288×288` with **`antialias=True`** → `mask_downscaling` 2×2 stride-2
    convs with `LayerNorm2d` to `72×72`, or `no_mask_embed` when `masks is None`), plus `get_dense_pe()`
    via `pe_layer.positional_encoding_gaussian_matrix` (`[2, 128]`).
  - `tracker.interactive_sam_mask_decoder` (`SAMMaskDecoder`, single-object, `num_mask_tokens = 4`)
    runs `SAMTwoWayTransformer` (depth 2), upscales with `output_upscaling` + `conv_s1(feat_144)` +
    `conv_s0(feat_288)` to `288×288`, applies `output_hypernetworks_mlps.{0..3}`, gates by
    `object_score_logits > 0` (else `-1024.0`), selects mask index `0` when `multimask_output = False`
    (or `argmax(ious)` among `1..3` when `0 < num_pts <= 1`), and bilinearly interpolates to
    `1008×1008`.
- **CLIP Text Encoder (`comfy/text_encoders/sam3_clip.py`)**:
  - `CLIPTextModel`: hidden 1024, 24 layers, 16 heads, `quick_gelu` (`x * sigmoid(1.702 * x)`),
    `vocab_size = 49408`, `max_position_embeddings = 32`, `layer_norm_eps = 1e-5`.
  - Special tokens: `start = 49406`, `end = 49407`, **`pad = 0` (`pad_with_end = False`)**.
  - `attention_mask`: `1` for active tokens (including start and end), `0` for trailing `0` pad tokens.
    Both the CLIP self-attention layers (combined with the causal mask) and the Detector receive this
    `attention_mask`.
  - Prompt syntax (`_parse_prompts`): parentheses stripped, comma-separated categories, optional
    `category:N` sets `max_detections = max(1, round(float(N)))` (default `1`).
- Vocabulary files come from ComfyUI's tokenizer directory (`<ComfyUI>/comfy/sd1_tokenizer/{vocab.json,merges.txt}`)
  and are copied once into `plugins/sam3/data/`.

### UI and Engine integration points (measured, with exact files, line numbers, and tests)

| Concern | File & Line | Details |
|---|---|---|
| `pub enum Tool` | `crates/ui-egui/src/state/tool.rs:4` | Add `Sam3Select` variant |
| `Tool::ALL` (`[Tool; 71]`) | `crates/ui-egui/src/state/tool.rs:85` | Resize `70` → `71`, insert `Tool::Sam3Select` in `Tool::ALL` matching `shell_commands::TOOLS` order |
| `Tool::label`, `key`, `from_name` | `crates/ui-egui/src/state/tool.rs:158, 272, 324` | `label() => "SAM Selection Tool"`, `key() => 'W'`, `from_name` supports `"sam3Select"`, `"samSelection"`, `"SAM Selection Tool"` |
| Engine tool shortcut table | `crates/engine/src/shell_commands.rs:110-181` | Add `("tools.select.sam3Select", "SAM Selection Tool", Some("W"))` in the **exact same order as `Tool::ALL`** (enforced by `crates/ui-egui/src/input/tool_shortcuts.rs:106` `the_engines_tool_table_matches_the_tools`) |
| `Tool::ALL.len()` & `README.md` test gate | `crates/ui-egui/src/state.rs:868, 894-912` & `README.md:188-189` | Update `assert_eq!(Tool::ALL.len(), 71)` in `state.rs:868`, update `<h4>🧰 71 tools</h4>` and add `SAM Selection` to the `·`-separated list in `README.md:189` (enforced by `readme_tool_list_matches_tool_all`) |
| Flyout group of the `W` slot | `crates/ui-egui/src/panels/toolbar.rs:26` | Change `&[Tool::ObjectSelection, Tool::QuickSelection, Tool::MagicWand]` → `&[Tool::ObjectSelection, Tool::QuickSelection, Tool::MagicWand, Tool::Sam3Select]` |
| `'W'` group cycle tests | `crates/ui-egui/src/tool_shift_cycle_order_tests.rs:96, 131` & `crates/ui-egui/src/input/tool_key_default_tests.rs:67` | Update expected `'W'` cycle sequences to include `Tool::Sam3Select` after `Tool::MagicWand` |
| Icon per tool | `crates/ui-egui/src/icons.rs:175, 205` | Map `Tool::Sam3Select` to a distinct vector icon (verified by `icons.rs:293` test over `Tool::ALL`) |
| Tool options state | `crates/ui-egui/src/state/tool_options.rs` | Add `sam3_prompt: String`, `sam3_threshold: f32`, `sam3_refine_iterations: u8`, `sam3_individual_masks: bool`, and `sample_all.sam3_select: bool` |
| Tool reset table & test gate | `crates/ui-egui/src/tools/tool_reset.rs:130, 212, 350-430` | Register `Tool::Sam3Select` in `sample_all_key`, `option_keys`, and `reset_options` (enforced by `tool_reset.rs` tests over `Tool::ALL`) |
| Options bar & gesture finish | `crates/ui-egui/src/tools/retouch_ui.rs:650-690, 920, 1192` & `crates/ui-egui/src/panels/options_bar.rs` | Render SAM Selection options bar (mode buttons New/Add/Subtract/Intersect, prompt input, Detect button, threshold, refine passes, Sample All Layers, Select and Mask); implement `finish_sam3_selection` |
| Canvas click / box-drag / preview / cursor | `crates/ui-egui/src/canvas/gesture.rs:107`, `canvas/tool_input.rs`, `canvas/paint.rs:653`, `canvas.rs:1046`, `canvas/tool_feedback.rs:51` | Box drag draws marquee preview (`paint.rs:653`) and commits `select.sam3` with `box` on release (`gesture.rs:107`); click commits `select.sam3` with `points` (or runs text prompt); selection mode badges (`+`/`-`/`×`) via `tool_feedback.rs` |
| Selection combine into `doc.selection` | `crates/engine/src/cmd/smartselect_cmds.rs:97-109` | `doc.selection` is `Option<photocraft_raster::Surface>` (`GRAY8`); convert the helper's mask into `photocraft_algo::selection::Region` and call `combine_region(doc.selection.as_ref(), Some(&region), mode)` |
| Command registration | `crates/engine/src/cmd.rs:160, 1368` & `crates/engine/src/cmd/atn_cmds/events.rs` | Register `select.sam3` in `smartselect_cmds::specs()` (or `sam3_select_cmds::specs()`) and map its action descriptor event in `atn_cmds/events.rs` |
| Japanese UI localization (`craft-i18n`) | `<craft-i18n>/locales/ja/photocraft.tsv` | Add the `SAM Selection Tool` and options bar rows so the Japanese UI renders `SAM選択ツール` below `自動選択ツール` |

## Architecture decision (Method 2: Pure Rust + CUDA direct `.safetensors` inference)

Instead of exporting 4 multi-GB ONNX files and fighting ONNX static-graph limitations or external
cuDNN / `onnxruntime_providers_cuda.dll` version mismatches, **`plugins/sam3` is implemented as a
Pure Rust + CUDA crate (`sam3-select`) using `candle-core` and `candle-nn` (`features = ["cuda"]`)
that loads `plugins/sam3/sam3.1_multiplex_hswq_convrot_int8.safetensors` directly onto the GPU.**

### Why this architecture is optimal

1. **Zero ONNX conversion & zero extra disk footprint**:
   - Reads `sam3.1_multiplex_hswq_convrot_int8.safetensors` (1.74 GB) directly via memory-mapped
     `safetensors`. No `sam3-export` crate and no ~2.5 GB of duplicate `.onnx` files.
2. **Direct 1:1 translation of `comfy/ldm/sam3/*` and `comfy/text_encoders/sam3_clip.py`**:
   - Dynamic branches (`if points.is_some()`, `if boxes.is_some()`, `multimask_output`,
     `_compute_box_rpb`, iterative `ref_points` refinement, and per-detection `_refine_mask` crop
     loops) are written as straightforward, type-safe Rust `if` and `for` statements over GPU
     `Tensor`s.
   - Trunk features (`trunk_out` and `interactive_convs` outputs) are naturally cached in GPU memory
     as Rust variables:
     - For point or box prompts on the full image, the 32-layer ViTDet trunk + `interactive_convs`
       runs **once**, and the lightweight `interactive_sam_mask_decoder` iterates `refine_iterations`
       times on the cached GPU tensors in milliseconds.
     - For text-prompted `_refine_mask` on a 10%-padded crop, `trunk(crop_frame)` + `interactive_convs`
       runs **once per detected box**, and the `refine_iterations` loop only re-runs
       `interactive_sam_prompt_encoder` + `interactive_sam_mask_decoder`.
3. **Self-contained CUDA binary (`sam3-select.exe`)**:
   - Links directly against the local CUDA toolkit (`cublas`, `curand`, `cuda_runtime`) under MSVC
     (`vcvars64.bat`), with no external `cudnn64_9.dll` or `onnxruntime_providers_cuda.dll` needed.
   - Automatically selects `Device::new_cuda(0)` (FP16 on Tensor Cores) and falls back cleanly to
     `Device::Cpu` if no CUDA GPU is present.

### Crate layout (`plugins/sam3/`)

Standalone crate `sam3-select` (`[workspace]` empty table in `plugins/sam3/Cargo.toml`), building
`plugins/sam3-select.exe`:

- `plugins/sam3/Cargo.toml`:
  - `candle-core = { version = "0.8", features = ["cuda"] }` (or latest compatible with local CUDA)
  - `candle-nn = { version = "0.8", features = ["cuda"] }`
  - `safetensors = "0.4"`, `memmap2 = "0.9"`, `half = "2"`, `image = "0.25"`, `serde`, `serde_json`, `thiserror = "2"`
- `plugins/sam3/data/`:
  - `vocab.json` + `merges.txt` (49,408-entry CLIP BPE, copied from `ComfyUI/comfy/sd1_tokenizer/`).
- `plugins/sam3/src/`:
  - `loader.rs`: memory-maps `sam3.1_multiplex_hswq_convrot_int8.safetensors`, constructs the
    deterministic $H_{256}$ Hadamard matrix on GPU, dequantizes + un-rotates the 41 ConvRot INT8
    Linear layers to FP16 on GPU, and performs the 4 mandatory key remaps (`tracker.model.*` →
    `tracker.*`, 24 CLIP `in_proj` splits, 37 Detector `SplitMHA` `in_proj` splits, and
    `sam_mask_decoder.transformer.*` renames).
  - `tokenizer.rs`: CLIP BPE tokenizer (`max_length = 32`, `start = 49406`, `end = 49407`, `pad = 0`,
    `pad_with_end = false`, `attention_mask`, and comma-separated `prompt:N` parser).
  - `clip.rs`: 24-layer `SAM3ClipModel` (`hidden_size = 1024`, 16 heads, `quick_gelu`, causal +
    attention mask) + `language_backbone.resizer` (`1024 → 256`).
  - `vitdet.rs`: 32-layer `ViTDet` trunk (`patch_embed` `14×14`, tiled `pos_embed[:, 1:5185]` over
    `72×72 = 5,184` tokens, `ln_pre`, 24×24 window partition/unpartition, global attention in blocks
    `7, 15, 23, 31`, 2D axial `rope_2d`) + 3-scale `FPNScaleConv` necks (`convs.{0,1,2}` and
    `interactive_convs.{0,1,2}` at scales `[4.0, 2.0, 1.0]` → `288×288, 144×144, 72×72`).
  - `detector.rs`: `PositionEmbeddingSine`, `GeometryEncoder` (bilinear `grid_sample` for points,
    `roi_align` for boxes, 3-layer encoder, `_run_geo_layer` on `cls_embed`), 6-layer
    `TransformerEncoder`, 6-layer `TransformerDecoder` (`201` tokens = `presence_token` + 200 queries,
    `norm2 → catext_norm → norm1 → norm3`, `_compute_box_rpb` log-scaled bias, iterative `bbox_embed`),
    `DotProductScoring`, and `SegmentationHead` (`cross_attend_prompt` + 3-stage `PixelDecoder` +
    `MaskPredictor`).
  - `tracker.rs`: `SAM31Tracker` interactive segmentation (`interactivity_no_mem_embed`,
    `interactive_sam_prompt_encoder` with `PositionEmbeddingRandom`, `interactive_sam_mask_decoder`
    with 2-layer `TwoWayTransformer`, `output_upscaling` + `conv_s0` / `conv_s1`, hypernetwork MLPs,
    IoU & object score heads) and `_refine_mask` (10%-padded crop, trunk + `interactive_convs`
    computed once per crop, mask decoder iterated `refine_iterations` times).
  - `main.rs`:
    - One-shot CLI mode:
      `sam3-select.exe --image <in.png> --output <mask.png> [--boxes-out <boxes.json>] [--prompt "face:3"] [--boxes <json>] [--pos-points <json>] [--neg-points <json>] [--threshold 0.5] [--refine-iterations 2] [--individual-masks] [--model <safetensors>]`
    - Persistent stdio worker mode (`sam3-select.exe --serve [--model <safetensors>]`): loads the
      model into CUDA VRAM once and processes newline-delimited JSON requests over stdin/stdout so
      consecutive clicks in PhotoCraft run in sub-second GPU time without reloading weights.

### Values generated from run-time geometry (must match reference bit-for-bit)

| Value | Exact Rule in SAM 3.1 (`multiplex=True`) |
|---|---|
| Image input scaling | `f32` / `f16` RGB in `[0.0, 1.0]`, bilinear resize to `1008×1008` (`align_corners=False`), **no mean/std subtraction** |
| Trunk token grid | `1008 / 14 = 72` → **`72×72 = 5,184` tokens** (`cls` is never concatenated to `x`) |
| `pos_embed` expansion | Tile stored `pos_embed[:, 1:]` (`24×24`) by `4×4`, crop to `72×72`, **drop `pos_embed[:, :1]` (`cls_pos`)**, add `[1, 5184, 1024]` to patch embeddings |
| `freqs_cis` (global) | `rope_2d(72, 72, head_dim=64, theta=10000.0, scale_pos=24.0/72.0)` |
| `freqs_cis_window` | `rope_2d(24, 24, head_dim=64, theta=10000.0, scale_pos=1.0)` |
| Attention partitioning | Windowed `24×24` in blocks `0..6, 8..14, 16..22, 24..30`; global `72×72` in blocks `7, 15, 23, 31` |
| FPN necks (`multiplex=True`) | 3 scales `[4.0, 2.0, 1.0]` → `288×288, 144×144, 72×72` (`scalp=0`, no `0.5` level): `convs` for detector, `interactive_convs` for interactive segment/refine |
| DETR sine pos encoding | `PositionEmbeddingSine(num_pos_feats=256, temperature=10000.0, normalize=True, scale=2π)` |
| DETR decoder queries | `201` tokens (`presence_token` [1] + `query_embed` [200]), `DecoderLayer` norm order `norm2 → catext_norm → norm1 → norm3`, `_compute_box_rpb` log-scale bias `[B, 8, 201, 5184]`, iterative `bbox_embed` update after layers `0..4` |
| Interactive tracker init | Add `tracker.interactivity_no_mem_embed` (`[1, 1, 256]`) to `int_f2` (`72×72`), decode with `interactive_sam_prompt_encoder` + `interactive_sam_mask_decoder` |
| Refine crop | Detection box padded by 10 % on each side, clamped to `[0..W, 0..H]`, cropped from original image and coarse mask, resized to `1008×1008`, trunk + `interactive_convs` run once per crop, decoder refined `refine_iterations` times, combined with `coarse | refined` |

## Inspection region and Web build

- **Inspection region**: SAM 3.1 resizes its input to `1008×1008`. On high-resolution canvases, the
  command can analyze either the full document or the active view/crop region and map the resulting
  mask back to document coordinates before calling `combine_region`.
- **Web build (`wasm32`)**: Like `lama_cleaner.rs`, the helper process is gated with
  `#[cfg(not(target_arch = "wasm32"))]`; on `wasm32`, `select.sam3` returns a clean error message
  (`"SAM Selection is only available in the desktop app"`).

## Milestones (each ends with a measured check)

- **M0 Vocabulary + CLIP BPE tokenizer (`src/tokenizer.rs`)**: copy `vocab.json` and `merges.txt` to
  `plugins/sam3/data/`, implement the CLIP BPE tokenizer in Rust with SAM3 rules (`max_length = 32`,
  `start = 49406`, `end = 49407`, `pad = 0`, `pad_with_end = false`, `attention_mask`, and `prompt:N`
  parsing). Check: token IDs and attention masks match `SAM3Tokenizer` bit-for-bit on test prompts.
- **M1 Direct `.safetensors` CUDA Loader, ConvRot Dequantizer & Key Remapper (`src/loader.rs`)**:
  load `plugins/sam3/sam3.1_multiplex_hswq_convrot_int8.safetensors` onto CUDA GPU, un-rotate the 41
  ConvRot INT8 Linear layers via $H_{256}$, and apply all 4 key remaps (`tracker.model.*`, 24 CLIP
  `in_proj`, 37 Detector `SplitMHA` `in_proj`, and `sam_mask_decoder.transformer.*`). Check: all
  expected module weights resolve with zero missing keys and match the reference dequantized tensors.
- **M2 CUDA Model Forward (`src/clip.rs`, `src/vitdet.rs`, `src/detector.rs`, `src/tracker.rs`,
  `src/main.rs`)**: implement the full SAM 3.1 Multiplex pipeline in Rust on CUDA (`CLIPTextModel`,
  `ViTDet` + FPN, `SAM3Detector`, `SAM31Tracker` interactive decoder, and `_refine_mask`), build
  `plugins/sam3-select.exe`, and verify masks and bounding boxes on a real test image (text prompt
  `"face"`, point prompt, box prompt) against the reference oracle on GPU.
- **M3 Engine command (`select.sam3`)**: implement `select.sam3` in `crates/engine`, resolving
  `sam3-select.exe` and `sam3.1_multiplex_hswq_convrot_int8.safetensors` via the `plugins/` /
  `plugins/sam3/` discovery chain, converting the output mask to `photocraft_algo::selection::Region`,
  and combining into `doc.selection` via `combine_region`. Register in `cmd.rs` and
  `atn_cmds/events.rs`. Check: `cargo test -p photocraft-engine --lib sam3` passes (including
  adversarial param tests for `panic_hunt`).
- **M4 UI + Localization (`SAM選択ツール`)**: wire `Tool::Sam3Select` into `state/tool.rs`,
  `shell_commands.rs`, `README.md`, `state.rs`, `panels/toolbar.rs:26`, `icons.rs`, `tool_options.rs`,
  `tools/tool_reset.rs`, `tools/retouch_ui.rs`, `canvas/gesture.rs`, `canvas/tool_input.rs`,
  `canvas/paint.rs`, `canvas/tool_feedback.rs`, update `tool_shift_cycle_order_tests.rs` and
  `input/tool_key_default_tests.rs`, and add `SAM選択ツール` to `craft-i18n/locales/ja/photocraft.tsv`.
  Check: all `photocraft-ui-egui` unit tests and `cargo xtask wasm` pass.
- **M5 End-to-end build & verification**: build release binaries with `CRAFT_I18N_DIR` and
  `CRAFT_FONTS_DIR` under MSVC (`vcvars64.bat`), verify the flyout menu and CUDA-accelerated SAM 3.1
  selection end-to-end.

---

## Measured verification of the interactive segmenter (M2d)

`src/tracker.rs` implements `SAM31Tracker`'s interactive path (`forward_segment`). The oracle is
`data/tracker_reference.py`, which re-implements `tracker.py:268-329 / 425-515 / 518-577` and
`sam.py:28-117` on the same pack and the same trunk as `data/vitdet_reference.py`.

Test image `data/test_face.png` (640×640 synthetic face resized to 1008×1008), HSWQ ConvRot INT8 pack.
Masks are the final `1008×1008` logits; `IoU` is taken on `logit > 0`.

| Prompt | IoU predicted (Rust / ref) | mask cosine | max abs | mean abs | IoU (>0) |
|---|---|---|---|---|---|
| box `190.0,142.9,632.0,583.7` | 4.769531 / 4.765625 | 0.999999046 | 0.17188 | 0.008998 | 0.999915 |
| point `410.0,360.0` (multimask) | 4.820312 / argmax 4.8203125 | 0.999999141 | 0.22656 | 0.010971 | 0.999974 |
| mask prompt (`data/seg_ref.bin`) | 4.468750 / 4.468750 | 0.999999476 | 0.08008 | 0.005324 | 0.999825 |

`object_score` (logit): `26.171875 / 26.203125`, `19.953125 / 19.953125`, `5.273438 / 5.257813`.
The point prompt selects the same multimask column on both sides (index `2`).

Two behaviours had to be reproduced exactly and are covered by `nn.rs` unit tests:

1. **`label = -1` tokens are zeroed before `not_a_point_embed` is added** (`tracker.py:559-561`).
   Adding the embedding to the raw coordinate encoding instead shifted `object_score` from `5.26` to
   `0.92` on the mask-prompt case.
2. **The `1008×288` mask-prompt resize uses `antialias=True`** (`tracker.py:293`). The filter is the
   separable triangle kernel: `scale = in/out`, `support = max(scale, 1)`,
   `center = (o + 0.5) * scale - 0.5` clamped at zero, taps inside the input, weight
   `1 - |i - center| / support`, normalized. `nn::resample_bilinear_antialias` matches
   `F.interpolate(..., antialias=True)` to `3.2e-07` over ten size pairs. Without it `object_score`
   reads `4.54` instead of `5.26`.

Runtime on the RTX 5060 Ti: backbone `12.3-13.5 s`, segmenter `123-204 ms` per prompt.

## Measured verification of the selection pipeline (M2e)

`src/select.rs` implements `_refine_mask` (`nodes_sam3.py:42-84`) and the mask assembly
(`193-255`); `src/selection.rs` is the `--select` driver. The oracle is
`data/select_reference.py`, which takes the detections from an already-verified
`detector_reference.py` dump so the comparison isolates the refinement stage.

Test image `data/test_face.png` (640x640), pack `sam3.1_multiplex_hswq_convrot_int8.safetensors`,
`threshold = 0.5`, `refine_iterations = 2`.

| Stage | Comparison | Result |
|---|---|---|
| `_refine_mask` (coarse from the oracle dump) | binary mask, Rust vs `select_reference.py` | IoU **0.999968** (61,756 / 61,758 px), same box and score |
| box prompt, `refine_iterations = 1` | Rust `--select-box` vs the M2d box mask resized to 640x640 | IoU **1.000000** (61,568 px both) |
| text prompt `"face"`, end to end | Rust `--select-prompt` vs the oracle dump + refine | IoU **0.999968**, coverage `0.150771` both, score `0.981934` vs `0.981453` |

Runtime: `stack loaded in 15.2 s` (one-off), `selection in 12.9 s` for the text path
(detector + CLIP + 2 refinement passes).

Boxes come out in original pixel space, matching `detector.py:483-486` (`orig_size` scales both
the boxes and the coarse masks). `select.rs` also takes the model dtype explicitly: the image crop
is resized in f32 and only then cast to f16, the order `common_upscale(...).to(dtype=dtype)` uses.

`--select-raw <path> --select-raw-size <w>x<h>` takes interleaved 8-bit RGB instead of a PNG, which is
the form the engine hands over (no image codec on either side). Measured against the PNG path on the
same image: IoU **0.999968** (61,754 vs 61,756 px, 2 px different), the same f16 noise floor as the
other stages.

---

## Measured verification of the engine command (M3)

`crates/engine/src/cmd/sam3_select_cmds.rs`: adds `select.sam3`. It moves pixels only: the
command rasterises the sampled layer (or the composite with `sampleAllLayers`) into interleaved
8-bit RGB, runs the helper, and turns its `[N, H, W]` binary masks into a selection through
`combine_region`. Registered in `cmd.rs` (`pub mod sam3_select_cmds`, plus its `specs()`
extend) and re-exported from `lib.rs`. `smartselect_cmds.rs` is not modified; the command
reuses its `pub(crate) with_doc_sampler`.

Helper and pack resolution is explicit and every searched path is named on failure:
`CRAFT_SAM3_HELPER` / `CRAFT_SAM3_PACK`, then `<exe dir>/sam3-select`, then
`plugins/sam3/target/release/` under the exe's and the working directory's ancestors, and the pack
beside the helper or in `plugins/sam3/`. No absolute path is baked into the source.

Measured on this machine (RTX 5060 Ti, 256x256 document with a green disc of radius 40):

| Check | Result |
|---|---|
| `cargo test -p photocraft-engine --lib sam3` | 5 passed, 1 ignored |
| `a_point_selects_the_disc_under_it_end_to_end` with the real helper (--ignored) | **pass**, 19.0 s, selection bounds between 40 and 160 px across |
| `cargo build -p photocraft-engine` | clean |

The command accepts `prompt` (detect + refine), `point` (the object under the click) or
`box` (the object inside the box) in document pixels, plus `threshold`, `refine` (0-5),
`individual`, `sampleAllLayers` and `mode`. Point and box are scaled to the model's
1008 grid exactly as `SAM3_Detect` does (`x / W * 1008`).

---

## Measured verification of the UI wiring (M4)

The tool is `Tool::Sam3Select`, appended at the end of the enum so the existing tool order does not
move (`Tool::ALL` is `[Tool; 71]`, `README.md` says `71 tools`).

| Place | Change |
|---|---|
| `state/tool.rs` | enum, `ALL`, label `SAM Selection Tool`, key group `W`, glyph `✳` |
| `icons.rs` | `scan` |
| `panels/toolbar.rs` | the `W` flyout: Object Selection, Quick Selection, Magic Wand, SAM Selection |
| `shell_commands.rs` | `tools.select.sam3Select` |
| `state/tool_options.rs` | `sam3_prompt` / `sam3_threshold` (0.5) / `sam3_refine` (2) / `sam3_individual`, plus `SampleAllLayers::sam3_select` |
| `tools/tool_reset.rs` | reset keys |
| `tools/retouch_ui.rs` | options bar (mode buttons, Sample All Layers, Prompt, Threshold, Refine, Individual Masks, Detect, hint) and `finish_sam3_selection` |
| `canvas/gesture.rs` | the release commits the gesture |
| `canvas/paint.rs` | marching-ants rectangle while dragging |
| `canvas/tool_feedback.rs` | mode badge |
| `canvas.rs` | press modifiers captured, so a selection keeps Add/Subtract when Shift/Alt is released |
| `input/hold_keys.rs` | Space repositions an in-progress drag |
| `app/commands.rs` | a made selection reveals View > Extras, as the other `select.*` commands do |

A click (or a sub-2 px nudge) sends `point`; anything larger sends `box` in document pixels. Both
carry `threshold`, `refine`, `individual`, `sampleAllLayers` and the effective `mode` (⇧⌥ intersects,
⇧ adds, ⌥ subtracts, otherwise the options bar). The Detect button sends the typed `prompt` instead.

Measured on this machine:

| Check | Result |
|---|---|
| `cargo test -p photocraft-ui-egui --lib -- canvas:: hold_keys:: tool_feedback marquee retouch_ui tool_reset sam_selection` | **1258 passed, 0 failed** (9 ignored) |
| `cargo test -p photocraft-ui-egui --lib` (whole crate) | **7510 passed, 1 failed** — the one failure is `menus::native_menu::tests::opening_a_document_changes_state_not_structure`, which fails the same way on the unmodified tree (7500 passed, 1 failed there) and passes on its own: a pre-existing order-dependent failure, unrelated to this tool |
| `cargo test -p photocraft-ui-egui --test display_names` | 4 passed |
| `sam_selection_sends_a_point_for_a_click_and_a_box_for_a_drag` | pass — point for a click, box `[5, 5, 55, 50]` for a drag, `intersect` under ⇧⌥ |
| `sam_selection_box_selects_the_disc_end_to_end` (real helper, `--ignored`) | **pass**, 34.9 s — the click through `tool_event` runs `select.sam3` and leaves a selection 40-160 px across |
| `sam_selection_strings_are_translated_in_japanese` | pass — `SAM Selection Tool` → `SAM選択ツール`, `Prompt`/`Detect`/`Individual Masks` and both status lines resolve from the embedded catalog |
| `sam3_options_bar_tests` (a real egui harness) | 2 passed — the bar renders with Prompt, Threshold, Refine, Individual Masks, Sample All Layers and Detect reachable by name, and another tool reports no bar; the Threshold and Refine fields are `value_field_named`, so the accessibility check sees their names |
| `cargo build -p photocraft-ui-egui` | clean, no warnings from the new code |

### The canvas path, measured through the real harness

`sam3_canvas_tests.rs` drives `canvas::document_area` with raw `egui` events (the shape
`alt_click_tests.rs` uses) on a 256x256 document with a green disc of radius 40 at (128, 128).

| Check | Result |
|---|---|
| `a_click_through_the_real_canvas_selects_the_disc` | **pass** — the press, move and release run `select.sam3` and leave a selection 40-160 px across |
| `a_modifier_held_at_the_press_still_counts_when_let_go_first` | **pass** — a ⇧ press whose key is let go before the button still adds (the outline never shrinks to a replacement) |
| `the_mode_badge_follows_the_held_modifiers` | pass — none / Add / Subtract / Intersect for none / ⇧ / ⌥ / ⇧⌥ |
| `sam3_options_bar_tests` | 3 passed — the bar renders with the four mode buttons, Sample All Layers, Prompt, Threshold, Refine, Individual Masks and Detect reachable by name; another tool reports no bar |

### The toolbar flyout, measured

`sam3_toolbar_tests.rs` renders the real toolbar in an `egui_kittest` harness.

| Check | Result |
|---|---|
| `the_w_slot_lists_sam_selection_last` | pass — the W slot is [Object Selection, Quick Selection, Magic Wand, SAM Selection] |
| `right_clicking_the_w_slot_opens_its_flyout` | pass — the flyout opens anchored on that button (`egui::Id::unique("tool-flyout")` holds its rect) |
| `a_click_on_a_flyout_row_picks_it` | pass — a click on the first row selects Object Selection and closes the flyout |
| `the_tool_has_its_shortcut_row` | pass — `tools.select.sam3Select`, key `W`, listed by Edit › Keyboard Shortcuts with the SAM Selection label |

The rendered flyout (`Harness::render` on the open flyout, 1800x900) reads, top to bottom:
`Object Selection Tool W`, `Quick Selection Tool W`, `Magic Wand Tool W`, `SAM Selection
Tool W` — SAM Selection last, as the slot's order says.
Two defects this measured and fixed:

1. **Alt released before the button dropped the mode.** `finish_release` passed
   `d.modifiers` (the press's modifiers *minus* the ones let go: `track_snapshot`), so a quick
   ⌥ click selected instead of subtracting. `canvas/gesture.rs:110` now reads the press as the
   marquees do — `alt: d.modifiers.alt && !d.released.alt` — so a modifier held at the press
   keeps its meaning when released first, the same rule the other selection tools follow.
2. **A typed prompt was ignored by click and drag.** `sam3_select_params` now sends the text
   prompt whenever one is typed (the engine's text path covers the whole document; a box beside it
   would be ignored), and point/box only when the prompt is empty.
Japanese rows were added to `craft-i18n/locales/ja/photocraft.tsv` (9 rows, one per new string);
`Threshold` and `Refine` already resolve from `locales/ja/common.tsv`, so they have no row of their
own — the same split the other apps use.

The release build is **not** part of this record: `target/release/photocraft.exe` was locked by a
running instance (PID 25208, started 23:29), so the release binary was left untouched and everything
above was measured on the dev profile.

---

## Corrections from revision 4

1. **Replaced ONNX + `ort` (Method 1) with Pure Rust + CUDA direct `.safetensors` inference via
   `candle` (Method 2)**:
   - Eliminated the `sam3-export` crate and the ~2.5 GB of intermediate `.onnx` files.
   - `sam3-select.exe` now loads `plugins/sam3/sam3.1_multiplex_hswq_convrot_int8.safetensors`
     directly via `safetensors` + `memmap2`, performs the 41-layer $H_{256}$ ConvRot un-rotation and
     4 key remaps in memory on the CUDA device, and executes `CLIPTextModel`, `ViTDet`, `SAM3Detector`,
     and `SAM31Tracker` directly on CUDA (`Device::new_cuda(0)`).
   - Avoids ONNX control-flow/dynamic-shape limitations and avoids external `cudnn` /
     `onnxruntime_providers_cuda.dll` version dependencies.

## Corrections from revision 3

1. **Checkpoint & architecture mismatch fixed (SAM 3 vs SAM 3.1 Multiplex)**: Revision 3 described
   `sam3_convrot_int8.safetensors` (1.29 GB, 2,360 tensors, 274 INT8 layers, 4 FPN scales
   `[4.0, 2.0, 1.0, 0.5]`, `scalp=1`, and `sam2_convs`). Direct measurement of
   `plugins/sam3/sam3.1_multiplex_hswq_convrot_int8.safetensors` (1.74 GB) shows it is **SAM 3.1
   Multiplex** (`1,672` tensors: `1,549` FP16, `41` INT8 Linear, `41` FP32 scales, `41` U8
   `comfy_quant` all with `groupsize=256`; 3 FPN scales `[4.0, 2.0, 1.0]` with `scalp=0`;
   `interactive_convs` instead of `sam2_convs`; `interactive_sam_prompt_encoder` /
   `interactive_sam_mask_decoder` and `interactivity_no_mem_embed`).
2. **Two missing weight-key remaps added**: Revision 3 listed only `tracker.model.*` and CLIP
   `in_proj`. Added the **37 Detector `SplitMHA` `.in_proj_{weight,bias}` splits** (`768×256` →
   three `256×256` projections) and the **`sam_mask_decoder.transformer.*` key rename**
   (`.mlp.lin1.` → `.mlp.0.`, `.mlp.lin2.` → `.mlp.2.`, `.norm_final_attn.` → `.norm_final.`).
3. **ViTDet token count corrected (`5,184`, not `5,185`)**: Revision 3 stated the trunk runs on
   `5,185` tokens (`5,184 + cls`). In `sam.py:282`, `pos[:, :1]` (`cls_pos`) is sliced off before
   adding `pos` to `x`; the trunk runs strictly on the `72×72 = 5,184` spatial tokens.
4. **ConvRot Hadamard rule pinned**: Revision 3 treated the ConvRot matrix as an unknown risk item.
   Pinned from `convert_sam31_convrot_int8.py:86-105`: it is the deterministic Kronecker power
   $H_{256} = H_4^{\otimes 4} / 16$ ($H_{256}^T = H_{256},\ H_{256}^2 = I_{256}$).
5. **Image normalization, CLIP pad token, and DETR decoder details pinned**: Recorded that
   `ViTDet.patch_embed` takes `[0.0, 1.0]` RGB without ImageNet mean/std subtraction; CLIP pads with
   `0` (`pad_with_end = False`) and passes `attention_mask`; DETR decoder runs on `201` tokens
   (`presence_token` + 200 queries) with `_compute_box_rpb` log-scaled bias and `norm2 → catext_norm →
   norm1 → norm3` order.
6. **Complete UI/Engine test gates and Japanese localization (`craft-i18n`) added**: Fixed the
   `doc.selection` type description (`Option<Surface>` combined via `combine_region` on `Region` at
   `smartselect_cmds.rs:105`), fixed the `ObjectSelection` vs `MagicWand` label table row, and added
   `crates/engine/src/shell_commands.rs` (`TOOLS`), `README.md:188-189`, `crates/ui-egui/src/state.rs:868,894`,
   `tool_shift_cycle_order_tests.rs`, `input/tool_key_default_tests.rs`, `tools/tool_reset.rs`, and
   `craft-i18n/locales/ja/photocraft.tsv` (`SAM選択ツール`).
