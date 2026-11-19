use craft::menu::{Labeled, MenuBarUi};

use super::PhotocraftApp;
use super::items;
use super::menu_gate;

/// Mnemonic letters of the top-level menus (File F, Edit E, …). A title takes its letter only when
/// no earlier title took it.
pub(super) const TOP_PREFERRED: [(&str, char); 10] = [
    ("File", 'F'),
    ("Edit", 'E'),
    ("Image", 'I'),
    ("Layer", 'L'),
    ("Type", 'Y'),
    ("Select", 'S'),
    ("Filter", 'T'),
    ("View", 'V'),
    ("Window", 'W'),
    ("Help", 'H'),
];

/// Draws the in-window menu bar over the one menu model. Picks are fired commands from
/// `FireSource::Menu`, run by the frame's command drain; the bar's keys wait while a dialog or the
/// shortcut recorder has the keyboard.
pub fn menu_bar(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    let (gate, _) = menu_gate(app);
    let ctx = ui.ctx().clone();
    // The bar is laid out in the title bar's leading region, which declares the width the bar's titles take
    // (`TitleBar::leading_width`): laid out at the share of the line the empty title strip leaves, the bar cut its
    // last titles (Window, Help) into the overflow menu. The bar reports the width its titles take, and the next
    // frame's region declares it, so every title keeps its place.
    let declared = app.menus_bar_width;
    let measured = {
        let model = items::model(app, &ctx);
        let english = english_of(model);
        let bar = MenuBarUi::new(&model.bar).english(&english).mnemonics(&TOP_PREFERRED).help_search(true);
        let out = bar.keyboard(!(gate.modal || gate.recording_shortcut)).width(declared).show(ui);
        out.titles_width
    };
    app.menus_bar_width = (measured > 0.0).then_some(measured);
}

/// The English name of a command or menu title of `model`, for the bar's and the palette's search.
fn english_of(model: &items::MenuModel) -> impl Fn(Labeled<'_>) -> Option<String> + '_ {
    move |labeled| match labeled {
        Labeled::Command(command) => model.english_label(command),
        Labeled::Title(title) => model.english_title(title),
        _ => None,
    }
}

/// The commands of the menu model's bar as the command palette lists them, with each path and English name. The bar
/// is laid out with the options the model was built with, so the palette and the bars agree.
pub(crate) fn help_entries(app: &mut PhotocraftApp, ctx: &egui::Context) -> Vec<craft::ui::menu::HelpEntry> {
    use craft::menu::{LayoutOptions, Target};
    let lang = crate::i18n::current();
    let model = items::model(app, ctx);
    let label = |s: &'static str| crate::i18n::tr(lang, s).to_string();
    let keys = |chord: &craft::keymap::Chord| craft::ui::runtime::commands::holder(ctx, chord);
    let options = LayoutOptions::new(crate::native_menu::APP_NAME).label(&label).keys(&keys);
    let entries = model.bar.help_entries(Target::current(), &options, Some(&english_of(model)));
    // A row no catalog translates (the theme names, which the framework names in English) already reads the
    // same in every language, so its label is its English name: a query in English still finds it.
    entries
        .into_iter()
        .map(|entry| match entry.english() {
            Some(_) => entry,
            None => {
                let label = entry.label().clone();
                entry.with_english(label)
            }
        })
        .collect()
}
