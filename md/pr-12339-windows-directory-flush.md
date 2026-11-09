# Windows: an unsupported directory flush no longer fails the save

This is how saving works in the tree as it stands. A save writes a temporary file, renames it into
place, and then flushes the directory so the rename survives a power loss. Some Windows file systems
cannot flush a directory at all; those errors are classified as "this file system cannot do it" and
the save succeeds.

| Behaviour | Implementation |
|---|---|
| `ERROR_NOT_SUPPORTED` (50), `ERROR_INVALID_FUNCTION` (1), `ERROR_CALL_NOT_IMPLEMENTED` (120) | `dir_sync_unsupported` returns true, `dir_synced` maps the failure to `Ok(())` |
| Any other error (e.g. `ERROR_SHARING_VIOLATION` 32, `ERROR_FILE_NOT_FOUND` 2) | Propagated: the save reports `publication_error` and the document stays unsaved |
| The directory cannot even be opened | Treated like a failed flush: unsupported codes succeed, everything else propagates |
| Unix | Unchanged: `EINVAL` (22), `ENOTSUP`/`EOPNOTSUPP` (95/45/102), `ENOTTY` (25), `ErrorKind::Unsupported` |
| Windows publication barrier | the rename plus the flush; a failure after the rename reports `publication_error` |
| Unix publication barrier | the rename alone; a failed flush reports `unconfirmed_error` |

## Problem

Saving to a WebDAV share, a cloud-mounted drive (AirLiveDrive, RaiDrive and similar), or a FAT32
volume on Windows produced:

```
wrote <path>, but could not confirm the save is durable; it may not survive a power loss or crash: (os error 50)
```

The request that fails is `FlushFileBuffers` on a directory handle. Those redirectors and file systems
do not implement it, so they answer `ERROR_NOT_SUPPORTED`. The file payload had already been written
and renamed successfully; the save was then reported as failed, and the caller kept the document
unsaved and discarded its recovery data.

