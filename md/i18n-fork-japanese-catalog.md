# The Japanese Transform submenu, and why this fork now carries its own catalog

**Date:** 2026-11-20 · **Version:** 0.23.1 · **Commits:** `68e7d33e92` (catalog and build script), `8b15cb1087` (guard comments), plus this document · **Branch:** `main`, pushed to `origin` (`ussoewwin/photocraft`)

This is the full account of one defect and its fix: the Japanese interface labelled `Skew` as 斜体 and
`Distort` as ゆがみ, edits made to fix it disappeared with the next build, and the fix ended up moving
the fork's Japanese catalog into the repository itself.

---

## 0. The short version

| | |
|---|---|
| Symptom | `Edit > Transform > Skew` read 斜体 ("italic"); `Distort` read ゆがみ, the same word as the `Filter > Distort` submenu title |
| Cause | Interface strings are build-time inputs from an external checkout (`CRAFT_I18N_DIR`); the Japanese rows existed only in `storytold/craft-i18n`, which the fork cannot write to. A local copy of that checkout was temporary by definition, so every edit died with it |
| Second cause | `Distort` serves two meanings (a command and a submenu title) and only the plain source string was translatable, so the two could not be given different words without an id-keyed row |
| Fix | `crates/ui-egui/i18n/fork/ja/{photocraft,common}.tsv` shipped in the repository, embedded by `build.rs` with or without a checkout, winning over the checkout for the same locale; four rows settled |
| Evidence | 37 i18n tests pass with 35 locales and `CRAFT_I18N_REQUIRED=1`; the new test passes with no checkout at all; the release binary (303,841,280 B, 20:06:37) carries `多方向に伸縮` and `水平方向のゆがみを設定` and no longer carries `水平方向の斜体を設定` |

---

## 1. What went wrong

### 1.1 The labels

Menu paths, English source strings and the words the Japanese interface showed:

| Where | English (the lookup key) | Shown before | Should read |
|---|---|---|---|
| `Edit > Transform > Skew` | `Skew` | 斜体 ("italic") | ゆがみ |
| Tooltip of the Skew transform | `Set horizontal skew` | 水平方向の斜体を設定 | 水平方向のゆがみを設定 |
| Tooltip of the Skew transform | `Set vertical skew` | 垂直方向の斜体を設定 | 垂直方向のゆがみを設定 |
| `Edit > Transform > Distort` | `Distort` (`edit.transform.distort`) | ゆがみ | 多方向に伸縮 |

The recorded source of the first three was the app catalog row at `locales/ja/photocraft.tsv:2853`
(`Skew` → 斜体) in the craft-i18n checkout; the two `Set … skew` rows were built from the same word.
`Distort` had **no app row at all**, so it fell back to the shared catalog's row at
`common.tsv:344` (`Distort` → ゆがみ).

### 1.2 Two meanings, one key

`Distort` is two different things in PhotoCraft:

- a **command** — `crates/ui-egui/src/menu_catalog.rs:111`:
  `(&["Edit", "Transform"], "Distort", None, "edit.transform.distort")`
- a **submenu title** — the `Filter > Distort` group, whose eight items are declared at
  `crates/ui-egui/src/menu_catalog.rs:597-605` with `["Filter", "Distort"]` as their path.

The two reach the catalogs by different routes, and only one of them can carry an id:

| Surface | Code path | Lookup |
|---|---|---|
| A command's label | `crates/ui-egui/src/menus/items.rs:388` → `crates/ui-egui/src/menus/state.rs:233` | `tr_id(lang, id, label)` — an `@id` row beats the plain row |
| A menu or submenu **title** | `crates/ui-egui/src/menus/items.rs:355` (`translate_titles`, `level`) | `tr(lang, title)` — the plain source string only |
| A tooltip | `crates/ui-egui/src/menus/items.rs:389` | `tr(lang, tip)` |

So editing the plain `Distort` row to 多方向に伸縮 would have renamed the **Filter submenu** too —
`Filter > 多方向に伸縮 > Ripple…`. The command therefore had to be translated by command id, and the
plain row had to keep the word the submenu needs.

### 1.3 Why fixes kept vanishing

Three separate reasons, all of which had to be removed:

1. The Japanese rows lived in `storytold/craft-i18n`, a repository the fork's owner has no write
   access to. Anything edited in a local copy of it was a local scratch file.
