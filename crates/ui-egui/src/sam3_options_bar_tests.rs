//! The SAM Selection tool's options bar: the mode buttons, the prompt field, Threshold, Refine,
//! Individual Masks, Sample All Layers and the Detect button, rendered without a panic and
//! reachable by name.

use egui::vec2;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use serde_json::json;

use crate::PhotocraftApp;
use crate::state::Tool;

fn bar(tool: Tool, drawn: bool) -> Harness<'static, Box<PhotocraftApp>> {
    let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), crate::Services::default());
    app.run("file.new", json!({"width": 64, "height": 64})).unwrap();
    app.ui.tool = tool;
    let mut ready = false;
    let mut h = Harness::builder().with_size(vec2(1400.0, 60.0)).build_ui_state(
        move |ui: &mut egui::Ui, app: &mut Box<PhotocraftApp>| {
            if !ready {
                crate::theme::install_fonts(ui.ctx());
                crate::theme::apply(ui.ctx(), &crate::theme::AppTheme::pro());
                ready = true;
                return;
            }
            assert_eq!(crate::tools::retouch_ui::bar::options_bar(ui, app, app.ui.tool), drawn, "{:?}", app.ui.tool);
        },
        app,
    );
    h.run_steps(4);
    h
}

#[test]
fn the_sam_selection_bar_shows_its_prompt_threshold_refine_and_detect() {
    let h = bar(Tool::Sam3Select, true);
    for label in ["Detect", "Sample All Layers", "Individual Masks", "Prompt", "Threshold", "Refine"] {
        assert!(h.query_all(egui_kittest::kittest::by().label_contains(label)).next().is_some(), "the options bar has no {label}");
    }
}

/// The mode buttons: the tool combines like the other selection tools, so the options bar carries
/// New / Add / Subtract / Intersect.
#[test]
fn the_sam_selection_bar_has_the_four_mode_buttons() {
    let h = bar(Tool::Sam3Select, true);
    for tip in ["New selection", "Add to selection", "Subtract from selection", "Intersect with selection"] {
        assert!(h.query_all(egui_kittest::kittest::by().label_contains(tip)).next().is_some(), "the options bar has no {tip} button");
    }
}

/// A tool without a bar still says so, so the frame draws none.
#[test]
fn another_tool_reports_no_bar() {
    bar(Tool::Hand, false);
}
