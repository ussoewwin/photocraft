# LaMa Cleaner CUDA Support — Technical Reference

This document explains the LaMa Cleaner GPU (CUDA) support end to end: `paint.lamaCleaner`. Everything is Rust: inference runs in the helper process `plugins/lama-cleaner` through ONNX Runtime (`ort` 2.0.0-rc.9). The engine itself does not link `ort`; it only starts the helper with `std::process::Command`.

- Commits: `049063b90` → `85c1b5a1c` → `b14e2651a` (landed on main through `5810f9063` and `72827c369`)
- Main files: `plugins/lama-cleaner/src/lib.rs` (inference and provider selection), `plugins/lama-cleaner/src/main.rs` (CLI and the resident loop), `crates/engine/src/cmd/retouch_cmds/lama_cleaner.rs` (the command and resident-worker management)
- All numbers below were measured on 2026-11-16 (Windows / CUDA 13 + cuDNN 9.26 runtimes / RTX GPU / `big-lama.onnx` at 198.4 MiB)

---

## 1. Overview

The CUDA support consists of three parts:

1. **The execution-provider (EP) chain**: CUDA → DirectML → CPU is chosen when the model loads and degraded at run time when a provider stops working (`plugins/lama-cleaner/src/lib.rs:91`, `lib.rs:379`).
2. **CUDA/cuDNN DLL discovery and folder-wide preloading**: CUDA works without any PATH setup (`lib.rs:432`, `lib.rs:465`).
3. **The resident worker (`--serve`)**: the model stays loaded in one process, so a stroke no longer pays the 6.7–8.6 s model load (`plugins/lama-cleaner/src/main.rs:160`, `crates/engine/src/cmd/retouch_cmds/lama_cleaner.rs:232`).

Measured performance is in §9: 87–95 ms per stroke in the CUDA steady state.

## 2. Components and data flow

```text
UI (the LaMa Cleaner tool, its options bar)
  └─ engine command paint.lamaCleaner          crates/engine/src/cmd/retouch_cmds.rs:336
       └─ lama_cleaner.rs
            ├─ plan(): strokes → the inpainting window (§7.1)
            ├─ hole(): build the hole mask (§7.1)
            └─ jobs::run → execute_lama()
                 ├─ encode the window as RGB8 / Gray8 PNGs into temp files
                 ├─ resident::apply() → lama-cleaner --serve (§6)
                 │     └─ on failure, fall back to the one-shot run_helper_once()
                 ├─ decode the result PNG
                 └─ replace only the hole's pixels → one undo step
```

The helper has three launch modes (see USAGE in `plugins/lama-cleaner/src/main.rs`):

| Mode | Arguments | Purpose |
|---|---|---|
| OneShot | `-i <in.png> -m <mask.png> -o <out.png> [--model <m>]` | One image, one process. The engine's fallback path |
| Serve | `--serve [--model <m>]` | Load the model once, then answer one JSON request per stdin line |
| Self-test | (no arguments) | Run one inference on a synthetic image and save `%TEMP%\photocraft-lama-selftest\test_inpainted.png` |

The wasm build cannot start a helper process, so the command returns "LaMa Cleaner is only available in the desktop app" (`lama_cleaner.rs:531`).

## 3. Build configuration

`plugins/lama-cleaner/Cargo.toml:10-14`:

```toml
ort = { version = "2.0.0-rc.9", features = ["cuda", "directml", "preload-dylibs", "lax-feature-matching"] }
```

- The Windows x86_64 distribution table has exactly one CUDA prebuilt (`cuda13,tensorrt,nvrtx,directml`), and `cuda` alone never matches it. `lax-feature-matching` selects that single build.
- `preload-dylibs` enables `ep::cuda::preload_dylibs()` and `ort::util::preload_dylib()`, which is what lets the helper load CUDA/cuDNN from a folder of its choosing instead of depending on PATH (§5).
- Other dependencies: `ndarray 0.17` (tensors), `image 0.25` (png/jpeg only), `serde` / `serde_json` (the protocol), `thiserror`.

