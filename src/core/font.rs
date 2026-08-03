//! Font atlas for text rendering
//!
//! Manages glyph caching in an OpenGL texture atlas. Glyphs are parsed and
//! rasterized in pure Rust (`ttf-parser` + `ab_glyph_rasterizer`) — the same
//! code runs on every backend, so text lives above the sys contract.

use crate::core::engine::opengl::{
    gl_bind_texture, gl_delete_texture, gl_gen_texture, gl_pixel_storei, gl_tex_image_2d,
    gl_tex_parameteri, gl_tex_sub_image_2d, GL_CLAMP_TO_EDGE, GL_LINEAR, GL_R8, GL_RED,
    GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_TEXTURE_MIN_FILTER, GL_TEXTURE_WRAP_S,
    GL_TEXTURE_WRAP_T, GL_UNPACK_ALIGNMENT, GL_UNSIGNED_BYTE,
};
use std::collections::HashMap;

/// Where font bytes come from — same convergence-on-bytes design as
/// [`crate::core::image::ImageSource`]: backends without a filesystem
/// (wasm) use `Bytes`.
pub enum FontSource<'a> {
    Path(&'a str),
    Bytes(&'a [u8]),
}

impl<'a> From<&'a str> for FontSource<'a> {
    fn from(path: &'a str) -> Self {
        FontSource::Path(path)
    }
}

impl<'a> From<&'a [u8]> for FontSource<'a> {
    fn from(bytes: &'a [u8]) -> Self {
        FontSource::Bytes(bytes)
    }
}

impl<'a, const N: usize> From<&'a [u8; N]> for FontSource<'a> {
    fn from(bytes: &'a [u8; N]) -> Self {
        FontSource::Bytes(bytes)
    }
}

#[derive(Debug)]
pub enum FontError {
    Io(std::io::Error),
    Parse(ttf_parser::FaceParsingError),
}

impl std::fmt::Display for FontError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FontError::Io(e) => write!(f, "failed to read font: {e}"),
            FontError::Parse(e) => write!(f, "failed to parse font: {e}"),
        }
    }
}

impl std::error::Error for FontError {}

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

/// A rasterized glyph: 8-bit grayscale coverage bitmap plus metrics, in the
/// same conventions FreeType used (bearing_y = rows above the baseline,
/// bitmap rows top-down).
struct RasterGlyph {
    width: i32,
    height: i32,
    bearing_x: i32,
    bearing_y: i32,
    advance: f32,
    bitmap: Vec<u8>,
}

/// Feeds a glyph outline into the coverage rasterizer, mapping font units
/// (y-up, unscaled) to bitmap pixels (y-down, origin at the glyph bbox
/// top-left).
struct OutlinePainter {
    rasterizer: ab_glyph_rasterizer::Rasterizer,
    scale: f32,
    offset_x: f32,
    offset_y: f32,
    start: ab_glyph_rasterizer::Point,
    current: ab_glyph_rasterizer::Point,
}

impl OutlinePainter {
    fn map(&self, x: f32, y: f32) -> ab_glyph_rasterizer::Point {
        ab_glyph_rasterizer::point(x * self.scale - self.offset_x, self.offset_y - y * self.scale)
    }
}

impl ttf_parser::OutlineBuilder for OutlinePainter {
    fn move_to(&mut self, x: f32, y: f32) {
        self.start = self.map(x, y);
        self.current = self.start;
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let p = self.map(x, y);
        self.rasterizer.draw_line(self.current, p);
        self.current = p;
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let c = self.map(x1, y1);
        let p = self.map(x, y);
        self.rasterizer.draw_quad(self.current, c, p);
        self.current = p;
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let c0 = self.map(x1, y1);
        let c1 = self.map(x2, y2);
        let p = self.map(x, y);
        self.rasterizer.draw_cubic(self.current, c0, c1, p);
        self.current = p;
    }

