# SAM Selection(SAM 3.1)技術解説

SAM Selection ツール(`Tool::Sam3Select`、コマンド `select.sam3`)の全実装を解説する。SAM 3.1 Multiplex のオープン語彙検出と対話セグメンテーションを、**Pure Rust + CUDA(`candle-core` / `candle-nn`)** で直接実行する。ComfyUI も Python も実行時には一切使わない。ComfyUI の SAM 3.1 実装は数値リファレンス(開発時オラクル)としてのみ使われ、`plugins/sam3/data/*_reference.py` がそのコピーである(クリーンルーム)。

- 主要コミット: `395be2eff`(SAM Selection tool 追加)、`a4d67a60c`(日本語ラベルをツリー内に復帰)
- 主要ファイル: `plugins/sam3/src/`(推論ヘルパー `sam3-select`)、`crates/engine/src/cmd/sam3_select_cmds.rs`(コマンド `select.sam3`)、`crates/ui-egui/src/tools/retouch_ui.rs`(オプションバーとジェスチャ)
- 計画と検証の原典: `plugins/sam3/PLAN.md`(Revision 5、実測記録を含む)。本書は現状の main に合わせた要約である

---

## 1. 概要

- **モデル**: `sam3.1_multiplex_hswq_convrot_int8.safetensors`(1,738,165,347 バイト = 1.74 GB)。SAM 3.1 Multiplex チェックポイント(`image_model = "SAM31"`、`multiplex = True`)の HSWQ ConvRot INT8 パック。**ONNX 変換なし**で `safetensors` をメモリマップし、GPU 上で直接実行する。
- **推論バックエンド**: `candle-core` / `candle-nn` 0.11(`features = ["cuda"]`)。cublas / curand / cuda_runtime に直接リンクし、cuDNN や onnxruntime の DLL は不要。
- **プロンプト**: テキスト(`face:3, person` のカンマ区切り、`category:N` で検出上限)、ボックス、ポイント(1 点)。検出 → 精細化の 2 段階。
- **出力**: `[N, H, W]` バイナリマスク → ドキュメント座標の選択範囲(`doc.selection`)として合成。
- **常駐**: `--serve` モードでパック(1.74 GB、ConvRot 逆回転込み)を VRAM に保持し、画像フレームもキャッシュする。2 回目以降の選択はサブ秒。

## 2. 構成要素とデータフロー

```text
UI (W スロットの SAM Selection Tool)
  └─ ジェスチャ: クリック → point / ドラッグ → box / Detect ボタン → prompt
       └─ engine コマンド select.sam3        crates/engine/src/cmd/sam3_select_cmds.rs:606
            ├─ prompt_from(): ドキュメント座標 → 1008 グリッドへ変換 (:223)
            ├─ サンプリング: 対象レイヤー(または sampleAllLayers 合成)を
            │   インターリーブ 8bit RGB で一時ファイルへ (run_helper, :457)
            ├─ resident::apply() → sam3-select --serve (:427)
            │     └─ 使えないときは one-shot CLI(--verify-weights + --select-*)へ
            ├─ masks.bin(f32 LE) + masks.json(形状/coverage/boxes/scores)を読む
            └─ HelperMask(document 空間にトリム) → Region → combine_region (:163, :568)
```

wasm ビルドはヘルパーを起動できないため、常駐経路が `#[cfg(not(target_arch = "wasm32"))]` で閉じられ、one-shot も含めてデスクトップ専用である。

## 3. ビルド構成

`plugins/sam3/Cargo.toml`(スタンドアロンクレート、`[workspace]` 空テーブル):

```toml
candle-core = { version = "0.11", features = ["cuda"] }
candle-nn   = { version = "0.11", features = ["cuda"] }
memmap2 = "0.9"; safetensors = "0.8"; half = "2"
image = { version = "0.25", default-features = false, features = ["png", "jpeg"] }
regex = "1"   # CLIP BPE の事前トークン分割
```

release プロファイルは `opt-level = 3`、`lto = "thin"`、`codegen-units = 1`。MSVC(`vcvars64.bat`)の下でローカル CUDA ツールキットに直接リンクされ、`plugins/sam3/target/release/sam3-select.exe` を作る。

