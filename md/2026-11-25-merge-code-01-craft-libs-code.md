# craft-libs: the fork's code in the 2026-11-25 merge, explained

Scope: the six files where the fork's content survived the merge of `upstream/next` at `9f2961402`
(merge `65902c049`), and the nine files the fork repaired afterwards, up to `87ff4927bf`. Every diff
below is `git diff` output taken from the repository; every explanation names what the line does.

Read together with document 00 (method, merge heads, the isolation test).

## A. Merge resolutions the fork keeps

Each of these was a conflict because upstream changed the same lines. Measured at `HEAD`:
`git diff upstream/next..HEAD -- <path>` still reports 30, 25, 69, 86 and 125 lines respectively, so
none of this is upstream's code arriving - it is the fork's own.

### A.1 `.github/workflows/workspace.yml`

**Role.** The workspace CI workflow: which pushes run it, and the job that trims old caches.

```diff
@@ -2,7 +2,7 @@ name: Workspace
 
 on:
   push:
-    branches: [main, next]
+    branches: [main]
     tags: ["v*"]
   pull_request:
   workflow_dispatch:
@@ -225,7 +225,15 @@ jobs:
             entry=${BASH_REMATCH[1]}
             if [ -n "${kept[$entry]:-}" ]; then
               echo "delete $key"
-              gh cache delete "$id" --repo "$REPO"
+              # A scheduled run and a push prune in different concurrency groups, so both can list the
+              # same cache and one can delete it first: a missing cache is success here, while any other
+              # API error still fails the job.
+              if ! gh cache delete "$id" --repo "$REPO" 2>"$RUNNER_TEMP/cache-error"; then
+                if ! grep -Eqi 'not found|HTTP 404' "$RUNNER_TEMP/cache-error"; then
+                  cat "$RUNNER_TEMP/cache-error"
+                  exit 1
+                fi
+              fi
             else
               kept[$entry]=1
               echo "keep $key"
```

**What each hunk does.**

1. `branches: [main]` where upstream lists `[main, next]`: the fork's `next` is its integration branch, and its pushes are covered by the local runs this project uses, so the workflow does not run twice per change.
2. The cache-trim step tolerates a cache that is already gone. A scheduled run and a push prune in different concurrency groups, so both can list the same cache; whichever deletes second used to fail the job.
3. The `if ! ... 2>"$RUNNER_TEMP/cache-error"` captures the failure, and the nested `grep -Eqi 'not found|HTTP 404'` accepts exactly the already-gone case.
4. Any other error is printed with `cat` and exits 1, so a real API failure still fails the job - the tolerance is not a blanket ignore.

**If reverted.** A red CI job on every second prune, and a duplicate run per push to the fork's integration branch.

**How it is checked.** The workflow runs on GitHub only; the change is verified by reading, and the repository's rule keeps workflow edits in their own change.

### A.2 `crates/font-manager/src/picker.rs`

**Role.** The font picker's model: how a family is classified, and how its name is shown in the current language.

```diff
@@ -142,6 +142,11 @@ impl FontClass {
     pub fn of(catalog: &Catalog, family: &str) -> Option<FontClass> {
         rank::class_of(catalog, family)
     }
+
+    /// Classify a font family from its name, `sFamilyClass`, PANOSE, and fixed-pitch flag.
+    pub fn classify(family: &str, family_class: i16, panose: [u8; 10], fixed_pitch: bool) -> Option<FontClass> {
+        rank::classify(family, family_class, panose, fixed_pitch)
+    }
 }
 
 /// Photoshop's Font Preview Size (Type > Font Preview Size): how large the sample in each row is,
@@ -556,7 +561,7 @@ struct FamilyCodec {
 
 impl FamilyCodec {
     fn shown(&self, family: &str, info: &Info) -> String {
-        shown_name(info, family, &self.language, false)
+        shown_name(&info.localized, family, &self.language, false)
     }
 }
```

**What each hunk does.**

