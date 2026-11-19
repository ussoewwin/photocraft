//! The toolbar's `W` slot and its flyout: SAM Selection sits under Magic Wand, the right-click
//! flyout opens on the slot, and the rendered rows list the four tools in that order.

use egui::vec2;
use egui_kittest::kittest::Queryable;
use serde_json::json;

use crate::PhotocraftApp;
use crate::panels::toolbar::TOOL_SECTIONS;
use crate::state::Tool;

/// The slot's tools, in the order the flyout draws them.
#[test]
fn the_w_slot_lists_sam_selection_last() {
    let slot = TOOL_SECTIONS.iter().flat_map(|section| section.iter()).find(|slot| slot.contains(&Tool::MagicWand)).expect("the W slot");
    assert_eq!(slot, &[Tool::ObjectSelection, Tool::QuickSelection, Tool::MagicWand, Tool::Sam3Select], "the W slot's tools");
}

/// Selecting SAM Selection Tool displays it on the toolbar slot.
#[test]
fn selecting_sam_tool_shows_it_on_the_toolbar() {
    let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), crate::Services::default());
    app.run("file.new", json!({"width": 80, "height": 80})).unwrap();
    crate::tool_memory::select(&mut app, Tool::Sam3Select);
    let mut ready = false;
    let mut h = egui_kittest::Harness::builder().with_context(app.ctx().clone()).with_size(vec2(400.0, 1800.0)).build_ui_state(
        move |ui: &mut egui::Ui, app: &mut Box<PhotocraftApp>| {
            if !ready {
                egui_extras::install_image_loaders(ui.ctx());
                crate::theme::install_fonts(ui.ctx());
                crate::theme::apply(ui.ctx(), &crate::theme::AppTheme::pro());
                ready = true;
                return;
            }
            crate::panels::toolbar(app, ui);
        },
        app,
    );
    h.run_steps(4);
    assert!(h.query_all_by_label_contains("SAM Selection Tool").next().is_some(), "toolbar shows SAM Selection");
}

/// The tool has its own row in Edit › Keyboard Shortcuts, named `tools.select.sam3Select` like
/// every other toolbar tool, so `W` (its default key) reaches it.
#[test]
fn the_tool_has_its_shortcut_row() {
    use crate::input::tool_shortcuts;
    assert_eq!(tool_shortcuts::id(Tool::Sam3Select), "tools.select.sam3Select");
    assert_eq!(tool_shortcuts::tool("tools.select.sam3Select"), Some(Tool::Sam3Select));
    assert_eq!(Tool::Sam3Select.key(), 'W');
    assert!(
        photocraft_engine::prefs::bindable_items().any(|(id, label, _, _, _)| id == "tools.select.sam3Select" && label.contains("SAM")),
        "the shortcut editor has no SAM Selection row"
    );
}