## 4. モデル: HSWQ ConvRot INT8 パック

`plugins/sam3/sam3.1_multiplex_hswq_convrot_int8.safetensors` の実測内訳(全 1,672 テンソル):

| 種別 | 数 | 内容 |
|---|---|---|
| FP16 | 1,549 | 32 ブロック ViTDet trunk、FPN 3 ネック(convs / interactive_convs / propagation_convs)、24 層 CLIP テキストエンコーダ、SAM31Tracker 全体 |
| INT8 `weight` | 41 | すべて `detector.*` 内の 2D `nn.Linear`(resizer、dot_prod_scoring、geometry_encoder、segmentation_head.out_proj、decoder/encoder) |
| FP32 `weight_scale` | 41 | `[out, 1]` の行スケール(`amax / 127.0`) |
| U8 `comfy_quant` | 41 | 67 バイトの JSON `{"format":"int8_tensorwise","convrot":true,"convrot_groupsize":256}`。groupsize 64 や量子化 Conv2d は 0 個 |

**ConvRot の逆回転は決定的**(`loader.rs:29` の `GROUP_SIZE = 256`): 回転行列は正規 regular Hadamard 行列 H₄(符号配置のみ異なる 4×4)の 4 回クロネッカー冪を √256 で割った H₂₅₆ で、H₂₅₆ᵀ = H₂₅₆、H₂₅₆² = I が成り立つため、ロード時の復元は GPU 上の 1 行列乗算で済む:

1. `deq = f32(q) × weight_scale`
2. `W = f16(reshape(deq.view(O, I/256, 256) · H₂₅₆))`

`build_hadamard()`(`loader.rs:450`)が H₂₅₆ を GPU 上で構築し、`Pack::load`(`loader.rs:116`)が 41 層を一括で非量子化・f16 アップロードする。量子化されていない Conv2d は 0 個なので、畳み込みはすべて元の FP16 のまま実行される。

## 5. キーのリマップ(ロード時に必須の 4 種)

チェックポイントは upstream のキー配置で保存されており、`remap_name()`(`loader.rs:94`)とローダの分割処理で 4 種の再マップを行う:

1. **`tracker.model.*` → `tracker.*`**: ファイルは `tracker.model.*` が 457 個、素の `tracker.*` は 0 個。`.model` セグメントを剥がす。
2. **CLIP 融合 QKV の分割**(24 モジュール、48 テンソル): `encoder.transformer.resblocks.{0..23}.attn.in_proj_weight`(`[3072, 1024]`)と bias(`[3072]`)を dim0 で 3 分割し、`.q_proj` / `.k_proj` / `.v_proj`(`[1024, 1024]` と `[1024]`)へ。
3. **Detector `SplitMHA` 融合 QKV の分割**(37 モジュール、74 テンソル): `geometry_encoder.encode.{0..2}.{self_attn, cross_attn_image}`(6)、`segmentation_head.cross_attend_prompt`(1)、`transformer.decoder.layers.{0..5}.{self_attn, cross_attn, ca_text}`(18)、`transformer.encoder.layers.{0..5}.{self_attn, cross_attn_image}`(12)。`[768, 256]` → 3 × `[256, 256]`。
4. **Tracker `TwoWayTransformer` のリネーム**: `sam_mask_decoder.transformer.*` を含むキーへ `.mlp.lin1.` → `.mlp.0.`、`.mlp.lin2.` → `.mlp.2.`、`.norm_final_attn.` → `.norm_final.`。

## 6. モデル構造(入力 1008×1008)

**前処理**: RGB を `[0.0, 1.0]` に正規化し、bilinear(`align_corners=False`)で 1008×1008 へ。**ImageNet の mean/std 正規化は使わない**(`ViTDet.patch_embed` が `Conv2d(3, 1024, k=14, s=14, bias=False)` で直接受ける)。ヘルパーはエンジンから渡されたインターリーブ RGB を CHW f32 に並べ替えてから bilinear で 1008 にし、f16 にキャストする(`main.rs:844` `load_raw_rgb`、`main.rs:874` `load_image_1008`)。

