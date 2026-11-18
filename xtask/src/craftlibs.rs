//! craft-libs one-tree work: the local patch, and the check that keeps it out of a commit.
//!
//! A fix in the shared UI framework is made in a `craft-libs` checkout beside this one and built here
//! without a pin bump. `.cargo/config.toml` holds a `[patch]` section that points every craft-libs
//! crate at that checkout, and `Cargo.lock` then records those crates as local paths instead of the
//! git revision — so neither file may reach a commit: a clone without the neighbouring checkout
//! fails to build, and a lock without the sources cannot be resolved elsewhere.
//!
//! `on` writes the section, generated from the checkout's own crates so a new crate is never left
//! out; `off` removes it and puts the lock back on the pin; `status` reports both sides; `check`
//! fails when either file still carries the local work, so a release or a pull request cannot ship
//! it. See `md/craft-libs-並行管理手順.md`.

use std::fs;
use std::path::{Path, PathBuf};

use crate::run;

/// The git URL whose crates the patch replaces. The `[patch]` key and the crates' own source agree.
const SOURCE: &str = "https://github.com/storytold/craft-libs";

pub fn run_craftlibs(root: &Path, args: &[&str]) -> Result<(), String> {
    match args.first().copied() {
        Some("status") | None => status(root),
        Some("on") => on(root),
        Some("off") => off(root),
        Some("check") => check(root),
        Some(other) => Err(format!("unknown craftlibs command `{other}` (status | on | off | check)")),
    }
}

/// The checkout beside this one, as the patch's paths are written relative to this workspace.
fn checkout(root: &Path) -> PathBuf {
    root.parent().map(|above| above.join("craft-libs")).unwrap_or_else(|| PathBuf::from("craft-libs"))
}

fn config_path(root: &Path) -> PathBuf {
    root.join(".cargo").join("config.toml")
}

/// The heading that opens the generated section.
fn heading() -> String {
    format!("[patch.\"{SOURCE}\"]")
}

/// The crates of `checkout` as (package name, path relative to this workspace), read from each
/// `crates/*/Cargo.toml` so a crate added upstream is patched without an edit here.
fn crates(checkout: &Path) -> Result<Vec<(String, String)>, String> {
    let dir = checkout.join("crates");
    let entries = fs::read_dir(&dir).map_err(|e| format!("read {}: {e}", dir.display()))?;
    let mut found = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| format!("read {}: {e}", dir.display()))?;
        let manifest = entry.path().join("Cargo.toml");
        if !manifest.is_file() {
            continue;
        }
        let text = fs::read_to_string(&manifest).map_err(|e| format!("read {}: {e}", manifest.display()))?;
        let Some(name) = package_name(&text) else { continue };
        let dir_name = entry.file_name().to_string_lossy().into_owned();
        found.push((name, format!("../craft-libs/crates/{dir_name}")));
    }
    found.sort();
    if found.is_empty() {
        return Err(format!("no crates in {}: is the checkout complete?", dir.display()));
    }
    Ok(found)
}

/// `name = "…"` of the `[package]` table, the first `name` a manifest carries.
fn package_name(manifest: &str) -> Option<String> {
    let mut in_package = false;
    for line in manifest.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_package = line == "[package]";
            continue;
        }
        if !in_package {
            continue;
        }
        let Some(rest) = line.strip_prefix("name") else { continue };
        let rest = rest.trim_start().strip_prefix('=')?.trim();
        return rest.strip_prefix('"')?.split('"').next().map(str::to_owned);
    }
    None
}

/// `config` with any generated section removed, so `on` writes one section and never two.
fn without_section(config: &str) -> String {
    let mut kept: Vec<&str> = Vec::new();
    let mut inside = false;
    for line in config.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            inside = trimmed == heading();
            if inside {
                continue;
            }
        }
        if !inside {
            kept.push(line);
        }
    }
    let mut text = kept.join("\n");
    while text.ends_with('\n') {
        text.pop();
    }
    if !text.is_empty() {
        text.push('\n');
    }
    text
}

