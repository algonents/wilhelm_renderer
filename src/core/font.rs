//! Font atlas for text rendering
//!
//! Manages glyph caching in an OpenGL texture atlas.

use crate::core::engine::freetype::{
    done_face, done_freetype, get_glyph_bitmap, get_glyph_metrics, init_freetype, load_char,
    new_face, set_pixel_sizes, FT_Face, FT_Library,
};
use crate::core::engine::opengl::{
    gl_bind_texture, gl_delete_texture, gl_gen_texture, gl_pixel_storei, gl_tex_image_2d,
    gl_tex_parameteri, gl_tex_sub_image_2d, GL_CLAMP_TO_EDGE, GL_LINEAR, GL_RED, GL_TEXTURE_2D,
    GL_TEXTURE_MAG_FILTER, GL_TEXTURE_MIN_FILTER, GL_TEXTURE_WRAP_S, GL_TEXTURE_WRAP_T,
    GL_UNPACK_ALIGNMENT, GL_UNSIGNED_BYTE,
};
use std::collections::HashMap;

/// Information about a cached glyph in the atlas
#[derive(Debug, Clone, Copy)]
pub struct GlyphInfo {
    /// UV coordinates (top-left)
    pub uv_x: f32,
    pub uv_y: f32,
    /// UV size
    pub uv_width: f32,
    pub uv_height: f32,
    /// Glyph size in pixels
    pub width: i32,
    pub height: i32,
    /// Offset from cursor to glyph top-left
    pub bearing_x: i32,
    pub bearing_y: i32,
    /// Horizontal advance (in pixels)
    pub advance: f32,
}

/// A font atlas that caches glyphs in an OpenGL texture
pub struct FontAtlas {
    library: FT_Library,
    face: FT_Face,
    texture_id: u32,
    atlas_width: u32,
    atlas_height: u32,
    /// Current packing position
    cursor_x: u32,
    cursor_y: u32,
    row_height: u32,
    /// Cached glyphs
    glyphs: HashMap<char, GlyphInfo>,
    /// Font size in pixels
    font_size: u32,
}