**ViTDet trunk**(`vitdet.rs`):

- `img_size=1008, patch_size=14, embed_dim=1024, depth=32, num_heads=16, mlp_ratio=4.625, window_size=24, global_att_blocks=(7,15,23,31), use_rope=True`。
- `patch_embed` → `[B, 1024, 72, 72]` → `72×72 = 5,184` トークン。**cls トークンは trunk に入らない**(保存 `pos_embed` は `[1, 577, 1024]` = cls + 24×24。空間グリッドを 4×4 タイリングして 72×72 にクロップし、`pos[:, 1:]` のみ使用、`cls_pos` は捨てる)。その後 `ln_pre`。
- 注意は 28 ブロックが 24×24 ウィンドウ(9 ウィンドウ × 576 トークン)、ブロック 7/15/23/31 がグローバル(5,184 トークン)。
- 位置エンコードは Flux 系 `EmbedND` の 2 軸 RoPE(`rope_2d`、head_dim 64): グローバルは `scale_pos = 24/72`、ウィンドウは `scale_pos = 1.0`、theta 10000。

**FPN ネック(SAM 3.1 multiplex のみの仕様)**: 3 スケール `[4.0, 2.0, 1.0]`、`scalp = 0`。SAM 3(multiplex=false)の 4 スケール `[4.0, 2.0, 1.0, 0.5]` と `sam2_convs` とは異なる。

- scale 4.0(288×288): `dconv_2x2_0`(1024→512, k2s2)→ GELU → `dconv_2x2_1`(512→256, k2s2)→ `conv_1x1` → `conv_3x3`(pad 1)
- scale 2.0(144×144): `dconv_2x2`(1024→512, k2s2)→ `conv_1x1` → `conv_3x3`
- scale 1.0(72×72): identity → `conv_1x1`(1024→256)→ `conv_3x3`

`convs.{0,1,2}` が検出用、`interactive_convs.{0,1,2}` が対話トラッカー用の特徴量(288/144/72)。`propagation_convs` は動画追跡用で静止画選択ではロードしない。

## 7. 検出(DETR)

`detector.rs`:

- **CLIP テキストエンコーダ**(`clip.rs` + `tokenizer.rs`): hidden 1024、24 層、16 ヘッド、`quick_gelu`(`x · sigmoid(1.702x)`)、vocab 49,408、max_position 32。special token は start 49406 / end 49407 / **pad 0(`pad_with_end = false`)**。attention_mask(有効トークン 1、末尾 pad 0)を CLIP の自己注意(因果マスクと併用)と Detector の両方に渡す。`language_backbone.resizer`(1024→256)が Detector へ投影する。
- **プロンプト構文**: 括弧を剥がし、カンマ区切りカテゴリ。`face:3` の `:N` で `max_detections = max(1, round(N))`(既定 1)。
- **位置エンコード**: `PositionEmbeddingSine(num_pos_feats=256, normalize=True, scale=2π)` を 72×72 レベルに適用。
- **GeometryEncoder**: 点プロンプトは `points_direct_project` + bilinear `grid_sample` + `points_pool_project` + sine pos + `label_embed`。ボックスは `boxes_direct_project` + `roi_align(roi_size=7, aligned=False)` + `boxes_pool_project`(`Conv2d 256→256, k=7`)。3 層の pre-norm EncoderLayer の後、`cls_embed` に対して `_run_geo_layer`(geo が `k = img + pos`、`v = img` に注意)を実行し、`[text_embeddings, geo_prompts, geo_cls]` を連結して `text_mask` を延長する。
- **TransformerEncoder**: 6 層の pre-norm(`self_attn` は `q = k = norm1(x) + pos, v = norm1(x)`、`cross_attn_image` が text_memory へ、FFN は ReLU)。
- **TransformerDecoder**: **201 トークン**(`presence_token` 1 + `query_embed` 200)上の 6 層 post-norm。LayerNorm の順序は `norm2 → catext_norm → norm1 → norm3`。`query_pos = ref_point_head(sineembed(ref_points))`(512→256、presence にはゼロを前置)。`_compute_box_rpb` は log スケール座標差 `sign(8d)·log₂(|8d|+1)/log₂8` を `boxRPB_embed_x/y`(2→256→8)に通し、`[B, 8, 201, 5184]` の cross_attn バイアスを作る。レイヤー 0..4 の後で反復ボックス精緻化 `ref_points = sigmoid(inv_sig(ref_points) + bbox_embed(norm(tgt)))`。
- **DotProductScoring**: `prompt_mlp`(2 局残差 MLP + LayerNorm)、マスク平均プール、`hs_proj` / `prompt_proj`、スケール `1/√256`、clamp ±12、`scores.sigmoid() > threshold` でフィルタ。
- **SegmentationHead**: `cross_attend_prompt`(`cross_attn_norm(memory)` が prompt に注意)→ `encoder_visual` を `seg_features[2]`(72×72)へ → 3 段 top-down PixelDecoder(`Conv2d 3×3 pad1` + `GroupNorm(8,256)` + ReLU、nearest 2× アップサンプル 72→144→288)→ `instance_seg_head`(256→256, k1)→ `MaskPredictor`(3 層 MLP `mask_embed` + `einsum("bqc,bchw->bqhw")`)で `[B, 200, 288, 288]` の粗マスク logits。

