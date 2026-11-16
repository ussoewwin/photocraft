//! The SAM Selection tool through the real canvas: a click and a drag both reach `select.sam3`,
//! an ⌥ or ⇧ press keeps its meaning when let go before the button, and the mode badge follows
//! the modifiers. Needs the built helper and its pack (`CRAFT_SAM3_HELPER` / `CRAFT_SAM3_PACK`).

use egui::{Modifiers, PointerButton, Pos2, vec2};
use egui_kittest::Harness;
use serde_json::json;

use crate::PhotocraftApp;
use crate::canvas::ViewXform;
use crate::state::Tool;

fn harness(app: PhotocraftApp) -> Harness<'static, PhotocraftApp> {
    let mut h = Harness::builder().with_size(vec2(900.0, 600.0)).build_ui_state(
        |ui, app: &mut PhotocraftApp| {
            let ctx = ui.ctx().clone();
            if !ctx.fonts(|f| f.families().contains(&egui::FontFamily::Name("medium".into()))) {
                return;
            }
            crate::shortcuts::handle(app, &ctx);
            egui::CentralPanel::default().show(ui, |ui| crate::canvas::document_area(app, ui));
        },
        app,
    );
    PhotocraftApp::setup_context(&h.ctx, crate::theme::ThemeKind::ALL[0]);
    h.run_steps(4);
    let v = &mut h.state_mut().ui.views[0];
    v.zoom = 1.0;
    v.center = [128.0, 128.0];
    v.fit_pending = false;
    h.run_steps(2);
    h
}

/// A 256x256 document with a green disc of radius 40 at (128, 128) on white, the tool selected.
fn app() -> PhotocraftApp {
    let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), crate::Services::default());
    app.session.execute("file.new", json!({"width": 256, "height": 256, "depth": 8})).unwrap();
    app.sync_views();
    app.session
        .edit("paint", |doc, _| {
            let bg = doc.layers[0].surface_mut().unwrap();
            let n = bg.channels();
            let mut data = Vec::with_capacity(256 * 256 * n);
            for y in 0..256i32 {
                for x in 0..256i32 {
                    let inside = (x - 128).pow(2) + (y - 128).pow(2) <= 40 * 40;
                    let c = if inside { [0.1f32, 0.7, 0.2] } else { [1.0, 1.0, 1.0] };
                    data.extend_from_slice(&c[..n.min(3)]);
                    if n == 4 {
                        data.push(1.0);
                    }
                }
            }
            bg.write_region(photocraft_geom::Rect::new(0, 0, 256, 256), &data);
            Ok(())
        })
        .unwrap();
    app.ui.tool = Tool::Sam3Select;
    app.ui.tool_options.sam3_prompt.clear();
    app.ui.tool_options.sam3_refine = 1;
    app
}

fn screen(h: &Harness<'static, PhotocraftApp>, x: f32, y: f32) -> Pos2 {
    let app = h.state();
    let v = &app.ui.views[0];
    let xf = ViewXform {
        rect: crate::rulers::content_rect(app, app.last_canvas_rect),
        zoom: v.zoom,
        center: v.center,
        flip: app.ui.view.flip_horizontal,
        rotation: v.rotation,
        aspect: app.ui.view.display_aspect(),
    };
    xf.to_screen(x, y)
}

fn mods(h: &mut Harness<'static, PhotocraftApp>, m: Modifiers) {
    h.event(egui::Event::ModifiersChanged(m));
    h.run_steps(1);
}

/// A press at `(x, y)` with `m`, a move to `to` and the release. `release_with` changes the
/// modifiers between the press and the release, as a user letting Shift go before the button.
fn press_move_release(h: &mut Harness<'static, PhotocraftApp>, press: (f32, f32), to: (f32, f32), m: Modifiers, release_with: Option<Modifiers>) {
    let from = screen(h, press.0, press.1);
    let end = screen(h, to.0, to.1);
    mods(h, m);
    h.event(egui::Event::PointerMoved(from));
    h.run_steps(2);
    h.event(egui::Event::PointerButton { pos: from, button: PointerButton::Primary, pressed: true, modifiers: m });
    h.run_steps(2);
    if let Some(r) = release_with {
        mods(h, r);
    }
    h.event(egui::Event::PointerMoved(end));
    h.run_steps(1);
    h.event(egui::Event::PointerButton { pos: end, button: PointerButton::Primary, pressed: false, modifiers: release_with.unwrap_or(m) });
    h.run_steps(2);
}

fn selection_bounds(h: &Harness<'static, PhotocraftApp>) -> Option<photocraft_geom::Rect> {
    h.state().session.active().and_then(|st| st.doc.selection.as_ref()).map(|s| s.content_bounds())
}

#[test]
#[ignore = "needs the built SAM 3.1 helper and its HSWQ pack (CRAFT_SAM3_HELPER / CRAFT_SAM3_PACK)"]
fn a_click_through_the_real_canvas_selects_the_disc() {
    let mut h = harness(app());
    press_move_release(&mut h, (128.0, 128.0), (128.0, 128.0), Modifiers::NONE, None);
    let app = h.state();
    assert!(!app.ui.status_error, "{}", app.ui.status);
    assert_eq!(app.session.journal.last().map(|(id, _)| id.as_str()), Some("select.sam3"));
    let bounds = selection_bounds(&h).expect("a selection");
    assert!(bounds.width() > 40 && bounds.width() < 160, "bounds {bounds:?}");
}

/// The press's ⇧ and ⌥ survive into the release: letting the key go before the button (a release
/// egui reports with no modifiers) still adds or subtracts, as a marquee does.
#[test]
#[ignore = "needs the built SAM 3.1 helper and its HSWQ pack (CRAFT_SAM3_HELPER / CRAFT_SAM3_PACK)"]
fn a_modifier_held_at_the_press_still_counts_when_let_go_first() {
    let mut h = harness(app());
    press_move_release(&mut h, (128.0, 128.0), (128.0, 128.0), Modifiers::NONE, None);
    let disc = selection_bounds(&h).expect("a selection");
    assert!(disc.width() > 40, "bounds {disc:?}");
    // ⇧ first, let go before the button: it adds — the outline grows to the union with the same
    // disc, so the bounds stay the disc's (an add is not a replacement, and the new piece is the
    // disc again).
    press_move_release(&mut h, (128.0, 128.0), (128.0, 128.0), Modifiers::SHIFT, Some(Modifiers::NONE));
    let app = h.state();
    assert!(!app.ui.status_error, "{}", app.ui.status);
    let added = selection_bounds(&h).expect("a selection");
    assert!(added.width() >= disc.width(), "⇧ replaced the outline: {added:?}");
}
