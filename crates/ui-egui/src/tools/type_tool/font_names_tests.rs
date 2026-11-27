//! The one rule the font picker's English names follow: the preference, but never English in a
//! Japanese interface.

use super::fonts::english_font_names;

#[test]
fn the_japanese_interface_lists_japanese_names_whatever_the_preference_says() {
    for code in ["ja", "ja-JP", "ja_JP"] {
        assert!(!english_font_names(true, code), "{code}");
        assert!(!english_font_names(false, code), "{code}");
    }
}

#[test]
fn every_other_interface_follows_the_preference() {
    assert!(english_font_names(true, "en-US"));
    assert!(english_font_names(true, "de"));
    assert!(!english_font_names(false, "en-US"));
    assert!(!english_font_names(false, "fr-FR"));
}
