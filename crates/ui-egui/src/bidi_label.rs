//! RTL label regression compatibility for native egui paragraph ordering.
//! Keep logical text to prevent a second bidi pre-pass from reversing words twice.

use std::borrow::Cow;

/// Does `text` contain a character from a right-to-left script?
pub fn has_rtl(text: &str) -> bool {
    text.chars().any(rtl_script)
}

/// Characters egui shapes right to left: those of the right-to-left scripts (HarfRust guesses a
/// run's direction from its script), including their marks and digits. The joiners count too,
/// so a Persian word with a ZWNJ stays one stretch.
fn rtl_script(c: char) -> bool {
    matches!(
        c as u32,
        0x0590..=0x05FF // Hebrew
            | 0x0600..=0x06FF // Arabic (letters, marks, Arabic-Indic and Persian digits)
            | 0x0700..=0x074F // Syriac
            | 0x0750..=0x077F // Arabic Supplement
            | 0x0780..=0x07BF // Thaana
            | 0x07C0..=0x07FF // NKo
            | 0x0860..=0x08FF // Syriac Supplement, Arabic Extended-A/B
            | 0xFB1D..=0xFB4F // Hebrew presentation forms
            | 0xFB50..=0xFDFF // Arabic Presentation Forms-A
            | 0xFE70..=0xFEFF // Arabic Presentation Forms-B
            | 0x10800..=0x10FFF // Historic RTL scripts
            | 0x1E800..=0x1EFFF // Mende Kikakui, Adlam, Arabic mathematical symbols
    ) || matches!(c, '\u{200C}' | '\u{200D}')
}

/// `text` arranged so that egui draws it in visual order. Text without right-to-left characters
/// is returned unchanged.
/// Native egui 0.37 at the pinned revision owns UAX #9 ordering. A second pre-pass would
/// reverse multiword RTL labels twice. Keep the logical text for shaping and accessibility.
pub fn for_egui(text: &str) -> Cow<'_, str> {
    Cow::Borrowed(text)
}

#[cfg(test)]
#[path = "bidi_label_tests.rs"]
mod tests;
