# craft-libs 並行管理手順

PhotoCraft の UI フレームワーク（キー配送、フォーカス、AccessKit）は共有ライブラリ `storytold/craft-libs`（フォーク `ussoewwin/craft-libs`）にある。PhotoCraft 側だけで完結する修正と違い、**直すべきコードが別リポジトリにある**ため、次の 3 つを常に同期させる必要がある。

| 対象 | 役割 |
|---|---|
| `photocraft/Cargo.toml` の `rev` | どの craft-libs コミットを使うかの正本（全員に効く） |
| `photocraft/.cargo/config.toml` の `[patch]` | 手元だけでローカル craft-libs を指す開発装置（**共有しない**） |
| `photocraft/Cargo.lock` | 依存の出所の記録（patch 中は手元専用になる） |

この 3 つの関係を取り違えると「手元では動くが CI と他の開発者が即死する」状態になる。以下はその事故を防ぐための手順である。

---

## 1. 初回セットアップ

craft-libs を PhotoCraft の隣に clone し、**PhotoCraft が固定している rev に合わせる**。`main` の先頭に合わせてはいけない（PhotoCraft が知らない API 変更で壊れる）。

```powershell
cd D:\USERFILES\GitHub
git clone --filter=blob:none https://github.com/storytold/craft-libs.git craft-libs
cd craft-libs
git checkout <Cargo.toml の rev>
```

rev はこれで取る：

```powershell
Select-String -Path D:\USERFILES\GitHub\photocraft\Cargo.toml -Pattern 'rev = "([0-9a-f]{40})"' |
  Select-Object -First 1
```

フォークを remote に足す（push 先）：

```powershell
cd D:\USERFILES\GitHub\craft-libs
git remote add fork https://github.com/ussoewwin/craft-libs.git
```

---

## 2. 日常：ローカル修正して PhotoCraft で動かす（patch on）

```powershell
cd D:\USERFILES\GitHub\photocraft
cargo xtask craftlibs on          # 201 クレート分の [patch] を生成する
```

これで終わり。以下は中で何が起きているかと、道具を使わずに直す場合の手順。**クレートは 1 つも漏らさない**。craft-libs 内のクレートは相互に path 依存しているので、一部だけ差し替えると同一クレートが 2 ソースになり解決に失敗する。

patch 段は craft-libs の `crates/*/Cargo.toml` を走査して機械生成する（手書きしない。クレートが増えると漏れる）：

```powershell
cd D:\USERFILES\GitHub\craft-libs
$map = @{}
Get-ChildItem 'crates' -Directory | ForEach-Object {
  $m = Join-Path $_.FullName 'Cargo.toml'
  if (Test-Path $m) {
    $n = (Select-String -Path $m -Pattern '^\s*name\s*=\s*"([^"]+)"' | Select-Object -First 1)
    if ($n) { $map[$n.Matches[0].Groups[1].Value] = "crates/$($_.Name)" }
  }
}
```

生成した `[patch."https://github.com/storytold/craft-libs"]` を `photocraft/.cargo/config.toml` の末尾に置く。各行は：

```toml
craft = { path = "../craft-libs/crates/craft" }
```

**正しく効いているかの確認**。`cargo check` が通ることだけでは不十分で、実際にローカルが使われたかを見る：

```powershell
cd D:\USERFILES\GitHub\photocraft
cargo tree -p photocraft-ui-egui -e normal --depth 0
```

- `craft` / `craft-ui-contract` / `craft-ui-canvas` がローカル解決なら成功
- `warning: patch 'craft-xxx ...' was not used in the crate graph` は**そのクレートが依存グラフに無いだけ**で無害。使われているクレートには出ない
- 現在の実測：201 クレート中、39 件がこの警告（正常）

### PhotoCraft 側のテストを回すときの注意

単一パッケージ指定は feature 解決が足りずに落ちる（`egui::Id: serde::Serialize` が 13〜36 件）。**6 パッケージ同時**で回す：

```powershell
$env:CRAFT_I18N_REQUIRED='1'
$env:CRAFT_I18N_DIR='D:\USERFILES\GitHub\craft-i18n'
$env:CRAFT_FONTS_DIR='D:\USERFILES\GitHub\photocraft\craft-fonts'
cargo test --lib -p photocraft-ui-egui -p photocraft-engine -p photocraft `
  -p photocraft-cli -p photocraft-text -p photocraft-automation
