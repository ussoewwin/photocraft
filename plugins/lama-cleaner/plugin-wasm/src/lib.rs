//! PhotoCraft LaMa AI Inpaint Plug-in (ABI v1)
#![no_std]

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

const MANIFEST: &str = r#"{
  "id": "org.photocraft.plugin.lama",
  "name": "AI消去 (LaMa Inpaint)",
  "version": "1.0.0",
  "kind": "filter",
  "author": "PhotoCraft Native Plugin",
  "description": "LaMa (Large Mask Inpainting) による高速AI消去・補完",
  "params": {}
}"#;

#[unsafe(no_mangle)]
pub extern "C" fn pc_abi_version() -> u32 {
    1
}

#[unsafe(no_mangle)]
pub extern "C" fn pc_manifest() -> u64 {
    ((MANIFEST.len() as u64) << 32) | MANIFEST.as_ptr() as u64
}

#[unsafe(no_mangle)]
pub extern "C" fn pc_alloc(size: u32) -> u32 {
    let pages = (size as usize).div_ceil(65536);
    let old = core::arch::wasm32::memory_grow(0, pages);
    if old == usize::MAX { 0 } else { (old * 65536) as u32 }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pc_filter(
    _buf: *mut f32,
    _len: u32,
    _width: u32,
    _height: u32,
    _channels: u32,
    _format: u32,
    _params: *const u8,
    _params_len: u32,
) -> i32 {
    0
}
