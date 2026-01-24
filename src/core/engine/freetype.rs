//! FreeType FFI bindings for text rendering

use std::ffi::{c_int, c_long, c_uchar, c_uint, c_ulong, CString};

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
    pub width: c_int,      // Glyph width in pixels
    pub height: c_int,     // Glyph height in pixels
    pub bearing_x: c_int,  // Horizontal bearing (left)
    pub bearing_y: c_int,  // Vertical bearing (top)
    pub advance: c_long,   // Horizontal advance (in 1/64th pixels)
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

/// Initialize the FreeType library
/// Returns Ok(FT_Library) on success, Err(error_code) on failure
pub fn init_freetype() -> Result<FT_Library, i32> {
    let mut library: FT_Library = std::ptr::null_mut();
    let error = unsafe { _ft_init_freetype(&mut library) };
    if error != 0 {
        Err(error)
    } else {
        Ok(library)
    }
}

/// Clean up the FreeType library
pub fn done_freetype(library: FT_Library) {
    unsafe {
        _ft_done_freetype(library);
    }
}

/// Load a font face from a file
pub fn new_face(library: FT_Library, filepath: &str, face_index: i64) -> Result<FT_Face, i32> {
    let c_filepath = CString::new(filepath).expect("Invalid filepath");
    let mut face: FT_Face = std::ptr::null_mut();
    let error = unsafe {
        _ft_new_face(
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
        _ft_done_face(face);
    }
}

/// Set the pixel size for a font face
pub fn set_pixel_sizes(face: FT_Face, width: u32, height: u32) -> Result<(), i32> {
    let error = unsafe { _ft_set_pixel_sizes(face, width as c_uint, height as c_uint) };
    if error != 0 {
        Err(error)
    } else {
        Ok(())
    }
}

/// Load a character glyph (and render it to bitmap)
pub fn load_char(face: FT_Face, char_code: char) -> Result<(), i32> {
    let error = unsafe { _ft_load_char(face, char_code as c_ulong, FT_LOAD_RENDER) };
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
        _ft_get_glyph_metrics(face, &mut metrics);
    }
    metrics
}

/// Get the bitmap buffer of the currently loaded glyph
/// Returns a slice of the grayscale bitmap data
pub fn get_glyph_bitmap(face: FT_Face) -> (*const u8, i32) {
    unsafe {
        let buffer = _ft_get_glyph_bitmap(face);
        let pitch = _ft_get_glyph_bitmap_pitch(face);
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

        // Use bundled font
        let font_path = "fonts/DejaVuSans.ttf";
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
}