## 4. The execution-provider chain

### 4.1 Order and settings

`Provider::dispatch()` (`plugins/lama-cleaner/src/lib.rs:57`):

| EP | Settings | Why |
|---|---|---|
| CUDA | `ConvAlgorithmSearch::Heuristic` + `with_tf32(true)` + `error_on_failure()` | The heuristic search skips cuDNN's exhaustive benchmark (seconds on the first run). TF32 uses the tensor cores on Ampere and later. `error_on_failure()` keeps a missing runtime from silently downgrading |
| DirectML | `ep::DirectML::default().build().error_on_failure()` | Any DirectX 12 GPU, including non-NVIDIA ones |
| CPU | No EP (ONNX Runtime's default) | The last resort |

### 4.2 Selection at load time

`LamaModel::load()` (`lib.rs:91`). `prepare_cuda()` (§5) first locates the CUDA runtimes; when it fails, CUDA is skipped. The remaining providers are tried CUDA → DirectML → CPU through `open_session()` (`lib.rs:422`), and the first one that opens wins. Every provider that fails to open contributes its name and error to `notes`, and they are joined into the `NoProvider` error.

### 4.3 Degradation at run time (the lazy fallback)

A session can open and the model can still fail to run (DLLs found but of another version, for instance). When the first `session.run()` fails, `run_patch()` (`lib.rs:379`):

1. Records the current provider in `state.failed` and picks the next unfailed EP with `next_provider()` (`lib.rs:418`).
2. Builds a replacement session with `open_session()` and **re-runs the same patch** — the stroke completes on a slower path instead of failing outright.
3. Writes a note about the replacement to stderr and to `notes`.

When CPU itself fails there is nothing left to degrade to, and the error is returned as is.

`LamaModel` keeps the session in `session: Mutex<Option<Session>>`: `inpaint_512` (`lib.rs:122`) takes it out for a run and puts it back afterwards, so the replacement happens while no inference borrows the session. Mutex poisoning is recovered with `unwrap_or_else(PoisonError::into_inner)`.

## 5. CUDA / cuDNN DLL discovery and folder-wide preloading

### 5.1 The probe and the search order

`prepare_cuda()` (`lib.rs:432`) looks for `cudart64_13.dll` and `cudnn64_9.dll` (the first entries of `ort`'s `CUDA_DYLIBS` / `CUDNN_DYLIBS`, with fixed names as the fallback) through `find_runtime_dir()` (`lib.rs:498`). The search order:

1. The environment variables `CUDA_PATH` / `CUDNN_PATH` / `CUDA_HOME` / `CUDNN_HOME`; each root gets candidates appended by `push_toolkit_dirs()` (`lib.rs:478`): `bin\x64` (the CUDA 13+ and cuDNN 9+ layout) → `bin\<version>\x64` and `bin\<version>` (directory names in descending order — the versioned cuDNN 9 layouts such as `bin\13.0\x64`) → `bin` → the root itself.
2. The helper executable's own folder and its parent (the `plugins` folder).
3. `Program Files\NVIDIA GPU Computing Toolkit\CUDA\*` and `Program Files\NVIDIA\CUDNN\*` (every version directory, descending).
4. `PATH`.

The first directory that actually holds the DLL wins. When either runtime is missing, a note names the missing side and CUDA is given up.

### 5.2 Why every DLL in the folder is preloaded

For both discovered directories:

1. `ep::cuda::preload_dylibs(Some(&cuda), Some(&cudnn))` loads ORT's own known list.
2. `preload_folder()` (`lib.rs:465`) then loads **every `.dll` in both folders, in name order**, through `ort::util::preload_dylib()`. A library that cannot be loaded (another version, another architecture) is skipped silently.

Why: the CUDA EP loads its dependencies **by name** later, and Windows' loader only resolves a name that is already loaded in the process or is on PATH. cuDNN 9.26 ships extra provider DLLs such as `cudnn_engines_tensor_ir64_9.dll`, which are not in ONNX Runtime's own list. Without the folder-wide preload, **the first convolution fails with "Could not locate ..."** (measured). The fix is commit `b14e2651a`.

Effect: with CUDA 13 and cuDNN 9.26 installed, **CUDA works with nothing added to PATH**.

### 5.3 Failure notes (the exact wording)

- `the CUDA 13 runtime (cudart64_13.dll) was not found; using DirectML`
- `cuDNN 9 (cudnn64_9.dll) was not found; using DirectML`
- `CUDA libraries could not be preloaded (<error>); using DirectML`

These come back through `LamaModel::notes()` and reach stderr as `lama-cleaner: <note>`.

## 6. The resident worker (`--serve`)

### 6.1 Motivation and protocol

The model load costs 6.7–8.6 s. `--serve` (`main.rs:160`) loads it once and then answers one JSON request per stdin line with one JSON reply per stdout line:

```text
request: {"image":"<png>","mask":"<png>","output":"<png>"}   or {"warmup":true}
reply:   {"ok":true,"infer_ms":42.1,"provider":"CUDA"}
         {"ok":false,"error":"..."}
```

Diagnostics go to stderr only; stdout carries replies only. An unknown option is an error (`parse_args` in `main.rs`): when an older helper prints prose, the engine correctly judges it as one that cannot serve and falls back to a one-shot run.

### 6.2 Warm-up design — no background thread is spawned blindly

`warmup_once()` (`main.rs:146`) runs one 512×512 pass so cuDNN initializes its kernels once (measured ~206 ms). But `serve()` does **not** spawn a background warm-up on its own: when the engine starts `--serve` because of a stroke, the first stdin line is already waiting, and a background thread would grab the session lock while `run_one` decodes the PNGs, forcing the first stroke to run `inpaint_512` twice (a measured contention, fixed in `85c1b5a1c`). Only a caller that pre-launches the worker while idle may send `{"warmup":true}` explicitly.

### 6.3 Engine-side worker management

The `resident` module (`lama_cleaner.rs:232`):

- **The slot**: `OnceLock<Mutex<Option<Worker>>>` (`lama_cleaner.rs:337`). One worker for the whole process. When `matches()` reports an exe / model mismatch, the worker is dropped immediately and replaced.
- **Starting** (`Worker::start`): `lama-cleaner --serve --model <model>` with piped stdin/stdout and inherited stderr. A dedicated thread reads stdout into an mpsc channel, so a hung helper cannot block the request path: the requester waits on `recv_timeout` instead of `read_line`.
- **Timeout**: `REPLY_TIMEOUT = 120 s` (`lama_cleaner.rs:242`). The first request includes the model load. A timeout or an exited process is `Unavailable`.
- **The two error kinds** (`parse_reply`, `lama_cleaner.rs:356`):
  - `Unavailable`: failed to start, exited, timed out, or printed something that is not JSON (an older helper's prose). The worker is dropped.
  - `Failed`: a healthy worker's `{"ok":false,"error":...}`. The worker stays loaded and the message is reported directly.
- **The restart policy** (`apply`, `lama_cleaner.rs:377`): a worker that died between strokes is replaced once with a fresh worker before giving up; a failure right after starting is `Unavailable` immediately. Any `Unavailable` outcome empties the slot.
- **Fallback** (`execute_lama`, `lama_cleaner.rs:470`): when `apply()` returns `Unavailable`, `run_helper_once()` (`lama_cleaner.rs:450`, the one-shot `-i -m -o --model` process) runs instead. A `Failed` becomes the command's error as is.

## 7. The inference pipeline

### 7.1 Engine side

The command spec (`retouch_cmds.rs:336`): `paint.lamaCleaner`, with `strokes` (several strokes together), `closeLoops`=true, `sampleAllLayers`=false, `newLayer`=false, `exe`, `model` (to name the helper and model outright when they are not beside the app).

1. **Window planning** (`plan()`, `lama_cleaner.rs:82`): the strokes' bounds inflate by `margin = clamp(the longer side / 2, 128..512)`, widths and heights below 512 grow to at least 512, the overshoot beyond the canvas is absorbed by shifting, and the window is intersected with the canvas last. `MAX_MARGIN = 512` (`lama_cleaner.rs:19`). The context is what lets LaMa's FFC see both local and global structure.
2. **The hole mask** (`hole()`, `lama_cleaner.rs:126`): brush coverage is unioned, `closeLoops` closes a stroke around an object (`close_loops`), `dilate` grows the hole by 5 px (`GROW`, `lama_cleaner.rs:22` — it covers anti-aliased fringes), and a selection is AND-ed in when present.
3. **Running and compositing** (`run_lama_cleaner` → `execute_lama`, `lama_cleaner.rs:535` / `:470`): the target window is encoded as RGB8 and the hole as Gray8 into temp PNGs. After the helper returns, the result PNG is decoded and **only the hole's pixels** are written back (the original alpha is kept; `clamp_samples` guarantees the range). The whole thing is one `jobs::run` job and one undo step; with `newLayer`, `fill_to_new_layer()` (`lama_cleaner.rs:564`) puts the result on a new layer instead.

### 7.2 Helper side

**Arbitrary resolutions** (`inpaint()`, `lib.rs:161`, the IOPaint-compatible crop-and-pad strategy):

1. Find the mask's bounding box (an empty mask returns a clone of the input).
2. Crop a region of `desired = max(512, bbox + 2 × margin)` centred on the mask, where `context_margin = max(half the bbox's longer side, 128)`.
3. When a cropped side is below 512, reflection-pad to a square of `pad_side = max(512, the longer side)` and scale to 512 (Lanczos3; the mask uses Nearest to stay binary). A crop above 512 is scaled down to 512 → inference → scaled back with Lanczos3.
4. Write the inpainted region back over the crop.

**The 512 patch** (`inpaint_512()`, `lib.rs:122`):

- Input tensors (`build_inputs()`, `lib.rs:335`): `image` = `[1, 3, 512, 512]` in 0.0..1.0 (CHW, divided by 255), `mask` = `[1, 1, 512, 512]` binary (threshold 127).
- The output is planar RGB (plane = 262144). `big-lama.onnx` writes 0..255, but a variant writing 0..1 is detected by `max > 1.5` and scaled.
- Blending: a boundary pixel looks up the nearest unmasked pixel within its 7×7 neighbourhood and takes the feather factor from the squared distance (`feather()`, `lib.rs:322`): `d² ≤ 1 → 0.6`, `≤ 4 → 0.85`, otherwise `1.0`. `alpha = mask value / 255 × factor`, and the pixel blends linearly with the original. Not one pixel outside the mask changes.

## 8. File resolution

- **The model** (`resolve_model()`, `main.rs:99`): `--model` → the `PHOTOCRAFT_LAMA_MODEL` environment variable → `big-lama.onnx` beside the helper or in its parent folder. Measured size: **208,044,816 bytes (198.4 MiB)**.
- **Helper and model on the engine side** (`LamaTool`, `lama_cleaner.rs:184-230`): the parameters' `exe` / `model` win outright. Otherwise, in order: (1) Preferences › Plug-ins' additional folder (only when enabled), (2) the `plugins` folder beside the running executable, (3) the executable's own folder. The not-found error **lists every place searched**. The working directory is never searched (it depends on how the app was started). That the repository code carries no machine's absolute path is guaranteed by a test (`no_machine_absolute_path_is_written_in_this_file`).

## 9. Measured performance

Environment: Windows, CUDA 13 + cuDNN 9.26 runtimes, RTX GPU, `big-lama.onnx` 198.4 MiB (2026-11-16).

| Item | Value |
|---|---|
| Model load | 6.7–8.6 s |
| CUDA steady state (inference through `--serve`) | 87–95 ms / stroke |
| Whole `paint.lamaCleaner` command | ~120 ms |
| First stroke after a fresh worker (load included) | ~11.6 s |
| One warm-up pass (512×512) | ~206 ms |

- The DirectML / CPU paths are not measured (the chain exists and the degradation is test-verified).
- **The graph-optimized ONNX cache was dropped**: 6.90 s with vs 6.67 s without, at ~197 MB per provider — no effect, real cost.

## 10. Deployment on a machine

1. Put `lama-cleaner.exe` and `big-lama.onnx` in the `plugins` folder.
2. In Preferences › Plug-ins, enable "Additional Plug-ins Folder" and point it there. The settings keys are camelCase: `"plugIns": {"useAdditionalPluginsFolder": true, "additionalPluginsFolder": "<the plugins folder>"}`.
3. A development build may also need the same files in `target\release\plugins\` (a hard link avoids keeping two copies).
4. **A stale helper build without `--serve` makes every stroke fall back to a one-shot run.** Make sure the helper is current; this is designed behaviour, not an error.
5. With CUDA 13 and cuDNN 9 installed, no PATH setup is needed (§5.2).

## 11. Diagnostics and troubleshooting

- **Which EP actually ran**: the `provider` field of a `--serve` reply, the startup stderr line `lama-cleaner: <EP> provider, model loaded in ...`, or the one-shot `SUCCESS: ... (Provider: ...)` line. The engine itself does not surface the provider string in the UI today (`execute_lama` discards it) — showing it is a possible future improvement.
- **When CUDA is not used, check in this order**:
  1. Whether `cudart64_13.dll` and `cudnn64_9.dll` exist somewhere in §5.1's search order (`CUDA_PATH`, `CUDNN_PATH`, `Program Files\NVIDIA\CUDNN\<ver>\bin\<ver>\x64`, and so on).
  2. The helper's stderr notes (the three wordings in §5.3).
  3. The versioned cuDNN layout `bin\<cuda_ver>\x64` is already supported.
  4. PATH additions are unnecessary; harmless, but the design does not need them.
- **A hung helper**: judged `Unavailable` after 120 s and the request re-runs one-shot.
- **Checking the helper alone**: start it with no arguments to run the self-test (it inpaints a synthetic image into `%TEMP%\photocraft-lama-selftest\test_inpainted.png` and prints the centre pixel before and after).

## 12. Tests

| Location | What they pin |
|---|---|
| `plugins/lama-cleaner/src/lib.rs` (5) | the feather ramp, provider names, the `next_provider` chain, GPU providers insisting on registration, the CUDA13/cuDNN9 `bin\x64` and `bin\<ver>\x64` search layouts |
| `plugins/lama-cleaner/src/main.rs` (6) | argument parsing (self-test / one-shot / serve / unknown option / missing path / partial set) |
| `crates/engine/src/cmd/retouch_cmds/lama_cleaner.rs` | folder order, the prefs folder searched only when enabled, params-given paths, the enumeration in not-found errors, the no-machine-paths guarantee, the resident request line / reply parsing / prose rejection / failed start |

Measured: the plugin tests pass (5 lib + 6 bin), all of them (2026-11-16).

## 13. Limits and recorded design decisions

- The model's side is fixed at 512 (`SIDE`, near the top of `lib.rs`). Arbitrary resolutions go through §7.2's crop-and-pad strategy.
- wasm is not supported; the command returns an explicit error.
- One worker slot for the whole process; it is not per document.
- The engine does not send a warm-up automatically (the contention in §6.2); only a caller that pre-launches may send `{"warmup":true}`. The first stroke therefore pays the model load.
- The graph-optimization cache is gone, with the measurement in §9.
- `REPLY_TIMEOUT` is 120 s; the first request's model load is inside it.

## 14. Change history

| Commit | What it did |
|---|---|
| `049063b90` | LaMa Cleaner runs on the GPU and keeps its model loaded — the EP chain (CUDA→DirectML→CPU) and the `--serve` resident worker |
| `85c1b5a1c` | Fix LaMa cleaner warm-up contention, bin\x64 toolkit paths and resident error handling — the warm-up contention, the `bin\<ver>\x64` layouts, and the `Unavailable` / `Failed` split |
| `b14e2651a` | Make the LaMa CUDA path work without the toolkit on PATH — the folder-wide DLL preload and the run-time lazy provider degradation |