## 8. 対話セグメンテーション(SAM31Tracker)

`tracker.rs`:

- `forward_segment`: `interactive_convs.{0,1,2}` から `high_res = [288, 144]` と `backbone_feat = feat_72 + interactivity_no_mem_embed`(`[1,1,256]`)を作る。
- **プロンプトエンコーダ**: 点(label が有効なもののみ `point_embeddings`、box なしのときは `label = -1` の `not_a_point_embed` を 1 件パッド)、ボックス(`point_embeddings[2]` と `[3]`)、マスク logits(`1008→288` へ **`antialias=True`** の bilinear、`mask_downscaling` の stride-2 2×2 conv + `LayerNorm2d` で 72×72 へ。マスクなしは `no_mask_embed`)。位置は `pe_layer` の `positional_encoding_gaussian_matrix`(`[2, 128]`)。
- **マスクデコーダ**: `SAMMaskDecoder`(single object、`num_mask_tokens = 4`)。`SAMTwoWayTransformer`(深さ 2)→ `output_upscaling` + `conv_s1(feat_144)` + `conv_s0(feat_288)` で 288×288 へ → `output_hypernetworks_mlps.{0..3}` → `object_score_logits > 0` でゲート(負なら `-1024.0`)。`multimask_output = false` ではマスク index 0、`0 < num_pts ≤ 1` の multimask では index 1..3 の `argmax(ious)`。最後に 1008×1008 へ bilinear 補間。

**実測で固定した 2 つの挙動**(`nn.rs` のユニットテストが守る):

1. **`label = -1` のトークンは `not_a_point_embed` を足す前に座標エンコードをゼロにする**。素の座標エンコードに足すと object_score が 5.26 → 0.92 にずれた(マスクプロンプト ケース)。
2. **`1008→288` のマスクプロンプトリサイズは `antialias=True`**。カーネルは分離三角形: `scale = in/out`、`support = max(scale, 1)`、`center = (o + 0.5)·scale − 0.5`(0 でクランプ)、入力内のタップに重み `1 − |i − center| / support` を正規化。`nn::resample_bilinear_antialias` は 10 通りのサイズ対で `F.interpolate(antialias=True)` と 3.2e-07 まで一致する。無いと object_score が 4.54 になる。

## 9. 選択パイプライン(精細化とマスク組立)

`select.rs`(`PAD_FRAC = 0.1`、`SIZE = 1008`、`refine_mask()` `:28`、`union_masks()` `:112`)と `selection.rs`(`DEFAULT_THRESHOLD = 0.5`、`DEFAULT_REFINE = 2`、`SelectionStack` `:24`):

