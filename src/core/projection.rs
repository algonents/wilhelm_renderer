//! Coordinate projection and camera system for 2D rendering.
//!
//! This module provides traits and implementations for transforming between
//! world coordinates and screen coordinates.

use crate::core::engine::opengl::Vec2;

/// Trait for coordinate transformations between world and screen space.
///
/// Implementors define how world coordinates map to screen pixels and vice versa.
pub trait Projection {
    /// Convert world coordinates to screen coordinates (pixels).
    fn world_to_screen(&self, world: Vec2) -> Vec2;

    /// Convert screen coordinates (pixels) to world coordinates.
    fn screen_to_world(&self, screen: Vec2) -> Vec2;
}

/// Identity projection where world coordinates equal screen coordinates.
///
/// This is a passthrough projection useful when working directly in pixel coordinates.
#[derive(Debug, Clone, Copy, Default)]
pub struct IdentityProjection;

impl Projection for IdentityProjection {
    fn world_to_screen(&self, world: Vec2) -> Vec2 {
        world
    }

    fn screen_to_world(&self, screen: Vec2) -> Vec2 {
        screen
    }
}

/// A 2D camera that defines the visible region of the world.
///
/// The camera manages pan and zoom state, converting between world coordinates
/// and screen coordinates. Screen origin is at top-left, with Y increasing downward.
///
/// # Example
///
/// ```
/// use wilhelm_renderer::core::{Camera2D, Projection, Vec2};
///
/// let camera = Camera2D::new(
///     Vec2::new(0.0, 0.0),   // center: looking at world origin
///     1.0,                    // scale: 1 pixel per world unit
///     Vec2::new(800.0, 600.0) // screen size
/// );
///
/// // World origin appears at screen center
/// let screen_pos = camera.world_to_screen(Vec2::new(0.0, 0.0));
/// assert_eq!(screen_pos.x, 400.0);
/// assert_eq!(screen_pos.y, 300.0);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Camera2D {
    /// World coordinates at the center of the screen.
    center: Vec2,
    /// Scale factor: pixels per world unit.
    /// Higher values = zoomed in, lower values = zoomed out.
    scale: f32,
    /// Screen dimensions in pixels.
    screen_size: Vec2,
}

impl Camera2D {
    /// Create a new camera.
    ///
    /// # Arguments
    /// * `center` - World coordinates that appear at screen center
    /// * `scale` - Pixels per world unit (zoom level)
    /// * `screen_size` - Window dimensions in pixels
    pub fn new(center: Vec2, scale: f32, screen_size: Vec2) -> Self {
        Self {
            center,
            scale,
            screen_size,
        }
    }

    /// Get the camera center in world coordinates.
    pub fn center(&self) -> Vec2 {
        self.center
    }

    /// Set the camera center in world coordinates (pan).
    pub fn set_center(&mut self, center: Vec2) {
        self.center = center;
    }

    /// Get the current scale (pixels per world unit).
    pub fn scale(&self) -> f32 {
        self.scale
    }

    /// Set the scale (zoom level).
    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale;
    }

    /// Get the screen size.
    pub fn screen_size(&self) -> Vec2 {
        self.screen_size
    }

    /// Update screen size (e.g., on window resize).
    pub fn set_screen_size(&mut self, screen_size: Vec2) {
        self.screen_size = screen_size;
    }

    /// Pan the camera by a delta in world coordinates.
    pub fn pan(&mut self, delta: Vec2) {
        self.center.x += delta.x;
        self.center.y += delta.y;
    }

    /// Pan the camera by a delta in screen coordinates.
    pub fn pan_screen(&mut self, delta_pixels: Vec2) {
        self.center.x -= delta_pixels.x / self.scale;
        self.center.y -= delta_pixels.y / self.scale;
    }

    /// Zoom by a factor, keeping the screen center fixed.
    ///
    /// Factor > 1.0 zooms in, factor < 1.0 zooms out.
    pub fn zoom(&mut self, factor: f32) {
        self.scale *= factor;
    }

    /// Zoom by a factor, keeping a specific screen point fixed.
    ///
    /// Useful for zoom-to-cursor behavior.
    pub fn zoom_at(&mut self, factor: f32, screen_point: Vec2) {
        // Get world position under the cursor before zoom
        let world_before = self.screen_to_world(screen_point);

        // Apply zoom
        self.scale *= factor;

        // Get world position under cursor after zoom
        let world_after = self.screen_to_world(screen_point);

        // Adjust center to keep the point fixed
        self.center.x += world_before.x - world_after.x;
        self.center.y += world_before.y - world_after.y;
    }

    /// Get the visible world bounds as (min_x, min_y, max_x, max_y).
    pub fn world_bounds(&self) -> (f32, f32, f32, f32) {
        let half_width = self.screen_size.x / (2.0 * self.scale);
        let half_height = self.screen_size.y / (2.0 * self.scale);
        (
            self.center.x - half_width,
            self.center.y - half_height,
            self.center.x + half_width,
            self.center.y + half_height,
        )
    }
}

