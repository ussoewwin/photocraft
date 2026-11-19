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