1. 検出ボックスを各辺 10% 膨張させ、`[0..W, 0..H]` にクランプして元画像と粗マスクからクロップ。
2. クロップを 1008×1008 にリサイズし、**trunk + `interactive_convs` はクロップごとに 1 回だけ**実行。
3. `interactive_sam_prompt_encoder` + `interactive_sam_mask_decoder` を `refine_iterations` 回だけ回す(軽い)。ボックス/点プロンプトの全画面経路では trunk を 1 回だけ実行して GPU テンソルをキャッシュし、デコーダのみ反復する(`segment_prompt_cached` / `detect_and_refine_cached`、`selection.rs:90` / `:144`)。
4. 粗マスクと精細マスクを `coarse | refined` で合成。`individual` 指定時はマスクごとに個別出力(`Selection::finish`)。
5. ボックスは元ピクセル空間で出力される(`orig_size` がボックスと粗マスクの両方をスケール)。`select.rs` はモデル dtype を明示し、画像クロップは f32 でリサイズしてから f16 にキャストする(参照の `common_upscale(...).to(dtype)` と同じ順序)。

## 10. ヘルパーのモードと CLI

`sam3-select.exe` の全モード(`usage()`、`main.rs:15`):

| モード | 引数 | 用途 |
|---|---|---|
| トークン検査 | `--tokens <text>` | 32 トークン ID と attention mask を表示 |
| プロンプト解析 | `--parse <text>` | カテゴリと検出上限を表示 |
| 重み検証 | `--verify-weights` | パックを開き、ConvRot 層を非量子化して報告(`--dump-dequant <out>` で統計 JSON) |
| 検出 | `--detect-image <png> --detect-prompt <text> --detect-out <bin>` | DETR 検出のみ |
| セグメント | `--segment-image <png> --segment-box x1,y1,x2,y2 / --segment-point x,y / --segment-mask <f32> --segment-out <bin>` | 対話セグメンテーション(点は multimask 有効) |
| 選択 | `--select-prompt <text> / --select-box <4 値> / --select-point <2 値>` + `--select-raw <path> --select-raw-size <w>x<h>` + `--select-threshold / --select-refine / --select-individual / --select-out <path>` | エンジンが使う統合入口。`--select-raw` は PNG ではなくインターリーブ 8bit RGB(PNG コーデックを両側で排除) |
| 常駐 | `--serve [--model <safetensors>] [--data <dir>]` | stdin/stdout の JSON-lines ワーカー |

共通: `--model <pack>`(既定は実行ファイル隣の `PACK_FILENAME`)、`--data <dir>`(`vocab.json` と `merges.txt` のあるディレクトリ、既定 `<exe folder>/data`)。デバイスは `default_device()`(`loader.rs:483`)が `Device::new_cuda(0)` を選び、GPU がなければ CPU に落ちる。

**出力形式**: `masks.bin` はマスクごとの f32(LE)を連結したもの、`masks.json` は `{"masks": [形状...], "coverage": [...], "boxes": [...], "scores": [...], "threshold", "refine_iterations", "individual"}`。エンジンは shapes を読んで f32 列を `[H, W]` ごとに切り出す。

## 11. 常駐ワーカー(--serve)

- **要求**(`ServeRequest`、`main.rs:916`): `{"warmup":true}`(何もせず即応答)または `{"raw":"<path>","width":W,"height":H,"prompt":"..."|"box_xyxy":[4]|"point_xy":[2],"threshold":0.5,"refine":2,"individual":false,"out":"<path>"}`。`raw` は `W×H×3` バイトのインターリーブ RGB で、長さ不一致はエラー。
- **フレームキャッシュ**: `fnv1a_hash()`(`main.rs:942`)が生バイト + 寸法の 64bit FNV-1a ダイジェストを作り、同一画像なら `CachedFrame`(`native` f32 CHW と `frame` 1008 f16)を再利用する(`serve_one`、`main.rs:963`)。別画像なら再構築する。
- **応答**: stdout へ 1 行の JSON。stderr は診断専用。
- **エンジン側**(`crates/engine/src/cmd/sam3_select_cmds.rs:261` の `resident` モジュール): スロットは `OnceLock<Mutex<Option<Worker>>>`(`:380`)。`matches()` は exe・pack・data_dir の 3 点一致を見る。`REPLY_TIMEOUT = 180 秒`(`:273`、初回はパックロード + ConvRot 非量子化を含む)。stdout は専用スレッドが mpsc へ流し、要求側は `recv_timeout` で待つ。`Unavailable`(起動失敗・終了・タイムアウト・非 JSON prose)はワーカーを破棄し、既存ワーカーの死なら 1 回だけ再起動して再試行、起動直後の失敗は即諦める。`Failed`(健全なワーカーの `{"ok":false,"error":...}`)はワーカーを保持してエラーを直接報告。`Unavailable` に落ちたら one-shot CLI へフォールバックする。
- エンジンは現状 `{"warmup":true}` を自動送信しない。最初の選択はパックロード(実測 15.2 秒)を支払う。

