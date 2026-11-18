# 本質的な統合計画書: 公式 #15449 前提でフォークを再構築する

作成日: 2026-11-18
対象リポジトリ: `D:\USERFILES\GitHub\photocraft`(`origin` = ussoewwin/photocraft、`upstream` = storytold/photocraft)
読者: この作業を引き継ぐすべての AI(どの AI でも、この文書だけで実行できることを目的とする)

---

## 0. 絶対条件(破ったら失敗)

1. **公式ファイルを上書きして競合を消さない。** 競合は「どちらの版を採るか」で片付けず、公式の新しい実装(#15449 以降)を土台に、フォーク側の意図を新しい API で書き直す。
2. **フォーク機能を壊さない。** 載せ替えるたびに、その機能のテストと `cargo check` で確認する。
3. **3 本の PR ブランチを削除しない。** `feat/localized-japanese-fonts`、`feat/quick-selection-mode-buttons`、`feat/move-tool-selection-drag-transfer`。
4. **`craft-libs` を変更しない。** `Cargo.toml` の git pin も変更しない。フォークは photocraft リポジトリだけで完結させる。
5. **#11971(`feat/move-tool-selection-drag-transfer`)は未完成。** 別の AI が修復中(テスト 33 件中 14 件が赤)。完成して監査を通るまで `main` へ統合しない。
6. Windows の `cargo` は MSVC 環境が必要(§7 参照)。グローバル Python は使わない。
7. `main` へ Push する前に、必ずユーザーの承認を得る。確認せずに `origin/main` を更新しない。

## 1. 何が起きているか(根本原因)

- 公式の #15449「PhotoCraft on the craft UI framework」(`ed14e1c413`)で、UI が photocraft 内の実装から外部ライブラリ `storytold/craft-libs` のフレームワークに置き換わった。`canvas.rs`、`icons.rs`、`options_bar.rs`、`font_preview.rs`(削除)、`type_tool/fonts.rs` などが大きく変わっている。
- フォークの `main` は、#15449 より前の公式(`ea6ced03b1` 付近)の上に作られている。中心は統合コミット `8da2be3707`(113 ファイル)で、次の 2 種類が 1 つに固まっている。
  - **PR 由来の内容**: #12452、#11837、#11971、#12339 など。
  - **フォーク専用機能**: LaMa Cleaner、SAM 3.1、日本語 i18n、`plugins/`、`md/`、`start-photocraft.cmd` など。
- そのため、公式最新を `main` に直接マージすると、`canvas.rs` など約 31 ファイルが競合する(`git merge-tree` で実測)。競合は「古い UI 実装の上に作ったフォーク変更」と「新しい UI フレームワーク」のぶつかり合いで、機械的な解消では壊れる。
- 一方、3 本の PR ブランチは #15449 を取り込み済みで、`upstream/main` に対する競合は **0 件**(上流の 3 コミット遅れ)。

測定値(2026-11-18 時点)

| 対象 | `upstream/main` に対する競合数 | 遅れ |
|---|---|---|
| `origin/feat/localized-japanese-fonts`(#12452) | 0 | 3 コミット |
| `origin/feat/quick-selection-mode-buttons`(#11837) | 0 | 3 コミット |
| `origin/feat/move-tool-selection-drag-transfer`(#11971) | 0 | 3 コミット |
| 古い `main` | 31 | 16 コミット |

## 2. 方針

**古い `main` に公式をマージしない。** 次の順に `main` を作り直す。

1. `upstream/main`(#15449 込み)を土台にする。
2. 競合ゼロの PR を 1 本ずつ取り込む(#12452 → #11837)。
3. フォーク専用機能を、公式の新しい UI フレームワーク前提で 1 機能ずつ載せ直す。
4. #11971 は完成・監査後に最後に取り込む。

## 3. 作業場所

- 作業用ワークツリー: `D:\USERFILES\GitHub\photocraft\target\wt-main`(ブランチ `main-rebuild`、`upstream/main` から作成)
- 既存の PR 用ワークツリー(触らない):
  - `target\wt-pr12452`(`pr12452-sync`)
  - `target\wt-pr11837`(`pr11837-sync`、暖機済みビルド `D:\pf_target_pr11837`)
  - `target\wt-pr11971`(`pr11971-sync`、暖機済みビルド `D:\pf_target_pr11971`)
- ワークツリーが途中状態なら捨てて作り直してよい: `git worktree remove --force target\wt-main`、`git branch -D main-rebuild`、`git worktree add -b main-rebuild target\wt-main upstream/main`。
- `index.lock` が残っていたら、git プロセスが動いていないことを確認してから削除する(`.git\worktrees\wt-main\index.lock`)。

## 4. 手順

### 手順 1: 土台と競合ゼロの PR(完了済みの場合は飛ばす)

```powershell
Set-Location D:\USERFILES\GitHub\photocraft
git fetch upstream origin
git worktree add -b main-rebuild target\wt-main upstream/main
Set-Location target\wt-main
git merge --no-edit origin/feat/localized-japanese-fonts      # #12452
git merge --no-edit origin/feat/quick-selection-mode-buttons  # #11837
```

期待: どちらも競合なし。`git status -sb` が `main-rebuild...upstream/main [ahead N]` で、未コミット変更がない。

### 手順 2: フォーク専用機能の仕分け(事実として確認済み)

差分の基準は `git merge-base origin/main upstream/main` = `2163bd45af`(以下 `$base`)。

PR 3 本のどれにも含まれない「フォーク専用」ファイル(`git diff --name-only $base origin/main` から PR 側を除いたもの):

- エンジン: `crates/engine/src/cmd/retouch_cmds.rs`、`retouch_cmds/lama_cleaner.rs`、`retouch_cmds/remove.rs`、`cmd/sam3_select_cmds.rs`、`cmd/analysis_cmds/tests.rs`、`cmd/print_cmds/tests.rs`
- フォーマット: `crates/format/src/atomic.rs`、`atomic/tests.rs`(#12339 Windows ディレクトリ flush)
- 自動化: `crates/automation/src/rpc/tests.rs`
- UI: `crates/ui-egui/src/i18n/ja.tsv`、`sam3_canvas_tests.rs`、`sam3_options_bar_tests.rs`、`sam3_toolbar_tests.rs`、`state/tool.rs`
- その他: `plugins/`(LaMa、SAM3 とバイナリ)、`md/`、`assets/icons/quick-selection-*.svg`、`.gitignore`、`README.md`、`start-photocraft.cmd`

PR のファイルの中にも、SAM/LaMa のための変更が混ざっている。`SAM3`、`lama` を含む追加行があるファイルは次のとおり(件数は追加行数):

`crates/engine/src/cmd.rs`(2)、`engine/src/lib.rs`(1)、`engine/src/prefs.rs`(2)、`prefs/tests.rs`(2)、`shell_commands.rs`(2)、`ui-egui/src/app/commands.rs`(1)、`canvas.rs`(1)、`canvas/gesture.rs`(2)、`canvas/paint.rs`(3)、`canvas/tool_feedback.rs`(1)、`icons.rs`(2)、`input/hold_keys.rs`(1)、`input/tool_key_default_tests.rs`(1)、`lib.rs`(3)、`panels.rs`(1)、`panels/toolbar.rs`(2)、`panels/type_flyout_tests.rs`(1)、`pencil_tests.rs`(3)、`smart_objects_menu_tests.rs`(1)、`state/tool_options.rs`(17)、`tool_shift_cycle_order_tests.rs`(3)、`toolbar_group_order_tests.rs`(2)、`tools/retouch_ui.rs`(37)、`tools/retouch_ui/tests.rs`(21)、`tools/tool_reset.rs`(3)、`dialogs/file_ui/tests.rs`(5)、`dialogs/discard_ui/tests/jpeg_tests.rs`(1)

この一覧は次のコマンドで再現できる。

```powershell
$base='2163bd45af'
git diff --name-only $base origin/main | Where-Object { $_ -notmatch '^(plugins/|md/)' } | ForEach-Object {
  $n = (git diff $base origin/main -- $_ | Select-String -Pattern '^\+.*(?i)(sam3|lama|SamSelect|flush)').Count
  if ($n -gt 0) { "$n  $_" }
}
```

### 手順 3: UI に依存しない部分を先に載せる(フェーズ A)

対象: エンジン、フォーマット、自動化、`plugins/`、`md/`、`assets/icons`、`.gitignore`、`README.md`、`start-photocraft.cmd`、`ja.tsv`。UI フレームワークに依存しないため、素直に載るはず。

```powershell
Set-Location D:\USERFILES\GitHub\photocraft\target\wt-main
$base='2163bd45af'
# 上記「フォーク専用」のエンジン/フォーマット/自動化ファイルと plugins md assets .gitignore README start-photocraft.cmd ja.tsv を指定
git diff --binary $base origin/main -- <パス...> > ..\fork_a.patch
git apply --3way --index ..\fork_a.patch
```

- 競合が出たファイルは、公式側(#15449 以降)の内容を残したまま、フォーク側の追加だけを手で足す。
- 注意: `crates/engine` の差分には #11971 由来のファイル(`float_cmds.rs`、`layer_copy_cmds.rs`、`pick_cmds.rs`)が含まれる。**フェーズ A では取り込まない。** 上のコマンドで対象パスを列挙するときに含めないこと。
- `plugins/` のバイナリ(`*.exe`、`*.dll`)は大きい(合計約 130 MB)。ユーザーの確認なしに増減させない。
- `crates/automation/src/server.rs` は競合した実績がある(公式側が変更済み)。公式側を残し、フォークの追加だけ足す。
- 確認: `cargo check -p photocraft-engine -p photocraft-format -p photocraft-automation` と、各クレートの該当テスト。

### 手順 4: UI 部分を #15449 の API で載せ直す(フェーズ B、本質部分)

対象は手順 2 で挙げた UI ファイル。次の方針で 1 機能ずつ行う。

1. 対象機能(SAM Selection、LaMa Cleaner、SAM のツール切替・ツールバー・オプションバー・キー割り当て・アイコン、`tool_options` の追加項目、`retouch_ui` の追加分)の「意図」を `md/sam3-select.md`、`md/lama-cleaner-cuda.md` で確認する。
2. 古い実装(`origin/main` のファイル)を読み、何をしていたかを把握する。
3. 公式の新しい構造(craft-libs のフレームワーク、`canvas/tools/*`、`panels/options_bar/*`、`tools/retouch_ui/bar/*` など)で、同じ機能を表現する場所を探す。
4. 公式のコードを土台に、フォークの追加を新しい API で書く。公式の既存コードは削らない。
5. その機能のテスト(`sam3_*_tests.rs` など)を新しい API に合わせて移植し、緑にする。
6. 機能ごとに `cargo test --lib -- <フィルタ>` を回し、緑を確認してからコミットする。

フェーズ B のコミットは機能ごとに分ける(例: SAM ツール登録、SAM オプションバー、LaMa のリタッチ配線、日本語 i18n 配線)。

### 手順 5: 総合確認

```powershell
cargo fmt --check
cargo clippy -p photocraft-ui-egui -p photocraft-engine --all-targets -- -D warnings
cargo test -p photocraft-ui-egui --lib
cargo test -p photocraft-engine --lib
cargo xtask layers
cargo xtask panel-commands
cargo xtask wasm
```

- 失敗したものは、フォークの変更が原因か公式側の既存の失敗かを `upstream/main` 単体で切り分ける(上流の失敗をフォークのせいにしない。逆も同じ)。
- 追加で確認すること: `SAM3`/`LaMa` が各ツールとして呼べる(ツールバー、オプションバー、メニュー)、#12452 のフォント分類と `あいうえお` プレビュー、#11837 のバッジ・ボタン配置・選択内ドラッグ。

### 手順 6: `main` の更新(ユーザー承認後)

1. `main-rebuild` の内容を `main` に反映する。`origin/main` は履歴の作り直しになるため、必ずユーザーに確認する。
2. `photocraft_sync_and_build`(`git_automation` MCP)は `repository_path` を **必ず指定** する(未指定だと `Invalid repository path` で失敗する)。例: `repository_path = D:\USERFILES\GitHub\photocraft`。
3. 統合は一括でなく 1 本ずつ行う。PR ブランチの同期(`sync_pr_branches`)は、他の AI が作業中のブランチ(#11971)を触らないよう、`false` にするか `pr_branches` を明示する。

### 手順 7: #11971 の統合(最後)

1. 別の AI が 14 件の赤を直し、`cargo test --lib -- layer_transfer` が緑になるまで待つ。
2. 監査する。確認項目は `md/pr-11971-move-tool-cross-document-transfer.md` に従い、整列タイルへのドロップ、タブへのドロップ、スプリングロード、マスク選択のコピー、ブロック時のドラッグ状態を実測する。
3. `main-rebuild` に `origin/feat/move-tool-selection-drag-transfer` をマージする。競合ゼロのはず。
4. 手順 5 を再実行する。

## 5. 現在の進捗(2026-11-18 13:30 時点)

- #12452: 監査済み、問題なし。
- #11837: 4 件の欠落(ブラシカーソルのバッジ、選択内ドラッグのライブ追従と 45°、モードボタンの配置、`Select and Mask…`)を修正し、`origin/feat/quick-selection-mode-buttons` へ Push 済み(`7427d24271`)。関連テスト 53 件が緑。
- #11971: 別 AI が修復中。完成を待つ。
- `main-rebuild`(`target\wt-main`): 手順 1 は完了(#12452 と #11837 をマージ済み)。手順 3 のパッチ適用を途中まで実行しており、`crates/automation/src/server.rs` が競合状態、残りは index に載っている。引き継ぐ AI は、`git status` で状態を確認し、必要なら手順 3 をやり直す。
- ローカルの `main` には、#12452 を統合した未 Push のコミット(`4d4570e52c`)がある。これは作り直し後に不要なら破棄する。`origin/main` は触っていない。

## 6. やってはいけないこと(再発防止)

- 古い `main` に `upstream/main` を直接マージして、競合を片側採用で消す。
- `canvas.rs`、`icons.rs`、`options_bar.rs` などの公式ファイルを、フォーク側の古い版で上書きする。
- 公式が削除したファイル(`text/font_preview.rs`、`canvas/gesture.rs`、`canvas/paint.rs`、`canvas/tool_input.rs`、`burn_bar_tooltips_tests.rs`)を、競合解消のために復活させる。
- テストが落ちたのでテストを削除する、アサートを弱める。
- #11971 を未完成のまま `main` へ入れる。
- ユーザーの承認なしに `origin/main` を更新する、PR ブランチを削除する。
- `craft-libs` を変更する、`Cargo.toml` の pin を変える。

## 7. 環境メモ(Windows)

- `cargo` を実行する前に MSVC 環境を読み込む。読み込まないと `aws-lc-sys` が `stdint.h` を見つけられず失敗する。

```powershell
$env:CARGO_TARGET_DIR = "D:\pf_target_main"   # 作業ごとに専用のターゲットを使う(各 10 GB 程度)
Set-Location D:\USERFILES\GitHub\photocraft\target\wt-main
cmd /c "call ""C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat"" >nul && cargo test --lib -- <フィルタ>"
```

- 暖機済みターゲットを使うときは、`-p` を付けずに `cargo test --lib -- <フィルタ>` とする(フィーチャ統合を既存キャッシュに合わせるため)。
- Python が必要な場合は `D:\USERFILES\ComfyUI\python_embeded\python.exe` をフルパスで指定する。グローバル Python は使わない。
- コード規約: 本番コードで `unwrap()`、`expect()`、`panic!()` を使わない。コミットやドキュメントに AI の視点や署名を入れない。

## 8. 完了の定義

- `main-rebuild` が `upstream/main` を祖先に持つ(`git merge-base --is-ancestor upstream/main main-rebuild` が成功)。
- #12452、#11837 の内容がすべて入っている(各 PR ブランチの `git merge-base --is-ancestor` が成功)。
- フォーク専用機能(SAM 3.1、LaMa Cleaner、日本語 i18n、#12339 の flush 修正)が動き、各テストが緑。
- 手順 5 のコマンドがすべて通る。
- ユーザーの承認後に `main` が更新され、#11971 は完成後に別途統合される。
