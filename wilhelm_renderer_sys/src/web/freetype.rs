//! FreeType contract stubs for the browser backend.
//!
//! FreeType is C and does not come along on `wasm32-unknown-unknown`.
//! Text rendering is not part of the spike; these stubs return error codes
//! so `FontAtlas` creation fails loudly rather than rendering garbage.
//! The planned production path implements these over a pure-Rust
//! rasterizer (see docs/DESIGN_WASM.md, "FreeType's 9 symbols").

#![allow(non_snake_case)]

use std::ffi::{c_int, c_long, c_uchar, c_uint, c_ulong};

use crate::freetype::{FT_Face, FT_Library, GlyphMetrics};

use super::console_log;

const FT_ERR_UNIMPLEMENTED: c_int = 1;

pub unsafe fn _ft_init_freetype(_library: *mut FT_Library) -> c_int {
    console_log("wilhelm wasm backend: text rendering is not implemented (FreeType stub)");
    FT_ERR_UNIMPLEMENTED
}

pub unsafe fn _ft_done_freetype(_library: FT_Library) {}

pub unsafe fn _ft_new_face(
    _library: FT_Library,
    _filepath: *const c_uchar,
    _face_index: c_long,
    _face: *mut FT_Face,
) -> c_int {
    FT_ERR_UNIMPLEMENTED
}

pub unsafe fn _ft_done_face(_face: FT_Face) {}

pub unsafe fn _ft_set_pixel_sizes(_face: FT_Face, _width: c_uint, _height: c_uint) -> c_int {
    FT_ERR_UNIMPLEMENTED
}

pub unsafe fn _ft_load_char(_face: FT_Face, _char_code: c_ulong, _load_flags: c_int) -> c_int {
    FT_ERR_UNIMPLEMENTED
}

pub unsafe fn _ft_get_glyph_metrics(_face: FT_Face, metrics: *mut GlyphMetrics) {
    unsafe { *metrics = GlyphMetrics::default() }
}

pub unsafe fn _ft_get_glyph_bitmap(_face: FT_Face) -> *const c_uchar {
    std::ptr::null()
}

pub unsafe fn _ft_get_glyph_bitmap_pitch(_face: FT_Face) -> c_int {
    0
}
