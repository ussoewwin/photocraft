# Rust版 ComfyUI コア作成計画書（第一版）
## — PhotoCraft ネイティブ Generative Fill 統合エンジン —

* **策定日:** 2026年11月19日
* **対象ディレクトリ:** `plugins/ComfyUI/`
* **ステータス:** 計画策定（第一版）
* **開発言語 / ランタイム:** Pure Rust + CUDA (`candle` / カスタム CUDA GEMM / C++ FFI)
* **依存環境:** 外部 Python 不要、外部 PyTorch ランタイム不要、完全スタンドアロン構成

---

## 1. 理念と究極目標 (Goal & Philosophy)

### 1.1. 「単なる画像生成」の完全な否定
Adobe Photoshop の **Generative Fill（生成塗りつぶし）** をローカル環境・オープンソースで実現するにあたり、単なる「プロンプトから絵を出すだけ（Text-to-Image）」の機能は実務において一切の価値を持たない。

本プロジェクトの絶対目標は、**「キャンバス上の既存の絵・コンテキスト（光源の方向・強度、陰影、パース、被写界深度、テクスチャ、画風）を深く解析・理解した上で、選択範囲に対して一切の破綻なく溶け込ませるシームレスな文脈認識インペイント（Generative Inpainting）」** の達成にある。

### 1.2. 2026年11月における最低達成基準モデル
旧世代の SD1.5 や SDXL は、CLIP ベースの英語 77 トークン制限、看板・ラベル・ポスター等の日本語文字描画の破綻、および解像度・ディテールの表現力不足により、2026年現在の実務基準では論外である。
本コアが第一級でサポートする対象モデルは、以下を最低条件とする：

1. **Moss-Image**:
   * 日本語テキスト・漢字のネイティブ描画能力を備え、デザイン・イラスト実務における看板やテキスト要素を破綻させずにインペイント可能。
2. **Qwen-Image 2.1**:
   * 強力なマルチモーダル（VLM / LLM）基盤を持ち、後付けのガウスぼかしや単純マスク補間ではない、**「事前学習レベルでコンテキスト理解とマスク領域補完が緊密に統合されたネイティブインペイント能力」** を保有。

---

## 2. アーキテクチャ設計 (Architectural Alignment)

### 2.1. PhotoCraft 実績ヘルパーパターンの完全継承
PhotoCraft の内部 WASM フィルター機構（`crates/plugins`）は、セキュリティサンドボックス仕様のため GPU や VRAM、巨大モデルに直接アクセスできない。
したがって、すでに PhotoCraft リポジトリ内で実証されている **`plugins/lama-cleaner` および `plugins/sam3` と同一の独立ヘルパークレート方式** を採用する。

```
plugins/
  ├── lama-cleaner/       # ONNX Runtime / CUDA による LaMa インペイント
  ├── sam3/               # candle + CUDA + HSWQ ConvRot INT8 による SAM 3.1
  └── ComfyUI/            # ★ 本プロジェクト（Rust版 ComfyUI コア）
        ├── Cargo.toml    # 独立クレート（[workspace] 空テーブル、メインビルド非汚染）
        ├── PLAN.md       # 本計画書
        ├── src/
        │    ├── main.rs  # ヘルパー CLI / IPC 常駐エントリ
        │    ├── graph/   # DAG 実行エンジン・不変キャッシュ
        │    ├── models/  # Qwen-Image / Moss-Image DiT 実装
        │    └── ops/     # HSWQ ConvRot GEMM / テンソル演算
        └── models/       # HSWQ safetensors 格納ディレクトリ
```

### 2.2. SAM 3.1（`plugins/sam3`）との直接連携ライン
PhotoCraft 内で「SAM 3.1 による高精度な対象物選択」が行われた後、その選択マスクとキャンバスのコンテキスト画像が、同一のメモリ空間・データ構造思想を持つ本 ComfyUI コアへ直接受け渡され、1 クリックで生成塗りつぶしが完結するパイプラインを確立する。

---

## 3. 核となる競争力：HSWQ 技術の完全統合

巨大なパラメータを持つ Qwen-Image 2.1 や Moss-Image クラスの最先端モデルを、一般のデスクトップ環境（VRAM 16GB〜24GB、またはそれ以下）でミリ秒単位・低遅延で駆動させるための最大の武器として、**HSWQ（Hybrid-Sensitivity-Weighted-Quantization）** をネイティブ統合する。

### 3.1. HSWQ ConvRot INT8 / NVFP4 の優位性
* **単純量子化の完全排除:**
  均一な INT8/FP8 化で生じるアテンションの崩壊や、潜在空間の軌跡分岐（bifurcation）を排除。
* **潜在軌跡影響度ハイブリッド & SVD×RMS 感度解析:**
  非線形な誤差増幅を捉え、影響度の高い層を FP16 で保護、または最適なアダマール回転 $H_{256}$（ConvRot）を適用。
* **実証済みアセットの即時投入:**
  すでに Hugging Face 上で公開・実証されている `Qwen-Image (Inpainting) ConvRot INT8` パックおよび `sam3.1_multiplex_hswq_convrot_int8.safetensors` のフォーマット仕様をそのまま Rust からロードする。

### 3.2. Pure Rust + CUDA ロード機構
* `candle-core` / `candle-nn` (`features = ["cuda"]`)、または Nunchaku 系の最適化 CUDA カーネルを Rust から直接バインド。
* Python インタプリタ、PyTorch の巨大ランタイム、CUDA プロバイダの複雑な依存関係を一切介さず、単一の静的リンク／軽量バイナリから GPU VRAM へ直接重みをマップする。

---

## 4. インペイント処理パイプラインの詳細設計

Photoshop Generative Fill と同等以上の品質を叩き出すため、以下の 4 ステージ・パイプラインを Rust コア内に実装する。

```
[ PhotoCraft: キャンバス画像 ＋ 選択マスク ＋ プロンプト ]
                            │
                            ▼
 1. コンテキスト分解・前処理 (Context Decomposition)
    ├─ (A) グローバルコンテキスト画像:
    │      キャンバス全体を 512x512 等にリサイズ。
    │      Vision Encoder（SigLIP / Qwen-VL）に入力し、環境光・パース・画風トークンを抽出。
    └─ (B) ローカルターゲット画像 ＋ マスク:
           選択範囲の外側にパディングを持たせた原寸領域をクロップ。
           解像度アライメント（64の倍数）調整を行い、VAE エンコード。
                            │
                            ▼
 2. デノイズ・推論 (DiT Inpaint Sampling)
    ・HSWQ ConvRot INT8 Qwen-Image / Moss-Image DiT を実行。
    ・グローバルトークン ＋ ユーザープロンプトによるクロスアテンション制御。
    ・非マスク領域の潜在変数を各ステップで整合維持。
                            │
                            ▼
 3. VAE デコード ＆ 逆投影 (Decoding & Reprojection)
    ・生成潜在変数をフル解像度 RGB バッファへデコード。
    ・元のキャンバス座標系へ逆変形配置。
                            │
                            ▼
 4. マスク境界シームレス合成 (Seamless Edge Blending)
    ・PhotoCraft L0/L2 レイヤーのアルゴリズムを活用。
    ・ポアソン画像合成（Poisson Blending）、周波数分離マッチング、フェザリング。
                            │
                            ▼
[ 出力: PhotoCraft の非破壊 Generative バリエーションレイヤー ]
```

