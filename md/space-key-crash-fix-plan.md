# Space key crash / unexpected menu: fix plan (craft-libs local checkout)

## Symptoms
- The process occasionally dies when Space is pressed.
- Space is the momentary Hand tool (grab and drag the canvas), but sometimes opens a menu instead.
- Behaviour is intermittent.
- Not seen before the UI moved onto craft-libs (#15449). Seen on the latest main.

## Background
- #15449 moved PhotoCraft's windows, menus, focus, keyboard and pointer routing onto the framework in `storytold/craft-libs`. About 283,000 lines of PhotoCraft's own egui UI code were removed.
- The crash and (probably) the Space routing are in craft-libs code. PhotoCraft pins craft-libs at rev `a6b9d9c986f7c4c399632d72d9a4a8c8ea0b86e9`, so the fix cannot be made or tested in this repository alone.
- Upstream tracking:
  - #15597: Windows crash on Image/Edit/Layer menu, same `node.rs:999` panic, first bad commit `ed14e1c41` (#15449), no PR yet.
  - #15476 (EPIC-UI-REWORK): the upstream flow is a craft-libs PR, then a pin bump here.
  - Old PRs such as #8679 (menu bar Space fix) predate the port. They do not apply to the new code.

## Dev environment (prerequisite)
1. Clone craft-libs next to PhotoCraft: `D:\USERFILES\GitHub\craft-libs`.
2. Check out the pinned rev `a6b9d9c986f7c4c399632d72d9a4a8c8ea0b86e9` first, not `next`. This keeps the reproduction on the same code the user runs.
3. Point PhotoCraft at the local checkout with a `[patch."https://github.com/storytold/craft-libs"]` section in `.cargo/config.toml`, so `Cargo.toml` stays untouched. Every craft-libs package that PhotoCraft uses must be patched to the same checkout; a partial patch builds a second copy of the framework. Use craft-libs's `tools/cl` and the settings in this workspace's `[workspace.metadata.cl]` (see craft-libs `docs/development.md`, "App workspaces").
4. Confirm `cargo run -p photocraft` builds and runs against the local checkout before changing anything.
5. Do not commit the `.cargo/config.toml` patch or the local path to PhotoCraft.

## Confirmed (from `photocraft.log`)
```text
photocraft::crash_guard: panicked at accesskit_consumer-0.39.2\src\node.rs:999:85:
called `Option::unwrap()` on a `None` value
```
`NodeRef::controls()` calls `state.node_by_id(id).unwrap()` for every ID in a node's `controls` list. A `controls` entry that points to a node absent from the tree panics. The same panic is reported upstream in #15597.

## Status of claims
| Claim | Status | How to confirm |
|---|---|---|
| Panic at `accesskit_consumer node.rs:999` (`controls().unwrap()`) | **Confirmed** (log, and #15597) | n/a |
| `ui-contract/src/runtime/access/emit.rs:258-260` adds open popup IDs to `relations.controls` without checking the popup has a node in the same update | Inferred (code reading, matches the #15597 report) | Step A1 test |
| A popup node is the missing one | Inferred | Step A1: log the missing ID and its role when the check fails |
| Space triggers the panic | Not verified | Reproduce in the local checkout, with a UI Automation client (Narrator, Inspect or Accessibility Insights) connected |
| UI Automation timing causes the intermittency | Not verified | Compare crash rates with and without a UI Automation client |
| A focused widget takes Space as Activate and opens the menu | Not verified | Step B1: log the focused widget ID when the menu opens |
| Space handling was lost with the removed PhotoCraft UI code | Not verified | Step B1: compare the key routing in craft-libs with the pre-#15449 code (`01b79958a`, the last good commit) |
| Another path opens the menu (e.g. the Hand context-menu mapping at `canvas_tool_menu.rs:1746`) | Not verified | Step B1 logging |

Do not state an inferred or unverified row as fact until its confirmation step has run. If a step disproves a claim, revise this plan before fixing.

## Plan

### A. Crash (fix in craft-libs)
1. Write a regression test in craft-libs that opens and closes popups, then asserts every ID in every relation list (`controls` and the others) exists in the emitted node set. Confirm it fails on the pinned rev.
2. In `emit.rs`, build the set of emitted node IDs and keep only `controls` entries in that set. Apply the same filter to the other relation lists if they have the same pattern.
3. In `egui/access/node.rs`, add a second guard before `set_controls`.
4. Check upstream before pushing: search craft-libs open PRs and #15597 for a claim, and comment on the issue first (house rules 2 and 4). Another agent may already be editing the same files, so check its changes first.
5. Open the craft-libs PR (`photocraft#15597`, `Part of #15476`), then bump the pinned rev here once it is merged. Do not push to `main`.

### B. Menu opens on Space (fix in craft-libs or PhotoCraft, decided by the logging)
1. Log the focused widget and the key routing path when the menu opens unexpectedly, to confirm the cause.
2. Fix where the logging points. Options: consume Space for the Hand tool before widgets see it while the canvas is the active target, or clear widget focus after canvas interaction.
3. Test: with a button or menu focused, pressing Space activates Hand and does not open a menu.
4. Check upstream for an existing item first; file a `BUG-UI-REWORK-...` issue with "Part of #15476" if none exists.

### C. Verification
- craft-libs: its own tests through `tools/cl`.
- PhotoCraft: `cargo test -p <crates>`, `cargo clippy -p <crates> --all-targets -- -D warnings`, `cargo xtask layers`, `cargo xtask wasm`.
- Manual: press Space 30 or more times, with and without a UI Automation client. `photocraft.log` must show no `accesskit` panic.
- Report only what ran.
