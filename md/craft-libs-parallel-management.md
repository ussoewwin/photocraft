# Managing craft-libs in parallel

PhotoCraft's UI framework (key delivery, focus, AccessKit) lives in the shared library `storytold/craft-libs`. Unlike a fix that can be finished inside PhotoCraft, **the code to fix is in another repository**, so these three always have to stay in sync.

| Target | Role | Commit |
|---|---|---|
| the `rev` in `photocraft/Cargo.toml` | The source of truth for which craft-libs is used (it applies to everyone) | **yes** |
| the `[patch]` in `photocraft/.cargo/config.toml` | A development device pointing at the local craft-libs | **no** |
| `photocraft/Cargo.lock` | The record of where the dependencies come from (a local path while patched) | **no** |

Get the relationship between these three wrong and you end up in a state where "it works on my machine but CI and every other developer die instantly".

---

## 1. Premise: PhotoCraft tracks `next`

`docs/craft-framework.md` (on the craft-libs side) sets the policy.

> The framework lands on one storytold branch, `next`, and reaches `main` in one pull request at the end.
> An app keeps its `craft-libs` pin on `main` until its port is done. **It then moves its pin to a `next` commit**, and to `main` after `next` merges.

PhotoCraft was ported to the new framework in PR #15449, so it tracks commits on `next`.

Measured (as of 2026-11-19):

| Item | Value |
|---|---|
| The tip of upstream `next` | `7a7f5302` |
| Upstream `main` | A different line from `next`. It has no `crates/ui-contract` |
| The pin in PhotoCraft's `Cargo.toml` | `a6b9d9c986f7c4c399632d72d9a4a8c8ea0b86e9` (1,895 commits behind `next`) |

**You cannot open a PR based on `main`.** Because the code to fix does not exist on `main`.

---

## 2. Repository layout

```
D:\USERFILES\GitHub\
├── photocraft\      fork ussoewwin/photocraft, only the main branch
└── craft-libs\      fork ussoewwin/craft-libs, only the next branch
```

The craft-libs fork holds **only `next`**. It does not hold `main` (the default branch is `next` as well). The reason is as in §1: PhotoCraft uses `next`, and keeping a `main` would only duplicate the same content.

`origin` is `storytold/craft-libs`, `fork` is `ussoewwin/craft-libs`. Pushes go to **`next` on `fork`**.

---

## 3. Day to day: running a local fix in PhotoCraft

Put the craft-libs checkout on `next` and apply the patch.

```powershell
cd D:\USERFILES\GitHub\craft-libs
git switch next && git pull fork next

cd D:\USERFILES\GitHub\photocraft
cargo xtask craftlibs on
```

`on` scans the checkout's `crates/*/Cargo.toml` and **generates** the `[patch]`. Nothing is missed when upstream adds crates (measured 2026-11-19: 211 crates).

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

**Checking that it is in effect** takes more than a successful `cargo check`. Look at whether the local copy was actually used.

```powershell
cargo tree -p photocraft-ui-egui -e normal --depth 0
```

- Success means `craft` / `craft-ui-contract` / `craft-ui-canvas` resolve locally
- `warning: patch 'craft-xxx ...' was not used in the crate graph` only means **that crate is not in the dependency graph**, and is harmless (measured 2026-11-19: 147 of the 211 crates)

### Running PhotoCraft's tests

Naming a single package fails because feature resolution is not sufficient (`egui::Id: serde::Serialize` in 13–36 places). Run **six packages at once**.

```powershell
$env:CRAFT_I18N_REQUIRED='1'
$env:CRAFT_I18N_DIR='D:\USERFILES\GitHub\craft-i18n'
$env:CRAFT_FONTS_DIR='D:\USERFILES\GitHub\photocraft\craft-fonts'
cargo test --lib -p photocraft-ui-egui -p photocraft-engine -p photocraft `
  -p photocraft-cli -p photocraft-text -p photocraft-automation
```

---

## 4. Sending a fix to `next` in craft-libs

The local craft-libs tracks `next`, so commit and push it as it is. `next` is a branch you **stack with fast-forwards**, and you never force-push it (`docs/craft-framework.md`).

```powershell
cd D:\USERFILES\GitHub\craft-libs
git fetch origin next
git rebase origin/next            # replay onto it if upstream has moved
git add <the files you changed>   # do not use git add -A
```

### Always verify the signature before committing

This execution environment overrides `GIT_AUTHOR_*` / `GIT_COMMITTER_*` with `zwork <zwork@local>`, and **the environment variables take precedence over `git -c user.name=...`**. This accident has actually happened twice (repaired with amend + force-with-lease).

Right before committing, set the environment variables too, inside the same command.

```powershell
$env:GIT_AUTHOR_NAME='ussoewwin'
$env:GIT_AUTHOR_EMAIL='136552381+ussoewwin@users.noreply.github.com'
$env:GIT_COMMITTER_NAME=$env:GIT_AUTHOR_NAME
$env:GIT_COMMITTER_EMAIL=$env:GIT_AUTHOR_EMAIL
git commit -m "<what you fixed>"
git log -1 --format='%an <%ae> | %cn <%ce>'   # check it by measurement. zwork means it is still not enough
```

### Pushing and verifying

```powershell
git push fork next
git ls-remote fork refs/heads/next   # is it the same SHA as the local HEAD
```

### If you pushed with the wrong signature

```powershell
# for the last single commit only
git commit --amend --reset-author --no-edit

