# craft-libs 並行管理

PhotoCraft の UI フレームワーク（キー配送、フォーカス、AccessKit）は共有ライブラリ `storytold/craft-libs` にある。PhotoCraft 側だけで完結する修正と違い、**直すべきコードが別リポジトリにある**ので、次の 3 つを常に同期させる。

| 対象 | 役割 | commit |
|---|---|---|
| `photocraft/Cargo.toml` の `rev` | どの craft-libs を使うかの正本（全員に効く） | **する** |
| `photocraft/.cargo/config.toml` の `[patch]` | 手元の craft-libs を指す開発装置 | **しない** |
| `photocraft/Cargo.lock` | 依存の出所の記録（patch 中はローカルパスになる） | **しない** |

この 3 つの関係を取り違えると「手元では動くが CI と他の開発者が即死する」状態になる。

---

## 1. 前提：PhotoCraft が追うのは `next`

`docs/craft-framework.md`（craft-libs 側）が方針を定めている。

> The framework lands on one storytold branch, `next`, and reaches `main` in one pull request at the end.
> An app keeps its `craft-libs` pin on `main` until its port is done. **It then moves its pin to a `next` commit**, and to `main` after `next` merges.

PhotoCraft は PR #15449 で新フレームワークへ移植したので、`next` のコミットを追う。

実測（2026-11-19 時点）：

| 項目 | 値 |
|---|---|
| 上流 `next` の先端 | `7a7f5302` |
| 上流 `main` | `next` と別系統。`crates/ui-contract` を持たない |
| PhotoCraft の `Cargo.toml` の pin | `a6b9d9c986f7c4c399632d72d9a4a8c8ea0b86e9`（`next` の 1895 コミット手前） |

**`main` を base にした PR は作れない。** 直すコードが `main` に存在しないため。

---

## 2. リポジトリの構成

```
D:\USERFILES\GitHub\
├── photocraft\      フォーク ussoewwin/photocraft、ブランチは main のみ
└── craft-libs\      フォーク ussoewwin/craft-libs、ブランチは next のみ
```

craft-libs のフォークは **`next` だけ**を持つ。`main` は持たない（デフォルトブランチも `next`）。理由は §1 のとおりで、PhotoCraft が使うのは `next` であり、`main` を置いても同じ内容の重複になるだけだから。

`origin` は `storytold/craft-libs`、`fork` は `ussoewwin/craft-libs`。push 先は **`fork` の `next`**。

---

## 3. 日常：ローカルの修正を PhotoCraft で動かす

craft-libs の checkout を `next` に置き、patch を入れる。

```powershell
cd D:\USERFILES\GitHub\craft-libs
git switch next && git pull fork next

cd D:\USERFILES\GitHub\photocraft
cargo xtask craftlibs on
```

`on` は checkout の `crates/*/Cargo.toml` を走査して `[patch]` を**生成**する。クレートが上流で増えても漏れない（2026-11-19 実測：211 クレート）。

```powershell
cargo xtask craftlibs status
```

```
workspace:   D:\USERFILES\GitHub\photocraft
checkout:    D:\USERFILES\GitHub\craft-libs (present)
.cargo/config.toml: PATCHED (local craft-libs)
Cargo.lock:  69 craft-libs crate(s) from local paths (NOT safe to commit)
pinned rev:  a6b9d9c986f7c4c399632d72d9a4a8c8ea0b86e9
checkout at: 8fbb0720
```

**効いているかの確認**は `cargo check` の成功だけでは足りない。実際にローカルが使われたかを見る。

```powershell
cargo tree -p photocraft-ui-egui -e normal --depth 0
```

- `craft` / `craft-ui-contract` / `craft-ui-canvas` がローカル解決なら成功
- `warning: patch 'craft-xxx ...' was not used in the crate graph` は**そのクレートが依存グラフに無いだけ**で無害（2026-11-19 実測：211 クレート中 147 件）

### PhotoCraft のテストを回すとき

単一パッケージ指定は feature 解決が足りずに落ちる（`egui::Id: serde::Serialize` が 13〜36 件）。**6 パッケージ同時**で回す。

