use super::defaults::apply;
use super::*;

/// Font families: bundled, system, and those the host serves (`photocraft_text::served`, fetched
/// when picked). Cached until the shared engine registers fonts or the served catalog changes.
pub fn families() -> Arc<Vec<String>> {
    type Cached = Option<((u64, u64), Arc<Vec<String>>)>;
    static FAMILIES: std::sync::Mutex<Cached> = std::sync::Mutex::new(None);
    let key = (photocraft_text::fonts::shared_generation(), photocraft_text::served::generation());
    let mut cached = FAMILIES.lock().unwrap_or_else(PoisonError::into_inner);
    if let Some((k, v)) = cached.as_ref()
        && *k == key
    {
        return v.clone();
    }
    let mut v = photocraft_text::shared().lock().map(|mut e| e.fonts.families()).unwrap_or_default();
    for f in photocraft_text::served::families() {
        if !v.contains(&f) {
            v.push(f);
        }
    }
    v.sort_by_key(|s| s.to_lowercase());
    let v = Arc::new(v);
    *cached = Some((key, v.clone()));
    v
}

/// Where [`sync_font_names`] leaves the font menus' localized labels for this frame.
pub(super) fn font_labels_id() -> egui::Id {
    egui::Id::unique("photocraft-font-menu-localized-names")
}

/// Preferences ▸ Type ▸ Show Font Names in English: when off, font menus list a family that has a
/// localized (native-language) name by that name, as Photoshop does ("MS Gothic" reads
/// "ＭＳ ゴシック"). Called once a frame; the menus read the labels from the context.
pub fn sync_font_names(ctx: &egui::Context, show_in_english: bool) {
    let labels = (!show_in_english).then(localized_families);
    ctx.data_mut(|d| {
        if let Some(labels) = labels {
            d.insert_temp(font_labels_id(), labels);
        } else {
            d.remove::<Arc<HashMap<String, String>>>(font_labels_id());
        }
    });
}

/// The fonts' localized family names, keyed by family. Cached until the shared engine registers fonts.
fn localized_families() -> Arc<HashMap<String, String>> {
    type Cached = Option<(u64, Arc<HashMap<String, String>>)>;
    static LOCALIZED: std::sync::Mutex<Cached> = std::sync::Mutex::new(None);
    let key = photocraft_text::fonts::shared_generation();
    let mut cached = LOCALIZED.lock().unwrap_or_else(PoisonError::into_inner);
    if let Some((k, v)) = cached.as_ref()
        && *k == key
    {
        return v.clone();
    }
    let v = Arc::new(photocraft_text::shared().lock().map(|mut e| e.fonts.localized_family_names()).unwrap_or_default());
    *cached = Some((key, v.clone()));
    v
}

/// The font menu's rows: each family with the label it shows (its localized name when `labels`
/// has one, else the family), matching `query` (lowercase) by either name. With localized labels
/// the rows sort by label, so the menu stays alphabetical as shown. As in Photoshop, recently
/// used families also head the menu in their own group, most recent first (Preferences ▸ Type ▸
/// Recent Fonts), and keep their alphabetical place below it; a query shows each match once.
/// Returns the rows and how many of them form the recent group.
pub(super) fn font_menu_rows<'a>(
    families: &'a [String],
    labels: Option<&'a HashMap<String, String>>,
    query: &str,
    recents: &[String],
) -> (Vec<(&'a String, &'a str)>, usize) {
    let label = |f: &'a String| labels.and_then(|l| l.get(f)).map_or(f.as_str(), String::as_str);
    let mut rows: Vec<(&String, &str)> = families
        .iter()
        .map(|f| (f, label(f)))
        .filter(|(f, l)| query.is_empty() || f.to_lowercase().contains(query) || l.to_lowercase().contains(query))
        .collect();
    if labels.is_some_and(|l| !l.is_empty()) {
        rows.sort_by_cached_key(|(_, l)| l.to_lowercase());
    }
    if !query.is_empty() {
        return (rows, 0);
    }
    let recent: Vec<(&String, &str)> = recents.iter().filter_map(|r| rows.iter().find(|(f, _)| *f == r).copied()).collect();
    let n = recent.len();
    (recent.into_iter().chain(rows).collect(), n)
}

