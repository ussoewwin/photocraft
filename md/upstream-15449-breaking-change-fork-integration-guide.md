# PhotoCraft #15449 Breaking Refactor: Fork Feature Preservation & Integration Technical Guide

## Overview

This document provides the authoritative technical reference and architectural specification detailing how our fork preserved, refactored, and integrated its four core extensions against the largest breaking architectural rewrite in the upstream repository (`storytold/photocraft`): **PR #15449 ("PhotoCraft on the craft UI framework")** (1,529 files modified, +108,256 lines added, −283,197 lines removed).

Rather than resorting to fragile monkey patches or temporary workarounds, every fork capability was fundamentally re-architected to conform to the modern `craft-libs` UI framework, Tool Rails system, and multi-viewport paradigm with zero feature regression.

---

## 1. Upstream #15449 Breaking Changes Analysis

### 1.1 Scope and Architectural Transition

PR #15449 completely dismantled and reconstructed the UI tier (`crates/ui-egui`), migrating from legacy monolithic egui/eframe direct state mutation to the shared `craft-libs` component ecosystem (`craft-ui`, `craft-ui-shell`, `craft-ui-canvas`, `craft-ui-panels`, `craft-ui-widgets`, `craft-color-ui`, `craft-font-manager`, `craft-layers`, `craft-timeline`, `craft-draw`, `craft-imaging`, `craft-text`, etc.).

```text
[Legacy Architecture (pre-#15449)]
  crates/ui-egui (Monolithic direct egui/eframe control)
    ├── canvas/ (Ad-hoc per-tool event processing)
    ├── panels/ (Single-window assumption throughout)
    ├── tools/  (Direct mutations on global app state)
    └── i18n/   (In-repo TSV catalog files)

[Modern Architecture (post-#15449)]
  crates/ui-egui (Thin shell application layer)
    ├── craft-ui-canvas  (Abstracted canvas & input pipeline)
    ├── craft-ui-shell   (Multi-window / multi-viewport mediation)
    ├── craft-ui-panels  (Docking and rail-based panel hierarchy)
    ├── craft-i18n       (External repository sync & tl! macro dynamic catalog)
    └── tools/           (Tool Rails slots and Options Bar contracts)
```

### 1.2 Direct Breakage on Fork Capabilities