```powershell
$env:CRAFT_I18N_REQUIRED='1'
$env:CRAFT_I18N_DIR='D:\USERFILES\GitHub\craft-i18n'
$env:CRAFT_FONTS_DIR='D:\USERFILES\GitHub\photocraft\craft-fonts'
cargo test --lib -p photocraft-ui-egui -p photocraft-engine -p photocraft `
  -p photocraft-cli -p photocraft-text -p photocraft-automation
```

---

## 4. 修正を craft-libs の `next` に送る

craft-libs のローカルは `next` を追っているので、そのまま commit して push する。`next` は **fast-forward で積む**ブランチで、force-push はしない（`docs/craft-framework.md`）。

```powershell
cd D:\USERFILES\GitHub\craft-libs
git fetch origin next
git rebase origin/next            # 上流が進んでいたら載せ替える
git add <変更したファイル>          # git add -A は使わない
```

### commit の前に必ず署名を確認する

この実行環境は `GIT_AUTHOR_*` / `GIT_COMMITTER_*` を `zwork <zwork@local>` に上書きしており、**環境変数は `git -c user.name=...` より優先される**。実際に 2 回この事故を起こしている（amend + force-with-lease で修復）。

commit の直前に、同じコマンド内で環境変数まで揃える。

```powershell
$env:GIT_AUTHOR_NAME='ussoewwin'
$env:GIT_AUTHOR_EMAIL='136552381+ussoewwin@users.noreply.github.com'
$env:GIT_COMMITTER_NAME=$env:GIT_AUTHOR_NAME
$env:GIT_COMMITTER_EMAIL=$env:GIT_AUTHOR_EMAIL
git commit -m "<何を直したか>"
git log -1 --format='%an <%ae> | %cn <%ce>'   # 実測で確認。zwork ならまだ足りない
```

### push と確認

```powershell
git push fork next
git ls-remote fork refs/heads/next   # ローカルの HEAD と同じ SHA か
```

### 署名を間違えたまま push してしまったら

```powershell
# 直前の 1 commit だけなら
git commit --amend --reset-author --no-edit

# 複数 commit の committer をまとめて直す
git rebase origin/next --exec 'git commit --amend --no-edit --reset-author'

