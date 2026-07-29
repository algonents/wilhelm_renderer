//! Safe Rust wrappers around the raw FreeType FFI exposed by
//! `wilhelm_renderer_sys`.
//!
//! The raw `extern "C"` functions are imported privately via the `sys`
//! alias and are **not** re-exported — client code cannot reach them
//! through this module.

use std::ffi::{CString, c_long, c_uchar, c_uint, c_ulong};

// Re-export the public FreeType types and constants as part of our API.
pub use wilhelm_renderer_sys::freetype::{
    FT_LOAD_DEFAULT, FT_LOAD_RENDER, FT_Face, FT_Library, GlyphMetrics,
};

// Private alias for the raw FFI. Not re-exported.
use wilhelm_renderer_sys::freetype as sys;

/// Initialize the FreeType library
/// Returns Ok(FT_Library) on success, Err(error_code) on failure
pub fn init_freetype() -> Result<FT_Library, i32> {
    let mut library: FT_Library = std::ptr::null_mut();
    let error = unsafe { sys::_ft_init_freetype(&mut library) };
    if error != 0 {
        Err(error)
    } else {
        Ok(library)
    }
}

/// Clean up the FreeType library
pub fn done_freetype(library: FT_Library) {
    unsafe {
        sys::_ft_done_freetype(library);
    }
}

/// Load a font face from a file
pub fn new_face(library: FT_Library, filepath: &str, face_index: i64) -> Result<FT_Face, i32> {
    let c_filepath = CString::new(filepath).expect("Invalid filepath");
    let mut face: FT_Face = std::ptr::null_mut();
    let error = unsafe {
        sys::_ft_new_face(
            library,
            c_filepath.as_ptr() as *const c_uchar,
            face_index as c_long,
            &mut face,
        )
    };
    if error != 0 {
        Err(error)
    } else {
        Ok(face)
    }
}

/// Clean up a font face
pub fn done_face(face: FT_Face) {
    unsafe {
        sys::_ft_done_face(face);
    }
}

/// Set the pixel size for a font face
pub fn set_pixel_sizes(face: FT_Face, width: u32, height: u32) -> Result<(), i32> {
    let error = unsafe { sys::_ft_set_pixel_sizes(face, width as c_uint, height as c_uint) };
    if error != 0 {
        Err(error)
    } else {
        Ok(())
    }
}

/// Load a character glyph (and render it to bitmap)
pub fn load_char(face: FT_Face, char_code: char) -> Result<(), i32> {
    let error = unsafe { sys::_ft_load_char(face, char_code as c_ulong, FT_LOAD_RENDER) };
    if error != 0 {
        Err(error)
    } else {
        Ok(())
    }
}

/// Load a character glyph outline without rendering it (for SDF rendering)
pub fn load_char_outline(face: FT_Face, char_code: char) -> Result<(), i32> {
    let error = unsafe { sys::_ft_load_char(face, char_code as c_ulong, FT_LOAD_DEFAULT) };
    if error != 0 {
        Err(error)
    } else {
        Ok(())
    }
}

/// Render the currently loaded glyph outline as an 8-bit signed distance field.
/// Texel value 128 lies exactly on the glyph edge, >128 is inside.
pub fn render_glyph_sdf(face: FT_Face) -> Result<(), i32> {
    let error = unsafe { sys::_ft_render_glyph_sdf(face) };
    if error != 0 {
        Err(error)
    } else {
        Ok(())
    }
}

/// Set the SDF spread (padding in pixels around the glyph ink box).
/// Valid range 2..=32, FreeType default is 8.
pub fn set_sdf_spread(library: FT_Library, spread: i32) -> Result<(), i32> {
    let error = unsafe { sys::_ft_set_sdf_spread(library, spread) };
    if error != 0 {
        Err(error)
    } else {
        Ok(())
    }
}

/// Get the metrics of the currently loaded glyph
pub fn get_glyph_metrics(face: FT_Face) -> GlyphMetrics {
    let mut metrics = GlyphMetrics::default();
    unsafe {
        sys::_ft_get_glyph_metrics(face, &mut metrics);
    }
    metrics
}

/// Get the bitmap buffer of the currently loaded glyph
/// Returns a slice of the grayscale bitmap data
pub fn get_glyph_bitmap(face: FT_Face) -> (*const u8, i32) {
    unsafe {
        let buffer = sys::_ft_get_glyph_bitmap(face);
        let pitch = sys::_ft_get_glyph_bitmap_pitch(face);
        (buffer, pitch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_freetype_init_and_cleanup() {
        let library = init_freetype().expect("Failed to initialize FreeType");
        assert!(!library.is_null());
        done_freetype(library);
    }

    #[test]
    fn test_load_font_and_glyph() {
        let library = init_freetype().expect("Failed to initialize FreeType");

        // Use the smallest bundled font so the published tarball stays lean.
        let font_path = "fonts/ArchitectsDaughter-Regular.ttf";
        let face = new_face(library, font_path, 0).expect("Failed to load font");

        set_pixel_sizes(face, 0, 48).expect("Failed to set pixel size");

        load_char(face, 'A').expect("Failed to load glyph");

        let metrics = get_glyph_metrics(face);
        println!("Glyph 'A' metrics: {:?}", metrics);
        assert!(metrics.width > 0);
        assert!(metrics.height > 0);

        let (bitmap, pitch) = get_glyph_bitmap(face);
        assert!(!bitmap.is_null());
        println!("Bitmap pitch: {}", pitch);

        done_face(face);
        done_freetype(library);
    }

    #[test]
    fn test_render_glyph_sdf() {
        let library = init_freetype().expect("Failed to initialize FreeType");
        set_sdf_spread(library, 8).expect("Failed to set SDF spread (sdf module missing?)");

        let font_path = "fonts/ArchitectsDaughter-Regular.ttf";
        let face = new_face(library, font_path, 0).expect("Failed to load font");
        set_pixel_sizes(face, 0, 48).expect("Failed to set pixel size");

        load_char_outline(face, 'A').expect("Failed to load glyph outline");
        // Fails with FT error 0x13 (cannot render this glyph format) if the
        // sdf renderer is not registered in ftmodule.h.
        render_glyph_sdf(face).expect("Failed to render glyph as SDF");

        let metrics = get_glyph_metrics(face);
        assert!(metrics.width > 0);
        assert!(metrics.height > 0);

        let (bitmap, pitch) = get_glyph_bitmap(face);
        assert!(!bitmap.is_null());
        // The SDF renderer emits a tightly packed 8-bit bitmap (pitch == width),
        // which the atlas upload path relies on.
        assert_eq!(pitch, metrics.width);

        // A real distance field spans both sides of the 128 edge value: texels
        // inside the glyph are above 128, texels at the spread boundary
        // approach 0. The interior maximum is capped by the stroke half-width
        // (this font's thin strokes reach ~169 with an 8px spread), so only
        // require clearance beyond the edge. A flat or garbage buffer fails.
        let len = (metrics.width * metrics.height) as usize;
        let texels = unsafe { std::slice::from_raw_parts(bitmap, len) };
        let max = *texels.iter().max().unwrap();
        let min = *texels.iter().min().unwrap();
        assert!(max > 140, "no interior texels found (max = {})", max);
        assert!(min < 50, "no exterior texels found (min = {})", min);

        done_face(face);
        done_freetype(library);
    }
}