impl Projection for Camera2D {
    fn world_to_screen(&self, world: Vec2) -> Vec2 {
        Vec2 {
            x: (world.x - self.center.x) * self.scale + self.screen_size.x * 0.5,
            y: (world.y - self.center.y) * self.scale + self.screen_size.y * 0.5,
        }
    }

    fn screen_to_world(&self, screen: Vec2) -> Vec2 {
        Vec2 {
            x: (screen.x - self.screen_size.x * 0.5) / self.scale + self.center.x,
            y: (screen.y - self.screen_size.y * 0.5) / self.scale + self.center.y,
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_projection() {
        let proj = IdentityProjection;
        let world = Vec2::new(100.0, 200.0);

        assert_eq!(proj.world_to_screen(world), world);
        assert_eq!(proj.screen_to_world(world), world);
    }

    #[test]
    fn test_camera_center_at_origin() {
        let camera = Camera2D::new(
            Vec2::new(0.0, 0.0),
            1.0,
            Vec2::new(800.0, 600.0),
        );

        // World origin should map to screen center
        let screen = camera.world_to_screen(Vec2::new(0.0, 0.0));
        assert_eq!(screen.x, 400.0);
        assert_eq!(screen.y, 300.0);

        // Screen center should map back to world origin
        let world = camera.screen_to_world(Vec2::new(400.0, 300.0));
        assert_eq!(world.x, 0.0);
        assert_eq!(world.y, 0.0);
    }

    #[test]
    fn test_camera_with_offset_center() {
        let camera = Camera2D::new(
            Vec2::new(100.0, 50.0), // looking at (100, 50)
            1.0,
            Vec2::new(800.0, 600.0),
        );

        // World (100, 50) should be at screen center
        let screen = camera.world_to_screen(Vec2::new(100.0, 50.0));
        assert_eq!(screen.x, 400.0);
        assert_eq!(screen.y, 300.0);

        // World origin should be offset from screen center
        let screen = camera.world_to_screen(Vec2::new(0.0, 0.0));
        assert_eq!(screen.x, 300.0); // 400 - 100
        assert_eq!(screen.y, 250.0); // 300 - 50
    }

    #[test]
    fn test_camera_with_scale() {
        let camera = Camera2D::new(
            Vec2::new(0.0, 0.0),
            2.0, // 2 pixels per world unit (zoomed in)
            Vec2::new(800.0, 600.0),
        );

        // World (10, 10) should be 20 pixels from screen center
        let screen = camera.world_to_screen(Vec2::new(10.0, 10.0));
        assert_eq!(screen.x, 420.0); // 400 + 10*2
        assert_eq!(screen.y, 320.0); // 300 + 10*2

        // Roundtrip
        let world = camera.screen_to_world(screen);
        assert!((world.x - 10.0).abs() < 0.001);
        assert!((world.y - 10.0).abs() < 0.001);
    }

    #[test]
    fn test_camera_world_bounds() {
        let camera = Camera2D::new(
            Vec2::new(0.0, 0.0),
            1.0,
            Vec2::new(800.0, 600.0),
        );

        let (min_x, min_y, max_x, max_y) = camera.world_bounds();
        assert_eq!(min_x, -400.0);
        assert_eq!(min_y, -300.0);
        assert_eq!(max_x, 400.0);
        assert_eq!(max_y, 300.0);
    }

    #[test]
    fn test_camera_zoom_at_center() {
        let mut camera = Camera2D::new(
            Vec2::new(0.0, 0.0),
            1.0,
            Vec2::new(800.0, 600.0),
        );

        // Zoom in 2x at screen center
        camera.zoom_at(2.0, Vec2::new(400.0, 300.0));

        // Center should remain unchanged
        assert_eq!(camera.center().x, 0.0);
        assert_eq!(camera.center().y, 0.0);
        assert_eq!(camera.scale(), 2.0);
    }

    #[test]
    fn test_camera_zoom_at_corner() {
        let mut camera = Camera2D::new(
            Vec2::new(0.0, 0.0),
            1.0,
            Vec2::new(800.0, 600.0),
        );

        // World position at top-left corner before zoom
        let corner_world_before = camera.screen_to_world(Vec2::new(0.0, 0.0));

        // Zoom in 2x at top-left corner
        camera.zoom_at(2.0, Vec2::new(0.0, 0.0));

        // World position at top-left corner should be the same
        let corner_world_after = camera.screen_to_world(Vec2::new(0.0, 0.0));

        assert!((corner_world_before.x - corner_world_after.x).abs() < 0.001);
        assert!((corner_world_before.y - corner_world_after.y).abs() < 0.001);
    }
}
