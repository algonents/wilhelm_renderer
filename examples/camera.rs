//! Example demonstrating Camera2D for pan and zoom.
//!
//! - Scroll wheel: zoom in/out (zooms toward cursor)
//!
//! Shapes are defined in world coordinates and transformed to screen
//! coordinates using the camera projection. Shape SIZES stay constant
//! in screen pixels (like map markers/waypoints) - only POSITIONS change
//! with zoom. When zoomed out, shapes cluster together; when zoomed in,
//! they spread apart.
//!
//! This example is production-ready: ShapeRenderables are created once
//! and their positions are updated each frame via set_position().

extern crate wilhelm_renderer;

use std::cell::Cell;
use wilhelm_renderer::core::{App, Camera2D, Color, Projection, Renderable, Renderer, Vec2, Window};
use wilhelm_renderer::graphics2d::shapes::{Circle, Rectangle, ShapeKind, ShapeRenderable, ShapeStyle};

thread_local! {
    static CAMERA_CENTER: Cell<(f32, f32)> = Cell::new((0.0, 0.0));
    static CAMERA_SCALE: Cell<f32> = Cell::new(1.0);
    static MOUSE_POS: Cell<(f64, f64)> = Cell::new((0.0, 0.0));
}

/// A shape with its world position and renderable.
struct WorldShape {
    world_x: f32,
    world_y: f32,
    renderable: ShapeRenderable,
}

impl WorldShape {
    fn new(world_x: f32, world_y: f32, shape: ShapeKind, color: Color) -> Self {
        // Create renderable once (allocates GPU resources)
        let renderable = ShapeRenderable::from_shape(
            0.0, 0.0, // Initial screen position (will be updated)
            shape,
            ShapeStyle::fill(color),
        );
        Self { world_x, world_y, renderable }
    }

    fn update_and_render(&mut self, camera: &Camera2D, renderer: &Renderer) {
        // Transform world position to screen position
        let screen_pos = camera.world_to_screen(Vec2::new(self.world_x, self.world_y));

        // Update position (no GPU allocation, just updates offset)
        self.renderable.set_position(screen_pos.x, screen_pos.y);

        // Render
        self.renderable.render(renderer);
    }
}

fn main() {
    let mut window = Window::new("Camera2D Example", 800, 600, Color::from_rgb(0.1, 0.1, 0.15));
    let renderer = Renderer::new(window.handle());

    // Handle scroll for zoom
    window.on_scroll(move |_, y_offset| {
        let zoom_factor = if y_offset > 0.0 { 1.1 } else { 1.0 / 1.1 };

        // Get current mouse position for zoom-at-cursor
        let mouse_pos = MOUSE_POS.with(|m| m.get());

        // Get current camera state
        let center = CAMERA_CENTER.with(|c| c.get());
        let scale = CAMERA_SCALE.with(|s| s.get());

        // Create temporary camera to compute zoom
        let mut camera = Camera2D::new(
            Vec2::new(center.0, center.1),
            scale,
            Vec2::new(800.0, 600.0),
        );

        // Zoom at cursor position
        camera.zoom_at(zoom_factor, Vec2::new(mouse_pos.0 as f32, mouse_pos.1 as f32));

        // Clamp scale
        let new_scale = camera.scale().clamp(0.1, 50.0);
        camera.set_scale(new_scale);

        // Update stored state
        CAMERA_CENTER.with(|c| c.set((camera.center().x, camera.center().y)));
        CAMERA_SCALE.with(|s| s.set(camera.scale()));

        println!(
            "scale: {:.2}, center: ({:.1}, {:.1})",
            camera.scale(),
            camera.center().x,
            camera.center().y
        );
    });

    // Track mouse position for zoom-at-cursor
    window.on_cursor_position(move |x, y| {
        MOUSE_POS.with(|m| m.set((x, y)));
    });

    // Create shapes ONCE (allocates GPU resources)
    let mut shapes = vec![
        // Grid of circles at various world positions
        WorldShape::new(0.0, 0.0, ShapeKind::Circle(Circle::new(30.0)), Color::from_rgb(1.0, 0.3, 0.3)),
        WorldShape::new(100.0, 0.0, ShapeKind::Circle(Circle::new(30.0)), Color::from_rgb(0.3, 1.0, 0.3)),
        WorldShape::new(200.0, 0.0, ShapeKind::Circle(Circle::new(30.0)), Color::from_rgb(0.3, 0.3, 1.0)),
        WorldShape::new(-100.0, 0.0, ShapeKind::Circle(Circle::new(30.0)), Color::from_rgb(1.0, 1.0, 0.3)),
        WorldShape::new(-200.0, 0.0, ShapeKind::Circle(Circle::new(30.0)), Color::from_rgb(1.0, 0.3, 1.0)),
        WorldShape::new(0.0, 100.0, ShapeKind::Circle(Circle::new(30.0)), Color::from_rgb(0.3, 1.0, 1.0)),
        WorldShape::new(0.0, -100.0, ShapeKind::Circle(Circle::new(30.0)), Color::from_rgb(1.0, 0.6, 0.3)),
        WorldShape::new(0.0, 200.0, ShapeKind::Circle(Circle::new(30.0)), Color::from_rgb(0.6, 0.3, 1.0)),
        WorldShape::new(0.0, -200.0, ShapeKind::Circle(Circle::new(30.0)), Color::from_rgb(0.3, 0.6, 0.3)),
        // Rectangles at corners
        WorldShape::new(150.0, 150.0, ShapeKind::Rectangle(Rectangle::new(80.0, 50.0)), Color::from_rgb(0.8, 0.4, 0.2)),
        WorldShape::new(-150.0, -150.0, ShapeKind::Rectangle(Rectangle::new(60.0, 90.0)), Color::from_rgb(0.2, 0.4, 0.8)),
        WorldShape::new(-150.0, 150.0, ShapeKind::Rectangle(Rectangle::new(70.0, 70.0)), Color::from_rgb(0.4, 0.8, 0.4)),
        WorldShape::new(150.0, -150.0, ShapeKind::Rectangle(Rectangle::new(50.0, 80.0)), Color::from_rgb(0.8, 0.8, 0.2)),
    ];

    // Origin marker (small white circle at world origin)
    let mut origin_marker = WorldShape::new(
        0.0, 0.0,
        ShapeKind::Circle(Circle::new(5.0)),
        Color::white(),
    );

    let mut app = App::new(window);

    app.on_render(move || {
        // Get current camera state
        let center = CAMERA_CENTER.with(|c| c.get());
        let scale = CAMERA_SCALE.with(|s| s.get());

        let camera = Camera2D::new(
            Vec2::new(center.0, center.1),
            scale,
            Vec2::new(800.0, 600.0),
        );

        // Update positions and render (no allocations per frame)
        for shape in &mut shapes {
            shape.update_and_render(&camera, &renderer);
        }

        // Render origin marker on top
        origin_marker.update_and_render(&camera, &renderer);
    });

    println!("Camera2D Example");
    println!("  Scroll: zoom in/out (zooms toward cursor)");
    println!("");
    println!("Shapes are in world coordinates, camera transforms to screen.");
    println!("Shape sizes stay constant; only positions change with zoom.");

    app.run();
}