2. `crates/ui-egui/build.rs` had exactly one input for catalogs: the checkout named by
   `CRAFT_I18N_DIR`. With it unset the generated table was empty and the interface was English only;
   with `CRAFT_I18N_REQUIRED=1` (CI and releases) it was a `cargo::error`. A fork-only catalog was
   not representable, so the fix could not live in the fork even as an addition.
3. A stale catalog file sits at `crates/ui-egui/src/i18n/ja.tsv` (204,286 B, 2026-11-20 13:31). No
   code reads it: it is mentioned nowhere except as a path string in the fixture list
   `docs/integration-testing/approved-selection.json`. Editing it changes nothing, which makes the
   "I fixed it and it came back" loop easy to enter.

---

## 2. The essential cause

**Interface translations in PhotoCraft are compile-time data, and the only compile-time input the
build accepted was a checkout outside the repository.**

A build script turns the catalogs into Rust source, and the binary contains the strings themselves.
Nothing is read at run time, so there is no file a user or a packager can edit after the build. The
chain is:

```
catalog TSV file
   └─ build.rs  ──────────►  $OUT_DIR/craft_i18n.rs
                                pub static CRAFT_I18N: &craft_i18n::Embedded = &[ (code, common, app), … ];
   └─ include_str!(tsv) ──►  the bytes, compiled into the crate
                                └─ craft_i18n::Embedded lookup, keyed by the English source string or by "@id" + command id
```

`crates/ui-egui/src/i18n.rs` then reads that table:

- `catalogs_embedded()` at `:315` — `!CRAFT_I18N.is_empty()`;
- `needs_catalogs()` at `:516` — the switch tests use; without catalogs it prints
  `skipped: built without craft-i18n …` and skips;
- `tr()` at `:448` — plain source string;
- `tr_ctx()` at `:453` — plain source string within a context;
- `tr_id()` at `:459` — `@id` row for a command id, falling back to `tr(lang, label)`;
- `own_id()` at `:467` — an `@id` row counts only if the language owns that id **and** the translation
  keeps the `{placeholders}` and the trailing `…` of the English label (`same_shape`). craft-i18n
  cannot check that itself, because an `@id` row's source is the id, not the label.

Because the only source was an external checkout, the word choice was tied to a repository the fork
does not own, and to whatever copy of it happened to be on the build machine. That is the root cause;
everything in §1.3 is a consequence of it.

Two further consequences of the same design, worth stating because they shaped the fix:

- **A locale is all-or-nothing.** The table holds one entry per locale: `(code, shared catalog, app
  catalog)`. There is no way to add three rows to a language; the entry either exists or the language
  falls back to English.
- **Precedence had to be defined for the first time.** Once two sources feed one locale, something
  must decide. It is the fork (see §5.1).

---

## 3. Files created and modified

| Path | Status | Size / delta | Role |
|---|---|---|---|
| `crates/ui-egui/i18n/fork/ja/photocraft.tsv` | **new** | 175,825 B, 3,396 lines | The whole Japanese app catalog, carried by the fork; four rows settled here |
| `crates/ui-egui/i18n/fork/ja/common.tsv` | **new** | 33,194 B, 1,289 lines | The whole Japanese shared catalog, so the locale is complete without a checkout |
| `crates/ui-egui/build.rs` | modified | +89 / −17, then comment-only follow-ups | Reads `i18n/fork/<locale>`, merges it with the checkout, fork winning; copies catalogs into `OUT_DIR` for `include_str!` |
| `crates/ui-egui/src/i18n/tests.rs` | modified | +13 | One test pinning the three settled rows; runs without a checkout |
| `docs/localization.md` | modified | +12 / −2 | New section "This fork's own catalogs", and the "English only without a checkout" sentence corrected |
| `ATTRIBUTION.md` | modified | +1 | A row for the fork's catalogs, with provenance and licence |

Both TSV files are whole catalogs copied from craft-i18n at the revision in `craft-i18n.rev`
(`e56d26244bbac099bfd42e23fb12f78a4be418a7`) and then edited in four rows. They are data, not code:
the sections below quote every row that changed and explain how to regenerate them.

---

## 4. The code

### 4.1 `crates/ui-egui/build.rs` (whole file)

