//! The toolbar's `W` slot and its flyout: SAM Selection sits under Magic Wand, the right-click
//! flyout opens on the slot, and the rendered rows list the four tools in that order.

use egui::{PointerButton, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use serde_json::json;

use crate::PhotocraftApp;
use crate::panels::toolbar::TOOL_SECTIONS;
use crate::state::Tool;

fn toolbar(tool: Tool) -> Harness<'static, PhotocraftApp> {
    let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), crate::Services::default());
    app.run("file.new", json!({"width": 80, "height": 80})).unwrap();
    app.ui.tool = tool;
    let mut h = Harness::builder().with_size(vec2(1800.0, 900.0)).build_ui_state(
        |ui, app: &mut PhotocraftApp| {
            crate::panels::toolbar(app, ui);
        },
        app,
    );
    PhotocraftApp::setup_context(&h.ctx, Default::default());
    h.run_steps(4);
    h
}

fn slot_rect(h: &Harness<'static, PhotocraftApp>, tool: Tool) -> egui::Rect {
    h.query_all(egui_kittest::kittest::by().label_contains(tool.label())).next().unwrap_or_else(|| panic!("no toolbar button for {}", tool.label())).rect()
}

fn click(h: &mut Harness<'static, PhotocraftApp>, at: egui::Pos2, button: PointerButton) {
    h.event(egui::Event::PointerMoved(at));
    h.run_steps(1);
    for pressed in [true, false] {
        h.event(egui::Event::PointerButton { pos: at, button, pressed, modifiers: egui::Modifiers::NONE });
        h.run_steps(1);
    }
    h.run_steps(2);
}

/// The slot's tools, in the order the flyout draws them.
#[test]
fn the_w_slot_lists_sam_selection_last() {
    let slot = TOOL_SECTIONS.iter().flat_map(|section| section.iter()).find(|slot| slot.contains(&Tool::MagicWand)).expect("the W slot");
    assert_eq!(slot, &[Tool::ObjectSelection, Tool::QuickSelection, Tool::MagicWand, Tool::Sam3Select], "the W slot's tools");
}

/// Right-clicking the slot opens its flyout anchored on that same button — the flyout the render
/// below draws SAM Selection in.
#[test]
fn right_clicking_the_w_slot_opens_its_flyout() {
    let mut h = toolbar(Tool::MagicWand);
    let rect = slot_rect(&h, Tool::MagicWand);
    click(&mut h, rect.center(), PointerButton::Secondary);
    let open = h.ctx.data(|d| d.get_temp::<(egui::Id, egui::Rect)>(egui::Id::unique("tool-flyout")));
    let (_, anchor) = open.expect("the W slot's flyout is open");
    assert_eq!(anchor, rect, "the flyout is anchored on the slot's button");
}

/// A click on a flyout row picks it and closes the flyout: the first row (Object Selection) moves
/// the tool, and the rendered flyout draws SAM Selection as the fourth (see the record in PLAN.md).
#[test]
fn a_click_on_a_flyout_row_picks_it() {
    let mut h = toolbar(Tool::MagicWand);
    let slot = slot_rect(&h, Tool::MagicWand);
    click(&mut h, slot.center(), PointerButton::Secondary);
    let (_key, anchor) = h.ctx.data(|d| d.get_temp::<(egui::Id, egui::Rect)>(egui::Id::unique("tool-flyout"))).expect("the flyout is open");
    // The rows are 26 px apart, the first centred 21 px below the button's top (measured on the
    // rendered flyout).
    let first_row = egui::pos2(anchor.right() + 100.0, anchor.top() + 21.0);
    click(&mut h, first_row, PointerButton::Primary);
    assert_eq!(h.state().ui.tool, Tool::ObjectSelection, "the flyout's first row (Object Selection) is picked");
    let open = h.ctx.data(|d| d.get_temp::<(egui::Id, egui::Rect)>(egui::Id::unique("tool-flyout")));
    assert!(open.is_none(), "picking a row closes the flyout");
}

/// The tool has its own row in Edit › Keyboard Shortcuts, named `tools.select.sam3Select` like
/// every other toolbar tool, so `W` (its default key) reaches it.
#[test]
fn the_tool_has_its_shortcut_row() {
    use crate::input::tool_shortcuts;
    assert_eq!(tool_shortcuts::id(Tool::Sam3Select), "tools.select.sam3Select");
    assert_eq!(tool_shortcuts::tool("tools.select.sam3Select"), Some(Tool::Sam3Select));
    assert_eq!(Tool::Sam3Select.key(), 'W');
    let app = PhotocraftApp::new(photocraft_engine::Session::new(), crate::Services::default());
    let items = crate::prefs_ui::shortcut_items(&app);
    assert!(items.iter().any(|(id, label, _, _)| id == "tools.select.sam3Select" && label.contains("SAM")), "the shortcut editor has no SAM Selection row");
}
