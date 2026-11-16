# LaMa Cleaner CUDA 対応 技術解説

LaMa Cleaner(`paint.lamaCleaner`)の GPU(CUDA)対応について、実装の全体を解説する。
すべて Rust 実装であり、推論はヘルパープロセス `plugins/lama-cleaner` が ONNX Runtime(`ort` 2.0.0-rc.9)で行う。エンジン本体は `ort` にリンクせず、ヘルパーを `std::process::Command` で起動するだけである。

- 対象コミット: `049063b90` → `85c1b5a1c` → `b14e2651a`(main には `5810f9063`・`72827c369` 経由で取り込み済み)
- 主要ファイル: `plugins/lama-cleaner/src/lib.rs`(推論と EP 選択)、`plugins/lama-cleaner/src/main.rs`(CLI と常駐ループ)、`crates/engine/src/cmd/retouch_cmds/lama_cleaner.rs`(コマンドと常駐ワーカー管理)
- 記載の数値は 2026-11-16 の実測値(Windows / CUDA 13 + cuDNN 9.26 ランタイム / RTX GPU / `big-lama.onnx` 198.4 MiB)

---

## 1. 概要

CUDA 対応の本体は次の 3 点。

1. **実行プロバイダ(EP)チェーン**: ロード時に CUDA → DirectML → CPU の順で選択し、実行中にプロバイダが壊れた場合は縮退する(`plugins/lama-cleaner/src/lib.rs:91`、`lib.rs:379`)。
2. **CUDA/cuDNN DLL の探索とフォルダ全件プリロード**: PATH 未設定でも CUDA が動く(`lib.rs:432`、`lib.rs:465`)。
3. **常駐ワーカー(`--serve`)**: モデルをプロセス内に保持し、ストロークごとのモデルロード(6.7〜8.6 秒)を不要にする(`plugins/lama-cleaner/src/main.rs:160`、`crates/engine/src/cmd/retouch_cmds/lama_cleaner.rs:232`)。

実測性能は §9。CUDA 定常状態でストローク 1 回あたり 87〜95 ms。

## 2. 構成要素とデータフロー

```text
UI (LaMa Cleaner ツール, オプションバー)
  └─ engine コマンド paint.lamaCleaner        crates/engine/src/cmd/retouch_cmds.rs:336
       └─ lama_cleaner.rs
            ├─ plan(): ストローク → 修復ウィンドウ (§7.1)
            ├─ hole(): 穴マスク生成 (§7.1)
            └─ jobs::run → execute_lama()
                 ├─ ウィンドウを RGB8 / Gray8 PNG で一時ファイルへ
                 ├─ resident::apply() → lama-cleaner --serve (§6)
                 │     └─ 失敗時は run_helper_once() の 1 回きり実行へフォールバック
                 ├─ 結果 PNG をデコード
                 └─ hole のみピクセル置換 → 1 undo ステップで合成
```

ヘルパーには 3 つの起動形態がある(`plugins/lama-cleaner/src/main.rs` の USAGE)。

| 形態 | 引数 | 用途 |
|---|---|---|
| OneShot | `-i <in.png> -m <mask.png> -o <out.png> [--model <m>]` | 1 画像 1 プロセス。エンジンのフォールバック経路 |
| Serve | `--serve [--model <m>]` | モデルをロードし続け、stdin の 1 行を 1 JSON リクエストとして処理 |
| セルフテスト | (引数なし) | 合成画像で 1 回推論し `%TEMP%\photocraft-lama-selftest\test_inpainted.png` に保存 |

wasm ビルドはヘルパーを起動できないため、コマンドは "LaMa Cleaner is only available in the desktop app" を返す(`lama_cleaner.rs:531`)。

## 3. ビルド構成

`plugins/lama-cleaner/Cargo.toml:10-14`:

```toml
ort = { version = "2.0.0-rc.9", features = ["cuda", "directml", "preload-dylibs", "lax-feature-matching"] }
```