---

## 5. 段階的実装ロードマップ (Milestones)

### Phase 1: スタンドアロン Rust 推論エンジンの開通（CLI 検証）
* `plugins/ComfyUI/Cargo.toml` の立ち上げ。
* HSWQ ConvRot INT8 形式の Qwen-Image / Moss-Image の重みを `safetensors` から直接 GPU VRAM へロードするローダーの実装。
* コマンドライン引数（`--image`, `--mask`, `--prompt`, `--output`）を受け取り、スタンドアロンで 1 枚のインペイント結果を出力・検証。

### Phase 2: PhotoCraft エンジン（`crates/engine`）との結合
* PhotoCraft のコマンド体系への登録（例: `paint.generativeFill`）。
* UI（`crates/ui-egui`）にプロンプト入力バーおよび進捗インジケータを配置。
* ヘルパープロセスとの IPC（標準入出力 JSON ストリーム、または共有メモリ）による高速な画像・マスク転送。

### Phase 3: 非破壊バリエーションレイヤーと完全統合
* PhotoCraft ドキュメントモデル（`crates/doc`）への `LayerKind::Generative` 実装。
* 1 回の生成で 3〜4 パターンのバリエーションを保持し、プロパティパネルで即座に切り替え可能な UI の完成。

---

## 6. まとめ
本計画は、単なるツールの焼き直しではなく、**「最新の漢字・日本語認識モデル（Moss-Image / Qwen-Image 2.1）」×「独自技術 HSWQ ConvRot INT8 による極限の省メモリ・高速化」×「PhotoCraft のネイティブ Rust 画像処理パイプライン」** が三位一体となって初めて成立する、唯一無二のローカル Generative Fill システムの構築を目指すものである。


---

# 追補（第二版）— 具体化計画

* **策定日:** 2026年11月27日
* **位置づけ:** 第一版（§1〜§6）は変更せず、本追補で実装単位まで具体化する。以降の番号は §7 から。
* **方針:** `plugins/sam3` / `plugins/lama-cleaner` で実装・検証済みの構造を雛形とし、新規に発明する部分を最小にする。未計測の数値・構造は「G0 で実測確定」とし、確定前に値を固定しない。
* **対象モデルの確定（2026年11月27日）:** Qwen-Image 2.1 のパックは未入手のため、**当面の実装対象は Moss-Image のみ**とする。Qwen-Image は入手後の後続フェーズ（§18.9）。第一版 §1.2 の「Qwen-Image 2.1 を最低条件とする」は、この時点では未達成の将来項目として扱う。
* **インペイント方針の確定（2026年11月27日）:** ComfyUI 標準に Moss 専用のインペイント経路は存在しないため、**インペイント部分（`inpaint/`・G5）は後日オーナーが選定する ComfyUI カスタムノードのアルゴリズムを Rust へ応用・移植する前提**とする。ローダー・VAE・TE・DiT・FLOW サンプラ基盤（G1〜G4）は ComfyUI 標準実装ベースで先行実装可能。
* **実測の反映:** 参照パック（`Moody-Moss-Mix-V4.1_00001__clean_convrot_int8.safetensors`・`flux2-vae.safetensors`・`Qwen3.5-9B-..._hswq_convrot_int8.safetensors`）と ComfyUI 参照実装から確定した事実は §18 に集約した。§8〜§17 の記述も §18 の確定事項に合わせて整合済みである。

---

## 7. 計測済みの前提（このリポジトリで確認済みの事実）

| 項目 | 事実 | 出典 |
|---|---|---|
| ヘルパー方式 | 独立クレート（`[workspace]` 空テーブル）、`candle-core/candle-nn 0.11 + cuda`、`safetensors 0.8`、`memmap2`、`half`、`image(png,jpeg)`、`serde_json`、`ureq`。release は `opt-level=3, lto="thin", codegen-units=1` | `plugins/sam3/Cargo.toml` |
| ヘルパー構成 | `lib.rs` + `main.rs` + `loader.rs`（ConvRot 逆回転・dequant・キー再マップ）+ `nn.rs`（共通層）+ モデル別モジュール | `plugins/sam3/src/*` |
| 常駐ワーカー | `--serve`：stdin/stdout の JSON 1 行 = 1 リクエスト／レスポンス。stderr は診断のみ。画像は `raw`（RGB 8bit のファイルパス）＋`width/height` で受け渡し、結果は `out` のファイルへ。同一画像は FNV-1a ハッシュでフレームキャッシュ | `md/sam3-select.md` §11 |
| エンジン側ワーカー管理 | `OnceLock<Mutex<Option<Worker>>>`、`REPLY_TIMEOUT=180s`、`Unavailable`（再起動 1 回→ワンショット CLI へフォールバック）と `Failed`（ワーカー維持・エラー報告）の 2 区分 | `crates/engine-core/src/cmd/sam3_select_cmds.rs` |
| ヘルパー探索順 | 環境変数 `CRAFT_*_HELPER` → 環境設定「追加プラグインフォルダ」→ exe 隣の `plugins` → 祖先ディレクトリの `plugins/<name>[/target/release]`。見つからない場合は探索した全候補を列挙してエラー | 同上 |
| コマンド登録先 | fork コマンドは `crates/engine-core`（engine 分割後）。登録は `install/command_ids.txt`・`install.rs`・`cmd.rs`・`prefs.rs` | 既存 `sam3_select_cmds.rs` / `lama_cleaner.rs` |
| ドキュメント層 | レイヤー内容は `crates/doc/src/lib.rs:658` の `enum LayerContent` | `crates/doc` |
| 選択→生成の入力 | SAM 選択結果は `Region`（GRAY8 カバレッジ）として `doc.selection` に入る。生成塗りつぶしはこの選択をマスクとして直接読める | `sam3_select_cmds.rs`（`combine_region`） |
| 配置 | HSWQ ConvRot INT8 パックは `plugins/<name>/models/` 配下に置き、`.gitkeep` のみコミット（モデル本体は Git 管理外） | `plugins/sam3/models/` |

---

## 8. クレート構成（確定案）

```
plugins/ComfyUI/                       # パッケージ名: comfy-fill（バイナリ: comfy-fill[.exe]）
├── Cargo.toml                         # [workspace] 空。依存は sam3 と同一バージョン系で揃える
├── PLAN.md
├── models/.gitkeep
├── data/                              # トークナイザ語彙等（モデル非依存の小ファイルのみ）
└── src/
    ├── lib.rs
    ├── main.rs                        # CLI + --serve（sam3 の main.rs と同一規約）
    ├── protocol.rs                    # Request/Response/Event 型（serde）。engine 側と文面を共有
    ├── loader.rs                      # sam3 の ConvRot dequant / 逆回転を移植（コピー元を明記）
    ├── nn.rs                          # Linear / RMSNorm / RoPE / Attention 共通層
    ├── sampler/
    │    ├── mod.rs                    # Sampler trait
    │    ├── flow_match.rs             # Flow-matching Euler（既定）
    │    └── schedule.rs               # sigma schedule（shift 付き）
    ├── models/
    │    ├── moss_image/               # ★当面の実装対象（構成は §18.2）
    │    │    ├── dit.rs               # 42 層 単一ストリーム DiT
    │    │    ├── text_encoder.rs      # Qwen3.5-9B（テキスト経路のみ）
    │    │    └── vae.rs               # Flux2 系 VAE（128ch / 16 倍）
    │    └── qwen_image/               # 未着手（パック入手後。§18.9）
    ├── inpaint/                       # ★カスタムノード応用前提（差し替え可能な戦略構造）
    │    ├── strategy.rs               # InpaintStrategy trait（前処理・ステップフック・後処理ステッチの 3 フック）
    │    ├── baseline.rs               # ベースライン実装（KSamplerX0Inpaint + DifferentialDiffusion）
    │    ├── crop.rs                   # 選択 bbox + padding → 64 の倍数アライン、座標逆写像
    │    ├── mask.rs                   # マスクの膨張・フェザー、潜在解像度へのダウンサンプル
    │    └── blend.rs                  # 画素空間ステッチ・境界ブレンド（カスタムノード方式を移植）
    ├── graph/
    │    ├── mod.rs                    # 固定 DAG（Encode→Denoise→Decode→Blend）の実行器
    │    └── cache.rs                  # ノード出力の内容ハッシュキャッシュ
    └── tests/
```