fn weight_name(w: f32) -> &'static str {
    match w.round() as i32 {
        ..=150 => "Thin",
        151..=250 => "ExtraLight",
        251..=350 => "Light",
        351..=450 => "Regular",
        451..=550 => "Medium",
        551..=650 => "SemiBold",
        651..=750 => "Bold",
        751..=850 => "ExtraBold",
        _ => "Black",
    }
}

/// Style names available for a family: each face's actual OpenType subfamily (including numeric
/// and nonstandard ones), plus the standard weights a variable face's `wght` axis covers.
pub fn styles(family: &str) -> Vec<String> {
    let faces = photocraft_text::shared().lock().unwrap_or_else(|e| e.into_inner()).fonts.faces(family);
    let v = style_names(&faces);
    if v.is_empty() { vec!["Regular".into()] } else { v }
}

/// [`styles`] of these faces, lightest first. A variable face reports only its default instance
/// (Montserrat's is Thin), yet the layout draws any weight its `wght` axis covers, so the standard
/// weights in that range are listed too.
pub(super) fn style_names(faces: &[photocraft_text::FaceInfo]) -> Vec<String> {
    let label = |weight: f32, italic: bool| {
        let name = match (weight_name(weight), italic) {
            ("Regular", true) => "Italic".to_string(),
            (w, true) => format!("{w} Italic"),
            (w, false) => w.to_string(),
        };
        (weight.round() as i32, italic, name)
    };
    let mut v: Vec<(i32, bool, String)> = faces.iter().map(|f| (f.weight.round() as i32, f.italic, f.style.clone())).collect();
    for f in faces {
        if let Some((_, min, _, max)) = f.axes.iter().find(|a| a.0 == "wght") {
            v.extend((100..=900).step_by(100).map(|w| w as f32).filter(|w| (*min..=*max).contains(w)).map(|w| label(w, f.italic)));
        }
    }
    v.sort();
    let mut names: Vec<String> = Vec::new();
    for (_, _, name) in v {
        if !names.contains(&name) {
            names.push(name);
        }
    }
    names
}

/// A variable-font axis: (tag, min, default, max), as [`photocraft_text::FaceInfo::axes`].
pub(crate) type Axis = (String, f32, f32, f32);

/// The axes the Character section shows as sliders (#2985): those of the faces matching the
/// style's italic, else of any face. Optical size has none: it follows the type size.
pub(crate) fn slider_axes(faces: &[photocraft_text::FaceInfo], italic: bool) -> Vec<Axis> {
    let pick = |want: Option<bool>| {
        let mut v: Vec<Axis> = Vec::new();
        for f in faces.iter().filter(|f| want.is_none_or(|w| f.italic == w)) {
            for a in &f.axes {
                let usable = a.0 != "opsz" && a.1.is_finite() && a.3.is_finite() && a.1 < a.3;
                if usable && !v.iter().any(|b| b.0 == a.0) {
                    v.push(a.clone());
                }
            }
        }
        v
    };
    let v = pick(Some(italic));
    if v.is_empty() { pick(None) } else { v }
}

/// The value an axis slider shows: the style's own value for the axis, else what the layout
/// draws (the style's weight on `wght`, the axis default otherwise), within the axis range.
pub(crate) fn axis_value(c: &photocraft_doc::text::CharStyle, (tag, min, default, max): &Axis) -> f32 {
    let v = c.variations.iter().find(|v| &v.axis == tag).map(|v| v.value).unwrap_or_else(|| match tag.as_str() {
        "wght" => f32::from(c.weight),
        "ital" if c.italic => *max,
        _ => *default,
    });
    let d = default.clamp(*min, *max);
    if v.is_finite() { v.clamp(*min, *max) } else { d }
}