- Windows x86_64 の配布テーブルには CUDA ビルドが 1 種のみ(`cuda13,tensorrt,nvrtx,directml`)があり、`cuda` だけでは一致しない。`lax-feature-matching` がこの単一ビルドを選ぶ。
- `preload-dylibs` は `ep::cuda::preload_dylibs()` と `ort::util::preload_dylib()` を有効化し、PATH に依存せず任意のフォルダから CUDA/cuDNN をロードできるようにする(§5)。
- その他の依存: `ndarray 0.17`(テンソル)、`image 0.25`(png/jpeg のみ)、`serde` / `serde_json`(プロトコル)、`thiserror`。

## 4. 実行プロバイダチェーン

### 4.1 順序と設定

`Provider::dispatch()`(`plugins/lama-cleaner/src/lib.rs:57`):

| EP | 設定 | 理由 |
|---|---|---|
| CUDA | `ConvAlgorithmSearch::Heuristic` + `with_tf32(true)` + `error_on_failure()` | ヒューリスティック探索は cuDNN の網羅ベンチマーク(初回に数秒)を省く。TF32 は Ampere 以降のテンソルコアを使う。`error_on_failure()` で、登録失敗を黙って CPU に落とさない |
| DirectML | `ep::DirectML::default().build().error_on_failure()` | DirectX 12 対応 GPU 全般(NVIDIA 以外も含む) |
| CPU | EP なし(ORT 既定) | 最後の手段 |

### 4.2 ロード時の選択

`LamaModel::load()`(`lib.rs:91`)。まず `prepare_cuda()`(§5)が CUDA ランタイムの所在を確認し、見つからなければ CUDA をスキップする。残りを CUDA → DirectML → CPU の順で `open_session()`(`lib.rs:422`)して、最初に開けたものを採用する。開かなかった EP の名前とエラーはすべて `notes` に積まれ、`NoProvider` エラーに連結される。

### 4.3 実行時の縮退(遅延フォールバック)

セッションは開いたのにモデルの実行が失敗するケース(DLL は見つかったが別バージョン、等)がある。`run_patch()`(`lib.rs:379`)は最初の `session.run()` が失敗すると:

1. 現在のプロバイダを `state.failed` に記録し、`next_provider()`(`lib.rs:418`)で未失敗の次の EP を選ぶ。
2. `open_session()` で置換セッションを作り、**同じパッチを再実行**する。ストロークは失敗ではなく「遅い経路」で完走する。
3. 置換の旨は stderr と `notes` に記録される。

CPU で失敗したらそれ以上の縮退先がなく、そのエラーを返す。

`LamaModel` は `session: Mutex<Option<Session>>` でセッションを保持し、実行中だけ `take()` して返却する(`inpaint_512`、`lib.rs:122`)。これは置換中に他の推論がセッションを借用しないための構造である。Mutex の毒は `unwrap_or_else(PoisonError::into_inner)` で回復する。

## 5. CUDA / cuDNN DLL の探索と全件プリロード

### 5.1 判定と探索順序

`prepare_cuda()`(`lib.rs:432`)は `cudart64_13.dll` と `cudnn64_9.dll`(それぞれ `ort` の `CUDA_DYLIBS` / `CUDNN_DYLIBS` の先頭名、取れなければ固定名)を `find_runtime_dir()`(`lib.rs:498`)で探す。探索順:

1. 環境変数 `CUDA_PATH` / `CUDNN_PATH` / `CUDA_HOME` / `CUDNN_HOME` の各ルートに、`push_toolkit_dirs()`(`lib.rs:478`)が候補を追加する: `bin\x64`(CUDA 13+ と cuDNN 9+ の新レイアウト)→ `bin\<バージョン>\x64` と `bin\<バージョン>`(ディレクトリ名の降順。`bin\13.0\x64` など cuDNN 9 のバージョン付きレイアウト)→ `bin` → ルート自身。
2. ヘルパー実行ファイル自身のフォルダと、その親(`plugins` フォルダ)。
3. `Program Files\NVIDIA GPU Computing Toolkit\CUDA\*` と `Program Files\NVIDIA\CUDNN\*`(バージョン ディレクトリを降順で全部)。
4. `PATH`。

