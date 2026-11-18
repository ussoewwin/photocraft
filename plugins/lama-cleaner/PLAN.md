# PhotoCraft `lama-cleaner` CUDA 高速化・常駐ワーカー化 実装計画書

**作成日**: 2026-11-15  
**対象ディレクトリ**:
- プラグイン本体: `plugins/lama-cleaner`
- エンジン連携部: `crates/engine/src/cmd/retouch_cmds/lama_cleaner.rs`
- モデルファイル: `plugins/big-lama.onnx` (208 MB)

---

## 1. 現状の実測ボトルネック分析

現行の `plugins/lama-cleaner.exe` を実測した結果、1ストロークあたり **合計約 12.0 秒** を要しており、内訳は以下の通りです。

| フェーズ | 現行の実測時間 | 原因 |
|---|---|---|
| **① プロセス起動 + ONNX モデルロード・グラフ最適化** | **約 10.6 秒**（毎回発生） | `crates/engine/src/cmd/retouch_cmds/lama_cleaner.rs` がストロークのたびに `lama-cleaner.exe` を単発起動して終了させており、毎回 208 MB の `big-lama.onnx` の読み込みと ONNX Runtime のグラフ最適化がゼロから走り直している。 |
| **② 512×512 LaMa 推論（FFC: Fast Fourier Convolution）** | **約 1.36 秒** | `plugins/lama-cleaner/Cargo.toml` で `ort` に GPU Execution Provider が指定されておらず、`Session::builder()` もデフォルトの CPU Execution Provider のみで動作している。 |
| **③ 一時 PNG エンコード/デコード I/O** | **約 0.05 秒** | エンジンと `lama-cleaner.exe` 間で一時 PNG ファイル（`lama_in.png`, `lama_mask.png`, `lama_out.png`）を経由している。 |
| **合計体感時間（1ストローク）** | **約 12.0 秒** | 実用上ストロークごとに10秒以上の待ち時間が発生。 |

---

## 2. 高速化アーキテクチャ方針（3本柱）

### 柱1: `ort` の CUDA Execution Provider（＋DirectML フォールバック）有効化
1. **`Cargo.toml` の Feature 構成（`ort-sys 2.0.0-rc.15` のプリビルド配布テーブルに完全準拠）**
   - `ort-sys 2.0.0-rc.15` の `build/download/dist.tsv` および `resolve.rs` において、`x86_64-pc-windows-msvc` 向けの CUDA プリビルドバイナリは `cuda13,tensorrt,nvrtx,directml` の組み合わせで配布されている。
   - `cuda` 単独指定では `resolve_dist()` の完全一致チェックで弾かれるため、`lax-feature-matching`（または `cuda`, `directml`, `preload-dylibs`）を有効化して確実に GPU 対応バイナリをリンクする：
     ```toml
     ort = { version = "2.0.0-rc.9", features = [
         "cuda",
         "directml",
         "preload-dylibs",
         "lax-feature-matching",
     ] }
     ```
2. **CUDA / cuDNN 動的ライブラリの自動プリロード**
   - `ort::ep::cuda::CUDA_DYLIBS`（`cudart64_13.dll`, `cublasLt64_13.dll`, `cublas64_13.dll`, `cufft64_12.dll`）および `CUDNN_DYLIBS`（`cudnn64_9.dll` ほか）を以下の優先順で自動探索し、`ort::ep::cuda::preload_dylibs` でプリロードする：
     1. 環境変数 `CUDA_PATH` / `CUDA_HOME`（例: `C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v13.2\bin`）
     2. 実行ファイルと同階層の `plugins/` ディレクトリ
     3. システム `PATH` 上の CUDA / cuDNN ランタイム
3. **Execution Provider 登録とオプション最適化**
   - `Session::builder()` に以下の優先順で Execution Provider を登録する：
     1. **`ort::ep::CUDA`**:
        - `with_conv_algorithm_search(ConvAlgorithmSearch::Heuristic)`: 初回推論時に数秒かかる `Exhaustive` ベンチマーク探索を回避し、cuDNN ヒューリスティックで即座に高速カーネルを選択。
        - `with_tf32(true)`: Ampere / Ada / Blackwell Tensor Core での TF32 高速演算を有効化。
     2. **`ort::ep::DirectML`**:
        - 万一 CUDA / cuDNN DLL が見つからない環境や非 NVIDIA GPU 環境でも、Windows 標準の DirectX 12 GPU（DirectML）で高速実行するフォールバック。
     3. **CPU（最終フォールバック）**
   - また、`onnxruntime_providers_shared.dll` および `onnxruntime_providers_cuda.dll` をビルド成果物から `plugins/`（`lama-cleaner.exe` の隣）へ配置する。

### 柱2: 常駐ワーカーモード（`--serve`）によるモデルロード時間の完全ゼロ化（10.6秒 → 0 ms）
1. **`lama-cleaner.exe --serve [--model <path>]` の追加**
   - 従来の単発 CLI（`lama-cleaner -i <in> -m <mask> -o <out> --model <model>`）との後方互換性を 100% 維持したまま、常駐モード `--serve` を追加する。
   - `--serve` 起動時に 1 回だけ `LamaModel::load` を実行して GPU VRAM 上に最適化済みセッションを保持し、標準入力（`stdin`）から 1 行 1 JSON リクエストを受け取り、標準出力（`stdout`）に 1 行 JSON レスポンスを返す：
     - リクエスト: `{"image":"<in_path>","mask":"<mask_path>","output":"<out_path>"}`
     - レスポンス: `{"ok":true,"infer_ms":42.1,"provider":"CUDA"}` または `{"ok":false,"error":"..."}`