設計判断:

1. **ComfyUI のワークフロー JSON ローダーは作らない。** 「Rust 版 ComfyUI コア」とは、ComfyUI の DAG 実行モデル（ノード出力のキャッシュ・再実行最小化）とノード実装を Rust に移植したものを指す。実行グラフは Generative Fill 用に固定した 4 ノードとし、ユーザー定義グラフは範囲外。
2. `graph/cache.rs` のキー: `(ノード種別, 入力テンソルの内容ハッシュ, パラメータ)`。同一画像・同一プロンプトでシードだけ変えた再生成では、Encode 系をキャッシュヒットさせ Denoise から再実行する（バリエーション 3〜4 枚生成の高速化の根拠）。
3. `loader.rs` は sam3 の実装をコピーして起点とし、テンソル名の再マップ規則だけモデル別に分離する。共通化のための共有クレート化は、2 つ目の利用者（本クレート）が動いた時点で別 PR として判断する（先に抽象化しない）。

---

## 9. ヘルパープロトコル仕様（engine ⇔ comfy-fill）

sam3 の `--serve` を踏襲し、生成が数十秒〜に及ぶため **進捗とキャンセル**を追加する。

### 9.1 CLI（検証・フォールバック用）

```
comfy-fill --verify-weights --model <pack>          # パックを開き dequant 結果の統計を報告（G1 検証）
comfy-fill --vae-roundtrip --image <png> --out <png>    # VAE encode→decode 往復（G2 検証）
comfy-fill --encode-test  --model <pack> --prompt <t>   # テキスト埋め込みのみ（G3 検証）
comfy-fill --image <png> --mask <png> --prompt <t> --seed <n> --steps <n> --out <png>  # 一括実行（G5 検証）
comfy-fill --serve                                  # 常駐ワーカー（G6 検証）
```

### 9.2 `--serve`（JSON 1 行／メッセージ）

リクエスト:

```json
{"op":"fill","id":"<job-id>",
 "raw":"<path to RGB8 canvas>","width":W,"height":H,
 "mask":"<path to GRAY8 mask, W*H bytes>",
 "prompt":"...","negative":"",
 "model":"moss-image","seed":123,"steps":20,"cfg":4.0,
 "variants":4,"padding":128,"feather":8,
 "out_dir":"<dir>"}
{"op":"cancel","id":"<job-id>"}
{"op":"warmup"}
```

レスポンス（stdout、1 行 1 JSON。`id` で対応づけ）:

```json
{"id":"..","event":"progress","stage":"encode|denoise|decode|blend","step":7,"of":20,"variant":2}
{"id":"..","ok":true,"variants":[{"png":"<dir>/v0.png","seed":123},...],"bounds":[x,y,w,h],"ms":{...}}
{"id":"..","ok":false,"error":"..."}
```

規約（sam3 と同一）: stderr は診断専用／画像・マスクはファイル受け渡し（巨大 JSON を流さない）／同一入力は内容ハッシュでキャッシュ／`ok:false` は「ワーカー健全」の意で、エンジン側はワーカーを維持する。起動失敗・タイムアウト・非 JSON 出力は `Unavailable` としてワーカーを破棄する。

追加規約:

* `progress` は 1 ステップごとに出す。エンジン側は `REPLY_TIMEOUT`（総時間）ではなく **最後のイベントからの無通信時間**で打ち切る（初回のパック読み込みは最大 180s を維持）。
* `cancel` を受けたら、現在のステップ完了直後に中断し `{"ok":false,"error":"cancelled"}` を返す。
* 出力は常に「バリエーション別の PNG（RGBA、ブレンド済み）＋合成用の生成領域 bbox」。ドキュメント座標への配置はエンジン側が行う。

---

## 10. モデル実装の具体化

### 10.1 共通方針

* 重みは `safetensors` を `memmap2` で開き、ConvRot INT8 は読み込み時に dequant（BF16/FP16）して GPU へ載せる（または層ごとにオンデマンド展開する）方式で開始する（sam3 と同一。正しさ優先）。INT8 のままの専用 GEMM カーネルは G9 の最適化項目とし、ここでは先取りしない。
* 数値検証の基準は **参照実装の出力**。同一入力・同一シードの参照出力（中間テンソル含む）を `tests/fixtures/` に保存し、Rust 側が許容誤差内で一致することを各マイルストーンの合格条件とする（sam3 の「reference bit-for-bit」方針を踏襲）。

### 10.2 G0 で実測確定する項目（§18 進捗反映済み）

| 確定項目 | 状態 | 成果物／参照先 |
|---|---|---|
| Moss-Image DiT 構造（42 層・dim 3072・situ・shared 変調・4 軸 RoPE） | **確定済み** | §18.1・§18.2 |
| Qwen3.5-9B TE の層構成・サイズ・トークナイザ資材 | **確定済み**（実運用ファイルの stock/Heretic 確認のみ残） | §18.3・§18.10 Q2 |
| Flux2 VAE の所在・テンソル構成・`bn` 正規化式 | **確定済み**（`flux2-vae.safetensors`） | §18.4 |
| FLOW サンプラ数式・インペイント方針（カスタムノード応用前提） | **確定済み** | §18.5 |
| HSWQ ConvRot パックの対象層・H256/H64 互換・キー再マップ規則 | **確定済み** | §18.1・§18.7 |
| candle 側の Qwen3.5 線形アテンション対応有無・必要 VRAM 実機計測・参照中間テンソル保存 | **未実施（G0 残タスク）** | `md/comfy-fill-model-structure.md`・`tests/fixtures/` |

### 10.3 サンプラ

* 既定は flow-matching の Euler ステップ（`MOSSImage`: `shift = 5.0`、`multiplier = 1.0`、§18.5 の確定式を使用）。
* インペイント時のステップ制御は `InpaintStrategy` のステップフック経由で実行する。ベースラインでは `KSamplerX0Inpaint`（非マスク領域の潜在置換）と `DifferentialDiffusion`（進捗閾値による段階マスク）を提供し、後日選定するカスタムノード固有のステップ制御をそのまま差し込める構成とする。

### 10.4 インペイント前処理・後処理（`inpaint/` — ベースライン仕様）

※本節はベースライン構成であり、実務向けの具体的なクロップ・マスク処理・ステッチングは後日選定する ComfyUI カスタムノードのアルゴリズムを `InpaintStrategy` 実装として移植・置換する（§18.5）。

