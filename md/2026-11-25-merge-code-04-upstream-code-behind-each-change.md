# The upstream code behind each fork adaptation

Documents 01 and 02 explain the fork's own lines. This one shows the **upstream code that forced each
adaptation**, quoted from `upstream/main` (photocraft) and `upstream/next` (craft-libs) with the path it
came from, and says exactly what that code does that the fork had to answer.

Why the whole of upstream is not here: the two merges carry 629 + 1,279 commits (document 03 lists every
one of them). Quoting all of that would be a copy of `storytold`'s repositories. What a reader of *this
fork* needs is the code that intersects the fork's own behaviour, and that is the set below - one entry
per adaptation the fork had to make.

## 1. The engine split: a core session installs no commands

`crates/engine-core/src/install.rs` (upstream `main`):

```rust
pub fn session() -> Session {
    // A conflicting install reports itself at the session's first command.
    let _ = install();
    Session::new()
}
```

`crates/engine-core/src/error.rs` (upstream `main`):

```rust
NotInstalled,
```

**What it does.** Upstream split `engine` into a facade over `engine-core`. `Session::new()` in the
core crate installs **nothing**; the first command then answers `NotInstalled`. The `engine` crate's
`Session::new()` installs every command, which is why an application never noticed.

**Why it broke the fork.** The fork's own trials inside `engine-core` (LaMa cleaner, Remove, pick
commands, SAM selection) built a bare `Session::new()` and ran commands - they were written before the
split. `cargo check` passes with this bug; only running them shows it.

**The fork's answer.** Every such trial starts from `crate::install::session()`, which calls `install()`
first (document 01, and the engine-core repair described in the skill's 3.1).

## 2. Space: upstream activates with it, the fork reserves it

`crates/ui-contract/src/runtime/focus/policy.rs` (upstream `next`):

```rust
Input::Pointer => (KeepKeys::None, ActivateKeys::Both, OwnerClass::Widget),
```

`crates/ui-contract/src/runtime/keys/route.rs` (upstream `next`), inside `press_route`:

```rust
None if press.key == Key::Space && plain && stop.keys.activate.space() => PressRoute::Activate,
```

**What it does.** Every pointer control takes `ActivateKeys::Both`, so Space activates it, on the
**press** (`PressRoute::Activate`). Upstream's `Link`, `word_run` and `select_list` trials assert that.

**Why it broke the fork.** The fork keeps a bare Space for the canvas's Hand key (and, on a canvas,
Space used to open the File menu, whose first item the next press ran). With `Both`, every suite trial
runs its Space variant - 26 of them broke when measured - and the canvas loses its key.

**The fork's answer.** `ActivateKeys::Enter` as the pointer default, a `space_activates` flag for the
controls that ask (`button`, `link`, `word_run`, `select`), and `PressRoute::Space` activating on the
**release** of the stop that was pressed (document 01, A.3-A.5).

## 3. Dates: upstream's fallback is the machine's region

`crates/ui-contract/src/runtime/env/host_words.rs` (upstream `next`):

```rust
pub(super) fn host_dates(ctx: &Context, host: Option<&Arc<Host>>) -> Arc<DateFormat> {
    if let Some(i18n) = host.and_then(|host| host.i18n_shared()) {
        return i18n.dates();
    }
    #[derive(Default)]
    struct System(Option<Arc<DateFormat>>);
    Service::<System>::of(ctx).with(|system| Arc::clone(system.0.get_or_insert_with(|| Arc::new(DateFormat::new(&craft_i18n::Region::system())))))
}
```

**What it does.** Without an installed host the date format comes from `Region::system()` - the
machine's region - while `host_words` above it gives **English** words.

**Why it broke the fork.** The trial that reads a calendar could not find "Su" on a non-English
machine: the weekdays were painted in the machine's language. Measured: it fails on upstream's own tree
too, which is how the cause was found - and that is a diagnosis, not the end of the work.

**The fork's answer.** `en-US` for both dates and numbers when no host is installed (document 01, B.5);
an application with catalogs never reaches the fallback.

## 4. ASS: upstream's own escape leaves `{}` as text

`crates/captions/src/ass.rs` (upstream `next`), inside `fn escape`:

```rust
} else if let Some(c) = rest.chars().next() {
    match c {
        '\n' | '\r' => out.push_str("\\N"),
        // Empty override block separates a literal slash from N/n/h.
        '\\' => out.push_str("\\{}"),
```

**What it does.** Upstream separates a literal backslash from a following `N`/`n`/`h` with `{}`,
assuming ffmpeg treats `{}` as an override block. It does not: `{}` is plain text to ffmpeg's reader.

**Why it broke the fork.** A caption's literal `\N` came back as `\{}N` - three characters where the
document had two - and the round-trip trial caught it on ffmpeg 7.1.

**The fork's answer.** `\{\i0}` instead: a real override block, stripped by ffmpeg, which leaves the
backslash before it (document 01, B.7). Measured both ways.