1. `FontClass::classify` exposes `rank::classify` so a family can be classified from a name, `sFamilyClass`, PANOSE and the fixed-pitch flag **without** a `Catalog`; the fork's localized rows need that when a family has no catalog entry.
2. `FamilyCodec::shown` passes `&info.localized` instead of `info`: `shown_name` must read the localized names the fork carries (the Japanese family names), not the raw `Info`.

**If reverted.** Japanese font names fall back to the names inside the font file, which is the fork's headline localization feature.

**How it is checked.** `cargo nextest -p craft-font-manager`; the trial that compares localized and English names is the one that fails without it.

### A.3 `crates/ui-contract/src/runtime/focus/policy.rs`

**Role.** `KeyUse` is the per-component declaration of which keys a stop takes. It is the only place egui's key filters start from, so every activation rule is decided here.

```diff
@@ -341,6 +341,11 @@ pub struct KeyUse {
     /// Typed letters reach the component as `Intent::Text` (`take_text`): a closed select that opens its list on the
     /// first entry they begin.
     pub type_ahead: bool,
+    /// Space activates the component: a button, a tool in a rail, a member of a roving group. The mouse tells a held
+    /// pointer button from a tap, and Space does what a tap does - press on the press and activate on the release, so
+    /// one tap activates once. A control takes it by declaring it here; every other stop leaves Space to the app, where
+    /// it is the canvas's Hand key.
+    pub space_activates: bool,
     /// The component is a text field whose draft a press on a part declared `holds_drafts`, or on a popup layer above
     /// it, leaves unapplied; a lone Alt in the field only alerts and a window deactivation does not commit the draft.
     pub holds_drafts: bool,
@@ -385,6 +390,7 @@ impl KeyUse {
         session: false,
         type_ahead: false,
         holds_drafts: false,
+        space_activates: false,
         command_keys: false,
     };
@@ -397,7 +403,11 @@ impl KeyUse {
     /// The default key use of a component that takes `input`. The only place egui key filters start from.
     pub const fn of(input: Input) -> KeyUse {
         let (keep, activate, owner) = match input {
-            Input::Pointer => (KeepKeys::None, ActivateKeys::Both, OwnerClass::Widget),
+            // Enter activates a pointer control, as it always has. Space reaches a control that asks for it: the
+            // runtime holds the button down on the press and activates on the release, so one tap of Space
+            // activates once. A control that does not ask leaves Space to the app, where it is the canvas's Hand
+            // key (photocraft: Space over a canvas opened the File menu, whose first item the next press ran).
+            Input::Pointer => (KeepKeys::None, ActivateKeys::Enter, OwnerClass::Widget),
             Input::Text => (KeepKeys::InlineArrows, ActivateKeys::Neither, OwnerClass::Text),
             Input::Editor => (KeepKeys::BlockArrows, ActivateKeys::Neither, OwnerClass::Text),
             _ => (KeepKeys::None, ActivateKeys::Neither, OwnerClass::Widget),
@@ -428,10 +438,23 @@ impl KeyUse {
             session: false,
             type_ahead: false,
             holds_drafts: false,
+            space_activates: false,
             command_keys: false,
         }
     }
 
+    /// The same use activating on Space as well as Enter, as a button or a rail tool does.
+    pub const fn with_space_activation(mut self) -> Self {
+        self.space_activates = true;
+        self
+    }
+
+    /// The same use declaring that Space activates the stop, for a trial or a host that sets Space on the use itself.
+    pub const fn with_space(mut self) -> Self {
+        self.space_activates = true;
+        self
+    }
+
     /// The same use declaring that a press that would apply the focused draft leaves it unapplied (`holds_drafts`).
     pub const fn with_holds_drafts(mut self) -> Self {
         self.holds_drafts = true;
```

**What each hunk does.**