/// Slider label for an axis: the registered axes by name, others by their tag (as the font
/// names them in its `fvar` table, e.g. `GRAD`).
fn axis_label(tag: &str) -> String {
    match tag {
        "wght" => tl!("Weight").to_string(),
        "wdth" => tl!("Width").to_string(),
        "slnt" => tl!("Slant").to_string(),
        "ital" => tl!("Italic").to_string(),
        _ => tag.to_string(),
    }
}

/// One slider row per variable-font axis of the style's font, below the font style (#2985).
pub(super) fn axis_sliders(app: &mut PhotocraftApp, ui: &mut egui::Ui, c: &photocraft_doc::text::CharStyle) {
    // Drawn every frame: no sliders until the background font scan is done (#3021).
    let Some(engine) = photocraft_text::try_shared() else { return };
    let axes = {
        let faces = engine.lock().unwrap_or_else(PoisonError::into_inner).fonts.faces(&c.font_family);
        slider_axes(&faces, c.italic)
    };
    let t = crate::theme::Tokens::get(ui.ctx());
    for axis in &axes {
        let (min, max) = (axis.1, axis.3);
        let mut v = axis_value(c, axis);
        let mut changed = false;
        ui.push_id(("type-axis", &axis.0), |ui| {
            ui.horizontal(|ui| {
                let axis_name = axis_label(&axis.0);
                ui.label(egui::RichText::new(&axis_name).color(t.text_dim));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    changed |= crate::widgets::value_field(ui, &mut v, min..=max, "", 74.0).accessible_name(axis_name).changed();
                });
            });
            changed |= crate::widgets::slider(ui, &mut v, min..=max, None).changed();
        });
        ui.add_space(crate::theme::ROW_GAP);
        if changed {
            // Whole units on wide axes (weight 100–900), hundredths on narrow ones (ital 0–1).
            let v = if max - min >= 10.0 { v.round() } else { (v * 100.0).round() / 100.0 };
            let mut m = serde_json::Map::new();
            m.insert(axis.0.clone(), json!(v));
            apply(app, ui.ctx(), json!({ "variations": m }));
        }
    }
}

/// PSD text can identify a face only by its PostScript name. Show its real subfamily too.
pub(crate) fn selected_style(style: &photocraft_doc::text::CharStyle) -> String {
    // Drawn every frame: while the font scan runs, the stored style name stands in.
    if let Some(ps) = &style.postscript_name
        && let Some(engine) = photocraft_text::try_shared()
    {
        let faces = engine.lock().unwrap_or_else(|e| e.into_inner()).fonts.faces(&style.font_family);
        if let Some(face) = faces.into_iter().find(|f| f.postscript_name.as_ref() == Some(ps)) {
            return face.style;
        }
    }
    if style.font_style.is_empty() { if style.italic { "Italic".into() } else { "Regular".into() } } else { style.font_style.clone() }
}

/// Localized style label for the dropdown. The raw string stays the engine's `fontStyle` key;
/// the words (weights and "Italic") are translated in the `fontWeight` context, where "Light"
/// means a weight — unlike the Camera Raw "Light" section.
pub fn style_label(style: &str) -> String {
    style.split(' ').map(|w| crate::i18n::tr_ctx(crate::i18n::current(), "fontWeight", w)).collect::<Vec<_>>().join(" ")
}

/// Starts scanning the installed fonts on a background thread (desktop start-up), so choosing
/// the Type tool doesn't stall the UI while they are read (#3021). Does nothing on the web.
pub fn warm_fonts() {
    photocraft_text::warm_shared();
}

