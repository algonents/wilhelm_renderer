extern crate wilhelm_renderer;

use std::f32::consts::PI;
use wilhelm_renderer::core::{App, Color, Window};
use wilhelm_renderer::graphics2d::shapes::{
    Circle, Rectangle, ShapeKind, ShapeRenderable, ShapeStyle, Triangle,
};

fn create_equilateral_triangle(size: f32) -> [(f32, f32); 3] {
    let height = (3.0f32).sqrt() / 2.0 * size;
    [
        (0.0, -2.0 * height / 3.0),  // Top vertex
        (-0.5 * size, height / 3.0), // Bottom left
        (0.5 * size, height / 3.0),  // Bottom right
    ]
}

fn main() {
    let window = Window::new("Rotations", 800, 600, Color::from_rgb(0.07, 0.13, 0.17));
    let mut app = App::new(window);

    // Triangle (vertices centered at origin)
    let mut triangle = ShapeRenderable::from_shape(
        150.0, 150.0,
        ShapeKind::Triangle(Triangle::new(create_equilateral_triangle(80.0))),
        ShapeStyle::fill(Color::from_rgb(1.0, 0.3, 0.3)),
    );
    triangle.set_rotation(PI / 4.0); // 45 degrees

    // Rectangle - geometry starts at (0,0) so rotation is around top-left corner
    let mut rectangle = ShapeRenderable::from_shape(
        200.0, 400.0,
        ShapeKind::Rectangle(Rectangle::new(100.0, 60.0)),
        ShapeStyle::fill(Color::from_rgb(0.3, 1.0, 0.3)),
    );
    rectangle.set_rotation(PI / 6.0); // 30 degrees

    // Circle (rotation has no visible effect on a circle)
    let circle = ShapeRenderable::from_shape(
        550.0, 150.0,
        ShapeKind::Circle(Circle::new(50.0)),
        ShapeStyle::fill(Color::from_rgb(0.3, 0.3, 1.0)),
    );

    // Rectangle with combined scale and rotation
    let mut scaled_rotated = ShapeRenderable::from_shape(
        550.0, 350.0,
        ShapeKind::Rectangle(Rectangle::new(80.0, 40.0)),
        ShapeStyle::fill(Color::from_rgb(1.0, 1.0, 0.3)),
    );
    scaled_rotated.set_rotation(PI / 3.0); // 60 degrees
    scaled_rotated.set_scale(1.5);

    // Image rotation
    let bunny = ShapeRenderable::image(400.0, 300.0, "images/bunny.png");

    app.add_shapes(vec![triangle, rectangle, circle, scaled_rotated, bunny]);

    app.on_pre_render(move |shapes, renderer| {
        let time = renderer.get_time() as f32;

        // Rotate triangle continuously
        if let Some(shape) = shapes.get_mut(0) {
            shape.set_rotation(time);
        }

        // Rotate rectangle at different speed
        if let Some(shape) = shapes.get_mut(1) {
            shape.set_rotation(time * 1.1);
        }

        // Rotate with pulsing scale
        if let Some(shape) = shapes.get_mut(3) {
            shape.set_rotation(-time * 0.7);
            shape.set_scale(1.0 + 0.3 * (time * 2.0).sin());
        }

        // Rotate bunny image
        if let Some(shape) = shapes.get_mut(4) {
            shape.set_rotation(time * 2.5);
        }
    });

    app.run();
}