## 12. エンジン統合(`select.sam3`)

コマンド仕様(`sam3_select_cmds.rs:606`):

```json
{"prompt":"text" | "box":[x,y,width,height] | "point":[x,y],
 "threshold":0..1=0.5, "refine":0..5=2, "individual":bool=false,
 "sampleAllLayers":bool=false, "mode":"replace|add|subtract|intersect"="replace"}
```

- **プロンプトのスケーリング**(`prompt_from`、`:223`): ドキュメント座標から bounds を引き、`(x − bounds.x0) × 1008/W` でモデルの 1008 グリッドへ変換する(ボックスは xyxy の 2 隅へ)。テキスト優先: `prompt` があれば全文書を対象に検出し、box/point は無視される(UI 側も同じ規則)。
- **サンプリング**: `with_doc_sampler`(smartselect_cmds から再利用)で対象レイヤー、または `sampleAllLayers` 時は合成を RGBA でサンプルし、8bit RGB へ落として一時ディレクトリの `image.rgb` に書く。推論後 `masks.bin` / `masks.json` を読み、使い終わった一時ディレクトリは削除する。
- **選択への合成**(`HelperMask`、`:163`; `sam3_selection`、`:568`): 各 `[H, W]` マスクは document 空間で内容 bbox にトリムされ、`Region`(GRAY8 の coverage)になって `combine_region(doc.selection, region, mode)` で置換/加算/減算/交差する。出力 JSON は `{"selected","changed","objects","bounds":[x,y,w,h]}`。マスクが空のときは `changed: false, objects: 0`。
- **ヘルパー/パック/データの解決**(`:22`〜`:159`): `CRAFT_SAM3_HELPER` / `CRAFT_SAM3_PACK` 環境変数が最優先(存在しないパスは明示エラー)。ヘルパーは Preferences › Plug-ins の追加フォルダ(有効時、直下と `sam3/` 直下)→ 実行ファイル隣の `plugins`・`plugins\sam3` → 実行ファイル直下 → 実行ファイルと CWD の各祖先の `plugins(\/sam3)(\/target\/release)`。パックはヘルパーの隣・`sam3/`・祖先の `plugins/sam3`・CWD 祖先。データディレクトリ(`vocab.json` のある場所)はパック/ヘルパーの親から推定。**見つからないエラーは探索した全候補を列挙**し、ソースにマシン固有パスは書かない。
- **one-shot フォールバック**(`run_helper`、`:457`): 常駐が `Unavailable` のとき、`--verify-weights`(パック検証)を付けた `--select-*` を 1 プロセスで実行する。stderr の最後の 4 行をエラーメッセージに載せる。

## 13. UI 統合

