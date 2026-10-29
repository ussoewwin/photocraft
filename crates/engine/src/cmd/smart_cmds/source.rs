use std::sync::PoisonError;

use super::{
    Arc, ColorMode, Document, Layer, LayerContent, LayerId, Metadata, Mutex, PixelFormat, Rect, Result, SampleType, Session, SmartObject, SmartSource, Surface,
    base_name, decode_source, json, other,
};

/// A linked file's bytes; `None` when it is missing or its path is not followed
/// ([`crate::doc_path::resolve`]).
///
/// The path is document data, not a request parameter, so every read rechecks it with the gate
/// of the session the work is done for: a filter, transform or refresh re-renders from here,
/// not only the contents commands that judge it up front. A refused path is handled like a
/// missing file: the layer keeps its last rendering.
fn read_file(path: &str) -> Option<Vec<u8>> {
    if path.is_empty() {
        return None;
    }
    crate::jobs::authorize_render_command("layer.smartObjects.updateModifiedContent", &json!({"path": path})).ok()?;
    photocraft_format::read_file(&crate::doc_path::resolve(path).ok()?).ok()
}

/// The source file of a smart object: embedded bytes, the PSD's embedded linked-layer data (PSD
/// import keeps the placed layer's uuid as the path), or a linked file on disk.
pub fn source_bytes(meta: &Metadata, src: &SmartSource) -> Option<(String, Arc<Vec<u8>>)> {
    embedded_source_bytes(meta, src).or_else(|| match src {
        SmartSource::Linked { path } => read_file(path).map(|b| (base_name(path), Arc::new(b))),
        SmartSource::Embedded { .. } => None,
    })
}

/// Memory-only resolution also covers Photoshop's embedded files addressed by a `Linked` UUID.
fn embedded_source_bytes(meta: &Metadata, src: &SmartSource) -> Option<(String, Arc<Vec<u8>>)> {
    match src {
        SmartSource::Embedded { file_name, bytes } => Some((file_name.clone(), bytes.clone())),
        SmartSource::Linked { path } => photocraft_io::linked::find_linked_file(meta, path).map(|f| (f.file_name, Arc::new(f.bytes))),
    }
}

/// The actual path is stored in the document, not in the command's params. Recheck it before
/// reading, reusing embedded PSD bytes rather than parsing their global blocks a second time.
pub(super) fn authorized_source_bytes(s: &Session, cmd: &str, meta: &Metadata, src: &SmartSource) -> Result<Option<(String, Arc<Vec<u8>>)>> {
    if let Some(source) = embedded_source_bytes(meta, src) {
        return Ok(Some(source));
    }
    let SmartSource::Linked { path } = src else { return Ok(None) };
    if let Some(gate) = s.authorize {
        gate(cmd, &json!({"path": path}))?;
    }
    Ok(read_file(path).map(|b| (base_name(path), Arc::new(b))))
}

/// Recheck a source's actual path before an untrusted command reads it: the path lives in the
/// document, not in the command's params. Embedded PSD UUIDs never reach the filesystem.
pub(super) fn authorize_source(s: &Session, cmd: &str, doc: &Document, src: &SmartSource) -> Result<()> {
    if let (Some(gate), SmartSource::Linked { path }) = (s.authorize, src)
        && embedded_source_bytes(&doc.metadata, src).is_none()
    {
        gate(cmd, &json!({"path": path}))?;
    }
    Ok(())
}

/// A decoded, composited source in its own pixel space (`bounds` starts at the origin).
#[derive(Clone, Debug)]
pub struct SourceImage {
    pub surface: Arc<Surface>,
    pub bounds: Rect,
}

type CacheKey = ([u8; 32], PixelFormat);

struct CacheEntry {
    key: CacheKey,
    image: SourceImage,
    decodes: u32,
    /// The surface's real tile bytes, counted once when it is cached.
    bytes: usize,
}

/// Decoded source composites by content hash, most recent last. Re-filtering or re-transforming
/// a smart object never decodes its contents again.
static SOURCE_CACHE: Mutex<Vec<CacheEntry>> = Mutex::new(Vec::new());