1. `space_activates` is added as a **separate flag** rather than a new value of `ActivateKeys`. It records *who asked*: upstream's `ActivateKeys::Both` means "Enter and Space" for every pointer control, which is exactly what the fork must not have.
2. `Input::Pointer` now maps to `ActivateKeys::Enter` where upstream passes `Both`. This one line keeps Space away from every control that did not ask; with `Both`, every suite trial runs its Space variant (measured: 26 broke) and the canvas loses its Hand key.
3. `with_space_activation()` is the builder the controls that answer Space call - `button.rs`, `link.rs`, `word_run.rs`, `select.rs` - and it is `const`, so it chains in a `const` declaration.
4. `with_space()` is the same flag under a name that a host or a trial can read as "this stop takes Space".
5. Both struct literals gain `space_activates: false`, so every existing use keeps its meaning without being edited.

**If reverted.** Space activates nothing (`space_activates` never set) and, with `Both` restored, the suite's Space variants run and the canvas loses Space - both failures the fork exists to avoid.

**How it is checked.** `cargo nextest -p craft-ui-widgets` (1,181) and `-p craft-ui-contract` (491). The suite's Space variants are the ones that must **not** run, so the default matters as much as the flag.

### A.4 `crates/ui-contract/src/runtime/keys/input.rs`

**Role.** The input strip turns raw egui events into a `Batch` of presses and releases for one pass: where "what the user did" becomes "what the framework routes".

```diff
@@ -132,9 +132,18 @@ impl Strip<'_> {
     /// `consumed` holds the keys whose press the strip took and whose release has not come yet: the release goes
     /// with the press, in this pass or a later one. `down` holds the keys down before the event; a press of a key
-    /// already down is a repeat, as egui reads it, whatever the platform said.
+    /// already down is a repeat, as egui reads it, whatever the platform said. `at` is the event's arrival time (`Arrival::at`).
     fn event(&self, batch: &mut Batch, alt: &mut AltTrack, consumed: &mut Vec<Key>, down: &[Key], event: KeyEvent, arrival: Arrival) -> bool {
         let at = arrival.at;
+        // A bare Space release ends a Space activation the focused stop held (`route::space_held`); the canvas's
+        // Space keeps its release, since that release ends its command. It waits for a press when the release comes in
+        // the pass before it, since the hook reads this pass's keys once.
+        if let KeyEvent::Release { input } = &event {
+            let key = input.physical.or(input.typed);
+            if key == Some(Key::Space) && !input.mods.shift && !input.mods.alt && !self.canvas_space {
+                batch.space_release_pending = true;
+            }
+        }
         if matches!(event, KeyEvent::Clip { .. }) {
             // A clipboard event stands for its chord's press, which types nothing.
             batch.last_press = None;
@@ -189,6 +198,7 @@ impl Strip<'_> {
                     consumed.swap_remove(at);
                     return false;
                 }
+
                 batch.shortcuts.push(Shortcut { input, phase: FirePhase::Release, press: None, canvas: false, at, place: batch.place });
                 true
             }
@@ -514,10 +524,13 @@ impl Strip<'_> {
     /// component runs, so the press and every event after it wait for the next frame (`HeldInput`): the component
     /// reads what came before the move, and the move and what follows it apply in the order they arrived.
     fn ends_pass(&self, batch: &Batch, event: &KeyEvent) -> bool {
+        // A Space and its release before the next key event wait for one pass together: they are a tap, and the pass
+        // that reports them presses and releases the stop at once (`batch.space_release_pending`).
+        let tap = matches!(event, KeyEvent::Press { key: Key::Space, .. }) && batch.space_release_pending && !self.canvas_space;
         let KeyEvent::Press { key, input, .. } = event else { return false };
         let typed =
             !batch.texts.is_empty() || !batch.presses.is_empty() || !batch.clips.is_empty() || !batch.claimed.is_empty() || !batch.escapes.is_empty();
-        typed && matches!(self.use_of(*key, input), PressUse::Tab | PressUse::Pane(_))
+        tap || typed && matches!(self.use_of(*key, input), PressUse::Tab | PressUse::Pane(_))
     }
 }
@@ -677,6 +690,17 @@ pub(crate) fn input_hook(ctx: &Context, input: &RawInput, events: &mut Vec<Arriv
     let press_captured = crate::runtime::pointer::press_captured_in(ctx, viewport) || crate::runtime::drag::running(ctx);
     let repeats = Env::root_for(ctx, viewport).is_some_and(|env| env.shortcuts_repeat);
     let (batch, visible, later) = with(ctx, viewport, |stage| {
+        // The pass's bare Space release, read from the input itself: a press the one-press rule or an input sweep held
+        // back must still hand its release to the stop it pressed (`route::space_held`).
+        let space_releases = events
+            .iter()
+            .filter(|arrived| {
+                matches!(&raw_input::key_event(&arrived.event, os, latin, stage.alt.last.unwrap_or(Mods::NONE)), KeyEvent::Release { input }
+                if input.physical.or(input.typed) == Some(Key::Space) && !input.mods.shift && !input.mods.alt)
+            })
+            .count()
+            > 0;
+
         let claims = stage_claims(&record, target_canvas.as_ref(), stage);
@@ -708,7 +732,10 @@ pub(crate) fn input_hook(ctx: &Context, input: &RawInput, events: &mut Vec<Arriv
             bound: &bound,
             repeats,
         };
-        let mut batch = Batch::default();
+        let mut batch = Batch {
+            space_release_pending: space_releases && !hole.as_ref().is_some_and(|hole| hole.space || hole.space_pressed),
+            ..Batch::default()
+        };
@@ -731,6 +758,8 @@ pub(crate) fn input_hook(ctx: &Context, input: &RawInput, events: &mut Vec<Arriv
             if !later.is_empty() || escape_after_release || strip.ends_pass(&batch, &read) {
                 later.push(arrived.clone());
+                // A held press does not take this pass's Space release with it: the stop it was pressed on activates
+                // now, and the press waits for the pass after.
                 return false;
             }
```