1. 選択マスクの bbox を取得 → `padding`（既定 128px。文脈を取り込む余白）を加えてキャンバス内にクランプ → 幅・高さを 64 の倍数（潜在 16 倍縮小の 4 倍）に切り上げ（キャンバス端では反射パディングで補う）。
2. 生成解像度の上限（VRAM 依存）を超える場合は縮小、逆に小領域で潜在トークン数が不足する場合は拡大して生成し、後段で元解像度へ戻す。伸縮率は結果 JSON に含める。
3. 結果は bbox 座標へ逆写像。ブレンドは画素空間で **マスク領域およびフェザー帯のみ**に適用し、マスク外のピクセルは 1 ビットも変更しない（検証項目 V3）。

---

## 11. エンジン統合（`crates/engine-core`）

### 11.1 追加ファイルと登録（sam3 と同一手順）

| 作業 | 場所 |
|---|---|
| コマンド実装 | `crates/engine-core/src/cmd/generative_fill_cmds.rs`（新規。`sam3_select_cmds.rs` の `resident` モジュール構造を踏襲） |
| コマンド ID 登録 | `install/command_ids.txt` に `paint.generativeFill`・`paint.generativeFill.cancel` |
| 登録呼び出し | `install.rs` / `cmd.rs` の fork コマンド登録箇所へ追加 |
| ヘルパー探索 | `CRAFT_COMFY_HELPER` / `CRAFT_COMFY_PACK` 環境変数 → 追加プラグインフォルダ → exe 隣 `plugins/ComfyUI` → 祖先の `plugins/ComfyUI[/target/release]`（探索候補は全件をエラーに列挙） |
| 環境設定の既定 | `prefs.rs`（モデル選択・既定ステップ数・バリエーション数）。保存互換のため追加フィールドは既定値付き |

### 11.2 コマンド仕様

```json
{"prompt":"text","negative":"", "model":"moss-image",
 "variants":1..4, "seed":null|int, "steps":int, "cfg":float,
 "padding":int, "feather":int, "sampleAllLayers":true,
 "target":"new-layer|variants-group"}
```

* 入力: アクティブ選択（`doc.selection`）をマスク、`sampleAllLayers` に従いレイヤーまたは合成結果を RGB8 でサンプリング（`with_doc_sampler` を再利用）。選択が空ならエラー（`bad params`）。
* 出力: 生成結果を **新規ラスターレイヤー（バリエーション別・生成領域 bbox 限定・RGBA）** として挿入し、元レイヤーは変更しない（非破壊）。
* History: 1 回の生成 = 1 ステップ。キャンセル時は History に何も残さない。
* 長時間処理: ジョブとして非同期実行し、`progress` イベントをジョブ進捗へ中継（`jobs.rs` の既存機構を利用）。UI スレッドを塞がない。
* エラー分類は sam3 と同一（`Unavailable` → 再起動 1 回 → ワンショット CLI／`Failed` → ワーカー維持して報告）。

### 11.3 自動化インターフェース

AGENTS.md の方針に従い、すべての操作をエンジンコマンド経由とし、`photocraft-cli` / MCP からも同一コマンドで GUI なし実行できるようにする（パネルはコマンド ID のディスパッチのみ）。

---

## 12. ドキュメント層の方針（段階分け）

第一版 Phase 3 の `LayerKind::Generative` は、`crates/doc` と保存形式（`crates/format`、PSD/pcraft 書き出し）に触れるため、**上流との衝突コストと互換性リスクが最も高い**。以下の 2 段階に分ける。

* **Phase 3a（形式変更なし・先行実施）:** バリエーションを「グループ内の複数ラスターレイヤー（うち 1 枚のみ可視）」で表現し、バリエーション情報（シード・プロンプト・モデル名）はレイヤー名／既存のメタデータ領域に保持する。既存フォーマットで保存・再読込でき、他アプリでもレイヤーとして開ける。切り替えは可視レイヤーの入れ替えコマンド（`paint.generativeFill.selectVariant`）として実装。
* **Phase 3b（判断ゲート）:** 3a の運用で不足が確認できた場合のみ `LayerContent` へ専用バリアントを追加する。着手条件は「上流の `LayerContent`／format の変更状況を fetch して実測し、衝突範囲と PSD 書き出し時の代替表現（ラスター化）を確定済みであること」。判断はオーナー確認の上で行う。

---

## 13. UI（`crates/ui-egui`）

* ツール／コンテキストバー: プロンプト入力、モデル選択、バリエーション数、「生成」「キャンセル」ボタン。SAM 選択ツールの選択確定後に同じバーから生成へ進める（§2.2 の 1 クリック導線）。
* 進捗: ステージ名（encode / denoise / decode / blend）とステップ数のインジケータ。
* バリエーション切り替え: プロパティパネルのサムネイル列（3a では可視レイヤー切替コマンドを呼ぶだけ）。
* 文言は翻訳キー（英語がキー）を通し、日本語は `crates/ui-egui/i18n/fork/ja/*.tsv` に追加（fork 側カタログ運用）。
* キーボード操作・アクセシビリティ（役割・名前・状態・ライブリージョン通知）は AGENTS.md ルール 21 に従う。色のみで状態を示さない。
* 色・サイズ・フォントはテーマトークンから取得。

---

## 14. マイルストーン（各項目に実測の合格条件を付す）

| ID | 内容 | 合格条件（すべて実測で確認） |
|---|---|---|
| **G0** | 残る未確定事項の解消（§10.2・§18.8） | 構造・VAE・数式は確定済み。残タスク: (1) TE 運用ファイルの確定、(2) candle 線形アテンション有無と VRAM 実測、(3) 参照実装の中間テンソル保存 |
| **G1** | クレート雛形＋`--verify-weights` | Moss の DiT パック（185 量子化層・H256/H64）と TE パック（`language_model` のみ）を開き未解決キー 0 を報告。`cargo build --release` 完走（`MAX_JOBS` 等のユーザー指定は無改変） |
| **G2** | VAE 往復 | `flux2-vae.safetensors` で `--vae-roundtrip` の PSNR が参照実装の同条件値と許容差内 |
| **G3** | テキストエンコーダ一致 | Qwen3.5-9B テキスト経路で、参照実装の最終隠れ状態と許容誤差内で一致（日本語・漢字看板文言を含む） |
| **G4** | DiT 単体・全ステップ生成一致 | 同一入力・同一シードで 1 ステップ出力（G4a）およびテキスト生成の最終画像（G4b）が参照と数値・視覚の両面で一致 |
| **G5** | インペイント統合（カスタムノード応用） | 後日選定する ComfyUI カスタムノードのインペイント方式を `InpaintStrategy` として移植。`--image/--mask/--prompt` で参照ノード出力と一致＋マスク外不変（V3） |
| **G6** | `--serve` ＋進捗＋キャンセル | 2 回目以降はパック読み込みなしで開始。キャンセルが次ステップ境界で効く。同一画像でシード違いはエンコーダ再実行なし |
| **G7** | エンジンコマンド統合 | `paint.generativeFill` を `photocraft-cli` から実行し、選択範囲に新規レイヤーが入る。History 1 ステップ。ヘルパー未配置時は探索候補を全列挙したエラー |
| **G8** | UI 統合（Phase 3a） | 実機ハーネスで操作 → スクリーンショットとアクセシビリティツリーを確認。キーボードのみで生成・キャンセル可能 |
| **G9** | 性能最適化（任意） | INT8 のまま GEMM する経路を、G4 の数値一致を維持したまま追加。前後の所要時間と VRAM を計測して記録 |