| 場所 | 内容 |
|---|---|
| `state/tool.rs:83` | `Tool::Sam3Select`(`Tool::ALL` の末尾に追加。既存ツール順を動かさない) |
| `shell_commands.rs:181` | `("tools.select.sam3Select", "SAM Selection Tool", Some("W"))` |
| `prefs.rs:1353` | `DEFAULT_TOOLBAR` の選択列 `[ObjectSelection, QuickSelection, MagicWand, Sam3Select]` |
| `panels/toolbar.rs:26` | W スロットのフライアウト同じ 4 項目(SAM は最後) |
| `icons.rs` | `scan` グリフ |
| `state/tool_options.rs:106-115` | `sam3_prompt: String`、`sam3_threshold: f32`(0.5)、`sam3_refine: u8`(2)、`sam3_individual: bool` + `SampleAllLayers::sam3_select` |
| `tools/retouch_ui.rs:689` | `sam3_select_params`: クリック(nudge < 2px)→ `point`、ドラッグ → `box`(document ピクセル)。**入力済みプロンプトがあれば point/box ではなく prompt を送る**(テキスト経路は全文書対象で、box は無視されるため)。修飾子は ⇧⌥ で intersect、⇧ で add、⌥ で subtract、なければオプションバーの mode |
| `tools/retouch_ui.rs:720` | `finish_sam3_selection`: リリースで `select.sam3` を実行 |
| `tools/retouch_ui.rs:1315, 1394` | オプションバー(`sam3_select_options_bar`): モードボタン 4 つ、Sample All Layers、Prompt(プレースホルダ `e.g. face`)、Detect、Threshold、Refine、Individual Masks、ヒント行 |
| `canvas/gesture.rs:110` | リリース時にコミット。修飾子は「プレス時に押されていたか」で判定(`alt: d.modifiers.alt && !d.released.alt`)— 早期リリースでもモードを保持 |
| `canvas/paint.rs` | ドラッグ中の marching-ants 矩形 |
| `canvas/tool_feedback.rs` | モードバッジ(+/−/×) |
| `canvas.rs` / `input/hold_keys.rs` | プレス時の修飾子確保、Space でドラッグ中の再配置 |
| `app/commands.rs` | 選択ができたら View › Extras を表示(他の `select.*` と同じ) |

**実測で発見・修正した 2 つの欠陥**: (1) ボタンを離す前に Alt を離すとモードが落ちて選択に化けていた → プレス時基準に修正。(2) 入力済みプロンプトがクリック/ドラッグに無視されていた → プロンプト優先に修正。

**日本語ラベル**: `SAM Selection Tool` → `SAM選択ツール` 等の行は **`crates/ui-egui/src/i18n.rs` の `JA_TEXT_OVERRIDES` でツリー内に持つ**(`tr()` がカタログより先に読む)。固定した craft-i18n ピンにはこれらの行が無いため、カタログ非依存で日本語表示を保証する。翻訳テストは `retouch_ui::tests::sam_selection_strings_are_translated_in_japanese`。

## 14. 実測性能と精度

計測環境: RTX 5060 Ti、`plugins/sam3/data/test_face.png`(640×640 合成顔を 1008×1008 へ)、HSWQ ConvRot INT8 パック。

**速度**:

| 項目 | 値 |
|---|---|
| パックロード(スタック構築、VRAM へ非量子化込み) | 15.2 秒(one-off) |
| ViTDet backbone(1008 フレーム 1 回) | 12.3〜13.5 秒 |
| 対話セグメンター(プロンプト 1 件) | 123〜204 ms |
| テキスト選択の end-to-end(検出 + CLIP + 精細化 2 回) | 12.9 秒 |
| エンジン e2e(point、`--ignored` テスト) | 19.0 秒 |
| UI e2e(box、実ヘルパー、`--ignored` テスト) | 34.9 秒 |

**精度**(マスクは最終 1008×1008 logits、IoU は `logit > 0`):

| 段階 | 比較 | 結果 |
|---|---|---|
| 対話セグメンター(box / point / mask プロンプト) | Rust vs `tracker_reference.py` | mask cosine 0.9999990〜0.9999995、logit max abs 0.08〜0.23、IoU(>0) 0.9998〜0.99997 |
| `_refine_mask`(オラクルの粗マスクから) | 2 値マスク | IoU 0.999968(61,756 / 61,758 px) |
| box プロンプト(refine = 1) | Rust vs M2d マスクの 640 リサイズ | IoU 1.000000(61,568 px 両側) |
| テキスト `"face"` end-to-end | Rust vs オラクル + refine | IoU 0.999968、coverage 両側 0.150771、score 0.981934 / 0.981453 |
| `--select-raw`(インターリーブ RGB)vs PNG 経路 | 同一画像 | IoU 0.999968(2 px 差) |

