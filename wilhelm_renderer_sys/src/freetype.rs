//! Raw FreeType FFI bindings for text rendering.
//!
//! All extern declarations, types, and constants used by `wilhelm_renderer`.
//! Safe Rust wrappers live in the upper crate.

use std::ffi::{c_int, c_long, c_uchar, c_uint, c_ulong};

/// Opaque FreeType library handle
#[allow(non_camel_case_types)]
pub type FT_Library = *mut std::ffi::c_void;

/// Opaque FreeType face handle
#[allow(non_camel_case_types)]
pub type FT_Face = *mut std::ffi::c_void;

/// Glyph metrics returned from FreeType
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct GlyphMetrics {
    pub width: c_int,     // Glyph width in pixels
    pub height: c_int,    // Glyph height in pixels
    pub bearing_x: c_int, // Horizontal bearing (left)
    pub bearing_y: c_int, // Vertical bearing (top)
    pub advance: c_long,  // Horizontal advance (in 1/64th pixels)
}

/// FreeType load flags
pub const FT_LOAD_RENDER: c_int = 4;

unsafe extern "C" {
    pub fn _ft_init_freetype(library: *mut FT_Library) -> c_int;
    pub fn _ft_done_freetype(library: FT_Library);

    pub fn _ft_new_face(
        library: FT_Library,
        filepath: *const c_uchar,
        face_index: c_long,
        face: *mut FT_Face,
    ) -> c_int;
    pub fn _ft_done_face(face: FT_Face);
    pub fn _ft_set_pixel_sizes(face: FT_Face, width: c_uint, height: c_uint) -> c_int;
    pub fn _ft_load_char(face: FT_Face, char_code: c_ulong, load_flags: c_int) -> c_int;

    pub fn _ft_get_glyph_metrics(face: FT_Face, metrics: *mut GlyphMetrics);
    pub fn _ft_get_glyph_bitmap(face: FT_Face) -> *const c_uchar;
    pub fn _ft_get_glyph_bitmap_pitch(face: FT_Face) -> c_int;
}