| Domain | Legacy Implementation (pre-#15449) | Modern Architecture (post-#15449) | Direct Impact on Fork Features |
|---|---|---|---|
| **Tool Registration & Cycling** | Flat `tools.rs` enum with static arrays | Tool Rails / Toolbar Slots / Options Bar separation | Lost tool slot registrations and Shift+W tool cycling order for SAM 3.1 and Quick Selection |
| **Multi-Window Support** | Implicit single-canvas assumption | `extra_windows` with distinct OS viewports | Move tool cross-document transfers suffered from gesture interception and premature aborts |
| **Event Consumption Pipeline** | Direct mouse event handling in tool loops | Decoupled canvas input handlers and panels | Broke Move tool selection marquee hit-testing and locked-layer drag detection |
| **Internationalization (i18n)** | Direct access to `crates/ui-egui/src/i18n/ja.tsv` | External `craft-i18n` catalogs + `tl!` lookup | Broke localized CJK font name display and in-repo translation override tables |

---

## 2. Fork Feature Preservation and Architectural Design

This section details how the four core fork feature areas were re-architected to achieve seamless compatibility with the modern infrastructure.

### 2.1 Feature 1: SAM 3.1 & LaMa Cleaner AI Tools Integration

#### Specification
- **SAM 3.1 (Segment Anything Model 3.1)**: Precision automated object segmentation via canvas point-and-click or bounding-box prompts.
- **LaMa Cleaner**: One-click object removal and inpainting.
- **Resident Worker Protocol**: Low-latency JSON IPC over stdout/stdin with resident Python/native workers, encapsulated within `photocraft-engine`.

#### Post-#15449 Re-Architecture
1. **Tool Rails Conformance**:
   - Re-implemented tool event handlers under `crates/ui-egui/src/canvas/tools/` satisfying the modern `Tool` trait.
   - Registered SAM 3.1 as the 4th tool in the W-key slot group (`ObjectSelection -> QuickSelection -> MagicWand -> Sam3Select`).
   - Verified Shift+W 4-tool cycle order via `tool_shift_cycle_order_tests`.
2. **Four-Mode Options Bar Implementation**:
   - Built a custom options bar conforming to `craft-ui-widgets`.
   - Implemented four discrete selection modes: `New`, `Add`, `Subtract`, and `Intersect`, synchronized with keyboard modifiers (Shift and Alt).

### 2.2 Feature 2: Quick Selection Mode Buttons Independence (PR #11837)

#### Specification
- Quick Selection tool provides dedicated `New`, `Add`, and `Subtract` mode buttons on its options bar.
- The tool's mode state remains completely independent from the document-global selection mode and other selection tools.

#### Post-#15449 Re-Architecture
1. **State Isolation**:
   - Encapsulated `quick_selection_mode` strictly within `RetouchUiState`.
   - Prevented cross-contamination between document-level selection modes and tool-specific modes during options bar rendering.
2. **Bidirectional Modifier Key Synchronization**:
   - Reconstructed real-time state synchronization between brush stroke modifier keys (Alt for temporary subtract, Shift for temporary add) and toolbar mode button active states.

### 2.3 Feature 3: Localized Japanese Font Name Display (PR #12452)

#### Specification
- The font picker displays Japanese font names (e.g., "游ゴシック", "メイリオ", "ヒラギノ角ゴシック") in localized Japanese script for system and bundled fonts (`craft-fonts`), regardless of the user's primary interface language preference.

#### Post-#15449 Re-Architecture
1. **Multilingual Font Resolution Pipeline**:
   - Maintained OpenType / TrueType Name Table traversal prioritizing `LanguageID == 0x0411` (Japanese) in `crates/ui-egui/src/tools/type_tool/fonts.rs`.
2. **Coordination with `craft-font-manager`**:
   - Preserved upstream font cache keys while decoupling internal font identifiers from UI display labels, ensuring display strings use localized family names without breaking font substitution mechanics.

### 2.4 Feature 4: Move Tool Cross-Document Selection Drag Transfer (PR #11971)

#### Specification
- Using the Move tool (V) to drag a selection marquee directly across document tabs, across arranged/tiled view cells, or between distinct OS document windows transfers and copies the selected pixels into the target document.
- The source document's pixels, layer structure, and undo/redo history remain 100% intact.

#### Post-#15449 Re-Architecture & Deep Repair
PR #11971 suffered the most severe disruption from upstream's multi-window refactoring. Four critical architectural gaps were systematically identified and resolved:

```text
[Move Tool Cross-Document Transfer Sequence]
User Mouse Press (Window A: Canvas)
       │
       ▼
 [tools/move_box.rs]
   app.move_viewport = app.current_viewport (Record gesture owner window A)
   app.layer_box = Some(MarqueeState)
       │
       ▼ Dragging across windows
 [canvas/document_windows.rs (Window B)]
   drag_to(pos, current_viewport)
   ※ Guarded by `owner != here` check to prevent Window B from destroying Window A's gesture
       │
       ▼ Drop on Window B Canvas / Tab
 [panels/layer_transfer.rs]
   finish(app, ctx)
   - Confirm (!primary_down || primary_released)
   - Dispatch engine transfer command (cmd/layer_copy_cmds)
   - Source history preserved; copy placed above destination active layer
       │
       ▼ Window Close / Cleanup
   When extra_windows closes, only release move_viewport if owned by that window
```

1. **Multi-Window Ownership Guards (`move_box.rs`)**:
   - `begin()` records the originating window ID (`app.current_viewport`) in `app.move_viewport`.
   - `drag_to()` and `release()` check `owner != here` to prevent extraneous viewports from prematurely mutating or canceling active transfer gestures.
   - `release()` and `cancel()` cleanly reset `app.move_viewport`.
2. **Secondary Document Window Lifecycle Hooks (`document_windows.rs`)**:
   - Added `crate::layer_transfer::finish(app, ui.ctx());` at the conclusion of `extra_windows` processing to finalize drops onto detached windows.
   - On window destruction, selectively clear `app.layer_box` and `move_blocked` only if that specific window owned the gesture.
3. **Marquee Lifecycle Preservation (`layer_transfer.rs`)**:
   - Fixed premature marquee destruction on secondary mouse button clicks or non-primary events. `layer_box` is only cleared when `(!primary_down || primary_released)` and `b.grabbed_layer.is_some()`.
4. **Locked-Layer Selection Transfer Fallback (`arm_move_transfer`)**:
   - Restored `.or_else(|| active.filter(...))` fallback resolution when dragging a selection marquee from a locked background layer, preventing lock-dialog blockage and allowing pixel transfer to proceed smoothly.

---

## 3. Verification & Quality Assurance Matrix

### 3.1 Test Suite Verification Results

| Test Category | Target Module | Test Count | Result | Key Invariants Verified |
|---|---|---|---|---|
| **Layer Transfer & Move Box** | `photocraft-ui-egui::panels::layer_transfer`<br>`photocraft-ui-egui::tools::move_box` | **39 tests** | **39 passed; 0 failed** | Cross-tab transfer, arranged view transfer, multi-window isolation, locked layer fallback, Shift-centering |
| **Engine Layer Copy Commands** | `photocraft-engine::cmd::layer_copy_cmds` | **36 tests** | **36 passed; 0 failed** | Inter-document pixel extraction, link group preservation, ICC profile conversion, paragraph style scaling |
| **AI Tools & Options Bars** | `sam3_toolbar`, `sam3_options_bar`<br>`lama_cleaner`, `quick_selection` | **15 tests** | **15 passed; 0 failed** | W-key slot order, 4-mode options bar, independent 3-mode states, IPC resident protocol |
| **Japanese Font Display** | `tools::type_tool::font_picker_tests` | **1 test** | **1 passed; 0 failed** | Localized Japanese font names correctly listed under English locale preference |
| **Architecture Layering** | `cargo xtask layers` | **26 crates** | **0 violations** | Strict L0 through L6 layering enforcement with zero circular dependencies |
| **Code Formatting** | `cargo fmt --all -- --check` | Workspace-wide | **Exit 0** | Full compliance with official Rust formatting standards |

### 3.2 Release Binary Build Verification

Compiled under Visual Studio 2022 MSVC toolset with full catalog embedding (`CRAFT_I18N_REQUIRED=1`), external catalog synchronization (`CRAFT_I18N_DIR`), and bundled CJK font packs (`CRAFT_FONTS_DIR`):

| Deliverable Artifact | Measured Size | Timestamp | Version | Verification Details |
|---|---|---|---|---|
| **`target\release\photocraft.exe`** | **346,361,344 B** (~330 MB) | 2026/11/18 18:42:36 | `FileVersion`: **0.23.1** | Verified binary strings: "クイック選択ツール", "長方形選択ツール", "オブジェクト選択ツール" all return `True` |
| **`target\release\photocraft-cli.exe`** | **132,097,536 B** (~126 MB) | 2026/11/18 18:17:10 | `0.23.1 (dev build)` | `--version` smoke test passed; headless batch/CLI capability verified |

- Build Execution Time: 51m 11s (Full release optimization across all 26 crates, LTO, CJK font embeds, and catalog packaging).

---

## 4. Upstream Synchronization & Maintenance Discipline (Two-Track Principle)

To continuously track upstream advancements while maintaining our fork's capabilities in perpetuity, the following principles are strictly enforced:

1. **Two-Track Synchronization Rule**:
   - `main` serves as the sole Integration Branch containing 100% of all pull request features and upstream updates.
   - Individual PR branches (`feat/localized-japanese-fonts`, `feat/quick-selection-mode-buttons`, `feat/move-tool-selection-drag-transfer`) must never absorb `main`. They merge only official `upstream/main` to preserve isolated, pristine pull request diffs.
2. **Worktree Isolation**:
   - All migration, repair, and feature testing operations are executed within isolated worktrees (`target/wt-*`), preventing build cache contamination and intermediate artifact leakage.
3. **Mandatory Catalog & Font Environment Variables**:
   - Builds and test runs must always set `CRAFT_I18N_DIR`, `CRAFT_I18N_REQUIRED=1`, and `CRAFT_FONTS_DIR` to physically prohibit the creation of degraded, English-only executables or missing font assets.

---
PhotoCraft Fork Maintenance Team