/// Whether the font database is ready. The installed fonts are scanned on a background thread
/// from launch (`photocraft_text::warm_shared`); until that finishes, font menus draw this
/// disabled "Loading fonts…" box `width` wide instead of blocking the UI, and a repaint picks the
/// menus up once the scan is done.
pub(crate) fn fonts_ready(ui: &mut egui::Ui, width: f32) -> bool {
    if photocraft_text::try_shared().is_some() {
        return true;
    }
    let label = tl!("Loading fonts…").to_string();
    ui.add_enabled_ui(false, |ui| crate::widgets::dropdown(ui, "fonts-loading", tl!("Font"), &mut label.clone(), &[(label.clone(), label.as_str())], width));
    ui.ctx().request_repaint_after(std::time::Duration::from_millis(100));
    false
}

/// Searchable font-family combo box.
pub(super) fn font_picker(ui: &mut egui::Ui, current: &mut String, width: f32, preview: photocraft_engine::prefs::FontPreview, recents: &[String]) -> bool {
    family_picker_in(ui, "type-font", current, width, &families(), preview, recents)
}

/// Maximum height of the font menu.
pub(super) const FONT_MENU_HEIGHT: f32 = 460.0;

/// [`font_picker`] over a given family list (tests pass their own).
#[cfg(test)]
pub(super) fn font_picker_in(
    ui: &mut egui::Ui,
    current: &mut String,
    width: f32,
    families: &[String],
    preview: photocraft_engine::prefs::FontPreview,
    recents: &[String],
) -> bool {
    family_picker_in(ui, "type-font", current, width, families, preview, recents)
}

/// The row Enter and ⌘/Ctrl+Enter apply to a pending family query: a full name wins over a
/// substring match (including localized family names), then the highlighted row, then the first.
pub(super) fn accept_row(rows: &[(&String, &str)], text: &str, selected: Option<usize>) -> Option<usize> {
    let exact = text.to_lowercase();
    rows.iter().position(|(f, shown)| f.to_lowercase() == exact || shown.to_lowercase() == exact).or(selected).or_else(|| (!rows.is_empty()).then_some(0))
}

/// Transient input belongs to one field, independently of the applied family and its peers.
#[derive(Clone)]
pub(super) struct FontFieldEdit {
    pub(super) text: String,
    pub(super) query: String,
    pub(super) current: String,
    pub(super) recents: Vec<String>,
    pub(super) row: Option<usize>,
    pub(super) opening: bool,
    pub(super) accepted: bool,
    pub(super) composing: bool,
    pub(super) menu_height: Option<f32>,
    /// The current family still has to be scrolled into view after the menu opened.
    pub(super) reveal: bool,
}