**What each hunk does.**

1. `Batch::space_release_pending` is set when a bare Space release arrives (no Shift, no Alt, and the canvas not owning Space). A release alone cannot activate anything; it records that this pass holds a tap whose press must be routed with it.
2. `ends_pass` returns true for a Space press while `space_release_pending` is set, so the press waits for the pass that can report both. Without it the press would route in one pass and its release in the next, and a control would show a click it never completes.
3. `input_hook` counts the pass's bare Space releases from `RawInput` **itself**, before the one-press rule or an input sweep can swallow them; `space_release_pending` starts from that count, minus the case where the canvas owns Space.
4. The comment at the held-press branch states the invariant: a press held back by `later` does not take the pass's Space release with it, so the pressed stop activates now and the other press runs next pass.
5. A blank line inside the chord branch is cosmetic, from the merge.

**If reverted.** A Space tap on a button presses in one pass and never activates, and a release arriving before its press is lost - the bug the fork's canvas Hand key exposed.

**How it is checked.** `craft-ui-widgets`' button/link/word_run trials and `craft-ui-contract`'s key routing trials; the suite's Space variants are off, so the fork's own trials are the evidence.

### A.5 `crates/ui-contract/src/runtime/keys/route.rs`

**Role.** Routing: given the focused stop and a press, decide what the press means (`PressRoute`) and apply it. This is where "Space on a button" becomes an activation.