    fn close(&mut self) {
        self.rasterizer.draw_line(self.current, self.start);
        self.current = self.start;
    }
}

/// Collects a glyph's outline bounds without rasterizing (first pass — the
/// rasterizer needs its dimensions before drawing can start).
struct NoopOutline;

impl ttf_parser::OutlineBuilder for NoopOutline {
    fn move_to(&mut self, _x: f32, _y: f32) {}
    fn line_to(&mut self, _x: f32, _y: f32) {}
    fn quad_to(&mut self, _x1: f32, _y1: f32, _x: f32, _y: f32) {}
    fn curve_to(&mut self, _x1: f32, _y1: f32, _x2: f32, _y2: f32, _x: f32, _y: f32) {}
    fn close(&mut self) {}
}

/// Rasterize one character at `font_size` pixels-per-em. Pure function of
/// the font bytes — no GL, testable headless. Returns `None` when the font
/// has no glyph for the character; whitespace and other empty glyphs return
/// a zero-sized bitmap with a valid advance.
fn rasterize_glyph(font_data: &[u8], ch: char, font_size: u32) -> Option<RasterGlyph> {
    let face = ttf_parser::Face::parse(font_data, 0).ok()?;
    let glyph_id = face.glyph_index(ch)?;

    let scale = font_size as f32 / face.units_per_em() as f32;
    let advance = face.glyph_hor_advance(glyph_id).unwrap_or(0) as f32 * scale;

    // Pass 1: bounds only. Empty outline (space etc.) -> advance-only glyph.
    let bbox = match face.outline_glyph(glyph_id, &mut NoopOutline) {
        Some(bbox) => bbox,
        None => {
            return Some(RasterGlyph {
                width: 0,
                height: 0,
                bearing_x: 0,
                bearing_y: 0,
                advance,
                bitmap: Vec::new(),
            });
        }
    };

    let x_min = (bbox.x_min as f32 * scale).floor();
    let x_max = (bbox.x_max as f32 * scale).ceil();
    let y_min = (bbox.y_min as f32 * scale).floor();
    let y_max = (bbox.y_max as f32 * scale).ceil();
    let width = (x_max - x_min) as i32;
    let height = (y_max - y_min) as i32;

    // Pass 2: rasterize into a width x height coverage grid.
    let mut painter = OutlinePainter {
        rasterizer: ab_glyph_rasterizer::Rasterizer::new(width as usize, height as usize),
        scale,
        offset_x: x_min,
        offset_y: y_max,
        start: ab_glyph_rasterizer::point(0.0, 0.0),
        current: ab_glyph_rasterizer::point(0.0, 0.0),
    };
    face.outline_glyph(glyph_id, &mut painter)?;

    let mut bitmap = vec![0u8; (width * height) as usize];
    painter.rasterizer.for_each_pixel_2d(|x, y, coverage| {
        bitmap[(y as i32 * width + x as i32) as usize] = (coverage * 255.0) as u8;
    });

    Some(RasterGlyph {
        width,
        height,
        bearing_x: x_min as i32,
        bearing_y: y_max as i32,
        advance,
        bitmap,
    })
}

/// A font atlas that caches glyphs in an OpenGL texture
pub struct FontAtlas {
    font_data: Vec<u8>,
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
    pub fn new(font_path: &str, font_size: u32, atlas_size: u32) -> Result<Self, FontError> {
        Self::from_source(font_path, font_size, atlas_size)
    }