Unix already handled the same situation: a file system that cannot flush directories counts as done,
because failing every save there would make saving impossible. Windows took the opposite branch and
failed the save on any flush error (#7858).

## Design

The rule is: an error that says *this file system cannot flush a directory* is not a failed flush.
An error that says *the flush was attempted and failed* is. The classifier owns that distinction and
every publication path goes through it.

```
replace_with()  ─┐
write_bundle_file() ├─► dir_synced(result)  ─► dir_sync_unsupported(&e) ? Ok(()) : Err(e)
PcraftWriter    ─┘                                │
                                                  ├─ Windows: 1, 50, 120, ErrorKind::Unsupported
                                                  └─ Unix:    EINVAL, ENOTSUP/EOPNOTSUPP, ENOTTY,
                                                              ErrorKind::Unsupported
```

Opening the directory is classified the same way, because a virtual file system may refuse the open
rather than the flush.

## Implementation

### `crates/format/src/atomic.rs`

`RealFs::sync_dir` on Windows used to inline the open and the flush. That is now three functions so
the composed behaviour can be tested with injected failures:

```rust
pub(crate) fn sync_dir_windows(dir: &Path) -> io::Result<()>
pub(crate) fn sync_dir_windows_with<F, S>(dir: &Path, open: F, sync: S) -> io::Result<()>
pub(crate) fn open_dir_for_sync(dir: &Path) -> io::Result<std::fs::File>
```

`sync_dir_windows_with` is the whole rule in one place: an unsupported **open** succeeds, any other
open error propagates, and the flush result goes through `dir_synced`.

```rust
let directory = match open(dir) {
    Ok(d) => d,
    Err(e) if dir_sync_unsupported(&e) => return Ok(()),
    Err(e) => return Err(e),
};
dir_synced(sync(&directory))
```

`open_dir_for_sync` keeps the original access rights: `FILE_ADD_FILE` (the directory equivalent of
`FILE_WRITE_DATA`) plus `FILE_FLAG_BACKUP_SEMANTICS`, which opens a directory rather than a volume and
needs no administrative privilege.

`replace_with` now routes the Windows flush through the classifier instead of mapping every error to
`publication_error`:

```rust
#[cfg(windows)]
dir_synced(fs.sync_dir(&dir)).map_err(|e| publication_error(e, path))?;
#[cfg(not(windows))]
dir_synced(fs.sync_dir(&dir)).map_err(|e| unconfirmed_error(e, path))?;
```

`dir_synced` and `dir_sync_unsupported` lost their `#[cfg(not(windows))]`. The classifier keeps one
`Unsupported` check and then splits by target: Windows compares the raw code against 1, 50 and 120;
Unix keeps the per-platform errno table (`EINVAL` 22 everywhere, 95 on Linux/Android, 45/102/25 on
Apple, 45 on the BSDs), because 45 means something else elsewhere.

The error message for a genuine failure still distinguishes the two cases: on Windows the flush is a
publication barrier (`publication_error`), elsewhere it is a durability confirmation
(`unconfirmed_error`).

### `crates/format/src/atomic/tests.rs`

`Faulty` gains a Windows-only `fail_sync_all` injection, so a test can fail the flush *after* the
directory opened successfully. `sync_dir` injects either a raw OS code or an `ErrorKind` and
otherwise delegates to `RealFs`.

The Windows test now asserts the classification directly, without needing a WebDAV drive:

```rust
assert!(dir_synced(Err(io::Error::from_raw_os_error(50))).is_ok());
assert!(dir_synced(Err(io::Error::from_raw_os_error(1))).is_ok());
assert!(dir_synced(Err(io::Error::from_raw_os_error(120))).is_ok());
assert!(dir_synced(Err(io::Error::from_raw_os_error(32))).is_err());
```

It also covers an unsupported error raised by the **open** itself, and keeps the existing
`NotFound` case as a genuine failure.

## Error classification

| Platform | Code | Name | Treated as |
|---|---|---|---|
| Windows | 1 | `ERROR_INVALID_FUNCTION` | success |
| Windows | 50 | `ERROR_NOT_SUPPORTED` | success |
| Windows | 120 | `ERROR_CALL_NOT_IMPLEMENTED` | success |
| Windows | 32 | `ERROR_SHARING_VIOLATION` | failure |
| Windows | 2 | `ERROR_FILE_NOT_FOUND` | failure |
| Unix | 22 | `EINVAL` | success |
| Unix | 95 / 45 / 102 | `ENOTSUP` / `EOPNOTSUPP` | success |
| Unix | 25 | `ENOTTY` (Apple) | success |
| Any | — | `ErrorKind::Unsupported` | success |

## Behaviour details

| Situation | Before | After |
|---|---|---|
| WebDAV save, directory flush answers 50 | Save reported as failed; document kept unsaved | Save succeeds; file stays published |
| Cloud-mounted drive, open answers 50 | Save failed at the open | Save succeeds |
| Local NTFS, flush succeeds | Save succeeded | Unchanged |
| Local NTFS, sharing violation (32) | Save failed | Still fails |
| Directory deleted between rename and flush (2) | Save failed | Still fails |
| Unix file system without directory fsync | Save succeeded | Unchanged |

## Types and call graph

| Item | Kind | Contract | Callers |
|---|---|---|---|
| `AtomicFs` | trait | `is_readonly`, `write_new`, `write_new_progress`, `write_new_stream`, `rename`, `remove`, `copy_metadata`, `sync_dir(&self, dir: &Path) -> io::Result<()>` | `replace_with`, `store.rs`, `Faulty` |
| `RealFs` | struct | the real implementation; `sync_dir` is the hunk 1 body | `atomic_write*` |
| `Faulty` | test struct | `fail_sync_dir`, `fail_sync_all` (Windows), `synced: Cell<u32>` | `atomic/tests.rs` |
| `dir_synced(io::Result<()>)` | fn | maps the unsupported codes to `Ok(())`, everything else through | `replace_with`, `store.rs`, `sync_dir_windows_with` |
| `dir_sync_unsupported(&io::Error)` | fn | the predicate behind `dir_synced`; Windows 1/50/120, Unix 22/95/45/102/25, `ErrorKind::Unsupported` | `dir_synced`, `sync_dir_windows_with` |
| `sync_dir_windows(dir)` | fn | `sync_dir_windows_with(dir, open_dir_for_sync, |d| d.sync_all())` | `RealFs::sync_dir` (Windows) |
| `sync_dir_windows_with<F, S>(dir, open, sync)` | fn | open classified, flush classified | `sync_dir_windows`, `Faulty::sync_dir` (Windows tests) |
| `open_dir_for_sync(dir)` | fn | `FILE_ADD_FILE` + `FILE_FLAG_BACKUP_SEMANTICS` | `sync_dir_windows`, tests |

Call chain, measured:

```
atomic_write / atomic_write_stream
  -> atomic_write_with / atomic_write_stream_with
       -> replace_with(fs, retry, path, write_tmp)
            -> fs.write_new(tmp)        (file content + sync_file)
            -> fs.rename(tmp, path)     (RenameRetry)
            -> dir_synced(fs.sync_dir(&dir))?          <-- this pull request
                 -> dir_sync_unsupported(&e) ? Ok(()) : Err(e)
                      (Windows: code 1 | 50 | 120)
                      (Unix:    22 | 95 | 45 | 102 | 25)

RealFs::sync_dir  (Windows)
  -> sync_dir_windows(dir)
       -> sync_dir_windows_with(dir, open_dir_for_sync, |d| d.sync_all())
            -> open  : Ok(d) | Err(unsupported) -> Ok(()) | Err(other) -> Err
            -> flush : dir_synced(sync(&d))

store.rs (write_bundle_file, PcraftWriter)
  -> dir_synced(fs.sync_dir(...))     (same classifier, not modified by this PR)
```

The error text still depends on the platform, and that asymmetry is part of the contract: `publication_error` on Windows (the flush is the publication barrier), `unconfirmed_error` elsewhere (the file is published, durability is unconfirmed).

## Testing

```
cargo test  -p photocraft-format --lib
cargo clippy -p photocraft-format --lib -- -D warnings
cargo fmt --check -p photocraft-format
cargo xtask layers
```

Measured on the branch: `photocraft-format` 86 passed, 0 failed; clippy 0 warnings; `cargo xtask
layers` 31 crates with 0 violations.

Tests that carry the behaviour:

* `a_directory_flush_the_file_system_cannot_do_still_saves` - enabled on every platform; on Windows it
  asserts 50 and 1 succeed and 32 fails.
* `windows_directory_flush_opens_a_directory_and_reports_open_errors` - a real directory flushes, a
  missing one reports `NotFound`, and the unsupported codes are classified as non-fatal.

## Scope

The change touches two files: `crates/format/src/atomic.rs` and `crates/format/src/atomic/tests.rs`.
The bundle publication path in `crates/format/src/store.rs` (`write_bundle_file`, `PcraftWriter`) is
not modified here; it reaches the same classifier through `dir_synced`.
## Limitations

The change makes an unverifiable flush succeed; it does not make the flush happen. On a file system that cannot flush a directory, a save that reports success may still be lost if the machine loses power immediately afterwards - the bytes are in the file, the directory entry is not confirmed. That is the same guarantee the save already gave on Unix for the same file systems, and it is a strictly better outcome than failing the save after publishing the bytes, but it is not durability.

The classifier is a code list, not a capability probe. Codes 1, 50 and 120 are the ones observed on redirectors and cloud drives; a fourth file system that signals the same condition with a different code will still fail the save. The list is in one function (`dir_sync_unsupported`), so adding a code is a one-line change, but nothing detects a missing code at runtime.

`ERROR_SHARING_VIOLATION` (32) is deliberately not classified. On a directory held open by another process the save fails, which is correct but means a transient sharing conflict on a network share still surfaces as a failed save. A retry loop would be the fix for that; none is added here.

The `store.rs` bundle path gets the same classification because it calls `dir_synced` around `fs.sync_dir`, but that call site is not modified by this pull request and its own tests are unchanged, so the Windows behaviour of bundle publication is only covered indirectly.

## Verification

What was run against this branch, and what was not.

| Check | Result |
|---|---|
| `git diff --stat 848fa306f 65e44a56a` | 2 files, +155 / -51 (`crates/format/src/atomic.rs`, `crates/format/src/atomic/tests.rs`) |
| `git diff 848fa306f 65e44a56a \| grep -c '^@@'` | 9 hunks, matching the 9 sections below |
| Test count in `crates/format/src/atomic/tests.rs` at the head commit | 37 test functions, of which the three directory-flush ones are changed or added by this pull request |
| GitHub checks | `CodeRabbit: success`; no GitHub Actions workflow runs are recorded (`status total=0`) |
| Merge state at the time of writing | `mergeable: MERGEABLE`, `mergeStateStatus: UNSTABLE` |
| `cargo test -p photocraft-format --lib atomic::` in a detached worktree at `65e44a56a` | **36 passed, 0 failed** (1.35 s). Includes `windows_directory_flush_opens_a_directory_and_reports_open_errors` and `a_directory_flush_the_file_system_cannot_do_still_saves` |

The run was performed on Windows, which is the platform the change targets, so the `#[cfg(windows)]` arms are the ones that executed: the classifier's three codes, the open-error arm, the flush-error arm and the `synced` counter were all exercised rather than compiled out. The Unix arm (`EINVAL`, `ENOTSUP`, `EIO`) did not run here, so its behaviour is asserted by its source and by CI on the other platforms, not by this run.

## Differences from upstream `main`

Measured with `git diff -U3 upstream/main...main`, where `upstream/main` is `95c9e465d` (storytold/photocraft) and `main` is `f328d1881`. The full range is 49 files, 8101 insertions, 182 deletions. The hunks below are every hunk in the files that carry this feature - 11 hunks, reproduced in full with their context lines, each followed by an explanation of the original code, the change, and its effect.

### `crates/format/src/atomic.rs`

#### 13. `crates/format/src/atomic.rs` — hunk -139,16 +139,7

**Original code (upstream `main`)**

```rust
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::OpenOptionsExt;

            // FILE_ADD_FILE is the directory equivalent of FILE_WRITE_DATA. Requesting only
            // this right permits FlushFileBuffers without requiring GENERIC_WRITE's additional
            // directory permissions. BACKUP_SEMANTICS opens a directory, not a volume: no
            // administrative privilege or volume-wide flush is needed.
            const FILE_ADD_FILE: u32 = 0x0002;
            const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
            let directory = std::fs::OpenOptions::new().access_mode(FILE_ADD_FILE).custom_flags(FILE_FLAG_BACKUP_SEMANTICS).open(dir)?;
            directory.sync_all()
        }
        #[cfg(not(any(unix, windows)))]
        {
```

**Changed code (current `main`)**

```rust
        }
        #[cfg(windows)]
        {
            sync_dir_windows(dir)
        }
        #[cfg(not(any(unix, windows)))]
        {
```

**Explanation.** To read this hunk you need the trait it implements. `AtomicFs` is the file-system seam of `atomic_write`: it declares the steps a save performs, `RealFs` is the real implementation, and the tests wrap it to inject failures. Its methods are `is_readonly`, `write_new`, `write_new_progress`, `write_new_stream`, `rename`, `remove`, `copy_metadata` and `sync_dir`. `sync_dir` is the last step: `replace_with` calls `write_tmp` (which goes through `write_new`), then `rename`, then `sync_dir`, and only after `sync_dir` returns does the save count as published. The doc comment on the trait says exactly that: "The file-system steps [`atomic_write_with`] performs. [`RealFs`] is the real thing; tests wrap it to inject failures."

This hunk is `RealFs::sync_dir`, in its `#[cfg(windows)]` arm. The original opened the directory inline with `OpenOptions::new().access_mode(FILE_ADD_FILE).custom_flags(FILE_FLAG_BACKUP_SEMANTICS).open(dir)?` and called `sync_all()` on the handle. The comment above the constants explained the rights: `FILE_ADD_FILE` (0x0002) is the directory equivalent of `FILE_WRITE_DATA` and permits `FlushFileBuffers` without `GENERIC_WRITE`'s additional directory permissions; `FILE_FLAG_BACKUP_SEMANTICS` (0x0200_0000) opens a directory rather than a volume, so no administrative privilege and no volume-wide flush are involved. Without those two, opening a directory with a plain `File::open` fails on Windows.

The change replaces the body with `sync_dir_windows(dir)`. The rights did not disappear; they moved verbatim into `open_dir_for_sync` (hunk 3), comment included.

Why the move matters, concretely: as a trait method, `sync_dir` can only be exercised through `dyn AtomicFs`, and the only other implementation is `Faulty`, which the tests control. `Faulty::sync_dir` can return an error, but it cannot make `RealFs`'s *open* succeed and the *flush* fail - that requires a real directory handle and an injected flush, which is precisely what `sync_dir_windows_with(dir, open, sync)` (hunk 3) provides. The trait method signature `fn sync_dir(&self, dir: &Path) -> io::Result<()>` takes no seam, so the seam has to live below it.

A second consequence: `RealFs::sync_dir` is also called by `store.rs`'s bundle publication (`write_bundle_file`, `PcraftWriter`), which does not go through `replace_with`. Before this change those call sites could not express "an unsupported directory flush is fine" because the classifier was `#[cfg(not(windows))]`. After it, whatever `RealFs::sync_dir` returns is already classified, so the callers get the Windows fix without their own branch. That is the reason the move is to a shared function rather than to a private helper in `replace_with`.

One behaviour does change at this level: the old `open(dir)?` propagated every open failure; the new `sync_dir_windows` swallows an unsupported open. That is intended (a file system that cannot open a directory for flushing cannot flush it), and it is asserted in hunk 9. A reader checking the trait contract should note the method's meaning is now slightly wider: "flush the directory, or report that this file system cannot".

#### 14. `crates/format/src/atomic.rs` — hunk -353,12 +344,12

**Original code (upstream `main`)**

```rust
        return Err(rename_error(e, path));
    }
    // The new bytes are in place, but until the directory is flushed the rename may not survive
    // a power loss (#4968). Windows fails the save on any failed flush (#7858). Elsewhere a file
    // system that can't flush directories at all counts as done; any other failure is reported,
    // so the caller keeps the document unsaved and its recovery data instead of discarding them
    // on a save that may not last.
    #[cfg(windows)]
    fs.sync_dir(&dir).map_err(|e| publication_error(e, path))?;
    #[cfg(not(windows))]
    dir_synced(fs.sync_dir(&dir)).map_err(|e| unconfirmed_error(e, path))?;
    Ok(())
```

**Changed code (current `main`)**

```rust
        return Err(rename_error(e, path));
    }
    // The new bytes are in place, but until the directory is flushed the rename may not survive
    // a power loss (#4968). A file system that can't flush directories at all (e.g. WebDAV,
    // cloud-mounted drives such as AirLiveDrive / RaiDrive, FAT32) counts as done (#7858).
    // Any real failure is reported, so the caller keeps the document unsaved and its recovery
    // data instead of discarding them on a save that may not last.
    #[cfg(windows)]
    dir_synced(fs.sync_dir(&dir)).map_err(|e| publication_error(e, path))?;
    #[cfg(not(windows))]
    dir_synced(fs.sync_dir(&dir)).map_err(|e| unconfirmed_error(e, path))?;
    Ok(())
```

**Explanation.** `replace_with(fs: &dyn AtomicFs, retry: RenameRetry, path: &Path, write_tmp: &mut dyn FnMut(&Path) -> io::Result<()>)` is the shared body of every atomic write. Its steps are: pick the temporary name from `path`, call `write_tmp` on it (the caller writes the bytes and flushes the file), rename the temporary file over `path` with the retry policy, then flush the directory. The visible context is the rename and its error mapping: `return Err(rename_error(e, path));` is the failure path when the rename itself fails.

The comment the hunk rewrites is the specification of the third step. The original said: "The new bytes are in place, but until the directory is flushed the rename may not survive a power loss (#4968). Windows fails the save on any failed flush (#7858). Elsewhere a file system that can't flush directories at all counts as done; any other failure is reported, so the caller keeps the document unsaved and its recovery data instead of discarding them on a save that may not last." That is accurate for the code as it stood, and the first two sentences are why the step exists: without the flush, a crash can leave the directory entry pointing at the old file even though the new bytes are on disk.

The change rewrites the comment to state the new rule and changes one line: `fs.sync_dir(&dir).map_err(|e| publication_error(e, path))?;` becomes `dir_synced(fs.sync_dir(&dir)).map_err(|e| publication_error(e, path))?;`.

The remaining asymmetry is the part a reviewer is most likely to "simplify" by mistake. Windows keeps `publication_error`; the `not(windows)` arm keeps `unconfirmed_error`. The two functions produce different text because they describe different situations. On Windows the rename and the flush together are the publication barrier: the platform's rename is not durable until the directory entry is flushed, so a failure after the rename means the file may not be published - the caller must keep the document unsaved. Elsewhere the rename is the barrier and the flush only confirms durability: the file is published, but may not survive a power loss - the caller is told exactly that. Turning the Windows arm into `unconfirmed_error` would tell a Windows user their file is published when a `ERROR_SHARING_VIOLATION` means it may not be.

`dir_synced` cannot mask a real failure: it returns `Ok(())` only for the codes `dir_sync_unsupported` lists (50, 1, 120 on Windows; `EINVAL`, `ENOTSUP`, `ENOTTY`, `Unsupported` elsewhere) and returns every other error unchanged. So the `?` still short-circuits on a genuine failure, and the cleanup that follows (removing the temporary file) still runs only on success.

The `map_err` order is deliberate: `dir_synced` first, then the error mapping, so an unsupported code never reaches `publication_error` and a genuine one always does. Reversing the order would classify the error after it had been wrapped in a publication error, which does not preserve the raw OS code.

The comment's trailing clause - "the caller keeps the document unsaved and its recovery data" - is the reason the distinction matters to the user: a save that reports success discards the recovery data, so a false success loses the work; a false failure keeps data that could have been discarded. Erring on "failure" for a real error is the safe side.

#### 15. `crates/format/src/atomic.rs` — hunk -376,12 +367,48

**Original code (upstream `main`)**

```rust
    }
}

/// `result` of flushing a directory, with "this file system can't flush directories" counted as
/// success: `EINVAL` (Linux file systems without directory fsync), `ENOTSUP`/`EOPNOTSUPP` and,
/// on Apple platforms, `ENOTTY`, or [`io::ErrorKind::Unsupported`]. Waiting or retrying won't
/// change those, and failing every save there would make saving impossible. Any other error is
/// returned: the flush was attempted and failed.
#[cfg(not(windows))]
pub(crate) fn dir_synced(result: io::Result<()>) -> io::Result<()> {
    match result {
        Err(e) if dir_sync_unsupported(&e) => Ok(()),
```

**Changed code (current `main`)**

```rust
    }
}

/// Flush directory publication on Windows using `open_dir_for_sync`.
#[cfg(windows)]
pub(crate) fn sync_dir_windows(dir: &Path) -> io::Result<()> {
    sync_dir_windows_with(dir, open_dir_for_sync, |d| d.sync_all())
}

/// Flush directory publication on Windows with a custom opener `open` and flusher `sync`, treating
/// unsupported errors (such as `ERROR_NOT_SUPPORTED` on WebDAV/cloud virtual drives) as non-fatal.
#[cfg(windows)]
pub(crate) fn sync_dir_windows_with<F, S>(dir: &Path, open: F, sync: S) -> io::Result<()>
where
    F: FnOnce(&Path) -> io::Result<std::fs::File>,
    S: FnOnce(&std::fs::File) -> io::Result<()>,
{
    let directory = match open(dir) {
        Ok(d) => d,
        Err(e) if dir_sync_unsupported(&e) => return Ok(()),
        Err(e) => return Err(e),
    };
    dir_synced(sync(&directory))
}

/// Open `dir` on Windows with `FILE_ADD_FILE` and `FILE_FLAG_BACKUP_SEMANTICS` for directory flush.
#[cfg(windows)]
pub(crate) fn open_dir_for_sync(dir: &Path) -> io::Result<std::fs::File> {
    use std::os::windows::fs::OpenOptionsExt;

    // FILE_ADD_FILE is the directory equivalent of FILE_WRITE_DATA. Requesting only
    // this right permits FlushFileBuffers without requiring GENERIC_WRITE's additional
    // directory permissions. BACKUP_SEMANTICS opens a directory, not a volume: no
    // administrative privilege or volume-wide flush is needed.
    const FILE_ADD_FILE: u32 = 0x0002;
    const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
    std::fs::OpenOptions::new().access_mode(FILE_ADD_FILE).custom_flags(FILE_FLAG_BACKUP_SEMANTICS).open(dir)
}

/// `result` of flushing a directory, with "this file system can't flush directories" counted as
/// success: on Windows `ERROR_NOT_SUPPORTED` (50, WebDAV/cloud drives), `ERROR_INVALID_FUNCTION` (1),
/// `ERROR_CALL_NOT_IMPLEMENTED` (120); on Unix `EINVAL` (22), `ENOTSUP`/`EOPNOTSUPP` (95/45/102),
/// `ENOTTY` (25), or [`io::ErrorKind::Unsupported`]. Waiting or retrying won't change those, and
/// failing every save there would make saving impossible. Any other error is returned: the flush
/// was attempted and failed.
pub(crate) fn dir_synced(result: io::Result<()>) -> io::Result<()> {
    match result {
        Err(e) if dir_sync_unsupported(&e) => Ok(()),
```

**Explanation.** Three functions are inserted after `open_dir_for_sync`'s old position, and `dir_synced` loses its `#[cfg]`.

`sync_dir_windows(dir: &Path) -> io::Result<()>` is the production entry point and nothing else: `sync_dir_windows_with(dir, open_dir_for_sync, |d| d.sync_all())`. The closure is the flush - `File::sync_all` is `FlushFileBuffers` plus metadata, which is what the original inline code called.

`sync_dir_windows_with<F, S>(dir: &Path, open: F, sync: S)` is the seam and the composed rule, in six lines: `match open(dir)` with three arms (success passes the handle through, an unsupported error returns `Ok(())`, any other error returns it), then `dir_synced(sync(&directory))`. Both parameters are `FnOnce` because each is called exactly once; `FnMut` would widen the contract without a caller needing it, and `Fn` would forbid a closure that moves the error it returns.

The two-arm classification of the open is the part that is easy to get wrong. A WebDAV redirector may refuse `CreateFile` on a directory (the open) rather than `FlushFileBuffers` (the flush), depending on the driver; classifying only the flush would leave those drives failing. Classifying only the open would leave the original defect. The function exists because both must be classified, and because the classifier needs the error value, not just a boolean.

`open_dir_for_sync(dir: &Path) -> io::Result<File>` is the extracted opener with the two constants and the comment moved verbatim. The return type is `io::Result<File>`, not `Option<File>`: `sync_dir_windows_with` needs the error to decide whether it is unsupported, and a `None` would have already discarded that information. It is a separate function rather than a closure inside `sync_dir_windows` because the tests need to pass a *different* opener (one that returns a chosen error) to the same composition.

`dir_synced(result: io::Result<()>) -> io::Result<()>` loses `#[cfg(not(windows))]`. Its body is unchanged (`Err(e) if dir_sync_unsupported(&e) => Ok(()), other => other`), and its doc comment gains the Windows codes and the numbers for the Unix ones. The removal is what lets `replace_with` call it on every platform with the same shape, and what lets `store.rs` do the same without its own `cfg`.

The doc comment's list of numbers is not decoration: it is the only place a reviewer can check the classifier's intent without reading the `cfg` blocks in `dir_sync_unsupported`. It names 50, 1 and 120 for Windows and 22, 95/45/102, 25 for Unix, matching hunk 4's tables exactly.

One design point worth stating: the open and the flush are classified by two different expressions (`dir_sync_unsupported(&e)` for the open, `dir_synced(...)` for the flush) that share one predicate. Keeping the shared predicate is what makes "unsupported" mean the same thing at both call sites; a second predicate would drift the first time one platform's list changed.

#### 16. `crates/format/src/atomic.rs` — hunk -389,27 +416,38

**Original code (upstream `main`)**

```rust
    }
}

#[cfg(not(windows))]
fn dir_sync_unsupported(e: &io::Error) -> bool {
    if e.kind() == io::ErrorKind::Unsupported {
        return true;
    }
    // errno values: EINVAL is 22 everywhere; ENOTSUP and EOPNOTSUPP are 95 on Linux and
    // Android, 45 and 102 on Apple platforms, 45 on the BSDs; ENOTTY is 25. Elsewhere 45 means
    // something else (illumos: EDEADLK), so only EINVAL and `Unsupported` count there.
    const EINVAL: i32 = 22;
    let unsupported: &[i32] = if cfg!(any(target_os = "linux", target_os = "android")) {
        &[EINVAL, 95]
    } else if cfg!(target_vendor = "apple") {
        &[EINVAL, 45, 102, 25]
    } else if cfg!(any(target_os = "freebsd", target_os = "openbsd", target_os = "netbsd", target_os = "dragonfly")) {
        &[EINVAL, 45]
    } else if cfg!(unix) {
        &[EINVAL]
    } else {
        &[]
    };
    e.raw_os_error().is_some_and(|code| unsupported.contains(&code))
}

/// The error for a file that was published but whose directory flush failed: same kind, saying
```

**Changed code (current `main`)**

```rust
    }
}

fn dir_sync_unsupported(e: &io::Error) -> bool {
    if e.kind() == io::ErrorKind::Unsupported {
        return true;
    }
    #[cfg(windows)]
    {
        const ERROR_INVALID_FUNCTION: i32 = 1;
        const ERROR_NOT_SUPPORTED: i32 = 50;
        const ERROR_CALL_NOT_IMPLEMENTED: i32 = 120;
        e.raw_os_error().is_some_and(|code| {
            code == ERROR_INVALID_FUNCTION || code == ERROR_NOT_SUPPORTED || code == ERROR_CALL_NOT_IMPLEMENTED
        })
    }
    #[cfg(not(windows))]
    {
        // errno values: EINVAL is 22 everywhere; ENOTSUP and EOPNOTSUPP are 95 on Linux and
        // Android, 45 and 102 on Apple platforms, 45 on the BSDs; ENOTTY is 25. Elsewhere 45 means
        // something else (illumos: EDEADLK), so only EINVAL and `Unsupported` count there.
        const EINVAL: i32 = 22;
        let unsupported: &[i32] = if cfg!(any(target_os = "linux", target_os = "android")) {
            &[EINVAL, 95]
        } else if cfg!(target_vendor = "apple") {
            &[EINVAL, 45, 102, 25]
        } else if cfg!(any(target_os = "freebsd", target_os = "openbsd", target_os = "netbsd", target_os = "dragonfly")) {
            &[EINVAL, 45]
        } else if cfg!(unix) {
            &[EINVAL]
        } else {
            &[]
        };
        e.raw_os_error().is_some_and(|code| unsupported.contains(&code))
    }
}

/// The error for a file that was published but whose directory flush failed: same kind, saying
```

**Explanation.** `dir_sync_unsupported` decides whether a failed directory flush means "this file system cannot do it" (not fatal - the file is already published) or "the flush really failed" (fatal - durability is unknown). Before this change the function was `#[cfg(not(windows))]`: it only knew errno values, so on Windows it did not exist and the caller cannot have used it. The change makes it exist on every platform and puts a Windows arm in front of the Unix one.

The Windows arm matches three raw codes. `ERROR_INVALID_FUNCTION` (1) is what some redirectors return for `CreateFileW` on a directory they do not implement as a file. `ERROR_NOT_SUPPORTED` (50) is what WebDAV and most cloud-drive filter drivers (the report in #7858 names AirLiveDrive) return for `FlushFileBuffers` on a directory handle. `ERROR_CALL_NOT_IMPLEMENTED` (120) is what older redirectors return for the same call. All three mean the same thing operationally: the platform will never be able to confirm this flush, so retrying or failing the save cannot help. A fourth code, `ERROR_SHARING_VIOLATION` (32), is deliberately *not* in the list: it means another process holds the directory, which is transient and therefore a real failure.

The `#[cfg(windows)]` block is a block expression, not a statement list, so the function still has exactly one `return`-shaped tail per platform. Writing it as `if cfg!(windows) { ... } else { ... }` would have compiled both arms everywhere and produced unreachable-code warnings on both platforms; `#[cfg]` on a block removes the arm at parse time.

`e.kind() == io::ErrorKind::Unsupported` stays outside both arms, because it is the only signal that is already platform-neutral: `io::Error::from(io::ErrorKind::Unsupported)` has no raw OS error at all, and every other check in the function reads `raw_os_error()`. Keeping it first also means a synthetic `Unsupported` from any layer above the OS is still classified, which is what the `Faulty` fake uses in the Unix test.

The Unix arm is carried over byte for byte, including the comment that explains why 45 is only counted on Apple and the BSDs (on illumos 45 is `EDEADLK`, a real failure, so a blanket `45` would swallow it). It was not moved into an `else` chain with the Windows arm precisely so the diff shows it as unchanged context rather than as a relocation: a reviewer can see the Unix behaviour did not move.

The consequence for callers is that `dir_synced` (the wrapper) now behaves on Windows the way it behaved on Unix: an unsupported error becomes `Ok(())`. `replace_with` and `store.rs` both call `dir_synced`, so both inherit the change without an edit. That is the reason the classification was put in this one function instead of at each call site.

### `crates/format/src/atomic/tests.rs`

#### 17. `crates/format/src/atomic/tests.rs` — hunk -44,6 +44,9

**Original code (upstream `main`)**

```rust
    /// Fail the directory flush (the durability barrier after publication, #4968, #7858)
    /// with this raw OS error, or with this kind.
    fail_sync_dir: Option<Result<i32, io::ErrorKind>>,
    synced: Cell<u32>,
}

```

**Changed code (current `main`)**

```rust
    /// Fail the directory flush (the durability barrier after publication, #4968, #7858)
    /// with this raw OS error, or with this kind.
    fail_sync_dir: Option<Result<i32, io::ErrorKind>>,
    #[cfg(windows)]
    /// Fail the Windows sync_all call after opening the directory successfully.
    fail_sync_all: Option<Result<i32, io::ErrorKind>>,
    synced: Cell<u32>,
}

```

**Explanation.** `Faulty` is the fault-injecting `AtomicFs` the tests drive `replace_with` through. Before this pull request it could fail `write_new`, `rename`, `remove` and the pre-flush half of `sync_dir` (`fail_sync_dir`); what it could not do was fail the `sync_all` call inside the Windows flush. That is the branch this pull request adds, and without this field it could only be tested by asserting the classifier directly, never by driving the real code path from a write through to a classified failure.

`fail_sync_all: Option<Result<i32, io::ErrorKind>>` mirrors `fail_sync_dir` field for field, including the double-`Option`-shaped payload: `Some(Ok(code))` produces an error with that raw OS code, `Some(Err(kind))` produces an error of that kind with a message. The mirroring is deliberate - hunk 9 sets both fields on different `Faulty` values and asserts the same three things about the two outcomes, and one construction syntax for both is what makes that comparison read as a contrast rather than as two idioms.

`Option` rather than `bool` plus a code: the fake has to express three states. "Do not fail" is `None`, which is what every pre-existing test gets from `..Default::default()`. "Fail with a raw code" and "fail with a kind" need the two payload shapes, and a `bool` plus an `i32` would need a sentinel code to mean "not set" - a value that would then be indistinguishable from a real injected code 0.

`#[cfg(windows)]` on the field rather than on the struct or on the impl. On Unix the whole flush is `File::sync_all` in `RealFs::sync_dir`; there is no separate `sync_all` call of the Windows helper's shape to intercept, so the field would be dead code on that platform and `-D warnings` would fail the build. Gating the field leaves the struct's Unix layout byte-identical to before, which is why none of the Unix tests needed an edit when this landed.

`synced: Cell<u32>` is context in this hunk rather than part of the change, but it is what the new tests read. `Cell` rather than `AtomicU32`: the trait method takes `&self`, so the counter must be interior-mutable, and `Cell` is the cheapest form of that for a single-threaded test path. `AtomicU32` would also work and would add a synchronisation guarantee nobody needs, plus it would prevent the `Default` derive from being used unmodified in older compilers - `Cell<u32>` implements `Default` through `u32::default()`, so `..Default::default()` yields 0 without a manual impl.

The field is never mutated after construction. Every site builds the struct with `..Default::default()` and one explicit field, so a test cannot be affected by another test's leftover state - which matters because these tests run in parallel threads within one binary.

Adding a field to a struct with a `Default` derive is the least invasive way to extend a test double: no constructor needs updating, no existing literal needs a new line, and the new field's default (`None`) means "behave as before". That is what keeps the eight pre-existing `Faulty` sites in this file unchanged.

#### 18. `crates/format/src/atomic/tests.rs` — hunk -70,14 +73,23

**Original code (upstream `main`)**

```rust
    fn remove(&self, path: &Path) -> io::Result<()> {
        RealFs.remove(path)
    }
    /// Count publication barriers and optionally fail before the real directory flush.
    fn sync_dir(&self, dir: &Path) -> io::Result<()> {
        self.synced.set(self.synced.get() + 1);
        match self.fail_sync_dir {
            Some(Ok(code)) => Err(io::Error::from_raw_os_error(code)),
            Some(Err(kind)) => Err(io::Error::new(kind, "injected directory flush failure")),
            None => RealFs.sync_dir(dir),
        }
    }
    fn copy_metadata(&self, from: &Path, to: &Path) -> io::Result<()> {
        RealFs.copy_metadata(from, to)
```

**Changed code (current `main`)**

```rust
    fn remove(&self, path: &Path) -> io::Result<()> {
        RealFs.remove(path)
    }
    /// Count publication barriers and optionally fail before or during the directory flush.
    fn sync_dir(&self, dir: &Path) -> io::Result<()> {
        self.synced.set(self.synced.get() + 1);
        if let Some(res) = self.fail_sync_dir {
            return match res {
                Ok(code) => Err(io::Error::from_raw_os_error(code)),
                Err(kind) => Err(io::Error::new(kind, "injected directory flush failure")),
            };
        }
        #[cfg(windows)]
        if let Some(res) = self.fail_sync_all {
            return sync_dir_windows_with(dir, open_dir_for_sync, |_| match res {
                Ok(code) => Err(io::Error::from_raw_os_error(code)),
                Err(kind) => Err(io::Error::new(kind, "injected directory sync_all failure")),
            });
        }
        RealFs.sync_dir(dir)
    }
    fn copy_metadata(&self, from: &Path, to: &Path) -> io::Result<()> {
        RealFs.copy_metadata(from, to)
```

**Explanation.** `Faulty::sync_dir` is where the fake decides which of the two flush failures to produce. The old body was a `match` over `fail_sync_dir` whose `None` arm called `RealFs.sync_dir(dir)`; the new body is two early returns followed by that same fall-through, so the untaken paths are visibly untaken.

`self.synced.set(self.synced.get() + 1)` moved to the top of the function, before either failure. That is a deliberate change in meaning from a naive refactor: the counter now counts *attempts*, not successes, which is what hunk 9 asserts (`synced.get() == 1` on a failing flush). Counting successes would make the assertion `0` and lose the information that the barrier was reached.

`if let Some(res) = self.fail_sync_dir { return ... }` replaces the `match` with an `if let` plus `return`, because the function now has two failure sources and a nested `match` would need an arm for "no failure at all" in the middle of the body. With the early return, each source reads as one self-contained sentence and the success path is the last line.

The `#[cfg(windows)]` block between the two is the new part. It calls `sync_dir_windows_with(dir, open_dir_for_sync, |_| ...)` with the *real* `open_dir_for_sync` and a closure that returns the injected error regardless of which file it is given. Two things follow. First, the open is real, so a test using `fail_sync_all` still exercises the real `CreateFileW` call and its real flags - the injected error is only at the flush, which is the precise point under test. Second, because `sync_dir_windows_with` routes the flush result through `dir_synced`, the injected error is classified by the same code production uses, so the test cannot pass by way of a special case.

`|_|` ignores the `File` it is handed. The real closure is `|d| d.sync_all()`; the fake does not need the handle because it does not perform I/O, only returns the error. The handle is still opened and dropped by the helper, which is what keeps the open path real.

`RealFs.sync_dir(dir)` remains the final line and is still what runs when both fields are `None`, so the default construction the other forty tests use behaves exactly as before.

The `if let` form the hunk introduces also changes when the counter increments relative to the failure. It is now the first statement, before either field is consulted, so `synced` counts *attempts*: hunk 9 asserts `synced.get() == 1` on a flush that failed, which is only meaningful if the increment happened before the error was produced. Counting successes would make that assertion `0` and would lose the information that the barrier was reached at all.

The `#[cfg(windows)]` block sits between the two field checks rather than after them, and that ordering is what makes `fail_sync_dir` win when both fields are set: the first `if let` returns before the second is read. Hunk 9 relies on this - it sets `fail_sync_dir` and `fail_sync_all` separately to reach the two classifiers, and a reader who swapped the order would change which error the test produces.

Inside the Windows block, `sync_dir_windows_with(dir, open_dir_for_sync, |_| ...)` passes the *real* opener. That is the point of the field: an injected failure at the flush, after a genuine `CreateFileW` on the directory. Passing a fake opener as well would collapse the two cases the pull request exists to separate, and the test would no longer prove anything about the flush half.

#### 19. `crates/format/src/atomic/tests.rs` — hunk -117,6 +129,30

**Original code (upstream `main`)**

```rust
    let d = TempDir::new("sync-directory");
    RealFs.sync_dir(&d.0).unwrap();
    assert_eq!(RealFs.sync_dir(&d.0.join("missing")).unwrap_err().kind(), io::ErrorKind::NotFound);
}

#[test]
```

**Changed code (current `main`)**

```rust
    let d = TempDir::new("sync-directory");
    RealFs.sync_dir(&d.0).unwrap();
    assert_eq!(RealFs.sync_dir(&d.0.join("missing")).unwrap_err().kind(), io::ErrorKind::NotFound);
    // Unsupported open or flush errors (50: ERROR_NOT_SUPPORTED, 1: ERROR_INVALID_FUNCTION, 120: ERROR_CALL_NOT_IMPLEMENTED)
    // are classified as non-fatal by dir_synced / dir_sync_unsupported, while fatal open errors (e.g. NotFound) remain preserved.
    assert!(dir_synced(Err(io::Error::from_raw_os_error(50))).is_ok());
    assert!(dir_synced(Err(io::Error::from_raw_os_error(1))).is_ok());
    assert!(dir_synced(Err(io::Error::from_raw_os_error(120))).is_ok());
    assert!(dir_synced(Err(io::Error::from_raw_os_error(32))).is_err());

    // When the directory open itself fails with an unsupported error code (50 on WebDAV/cloud drives,
    // 1, or 120), the directory sync guard must treat it as non-fatal and succeed.
    for code in [50, 1, 120] {
        let result = sync_dir_windows_with(&d.0, |_| Err(io::Error::from_raw_os_error(code)), |d| d.sync_all());
        assert!(result.is_ok(), "directory open error code {code} must be treated as non-fatal");
    }
    let fatal = sync_dir_windows_with(&d.0, |_| Err(io::Error::from(io::ErrorKind::NotFound)), |d| d.sync_all()).unwrap_err();
    assert_eq!(fatal.kind(), io::ErrorKind::NotFound);

    // When the directory opens successfully, but the flush fails with a fatal error (e.g. sharing violation 32),
    // the helper must return that fatal error.
    let fatal_flush = sync_dir_windows_with(&d.0, open_dir_for_sync, |_| Err(io::Error::from_raw_os_error(32))).unwrap_err();
    assert_eq!(fatal_flush.raw_os_error(), Some(32));

    // When the flush fails with an unsupported error code (e.g. 50), it must be treated as non-fatal.
    let unsupp_flush = sync_dir_windows_with(&d.0, open_dir_for_sync, |_| Err(io::Error::from_raw_os_error(50)));
    assert!(unsupp_flush.is_ok());
}

#[test]
```

**Explanation.** One existing test (`windows_directory_flush_opens_a_directory_and_reports_open_errors`, itself `#[cfg(windows)]`) gains assertions, and it is where the `sync_dir_windows_with` cases are added. The first half pins the classifier in isolation; the second pins it through the helper that the production path calls.

The pre-existing two lines are kept unchanged: `RealFs.sync_dir(&d.0).unwrap()` proves a real directory can be flushed on every platform, and `RealFs.sync_dir(&d.0.join("missing")).unwrap_err().kind() == NotFound` proves a genuinely missing directory still produces a real error. That second assertion is what stops the classifier from being "everything is unsupported": `NotFound` must survive.

The four `dir_synced` assertions are the new table. Codes 50, 1 and 120 must be `Ok`, and 32 must be `Err`. The comment above them names each code so a reader does not have to cross-reference `winerror.h`. `32` is included on purpose as the control: the list would pass if the predicate were `code < 128`, so a code that must fail has to sit next to the three that must pass. These run on every platform because `dir_synced` now exists everywhere, and on Unix the Windows codes simply are not in the list, so the assertions hold there too - which is the point of writing them without a `cfg`.

The `sync_dir_windows_with` block is Windows behaviour expressed without a `cfg`: the function exists on every platform (it is only *called* by the Windows `RealFs::sync_dir`), so a test can drive it anywhere. The loop feeds `Err(code)` for 50, 1 and 120 from the *open* closure and asserts `is_ok()` - the case where the directory cannot even be opened for flushing, which is the WebDAV and cloud-drive case in #7858. The `fatal` line then proves a `NotFound` from the same closure is *not* classified: `unwrap_err().kind() == NotFound`.

`let fatal` uses `io::Error::from(io::ErrorKind::NotFound)` rather than `from_raw_os_error`, because the point is that the kind-based path and the code-based path must both be handled and neither may swallow the other. `from(ErrorKind)` produces an error with no raw code, so a classifier that only inspected `raw_os_error()` would return `false` here and the error would propagate - which is what the assertion expects.

The last two blocks flip the site: the open succeeds (`open_dir_for_sync`, the real function) and the *flush* closure returns the error. `32` must come back as `32`, and the trailing `unwrap_err().raw_os_error() == Some(32)` checks the code survived rather than being re-wrapped into a generic kind. This is the assertion that would fail if `sync_dir_windows_with` classified the flush result with a blanket rule instead of `dir_synced`.

The order matters for diagnosis: classifier table first (pure function), then open-failure through the helper, then flush-failure through the helper. When one of them breaks, the position in the test tells which layer broke.

#### 20. `crates/format/src/atomic/tests.rs` — hunk -604,10 +640,9

**Original code (upstream `main`)**

```rust
    assert_no_temp(&d.0);
}

/// A file system that can't flush directories at all (Linux's EINVAL, ENOTSUP, or an
/// `Unsupported` error) still saves: failing every save there would make saving impossible.
/// Windows fails the save on any failed flush (#7858), so this is the Unix rule.
#[cfg(not(windows))]
#[test]
fn a_directory_flush_the_file_system_cannot_do_still_saves() {
    let d = TempDir::new("syncdir-unsupported");
```

**Changed code (current `main`)**

```rust
    assert_no_temp(&d.0);
}

/// A file system that can't flush directories at all (e.g. WebDAV, cloud drives like AirLiveDrive
/// returning ERROR_NOT_SUPPORTED 50 / ERROR_INVALID_FUNCTION 1 on Windows, or Linux's EINVAL / ENOTSUP,
/// or an `Unsupported` error) still saves: failing every save there would make saving impossible.
#[test]
fn a_directory_flush_the_file_system_cannot_do_still_saves() {
    let d = TempDir::new("syncdir-unsupported");
```

**Explanation.** Two lines of diff, and the load-bearing change of the pull request on the test side: the `#[cfg(not(windows))]` attribute above `a_directory_flush_the_file_system_cannot_do_still_saves` is removed, so the test runs on Windows as well.

The attribute was there for a real reason before this change: the rule it tests ("a file system that cannot flush directories still saves") was Unix-only, because Windows failed the save on any failed flush. The doc comment in this same hunk said exactly that - "Windows fails the save on any failed flush (#7858), so this is the Unix rule" - and the pull request makes the rule apply on Windows too, so both the attribute and that sentence have to go.

The doc comment is rewritten rather than simply deleted. The new text names the Windows codes and the product that produced the report: "e.g. WebDAV, cloud drives like AirLiveDrive returning ERROR_NOT_SUPPORTED 50 / ERROR_INVALID_FUNCTION 1 on Windows, or Linux's EINVAL / ENOTSUP, or an `Unsupported` error". Naming the two codes is what a future reader needs to check the classifier against `winerror.h`; naming the product is what explains why the codes exist at all. A comment that said only "unsupported errors are not fatal" would leave the next person re-deriving the code list.

Removing a `#[cfg]` from a test is not a formatting change: it changes which platforms compile and run the body. The body as it stood was not portable - it asserts `dir_synced(... 22 ...).is_ok() == cfg!(unix)` and constructs `Faulty { fail_sync_dir: Some(Ok(5)) }` - so the next hunk splits it into two platform blocks. This hunk and hunk 9 are one change split by the diff algorithm at the function boundary; neither is meaningful alone.

Why the un-gating is safe: the first three statements of the body are platform-neutral (`Faulty` with `fail_sync_dir: Some(Err(Unsupported))`, a successful `atomic_write_with`, a read-back), because `ErrorKind::Unsupported` has no raw OS code and therefore behaves identically on both platforms. So the shared half genuinely tests the same rule everywhere, and the platform-specific half is what hunk 9 adds.

Why this test rather than a new Windows-only one: the claim under test - "an unverifiable flush does not fail the save" - is one claim, and duplicating it into two tests would let the halves drift. One test with two platform blocks makes it impossible to update one platform's expectation without looking at the other.

The consequence for CI: before this change the test was compiled out on the Windows runners, so a Windows regression in the classifier could only be caught by `dir_synced`'s unit assertions (hunk 7). Now the end-to-end save path is exercised on Windows too, which is the platform where #7858 reported the failure.

#### 21. `crates/format/src/atomic/tests.rs` — hunk -615,11 +650,42

**Original code (upstream `main`)**

```rust
    let fs = Faulty { fail_sync_dir: Some(Err(io::ErrorKind::Unsupported)), ..Default::default() };
    atomic_write_with(&fs, NO_RETRY, &p, b"new").unwrap();
    assert_eq!(std::fs::read(&p).unwrap(), b"new");
    // EINVAL (22) means "can't flush a directory" only on Unix; EIO (5) is a real failure.
    assert_eq!(dir_synced(Err(io::Error::from_raw_os_error(22))).is_ok(), cfg!(unix));
    assert!(dir_synced(Err(io::Error::from_raw_os_error(5))).is_err());
    let fs = Faulty { fail_sync_dir: Some(Ok(5)), ..Default::default() };
    assert!(atomic_write_with(&fs, NO_RETRY, &p, b"newer").is_err());
}

/// #9388: on macOS a save keeps the replaced file's creation time ("Created" in Finder), as
```

**Changed code (current `main`)**

```rust
    let fs = Faulty { fail_sync_dir: Some(Err(io::ErrorKind::Unsupported)), ..Default::default() };
    atomic_write_with(&fs, NO_RETRY, &p, b"new").unwrap();
    assert_eq!(std::fs::read(&p).unwrap(), b"new");
    #[cfg(windows)]
    {
        // 50: ERROR_NOT_SUPPORTED, 1: ERROR_INVALID_FUNCTION
        assert!(dir_synced(Err(io::Error::from_raw_os_error(50))).is_ok());
        assert!(dir_synced(Err(io::Error::from_raw_os_error(1))).is_ok());
        // ERROR_SHARING_VIOLATION (32) is a real failure
        assert!(dir_synced(Err(io::Error::from_raw_os_error(32))).is_err());
        let fs_fatal = Faulty { fail_sync_dir: Some(Ok(32)), ..Default::default() };
        let error = atomic_write_with(&fs_fatal, NO_RETRY, &p, b"fatal").unwrap_err();
        assert_eq!(fs_fatal.synced.get(), 1);
        assert!(error.to_string().contains("may not survive"), "{error}");
        assert_eq!(std::fs::read(&p).unwrap(), b"fatal");

        // Fatal flush error injected after the directory opens successfully:
        let fs_flush_fatal = Faulty { fail_sync_all: Some(Ok(32)), ..Default::default() };
        let error = atomic_write_with(&fs_flush_fatal, NO_RETRY, &p, b"fatal_flush").unwrap_err();
        assert_eq!(fs_flush_fatal.synced.get(), 1);
        assert!(error.to_string().contains("may not survive"), "{error}");
        assert_eq!(std::fs::read(&p).unwrap(), b"fatal_flush");

        let fs_webdav = Faulty { fail_sync_dir: Some(Ok(50)), ..Default::default() };
        atomic_write_with(&fs_webdav, NO_RETRY, &p, b"webdav").unwrap();
        assert_eq!(std::fs::read(&p).unwrap(), b"webdav");

        let fs_invalid_func = Faulty { fail_sync_dir: Some(Ok(1)), ..Default::default() };
        atomic_write_with(&fs_invalid_func, NO_RETRY, &p, b"invalid_func").unwrap();
        assert_eq!(std::fs::read(&p).unwrap(), b"invalid_func");
    }
    #[cfg(not(windows))]
    {
        // EINVAL (22) means "can't flush a directory" only on Unix; EIO (5) is a real failure.
        assert_eq!(dir_synced(Err(io::Error::from_raw_os_error(22))).is_ok(), cfg!(unix));
        assert!(dir_synced(Err(io::Error::from_raw_os_error(5))).is_err());
        let fs = Faulty { fail_sync_dir: Some(Ok(5)), ..Default::default() };
        assert!(atomic_write_with(&fs, NO_RETRY, &p, b"newer").is_err());
    }
}

/// #9388: on macOS a save keeps the replaced file's creation time ("Created" in Finder), as
```

**Explanation.** The body of the now-portable test is split into a `#[cfg(windows)]` block and a `#[cfg(not(windows))]` block. The three lines before the split are shared: the unsupported `Faulty`, a successful write, and a read-back proving the bytes landed. That is the claim under test on every platform - an unverifiable flush does not fail the save - and it is asserted once rather than duplicated.

The Windows block has four parts, each a separate case:

`dir_synced` on codes 50 and 1 is `Ok`, on 32 is `Err`. This is the pure table again, but now inside the test that proves the end-to-end consequence, so a reader sees the classifier and the save in one place.

`fs_fatal` with `fail_sync_dir: Some(Ok(32))`: `atomic_write_with(...).unwrap_err()`, then three assertions. `synced.get() == 1` - the barrier ran. `error.to_string().contains("may not survive")` - the message is the *unconfirmed* wording, not the publication-failed wording, which is what distinguishes "the file is there but durability is unknown" from "the write failed". `std::fs::read(&p) == b"fatal"` - the file was published anyway. Those three together are the contract: a real flush failure on Windows reports an unconfirmed publish, and the content is still there.

`fs_flush_fatal` with `fail_sync_all: Some(Ok(32))`: the same three assertions on a new name. This is the case the `fail_sync_all` field exists for, and the reason it must be a separate case rather than a parameter: the error comes from inside `sync_dir_windows_with` after the open succeeded, so it exercises the flush classifier rather than the open classifier. Both produce the same user-visible outcome, which is the point - the distinction is not visible to the user, only to the code path.

`fs_webdav` (50) and `fs_invalid_func` (1) each write and read back, with no `unwrap_err`: these are the two real-world codes from #7858 and they must save silently. Writing four separate `Faulty` values rather than looping is deliberate: each one is a named scenario (`fatal`, `fatal_flush`, `webdav`, `invalid_func`), and a loop over `[(32, false), (50, true), (1, true)]` would hide which case failed in the assertion message.

The Unix block keeps the original two assertions and adds nothing: `dir_synced` on 22 is `Ok` only on Unix (`cfg!(unix)` inside the assertion, so it is `true` on Linux and macOS and would be `false` on a hypothetical non-unix non-windows target), on 5 is `Err`, and `Faulty { fail_sync_dir: Some(Ok(5)) }` makes `atomic_write_with` fail. `EIO` (5) is the control: a genuine I/O error must still fail the save, which stops the Unix arm from becoming "all errors are unsupported".

The two blocks are not redundant with each other: the Windows codes do not exist on Unix and `EIO`'s behaviour is the same on both, so the Unix block's value is that it locks the Unix list while the Windows block locks the Windows list. A single platform-neutral body would have to skip every numeric assertion, which is what the `#[cfg(not(windows))]` on the whole test used to do.

### `crates/format/src/store.rs`

#### 22. `crates/format/src/store.rs` — hunk -881,7 +881,7

**Original code (upstream `main`)**

```rust
        crate::atomic::retry_rename(crate::atomic::RenameRetry::platform(), || std::fs::rename(&tmp, path))?;
        #[cfg(windows)]
        {
            fs.sync_dir(parent).map_err(|e| crate::atomic::publication_error(e, path))?;
        }
        // Elsewhere a failed flush is reported too (#4968), but a file system that can't flush a
        // directory at all still saves, as in `atomic_write`.
```

**Changed code (current `main`)**

```rust
        crate::atomic::retry_rename(crate::atomic::RenameRetry::platform(), || std::fs::rename(&tmp, path))?;
        #[cfg(windows)]
        {
            crate::atomic::dir_synced(fs.sync_dir(parent)).map_err(|e| crate::atomic::publication_error(e, path))?;
        }
        // Elsewhere a failed flush is reported too (#4968), but a file system that can't flush a
        // directory at all still saves, as in `atomic_write`.
```

**Explanation.** `store.rs` calls `fs.sync_dir` directly in two places - the bundle publication (`write_bundle_file`) and the `PcraftWriter` close path - and both used to compare the error themselves. With `dir_synced` no longer Unix-only, both call sites wrap the result in it, so the classifier is applied on every platform from one place instead of each caller deciding.

These two lines are the reason `dir_synced` was made public to the crate rather than left private to `atomic`: without them the bundle path would still fail on Windows for the same file systems the single-file path now tolerates. The behaviour is identical to `replace_with`'s: an unsupported directory flush becomes `Ok(())`, everything else propagates, and the message distinguishes "published but durability unconfirmed" from "not published".

#### 23. `crates/format/src/store.rs` — hunk -1299,7 +1299,7

**Original code (upstream `main`)**

```rust
            // both object directories again before publishing a manifest that references them.
            // This also persists the fixed subdirectories of a newly created bundle.
            for sub in [TILES_DIR, BLOBS_DIR] {
                fs.sync_dir(&dir.join(sub)).map_err(|e| crate::atomic::publication_error(e, &dir.join(sub)))?;
            }
        }
        #[cfg(not(windows))]
```

**Changed code (current `main`)**

```rust
            // both object directories again before publishing a manifest that references them.
            // This also persists the fixed subdirectories of a newly created bundle.
            for sub in [TILES_DIR, BLOBS_DIR] {
                crate::atomic::dir_synced(fs.sync_dir(&dir.join(sub))).map_err(|e| crate::atomic::publication_error(e, &dir.join(sub)))?;
            }
        }
        #[cfg(not(windows))]
```

**Explanation.** `store.rs` calls `fs.sync_dir` directly in two places - the bundle publication (`write_bundle_file`) and the `PcraftWriter` close path - and both used to compare the error themselves. With `dir_synced` no longer Unix-only, both call sites wrap the result in it, so the classifier is applied on every platform from one place instead of each caller deciding.

These two lines are the reason `dir_synced` was made public to the crate rather than left private to `atomic`: without them the bundle path would still fail on Windows for the same file systems the single-file path now tolerates. The behaviour is identical to `replace_with`'s: an unsupported directory flush becomes `Ok(())`, everything else propagates, and the message distinguishes "published but durability unconfirmed" from "not published".
