# Building PhotoCraft From This Repository — Complete Procedure

This document is the full, repeatable procedure for building the PhotoCraft desktop app
(`photocraft`) and its CLI (`photocraft-cli`) from this fork's `main`, without breaking the
user's environment. It reflects the incidents of the 2026-11-12 build session (a stale
0.22.0 exe reported as "the version is still 0.22", and a catalog-less rebuild that turned
the whole interface English) and every number and behavior below was measured on this tree
at `main` = `e9c09cec6` (upstream `v0.23.1` merged).

Two invariants govern everything:

1. **Never replace an exe the user launches with a degraded configuration.** The interface
   language and the bundled fonts are compile-time properties. A build without the catalogs
   is English-only; a build without craft-fonts loses the bundled script coverage. Replacing
   the daily-use exe with such a build is a functional regression, not a refresh.
2. **Measure, never assume.** "The build finished" is not "the right exe is in the right
   place with the right data inside". Every step below ends in a measured result; the
   verification gates in §6 are the definition of done.

---

## 1. The two build locations — know which exe the user actually runs

| Location | Who builds there | What lives there |
|---|---|---|
| `<repo>\target\release` | `start-photocraft.cmd` (the user's daily launcher) via `cargo run --release -p photocraft`, because the launcher **clears `CARGO_TARGET_DIR`** | The desktop exe the user actually starts |
| `<drive>:\pf_target\release` (a data drive, never the system drive) | the procedure in this document, via `CARGO_TARGET_DIR` | Isolated release builds and the CLI deliverable |

They are **independent caches**. A perfect build in one does not update the other. The
2026-11-12 incident: `pf_target` was rebuilt at 0.23.1 while the launcher's
`target\release\photocraft.exe` was still a 2026-11-11 build reporting **FileVersion
0.22.0** — measured, not guessed, via `(Get-Item <exe>).VersionInfo.FileVersion`. Before
building, decide which location must change, and say so.

Before either build, `main` must carry the content of every open pull-request branch — see the
"Iron rule" in `upstream-merge-procedure.md`. The daily exe is built from `main`, so a pull
request that exists only on its own branch is missing from the app the user runs; that is the
2026-11-12 font incident, not a build detail. The end of a build session reports the 0-byte
verification, one line per open pull-request branch.

Also know the cost of each location. Measured on 2026-11-12: the repository `target` holds
**20.01 GB** after the catalog rebuild (`target\release\deps` alone is 8.41 GB; the release
`incremental` directory stays empty at 0.00 GB); a fresh `pf_target` reached **~28 GB**
(debug test cache 24.26 GB + release 3.71 GB) after a full test-and-release cycle. Both are
regenerable; a release rebuild alone takes ~10–12 minutes (measured: 11m30s for
`-p photocraft -p photocraft-cli` cold on `pf_target`, 10m29s for `-p photocraft` cold on the
repository `target`, ~8 minutes for a rebuild forced by a change of `CRAFT_I18N_DIR`).

---

## 2. Environment variables and what actually happens when each is unset

Verified against `crates/ui-egui/build.rs` and `crates/text/build.rs`:

| Variable | Set (required values) | Unset — measured consequence |
|---|---|---|
| `CRAFT_I18N_DIR` | a craft-i18n checkout (storytold/craft-i18n) at the SHA in the repository root file `craft-i18n.rev` (currently `e02939ef009218e81c44e8ec001fbb7603461de4`) | `build.rs` embeds an empty catalog list and prints *"CRAFT_I18N_DIR: … the interface is English only"* — **the interface is English only** |
| `CRAFT_I18N_REQUIRED` | `1` in CI and releases: turns the missing-catalog case from a warning into a hard error | missing catalogs are only a warning; the build still succeeds and ships English-only |
| `CRAFT_FONTS_DIR` | the craft-fonts checkout (contains `fonts/manifest.txt`) | no bundled faces are compiled in; the app falls back to system fonts |
| `CARGO_TARGET_DIR` | the isolated data-drive directory for procedure builds | unset → cargo builds into `<repo>\target` (the launcher's location) |
| `CARGO_INCREMENTAL` | `0` for procedure builds | unset → cargo's default applies. The workspace `Cargo.toml` sets no `incremental` value (verified), and the measured `target\release\incremental` directory stays empty (0.00 GB) for release builds, so this variable is a safeguard for dev/test runs, not a release-size lever. The release target size is dominated by `deps` (measured 8.41 GB). |

`build.rs` declares `rerun-if-env-changed` for both `CRAFT_I18N_DIR` and `CRAFT_FONTS_DIR`.
Consequence (measured): **changing these variables between builds forces a rebuild of the
affected crates even with nothing else changed** (~8 minutes for the app crate chain on this
machine), and a rebuild *without* the variables silently produces the degraded configuration.

---

## 3. Prerequisites

1. **Rust** via rustup (stable, MSVC host; minimum 1.95) and a Visual Studio 2022 C++ toolset.
2. **craft-i18n checkout** for the catalogs. One-time setup, repeat whenever `craft-i18n.rev`
   changes:
   ```powershell
   git clone https://github.com/storytold/craft-i18n <craft-i18n-checkout>
   cd <craft-i18n-checkout>
   git checkout (Get-Content <repo>\craft-i18n.rev)
   ```
   Never edit this checkout to carry fork wording; fork-specific rows belong in the in-repo
   override tables (`JA_TEXT_OVERRIDES` / `JA_ID_OVERRIDES` in `crates/ui-egui/src/i18n.rs`,
   see `upstream-merge-procedure.md` §7.1).
3. **craft-fonts checkout** for the bundled faces (Japanese, Arabic, Thai, …). It is not
   tracked in this repository (`git ls-files craft-fonts` → 0 files); the local checkout
   inside the working tree is a convenience, and `CRAFT_FONTS_DIR` must point at it for
   builds that need the faces.
4. **Compiler environment normalization.** Wrong `INCLUDE`/`LIB`/`CL` leaking from a
   developer shell makes `aws-lc-sys` fail (`cl.exe` exit code 2). Clear them in every build
   shell (the launcher does exactly this):
   ```powershell
   $env:INCLUDE=$null; $env:LIB=$null; $env:CL=$null; $env:_CL_=$null
   $env:CC=$null; $env:CXX=$null; $env:CFLAGS=$null; $env:CXXFLAGS=$null; $env:RUSTFLAGS=$null
   ```

---

## 4. What `start-photocraft.cmd` does, exactly

Read it before trusting any description: it `cd`s to its own folder, clears the compiler
variables above **plus `CARGO_TARGET_DIR` and `CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS`**,
then runs `cargo run --release -p photocraft`.

Measured consequences:

- It builds into `<repo>\target\release` — **it does not touch `pf_target`**.
- It sets neither `CRAFT_I18N_DIR` nor `CRAFT_FONTS_DIR`. A launcher run therefore embeds
  catalogs **only if those variables are already present in the calling shell** (they are not
  persistent user/system variables on this machine — verified via `reg query HKCU\Environment`
  and the corresponding `HKLM` key). A bare shell run rebuilds the interface English-only the
  moment the env change invalidates the previous build.
- The first run compiles the whole workspace and takes ~10+ minutes.

To run the launcher **with** the full configuration, set the variables in that shell first:

```powershell
$env:CRAFT_I18N_DIR  = '<craft-i18n-checkout>'
$env:CRAFT_FONTS_DIR = '<craft-fonts-checkout>'
& 'D:\USERFILES\GitHub\photocraft\start-photocraft.cmd'
```

Changing the launcher script itself is a repository change and needs its own explicit
instruction; this document does not modify it.

---

## 5. The builds

### 5.1 Refresh the daily-use desktop exe (`<repo>\target\release\photocraft.exe`)

This is the correct way to replace what the user runs. Full configuration, launcher location:

```powershell
cd <repo>
# compiler normalization from §3, then:
$env:CARGO_TARGET_DIR=$null        # build where the launcher builds
$env:CRAFT_I18N_REQUIRED='1'       # a missing catalog must fail the build, not warn
$env:CRAFT_I18N_DIR  = '<craft-i18n-checkout>'
$env:CRAFT_FONTS_DIR = '<craft-fonts-checkout>'
cargo build --release -p photocraft
```

Measured on 2026-11-12: exit 0, 10m29s cold; the catalog+fonts rebuild after an
English-only build was ~8 minutes. Result: `photocraft.exe` 161.8 MB (169,659,392 B),
created 16:43, `FileVersion` 0.23.1. Skip `CRAFT_FONTS_DIR` only if the bundled faces are
genuinely not wanted — never by omission.

### 5.2 Isolated release build (deliverable + CLI) on the data drive

```powershell
cd <repo>
# compiler normalization from §3, then:
$env:CARGO_TARGET_DIR   = '<drive>:\pf_target'
$env:CARGO_INCREMENTAL  = '0'
$env:CRAFT_I18N_REQUIRED = '1'      # a release must embed the translations
$env:CRAFT_I18N_DIR     = '<craft-i18n-checkout>'
$env:CRAFT_FONTS_DIR    = '<craft-fonts-checkout>'
cargo build --release -p photocraft -p photocraft-cli
```

Measured on 2026-11-12: exit 0, 11m30s cold. Artifacts land in
`<drive>:\pf_target\release\` — `photocraft.exe` 162.1 MB, `photocraft-cli.exe` 118.9 MB.
This is the release deliverable location; `upstream-merge-procedure.md` §12 covers the
release/tag steps that may follow, which need their own explicit instruction.

---

## 6. Verification gates — all must pass, measured, never assumed

After every build, run all of these. They are cheap; skipping them is how the English-only
and 0.22.0 incidents happened.

```powershell
# (a) The build ended clean.
#     The log must contain "Finished `release` profile [optimized]" and exit code 0.

# (a2) The build did not fall back to English only.
#      crates/ui-egui/build.rs prints this exact warning when the catalogs are missing
#      or unreadable; with CRAFT_I18N_REQUIRED=1 the same case is a hard error instead.
#      The log must NOT contain it.
if (Select-String -Path <build-log> -Pattern 'the interface is English only' -Quiet) {
    'FAIL: the build embedded no catalogs — do not ship this exe'
}

# (b) The artifacts exist, with fresh timestamps (an old mtime = you are looking at a stale exe).
Get-Item <exe> | Select-Object Name, Length, LastWriteTime
#     Reference for a catalog+fonts desktop build: photocraft.exe, 169,659,392 B (161.8 MB).
#     A catalog-less build of the same tree measured 110.9 MB — the size itself is a first signal.

# (c) The exe metadata reports the expected version (the 0.22.0 incident detector).
(Get-Item <exe>).VersionInfo.FileVersion      # expected: 0.23.1 on this tree

# (d) The CLI reports its version (GUI executables are never launched as a check).
<target-dir>\release\photocraft-cli.exe --version    # measured: "photocraft-cli 0.23.1 (dev build)"

# (e) Catalogs are embedded: byte-match Japanese catalog rows against the exe (measured
#     reference run on the refreshed launcher exe: 3377/3377 rows of
#     locales/ja/photocraft.tsv found inside the exe bytes).
#     `python` on PATH measured 3.13.15; any Python 3 works, e.g. the embedded interpreter
#     at <ComfyUI>\python_embeded\python.exe.
python -X utf8 -c "
import sys; sys.stdout.reconfigure(encoding='utf-8')
data = open(r'<exe>', 'rb').read()
total = hit = 0
for l in open(r'<craft-i18n>\locales\ja\photocraft.tsv', encoding='utf-8', errors='replace'):
    parts = l.rstrip('\r\n').split('\t')
    if len(parts) < 2: continue
    s = parts[-1].strip().encode('utf-8')
    if len(s) < 3: continue
    total += 1
    if s in data: hit += 1
print('catalog rows in exe:', hit, '/', total)   # must be total/total, never a subset"

# (f) Bundled faces are embedded (measured markers).
python -X utf8 -c "
data = open(r'<exe>', 'rb').read()
print('Shippori Mincho:', b'Shippori Mincho' in data)
print('BIZ UDPGothic:', b'BIZ UDPGothic' in data)"
```

Gate (e) failing with a partial count, or (f) printing `False`, means the build silently
dropped catalogs or faces — **do not deliver it, do not leave it in a location the user
launches from**. Rebuild per §5.1 and re-run the gates.

---

## 7. Incident catalog — what actually went wrong, and the rule each incident wrote

| Incident | What happened | The rule |
|---|---|---|
| "The version is still 0.22" | The user's launcher exe (`target\release`) was a 2026-11-11 build with `FileVersion 0.22.0`; a fresh 0.23.1 build landed in `pf_target`, which the launcher never reads. | Identify the exe the user actually launches **before** building; build into that location when refreshing it, then verify gate (c). |
| The whole interface turned English | The launcher-location rebuild was done without `CRAFT_I18N_DIR`, silently replacing a catalog-embedded exe with an English-only one. The manual even documented the behavior ("§2.5: unset = English only") and the rebuild was still made that way. | An exe the user launches is only ever replaced by a build that passes gates (e) and (f). Knowing the trap and walking into it anyway is the same crime as not knowing. |
| The Japanese font feature vanished from the app | The launcher exe was rebuilt from a `main` that did not carry the font pull request; the feature existed only on its branch. | Every pull-request content is merged into `main` before any rebuild, and the 0-byte verification of `upstream-merge-procedure.md` ("Iron rule") is reported with the build. |
| The English-only state persisted one build longer than necessary | The gap was reported as a fact ("the launcher does not set the variable") instead of being fixed immediately, under "changing it was not instructed". | Reporting a known regression is not a substitute for restoring the working state; restoration of what you broke is always in scope of the instruction that caused it. |
| Build-wait noise | Repeated manual sleep-and-poll loops while a registered completion monitor was already watching the log. | Trust the registered monitor; do one completion check instead of polling loops. |

---

## 8. Cleanup and SSD discipline

- Scratch logs are deleted in the same session (`build_*.log` on the drive root, temp
  scripts in `%TEMP%`). Measured debris from one session: 77 log files and 259 temp
  scripts — all removable, all removed.
- Build caches are regenerable but large. Keep `target\release` (the current deliverable or
  the launcher exe) unless the instruction is explicitly to delete it; a deleted cache costs
  ~10–12 minutes to rebuild.
- A "cleanup" that deletes something of unknown ownership is forbidden. Measure first
  (`Get-ChildItem`, sizes, mtimes), name every item, delete only what is provably this
  session's debris, and report what was deliberately left alone.

---

## 9. Final checklist

- [ ] Decided which build location must change; said so before building
- [ ] Compiler environment normalized (`INCLUDE`/`LIB`/`CL`/`_CL_`/`CC`/`CXX`/`CFLAGS`/`CXXFLAGS`/`RUSTFLAGS` cleared)
- [ ] `CRAFT_I18N_DIR` set to the checkout at the `craft-i18n.rev` SHA — never a bare build over a working exe
- [ ] `CRAFT_FONTS_DIR` set (or its omission is an explicit, stated decision)
- [ ] Build ends with `Finished release profile` and exit 0
- [ ] Build log does not contain "the interface is English only"
- [ ] Artifacts fresh (mtime), expected sizes
- [ ] `FileVersion` = expected version
- [ ] CLI smoke test passed (`--version`)
- [ ] Catalog gate: `3377/3377`-style full byte match
- [ ] Fonts gate: `Shippori Mincho` / `BIZ UDPGothic` found
- [ ] Scratch logs deleted; caches and checkouts kept; nothing of unknown ownership touched
- [ ] Nothing pushed, tagged, or published without an explicit instruction