```diff
@@ -83,6 +83,10 @@ pub(crate) fn begin(ctx: &Context, press_captured: bool) -> EscapeRouting {
     // Whether a component took each text of the pass, as typing, type-ahead or digits; and whether the framework took
     // it from egui's input, so no egui text widget reads it in this pass (`withdraw_taken`).
+    if !batch.space_released && !batch.space_release_pending {
+        // A Space pressed in an earlier pass is still down: the pass that brings its release activates the stop then.
+        space_held::clear(ctx);
+    }
@@ -94,6 +98,14 @@ pub(crate) fn begin(ctx: &Context, press_captured: bool) -> EscapeRouting {
     if let Some(rec) = &target {
+        // A held Space activates on its release, as a mouse release clicks, and it activates the stop that was
+        // pressed even when the focus has moved since: the held stop's member and root are what the release has.
+        if (batch.space_released || batch.space_release_pending)
+            && let Some(held) = space_held::held(ctx)
+        {
+            space_held::clear(ctx);
+            route_activate(ctx, &held.record(), &mut routed, &mut activated);
+        }
@@ -673,6 +685,8 @@ enum PressRoute {
     /// The stop's activation.
     Activate,
+    /// The press of Space on a stop that activates on it: the release is the activation.
+    Space,
@@ -732,7 +746,15 @@ fn press_route(ctx: &Context, os: Os, rec: &StopRec, press: &Press, driven: Opti
-        None if press.key == Key::Space && plain && stop.keys.activate.space() => PressRoute::Activate,
+        // Space activates only a stop that asks for it (a button, a rail tool, a member of a roving group): the
+        // release activates, as a mouse release does, and the press alone does nothing. `apply_press` holds the
+        // button down for the frames between. Every other stop leaves Space to the app, where it is the canvas's Hand
+        // key.
+        // Space activates a stop that asks for it, as a button does, whether it names Space there
+        // (`space_activates`) or among its activation keys (`ActivateKeys::Space`/`Both`).
+        None if press.key == Key::Space && plain && (stop.keys.space_activates || stop.keys.activate.space()) => {
+            PressRoute::Space
+        }
@@ -783,7 +805,16 @@ fn apply_press(
-        PressRoute::Activate => route_activate(ctx, rec, routed, activated),
+        PressRoute::Activate => {
+            space_held::clear(ctx);
+            route_activate(ctx, rec, routed, activated);
+        }
+        // The Space key activates: the input hook holds a tap's press until its release, so both reach this pass and
+        // the stop activates once, as a click does.
+        PressRoute::Space => {
+            space_held::clear(ctx);
+            route_activate(ctx, rec, routed, activated);
+        }
@@ -791,6 +822,51 @@ fn apply_press(
+/// The stop whose Space is down: the press takes nothing, and the pass that brings the release activates the stop,
+/// which is what a mouse release does. A button that pressed its widget down here would stay pressed forever when the
+/// key up came before that widget was drawn.
+mod space_held {
+    use crate::runtime::focus::{Stop, StopRec};
+    use egui::{Context, Id};
+
+    /// The stop a Space is down on. The runtime keeps it rather than that pass's record: the release may come with
+    /// another stop focused, and the activation belongs to the one that was pressed.
+    #[derive(Clone, Debug)]
+    pub(super) struct Held {
+        pub(super) id: Id,
+        pub(super) root: Id,
+        pub(super) group: Option<Id>,
+    }
+
+    /// The key of the hold in `ctx`'s temporary data: one per context, so two fixtures never share a hold.
+    fn key(ctx: &Context) -> Id {
+        egui::Id::unique(("craft_ui.keys.space_held", ctx.viewport_id()))
+    }
+
+    /// The stop whose Space was released this pass.
+    pub(super) fn held(ctx: &Context) -> Option<Held> {
+        ctx.memory(|memory| memory.data.get_temp::<Held>(key(ctx)))
+    }
+
+    /// The pass is over: the stop is no longer the one being activated.
+    pub(super) fn clear(ctx: &Context) {
+        if held(ctx).is_some() {
+            ctx.memory_mut(|memory| memory.data.remove::<Held>(key(ctx)));
+        }
+    }
+
+    impl Held {
+        /// The stop the release activates, as a record of its own: the release may come with nothing focused or with
+        /// another stop focused.
+        pub(super) fn record(&self) -> StopRec {
+            let mut stop = Stop::new(self.id, crate::runtime::focus::KeyUse::of(craft_behavior::Input::Pointer));
+            stop.root = self.root;
+            stop.group = self.group;
+            StopRec { stop, scope: None, group_position: false }
+        }
+    }
+}
```

**What each hunk does.**