## 5. Print: upstream flattens a failure into an empty list

`crates/print/src/spool.rs` (upstream `next`):

```rust
let printers = backend.printers().unwrap_or_default();
```

**What it does.** `SpoolBackend::printers()` answers `Option<Vec<Printer>>`; `None` means "the platform
could not list them". `unwrap_or_default()` turns that into `Vec::new()`, which the dialog cannot tell
apart from "the spooler answered and there are no printers".

**Why it broke the fork.** The fork's dialog kept the remembered printer when it saw an empty list
(a spooler hiccup must not drop the user's queue), while upstream's trial asserts an empty list
*replaces* it. The two rules cannot both hold while the failure has no representation.

**The fork's answer.** The failure keeps its own path: `printer_queues_checked()` carries `None` to the
dialog, the discovery channel closes without an answer, and the panel keeps the printer and says the
listing failed - while a listing that answers still replaces a destination it does not name
(document 01, B.1-B.4).

## 6. The control channel's own trial: a request line without its head

`crates/automation/tests/it/control.rs` (upstream `next`):

```rust
fn a_browser_post_never_reaches_the_app() {
    // Far longer than the trial, so the close comes from the refusal, not the deadline.
    let (rx, server) = server::serve_with_deadline(TcpListener::bind("127.0.0.1:0").expect("bound"), TOKEN.into(), host(), Duration::from_secs(60))
        .expect("started");
    let mut stream = TcpStream::connect(server.addr).expect("connected");
    stream.set_read_timeout(Some(Duration::from_secs(5))).expect("timeout");
    let mut reader = BufReader::new(stream.try_clone().expect("cloned"));
    let body = format!("{}\n{}\n", json!({"id": 1, "method": "auth", "params": {"token": TOKEN}}), json!({"id": 2, "method": "test.ping"}));
    write!(stream, "POST / HTTP/1.1\r\n").expect("written");
    let refused = read_json(&mut reader).expect("a reply to the request line");
    assert_eq!(refused["ok"], false, "{refused}");
    assert_eq!(refused["error"], "authentication required", "{refused}");
    // The rest of what the browser sends; the server may already have closed.
    let _ = write!(stream, "Host: 127.0.0.1\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{body}", body.len());
```

**What it does.** It writes the request line **alone**, then waits for the refusal, then sends the
headers and the body. The assertion is that a web page cannot reach the channel: the refusal answers
the request line, and the body's `auth` line is never dispatched.

**Why it broke the fork.** On a host running ESET's HTTP inspection (`epfwwfp`), an incomplete request
head is held and never passed on: measured with plain sockets, a bare `POST / HTTP/1.1\r\n` arrived
with 0 bytes in 10 seconds, the same line plus the blank line in 13 ms. The server never saw the line,
so no server change could answer it.

**The fork's answer.** The fixture writes the request line **and the end of the head** in one write, as
a browser does (document 01, B.9). No assertion moved: the reply still answers the request line, the
body's token still never reaches the application, the connection still closes.

## 7. The rest of the surface, named rather than quoted

| upstream change | where it lives | the fork's answer |
|---|---|---|
| `retouch_cmds` moved from `engine-core` to `engine-paint` | `crates/engine-paint/src/cmd/retouch_cmds/` | `engine-paint` gains `photocraft-codecs` and `tempfile`; the module lines leave `engine-core` |
| `layer_copy_cmds` trials moved into `engine/tests/it/*_in_layer_copy_cmds.rs` | `crates/engine/tests/it/` | take upstream's empty side, drop the duplicate block |
| upstream's preference trials look for the English words they paint | `crates/ui-egui/src/prefs/prefs_ui/tests.rs` | the shared harness pins `interface.language = "en"`; the trial bodies are untouched, the application stays Japanese |
| the workspace-test harness builds `PATH` the POSIX way | `scripts/test-workspace-tests.py` | take upstream's copy whole (document 01, C) |
| the design lint forbids egui memory access outside `runtime::egui::memory` | `design-lint` rules | the held-Space record goes through `memory::unique` / `peek_clone` / `take` |
| the nextest profile groups GPU trials one at a time | `.config/nextest.toml` | the fork adds `retries = 1` and keeps `gpu = { max-threads = 1 }` |

## 8. What a reader should take from this

- Upstream changed behaviour in six places that the fork had opinions about; each fork opinion survives
  as an explicit declaration, not as a hidden patch: a `KeyUse` flag, a checked listing, an `en-US`
  fallback, a real override block, a memory access through the allowed module.
- Two of those six were **upstream bugs the fork found by tripping over them** (`{}` in ASS, `Region::system()`
  as the library default); the fork's fix is also a fix upstream will want.
- One was **the host's network filter**, not either side's code: the trial was right, the fixture had to
  send what a browser sends.
- Nothing here edits an assertion. Where a trial was wrong it was the fixture's *shape* that changed,
  and where the fork was wrong it was the fork's *code*.
