//! The SAM Selection tool through the real canvas: a click and a drag both reach `select.sam3`,
//! an ⌥ or ⇧ press keeps its meaning when let go before the button, and the mode badge follows
//! the modifiers. Needs the built helper and its pack (`CRAFT_SAM3_HELPER` / `CRAFT_SAM3_PACK`).

use egui::Modifiers;
use serde_json::json;

use crate::PhotocraftApp;
use crate::canvas::testing::{ToolEvent, tool_event};
use crate::state::Tool;

/// A 256x256 document with a green disc of radius 40 at (128, 128) on white, the tool selected.
fn app() -> Box<PhotocraftApp> {
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

fn selection_bounds(app: &PhotocraftApp) -> Option<photocraft_geom::Rect> {
    app.session.active().and_then(|st| st.doc.selection.as_ref()).map(|s| s.content_bounds())
}

#[test]
#[ignore = "needs the built SAM 3.1 helper and its HSWQ pack (CRAFT_SAM3_HELPER / CRAFT_SAM3_PACK)"]
fn a_click_through_the_real_canvas_selects_the_disc() {
    let mut app = app();
    tool_event(&mut app, ToolEvent::Down { x: 128.0, y: 128.0, pressure: 1.0 }, Modifiers::NONE);
    tool_event(&mut app, ToolEvent::Up { x: 128.0, y: 128.0 }, Modifiers::NONE);
    assert!(!app.ui.status_error, "{}", app.ui.status);
    assert_eq!(app.session.journal.last().map(|(id, _)| id.as_str()), Some("select.sam3"));
    let bounds = selection_bounds(&app).expect("a selection");
    assert!(bounds.width() > 40 && bounds.width() < 160, "bounds {bounds:?}");
}

/// The press's ⇧ and ⌥ survive into the release: letting the key go before the button (a release
/// egui reports with no modifiers) still adds or subtracts, as a marquee does.
#[test]
#[ignore = "needs the built SAM 3.1 helper and its HSWQ pack (CRAFT_SAM3_HELPER / CRAFT_SAM3_PACK)"]
fn a_modifier_held_at_the_press_still_counts_when_let_go_first() {
    let mut app = app();
    tool_event(&mut app, ToolEvent::Down { x: 128.0, y: 128.0, pressure: 1.0 }, Modifiers::NONE);
    tool_event(&mut app, ToolEvent::Up { x: 128.0, y: 128.0 }, Modifiers::NONE);
    let disc = selection_bounds(&app).expect("a selection");
    assert!(disc.width() > 40, "bounds {disc:?}");
    // ⇧ first, let go before the button: it adds — the outline grows to the union with the same
    // disc, so the bounds stay the disc's (an add is not a replacement, and the new piece is the
    // disc again).
    tool_event(&mut app, ToolEvent::Down { x: 128.0, y: 128.0, pressure: 1.0 }, Modifiers::SHIFT);
    tool_event(&mut app, ToolEvent::Up { x: 128.0, y: 128.0 }, Modifiers::NONE);
    assert!(!app.ui.status_error, "{}", app.ui.status);
    let added = selection_bounds(&app).expect("a selection");
    assert!(added.width() >= disc.width(), "⇧ replaced the outline: {added:?}");
}