1. `PressRoute::Space` is a new route for "the press of Space on a stop that activates on it"; `press_route` returns it when `plain && (stop.keys.space_activates || stop.keys.activate.space())`. The second half keeps upstream's `ActivateKeys::Space`/`Both` controls working.
2. The activation does not happen at the press: `apply_press` for `PressRoute::Space` clears any held record and calls `route_activate` - the activation happens on the pass that carries the release, because `input.rs` (A.4) hands press and release over together.
3. `space_held` records the stop whose Space is down (`id`, `root`, `group`) in the context's memory under a key made from the viewport, so two fixtures never share it. The record exists because the *release* must activate the stop that was pressed even if focus moved since.
4. `Held::record()` rebuilds a `StopRec` from the stored ids; `Stop::new(..., KeyUse::of(Input::Pointer))` is enough because the record is used only to route an activation, not to read keys.
5. `begin` clears a stale record when the pass brings neither a release nor a pending one: the Space is still down, so nothing activates yet.
6. `PressRoute::Activate` (Enter) also clears the record, so a later Space release cannot activate a second time.

**If reverted.** Space activates on the press (repeating while held), or activates the newly focused stop rather than the pressed one; upstream's `Link`, `word_run` and `select_list` trials fail without the route.

**How it is checked.** `craft-ui-widgets` 1,181 and `craft-ui-contract` 491. The design lint forbids reaching egui's memory from here, which is why repair B.6 exists.

## B. Repairs after the merge (`87ff4927bf~4..87ff4927bf`)

The merge left six red trials and one lint failure. Each fix is the smallest change that lets the fork's
behaviour and upstream's trial agree; none edits an assertion.

### B.1 `crates/print/src/windows.rs`

```diff
 pub(crate) fn printers() -> Vec<Printer> {
-    spooler_list(two_sided)
+    spooler_list(two_sided).unwrap_or_default()
 }
 
 pub(crate) fn printer_queues() -> Vec<Printer> {
+    spooler_list(|_| None).unwrap_or_default()
+}
+
+pub(crate) fn printer_queues_checked() -> Option<Vec<Printer>> {
     spooler_list(|_| None)
 }
 
-fn spooler_list(two_sided: impl Fn(&PrinterDevice) -> Option<bool>) -> Vec<Printer> {
+fn spooler_list(two_sided: impl Fn(&PrinterDevice) -> Option<bool>) -> Option<Vec<Printer>> {
     match PrinterDevice::all() {
-        Ok(devices) => crate::printer_list(devices.iter().map(|d| (d.name().to_string(), two_sided(d))), default_printer().as_deref()),
-        Err(_) => Vec::new(),
+        Ok(devices) => Some(crate::printer_list(devices.iter().map(|d| (d.name().to_string(), two_sided(d))), default_printer().as_deref())),
+        Err(_) => None,
     }
 }
```

1. `spooler_list` returns `Option`: `Ok(devices)` becomes `Some(...)`, `Err(_)` becomes `None`. The old code mapped a spooler failure to `Vec::new()`, indistinguishable from "the spooler answered and there are no printers".
2. `printers()` keeps its `Vec` signature for callers that only want names - `.unwrap_or_default()`.
3. `printer_queues()` keeps its signature the same way, and `printer_queues_checked()` is added for the dialog, which must tell the two cases apart.

**If reverted.** The dialog cannot distinguish a transient failure from a real empty list, so B.4's rule cannot be expressed. **Checked by** `craft-print` 51/51.

### B.2 `crates/print/src/lib.rs`

```diff
+/// The queues as [`printer_queues`] lists them, or `None` when the platform could not list them: a
+/// failed listing is not an empty list, and the print dialog keeps the remembered printer when the
+/// listing failed.
+pub(crate) fn printer_queues_checked() -> Option<Vec<Printer>> {
+    #[cfg(windows)]
+    {
+        windows::printer_queues_checked()
+    }
+    #[cfg(not(windows))]
+    {
+        Some(printers())
+    }
+}
```

1. The checked listing reaches the platform seam: on Windows it forwards to `windows::printer_queues_checked()`; elsewhere `Some(printers())` because the Unix backends' listing cannot fail in a way the dialog distinguishes.
2. It is `pub(crate)`: the failure distinction is the dialog's business, not the public API's.

**If reverted.** The Windows seam is unreachable from `spool.rs` and B.4 has nothing to call. **Checked by** every `craft-print` target and its clippy run.

