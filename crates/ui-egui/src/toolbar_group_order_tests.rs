//! Shift+letter steps a key's tool group in the toolbar's slot order.

use egui::{Key, Modifiers};
use serde_json::json;

use crate::PhotocraftApp;
use crate::state::Tool;

const HEALING: [Tool; 6] = [Tool::SpotHealing, Tool::Remove, Tool::Healing, Tool::Patch, Tool::ContentAwareMove, Tool::RedEye];
const BRUSH: [Tool; 5] = [Tool::Brush, Tool::Pencil, Tool::ColorReplacement, Tool::MixerBrush, Tool::LamaCleaner];

/// Every tool the engine can select has a slot on the toolbar. A tool with no slot is
/// unreachable in the app even though its command exists: the LaMa Cleaner and SAM Selection
/// tools were lost that way when a stored toolbar layout arrived from upstream and named
/// neither of them.
#[test]
fn every_selectable_tool_has_a_toolbar_slot() {
    let sections: Vec<Tool> = crate::panels::toolbar::TOOL_SECTIONS.iter().flat_map(|section| section.iter()).flat_map(|slot| slot.iter()).copied().collect();
    for (id, label, _) in photocraft_engine::shell_commands::TOOLS {
        let Some(name) = id.strip_prefix("tools.select.") else { continue };
        let Some(tool) = Tool::from_name(name) else {
            panic!("{label}: the engine lists {id} but the toolbar has no Tool for it");
        };
        assert!(sections.contains(&tool), "{label} ({name}) has no slot in TOOL_SECTIONS");
    }
}

fn app() -> PhotocraftApp {
    let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), crate::Services::default());
    app.run("file.new", json!({"width": 20, "height": 10})).unwrap();
    app.run("layer.new.layer", json!({})).unwrap();
    app
}

fn press(app: &mut PhotocraftApp, key: Key, modifiers: Modifiers) {
    crate::canvas::testing::key(app, key, modifiers);
}

fn shift_sequence(key: Key, steps: usize) -> Vec<Tool> {
    let mut app = app();
    app.ui.tool = Tool::Hand;
    (0..steps)
        .map(|_| {
            press(&mut app, key, Modifiers::SHIFT);
            app.ui.tool
        })
        .collect()
}

#[test]
fn shift_j_follows_the_reference_sequence() {
    let want: Vec<Tool> = HEALING.iter().chain(&HEALING[..1]).copied().collect();
    assert_eq!(shift_sequence(Key::J, 7), want);
}

#[test]
fn shift_b_follows_the_reference_sequence() {
    let want: Vec<Tool> = BRUSH.iter().chain(&BRUSH[..1]).copied().collect();
    assert_eq!(shift_sequence(Key::B, 6), want);
}
