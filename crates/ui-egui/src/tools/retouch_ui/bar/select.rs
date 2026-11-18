//! The Retouch options bar's selection family: Quick Selection and Object Selection.

use craft::ui::buttons::Button;
use craft::ui::contract::{Bind, Props as _, Show as _, clicked};
use craft::ui::display::Divider;
use craft::ui::runtime::env::Label;
use craft::ui::text::{RangeAlert, TextField};
use serde_json::json;

use super::heal::check;
use crate::PhotocraftApp;
use crate::option_cells::{hint, number};
use crate::state::Tool;

/// The Select Subject button both selection tools carry; `empty` is the message when nothing is found.
fn select_subject(ui: &mut egui::Ui, app: &mut PhotocraftApp, empty: &str) {
    if clicked(&Button::new(tl!("Select Subject")).show(ui)) {
        let r = app.run("select.subject", json!({}));
        crate::retouch_ui::report_smart_result(app, r, empty);
    }
}

/// Quick Selection's options.
pub(super) fn quick_selection(ui: &mut egui::Ui, app: &mut PhotocraftApp) -> Option<RangeAlert> {
    let o = &mut app.ui.tool_options;
    check(ui, tl!("Sample All Layers"), &mut o.sample_all.quick_selection, true, None);
    check(ui, tl!("Enhance Edge"), &mut o.enhance_edge, true, None);
    hint(ui, crate::i18n::fmt(tl!("{key} to subtract"), &[("key", &crate::shortcuts::pretty("Alt"))]));
    Divider::vertical().show(ui);
    select_subject(ui, app, tl!("No subject found — try Object Selection or the Lasso"));
    Divider::vertical().show(ui);
    crate::panels::options_bar::selection::select_and_mask(ui);
    None
}

/// Object Selection's options: the selection operation (#12853), Object Finder, Sample All Layers
/// and Hard Edge.
pub(super) fn object_selection(ui: &mut egui::Ui, app: &mut PhotocraftApp) -> Option<RangeAlert> {
    crate::panels::options_bar::selection::operation_segments(app, ui, true);
    Divider::vertical().show(ui);
    let o = &mut app.ui.tool_options;
    check(ui, tl!("Object Finder"), &mut o.object_finder, true, None);
    check(ui, tl!("Sample All Layers"), &mut o.sample_all.object_selection, true, None);
    check(ui, tl!("Hard Edge"), &mut o.object_hard_edge, true, None);
    hint(ui, tl!("Drag a rectangle around the object"));
    Divider::vertical().show(ui);
    select_subject(ui, app, tl!("No subject found — try a rectangle around the object"));
    None
}

/// SAM 3.1 Selection's options: operation segments, Sample All Layers, Prompt, Threshold, Refine,
/// Individual Masks, and the Detect button.
pub(super) fn sam3_select(ui: &mut egui::Ui, app: &mut PhotocraftApp) -> Option<RangeAlert> {
    crate::panels::options_bar::selection::operation_segments(app, ui, true);
    Divider::vertical().show(ui);
    let o = &mut app.ui.tool_options;
    check(ui, tl!("Sample All Layers"), &mut o.sample_all.sam3_select, true, None);
    Divider::vertical().show(ui);
    TextField::new(Label::from(tl!("Prompt")), Bind::bound(&mut o.sam3_prompt)).show(ui);
    let mut threshold = o.sam3_threshold;
    let alert1 = number(tl!("Threshold"), &mut threshold, 0.0..=1.0).show(ui);
    o.sam3_threshold = threshold;
    let mut refine = o.sam3_refine as f32;
    let alert2 = number(tl!("Refine"), &mut refine, 0.0..=5.0).whole().show(ui);
    o.sam3_refine = refine.round().clamp(0.0, 5.0) as u8;
    check(ui, tl!("Individual Masks"), &mut o.sam3_individual, true, None);
    Divider::vertical().show(ui);
    let can_detect = !o.sam3_prompt.trim().is_empty();
    if clicked(&Button::new(tl!("Detect")).enabled(can_detect).show(ui)) {
        let prompt = o.sam3_prompt.clone();
        let threshold = o.sam3_threshold;
        let refine = o.sam3_refine;
        let individual = o.sam3_individual;
        let sample_all = o.sample_all.sam3_select;
        let mode = crate::canvas::tool_feedback::mode_name(crate::canvas::tool_feedback::selection_op(
            Tool::Sam3Select,
            app.ui.selection_mode,
            craft::keymap::Mods::NONE,
            crate::canvas::tool_feedback::has_selection(app),
        ));
        let params = json!({
            "prompt": prompt,
            "threshold": threshold,
            "refine": refine,
            "individual": individual,
            "sampleAllLayers": sample_all,
            "mode": mode,
        });
        let r = app.run("select.sam3", params);
        crate::retouch_ui::report_smart_result(app, r, tl!("Nothing matched the prompt"));
    }
    hint(ui, tl!("or click, or drag a box"));
    alert1.out_of_range.or(alert2.out_of_range)
}