最初に目的の DLL が見つかったディレクトリを採用する。どちらか片方でも欠けたら、欠けている方の名前を明記した note を出して CUDA を諦める。

### 5.2 フォルダ全件プリロード — なぜ 1 個ずつでは足りないのか

見つかった CUDA / cuDNN の両ディレクトリに対し:

1. `ep::cuda::preload_dylibs(Some(&cuda), Some(&cudnn))` で ORT の既知リストを読み込む。
2. その後 `preload_folder()`(`lib.rs:465`)で、**両フォルダ内の `.dll` を名前順に全件** `ort::util::preload_dylib()` でロードする。ロードできない DLL(別バージョン、別アーキテクチャ)は黙ってスキップする。

理由: CUDA EP は実行時に依存 DLL を「名前」で読みに行く。Windows のローダによる名前解決は「すでにプロセスにロード済み」か「PATH 上」しか見ない。ところが cuDNN 9.26 は `cudnn_engines_tensor_ir64_9.dll` のような追加プロバイダ DLL を同梱しており、これは ONNX Runtime 側の固定リストに存在しない。フォルダ全件を先にロードしておかないと、**最初の畳み込みで "Could not locate ..." が出て CUDA が落ちる**(実測)。この修正が `b14e2651a` である。

効果: CUDA 13 と cuDNN 9.26 がインストールされていれば、**PATH に何も追加しなくても** CUDA で動く。

### 5.3 失敗時の診断(note 文面)

- `the CUDA 13 runtime (cudart64_13.dll) was not found; using DirectML`
- `cuDNN 9 (cudnn64_9.dll) was not found; using DirectML`
- `CUDA libraries could not be preloaded (<エラー>); using DirectML`

これらは `LamaModel::notes()` として返り、ヘルパーの stderr に `lama-cleaner: <note>` の形で出る。

## 6. 常駐ワーカー(`--serve`)

### 6.1 モチベーションとプロトコル

モデルロードは 6.7〜8.6 秒かかる。`--serve`(`main.rs:160`)はモデルを 1 回だけロードし、stdin から 1 行 = 1 JSON リクエスト、stdout へ 1 行 = 1 JSON リプライで答える。

```text
要求: {"image":"<png>","mask":"<png>","output":"<png>"}   または {"warmup":true}
応答: {"ok":true,"infer_ms":42.1,"provider":"CUDA"}
      {"ok":false,"error":"..."}
```

診断はすべて stderr。stdout はリプライ専用である。未知のオプションはエラーになる(`main.rs` の `parse_args`): 旧ビルドのヘルパーが prose を出した場合に、エンジン側がそれを「serve できないヘルパー」と正しく判定して one-shot へ落とす設計と噛み合う。

### 6.2 ウォームアップの設計 — 背景スレッドを勝手に起こさない

`warmup_once()`(`main.rs:146`)は 512×512 の空画像を 1 パスして cuDNN のカーネルを初期化する(実測 約 206 ms)。ただし `serve()` は**自発的な背景ウォームアップを起こさない**。ストロークをきっかけに `--serve` が起動された場合、最初の stdin 行はすでに待っており、背景スレッドがセッションロックを掴んで `run_one` の PNG デコードと競合し、初回ストロークが `inpaint_512` を 2 回実行してしまう(実測された競合、`85c1b5a1c` で修正)。アイドル中にワーカーを先行起動する呼び出し側だけが、`{"warmup":true}` を明示送信してよい。

### 6.3 エンジン側のワーカー管理

`lama_cleaner.rs:232` の `resident` モジュール。