各マイルストーンは「実装 → 実測 → `md/` に記録」で完結とし、実測値のない完了報告をしない。

---

## 15. 検証項目（回帰ガード）

| ID | 項目 | 方法 |
|---|---|---|
| V1 | パック読み込みの再現性 | `--verify-weights` の出力（層数・未解決キー）をテストで固定 |
| V2 | 参照一致 | `tests/fixtures/` の参照テンソルとの許容誤差内一致（G3・G4） |
| V3 | マスク外不変 | 出力と入力のマスク外画素を比較し差分 0（関係比較で記述） |
| V4 | 座標逆写像 | bbox・padding・64 アライン・縮小時の往復で、境界が 1px 以内にずれない |
| V5 | キャンセル | キャンセル後に History 無変更、ワーカー健全で次のジョブが通る |
| V6 | プロトコル | 非 JSON 出力・タイムアウト・異常終了で `Unavailable` に分類されワンショットへ退避 |
| V7 | 探索 | 環境変数・追加フォルダ・祖先探索の各経路でヘルパー／パックを解決（マシン固有パスをソースに書かない） |
| V8 | fork 保全 | 上流マージ後も `generative_fill_cmds.rs` と登録が残ることを `photocraft_sync` の fork 機能監査に追加（監査対象リストへ追記） |

---

## 16. リスクと未決事項（オーナー判断が必要なもの）

| # | 内容 | 現状 |
|---|---|---|
| R1 | Moss-Image の HSWQ ConvRot INT8 パック（Moody-Moss-Mix）の配布形態・ライセンス。Qwen-Image 2.1 は未入手 | Moss は手元に実物あり（§18.1）。Qwen は入手後に再評価。再配布可否は G0 前に確定する |
| R2 | 16GB / 24GB VRAM での成立可否 | G0 の実測で確定。成立しない構成は「非対応」と明示し、黙って劣化させない |
| R3 | `LayerContent` 追加（Phase 3b）の是非 | §12 の判断ゲートで決定。それまで `crates/doc` / `crates/format` は変更しない |
| R4 | 上流が同種機能（生成塗りつぶし）を導入した場合の重複 | 週次の上流同期時に `upstream/main` を確認し、重複が出た場合は fork 側を上流へ寄せるか判断する |
| R5 | ヘルパーがユーザー環境の CUDA に依存 | LaMa（`md/lama-cleaner-cuda.md`）と同様、未検出時は明示エラー。CPU 経路は提供しない（実用速度に達しないため） |
| R6 | 生成物の出自表示（C2PA 等） | 範囲外。必要性はオーナー判断 |
| R7 | インペイント用カスタムノードの選定（G5 着手前提） | 後日オーナーが選定・提示する方針（§18.5）。G1〜G4 の先行実装はブロックしない |

---

## 17. 直近の着手順序

1. **G0 残タスク完了**（TE 実運用ファイルの確認、candle 線形アテンション対応確認、参照中間テンソルの保存）。
2. **G1〜G4 を先行実装**（カスタムノード選定を待たずに進行可能）：
   * G1（`plugins/ComfyUI/` 雛形＋`--verify-weights`）
   * G2（`flux2-vae.safetensors` 往復一致）
   * G3（Qwen3.5-9B テキスト経路一致）
   * G4a → G4b（Moss-Image DiT 1 ステップ→全ステップ生成一致。ここまで `crates/engine-core` は無変更）
3. **インペイント用カスタムノード選定・解析 → G5**（選定ノードの方式を `InpaintStrategy` へ移植・参照一致確認）。
4. **G6**（`--serve` 常駐ワーカー・進捗・キャンセル・キャッシュ）。
5. **G7 → G8**（`crates/engine-core`・`crates/ui-egui` 統合、Phase 3a）。
6. 各段階で `md/comfy-fill-*.md` に実測を記録し、`photocraft_sync` の fork 監査対象へ登録。


---

## 18. Moss-Image 実測反映（参照パックと ComfyUI 参照実装に基づく確定事項）

調査対象（読み取りのみ。ファイルの変更・モデルの実行はしていない）:

* DiT: `D:\USERFILES\ComfyUI\ComfyUI\models\unet\Moody-Moss-Mix-V4.1_00001__clean_convrot_int8.safetensors`（5,088,552,570 bytes、748 テンソル。同名の非量子化版 `…__clean.safetensors` は 10,164,663,992 bytes）
* 参照実装: `ComfyUI\comfy\ldm\moss_image\model.py`、`comfy\supported_models.py`（`MOSSImage`）、`comfy\model_base.py`（`MOSSImage`）、`comfy\text_encoders\moss_image.py`、`comfy_extras\nodes_moss_image.py`、`comfy\samplers.py`（`KSamplerX0Inpaint`）
* テキストエンコーダ候補: `models\clip\Qwen3.5-9B-The-Defiant-Fable-Uncensored-Heretic-NEO-IMATRIX-MAX-MTP_hswq_convrot_int8.safetensors`（9,665,636,078 bytes、1,513 テンソル）

### 18.1 パックの実測内容（DiT）

| 項目 | 実測値 |
|---|---|
| dtype 内訳 | I8 185 / U8 185 / F32 185 / BF16 193（量子化 185 層ぶんの `weight`・`comfy_quant`・`weight_scale` が組、BF16 はバイアスとノルム） |
| 量子化メタデータ | ヘッダ `_quantization_metadata`。185 層中 184 層が `int8_tensorwise` + `convrot:true` + `convrot_groupsize:256`、**1 層（`video_embedder`）のみ `convrot_groupsize:64`**（入力次元が 128 で 256 に満たないため） |
| `weight_scale` の形 | `[out, 1]`（出力行ごと。例: qkv は `[9216,1]`）。名称は tensorwise だが行ごとのスケールである点に注意。dequant 手順は sam3 の `loader.rs` と同様に G1 で突き合わせて確定 |
| `comfy_quant` | 各層 66〜67 バイトの JSON（例 `{"format":"int8_tensorwise","convrot":true,"convrot_groupsize":256}`） |
| 層数 | `layers.0`〜`layers.41` の **42 層**、`text_embedder.blocks` 2 層、`time_embedding.0 / .2` |
| 次元 | `dim=3072`、`num_heads=24`（head_dim 128、`q_norm/k_norm` が `[128]`）、`ffn_dim=8192` |
| 融合 | 本パックは `proj.qkv`（`[9216,3072]` = q/k/v 融合）、`proj.o`、`proj.ffn_up_gate`（`[16384,3072]` = ffn_dim×2 の融合）、`proj.ffn_down`（`[3072,8192]`）。ComfyUI 側の `fused_qkv=True` 経路に相当 |
| 層ごとの変調重み | **なし**。変調は全層共通の `shared_block_modulation`（`[18432,3072]` = 6×3072）1 つだけ。層ごとに持つのは `norm1/norm2`（`[3072]`）と `q_norm/k_norm` のみ |
| 入出力 | `video_embedder`: `[3072,128]`（潜在 128ch → dim）、`final_linear_video`: `[128,3072]`（バイアスなし）、`final_norm_video`、`video_head_modulation`: `[6144,3072]`（2×dim） |
| テキスト入力 | `text_embedder.proj`: `[3072,4096]`（テキスト特徴 4096 次元 → dim）＋ 2 層の `TextBlock`（`ffn_gate/ffn_up/ffn_down` 分離） |