    /// Create a font atlas from any [`FontSource`] (path or bytes).
    pub fn from_source<'a>(
        source: impl Into<FontSource<'a>>,
        font_size: u32,
        atlas_size: u32,
    ) -> Result<Self, FontError> {
        let font_data = match source.into() {
            FontSource::Path(path) => std::fs::read(path).map_err(FontError::Io)?,
            FontSource::Bytes(bytes) => bytes.to_vec(),
        };

        // Validate the face up front so errors surface at load, not per-glyph.
        ttf_parser::Face::parse(&font_data, 0).map_err(FontError::Parse)?;

        // Create OpenGL texture
        let texture_id = gl_gen_texture();
        gl_bind_texture(GL_TEXTURE_2D, texture_id);

        // Set texture parameters
        gl_tex_parameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE);
        gl_tex_parameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE);
        gl_tex_parameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
        gl_tex_parameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);

        // Allocate empty texture (single channel for grayscale glyphs).
        // Sized GL_R8: WebGL2 rejects unsized GL_RED as an internalformat.
        gl_pixel_storei(GL_UNPACK_ALIGNMENT, 1);
        gl_tex_image_2d(
            GL_TEXTURE_2D,
            0,
            GL_R8 as i32,
            atlas_size as i32,
            atlas_size as i32,
            0,
            GL_RED,
            GL_UNSIGNED_BYTE,
            std::ptr::null(),
        );

        Ok(Self {
            font_data,
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
        let glyph = rasterize_glyph(&self.font_data, ch, self.font_size)?;

        if glyph.width == 0 || glyph.height == 0 {
            // Space or empty glyph - still need to track advance
            let info = GlyphInfo {
                uv_x: 0.0,
                uv_y: 0.0,
                uv_width: 0.0,
                uv_height: 0.0,
                width: 0,
                height: 0,
                bearing_x: glyph.bearing_x,
                bearing_y: glyph.bearing_y,
                advance: glyph.advance,
            };
            self.glyphs.insert(ch, info);
            return Some(info);
        }

        let glyph_width = glyph.width as u32;
        let glyph_height = glyph.height as u32;

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
            glyph.bitmap.as_ptr() as *const std::ffi::c_void,
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
            width: glyph.width,
            height: glyph.height,
            bearing_x: glyph.bearing_x,
            bearing_y: glyph.bearing_y,
            advance: glyph.advance,
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dejavu() -> Vec<u8> {
        std::fs::read("fonts/DejaVuSans.ttf").expect("bundled test font")
    }

    #[test]
    fn rasterizes_a_letter() {
        let glyph = rasterize_glyph(&dejavu(), 'A', 48).expect("glyph for 'A'");
        assert!(glyph.width > 0 && glyph.height > 0);
        assert!(glyph.advance > 0.0);
        assert!(glyph.bearing_y > 0, "cap letter sits above the baseline");
        assert_eq!(glyph.bitmap.len(), (glyph.width * glyph.height) as usize);
        assert!(
            glyph.bitmap.iter().any(|&px| px > 128),
            "bitmap has substantial coverage"
        );
        // Nothing outside the bbox: the outline touches its bounds, so at
        // least one pixel in the first and last rows must be inked.
        let w = glyph.width as usize;
        assert!(glyph.bitmap[..w].iter().any(|&px| px > 0));
        assert!(glyph.bitmap[glyph.bitmap.len() - w..].iter().any(|&px| px > 0));
    }

    #[test]
    fn space_has_advance_but_no_bitmap() {
        let glyph = rasterize_glyph(&dejavu(), ' ', 48).expect("glyph for space");
        assert_eq!(glyph.width, 0);
        assert_eq!(glyph.height, 0);
        assert!(glyph.advance > 0.0);
        assert!(glyph.bitmap.is_empty());
    }

    #[test]
    fn unmapped_char_is_none() {
        // Egyptian hieroglyph — not covered by DejaVu Sans.
        assert!(rasterize_glyph(&dejavu(), '\u{13000}', 48).is_none());
    }

    #[test]
    fn scales_with_font_size() {
        let small = rasterize_glyph(&dejavu(), 'M', 16).unwrap();
        let large = rasterize_glyph(&dejavu(), 'M', 64).unwrap();
        assert!(large.width > small.width * 3);
        assert!(large.advance > small.advance * 3.9 && large.advance < small.advance * 4.1);
    }
}