- **スロット**: `OnceLock<Mutex<Option<Worker>>>`(`lama_cleaner.rs:337`)。プロセス全体で 1 ワーカー。`matches()` で exe / model が不一致なら即破棄して作り直す。
- **起動**(`Worker::start`): `lama-cleaner --serve --model <model>` を stdin/stdout piped・stderr inherit で起動。stdout は専用スレッドが `read_line` して mpsc チャンネルへ流す。要求側は `read_line` でではなく `recv_timeout` で待つため、ヘルパーが固でも要求スレッドがパイプで詰まない。
- **タイムアウト**: `REPLY_TIMEOUT = 120 秒`(`lama_cleaner.rs:242`)。初回要求はモデルロードを含む。タイムアウト・プロセス終了は `Unavailable`。
- **エラーの二分**(`parse_reply`、`lama_cleaner.rs:356`):
  - `Unavailable`: 起動失敗・プロセス死・タイムアウト・非 JSON の出力(旧ヘルパーの prose など)。ワーカーを破棄する対象。
  - `Failed`: 健全なワーカーからの `{"ok":false,"error":...}`。ワーカーは保持し、エラーメッセージをそのまま報告する。
- **再起動ポリシー**(`apply`、`lama_cleaner.rs:377`): 既存ワーカーが死んでいた場合は新ワーカーで 1 回だけ再試行する。起動直後の失敗は即 `Unavailable`。`Unavailable` で終わった場合はスロットを空にして呼び出し側へ返す。
- **フォールバック**(`execute_lama`、`lama_cleaner.rs:470`): `apply()` が `Unavailable` なら `run_helper_once()`(`lama_cleaner.rs:450`、`-i -m -o --model` の 1 回きりプロセス)を実行する。`Failed` はそのままコマンドのエラーになる。

## 7. 推論パイプライン

### 7.1 エンジン側

コマンド仕様(`retouch_cmds.rs:336`): `paint.lamaCleaner`、パラメータは `strokes`(複数ストローク可)、`closeLoops`=true、`sampleAllLayers`=false、`newLayer`=false、`exe`、`model`(ヘルパーとモデルがアプリの隣にない場合の明示指定)。

1. **ウィンドウ計画**(`plan()`、`lama_cleaner.rs:82`): ストローク群の境界に、余白 `margin = clamp(長辺の 1/2, 128..512)` を足し、幅・高さが 512 未満なら 512 以上まで拡張する。キャンバスからはみ出す分はシフトで吸収し、最後にキャンバスと交差させる。`MAX_MARGIN = 512`(`lama_cleaner.rs:19`)。これは LaMa の FFC が局所・大域の両構造を見るための文脈確保である。
2. **穴マスク**(`hole()`、`lama_cleaner.rs:126`): ブラシカバレッジを重ね、`closeLoops` で閉曲線の内部化(`close_loops`)、`dilate` で 5 px 膨張(`GROW`、`lama_cleaner.rs:22`。アンチエイリアスのフリンジを吸収)、選択範囲があれば AND を取る。
3. **実行と合成**(`run_lama_cleaner` → `execute_lama`、`lama_cleaner.rs:535` / `:470`): 対象ウィンドウのピクセルを RGB8 に、穴を Gray8 にエンコードして一時 PNG へ。ヘルパー実行後、結果 PNG をデコードし、**穴のピクセルだけ**元データへ書き戻す(アルファは元の値を保持、`clamp_samples` でレンジ保証)。`jobs::run` の 1 ジョブ・1 undo ステップで、`newLayer` 指定時は `fill_to_new_layer()`(`lama_cleaner.rs:564`)が結果を新規レイヤーに置く。

### 7.2 ヘルパー側

**任意解像度の処理**(`inpaint()`、`lib.rs:161`、IOPaint 互換の crop & pad 戦略):