# fix the committer of several commits at once
git rebase origin/next --exec 'git commit --amend --no-edit --reset-author'

git push --force-with-lease fork next
```

Always confirm the expected remote SHA before using `--force-with-lease`. If it rejects you as stale, `git fetch fork` and run it again.

---

## 5. When committing (removing the patch)

**Do not commit the patch, nor the lock that it rewrote.** If you do, anyone who clones has no `craft-libs` next to it, and `cargo build` dies instantly.

```powershell
cd D:\USERFILES\GitHub\photocraft
cargo xtask craftlibs check      # do not commit unless this is 0
cargo xtask craftlibs off        # remove the patch and return the lock to the pin
git status --short               # .cargo/config.toml and Cargo.lock should be gone
```

`off` runs `git checkout -- Cargo.lock .cargo/config.toml`. Your work (the commit on the craft-libs side) is not lost.

`check` looks at the following and, if it is dangerous, exits non-zero listing the crates concerned.

- A `[patch."https://github.com/storytold/craft-libs"]` is left in `.cargo/config.toml`
- `Cargo.lock` has a craft-libs crate with no `source`

**Verifying right after reverting**: the `craft-libs?rev=` lines in the lock are back to 77 lines (8 lines while patched).

```powershell
(Select-String -Path Cargo.lock -Pattern 'craft-libs\?rev=' | Measure-Object).Count
```

Once the commit is done, turn the patch back on with `cargo xtask craftlibs on` to keep working.

---

## 6. Raising the rev once it is merged upstream

A commit on the fork's `next` **cannot be written into `Cargo.toml`** until it is merged into `next` on upstream `storytold/craft-libs` (the SHA does not exist upstream, and `cargo` fails to fetch it).

So raise the rev only after it has landed upstream:

1. Confirm that your commit is in upstream `next`
2. Replace the `rev` of the craft-libs dependencies in `Cargo.toml` with that SHA (**all 35 places**. 39 lines mention craft-libs, but only 35 carry a `rev`)
3. `cargo update -p craft --precise <SHA>`
4. Confirm that `cargo xtask craftlibs check` returns 0 and that `Cargo.lock` points at the new rev
5. commit & push

Checking for missed replacements:

```powershell
(Select-String -Path Cargo.toml -Pattern 'craft-libs' | Measure-Object).Count   # 35
(Select-String -Path Cargo.toml -Pattern '<old SHA>' | Measure-Object).Count    # 0
```

Leave even one line on the old rev and that one crate comes from a different commit, breaking consistency.

---

## 7. Accidents and how to handle them

| Symptom | Cause | Action |
|---|---|---|
| `cargo check` says "path not found" | The patch is there but the checkout is not | Check that `D:\USERFILES\GitHub\craft-libs` exists. If it is elsewhere, fix the relative path in `[patch]` |
| It says the same crate exists twice | The patch is missing a crate | Run `cargo xtask craftlibs on` again (it is generated, so nothing is missed) |
| `egui::Id: serde::Serialize` appears in bulk | A single package was named | Run the six packages together as in §3 |
| One craft-libs test fails | An existing failure that also fails on the unmodified tree | Measure the control case with `git stash push -- <file>` before deciding |
| The commit is attributed to `zwork` | The environment variables override it | amend / rebase as in §4, then force-with-lease |
| Behaviour does not change after a pin bump | `cargo update` ran while the patch was still in place | Check for `PATCHED` with `cargo xtask craftlibs status`, run `off`, then do it again |

---

## 8. Current state (2026-11-19)

| Item | Value |
|---|---|
| The craft-libs checkout | `D:\USERFILES\GitHub\craft-libs`, branch `next`, tip `8fbb0720` |
| The fork | `ussoewwin/craft-libs`, **`next` only**, default branch `next`, `main` deleted |
| Sent (the fork's `next`) | `8fbb0720` ← `e875a7c9` ← `f7fc484a` (already rebased directly on top of upstream `next` `7a7f5302`) |
| PhotoCraft's pin | `a6b9d9c9` (the upstream source of truth. Not raised until the conditions of §6 are met) |
| PhotoCraft's patch | **Applied** (211 crates). Run `cargo xtask craftlibs off` when committing |
| PhotoCraft's branch | `main` of `ussoewwin/photocraft`, tip `50026dac3d` |

The three commits already sent:

1. `f7fc484a` — the canvas gets Space back (`activate_on_space: false`)
2. `e875a7c9` — do not put IDs into AccessKit's `controls` that have no node in the same update
3. `8fbb0720` — **Space never becomes an activation** (the default of `KeyUse::of(Pointer)` being `ActivateKeys::Both` was the real cause of the menu opening)

Open: waiting on upstream review. Once it is merged, raise the rev as in §6.

---

## 9. Tooling (`cargo xtask craftlibs`)

| Command | What it does |
|---|---|
| `status` | Shows on one screen whether the patch is present, where the checkout is and its HEAD, the pinned rev, and whether the lock uses local paths |
| `on` | Scans `../craft-libs/crates/*/Cargo.toml` and generates the `[patch]` |
| `off` | Removes `[patch]`, then `git checkout -- Cargo.lock .cargo/config.toml` |
| `check` | The pre-commit check. If it is dangerous it exits non-zero and lists the crates concerned |

Going through `cargo xtask` makes cargo itself emit a flood of patch warnings. To avoid them, call the exe directly.

```powershell
target\debug\xtask.exe craftlibs check
```

**Limit**: `check` only stops your own commit; it does not protect anyone else's environment.
---

## 10. Clean-up (temporary files only. Do not delete build artefacts)

**Do not delete build artefacts.** `target\release\photocraft.exe` is the artefact, and deleting it makes the rebuild take 20–30 minutes. `target\` stays.

Only temporary files that contribute nothing to the build may be deleted.

| To delete | Examples |
|---|---|
| Investigation logs and diagnostic output | `target\diag\*`, `*-log.txt`, `space-log.txt` |
| Investigation scripts | throwaway `*.cmd` |
| Backups of settings and the lock | `*.bak-*`, `Cargo.lock.before-*` |
| PR description drafts | `md/pr-*.md` (after submitting) |

| Never delete | Reason |
|---|---|
| `target\release\photocraft.exe` | The artefact itself. Deleting it means a 20–30 minute rebuild |
| `target\debug` | **Test binaries are not artefacts. Delete them.** Create them only when running tests, and delete them when done |
| `target\wt-*` | Another session's worktree |
| someone else's `md/`, `plugins/` | You did not create them |

### When disk is tight

First delete the "to delete" items above and `target\debug` (the test binaries). If that is still not enough, **ask the owner whether `target\release` may be deleted**. The exe is the artefact, so never delete it on your own judgement.

```powershell
# measure what takes how much
Get-ChildItem target -Directory | ForEach-Object {
  $s = (Get-ChildItem $_.FullName -Recurse -File -Force -ErrorAction SilentlyContinue | Measure-Object Length -Sum).Sum
  [pscustomobject]@{ Name = $_.Name; GB = [math]::Round($s/1GB, 2) }
} | Sort-Object GB -Descending
```

### Right after a build

**Do not start another build or test right after a build has finished.** The owner may not have used that build's result yet. If a test is needed, ask whether you may run it.

---

## 11. Keep the fork's Actions disabled (a fork feature)

**Do not run GitHub Actions in this repository's fork.** A fork has no secrets, cannot reach stepsecurity's monitor, and `Workspace / lint` always fails. It only adds red marks and protects nothing.

Treat the disabling as **part of this fork's features**. It is not over once it is set: **check it and put it back every time you take in upstream**.

### Current state (measured 2026-11-19)

| Repository | Actions permission | Workflows |
|---|---|---|
| `ussoewwin/craft-libs` | `enabled: false` | all 30 `disabled_manually` |
| `ussoewwin/photocraft` | `enabled: false` | all 10 `disabled_manually` |

### Why they come back

When a merge / rebase of upstream brings in new `.github/workflows/*.yml` files, GitHub registers those workflows as `active` again. The repository's Actions permission (`enabled`) stays, but **the new files come back enabled**.

### Always run this right after taking in upstream

```powershell
$env:GH_TOKEN = ((('protocol=https`nhost=github.com`n') | git credential fill) 2>$null |
  Select-String '^password=(.*)$').Matches[0].Groups[1].Value

foreach ($r in @('ussoewwin/craft-libs','ussoewwin/photocraft')) {
  # 1. disable the permission
  gh api -X PUT "repos/$r/actions/permissions" -F enabled=false | Out-Null
  # 2. disable every workflow
  gh api "repos/$r/actions/workflows?per_page=100" --jq '.workflows[] | .id' | ForEach-Object {
    gh api -X PUT "repos/$r/actions/workflows/$_/disable" | Out-Null
  }
  # 3. stop anything running
  gh api "repos/$r/actions/runs?per_page=20" --jq '.workflow_runs[] | select(.status=="in_progress" or .status=="queued") | .id' | ForEach-Object {
    gh api -X POST "repos/$r/actions/runs/$_/cancel" | Out-Null
  }
  # 4. verify
  gh api "repos/$r/actions/permissions" --jq '.enabled'
  gh api "repos/$r/actions/workflows?per_page=100" --jq '[.workflows[] | .state] | group_by(.) | map({(.[0]): length}) | add'
}
```

Expected output: `false` and `{"disabled_manually": N}` (N is the total number of workflows, with `active` at 0).

### Forbidden

- Do not run workflows in the fork. Do not use `gh workflow run` either
- Do not assume "it is fine because it was disabled once". Always run the four steps above after taking in upstream
- Do not undo the disabling. If undoing it is needed, confirm the reason with the owner