fn on(root: &Path) -> Result<(), String> {
    let checkout = checkout(root);
    if !checkout.is_dir() {
        return Err(format!(
            "no checkout at {}: clone storytold/craft-libs there and check out the revision Cargo.toml pins",
            checkout.display()
        ));
    }
    let found = crates(&checkout)?;
    let config = config_path(root);
    let existing = fs::read_to_string(&config).unwrap_or_default();
    let mut text = without_section(&existing);
    text.push('\n');
    text.push_str("# Local craft-libs for one-tree development: the shared UI framework, the key and focus routing\n");
    text.push_str("# and AccessKit live there, so a fix is made in the checkout beside this one and built here\n");
    text.push_str("# without a pin bump. The workspace Cargo.toml keeps its pinned revision, so this section and\n");
    text.push_str("# the lock it rewrites must not be committed. `cargo xtask craftlibs off` removes both.\n");
    text.push_str(&heading());
    text.push('\n');
    for (name, path) in &found {
        text.push_str(&format!("{name} = {{ path = \"{path}\" }}\n"));
    }
    if let Some(parent) = config.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("create {}: {e}", parent.display()))?;
    }
    fs::write(&config, text).map_err(|e| format!("write {}: {e}", config.display()))?;
    println!("patched {} craft-libs crates from {}", found.len(), checkout.display());
    println!("now: cargo tree -p photocraft-ui-egui -e normal --depth 0   (local paths mean it took)");
    Ok(())
}

fn off(root: &Path) -> Result<(), String> {
    let config = config_path(root);
    if let Ok(existing) = fs::read_to_string(&config) {
        fs::write(&config, without_section(&existing)).map_err(|e| format!("write {}: {e}", config.display()))?;
    }
    // The lock records what the patch resolved; the pin is what a commit carries.
    let mut cmd = std::process::Command::new("git");
    cmd.args(["checkout", "--", "Cargo.lock", ".cargo/config.toml"]).current_dir(root);
    run(cmd, "git checkout -- Cargo.lock .cargo/config.toml")?;
    println!("patch removed and Cargo.lock put back on the pin");
    Ok(())
}

/// Fails when `.cargo/config.toml` still patches or `Cargo.lock` records a craft-libs crate without
/// a `source`: either would land a build that only works on this machine.
fn check(root: &Path) -> Result<(), String> {
    let mut problems: Vec<String> = Vec::new();
    let config = config_path(root);
    if let Ok(text) = fs::read_to_string(&config)
        && text.contains(&heading())
    {
        problems.push(format!(
            "{} patches craft-libs ({}): `cargo xtask craftlibs off` removes it, and its lock change",
            config.display(),
            heading()
        ));
    }
    let lock = root.join("Cargo.lock");
    let text = fs::read_to_string(&lock).map_err(|e| format!("read {}: {e}", lock.display()))?;
    let locals = local_crates(&text);
    if !locals.is_empty() {
        problems.push(format!(
            "Cargo.lock records {} craft-libs crate(s) with no source, so a clone cannot resolve them: {}",
            locals.len(),
            locals.join(", ")
        ));
    }
    if problems.is_empty() {
        println!("craft-libs is on the pin: safe to commit");
        return Ok(());
    }
    let mut message = String::from("craft-libs is patched locally; do not commit this tree:\n");
    for problem in problems {
        message.push_str("  - ");
        message.push_str(&problem);
        message.push('\n');
    }
    Err(message)
}