2. **エンジン側（`crates/engine/src/cmd/retouch_cmds/lama_cleaner.rs`）のワーカープール保持**
   - `OnceLock<Mutex<Option<LamaWorker>>>` により、起動済みの `lama-cleaner.exe --serve` 子プロセス（`Child`, `BufWriter<ChildStdin>`, `BufReader<ChildStdout>`）を保持する。
   - 初回ストローク時（またはプロセス異常終了時）のみ子プロセスを起動し、2回目以降のストロークでは既に GPU 上にロード済みのワーカーへパイプ経由で即座にリクエストを送る。
   - もし `--serve` 非対応の旧バイナリやモックテスト用のダミースクリプトが渡された場合は、自動的に従来の単発 CLI 呼び出し（`-i ... -m ... -o ...`）へフォールバックすることで、既存の全ユニットテストとの互換性を完全に維持する。

### 柱3: 最適化済み ONNX グラフキャッシュと前処理/後処理の高速化
1. **最適化済み ONNX グラフのディスクキャッシュ**
   - `SessionBuilder::with_optimized_model_filepath` を用い、初回ロード時に最適化されたグラフ（`big-lama.opt.onnx`）をキャッシュすることで、アプリ再起動後の初回ロード時間自体も約 10.6 秒 → 約 0.5〜1.0 秒へ短縮する。
2. **512×512 テンソル変換と境界フェザリングの高速化**
   - `plugins/lama-cleaner/src/lib.rs` の `inpaint_512()` における入力テンソル生成・出力画像復元をスライス直接操作に最適化。
   - `inpaint()` の境界 3px フェザリング処理を整理し、CPU 側のオーバーヘッドを数ミリ秒以下に抑える。

---

## 3. 変更対象ファイル一覧

| ファイル | 変更内容 |
|---|---|
| `plugins/lama-cleaner/Cargo.toml` | `ort` に `features = ["cuda", "directml", "preload-dylibs", "lax-feature-matching"]` を追加、JSON 通信用に `serde`, `serde_json` を追加。 |
| `plugins/lama-cleaner/src/lib.rs` | `CUDA`（`Heuristic`, `tf32=true`）→ `DirectML` → `CPU` の Execution Provider 構成、CUDA/cuDNN DLL プリロード、最適化済みグラフキャッシュ、アクティブ EP 名の取得・ログ出力を実装。 |
| `plugins/lama-cleaner/src/main.rs` | 既存の単発 CLI に加え、`--serve` 常駐 JSON Lines ループモードを追加。 |
| `crates/engine/src/cmd/retouch_cmds/lama_cleaner.rs` | 常駐 `LamaWorker` プロセス再利用機構の実装（単発 CLI への安全なフォールバック付き）、既存テスト 6 件＋新規テストの維持。 |

---

## 4. マイルストーンと検証手順

### M1: `plugins/lama-cleaner` の CUDA / DirectML 対応と単体ベンチマーク
1. `plugins/lama-cleaner/Cargo.toml`, `src/lib.rs`, `src/main.rs` を更新。
2. MSVC 環境（`vcvars64.bat`）で `cargo build --release` を実行し、生成された `lama-cleaner.exe` とプロバイダ DLL（`onnxruntime_providers_*.dll`）を `plugins\` に配置。
3. `plugins\lama-cleaner.exe` を実行し：
   - 使用された Execution Provider（CUDA / DirectML）が表示されること
   - 512×512 推論時間が従来の `1.36s` から **数十ミリ秒（30〜80 ms）** に短縮されること
   - 出力画素（赤矩形のインペイント除去結果）が正常であることを実測確認。

### M2: `--serve` 常駐モードの実装と連続推論ベンチマーク
1. `lama-cleaner.exe --serve` で連続リクエストを送信し、2回目以降のリクエストが **モデルロード 0 ms・合計 50 ms 前後** で完了することを実測確認。

### M3: `crates/engine` 側の常駐ワーカー連携とテスト検証
1. `crates/engine/src/cmd/retouch_cmds/lama_cleaner.rs` に `LamaWorker` 常駐連携を実装。
2. 以下のチェックとテストをすべて通過させる：
   - `cargo test -p photocraft-engine --lib lama_cleaner`（既存のパス解決・絶対パス禁止テストを含む全件パス）
   - `cargo clippy -p photocraft-engine --all-targets -- -D warnings`
   - `cargo xtask wasm`

---

## 5. 期待される改善数値まとめ

| 指標 | 改修前（現行） | 改修後（目標） |
|---|---|---|
| 初回モデルロード時間 | 約 10.6 秒 | **約 0.8〜1.5 秒**（最適化グラフキャッシュ時） |
| 2ストローク目以降のモデルロード時間 | 約 10.6 秒（毎回再ロード） | **0 ms（`--serve` 常駐保持）** |
| 512×512 推論時間（1回あたり） | 約 1.36 秒（CPU） | **約 30〜80 ms（GPU）** |
| **2ストローク目以降の体感待ち時間** | **約 12.0 秒** | **約 0.05〜0.1 秒（約 100〜200 倍高速化）** |