impl FontAtlas {
    /// Create a new font atlas
    ///
    /// # Arguments
    /// * `font_path` - Path to the TTF/OTF font file
    /// * `font_size` - Font size in pixels
    /// * `atlas_size` - Size of the texture atlas (width and height, must be power of 2)
    pub fn new(font_path: &str, font_size: u32, atlas_size: u32) -> Result<Self, String> {
        // Initialize FreeType
        let library = init_freetype().map_err(|e| format!("Failed to init FreeType: {}", e))?;

        // Load font face
        let face =
            new_face(library, font_path, 0).map_err(|e| format!("Failed to load font: {}", e))?;

        // Set font size
        set_pixel_sizes(face, 0, font_size)
            .map_err(|e| format!("Failed to set font size: {}", e))?;

        // Create OpenGL texture
        let texture_id = gl_gen_texture();
        gl_bind_texture(GL_TEXTURE_2D, texture_id);

        // Set texture parameters
        gl_tex_parameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE);
        gl_tex_parameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE);
        gl_tex_parameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
        gl_tex_parameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);

        // Allocate empty texture (single channel for grayscale glyphs)
        gl_pixel_storei(GL_UNPACK_ALIGNMENT, 1);
        gl_tex_image_2d(
            GL_TEXTURE_2D,
            0,
            GL_RED as i32,
            atlas_size as i32,
            atlas_size as i32,
            0,
            GL_RED,
            GL_UNSIGNED_BYTE,
            std::ptr::null(),
        );

        Ok(Self {
            library,
            face,
            texture_id,
            atlas_width: atlas_size,
            atlas_height: atlas_size,
            cursor_x: 0,
            cursor_y: 0,
            row_height: 0,
            glyphs: HashMap::new(),
            font_size,
        })
    }

    /// Get the OpenGL texture ID
    pub fn texture_id(&self) -> u32 {
        self.texture_id
    }

    /// Get glyph info, loading it into the atlas if necessary
    pub fn get_glyph(&mut self, ch: char) -> Option<GlyphInfo> {
        // Return cached glyph if available
        if let Some(&info) = self.glyphs.get(&ch) {
            return Some(info);
        }

        // Load and cache the glyph
        self.cache_glyph(ch)
    }

    /// Cache a glyph into the atlas
    fn cache_glyph(&mut self, ch: char) -> Option<GlyphInfo> {
        // Load the glyph
        if load_char(self.face, ch).is_err() {
            return None;
        }

        let metrics = get_glyph_metrics(self.face);
        let (bitmap_ptr, _pitch) = get_glyph_bitmap(self.face);

        if bitmap_ptr.is_null() || metrics.width == 0 || metrics.height == 0 {
            // Space or empty glyph - still need to track advance
            let info = GlyphInfo {
                uv_x: 0.0,
                uv_y: 0.0,
                uv_width: 0.0,
                uv_height: 0.0,
                width: 0,
                height: 0,
                bearing_x: metrics.bearing_x,
                bearing_y: metrics.bearing_y,
                advance: (metrics.advance >> 6) as f32, // Convert from 1/64th pixels
            };
            self.glyphs.insert(ch, info);
            return Some(info);
        }

        let glyph_width = metrics.width as u32;
        let glyph_height = metrics.height as u32;

        // Check if we need to move to next row
        if self.cursor_x + glyph_width > self.atlas_width {
            self.cursor_x = 0;
            self.cursor_y += self.row_height + 1; // +1 for padding
            self.row_height = 0;
        }

        // Check if atlas is full
        if self.cursor_y + glyph_height > self.atlas_height {
            eprintln!("Font atlas is full!");
            return None;
        }

        // Upload glyph bitmap to texture
        gl_bind_texture(GL_TEXTURE_2D, self.texture_id);
        gl_pixel_storei(GL_UNPACK_ALIGNMENT, 1);

        gl_tex_sub_image_2d(
            GL_TEXTURE_2D,
            0,
            self.cursor_x as i32,
            self.cursor_y as i32,
            glyph_width as i32,
            glyph_height as i32,
            GL_RED,
            GL_UNSIGNED_BYTE,
            bitmap_ptr as *const std::ffi::c_void,
        );

        // Calculate UV coordinates
        let uv_x = self.cursor_x as f32 / self.atlas_width as f32;
        let uv_y = self.cursor_y as f32 / self.atlas_height as f32;
        let uv_width = glyph_width as f32 / self.atlas_width as f32;
        let uv_height = glyph_height as f32 / self.atlas_height as f32;

        let info = GlyphInfo {
            uv_x,
            uv_y,
            uv_width,
            uv_height,
            width: metrics.width,
            height: metrics.height,
            bearing_x: metrics.bearing_x,
            bearing_y: metrics.bearing_y,
            advance: (metrics.advance >> 6) as f32,
        };

        // Update cursor position
        self.cursor_x += glyph_width + 1; // +1 for padding
        self.row_height = self.row_height.max(glyph_height);

        self.glyphs.insert(ch, info);
        Some(info)
    }

    /// Pre-cache ASCII characters (useful for initialization)
    pub fn cache_ascii(&mut self) {
        for ch in 32u8..127u8 {
            self.get_glyph(ch as char);
        }
    }

    /// Calculate the width of a string in pixels
    pub fn measure_text(&mut self, text: &str) -> f32 {
        let mut width = 0.0;
        for ch in text.chars() {
            if let Some(glyph) = self.get_glyph(ch) {
                width += glyph.advance;
            }
        }
        width
    }

    /// Get font size
    pub fn font_size(&self) -> u32 {
        self.font_size
    }
}

impl Drop for FontAtlas {
    fn drop(&mut self) {
        // Clean up OpenGL texture
        gl_delete_texture(self.texture_id);

        // Clean up FreeType resources
        done_face(self.face);
        done_freetype(self.library);
    }
}