git push --force-with-lease fork next
```

`--force-with-lease` は必ず「期待するリモート SHA」を確認してから使う。stale で弾かれたら `git fetch fork` してから再実行する。

---

## 5. commit するとき（patch を外す）

**patch と、それが書き換えた lock は commit しない。** commit すると、clone した人の隣に `craft-libs` が無いので `cargo build` が即死する。

```powershell
cd D:\USERFILES\GitHub\photocraft
cargo xtask craftlibs check      # 0 でなければ commit しない
cargo xtask craftlibs off        # patch を削除し、lock を pin に戻す
git status --short               # .cargo/config.toml と Cargo.lock が消えていること
```

`off` は `git checkout -- Cargo.lock .cargo/config.toml` を実行する。作業内容（craft-libs 側の commit）は失われない。

`check` は次を見て、危険なら非ゼロ終了して該当クレートを列挙する。

- `.cargo/config.toml` に `[patch."https://github.com/storytold/craft-libs"]` が残っている
- `Cargo.lock` に `source` の無い craft-libs クレートがある

**戻した直後の確認**：lock の `craft-libs?rev=` 行が 77 行に戻っていること（patch 中は 8 行）。

```powershell
(Select-String -Path Cargo.lock -Pattern 'craft-libs\?rev=' | Measure-Object).Count
```

commit が終わったら、作業を続けるために `cargo xtask craftlibs on` で戻す。

---

## 6. 上流にマージされたら rev を上げる

fork の `next` のコミットは、上流 `storytold/craft-libs` の `next` にマージされるまで **`Cargo.toml` に書けない**（上流にその SHA が無く、`cargo` が取得に失敗する）。

だから rev を上げるのは、上流に入った後：

1. 上流 `next` に自分の commit が入ったことを確認する
2. `Cargo.toml` の craft-libs 依存の `rev` を、その SHA に置き換える（**35 箇所すべて**。craft-libs を含む行は 39 行あるが、`rev` を持つのは 35 行）
3. `cargo update -p craft --precise <SHA>`
4. `cargo xtask craftlibs check` が 0 を返すこと、`Cargo.lock` が新しい rev を指すことを確認
5. commit & push

置換漏れの確認：

```powershell
(Select-String -Path Cargo.toml -Pattern 'craft-libs' | Measure-Object).Count   # 35
(Select-String -Path Cargo.toml -Pattern '<古い SHA>' | Measure-Object).Count   # 0
```

1 行でも古い rev が残ると、そのクレートだけ別コミットになり整合性が壊れる。

---

## 7. 事故と対処

| 症状 | 原因 | 対処 |
|---|---|---|
| `cargo check` が「path が見つからない」 | patch はあるが checkout が無い | `D:\USERFILES\GitHub\craft-libs` の存在を確認。別の場所なら `[patch]` の相対パスを直す |
| 同一クレートが 2 つあると言われる | patch のクレートが漏れている | `cargo xtask craftlibs on` を回し直す（生成なので漏れない） |
| `egui::Id: serde::Serialize` が大量に出る | 単一パッケージ指定 | §3 の 6 パッケージ同時で回す |
| craft-libs のテストが 1 件落ちる | 未変更の原樹でも落ちる既存失敗 | `git stash push -- <ファイル>` で対照実測してから判断する |
| commit が `zwork` 名義になった | 環境変数の上書き | §4 の手順で amend / rebase して force-with-lease |
| pin bump 後も挙動が変わらない | patch が残ったまま `cargo update` した | `cargo xtask craftlibs status` で `PATCHED` を確認し、`off` してからやり直す |

---

## 8. 現在の状態（2026-11-19）

| 項目 | 値 |
|---|---|
| craft-libs の checkout | `D:\USERFILES\GitHub\craft-libs`、ブランチ `next`、先端 `8fbb0720` |
| フォーク | `ussoewwin/craft-libs`、**`next` のみ**、デフォルトブランチ `next`、`main` は削除 |
| 送信済み（fork の `next`） | `8fbb0720` ← `e875a7c9` ← `f7fc484a`（上流 `next` `7a7f5302` の直上に rebase 済み） |
| PhotoCraft の pin | `a6b9d9c9`（上流の正本。§6 の條件が揃うまで上げない） |
| PhotoCraft の patch | **適用中**（211 クレート）。commit 時は `cargo xtask craftlibs off` |
| PhotoCraft のブランチ | `ussoewwin/photocraft` の `main`、先端 `50026dac3d` |

送信済みの 3 commit：

1. `f7fc484a` — キャンバスが Space を取り戻す（`activate_on_space: false`）
2. `e875a7c9` — AccessKit の `controls` に、同じ更新にノードの無い ID を入れない
3. `8fbb0720` — **Space は決してアクティベーションにならない**（`KeyUse::of(Pointer)` の既定が `ActivateKeys::Both` だったのが、メニューが開く真因）

未解決：上流のレビュー待ち。マージされたら §6 で rev を上げる。

---

## 9. 道具（`cargo xtask craftlibs`）

| コマンド | すること |
|---|---|
| `status` | patch の有無、checkout の場所と HEAD、pin の rev、lock がローカルパスかを 1 画面で出す |
| `on` | `../craft-libs/crates/*/Cargo.toml` を走査して `[patch]` を生成する |
| `off` | `[patch]` を削除し、`git checkout -- Cargo.lock .cargo/config.toml` |
| `check` | commit 前検査。危険なら非ゼロ終了して該当クレートを列挙する |

`cargo xtask` 経由だと cargo 自身が patch の警告を大量に出す。避けたいときは exe を直接叩く。

```powershell
target\debug\xtask.exe craftlibs check
```

**限界**：`check` は手元の commit を止めるだけで、他人の環境は守らない。リポジトリ側の防壁は CI だが、CI は feature / fix の PR で触らない決まりなので、必要なら別 issue の案件。
