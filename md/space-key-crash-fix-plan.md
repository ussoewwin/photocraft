# Space Key Crash & Menu Activation Resolution Report (craft-libs integration)

> **Repository:** `D:\USERFILES\GitHub\photocraft` (tracking `storytold/photocraft`)  
> **Associated Framework:** `D:\USERFILES\GitHub\craft-libs` (`next` branch, tag `v1.0.3`, PR #2239)  
> **Status:** Completely Resolved and Verified  

---

## 1. Overview of Resolved Regressions

Following the downstream migration of PhotoCraft's UI onto the `craft-libs` shared framework (#15449), two severe runtime regressions were identified and resolved across the two repositories:

### 1.1. AccessKit UIA Crash (Resolved in Upstream PR #2328)
- **Symptom**: Clicking top-level menus (Image, Edit, Layer, Filter) on Windows with UI Automation active crashed the process within six seconds.
- **Root Cause**: `accesskit_consumer-0.39.2\src\node.rs:999` unwrapped missing node IDs from transient `controls` relations when popup nodes were emitted across separate frame passes.
- **Resolution**: Implemented `drop_dangling_relations` in `crates/ui-contract/src/runtime/egui/access.rs` to prune uncarried node IDs. Upstream independently reviewed and merged this fix into official `next` via **PR #2328**.

### 1.2. Canvas Space-Key Hand Tool Hijacking & File Menu Activation (Resolved in Fork PR #2239)
- **Symptom**: Holding `Space` over the active canvas to pan intermittently activated the focused menu bar button (**File**), opening its dropdown and dispatching **New Document**.
- **Root Cause**:
  1. `crates/ui-contract/src/runtime/focus/policy.rs:350`: `KeyUse::of(Input::Pointer)` configured `ActivateKeys::Both` by default.
  2. `crates/ui-contract/src/runtime/keys/route.rs:542`: Unadorned Space was routed to `PressRoute::Activate` for any stop with Space activation. Menu bar buttons (`crates/ui-widgets/src/menu/bar.rs`) inherited Space activation and fired on Space.
  3. `crates/ui-contract/src/runtime/egui/response.rs`: egui's default context generated synthetic primary clicks (`FAKE_PRIMARY_CLICKED`) on Space.
- **Resolution**:
  1. `policy.rs`: Changed default to `ActivateKeys::Enter` across all pointer controls.
  2. `route.rs`: Rerouted unopted Space to `PressRoute::Free`, keeping Space exclusively available for canvas Hand tool navigation.
  3. `response.rs`: Passed `activate_on_space: false` in `InteractOptions` to suppress egui synthetic clicks.
  4. `flyout.rs` & `button.rs`: Added explicit `.with_space_activation()` to `Button` and `FlyoutButton` so intentional keyboard button activation functions cleanly.

---

## 2. Fork Feature Preservation Protocol (Crucial Rules)

During architectural refactoring and upstream harmonization, the following rules must strictly be followed:

1. **Local Framework Patching**:
   PhotoCraft must be linked to the local `craft-libs` fork checkout via:
   ```powershell
   cd D:\USERFILES\GitHub\photocraft
   cargo xtask craftlibs on
   ```
   If unpatched, PhotoCraft builds against the upstream pinned rev in `Cargo.lock`, which lacks the fork's Space-key isolation and CJK font classification.
2. **Never Strip `with_space_activation()` on `FlyoutButton`**:
   In `crates/ui-panels/src/rails/flyout.rs`:
   ```rust
   impl<T: Value + 'static> Component for FlyoutButton<'_, T> {
       const ROLE: Role = Role::BUTTON;
       /// Space runs the face's command as a tap does: the focus has no held press, so it opens no flyout.
       const KEYS: KeyUse = KeyUse::of(Input::Pointer).with_space_activation();
   ```
   This associated constant is mandatory. When retiring obsolete closure hooks (e.g., `RailLabels`), `KEYS` must remain intact; removing it breaks keyboard button trials and destabilizes key stage routing.
3. **Upstream Title Bar Harmonization**:
   Upstream's `TitleBar::show_titled` layout pipeline naturally distributes title widths across the bar. Custom width negotiation hooks (`leading_width`, `own_width`, `wanted_width`) were retired in v1.0.3, maintaining clean upstream alignment while ensuring all 10 menu items render unclipped side-by-side.

---

## 3. Verification Matrix

| Component | Target | Status |
|---|---|---|
| AccessKit UIA Menu Abort | `photocraft#15597` | **Fixed & Merged Upstream (#2328)** |
| Canvas Space Hand Tool Hijacking | `craft-libs` PR #2239 | **Fixed & Verified in PhotoCraft** |
| Button Keyboard Space Activation | `craft-ui-panels` | **Fixed & Verified (222 passed)** |
| CJK Font Heuristics & Localized Names | `craft-font-manager` | **Fixed & Verified (60 passed)** |
| PSD Layr / Mt16 Transparency | `craft-psd` | **Fixed & Verified (189 passed)** |
| Large ZIP EOCD64 Chunked Scanning | `craft-zip` | **Fixed & Verified (45 passed)** |