#[cfg(not(target_arch = "wasm32"))]
const CACHE_ENTRIES: usize = 32;
#[cfg(target_arch = "wasm32")]
const CACHE_ENTRIES: usize = 4;

/// Accounted tile bytes; a source larger than this on its own isn't kept.
#[cfg(not(target_arch = "wasm32"))]
const CACHE_BYTES: usize = 1 << 30;
#[cfg(target_arch = "wasm32")]
const CACHE_BYTES: usize = 384 << 20;

fn cache() -> std::sync::MutexGuard<'static, Vec<CacheEntry>> {
    SOURCE_CACHE.lock().unwrap_or_else(PoisonError::into_inner)
}

pub(super) fn cache_key(bytes: &[u8], fmt: PixelFormat) -> CacheKey {
    (*blake3::hash(bytes).as_bytes(), fmt)
}

pub(super) fn cache_get(key: &CacheKey) -> Option<SourceImage> {
    let mut c = cache();
    let i = c.iter().position(|e| e.key == *key)?;
    let e = c.remove(i);
    let img = e.image.clone();
    c.push(e);
    Some(img)
}

pub(super) fn cache_put(key: CacheKey, image: SourceImage) {
    cache_put_within(key, image, CACHE_BYTES);
}

/// [`cache_put`] with `limit` as the byte budget.
fn cache_put_within(key: CacheKey, image: SourceImage, limit: usize) {
    let bytes = super::stack_cache::surface_bytes(&image.surface);
    let mut c = cache();
    let decodes = match c.iter().position(|e| e.key == key) {
        Some(i) => c.remove(i).decodes + 1,
        None => 1,
    };
    if bytes > limit {
        return;
    }
    c.push(CacheEntry { key, image, decodes, bytes });
    while c.len() > CACHE_ENTRIES || c.iter().map(|e| e.bytes).sum::<usize>() > limit {
        c.remove(0);
    }
}

/// How many times these contents were decoded at this pixel format (0 = never / evicted).
pub fn source_decode_count(bytes: &[u8], fmt: PixelFormat) -> u32 {
    let key = cache_key(bytes, fmt);
    cache().iter().find(|e| e.key == key).map(|e| e.decodes).unwrap_or(0)
}

/// Bytes held by cached source composites.
pub fn source_cache_bytes() -> usize {
    cache().iter().map(|e| e.bytes).sum()
}

/// Drop every cached source composite (Edit › Purge › All, closing a document with smart
/// objects). Returns the bytes released.
pub fn purge_source_cache() -> usize {
    let mut c = cache();
    let freed = c.iter().map(|e| e.bytes).sum();
    c.clear();
    freed
}

pub(crate) fn take_caches(layers: &mut [Layer]) -> Vec<(LayerId, Surface)> {
    let mut out = Vec::new();
    for l in layers {
        match &mut l.content {
            LayerContent::Smart(sm) => out.extend(sm.cache.take().map(|c| (l.id, c))),
            LayerContent::Group(g) => out.extend(take_caches(&mut g.children)),
            _ => {}
        }
    }
    out
}

pub(crate) fn restore_unrendered<E>(
    doc: &mut Document,
    taken: Vec<(LayerId, Surface)>,
    mut fallback: impl FnMut(&Surface) -> std::result::Result<Surface, E>,
) -> std::result::Result<(), E> {
    for (id, cache) in taken {
        if let Some(LayerContent::Smart(sm)) = doc.layer_mut(id).map(|l| &mut l.content)
            && sm.cache.is_none()
        {
            sm.cache = Some(fallback(&cache)?);
        }
    }
    Ok(())
}

pub(super) fn buffer_to_surface(buf: &photocraft_compose::Buffer, fmt: PixelFormat) -> Surface {
    let n = fmt.channels();
    let mut data = vec![0.0f32; buf.px.len() * n];
    for (p, out) in buf.px.iter().zip(data.chunks_exact_mut(n)) {
        photocraft_raster::from_rgba_into(&fmt, *p, out);
    }
    let mut s = Surface::new(fmt);
    if !buf.rect.is_empty() {
        s.write_region(buf.rect, &data);
    }
    s.prune();
    s
}