```rust
//! Bake the contributor credits into the binary as static Rust tables, and embed the craft-i18n
//! interface catalogs. Nothing is read at run time.
//!
//! - contributors/contributors.json: commit stats per GitHub username, with a mark for whose people.toml
//!   entry is verified.
//! - contributors/people.toml: the names people chose for themselves. A name is used only when the
//!   JSON says the entry was verified (`people_entry`).
//!
//! A missing or malformed file gives empty tables (or no names) and a warning.
//!
//! - `CRAFT_I18N_DIR`, a craft-i18n checkout (storytold/craft-i18n, at the SHA in
//!   `craft-i18n.rev`): `craft_i18n.rs` holds `CRAFT_I18N`, one (locale, shared catalog, PhotoCraft
//!   catalog) entry per locale that has a `photocraft.tsv`, embedded raw. Unset, or not a checkout
//!   (a warning), only this fork's own `i18n/fork/<locale>` catalogs are embedded and the rest of
//!   the interface is English; with `CRAFT_I18N_REQUIRED=1` (CI and releases) that is an error
//!   instead. Recipe: craft-i18n's `docs/integration.md`.
//!
//! - `i18n/fork/<locale>/{photocraft,common}.tsv`: this fork's own whole catalogs, embedded with or
//!   without a checkout and winning over the checkout's catalog for the same locale, so a word the
//!   fork settles differently lives in the fork and needs no write access to craft-i18n.

use std::fmt::Write as _;

use serde_json::Value;

fn main() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../contributors");
    let json = read(&dir.join("contributors.json"), "the About window shows no contributors", |t| serde_json::from_str::<Value>(t).map_err(|e| e.to_string()))
        .unwrap_or(Value::Null);
    let people =
        read(&dir.join("people.toml"), "contributors are shown by username only", |t| t.parse::<toml::Table>().map_err(|e| e.to_string())).unwrap_or_default();
    let out = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap_or_default()).join("credits.rs");
    if let Err(e) = std::fs::write(&out, generate(&json, &people)) {
        println!("cargo::warning=writing {}: {e}", out.display());
    }
    let out = out.with_file_name("craft_i18n.rs");
    if let Err(e) = std::fs::write(&out, craft_i18n()) {
        println!("cargo::error=writing {}: {e}", out.display());
    }
}

/// The catalog this app reads in each locale: `locales/<locale>/photocraft.tsv`.
const APP_CATALOG: &str = "photocraft.tsv";

/// This fork's own catalogs: `i18n/fork/<locale>/<catalog>`, each a whole catalog.
const FORK_CATALOGS: &str = "i18n/fork";

/// `OUT_DIR`, where a checkout's catalogs are copied so `include_str!` reads one place.
fn out_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap_or_default())
}

/// `CRAFT_I18N`: the `Embedded` list the craft-i18n crate reads. A locale comes from the checkout
/// `CRAFT_I18N_DIR` names, from this fork's own `i18n/fork/<locale>`, or from both, the fork's
/// catalog winning: the fork ships the whole Japanese catalog it settles, so it needs no checkout.
fn craft_i18n() -> String {
    println!("cargo::rerun-if-env-changed=CRAFT_I18N_DIR");
    println!("cargo::rerun-if-env-changed=CRAFT_I18N_REQUIRED");
    let required = std::env::var("CRAFT_I18N_REQUIRED").is_ok_and(|v| v.trim() == "1");
    let out = out_dir();
    let mut entries = match std::env::var_os("CRAFT_I18N_DIR").filter(|d| !d.is_empty()) {
        // Not canonicalized: on Windows that gives `\\?\C:\…`, which `include_str!` can't read.
        Some(dir) => match locales(&std::path::PathBuf::from(dir).join("locales")) {
            Ok(entries) => entries,
            Err(e) => {
                println!("cargo::warning=CRAFT_I18N_DIR: {e}; this fork's own catalogs are used");
                Vec::new()
            }
        },
        None => Vec::new(),
    };
    for (code, shared, app) in fork_locales() {
        match entries.iter_mut().find(|(known, _, _)| known == &code) {
            Some(entry) => {
                entry.1 = shared;
                entry.2 = app;
            }
            None => entries.push((code, shared, app)),
        }
    }
    if entries.is_empty() && required {
        println!("cargo::error=CRAFT_I18N_REQUIRED=1 but no catalog: set CRAFT_I18N_DIR to a craft-i18n checkout, or add one under i18n/fork");
    }
    let mut o = String::from("/// (locale, shared catalog, PhotoCraft's catalog), one entry per locale; `\"\"` for a missing file.\n");
    o.push_str("pub static CRAFT_I18N: &craft_i18n::Embedded = &[\n");
    for (code, shared, app) in entries {
        let shared = shared.map_or_else(|| "\"\"".to_string(), |p| embed(&p, &out));
        let _ = writeln!(o, "    ({code:?}, {shared}, {}),", embed(&app, &out));
    }
    o.push_str("];\n");
    o
}

