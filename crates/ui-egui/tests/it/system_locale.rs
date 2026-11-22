//! Use the actual platform UI-language API and the production (non-cfg(test)) Auto path.
#![cfg(not(target_arch = "wasm32"))]

use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use photocraft_ui_egui::{PhotocraftApp, i18n};

#[test]
fn native_first_launch_follows_os_ui_preferences() {
    let tags: Vec<String> = match std::env::var("PHOTOCRAFT_LOCALE").ok().filter(|tag| !tag.trim().is_empty()) {
        Some(tag) => vec![tag],
        None => sys_locale::get_locales().take(64).collect(),
    };
    let expected = tags
        .iter()
        .filter(|tag| tag.len() <= 128 && tag.trim().bytes().all(|c| c.is_ascii_alphanumeric() || matches!(c, b'-' | b'_' | b'.' | b'@')))
        .find_map(|tag| i18n::lang_from_tag(tag.trim()))
        .unwrap_or(i18n::Lang::EN);
    eprintln!("native UI-language preferences: {tags:?}; resolved: {}", expected.code());
    let mut h = Harness::builder().with_size(egui::vec2(1200.0, 800.0)).with_max_steps(64).build_eframe(|cc| {
        PhotocraftApp::setup_context(&cc.egui_ctx, Default::default());
        PhotocraftApp::with_context(cc.egui_ctx.clone(), photocraft_engine::Session::new(), Default::default())
    });
    h.run_steps(4);
    assert_eq!(h.state().session.prefs().interface.language, "auto");
    assert_eq!(i18n::current(), expected, "native language preferences: {tags:?}");
    // The menu the window draws is in the OS UI language: `ui.menu.list` reports the rows the
    // window shows, their top-level titles with the English catalogue's keys. With no document
    // open the home screen covers the canvas, but not the menu bar.
    let (req, _rx) = photocraft_ui_egui::ControlRequest::new("ui.menu.list", serde_json::json!({}));
    let mut ctx = h.ctx.clone();
    let reply = photocraft_ui_egui::control::handle(h.state_mut(), &ctx, &req);
    let rows = match reply {
        craft::automation::Reply::Done(v) => v,
        _ => panic!("ui.menu.list answers at once"),
    };
    let tops: Vec<String> = rows
        .get("result")
        .and_then(serde_json::Value::as_array)
        .map(|rows| {
            rows.iter()
                .filter(|row| row["path"].as_array().is_some_and(|p| p.len() == 1))
                .filter_map(|row| row["path"][0].as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();
    assert!(tops.iter().any(|t| t == "File"), "the first-launch menu must use the OS UI language ({:?}): {tops:?}", expected.code());
}