/// A source's pixels in `fmt` from a flattened `buf` (the stack-mode combine, which has no
/// document to composite).
pub(super) fn source_surface_from_buffer(src: &Document, buf: &photocraft_compose::Buffer, fmt: PixelFormat) -> Result<Surface> {
    let raster = |f| buffer_to_surface(buf, f);
    convert_source(src, fmt, raster)
}

/// `src` composited straight into `fmt` in strips, without the full-size RGBA buffer
/// [`photocraft_compose::flatten`] allocates (16 bytes a pixel on top of the surface).
pub(super) fn flattened_source(src: &Document, fmt: PixelFormat) -> Result<Surface> {
    let b = src.bounds();
    if u64::from(b.width()) * u64::from(b.height()) > photocraft_compose::MAX_RENDER_PIXELS {
        return Err(other(format!("the smart object's {}×{} contents are too large to render", b.width(), b.height())));
    }
    // The banded compositor composites CMYK in inks and Lab in L, a and b: not the same pixels as
    // the RGBA buffer, so those modes keep the buffer path.
    let native = |m| matches!(m, ColorMode::Cmyk | ColorMode::Lab);
    if native(src.mode) || native(fmt.mode) {
        return source_surface_from_buffer(src, &photocraft_compose::flatten(src), fmt);
    }
    convert_source(src, fmt, |f| composite_in_strips(src, f))
}

/// Pixels per strip of [`composite_in_strips`]: the compositor's own bands are 8 Mpx, which for
/// most sources is the whole image again (a band and its f32 conversion are 32 bytes a pixel).
const STRIP_PIXELS: u64 = 1 << 20;

/// [`photocraft_compose::flatten_to_surface`] of `src`, composited a strip of rows at a time
/// so the float composite and its conversion never cover more than [`STRIP_PIXELS`].
fn composite_in_strips(src: &Document, fmt: PixelFormat) -> Surface {
    let b = src.bounds();
    let rows = strip_rows(b.width());
    let mut out = Surface::new(fmt);
    let mut y = b.y0;
    while y < b.y1 {
        let strip = Rect::new(b.x0, y, b.x1, y.saturating_add(rows).min(b.y1));
        out.write_region(strip, &photocraft_compose::composite_to_surface(src, strip, fmt, None).read_region(strip));
        y = strip.y1;
    }
    out.prune();
    out
}

/// Rows per strip of a `width`-pixel source: as many as fit in [`STRIP_PIXELS`], at least one.
/// Not rounded up to the compositor's tile height, which for a very wide source would be many
/// times the budget (a 300000 px row is already 0.3 Mpx).
fn strip_rows(width: u32) -> i32 {
    i32::try_from((STRIP_PIXELS / u64::from(width.max(1))).max(1)).unwrap_or(i32::MAX)
}

/// Runs the gray and 32-bit conversions a source needs to reach `fmt`; `raster` yields the
/// source's composite at a pixel format.
fn convert_source(src: &Document, fmt: PixelFormat, raster: impl FnOnce(PixelFormat) -> Surface) -> Result<Surface> {
    let gray = fmt.mode == ColorMode::Grayscale && src.pixel_format().mode != ColorMode::Grayscale;
    let linear = fmt.sample == SampleType::F32 && src.depth != SampleType::F32;
    if !gray && !linear {
        return Ok(raster(fmt));
    }
    let mut doc = Document::new(src.name.clone(), src.size, src.mode, src.depth);
    doc.icc_profile = src.icc_profile.clone();
    doc.layers = vec![Layer::new("source", LayerContent::Raster(raster(src.pixel_format())))];
    if gray {
        let gray_profile = crate::color_cmds::working_profile(ColorMode::Grayscale);
        crate::color_cmds::convert_document(&mut doc, &gray_profile, photocraft_cms::Intent::RelativeColorimetric, true)?;
    }
    if linear {
        crate::image_cmds::change_depth(&mut doc, SampleType::F32);
    }
    let surface = doc.layers.pop().and_then(|l| match l.content {
        LayerContent::Raster(s) => Some(s),
        _ => None,
    });
    let surface = surface.ok_or_else(|| other("the smart object's source could not be converted"))?;
    Ok(if surface.format() == fmt { surface } else { surface.convert(fmt) })
}