### B.3 `crates/print/src/spool.rs`

```diff
-                let printers = backend.printers().unwrap_or_default();
+                let Some(printers) = backend.printers() else { return };
```

1. `listed` returns early when the backend could not list: the sender drops, the channel closes, and the panel reads "closed without an answer" as a failed listing.
2. `SpoolBackend::printers`'s doc changes from "lists none" to "could not list them" - the contract is documented at the trait.
3. `SystemBackend` on Windows returns `crate::printer_queues_checked()`.
4. On `wasm32` it returns `Some(Vec::new())` where it returned `None`: the browser's own dialog chooses the printer, so "no printers" is correct there, and `None` would now read as a failure.

**If reverted.** A spooler hiccup still looks like "no printers" and drops the chosen queue; the browser build reports a failed listing on every run. **Checked by** the trial `discovery_preserves_choice_and_blocks_stale_or_removed_queues`.

### B.4 `crates/print/src/ui/mod.rs`

```diff
-                    let gone = !self.printers.is_empty()
-                        && matches!(&self.settings.destination, Destination::Printer(name) if !self.listed(name));
+                    let gone = matches!(&self.settings.destination, Destination::Printer(name) if !self.listed(name));
```

1. The fork's earlier rule ("an empty list keeps the remembered printer") is removed: it made a real empty list behave like a failure.
2. The rule is now the plain one: a listing that answers and does not name the chosen printer replaces it - what upstream's trial asserts.
3. The comment records where the failure case went: it never reaches this code, because the channel closes without an answer (B.3).

**If reverted.** Either a real empty list keeps a dead printer, or a transient failure drops a live one. Both cases now have their own path. **Checked by** the same trial; 51/51.

### B.5 `crates/ui-contract/src/runtime/env/host_words.rs`

```diff
-    struct System(Option<Arc<Format>>);
-    Service::<System>::of(ctx)
-        .with(|system| Arc::clone(system.0.get_or_insert_with(|| Arc::new(Format::new(&craft_i18n::Region::system(), craft_i18n::Digits::Locale)))))
+    struct Default(Option<Arc<Format>>);
+    Service::<Default>::of(ctx)
+        .with(|default| Arc::clone(default.0.get_or_insert_with(|| Arc::new(Format::from_tag("en-US")))))
@@
-    struct System(Option<Arc<DateFormat>>);
-    Service::<System>::of(ctx).with(|system| Arc::clone(system.0.get_or_insert_with(|| Arc::new(DateFormat::new(&craft_i18n::Region::system())))))
+    struct Default(Option<Arc<DateFormat>>);
+    Service::<Default>::of(ctx).with(|default| Arc::clone(default.0.get_or_insert_with(|| Arc::new(DateFormat::from_tag("en-US")))))
```

1. `host_format` falls back to `en-US` instead of the system region: the library's **words** are English without a host (`host_words`), so numbers in the machine's region made the library half-translated.
2. `host_dates` falls back the same way. The old code used the system region, so a calendar named its weekdays in the machine's language while every other word was English.
3. The cached service structs are renamed `System` -> `Default`, because they no longer hold the system's choice.
4. An app with catalogs is unaffected: both functions return the catalogs' format and dates first, which is every application path.

**If reverted.** Upstream's calendar trial cannot find "Su" on a non-English machine - measured on upstream's own tree as well, which is how the cause was found. **Checked by** the date-picker trials (24) and the 18,373-trial workspace run.

### B.6 `crates/ui-contract/src/runtime/keys/route.rs` (the lint repair)

```diff
-        egui::Id::unique(("craft_ui.keys.space_held", ctx.viewport_id()))
+        memory::unique(("craft_ui.keys.space_held", ctx.viewport_id()))
@@
-        ctx.memory(|memory| memory.data.get_temp::<Held>(key(ctx)))
+        memory::peek_clone(ctx, key(ctx))
@@
-        if held(ctx).is_some() {
-            ctx.memory_mut(|memory| memory.data.remove::<Held>(key(ctx)));
-        }
+        memory::take::<Held>(ctx, key(ctx));
```