/// The names of `[[package]]` tables that have no `source`, i.e. that came from a path, limited to
/// the framework's own crates (`craft` and `craft-*`).
fn local_crates(lock: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    let mut name: Option<String> = None;
    let mut sourced = false;
    let flush = |name: Option<String>, sourced: bool, found: &mut Vec<String>| {
        if let Some(name) = name {
            let ours = name == "craft" || name.starts_with("craft-");
            if ours && !sourced {
                found.push(name);
            }
        }
    };
    for line in lock.lines() {
        let trimmed = line.trim();
        if trimmed == "[[package]]" {
            flush(name.take(), sourced, &mut found);
            sourced = false;
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("name = ") {
            name = rest.trim().trim_matches('"').to_owned().into();
            continue;
        }
        if trimmed.starts_with("source = ") {
            sourced = true;
        }
    }
    flush(name.take(), sourced, &mut found);
    found
}

fn status(root: &Path) -> Result<(), String> {
    let config = config_path(root);
    let patched = fs::read_to_string(&config).is_ok_and(|text| text.contains(&heading()));
    let checkout = checkout(root);
    println!("workspace:   {}", root.display());
    println!("checkout:    {} ({})", checkout.display(), if checkout.is_dir() { "present" } else { "absent" });
    println!(".cargo/config.toml: {}", if patched { "PATCHED (local craft-libs)" } else { "on the pin" });
    let lock = root.join("Cargo.lock");
    match fs::read_to_string(&lock) {
        Ok(text) => {
            let locals = local_crates(&text);
            if !locals.is_empty() {
                println!("Cargo.lock:  {} craft-libs crate(s) from local paths (NOT safe to commit)", locals.len());
            } else if patched {
                // The section decides the lock: cargo rewrites it on the next run, so it is not a
                // commit's to carry even before that run happened.
                println!("Cargo.lock:  still on the pin, but the next cargo run rewrites it (NOT safe to commit)");
            } else {
                println!("Cargo.lock:  sources recorded (safe to commit)");
            }
        }
        Err(e) => println!("Cargo.lock:  unreadable: {e}"),
    }
    if let Ok(manifest) = fs::read_to_string(root.join("Cargo.toml"))
        && let Some(rev) = pinned_rev(&manifest)
    {
        println!("pinned rev:  {rev}");
    }
    if let Some(head) = checkout_head(&checkout) {
        println!("checkout at: {head}");
    }
    Ok(())
}

/// The first 40-character revision a manifest pins for the framework.
fn pinned_rev(manifest: &str) -> Option<String> {
    let at = manifest.find("storytold/craft-libs")?;
    let rest = &manifest[at..];
    let start = rest.find("rev = \"")? + "rev = \"".len();
    let hex: String = rest[start..].chars().take_while(|c| c.is_ascii_hexdigit()).collect();
    (hex.len() == 40).then_some(hex)
}

fn checkout_head(checkout: &Path) -> Option<String> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .current_dir(checkout)
        .output()
        .ok()?;
    output.status.success().then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The `name` a manifest carries is the package's, not a dependency's: the latter sits under
    /// `[dependencies]` and must not name the crate.
    #[test]
    fn package_name_reads_only_the_package_table() {
        let manifest = "[package]\nname = \"craft-ui-contract\"\nversion = \"0.1.0\"\n\n[dependencies]\nname = \"not-this\"\n";
        assert_eq!(package_name(manifest).as_deref(), Some("craft-ui-contract"));
    }

    #[test]
    fn package_name_is_none_without_a_package_table() {
        assert_eq!(package_name("[dependencies]\nname = \"x\"\n"), None);
    }

    /// A crate that came from the checkout has no `source`; one from the git revision has it. Only
    /// the framework's own crates count: a path dependency that is not craft-libs is fine.
    #[test]
    fn local_crates_lists_craft_crates_without_a_source() {
        let lock = "[[package]]\nname = \"craft\"\nversion = \"0.1.0\"\n\n\
[[package]]\nname = \"craft-ui-contract\"\nversion = \"0.1.0\"\n\
source = \"git+https://github.com/storytold/craft-libs?rev=abc#abc\"\n\n\
[[package]]\nname = \"photocraft-doc\"\nversion = \"0.1.0\"\n";
        assert_eq!(local_crates(lock), vec!["craft".to_owned()]);
    }

    #[test]
    fn local_crates_is_empty_when_every_craft_crate_has_a_source() {
        let lock = "[[package]]\nname = \"craft\"\nversion = \"0.1.0\"\n\
source = \"git+https://github.com/storytold/craft-libs?rev=abc#abc\"\n";
        assert!(local_crates(lock).is_empty());
    }

    /// Writing the section twice must not leave two: the check looks for the heading, and cargo
    /// rejects a repeated table.
    #[test]
    fn without_section_removes_the_generated_table_and_keeps_the_rest() {
        let config = "[alias]\nxtask = \"run -p xtask --\"\n\n[patch.\"https://github.com/storytold/craft-libs\"]\ncraft = { path = \"../craft-libs/crates/craft\" }\n";
        let kept = without_section(config);
        assert!(kept.contains("[alias]"), "{kept}");
        assert!(!kept.contains("[patch."), "{kept}");
    }

    #[test]
    fn without_section_leaves_a_config_that_has_none_alone() {
        let config = "[alias]\nxtask = \"run -p xtask --\"\n";
        assert_eq!(without_section(config), config);
    }
}