```

2026-11-18 の実測：**4546 passed / 0 failed**（142 + 7 + 4045 + 352）。

---

## 3. 修正を craft-libs に送る

craft-libs は detached HEAD で clone しているので、**必ずブランチを切る**。

```powershell
cd D:\USERFILES\GitHub\craft-libs
git switch -c fix/<短い名前>
git add <変更したファイル>          # git add -A は使わない
```

commit の前に **author を実測で確認する**。実行環境によっては環境変数が git の設定を上書きし、意図しない名前で commit される。

```powershell
"user.name = $(git config user.name) / env = $env:GIT_AUTHOR_NAME"
```

意図した名義と違う場合は、commit の直前に環境変数も明示的に揃える：

```powershell
$env:GIT_AUTHOR_NAME='ussoewwin'
$env:GIT_AUTHOR_EMAIL='136552381+ussoewwin@users.noreply.github.com'
$env:GIT_COMMITTER_NAME=$env:GIT_AUTHOR_NAME
$env:GIT_COMMITTER_EMAIL=$env:GIT_AUTHOR_EMAIL
git commit -m "<何を直したか>"
git log -1 --format='%an <%ae> | %cn <%ce>'   # 実測で確認
```

push と、リモート実バイトの確認：

```powershell
git push -u fork fix/<短い名前>
git rev-parse HEAD
git ls-remote fork fix/<短い名前>    # 同じ SHA が返ること
```

---

## 4. pin bump（PhotoCraft が新しい craft-libs を使う）

craft-libs の修正がフォークに乗ったら、PhotoCraft の `Cargo.toml` の `rev` を差し替える。**patch を外してから**行う（patch が残っていると、lock にローカルパスが記録されて pin の意味が消える）。

```powershell
cd D:\USERFILES\GitHub\photocraft
git checkout -- Cargo.lock .cargo/config.toml     # まず patch を外す
# Cargo.toml の rev = "..." を新しい SHA に置き換える（craft-libs 由来の全行）
cargo update -p craft --precise <新しい SHA>
cargo check -p photocraft-ui-egui --lib           # 単体では feature 不足で落ちる場合あり
```

`Cargo.toml` の craft-libs 依存は**同じ rev が 39 行**並んでいる。1 行でも古い rev が残ると、そのクレートだけ別コミットになり整合性が壊れる。置換後に必ず数える（2026-11-18 実測：39 行）：

```powershell
(Select-String -Path Cargo.toml -Pattern 'craft-libs' | Measure-Object).Count
(Select-String -Path Cargo.toml -Pattern '<古い SHA>' | Measure-Object).Count   # 0 であること
```

上流（`storytold/photocraft`）へ PR を出す場合、`rev` がフォークの SHA を指したままだと上流から取得できない。上流に craft-libs 側の修正が入るまでは、PR の説明に依存関係を明記するか、上流の修正を待つ。

---

## 5. commit 前チェック（最重要）

**この 2 つは絶対に commit しない。**

| ファイル | patch on のときに起きること | commit するとどうなるか |
|---|---|---|
| `.cargo/config.toml` | `[patch]` 段が 201 行入る | clone した人の隣に `craft-libs` が無く、`cargo build` が即失敗する |
| `Cargo.lock` | craft-libs 由来 69 個の `source = "git+https://..."` 行が消える（77 → 8） | 依存の出所が不明になり、他環境で `cargo` が lock を勝手に書き換える |

commit の直前に必ず実行する：

```powershell
cd D:\USERFILES\GitHub\photocraft
cargo xtask craftlibs check       # 0 でなければ commit しない
git status --short
```

`check` が 0 を返せば両方とも pin の状態。1 を返したら、理由と該当クレートを並べて出すので、`cargo xtask craftlibs off` で戻す。

`Cargo.lock` の差分に `-source = "git+https://github.com/storytold/craft-libs` が並んでいたら、それは patch の副作用である。次で戻す：

```powershell
git checkout -- Cargo.lock .cargo/config.toml
```

戻すと git の pin でビルドする状態に戻る。作業内容（craft-libs 側の commit）は失われない。再開したくなったら §2 の patch を入れ直す。

**戻した直後の確認**：`Cargo.lock` に `source = "git+https://github.com/storytold/craft-libs?rev=` が戻っていること。期待値は **77 行**（patch 中は 69 行消えて 8 行になる。残る 8 行は patch していないクレート＝`krilla` / `hayro` 系など、craft-libs の git から取る外部依存）。

```powershell
(Select-String -Path Cargo.lock -Pattern 'craft-libs\?rev=' | Measure-Object).Count
```

---

## 6. 事故と対処

### `cargo check` が「path が見つからない」で落ちる
patch は入っているが craft-libs が無い。`D:\USERFILES\GitHub\craft-libs` の存在を確認する。別の場所に clone したなら patch の相対パスを直す（`../craft-libs/...`）。