1. マスクの bounding box を求める(マスクなしなら元画像のクローンを返す)。
2. コンテキスト余白 `context_margin = max(bbox 長辺の 1/2, 128)`、目標辺 `desired = max(512, bbox + 2 × margin)` のクロップを画像中心寄りで切る。
3. クロップが 512 未満の辺を含むなら、反射パディングで `pad_side = max(512, 辺の最大値)` の正方形にしてから 512 に縮小する(Lanczos3。マスクは Nearest で 2 値性を保つ)。512 を超えるクロップは 512 に縮小 → 推論 → 元サイズへ Lanczos3 で拡大。
4. 推論結果をクロップ領域へ戻す。

**512 パッチの推論**(`inpaint_512()`、`lib.rs:122`):

- 入力テンソル(`build_inputs()`、`lib.rs:335`): `image` = `[1, 3, 512, 512]` の 0.0..1.0(CHW、255 で除算)、`mask` = `[1, 1, 512, 512]` の 0/1(閾値 127)。
- 出力は平面配置 RGB(plane = 262144)。`big-lama.onnx` は 0..255 で出力するが、0..1 を出す変種も検出できるように `max > 1.5` で判定してスケールする。
- ブレンド: マスク境界ピクセルは、7×7 近傍内の最短「非マスク」距離の二乗からフェザー係数を取る(`feather()`、`lib.rs:322`): `d² ≤ 1 → 0.6`、`≤ 4 → 0.85`、それ以上 `1.0`。`alpha = マスク値/255 × 係数` で元画像と線形混合する。マスク外は 1 ピクセルも変わらない。

## 8. ファイル解決

- **モデル**(`resolve_model()`、`main.rs:99`): `--model` → 環境変数 `PHOTOCRAFT_LAMA_MODEL` → ヘルパー実行ファイルの隣、次にその親フォルダの `big-lama.onnx`。実寸 **208,044,816 バイト(198.4 MiB)**。
- **ヘルパーとモデル(エンジン側)**(`LamaTool`、`lama_cleaner.rs:184-230`): パラメータの `exe` / `model` が最優先。なければ (1) Preferences › Plug-ins の追加プラグインフォルダ(有効な場合のみ)、(2) 実行ファイル隣の `plugins`、(3) 実行ファイル自身のフォルダ、の順で探す。見つからないエラーは**探索した全候補を列挙**する。ワーキングディレクトリは起動のされ方に依存するため探索しない。リポジトリのコードにマシン固有の絶対パスを書かないことはテストで保証されている(`no_machine_absolute_path_is_written_in_this_file`)。

## 9. 実測性能

計測環境: Windows、CUDA 13 + cuDNN 9.26 ランタイム、RTX GPU、`big-lama.onnx` 198.4 MiB(2026-11-16)。

| 項目 | 値 |
|---|---|
| モデルロード | 6.7〜8.6 秒 |
| CUDA 定常状態(`--serve` 経由の推論) | 87〜95 ms / ストローク |
| `paint.lamaCleaner` 経由の全体 | 約 120 ms |
| 起動直後の初回ストローク(ロードを含む) | 約 11.6 秒 |
| ウォームアップ 1 パス(512×512) | 約 206 ms |

- DirectML / CPU 経路の定数は未計測である(チェーンが存在し、縮退は動作確認済み)。
- **グラフ最適化 ONNX キャッシュは廃止した**: あり = 6.90 秒、なし = 6.67 秒で効果がなく、プロバイダごとに約 197 MB を消費するため。

## 10. 実機デプロイ

