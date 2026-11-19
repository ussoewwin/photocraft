# Building PhotoCraft: the absolute rules

Read this file before every build. The gate below is not advisory: a build that skips it produces a
binary that silently lacks the fixes it is supposed to contain, and reporting it as done is a lie.

## Why this file exists

The same mistake was made twice. A release build was started with the local `craft-libs` patch
removed from `.cargo/config.toml`, so cargo built the **pinned** revision instead of the checkout the
fixes live in. The binary came out looking fine — right size, right time — and the bug the owner had
already reported came back with it (Space opening the File menu again, the menu bar click aborting the
process). Nothing in the build output said so.

The patch disappears whenever `cargo xtask craftlibs off` runs (it ends with
`git checkout -- Cargo.lock .cargo/config.toml`), and that command is easy to run while checking
something else and then forget.

## Never

1. **Never start a build before the gate passes.** All five checks, in order, every time.
2. **Never commit `.cargo/config.toml` or `Cargo.lock`.** With the patch in, the lock records local
   paths; committing it breaks every other checkout and CI.
3. **Never build after `cargo xtask craftlibs off`.** `off` exists for the moment of committing.
   Run `cargo xtask craftlibs on` immediately afterwards and re-run the gate.
4. **Never delete `target\release\photocraft.exe` or `target\release\photocraft.pdb`.** They are the
   deliverable. Deleting the exe costs a full rebuild and cannot be undone.
5. **Never report a build as finished without measuring it**: the exe's timestamp and size, and the
   `Finished` line from the log.
6. **Never start another build or a test run right after one finished.** The owner may not have used
   the result yet.
7. **Never run the app or the tests the owner said he would run.** Hand over the exe and the paths.

## The gate

Run these in the repository root. Every expected value is measured, not remembered.

```powershell
# 1. the patch is present: 211 crates from the checkout
(Select-String -Path .cargo\config.toml -Pattern 'path = "\.\./craft-libs' | Measure-Object).Count   # 211

# 2. the lock is on those local paths: 8 lines, down from 77 on the pin
(Select-String -Path Cargo.lock -Pattern 'craft-libs\?rev=' | Measure-Object).Count                  # 8

# 3. the checkout is there and carries the fixes being built
git -C ..\craft-libs log -1 --format='%h %s'

# 4. the same three commits are the ones in the branch this build is meant to carry
git -C ..\craft-libs status --short        # empty, or only files being edited on purpose
```

Anything other than `211` and `8` stops here. Restore the patch, do not build:

```powershell
# regenerate the [patch] section from the checkout (211 crates)
# see md/craft-libs-parallel-management.md §9 for the tooling
cargo xtask craftlibs on
(Select-String -Path .cargo\config.toml -Pattern 'path = "\.\./craft-libs' | Measure-Object).Count   # 211
```

`Cargo.lock` reaches `8` after the next cargo command runs (`cargo tree` is enough); `on` itself only
writes the `[patch]` section.

## The build

```powershell
$env:CRAFT_I18N_REQUIRED='1'
$env:CRAFT_I18N_DIR='D:\USERFILES\GitHub\craft-i18n'
$env:CRAFT_FONTS_DIR='D:\USERFILES\GitHub\photocraft\craft-fonts'
cargo build --release -p photocraft
```

## The check that catches the mistake anyway

The compiler names where every craft crate comes from. Read the log, do not assume:

| Line in the log | Meaning |
|---|---|
| `Compiling craft-ui-contract v0.1.0 (D:\USERFILES\GitHub\craft-libs\crates\ui-contract)` | **The patch is in use.** This is the build you wanted |
| `Compiling craft v0.1.0 (https://github.com/storytold/craft-libs?rev=…)` | **The pin is being built.** Stop: the patch is missing, the binary will not carry the fixes |

Do not wait for the build to finish when the second line appears: cancel it, run the gate, start over.

## After the build

```powershell
Get-Item target\release\photocraft.exe | Select-Object LastWriteTime, Length
Get-Content <log> | Select-String 'Finished'
```

Then clean only what the build does not need — `target\debug`, `target\release\deps`,
`target\release\build`, `target\release\.fingerprint`, `target\release\incremental`, and the logs.
Keep the exe and the pdb (see `md/craft-libs-parallel-management.md` §10).