### 18.2 DiT の計算仕様（`comfy/ldm/moss_image/model.py` より）

1. **単一ストリーム**: クロスアテンションは無い。シーケンスは `[画像トークン (H×W) | テキストトークン (L) | 参照画像トークン(任意)]` の連結で、全層が全トークンに自己アテンションをかける。出力は先頭 `H*W` トークンのみ使う。
2. **画像トークン化**: 潜在 `[B,128,H,W]` を `flatten(2).transpose(1,2)` して `video_embedder` に通すだけ（パッチ化なし。128ch がすでに 2×2 パッチ化された Flux2 系潜在のため）。
3. **時間埋め込み**: `timestep_embedding(t, 256)` → `Linear(256,3072)` → SiLU → `Linear(3072,3072)` → `vec`。`shared_block_modulation(vec)` を 6 分割し、各層で **`mod[0]=shift1, mod[1]=scale1, mod[2]=gate1, mod[3]=shift2, mod[4]=scale2, mod[5]=gate2`** として全層同一値を使う。
4. **ブロック**: `y = RMSNorm(x)·w·(1+scale)+shift` → 融合 qkv → `q_norm/k_norm`（head 単位 RMSNorm）→ RoPE → アテンション → `proj.o` → `x += gate·out`。続いて同様に norm2 + 変調 → `ffn_up_gate` → **situ** → `ffn_down` → ゲート付き残差。
5. **situ 活性化**: `out = (4·tanh(gate/4)·sigmoid(gate)) · (25·tanh(hidden/25))`。`ffn_up_gate` の出力を `[hidden | gate]` の前後半に分ける。ソフトキャップ付きゲート活性化であり、SwiGLU の流用は不可。
6. **RoPE**: 4 軸 `(r, f, h, w)`、各軸 `head_dim/4 = 32` 次元、**split-half** 方式、`theta=10000`（NTK 係数は既定 1.0）。画像トークンは `f = テキスト長`、`h/w` は中心を 0 とした座標、テキストトークンは `f = 0..L-1`。参照画像は `r = i+1`。
7. **最終段**: `final_linear_video( RMSNorm(x[:, :H·W])·w·(1+head_scale)+head_shift )`、`head_shift/head_scale` は `video_head_modulation(vec)` を 2 分割したもの。
8. **サンプリング設定**（`supported_models.MOSSImage`）: モデル種別は FLOW、`shift=5.0`、`multiplier=1.0`、推論 dtype は bfloat16/float32、潜在形式は `Flux2`（128ch、空間縮小率 16）。

### 18.3 テキストエンコーダ（Qwen3.5-9B）— 本計画で最大の実装負荷

実測（パックのテンソル形状）:

| 項目 | 値 |
|---|---|
| 言語モデル | `model.language_model.layers` 32 層、hidden 4096、MLP 中間 12288、語彙 248,320（`embed_tokens`・`lm_head` とも I8 `[248320,4096]`） |
| 層の種類 | **24 層が `linear_attn`（DeltaNet 系の線形アテンション）、8 層が通常の `self_attn`**。層 `i` は `(i+1) % 4 == 0` のとき full attention（参照実装 `_qwen35_layer_types`） |
| linear_attn | `in_proj_qkv [8192,4096]`、`in_proj_z [4096,4096]`、`in_proj_a/b [32,4096]`、`conv1d [8192,1,4]`（カーネル 4、depthwise）、`A_log [32]`、`dt_bias [32]`、`norm.weight [128]`、`out_proj [4096,4096]` |
| full attention | `q_proj [8192,4096]`（出力ゲート付きで 2 倍）、`k_proj/v_proj [1024,4096]`、`o_proj [4096,4096]`、`q_norm/k_norm [256]`（head_dim 256、16 ヘッド、KV 4 ヘッド、部分 RoPE 係数 0.25、mrope） |
| 付属 | `model.visual`（ViT 27 層。参照画像をテキスト側に見せるときのみ必要）、`mtp.*`（投機デコード用。テキスト埋め込みには不要）、`lm_head`（不要） |
| Moss が使う出力 | 「最後の隠れ状態（`layer="last"`）」を DiT の `context`（`text_dim=4096`）として渡す。プロンプトは `<|im_start|>user\n{text}<|im_end|>\n<|im_start|>assistant\n<think>\n`（開いた think ブロック）に整形し、テキストは `strip()` する |

実装上の含意:

* **候補 Rust ライブラリ（candle）に Qwen3.5 の線形アテンションは無い想定**で、`linear_attn` の再帰（ゲート付き delta rule）、depthwise causal conv1d、`RMSNormGated`、部分 RoPE/mrope を自前実装する。プロンプトは数十〜数百トークンなので、まずは **時間方向に逐次計算する素直な実装**で正しさを取り、チャンク並列化は後回しにする。`candle` 側の対応状況は G0 で確認し、無い場合のみ自前カーネルを書く（先に前提にしない）。
* パックは `model.language_model.` / `model.visual.` / `lm_head.` というプレフィックス。参照実装は読み込み時に `model.language_model.→model.`、`model.visual.→visual.`、`lm_head.→model.lm_head.` へ置換している。Rust 側も同じ再マップ表を `loader.rs` に持つ。
* 手元の候補は「Heretic」系ファインチューン（abliterated）。参照実装のコメントは **stock Qwen3.5-9B** を前提としている。埋め込みの分布が変わるため、品質基準は stock 版との出力比較で確認する（G3）。stock 版が手元にあるかは未確認。
* テキストだけを使う経路では `visual` と `mtp` と `lm_head` を読み込まない。実測: パック全体 **9.665 GB** のうち `model.language_model.*` のみは **7.944 GB**（1.721 GB 削減）。さらに `language_model` 内の量子化層はすべて `convrot_groupsize: 256`（64 と 16 は `model.visual` のみ）。
* **オンデマンド `embed_tokens` 展開（重要最適化）:** `model.language_model.embed_tokens.weight` は I8 `[248320, 4096]` + `weight_scale` F32 `[248320, 1]` + ConvRot H256 で格納されている。全 248,320 行を一括 dequant すると BF16 で約 2.03 GB を消費するため、**入力トークン ID に対応する行だけ mmap から切り出して `(q_row * s_row) @ H_256` で展開**する。これにより埋め込みテーブル常駐を 0 に抑える。
* トークナイザ資材は ComfyUI 標準 `comfy/text_encoders/qwen35_tokenizer/` の `vocab.json`（6.97 MB）・`merges.txt`（3.60 MB）・`tokenizer_config.json`（17 KB）を参照する。

### 18.4 VAE（所在確認済み）