/// An SVG source rendered at `scale` times its own size (cached per scale), so a vector smart
/// object stays sharp however it is placed.
pub fn svg_source_image(bytes: &[u8], fmt: PixelFormat, scale: f64) -> Result<SourceImage> {
    let mut h = blake3::Hasher::new();
    h.update(bytes);
    h.update(&scale.to_bits().to_le_bytes());
    let key = (*h.finalize().as_bytes(), fmt);
    if let Some(img) = cache_get(&key) {
        return Ok(img);
    }
    let buf = photocraft_io::svg::rasterize(bytes, scale as f32).map_err(|e| other(format!("can't render the SVG smart object: {e}")))?;
    let img = SourceImage { surface: Arc::new(buffer_to_surface(&buf, fmt)), bounds: buf.rect };
    cache_put(key, img.clone());
    Ok(img)
}

/// The scale a vector source is rasterised at for its placement: the larger axis scale of the
/// transform in 1/16 steps (so nudges reuse the cache), between 1/16 and 64.
pub(super) fn vector_scale(sm: &SmartObject) -> f64 {
    let [a, b, c, d, ..] = sm.transform.m;
    let k = (a * a + b * b).sqrt().max((c * c + d * d).sqrt());
    if !k.is_finite() {
        return 1.0;
    }
    ((k * 16.0).ceil() / 16.0).clamp(1.0 / 16.0, 64.0)
}

/// The composited source image of `bytes` in pixel format `fmt` (decoded once, then cached).
pub fn source_image(file_name: &str, bytes: &[u8], fmt: PixelFormat) -> Result<SourceImage> {
    let key = cache_key(bytes, fmt);
    if let Some(img) = cache_get(&key) {
        return Ok(img);
    }
    let doc = decode_source(file_name, bytes)?;
    let img = SourceImage { surface: Arc::new(flattened_source(&doc, fmt)?), bounds: doc.bounds() };
    cache_put(key, img.clone());
    Ok(img)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn image(side: i32) -> SourceImage {
        let mut s = Surface::new(PixelFormat::RGBA8);
        s.fill_rect(Rect::new(0, 0, side, side), &[0.2, 0.4, 0.6, 1.0]);
        SourceImage { surface: Arc::new(s), bounds: Rect::new(0, 0, side, side) }
    }

    #[test]
    fn a_source_larger_than_the_limit_is_returned_but_not_cached() {
        let key = cache_key(b"oversize", PixelFormat::RGBA8);
        let img = image(256);
        let bytes = super::super::stack_cache::surface_bytes(&img.surface);
        cache_put_within(key, img.clone(), bytes - 1);
        assert!(cache_get(&key).is_none(), "{bytes} bytes against a limit of {}", bytes - 1);
        cache_put_within(key, img, bytes);
        assert!(cache_get(&key).is_some());
    }

    #[test]
    fn strips_stay_within_their_pixel_budget_however_wide_the_source() {
        for width in [1u32, 255, 4096, 5000, 300_000, u32::MAX] {
            let rows = strip_rows(width);
            assert!(rows >= 1, "{width}: {rows}");
            assert!(rows == 1 || u64::from(width) * rows as u64 <= STRIP_PIXELS, "{width} px × {rows} rows");
        }
        assert_eq!(strip_rows(4096), 256);
    }

    #[test]
    fn accounting_counts_the_real_tile_bytes() {
        let img = image(256);
        let bytes = super::super::stack_cache::surface_bytes(&img.surface);
        // 8-bit RGBA is 4 bytes a pixel (plus the default pixel), not the 16 of an f32 estimate.
        assert!((256 * 256 * 4..256 * 256 * 5).contains(&bytes), "{bytes}");
    }
}