/// Shared by the Type options, Character, style and Glyphs panels. Selection changes still
/// flow through each caller's existing engine command.
pub(crate) fn family_picker_in(
    ui: &mut egui::Ui,
    salt: &str,
    current: &mut String,
    width: f32,
    families: &[String],
    preview: photocraft_engine::prefs::FontPreview,
    recents: &[String],
) -> bool {
    let field = ui.make_persistent_id(egui::IdSalt::new(salt));
    let popup = field.with("popup");
    let edit_id = field.with("edit");
    let was_open = egui::Popup::is_id_open(ui.ctx(), popup);
    let had_focus = ui.memory(|m| m.has_focus(field));
    let labels: Option<Arc<HashMap<String, String>>> = ui.ctx().data(|d| d.get_temp(font_labels_id()));
    let labels = labels.as_deref();
    let label = |f: &str| labels.and_then(|l| l.get(f)).cloned().unwrap_or_else(|| f.to_string());
    let mut edit =
        ui.data(|d| d.get_temp::<FontFieldEdit>(edit_id)).filter(|e| e.current == *current && (was_open || had_focus)).unwrap_or_else(|| FontFieldEdit {
            text: label(current),
            query: String::new(),
            current: current.clone(),
            recents: recents.to_vec(),
            row: None,
            opening: false,
            accepted: false,
            composing: false,
            menu_height: None,
            reveal: false,
        });
    // Take navigation before TextEdit interprets it, and before the canvas can see it.
    let owns_keys = ui.is_enabled() && (had_focus || was_open);
    if had_focus {
        ui.memory_mut(|m| m.set_focus_lock_filter(field, egui::EventFilter { vertical_arrows: true, ..Default::default() }));
    }
    // IME confirmation/navigation belongs to composition, even when its current spelling
    // happens to be an exact font name. A separate Enter accepts the completed name.
    let composing = edit.composing || ui.input(|i| i.events.iter().any(|e| matches!(e, egui::Event::Ime(_))));
    if owns_keys {
        ui.input(|i| {
            for event in &i.events {
                match event {
                    egui::Event::Ime(egui::ImeEvent::Preedit { text, .. }) => edit.composing = !text.is_empty(),
                    egui::Event::Ime(egui::ImeEvent::Commit(_)) => edit.composing = false,
                    _ => {}
                }
            }
        });
    }
    let key = |k| owns_keys && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, k));
    let navigation_down = key(egui::Key::ArrowDown);
    let navigation_up = key(egui::Key::ArrowUp);
    let down = navigation_down && !composing;
    let up = navigation_up && !composing;
    let enter = key(egui::Key::Enter) && !composing;
    let escape = key(egui::Key::Escape) && !composing;
    let tab = owns_keys && ui.input(|i| i.key_pressed(egui::Key::Tab));
    if navigation_down || navigation_up {
        ui.memory_mut(|m| m.move_focus(egui::FocusDirection::None));
    }
    if edit.opening {
        ui.memory_mut(|m| m.request_focus(field));
    }
    let (input, rect) = crate::widgets::text_field_with_trailing(ui, field, &mut edit.text, tl!("Font"), width, crate::widgets::POPUP_ARROW_W);
    let arrow_rect =
        egui::Rect::from_min_max(egui::pos2(rect.right() - crate::widgets::POPUP_ARROW_W - 2.0, rect.top() + 2.0), rect.max - egui::vec2(2.0, 2.0));
    let arrow = ui.interact(arrow_rect, field.with("arrow"), egui::Sense::click());
    arrow.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, ui.is_enabled(), tl!("Font")));
    crate::widgets::chevron_icon(ui, arrow_rect, ui.style().interact(&arrow), was_open);
    // The arrow owns press-drag-release; dragging in the name remains normal text selection.
    // Its menu is anchored to the full control and keeps the existing stable popup id.
    let mut anchor = arrow.clone();
    anchor.id = field;
    anchor.rect = rect;
    anchor.interact_rect = rect;
    if arrow.clicked() && !ui.input(|i| i.pointer.any_click()) {
        egui::Popup::toggle_id(ui.ctx(), popup);
    }
    let mut select_all = input.gained_focus() || (!was_open && input.clicked());
    if (input.gained_focus() && !edit.accepted) || (!was_open && input.clicked()) || input.changed() {
        egui::Popup::open_id(ui.ctx(), popup);
    }
    if input.changed() {
        edit.query = edit.text.to_lowercase();
        edit.row = None;
        edit.accepted = false;
        select_all = false;
    }
    if escape || tab {
        egui::Popup::close_id(ui.ctx(), popup);
        if escape {
            ui.memory_mut(|m| m.surrender_focus(field));
        }
        edit.opening = false;
    }
    let mut changed = false;
    let mut picked = false;
    // The name is outside the popup: clicking it to move the caret keeps the list open.
    let close = if input.contains_pointer() { egui::PopupCloseBehavior::IgnoreClicks } else { egui::PopupCloseBehavior::CloseOnClickOutside };
    let response = crate::press_menu::show_with_close_behavior(&anchor, close, |ui, press| {
        // Rows normally size the popup. With no matches, keep a readable width and omit
        // the cached scroll viewport so an empty list cannot become a tall, narrow bar.
        ui.set_min_width(width.max(330.0));
        // Each opening reveals the current family once, on its first laid-out pass.
        if !was_open {
            edit.reveal = true;
        }
        let revealing = edit.reveal && !ui.is_sizing_pass();
        if revealing {
            edit.reveal = false;
        }
        let (rows, n_recent) = font_menu_rows(families, labels, &edit.query, &edit.recents);
        if rows.is_empty() {
            ui.weak(tl!("No matching fonts"));
            return None;
        }
        let mut selected = edit.row.filter(|&i| rows.get(i).is_some_and(|(f, _)| *f == current)).or_else(|| rows.iter().position(|(f, _)| *f == current));
        // The menu opens on the current family, as Photoshop's does: with the recent group at the
        // top of the view when the marked row fits below it, else the marked row itself, at the
        // top of the view when it is not a recent one. Applied on the first laid-out pass (the
        // popup's first pass only measures), whether or not a row matches, so a missing font or
        // a filtered list never scrolls the menu later.
        let reveal = if revealing { selected } else { None };
        let reveal_from = reveal.map(|i| if i < n_recent { 0 } else { i });
        let mut stepped = None;
        if down || up {
            let next = match selected {
                Some(i) if down => i.saturating_add(1).min(rows.len().saturating_sub(1)),
                Some(i) => i.saturating_sub(1),
                None if down => 0,
                None => rows.len().saturating_sub(1),
            };
            if let Some((f, shown)) = rows.get(next) {
                changed = **f != *current;
                current.clone_from(f);
                photocraft_text::served::request(f);
                edit.text = (*shown).to_string();
                edit.row = Some(next);
                selected = Some(next);
                stepped = Some(next);
                select_all = true;
            }
        }
        if enter && !edit.text.trim().is_empty() {
            // A full name wins over a substring match (including localized family names).
            let accept = accept_row(&rows, &edit.text, selected);
            if let Some((f, _)) = accept.and_then(|i| rows.get(i)) {
                // Acceptance is an application to the selected characters, even when a
                // mixed selection's representative already has this family (as row clicks do).
                changed = true;
                current.clone_from(f);
                photocraft_text::served::request(f);
                picked = true;
                ui.close();
            }
        }
        let preview = crate::font_preview::sample(preview);
        let row_h = preview.map_or(28.0, |(_, [_, h])| (h + 4.0).max(28.0));
        let mut hovered: Option<String> = None;
        let reveal = if stepped.is_some() { None } else { reveal };
        let mut reveal_top: Option<egui::Rect> = None;
        let mut scroll = egui::ScrollArea::vertical().max_height(FONT_MENU_HEIGHT);
        if let Some(height) = edit.menu_height {
            scroll = scroll.max_height(height).min_scrolled_height(height).auto_shrink([true, false]);
        }
        let list = scroll.show(ui, |ui| {
            for (i, (f, shown)) in rows.into_iter().enumerate() {
                if i == n_recent && n_recent > 0 {
                    ui.separator();
                }
                // No hover stroke: growing rows would make the list jump (#7406).
                let mut button = egui::Button::selectable(selected == Some(i), shown).truncate().stroke(egui::Stroke::NONE);
                if let Some((size, [_, h])) = preview {
                    button = button.right_text(egui::Atom::custom(
                        egui::IdSalt::new(("font-sample", f)),
                        egui::vec2(crate::font_preview::slot_width(ui.ctx(), f, size), h),
                    ));
                }
                let response = ui.add_sized([330.0, row_h], button);
                if stepped == Some(i) {
                    response.scroll_to_me(Some(egui::Align::Center));
                } else if reveal.is_some() {
                    if reveal_from == Some(i) {
                        reveal_top = Some(response.rect);
                    }
                    if reveal == Some(i) {
                        // Jump, not glide: the menu opens with its current family in view. Arrow
                        // steps above keep the animation. A recent row too far below the top of
                        // the group (large previews, many recents) is brought up to the bottom.
                        let top = reveal_top.unwrap_or(response.rect);
                        let (rect, align) = if response.rect.bottom() - top.top() <= ui.clip_rect().height() {
                            (top, egui::Align::Min)
                        } else {
                            (response.rect, egui::Align::Max)
                        };
                        ui.scroll_to_rect_animation(rect, Some(align), egui::style::ScrollAnimation::none());
                    }
                }
                if let Some((size, _)) = preview
                    && ui.is_rect_visible(response.rect)
                {
                    crate::font_preview::paint(ui, f, response.rect, size);
                }
                if response.hovered() {
                    hovered = Some(f.clone());
                }
                if press.chosen(ui, &response) {
                    changed = true;
                    current.clone_from(f);
                    photocraft_text::served::request(f);
                    picked = true;
                    ui.close();
                }
            }
        });
        // Freeze the measured viewport, rather than the popup's padded minimum height:
        // feeding the latter back into the scroll area makes clearing a query grow it (#1369).
        edit.menu_height.get_or_insert(list.inner_rect.height());
        if picked { None } else { hovered }
    });
    let open = egui::Popup::is_id_open(ui.ctx(), popup);
    if !was_open && open {
        edit.recents = recents.to_vec();
        edit.opening = arrow.is_pointer_button_down_on() && ui.input(|i| i.pointer.primary_down());
        ui.memory_mut(|m| m.request_focus(field));
        select_all = !input.changed();
    }
    // A held opening arrow can regain focus on release; keep the name selected through it.
    if edit.opening {
        ui.memory_mut(|m| m.request_focus(field));
        select_all |= !input.changed();
        edit.opening = ui.input(|i| i.pointer.primary_down());
    }
    if !open && !escape && ui.is_enabled() {
        let notches = if had_focus { f32::from(u8::from(up)) - f32::from(u8::from(down)) } else { 0.0 };
        let hovered = input.hovered() || arrow.hovered();
        if hovered {
            ui.input_mut(|i| i.smooth_scroll_delta = egui::Vec2::ZERO);
        }
        let notches = notches + crate::widgets::wheel_notches_of(ui, field.with("family-wheel"), hovered, 1.0);
        // Closed-field navigation stays alphabetical, independent of recent picks (#3350, #7407).
        let (rows, _) = font_menu_rows(families, labels, "", &[]);
        if let Some(i) = rows.iter().position(|(f, _)| *f == current)
            && let Some(n) = wheel_family_index(i, rows.len(), notches)
            && let Some((f, _)) = rows.get(n)
            && **f != *current
        {
            current.clone_from(f);
            photocraft_text::served::request(f);
            changed = true;
            select_all |= had_focus;
        }
    }
    if picked || (!open && (was_open || changed || escape)) {
        edit.text = label(current);
        edit.query.clear();
        edit.row = None;
        edit.accepted = true;
        edit.opening = false;
        edit.composing = false;
    }
    if picked {
        ui.memory_mut(|m| m.request_focus(field));
        select_all = true;
    }
    if select_all && !escape {
        crate::field_tab::select_all(ui.ctx(), field, &edit.text);
    }
    edit.current.clone_from(current);
    if open || ui.memory(|m| m.has_focus(field)) {
        ui.data_mut(|d| d.insert_temp(edit_id, edit));
    } else {
        ui.data_mut(|d| d.remove::<FontFieldEdit>(edit_id));
    }
    if salt == "type-font" && (was_open || open || response.is_some()) {
        // Only this popup's owner clears its preview; a closed peer must leave it alone (#5152).
        let hover_id = egui::Id::unique("type-font-hover");
        if let Some(f) = response.flatten().filter(|_| open) {
            ui.ctx().data_mut(|d| d.insert_temp(hover_id, f));
        } else {
            ui.ctx().data_mut(|d| d.remove::<String>(hover_id));
        }
    }
    changed
}

/// The row `notches` wheel notches from row `index` of `len` (down, negative, is the next one),
/// stopping at the ends; none when the wheel did not move.
pub(super) fn wheel_family_index(index: usize, len: usize, notches: f32) -> Option<usize> {
    let steps = notches.abs().min(len as f32) as usize;
    if steps == 0 || len == 0 {
        return None;
    }
    let last = len - 1;
    Some(if notches < 0.0 { index.saturating_add(steps).min(last) } else { index.saturating_sub(steps) })
}
