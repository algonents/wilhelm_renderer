//! 2D camera system for pan, zoom, and coordinate transformations.
//!
//! Provides [`Camera2D`] for managing the visible region of a 2D world,
//! [`CameraController`] for handling input-driven pan and zoom,
//! and the [`Projection`] trait for custom coordinate transformations.

use crate::core::engine::glfw::{GLFW_MOUSE_BUTTON_LEFT, GLFW_PRESS};
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

/// Input-driven controller for [`Camera2D`] with drag-to-pan and scroll-to-zoom.
///
/// `CameraController` wraps a `Camera2D` and handles mouse/scroll input to provide
/// standard camera controls. Connect it to window callbacks to enable:
/// - **Drag-to-pan**: Hold left mouse button and drag to pan the view
/// - **Scroll-to-zoom**: Mouse wheel zooms in/out with smooth interpolation
///
/// # Example
///
/// ```ignore
/// use wilhelm_renderer::core::{Camera2D, CameraController, Vec2};
///
/// let camera = Camera2D::new(Vec2::new(0.0, 0.0), 1.0, Vec2::new(800.0, 600.0));
/// let mut controller = CameraController::new(camera);
///
/// // Connect to window callbacks
/// window.on_mouse_button(|button, action, _| {
///     controller.on_mouse_button(button, action);
/// });
/// window.on_cursor_position(|x, y| {
///     controller.on_cursor_move(x, y);
/// });
/// window.on_scroll(|_, y| {
///     controller.on_scroll(y);
/// });
///
/// // In render loop - call update() for smooth zoom animation
/// app.on_render(|renderer| {
///     controller.update(dt);
///     // ... render using controller.camera()
/// });
/// ```
pub struct CameraController {
    camera: Camera2D,
    target_scale: f32,
    target_center: Vec2,
    is_dragging: bool,
    last_cursor_pos: Vec2,
    zoom_sensitivity: f32,
    zoom_smoothness: f32,
}

impl CameraController {
    /// Create a new controller wrapping the given camera.
    pub fn new(camera: Camera2D) -> Self {
        let scale = camera.scale();
        let center = camera.center();
        Self {
            camera,
            target_scale: scale,
            target_center: center,
            is_dragging: false,
            last_cursor_pos: Vec2::new(0.0, 0.0),
            zoom_sensitivity: 1.1,
            zoom_smoothness: 6.0,
        }
    }

    /// Set zoom sensitivity. Default is 1.1 (10% zoom per scroll tick).
    ///
    /// Values > 1.0 control how much each scroll tick zooms.
    /// For example, 1.2 means 20% zoom per tick.
    pub fn set_zoom_sensitivity(&mut self, sensitivity: f32) {
        self.zoom_sensitivity = sensitivity;
    }

    /// Set zoom animation smoothness. Default is 10.0.
    ///
    /// Higher values = faster animation (snappier).
    /// Lower values = slower animation (smoother).
    /// A value of 10.0 reaches ~99% of target in ~0.5 seconds.
    pub fn set_zoom_smoothness(&mut self, smoothness: f32) {
        self.zoom_smoothness = smoothness;
    }

    /// Handle mouse button events. Call this from `Window::on_mouse_button`.
    pub fn on_mouse_button(&mut self, button: i32, action: i32) {
        if button == GLFW_MOUSE_BUTTON_LEFT {
            self.is_dragging = action == GLFW_PRESS;
        }
    }

    /// Handle cursor movement. Call this from `Window::on_cursor_position`.
    pub fn on_cursor_move(&mut self, x: f64, y: f64) {
        let cursor = Vec2::new(x as f32, y as f32);

        if self.is_dragging {
            let delta = Vec2::new(
                cursor.x - self.last_cursor_pos.x,
                cursor.y - self.last_cursor_pos.y,
            );
            // Update target_center only - let update() smoothly interpolate
            let scale = self.target_scale;
            self.target_center.x -= delta.x / scale;
            self.target_center.y -= delta.y / scale;
        }

        self.last_cursor_pos = cursor;
    }

    /// Handle scroll events for zooming. Call this from `Window::on_scroll`.
    ///
    /// Zooms centered on the current cursor position with smooth animation.
    pub fn on_scroll(&mut self, y_offset: f64) {
        let factor = if y_offset > 0.0 {
            self.zoom_sensitivity
        } else {
            1.0 / self.zoom_sensitivity
        };

        // Compute target state using zoom_at logic
        // Get world position under cursor at current target state
        let world_point = self.world_at_screen(self.last_cursor_pos);

        // Apply zoom factor to target scale
        self.target_scale *= factor;

        // Compute new target center to keep world_point under cursor
        // screen_point = (world - center) * scale + screen_size/2
        // Solving for center: center = world - (screen_point - screen_size/2) / scale
        let screen_size = self.camera.screen_size();
        self.target_center = Vec2 {
            x: world_point.x - (self.last_cursor_pos.x - screen_size.x * 0.5) / self.target_scale,
            y: world_point.y - (self.last_cursor_pos.y - screen_size.y * 0.5) / self.target_scale,
        };
    }

    /// Update camera animation. Call this each frame with delta time in seconds.
    ///
    /// This smoothly interpolates the camera toward the target zoom level.
    pub fn update(&mut self, dt: f32) {
        // Exponential decay interpolation
        let t = 1.0 - (-self.zoom_smoothness * dt).exp();

        // Interpolate scale
        let current_scale = self.camera.scale();
        let new_scale = current_scale + (self.target_scale - current_scale) * t;
        self.camera.set_scale(new_scale);

        // Interpolate center
        let current_center = self.camera.center();
        let new_center = Vec2 {
            x: current_center.x + (self.target_center.x - current_center.x) * t,
            y: current_center.y + (self.target_center.y - current_center.y) * t,
        };
        self.camera.set_center(new_center);
    }

    /// Get world coordinates at a screen position using target state.
    fn world_at_screen(&self, screen: Vec2) -> Vec2 {
        let screen_size = self.camera.screen_size();
        Vec2 {
            x: (screen.x - screen_size.x * 0.5) / self.target_scale + self.target_center.x,
            y: (screen.y - screen_size.y * 0.5) / self.target_scale + self.target_center.y,
        }
    }

    /// Get a reference to the underlying camera.
    pub fn camera(&self) -> &Camera2D {
        &self.camera
    }

    /// Get a mutable reference to the underlying camera.
    pub fn camera_mut(&mut self) -> &mut Camera2D {
        &mut self.camera
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