/// A catalog as a Rust `include_str!`: the file itself when it is under `OUT_DIR`, else a copy there.
/// The copy keeps a checkout's rows still when the checkout moves, and a path outside `OUT_DIR` out
/// of the generated code.
fn embed(path: &std::path::Path, out: &std::path::Path) -> String {
    if path.starts_with(out) {
        return format!("include_str!({path:?})");
    }
    let _ = std::fs::create_dir_all(out);
    let Some(name) = path.file_name() else { return format!("include_str!({path:?})") };
    let Some(parent) = path.parent().and_then(|p| p.file_name()) else { return format!("include_str!({path:?})") };
    let target = out.join(format!("{}-{}", parent.to_string_lossy(), name.to_string_lossy()));
    match std::fs::copy(path, &target) {
        Ok(_) => format!("include_str!({target:?})"),
        Err(e) => {
            println!("cargo::warning=copying {}: {e}", path.display());
            format!("include_str!({path:?})")
        }
    }
}

/// One embedded locale: its code, its `common.tsv` if it has one, and its PhotoCraft catalog.
type Locale = (String, Option<std::path::PathBuf>, std::path::PathBuf);

/// This fork's own locales: `i18n/fork/<code>/photocraft.tsv`, with `common.tsv` beside it when it
/// has one. A whole catalog per locale, so the fork needs no checkout.
fn fork_locales() -> Vec<Locale> {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(FORK_CATALOGS);
    println!("cargo::rerun-if-changed={}", dir.display());
    let Ok(list) = std::fs::read_dir(&dir) else { return Vec::new() };
    let mut out = Vec::new();
    for entry in list.flatten() {
        let path = entry.path();
        let Some(code) = path.file_name().and_then(|n| n.to_str()).map(str::to_string) else { continue };
        let app = path.join(APP_CATALOG);
        if !app.is_file() {
            continue;
        }
        println!("cargo::rerun-if-changed={}", app.display());
        let shared = Some(path.join("common.tsv")).filter(|p| p.is_file());
        if let Some(p) = &shared {
            println!("cargo::rerun-if-changed={}", p.display());
        }
        out.push((code, shared, app));
    }
    out.sort();
    out
}

/// Every locale under `dir` with PhotoCraft's catalog, by code. An error when `dir` isn't a
/// craft-i18n `locales/` directory.
fn locales(dir: &std::path::Path) -> Result<Vec<Locale>, String> {
    // Build scripts run in the crate's directory and `include_str!` resolves against the
    // generated file, so a relative path would mean two different places.
    if !dir.is_absolute() {
        return Err(format!("{} is not an absolute path", dir.display()));
    }
    // Before read_dir, so a checkout that appears later reruns the script.
    println!("cargo::rerun-if-changed={}", dir.display());
    let list = std::fs::read_dir(dir).map_err(|e| format!("{} is not a craft-i18n checkout ({e})", dir.display()))?;
    let mut out = Vec::new();
    for entry in list.flatten() {
        let path = entry.path();
        let Some(code) = path.file_name().and_then(|n| n.to_str()).map(str::to_string) else { continue };
        let app = path.join(APP_CATALOG);
        if !app.is_file() {
            continue;
        }
        println!("cargo::rerun-if-changed={}", app.display());
        let shared = Some(path.join("common.tsv")).filter(|p| p.is_file());
        if let Some(p) = &shared {
            println!("cargo::rerun-if-changed={}", p.display());
        }
        out.push((code, shared, app));
    }
    if out.is_empty() {
        return Err(format!("{} has no locale with a {APP_CATALOG}", dir.display()));
    }
    out.sort();
    Ok(out)
}

fn read<T>(path: &std::path::Path, fallback: &str, parse: impl Fn(&str) -> Result<T, String>) -> Option<T> {
    println!("cargo::rerun-if-changed={}", path.display());
    let r = std::fs::read_to_string(path).map_err(|e| e.to_string()).and_then(|t| parse(&t));
    if let Err(e) = &r {
        println!("cargo::warning={}: {e}; {fallback}", path.display());
    }
    r.ok()
}