* Moss の潜在は **128ch・空間縮小 16 倍**（`Flux2` 潜在形式）。これは Flux2 系 VAE（32ch 潜在を 2×2 パッチ化して 128ch にしたもの）に対応する。
* 手元の `models\vae\ae.safetensors`（335,304,388 bytes）をヘッダで確認したところ、`decoder.conv_in.weight = [512,16,3,3]`、`encoder.conv_out.weight = [32,512,3,3]` で **16ch 潜在（Flux1 系）**。Moss の 128ch 潜在とは**一致しない**。
* `models` 配下を「flux2 を含むファイル名」「vae を含む .safetensors」で名前検索した範囲では、Flux2 系 VAE は見つかっていない（名前検索のみ。中身の全走査はしていない）。
* → **所在確認済み（2026年11月27日）**: `D:\USERFILES\StableDiffusion\models\VAE\flux2-vae.safetensors`（336,211,292 bytes、251 テンソル）。ヘッダ実測: `decoder.conv_in.weight=[512,32,3,3]`、`decoder.post_quant_conv.weight=[32,32,1,1]`、`encoder.conv_out.weight=[64,512,3,3]`（32ch の平均＋分散）、`encoder.quant_conv.weight=[64,64,1,1]`、**`bn.running_mean=[128]`**。32ch 潜在を 2×2 パッチ化した 128ch に対するバッチ正規化の統計が含まれており、Moss の潜在（128ch・16 倍縮小）と一致する。
* 同フォルダの他ファイルは対象外: `Ultra-flux1.vae` と `zImageBase_vae` は 16ch（Flux1 系）、`sdxl_vae`・`vae-mse-84000`・`klF8Anime2VAE` は 4ch、`Wan2_1_VAE_fp32` と `qwen_image_vae`（253,806,246 bytes）は別構造（Qwen-Image 実装時の候補。未解析）。
* Rust 側の実装要件（`comfy/ldm/models/autoencoder.py:185-254` より確定）:
  * テンソル内訳: 251 テンソル（F32 250 + I64 1 `bn.num_batches_tracked`）。`bn.running_mean [128]`・`bn.running_var [128]`・`bn_eps = 1e-4`。
  * **encode**: `encoder(x)` → `quant_conv`（64ch → mean 32ch）→ 2×2 パッチ化 `rearrange("... c (i 2) (j 2) -> ... (c 2 2) i j")`（128ch、1/16 解像度）→ BatchNorm 正規化 `z = (z - running_mean) / sqrt(running_var + 1e-4)`。
  * **decode**: 逆正規化 `z = z * sqrt(running_var + 1e-4) + running_mean` → 2×2 アンパッチ化 `rearrange("... (c 2 2) i j -> ... c (i 2) (j 2)")`（32ch、1/8 解像度）→ `post_quant_conv`（`[32,32,1,1]`）→ `decoder`。

### 18.5 インペイント方式（Moss に専用インペイント経路は無い）

ComfyUI の Moss-Image 実装にあるのは、(a) テキスト生成、(b) カラー／深度コントロール用 LoRA（`video_embedder` を拡張）、(c) 参照画像付き編集 LoRA（`TextEncodeMOSSImageEdit`、`index_timestep_zero_text_start` など）で、**インペイント用に学習された経路や重みは含まれない**。手元の `loras`/`inpaint`/`model_patches` を名前で検索しても Moss 用のものは無かった（`qwen_image_inpaint_diffsynth_controlnet` は Qwen 用）。

**ComfyUI 標準に Moss 専用のインペイントコードがあるか（2026年11月27日に確認した範囲）:**

* `comfy/ldm/moss_image/model.py`・`comfy/text_encoders/moss_image.py`・`comfy_extras/nodes_moss_image.py` に `inpaint` の語は無い。`mask` は「テキストトークンへのアテンション加算バイアス」（`context_attention_bias`）の意味でのみ使われ、画素や潜在のマスクではない。
* `model_base.MOSSImage.concat_cond` は `concat_latent_image` だけを読み、`concat_mask` / `denoise_mask` は読まない。しかも `video_embedder` の入力幅が 128 のままのパック（本パック）では `channels = 128 − 128 = 0` で `None` を返す。つまり標準の `InpaintModelConditioning`（`concat_latent_image` と `concat_mask` を条件に載せるノード）を Moss に使っても、**条件側の入力は捨てられ、効くのは `noise_mask`（サンプラ側の denoise-mask）だけ**になる。
* したがって Moss で動く標準のインペイントは、モデル非依存の次の 2 つだけ:
  1. `SetLatentNoiseMask` / `InpaintModelConditioning(noise_mask=True)` による **`KSamplerX0Inpaint`**（本節の式）。
  2. `comfy_extras/nodes_differential_diffusion.py` の **`DifferentialDiffusion`**（`model.set_model_denoise_mask_function` でマスクを差し替える）。式は `threshold = (現在の timestep − ts_to) / (ts_from − ts_to)`、`binary_mask = (denoise_mask >= threshold)`、`strength < 1` なら元マスクとブレンド。ステップが進むにつれて閾値が下がるため、羽根（階調）付きマスクが「弱いマスク部分ほど遅く生成領域に入る」段階的な境界になる。Moss でも使えるが、潜在マスクの粒度（16px）は変わらない。
* 結論: **Moss 専用のインペイント用学習済み重み・条件付けコードは標準に無い**。

**採用方針（2026年11月27日 確定 — カスタムノード応用前提）:**

* ベースラインとして標準の `KSamplerX0Inpaint` + `DifferentialDiffusion` 相当をサンプラへ実装しつつ、**実務品質のインペイント方式（クロップ・マスク前処理・条件付け・ステップ制御・ステッチング等）は、後日オーナーが選定・提示する ComfyUI カスタムノードのアルゴリズムを Rust へ移植・応用することを前提**とする。
* そのため、コード設計上は `inpaint/` と `sampler/` の境界を固定実装にせず、**インペイント戦略を差し替え可能な構造（前処理・ステップごとのマスク/潜在フック・後処理ステッチの 3 フック）**として分離しておく。
* これにより **G1〜G4（ローダー・VAE・TE・DiT 単体の一致検証）はカスタムノード選定を待たずに先行完了**でき、選定後に **G5（インペイント統合）** で対象カスタムノードのロジックを移植・検証する。

ベースラインとなる標準 **denoise-mask 方式**（`KSamplerX0Inpaint`）の式:

```
各ステップ（現在の sigma）で
  x      = x · mask + scale_latent_inpaint(x, sigma, noise, latent_image, mask) · (1 − mask)
  out    = model(x, sigma)
  out    = out · mask + latent_image · (1 − mask)
```

（`mask=1` が生成領域。`MOSSImage` は `BaseModel.scale_latent_inpaint` + `CONST` + `ModelSamplingDiscreteFlow`（`shift=5.0, multiplier=1.0`）をそのまま継承しているため、数式はすべて確定済み:
  * `sigma(t) = time_snr_shift(5.0, t) = 5.0 * t / (1.0 + 4.0 * t)`（`t ∈ (0, 1]`）
  * `scale_latent_inpaint(sigma, noise, latent_image) = sigma * noise + (1.0 - sigma) * latent_image`
  * `calculate_denoised(sigma, model_output, model_input) = model_input - model_output * sigma`）

**この方式固有の制約（設計に織り込む）:**