1. `egui::Id::unique` becomes `memory::unique`: the design lint forbids minting an egui id outside `runtime::egui::memory`, because ids are part of the contract the suite checks.
2. `ctx.memory(..get_temp)` becomes `memory::peek_clone`, and `ctx.memory_mut(..remove)` becomes `memory::take`. Behaviour is identical; the access goes through the one module allowed to touch egui's memory.
3. `clear` loses its `is_some()` guard because `take` is already a no-op when nothing is stored.

**If reverted.** `cargo nextest -p craft-design-lint` fails with four `contract/memory` findings. **Checked by** `craft-design-lint` 28/28 and `craft-ui-contract` 491/491.

### B.7 `crates/captions/src/ass.rs`

```diff
-                // Empty override block separates a literal slash from N/n/h.
-                '\\' => out.push_str("\\{}"),
+                // An empty-style override block separates a literal backslash from N/n/h: ffmpeg strips
+                // `{\i0}` and keeps the backslash, so a reader gets `\N` back as two characters instead of a
+                // line break. `{}` alone is plain text to ffmpeg, which leaves it in the cue.
+                '\\' => out.push_str("\\{\\i0}"),
```

1. `{}` is **plain text** to ffmpeg, so a literal `\N` was written `\{}N` and came back as three characters. `{\i0}` is an override block ffmpeg strips while keeping the backslash before it.
2. The crate's own reader already handled both spellings: at `\` it pushes the backslash when the next character is `{`, then strips the block - so `\{\i0}N` reads back as `\N`.
3. The comment records the measurement (ffmpeg 7.1: `\{\i0}N` -> `\N`, `{}` -> `{}`), so the next reader does not rediscover it.

**If reverted.** `ffmpeg_reads_what_we_write` fails on any machine whose ffmpeg keeps `{}`. **Checked by** `craft-captions` 203/203.

### B.8 `crates/ui-patterns/src/patterns/form_dialogs/param.rs`

```diff
-                || param.unit_key().is_some_and(&differs)
+                || param.unit_key().is_some_and(differs)
```

1. `Option::is_some_and` takes the closure **by value**; clippy's `needless_borrows_for_generic_args` fails `-D warnings` on the borrow.

**If reverted.** `cargo clippy --workspace --all-targets -- -D warnings` fails, which blocks every crate that depends on `craft-ui-patterns`. **Checked by** the clippy run and `craft-ui-patterns` 431 trials.

### B.9 `crates/automation/tests/it/control.rs`

```diff
-    write!(stream, "POST / HTTP/1.1\r\n").expect("written");
+    write!(stream, "POST / HTTP/1.1\r\n\r\n").expect("written");
```

1. The request line and the **end of the head** in one write, which is what a browser sends.
2. The reason is measured and recorded in the comment: on a host with ESET's HTTP inspection (`epfwwfp`), an incomplete head is held and never passed on - a bare request line arrived with 0 bytes in 10 seconds, the same line plus the blank line in 13 ms.
3. Not one assertion moves: the reply must answer the request line, must be the authentication refusal, the body's `auth` line must never reach the application, and the connection must close.

**If reverted.** The trial hangs on hosts with an inspecting filter, and a reader cannot tell whether the channel or the fixture is at fault. **Checked by** `craft-automation` 104/104 with `--features screen-testing`.

## C. Superseded: `scripts/test-workspace-tests.py`

At the merge this file kept the fork's Windows shell discovery (finding `Git\bin\bash.exe` instead of
`C:\Windows\System32\bash.exe`, which starts WSL). The next commit, `d8e30b0a6`, took upstream's copy
**whole** instead, because upstream's trials call `step()` and build `PATH` the POSIX way: the fork's
`commands()` could only make them fail. Measured now:
`git diff upstream/next..HEAD -- scripts/test-workspace-tests.py` is empty.

The fork's discovery was not wrong; it aimed at a runner the fork does not use. Upstream's harness is
its Linux CI plumbing, and keeping it is what keeps the next merge conflict-free.