/// A non-empty string field of someone's people.toml entry, as a Rust `Option<&str>` literal.
fn name(v: Option<&str>) -> String {
    match v.map(str::trim).filter(|s| !s.is_empty()) {
        Some(x) => format!("Some({x:?})"),
        None => "None".into(),
    }
}

fn s(v: &Value, k: &str) -> String {
    format!("{:?}", v.get(k).and_then(Value::as_str).unwrap_or(""))
}

fn n(v: &Value, k: &str) -> u64 {
    v.get(k).and_then(Value::as_u64).unwrap_or(0)
}

fn rows(json: &Value, k: &str) -> Vec<Value> {
    json.get(k).and_then(Value::as_array).cloned().unwrap_or_default()
}

fn generate(json: &Value, people: &toml::Table) -> String {
    let mut o = String::new();
    let _ = writeln!(o, "pub static TOTAL_COMMITS: u64 = {};", n(json, "commits"));
    let _ = writeln!(o, "pub static CONTRIBUTORS: &[Contributor] = &[");
    for c in rows(json, "contributors") {
        let login = c.get("login").and_then(Value::as_str).unwrap_or("");
        if login.is_empty() {
            continue;
        }
        // Names only from a verified, self-chosen people.toml entry.
        let entry = people.get("people").and_then(|p| p.get(login)).filter(|_| c.get("people_entry").and_then(Value::as_bool) == Some(true));
        let field = |k: &str| entry.and_then(|e| e.get(k)).and_then(toml::Value::as_str);
        let synced = entry.and_then(|e| e.get("sync_github_name")).and_then(toml::Value::as_bool) == Some(true);
        let display =
            field("display_name").filter(|s| !s.trim().is_empty()).or_else(|| if synced { c.get("github_name").and_then(Value::as_str) } else { None });
        let _ = writeln!(
            o,
            "    Contributor {{ login: {login:?}, display_name: {}, real_name: {}, prs: {}, commits: {}, lines_added: {}, lines_deleted: {}, binary_added: {}, binary_deleted: {}, first_commit: {}, last_commit: {} }},",
            name(display),
            name(field("real_name")),
            n(&c, "prs"),
            n(&c, "commits"),
            n(&c, "lines_added"),
            n(&c, "lines_deleted"),
            n(&c, "binary_added"),
            n(&c, "binary_deleted"),
            s(&c, "first_commit"),
            s(&c, "last_commit"),
        );
    }
    let _ = writeln!(o, "];");
    let _ = writeln!(o, "pub static MODELS: &[Model] = &[");
    for m in rows(json, "models") {
        let _ = writeln!(
            o,
            "    Model {{ company: {}, model: {}, version: {}, commits: {}, lines_added: {}, lines_deleted: {} }},",
            s(&m, "company"),
            s(&m, "model"),
            s(&m, "version"),
            n(&m, "commits"),
            n(&m, "lines_added"),
            n(&m, "lines_deleted"),
        );
    }
    let _ = writeln!(o, "];");
    o
}
```

**What changed, part by part**

1. `FORK_CATALOGS` and `out_dir()` are new: the fork's own directory, and `$OUT_DIR` in one place
   (`main()` used to build it inline).
2. `craft_i18n()` starts from the checkout's entries as before, then walks `fork_locales()` and
   **replaces** the entry with the same code or appends a new one. Replacement is what makes
   precedence explicit; `entry.1`/`entry.2` are the shared and app catalog paths.
3. The `CRAFT_I18N_REQUIRED=1` error now fires only when **no** catalog exists at all — a fork
   catalog satisfies it — and its message names both ways to provide one.
4. `embed()` is new: a path outside `OUT_DIR` (a checkout, or the fork directory) is **copied** into
   `OUT_DIR` first and the generated code references the copy. Two reasons, both in its comment: the
   copy keeps the rows still if the checkout later moves or is deleted between builds, and no path
   outside `OUT_DIR` ends up in generated code. A file already under `OUT_DIR` is used as it is.
5. `fork_locales()` is new: each `i18n/fork/<code>/photocraft.tsv` with `common.tsv` beside it when
   present, sorted by code. `rerun-if-changed` is emitted for the directory and every file, so an
   edited catalog rebuilds the crate.
6. Three comments were restored after the first commit (`8b15cb1087`) because they are the *reason*
   for guards that otherwise look arbitrary: why the checkout path is not canonicalized (Windows gives
   `\\?\C:\…`, which `include_str!` cannot read), why it must be absolute (build scripts run in the
   crate directory while `include_str!` resolves against the generated file), and why
   `rerun-if-changed` comes before `read_dir` (a checkout created later must still rerun the script).

### 4.2 `crates/ui-egui/src/i18n/tests.rs` (the added test)

```rust
/// The fork's own Japanese catalog settles the `変形` submenu the way the Japanese interface does:
/// `Skew` is ゆがみ and the Distort command is 多方向に伸縮, while the bare `Distort` the Filter menu's
/// category reads keeps ゆがみ.
#[test]
fn the_fork_settles_the_transform_submenu_in_japanese() {
    // This fork ships its own Japanese catalog, so the rows hold without a checkout and
    // without `CRAFT_I18N_REQUIRED`; only an English-only build (no catalogs) skips.
    let Some(ja) = Lang::from_code("ja") else { return };
    assert_eq!(tr(ja, "Skew"), "ゆがみ");
    assert_eq!(tr_id(ja, "edit.transform.distort", "Distort"), "多方向に伸縮");
    assert_eq!(tr(ja, "Distort"), "ゆがみ", "the Filter menu's category keeps its word");
}
```

The three assertions are the three mechanisms at once: a plain row, an id-keyed row, and the plain
row that must **not** change. It deliberately does not call `needs_catalogs()`, unlike its neighbours:
that helper skips the test whenever `CRAFT_I18N_REQUIRED` is unset, which is exactly the state a
fork-only build is in.

### 4.3 The catalog rows (`TSV`, tab-separated; `\t` written out)

`crates/ui-egui/i18n/fork/ja/photocraft.tsv` — four rows changed, all others identical to craft-i18n
at `e56d2624`:

| Line | Row (before) | Row (after) |
|---|---|---|
| 2705 | *(absent)* | `@id\tedit.transform.distort\t多方向に伸縮` |
| 2729 | `\tSet horizontal skew\t水平方向の斜体を設定` | `\tSet horizontal skew\t水平方向のゆがみを設定` |
| 2746 | `\tSet vertical skew\t垂直方向の斜体を設定` | `\tSet vertical skew\t垂直方向のゆがみを設定` |
| 2854 | `\tSkew\t斜体` | `\tSkew\tゆがみ` |

Left alone on purpose:

```tsv
# crates/ui-egui/i18n/fork/ja/photocraft.tsv:2855 — a different key, a different word
	Skewness	歪度