差分はすべて f16 のノイズフロアに収まる。

## 15. テスト

| 場所 | 件数と内容 |
|---|---|
| `crates/engine/src/cmd/sam3_select_cmds.rs` の tests | 8 件(7 pass + 1 ignored)。マスクの bbox トリム、空マスク、Region カバレッジ、プロンプト検証、1008 スケーリング、ヘルパー候補、resident の要求行/応答パース(prose 拒否を含む)、および実ヘルパー + 実パックを使う end-to-end(`a_point_selects_the_disc_under_it_end_to_end`、`CRAFT_SAM3_HELPER` / `CRAFT_SAM3_PACK` 必須で ignored) |
| `crates/ui-egui/src/sam3_toolbar_tests.rs` | 4 件。W スロットの 4 項目と順序、フライアウトのアンカー、行クリック、Edit › Keyboard Shortcuts の行 |
| `crates/ui-egui/src/sam3_options_bar_tests.rs` | 3 件。実 egui ハーネスでモードボタン 4 つ + Sample All Layers + Prompt + Threshold + Refine + Individual Masks + Detect が名前で到達可能、他ツールにバーが無いこと |
| `crates/ui-egui/src/sam3_canvas_tests.rs` | 実キャンバス経路(生 egui イベント): クリックで円が選択される、早期リリースでも ⇧ が加算になる、モードバッジ追従 |
| `retouch_ui::tests::sam_selection_strings_are_translated_in_japanese` | 日本語ラベル(ツリー内 `JA_TEXT_OVERRIDES` 由来) |

M4 の記録: UI 結線の回帰一式 `canvas:: hold_keys:: tool_feedback marquee retouch_ui tool_reset sam_selection` で **1258 passed / 0 failed**。

## 16. デプロイ

1. `plugins/sam3` で `cargo build --release` → `plugins/sam3/target/release/sam3-select.exe`。
2. ヘルパーを探せる場所に置く(§12 の探索順)。開発中は `CRAFT_SAM3_HELPER` と `CRAFT_SAM3_PACK` で明示するのが最速。
3. パック 1.74 GB はヘルパーの隣か `plugins/sam3/`。`vocab.json` / `merges.txt` は `plugins/sam3/data/`(パックやヘルパーの隣の `data/` でも可)。
4. CUDA ツールキット(cublas / curand / cuda_runtime)にリンクするため MSVC 環境が必要。**cuDNN は不要**(candle が直接使う)。GPU が無いマシンでは CPU に落ちる(速度は未計測)。
5. Preferences › Plug-ins の追加フォルダを使う場合の設定キーは LaMa と共通(`"plugIns": {"useAdditionalPluginsFolder", "additionalPluginsFolder"}`)。

## 17. 制限と設計判断の記録

- **クリーンルーム**: ComfyUI のコードは実行時に一切使わない。`data/*_reference.py` は数値検証専用のオラクル。
- **静止画専用**: `propagation_convs`(動画追跡用)はロードしない。
- **入力は 1008×1008 固定**: 高解像度キャンバスは全書類をスケーリングして解析する(部分領域解析は計画上の拡張項目で、未実装)。
- **最初の選択はパックロード 15.2 秒を支払う**: エンジンは `{"warmup":true}` を自動送信しない。アイドル中の先行起動は可能(常駐プロトコルで対応済み)。
- **VRAM 使用量は未計測**: パック 1.74 GB に f16 トランク + 41 層の非量子化が乗る。
- **1 スロット**: 常駐ワーカーはプロセス全体で 1 個。exe / pack / data_dir の不一致で即作り直す。
- **タイムアウト**: 180 秒(`REPLY_TIMEOUT`)。初回のパックロードを含む。
- ONNX + `ort` 構成(PLAN Revision 4 まで)は廃止: 動的制御流の困難、約 2.5 GB の中間 `.onnx`、外部 cuDNN / provider DLL のバージョン依存をすべて排除した(Revision 5 の決定)。