1. ヘルパー `lama-cleaner.exe` と `big-lama.onnx` を `plugins` フォルダに置く。
2. Preferences › Plug-ins で「Additional Plug-ins Folder」を有効にし、そのフォルダを指定する。設定ファイルのキーは camelCase で `"plugIns": {"useAdditionalPluginsFolder": true, "additionalPluginsFolder": "<plugins フォルダ>"}`。
3. 開発ビルドでは `target\release\plugins\` 側にも同名ファイルが必要な場合がある(ハードリンクで実体の二重化を避けられる)。
4. **旧ビルドのヘルパー(`--serve` 非対応)が残っていると、常駐は毎回 one-shot へフォールバックする**。入れ替えを確認すること。これはエラーではなく設計どおりの挙動である。
5. CUDA 13 と cuDNN 9 がインストールされていれば、PATH の設定は不要(§5.2)。

## 11. 診断とトラブルシューティング

- **どの EP で動いたか**: `--serve` リプライの `provider` フィールド、起動時 stderr の `lama-cleaner: <EP> provider, model loaded in ...`、one-shot の `SUCCESS: ... (Provider: ...)` 行。なおエンジンはヘルパーの provider 文字列を現在 UI へは表示していない(`execute_lama` が破棄している) — UI 表示は今後の課題として残す。
- **CUDA にならないときの確認順**:
  1. `cudart64_13.dll` と `cudnn64_9.dll` が §5.1 の探索順のどこかにあるか(`CUDA_PATH`、`CUDNN_PATH`、`Program Files\NVIDIA\CUDNN\<版>\bin\<版>\x64` など)。
  2. ヘルパー stderr の note を読む(§5.3 の 3 文面)。
  3. cuDNN のバージョン付きレイアウト `bin\<cuda_ver>\x64` は対応済み。
  4. PATH への追加は不要。追加しても壊れないが、設計上は不要。
- **ハング**: 120 秒で `Unavailable` 判定となり、one-shot で再実行される。
- **ヘルパー単体の動作確認**: 引数なしで起動するとセルフテストが走る(合成画像を修復し `%TEMP%\photocraft-lama-selftest\test_inpainted.png` に保存、中心ピクセルの Before/After を表示)。

## 12. テスト

| 場所 | 内容 |
|---|---|
| `plugins/lama-cleaner/src/lib.rs`(5 件) | feather ランプの固定、provider 名、`next_provider` のチェーン規則、GPU EP の登録主張、CUDA13/cuDNN9 の `bin\x64`・`bin\<ver>\x64` レイアウト探索 |
| `plugins/lama-cleaner/src/main.rs`(6 件) | 引数パース(セルフテスト/one-shot/serve/未知オプション/パス欠落/部分セット) |
| `crates/engine/src/cmd/retouch_cmds/lama_cleaner.rs` | フォルダ優先順、prefs 有効時のみ探索、params のパス指定、欠落時の全候補列挙、マシン絶対パス禁止、resident の要求行・応答パース・prose 拒否・起動失敗 |

実測: plugin テスト lib 5 件 + bin 6 件、すべて合格(2026-11-16)。

## 13. 制限と設計判断の記録

- モデルの入力辺は 512 固定(`SIDE`、`lib.rs` 先頭部)。任意解像度は §7.2 の ROI 戦略で対応する。
- wasm は非対応。コマンドが明示的なエラーを返す。
- ワーカースロットはプロセス全体で 1 個。ドキュメントごとの分離はしていない。
- エンジンは自動ウォームアップを送らない(§6.2 の競合回避)。先行起動する呼び出し側のみ `{"warmup":true}` を送れる。結果、最初のストロークはモデルロードを支払う。
- グラフ最適化キャッシュは実測に基づき廃止(§9)。
- `REPLY_TIMEOUT` は 120 秒。初回要求のモデルロードもこの中に含まれる。

## 14. 変更履歴

| コミット | 内容 |
|---|---|
| `049063b90` | LaMa Cleaner runs on the GPU and keeps its model loaded — EP チェーン(CUDA→DirectML→CPU)と `--serve` 常駐の導入 |
| `85c1b5a1c` | Fix LaMa cleaner warm-up contention, bin\x64 toolkit paths and resident error handling — ウォームアップ競合の除去、`bin\<ver>\x64` レイアウト対応、`Unavailable` / `Failed` の分離 |
| `b14e2651a` | Make the LaMa CUDA path work without the toolkit on PATH — DLL フォルダ全件プリロードと実行時の遅延プロバイダ縮退 |