# crates/ui-egui/i18n/fork/ja/common.tsv:344 — the word the Filter > Distort submenu title reads
	Distort	ゆがみ
```

The leading empty field is the *context* column: the row applies to the string in every context. No
`@id` row existed for `edit.transform.distort` before; it was added beside the other `@id` block
(after `@id\tselect.all`), not inside a context group.

### 4.4 `docs/localization.md`

Corrected sentence (was: "… without it PhotoCraft builds and runs in English only."):

```markdown
entries fall back to English. The build embeds them from the checkout `CRAFT_I18N_DIR` names, at
the commit in `craft-i18n.rev`; without it PhotoCraft runs on the catalogs this repository
carries itself (below) and English.

## This fork's own catalogs

Japanese ships with the repository: `crates/ui-egui/i18n/fork/ja/photocraft.tsv` and its `common.tsv`,
a whole catalog each. `crates/ui-egui/build.rs` embeds them with or without `CRAFT_I18N_DIR`, and a
fork catalog replaces the checkout's for the same locale. A word this fork settles differently (the
`変形` submenu's `ゆがみ` and `多方向に伸縮`, say) is edited there, never in the checkout, so the change
travels with the fork and nobody needs write access to `storytold/craft-i18n`. Other languages still
come from `CRAFT_I18N_DIR` when it is set.
```

### 4.5 `ATTRIBUTION.md`

```markdown
| `crates/ui-egui/i18n/fork/<locale>/photocraft.tsv`, `common.tsv` | UI translations this fork carries itself (Japanese), so the build needs no external checkout | craft-i18n and PhotoCraft contributors; the Japanese catalog comes from [craft-i18n](https://github.com/storytold/craft-i18n) at the revision in [`craft-i18n.rev`](craft-i18n.rev), with this fork's own word choices | <https://github.com/storytold/craft-i18n> | MIT OR Apache-2.0; Copyright (c) 2026 ArtCraft Team and the craft-i18n contributors; notice in [NOTICE](NOTICE) |
```

The license is unchanged from the checkout it came from, which is what the shared catalog row already
says; the new row exists so the copied files are attributed in the change that added them.

---

## 5. What it means

### 5.1 Precedence, now stated in one place

A locale's catalogs come from the checkout, from the fork, or from both — and when both, **the fork
wins wholesale**: shared *and* app catalog are replaced, not merged row by row. That is deliberate. A
row-by-row merge would make an English-source key appear twice with no rule for which wins, and would
make "why does this word differ?" unanswerable without diffing two 175 KB files. Wholesale replacement
means the fork's Japanese catalog *is* the Japanese catalog; the question "where is this word decided"
has one answer per locale.

The cost is a re-sync duty, stated plainly in §5.6.

### 5.2 Why the command needed an id-keyed row

`tr_id` asks `own_id` first (`crates/ui-egui/src/i18n.rs:459-470`): an `@id` row counts only when the
language owns the id and the translation keeps the placeholders and the trailing `…` of the English
label. Only then does `tid()` win over `tr(lang, label)`. So:

- `Edit > Transform > Distort` → `tr_id(lang, "edit.transform.distort", "Distort")` → 多方向に伸縮;
- `Filter > Distort` (a submenu title) → `translate_titles` → `tr(lang, "Distort")` → ゆがみ;
- `Filter > Distort > Ripple…` → `tr_id(lang, "filter.distort.ripple", "Ripple…")` → unchanged.

`多方向に伸縮` carries no placeholder and no ellipsis, and `Distort` has neither, so `same_shape`
passes and the row is used; a `@id` row that dropped a `{placeholder}` would be ignored and the plain
row would show instead. That guard is why an id row cannot silently break a formatted label.

### 5.3 Why `Skew` could stay a plain row

`Skew` as a bare source string is used by one surface only — the transform command. The near-miss key
`Skewness` is its own row (`photocraft.tsv:2855`, 歪度) and is not touched, so changing the plain
`Skew` row cannot leak into a statistics label. This is the opposite of the `Distort` case, and the
asymmetry is not an inconsistency: it follows from which surfaces share a key.

### 5.4 Why the catalogs are embedded, not read at run time

The binary must work from a read-only install and from a single executable with no data directory
next to it, and the web build has no file system at all. `include_str!` keeps both working and costs
7,237,632 B — measured: 303,841,280 B with every locale embedded, 296,603,648 B with Japanese only. The
trade-off is that translations cannot be patched in the field; a wrong word
needs a build. That is unchanged by this work — what changed is *who* can fix the word: the fork, in
its own repository, with a normal commit.

### 5.5 What the build now produces, measured

With a checkout of craft-i18n at `e56d2624` in `CRAFT_I18N_DIR`, `$OUT_DIR/craft_i18n.rs` held **35**
locale entries. The Japanese entry pointed at the fork's copy, not the checkout's:

```
("ja", include_str!("…/out/ja-common.tsv"), include_str!("…/out/ja-photocraft.tsv"))
```

and those two copies were 33,194 B and **175,825 B** — the fork's `photocraft.tsv` size, while the
checkout's own file at that revision is 175,770 B. Precedence is therefore not just asserted by the
code but visible in the output.

Without `CRAFT_I18N_DIR`, the same generation produced a single `ja` entry from `i18n/fork/ja`, and the
crate compiled: the Japanese interface now exists on a machine that has never seen craft-i18n.

### 5.6 Maintenance: the re-sync duty

The fork's Japanese catalog is a **snapshot**. Rows added to craft-i18n after `e56d2624` will not
appear in this fork's Japanese interface until the catalog is re-synced. The procedure is small and
should be written down whenever the pin moves:

```sh
# 1. a checkout at the new revision
git clone --filter=blob:none https://github.com/storytold/craft-i18n <tmp>
git -C <tmp> checkout <new sha>
# 2. replace the fork's two files with that revision's, then re-apply the rows in §4.3
cp <tmp>/locales/ja/photocraft.tsv crates/ui-egui/i18n/fork/ja/photocraft.tsv
cp <tmp>/locales/ja/common.tsv    crates/ui-egui/i18n/fork/ja/common.tsv
# 3. the same four rows (three edits, one added @id row), then
cargo test -p photocraft-ui-egui --lib i18n
```

A test that fails when the fork's catalog is older than the pin would be the durable guard; it needs a
network fetch or the checkout at test time, so it is not part of this change. State it as a known gap,
not as solved.

### 5.7 Reproduction and verification actually run

Environment for all of it: Windows, `CRAFT_FONTS_DIR` set to a craft-fonts checkout, `CRAFT_I18N_DIR`
pointing at a scratch clone of craft-i18n at `e56d2624` (removed afterwards, never in the repository),
and the craft-libs local patch on (`cargo xtask craftlibs on`) because the pinned revision lacks the
fork's UI-library API.

```powershell
# the whole i18n suite with every locale embedded
$env:CRAFT_I18N_DIR = '<scratch craft-i18n checkout>'
$env:CRAFT_I18N_REQUIRED = '1'
cargo test -p photocraft-ui-egui --lib i18n
#   → 37 passed; 0 failed

# the same test with no checkout at all
Remove-Item Env:CRAFT_I18N_DIR
cargo test -p photocraft-ui-egui --lib i18n::tests::the_fork
#   → 1 passed

# the release binary
cargo build --release -p photocraft
#   → Finished `release` profile in 21m 41s
rg -a -c "多方向に伸縮"           target/release/photocraft.exe   # 1
rg -a -c "水平方向のゆがみを設定" target/release/photocraft.exe   # 1
rg -a -c "水平方向の斜体を設定"   target/release/photocraft.exe   # 0 (no match)
rg -a -c "	Skew	ゆがみ"        target/release/photocraft.exe   # 1
```

Binary produced: `target/release/photocraft.exe`, **303,841,280 B**, **2026-11-20 20:06:37**.

The first attempt at this build was a fork-only build (no checkout) and the test suite **failed 27 of
37**, every failure being `no photocraft.tsv at craft-i18n.rev` for a locale other than Japanese.
That is worth recording: dropping the checkout silently reduced the shipped binary from the 35
registered languages (`LANGUAGE_COUNT` at `crates/ui-egui/src/i18n.rs:108`) to English and Japanese,
and the tests caught it. The exe was rebuilt with all locales, then the fork catalog confirmed
to override only Japanese.

Not run, and why:

- **No visual check of the running application.** The labels above are verified as embedded bytes and
  through the translation functions, not by opening `Edit > Transform` and looking. A screenshot check
  (`cargo run -p photocraft-ui-egui --example snapshot`) is the obvious next evidence and was not done.
- **No clippy run** in this round; the change is a build script, a test, and data.
- **`cargo fmt --check -p photocraft-ui-egui` is clean for `build.rs`**, but the crate still reports
  three files unrelated to this change (`src/canvas/tools/move_tool.rs:272`,
  `src/panels/layer_transfer/tests.rs:420`, `src/panels/toolbar.rs:174`). They predate this work and
  were left alone.
- The exe was linked **before** the two comment-only commits (`8b15cb1087` and the header fix in this
  document's commit). Comments do not change code generation, but the binary is not byte-identical to
  a build of the final commit; the string checks above are the evidence that the code is the same.

### 5.8 Changing a word from now on

1. Edit the row in `crates/ui-egui/i18n/fork/ja/photocraft.tsv` (or `common.tsv` for a shared string).
2. If the word belongs to **one command** and the same English string also titles a menu or submenu,
   add `@id\t<command id>\t<translation>` instead of editing the plain row — the plain row is what the
   submenu title reads (§5.2).
3. `cargo test -p photocraft-ui-egui --lib i18n`, then rebuild. `rerun-if-changed` covers the
   directory and each file, so the crate rebuilds without a `cargo clean`.
4. Never edit a checkout to fix a word: it is not in the repository, and the next clone loses it.

---

## 6. Appendix — the fix as commits

| Commit | Contents |
|---|---|
| `68e7d33e92` | `i18n/fork/ja/{photocraft,common}.tsv` (new), `build.rs` fork catalogs + `embed()`, the test, `docs/localization.md`, `ATTRIBUTION.md` |
| `8b15cb1087` | The three guard comments in `build.rs` restored |
| this document's commit | `build.rs` header names the fork catalogs; `md/i18n-fork-japanese-catalog.md` added |

One earlier attempt at the same fix lived in a checkout outside the repository (`D:\craft-i18n`,
since deleted). It proved the wording was right and proved nothing about durability — which is exactly
the defect described in §1.3. The version in this document is the one that survives a fresh clone.