### 同一クレートが 2 つあると言われる
patch のクレートが漏れている。§2 の生成スクリプトを回し直して全件入れ替える。

### craft-libs の `cargo test` で 1 件落ちる
`runtime::windows::a_palette_drops_its_level_while_the_app_is_inactive` は **未変更の原樹でも落ちる既存失敗**（2026-11-18 に `git stash` で対照実測済み）。変更のせいではない。切り分けは必ず stash で行う：

```powershell
cd D:\USERFILES\GitHub\craft-libs
git stash push -- <変更ファイル>
cargo test -p craft-ui-contract --test it -- <落ちたテスト名>
git stash pop
```

### PhotoCraft 側で `egui::Id: serde::Serialize` が大量に出る
単一パッケージ指定が原因。§2 の 6 パッケージ同時で回す。

### pin bump 後に挙動が変わった
patch が残ったまま `cargo update` した可能性がある。§5 のチェックで `Cargo.lock` の `source` 行数を確認する。

---

## 7. 現在の状態（2026-11-18）

- craft-libs ローカル：`D:\USERFILES\GitHub\craft-libs`、`a6b9d9c986f7c4c399632d72d9a4a8c8ea0b86e9` から分岐
- 送信済み：PR [#2204](https://github.com/storytold/craft-libs/pull/2204)（base は **`next`**、commit `9653cf55`、フォーク `ussoewwin/craft-libs`）
  - 内容：`crates/ui-contract/src/runtime/egui/response.rs` の `widget()` を `ui.interact` から `interact_opt(activate_on_space: false)` へ
  - 理由：egui は Space をフォーカス中ウィジェットのクリック（`FAKE_PRIMARY_CLICKED`）に変換し、その際フォーカスも捨てる。画布の Hand キーが直前に触ったボタンやパネル行を押してしまい、症状が「ときどき」になる。Enter は残すのでキーボード操作は維持
- 未解決：AccessKit のクラッシュ根本（`crates/ui-contract/src/runtime/access/emit.rs:260` が、ツリーに存在しない popup ノード id を `relations.controls` に入れている）。上流 `EPIC-UI-REWORK` の範囲
- PhotoCraft 側：`.cargo/config.toml` と `Cargo.lock` は **未 commit**（patch 状態のまま）

---

## 8. 道具（`cargo xtask craftlibs`）

手作業のうち間違えやすい 3 つは `xtask` のサブコマンドにしてある。`crates/ui-contract/src/runtime/` の下を見れば分かるとおり、**直すのは常に craft-libs 側**なので、この 3 つを回すのが日常の入口になる。

| コマンド | すること |
|---|---|
| `cargo xtask craftlibs status` | patch の有無、checkout の場所、pin の rev、checkout の HEAD、lock がローカル path かどうかを 1 画面で出す |
| `cargo xtask craftlibs on` | `../craft-libs/crates/*/Cargo.toml` を走査して `[patch]` を**生成**する（クレートが増えても漏れない） |
| `cargo xtask craftlibs off` | `[patch]` を削除し、`git checkout -- Cargo.lock .cargo/config.toml` で pin に戻す |
| `cargo xtask craftlibs check` | commit 前検査。patch が残っているか、lock に source 無しの craft-libs クレートがあるかを見て、あれば**非ゼロ終了**して該当クレートを列挙する |

### commit 前の必須手順

```powershell
cargo xtask craftlibs check      # 0 でなければ commit しない
```

`cargo xtask` 経由だと cargo 自身が patch の警告（`was not used in the crate graph`）を大量に出す。警告を避けたいときは exe を直接叩く：

```powershell
target\debug\xtask.exe craftlibs check
```

### 実測（2026-11-18）

- `craftlibs on` → `patched 201 craft-libs crates`
- `craftlibs status`（patch 中）→ `69 craft-libs crate(s) from local paths (NOT safe to commit)`
- `craftlibs off` → lock の `craft-libs?rev=` 行が 8 → **77** に戻り、`check` が `craft-libs is on the pin: safe to commit` で 0
- `craftlibs on` の直後（cargo 未実行）→ `status` は「次の cargo 実行で lock が書き換わる（NOT safe to commit）」と出す

### なぜ `on` が生成なのか

craft-libs のクレートは相互に path 依存している。1 つでも patch から漏れると、同じクレートが「git 由来」と「path 由来」の 2 つになり解決に失敗する。だから手書きの一覧ではなく、checkout の `crates/*/Cargo.toml` を毎回読んで生成する。

### 限界

`check` は**手元の commit を止めるだけ**で、他人の環境を守るものではない。リポジトリに入る防壁は CI しかないが、CI は feature 修正 PR で触らない決まりなので、必要なら別 issue で相談する。