1. 潜在が 16 倍縮小のため、**マスクの粒度が 16px 単位**になる。選択境界の再現は潜在だけでは粗いので、最終合成は必ず画素空間で行う（§10.4 のブレンド、マスクの羽根は画素マスクをそのまま使う）。
2. 専用学習が無い以上、境界の整合は「元画像の再ノイズ置換」に依存する。品質が不足する場合の追加策を段階で用意する: (i) 生成後に合成画像を再エンコードして低 denoise の仕上げ 1〜2 ステップ（境界帯のみ）、(ii) パディングを広げて文脈量を増やす、(iii) 編集 LoRA 経路の評価（LoRA の入手が前提）。
3. 生成領域が小さい場合、潜在上では数トークンしかない。**小領域は周辺を含めて切り出し、拡大して生成 → 縮小して戻す**（§10.4 の縮小処理の逆方向。拡大率は結果 JSON に含める）。

### 18.6 メモリ設計（実測値と、そこから導ける範囲のみ）

* DiT: INT8 パック 5.09 GB、同モデルの BF16 版 10.16 GB（いずれも実ファイルサイズ）。dequant して BF16 常駐させると約 10 GB。
* テキストエンコーダ: パック全体 **9.665 GB**、`model.language_model.*` のみで **7.944 GB**（§18.3 実測）。さらに `embed_tokens`（I8 約 1.02 GB / BF16 約 2.03 GB）はオンデマンド行単位展開で常駐 0 に抑えるため、実効の量子化重み量は **約 6.92 GB**。
* 方針: **TE と DiT を同時に BF16 常駐させない**。段階実行とする: ① プロンプトを埋め込み（`[L,4096]` の小テンソル）にして内容ハッシュでキャッシュ → ② TE を解放 → ③ DiT をロードしてサンプリング → ④ VAE。プロンプト不変のシード違い生成では ① をスキップできる（§8 のグラフキャッシュ）。
* INT8 のまま GPU に置き、層ごとに BF16 へ展開して使う方式を一次案とする（常駐量を重みの INT8 サイズに抑えられる）。INT8 GEMM 直実行は G9。
* 16 GB / 24 GB で成立するかは G0 の実機計測で確定する。この節では数値を断定しない。

### 18.7 Rust 実装の分担（Moss 限定、改訂版）

| モジュール | 内容 | 参照元 |
|---|---|---|
| `loader.rs` | ConvRot INT8 の dequant（`sam3/src/loader.rs` の `build_hadamard(size)` は 4 の累乗すべてに対応済みのため、`H_256`・`H_64`・`H_16` をそのまま処理可能）、キー再マップ、`embed_tokens` の行単位オンデマンド展開 | `plugins/sam3/src/loader.rs`、`md/sam3-select.md` §4 |
| `moss_image/dit.rs` | §18.2 の DiT。situ、shared 変調、4 軸 RoPE、融合 qkv/ffn | `comfy/ldm/moss_image/model.py` |
| `moss_image/text_encoder.rs` | Qwen3.5-9B テキスト経路（線形アテンション 24 層＋通常アテンション 8 層） | `comfy/text_encoders/qwen35.py`、`moss_image.py` |
| `tokenizer.rs` | 語彙 248,320 の BPE（ComfyUI 標準 `qwen35_tokenizer/` の `vocab.json`・`merges.txt`・`tokenizer_config.json` を読む）とチャットテンプレート | `comfy/text_encoders/moss_image.py`、`qwen35_tokenizer/` |
| `moss_image/vae.rs` | Flux2 系 VAE の encode/decode、2×2 パッチ／アンパッチ、`bn.running_mean/var` 正規化 | `flux2-vae.safetensors`、`comfy/ldm/models/autoencoder.py` |
| `sampler/flow_match.rs` | FLOW、shift=5.0、Euler、ベースライン denoise-mask / DifferentialDiffusion ＋ カスタムノード応用用のステップフック | `comfy/samplers.py`、`model_sampling.py`、`nodes_differential_diffusion.py` |
| `inpaint/` | クロップ・マスク前処理・ステッチング（選定したカスタムノードの方式を差し込める戦略トレイト構成） | 後日選定するカスタムノード |

### 18.8 マイルストーン改訂（§14 の G0〜G4 を Moss 向けに具体化）

| ID | 内容 | 合格条件（実測） |
|---|---|---|
| **G0** | 残る未確定事項の解消 | (1) ~~Flux2 系 VAE の所在と `bn` 統計の適用式~~ → 確定済み（§18.4）、(2) ~~FLOW サンプラの数式と `loader.rs` の H64/H256 互換~~ → 確定済み（§18.5・§18.7）、(3) stock Qwen3.5-9B の有無と candle 側の線形アテンション対応有無の確認、(4) 参照実装でプロンプト 1 件・固定シードの **中間テンソル（TE 出力、DiT 1 ステップ出力、VAE デコード）を保存**（以降の数値一致の基準） |
| **G1** | `--verify-weights` | DiT パックの 185 量子化層すべてを dequant し、未解決キー 0、`convrot_groupsize` が 256 と 64 の 2 種のみであることを確認。TE パックは `language_model` 系のみ読み込み、読み込みバイト数を記録 |
| **G2** | VAE 往復 | `flux2-vae.safetensors` で encode→decode の PSNR を参照実装と比較 |
| **G3** | TE 一致 | 固定プロンプト群（日本語・漢字看板文言を含む）で、参照実装の最終隠れ状態と許容誤差内で一致 |
| **G4a** | DiT 1 ステップ一致 | 保存済みの入力潜在・`vec`・テキスト特徴で 1 ステップ出力が一致（situ・RoPE・shared 変調の各中間値も個別に一致） |
| **G4b** | 全ステップ生成 | 同一シード・同一ステップ数で最終画像が参照実装と数値・視覚の両面で一致（テキスト生成） |
| **G5** | インペイント統合（カスタムノード応用） | 後日選定する ComfyUI カスタムノードのインペイント方式を `inpaint/`・`sampler/` へ移植。`--image/--mask/--prompt` で出力し、参照ノード出力と一致＋マスク外不変（V3） |
| **G6〜G9** | §14 のとおり（モデルは Moss のみ） | — |

### 18.9 Qwen-Image（後続フェーズ）

* パック未入手のため設計・工数を確定しない。入手後に §10.2 と同じ手順（テンソル名・形状の機械的列挙 → `md/` に対応表）から着手する。
* 手元に `model_patches\qwen_image_inpaint_diffsynth_controlnet_hswq_convrot_int8.safetensors`（1,135,921,920 bytes）があり、Qwen 側には専用インペイント ControlNet が存在する。Qwen を実装する段階では、Moss の denoise-mask 方式ではなくこの経路を一次案として評価する（現時点では未解析）。
* `crate` 構成上は `models/qwen_image/` を空けておくだけとし、Moss の実装を Qwen に先回りして抽象化しない。

### 18.10 未決事項（オーナー確認が必要）

| # | 質問 |
|---|---|
| Q1 | （解決）Moss の VAE は `D:\USERFILES\StableDiffusion\models\VAE\flux2-vae.safetensors`（§18.4）。 |
| Q2 | テキストエンコーダは実運用でどのファイルを使っていますか（`clip` フォルダの Heretic 系か、別の stock Qwen3.5-9B か）。 |
| Q3 | 品質基準を「参照実装との数値一致」に置いてよいですか（Rust 版が ComfyUI と同じ絵を出すことを合格とする）。 |
| Q4 | （方針確定）Moss のインペイントは標準 denoise-mask をベースラインとし、後日選定する ComfyUI カスタムノードの方式を Rust へ移植・応用する（§18.5）。G1〜G4 はカスタムノード選定を待たず先行可能。 |
