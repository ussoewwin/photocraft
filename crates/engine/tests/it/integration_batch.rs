//! Cross-PR acceptance checks for the merged parity batch: each test pins one fix through the
//! public command surface, so a bad merge between the PRs shows up here.
use photocraft_engine::Session;
use serde_json::json;
use std::time::{Duration, Instant};

fn session(width: u32, height: u32) -> Session {
    let mut session = Session::new();
    session.execute("file.new", json!({"width": width, "height": height})).unwrap();
    session
}

#[test]
fn a_new_fill_layer_consumes_the_selection_it_took_as_mask() {
    let mut s = session(32, 24);
    s.execute("select.rect", json!({"x": 2, "y": 2, "width": 10, "height": 8})).unwrap();
    s.execute("layer.newFillLayer.solidColor", json!({"color": "#4080c0"})).unwrap();
    assert!(s.active().unwrap().doc.selection.is_none());
}

#[test]
fn define_pattern_accepts_a_canvas_wider_than_the_old_4000_px_cap() {
    let mut s = session(4500, 8);
    s.execute("edit.definePattern", json!({"name": "Wide"})).unwrap();
    let mut s = session(8001, 8);
    assert!(s.execute("edit.definePattern", json!({"name": "TooWide"})).is_err());
}

#[test]
fn a_header_declaring_more_pixels_than_the_file_holds_is_refused_quickly() {
    // BMP: 30000 x 30000 at 24 bpp declared in a 54-byte file.
    let mut bmp = vec![0u8; 54];
    bmp[0..2].copy_from_slice(b"BM");
    bmp[10..14].copy_from_slice(&54u32.to_le_bytes());
    bmp[14..18].copy_from_slice(&40u32.to_le_bytes());
    bmp[18..22].copy_from_slice(&30000i32.to_le_bytes());
    bmp[22..26].copy_from_slice(&30000i32.to_le_bytes());
    bmp[26..28].copy_from_slice(&1u16.to_le_bytes());
    bmp[28..30].copy_from_slice(&24u16.to_le_bytes());
    let start = Instant::now();
    assert!(photocraft_codecs::decode(&bmp).is_err());
    assert!(start.elapsed() < Duration::from_secs(5));
}


#[test]
fn gaussian_blur_accepts_radii_below_one_without_failing() {
    for radius in [0.05, 0.1, 0.3, 0.7, 1.0] {
        let mut s = session(24, 16);
        s.execute("filter.blur.gaussianBlur", json!({"radius": radius})).unwrap();
    }
}
