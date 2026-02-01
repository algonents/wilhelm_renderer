extern crate wilhelm_renderer;

use wilhelm_renderer::core::{App, Color, Window};
use wilhelm_renderer::graphics2d::shapes::{Arc, Circle, Ellipse, Line, MultiPoint, Polygon, Polyline, Rectangle, RoundedRectangle, ShapeKind, ShapeRenderable, ShapeStyle, Text, Triangle};

fn create_equilateral_triangle() -> [(f32, f32); 3] {
    let side = 20.0;
    let height = (3.0f32).sqrt() / 2.0 * side;

    let vertices = [
        (0.0, 2.0 * -height / 3.0),  // Top vertex
        (-0.5 * side, height / 3.0), // Bottom left
        (0.5 * side, height / 3.0),  // Bottom right
    ];
    vertices
}

fn generate_sine_wave_local(
    amplitude: f32,
    points: usize,
    wavelength: f32,
) -> Vec<(f32, f32)> {
    let mut result = Vec::with_capacity(points);
    let dx = wavelength / (points - 1) as f32;

    for i in 0..points {
        let x = i as f32 * dx;
        let y = amplitude * (x / wavelength * std::f32::consts::TAU).sin();
        result.push((x, y));
    }

    result
}



fn main() {
    let window = Window::new("Shapes", 800, 800, Color::from_rgb(0.07, 0.13, 0.17));
    let mut app = App::new(window);
    app.renderer().set_point_size(6.0);

    // Convert polyline points to relative coordinates (relative to first point)
    let polyline_points = vec![(0.0, 0.0), (50.0, 130.0), (100.0, 110.0), (100.0, 200.0)];

    let polygon_local: Vec<(f32, f32)> = vec![
        (0.0, 0.0),          // anchor vertex
        (-25.0, 43.3),
        (-75.0, 43.3),
        (-100.0, 0.0),
        (-75.0, -43.4),
        (-25.0, -43.4),
    ];

    let sine_wave_local = generate_sine_wave_local(
        30.0,   // amplitude
        200,     // points
        100.0,  // wavelength
    );

    app.add_shapes(vec![
        // Text
        ShapeRenderable::from_shape(
            160.0, 280.0,
            ShapeKind::Text(Text::new("Hello, Wilhelm renderer!", "fonts/ArchitectsDaughter-Regular.ttf", 48)),
            ShapeStyle::fill(Color::from_rgb(0.94, 0.91, 0.78)),
        ),
        // Line from (100, 200) to (300, 250)
        ShapeRenderable::from_shape(
            0.0, 0.0,
            ShapeKind::Line(Line::new((100.0, 200.0), (300.0, 250.0))),
            ShapeStyle::stroke(Color::from_rgb(0.0, 1.0, 0.0), 5.0),
        ),
        // Polyline starting at (100, 300)
        ShapeRenderable::from_shape(
            100.0, 300.0,
            ShapeKind::Polyline(Polyline::new(polyline_points)),
            ShapeStyle::stroke(Color::from_rgb(1.0, 0.0, 0.0), 10.0),
        ),
        // Arc centered at (700, 600)
        ShapeRenderable::from_shape(
            700.0, 600.0,
            ShapeKind::Arc(Arc::new(70.0, 0.0, std::f32::consts::PI / 2.0)),
            ShapeStyle::stroke(Color::from_rgb(0.0, 0.0, 1.0), 10.0),
        ),
        // Rectangle at (50, 50)
        ShapeRenderable::from_shape(
            50.0, 50.0,
            ShapeKind::Rectangle(Rectangle::new(200.0, 80.0)),
            ShapeStyle::fill(Color::from_rgb(0.2, 0.5, 0.9)),
        ),
        // Triangle at (50, 50)
        ShapeRenderable::from_shape(
            50.0, 50.0,
            ShapeKind::Triangle(Triangle::new(create_equilateral_triangle())),
            ShapeStyle::fill(Color::from_rgb(1.0, 0.0, 0.0)),
        ),
        // Rectangle at (400, 200)
        ShapeRenderable::from_shape(
            400.0, 200.0,
            ShapeKind::Rectangle(Rectangle::new(100.0, 50.0)),
            ShapeStyle::fill(Color::from_rgb(1.0, 0.0, 0.0)),
        ),
        // Circle at (400, 400)
        ShapeRenderable::from_shape(
            400.0, 400.0,
            ShapeKind::Circle(Circle::new(50.0)),
            ShapeStyle::fill(Color::from_rgb(0.0, 0.0, 1.0)),
        ),
        // Point at (650, 260)
        ShapeRenderable::from_shape(
            650.0, 260.0,
            ShapeKind::Point,
            ShapeStyle::fill(Color::from_rgb(1.0, 0.0, 0.0)),
        ),
        // MultiPoint (sine wave)
        ShapeRenderable::from_shape(
            500.0, 100.0,
            ShapeKind::MultiPoint(MultiPoint::new(sine_wave_local)),
            ShapeStyle::fill(Color::from_rgb(0.0, 0.0, 1.0)),
        ),
        // Ellipse at (600, 200)
        ShapeRenderable::from_shape(
            600.0, 200.0,
            ShapeKind::Ellipse(Ellipse::new(80.0, 40.0)),
            ShapeStyle::fill(Color::from_rgb(0.5, 0.2, 0.8)),
        ),
        // Rounded rectangle at (100, 600)
        ShapeRenderable::from_shape(
            100.0, 600.0,
            ShapeKind::RoundedRectangle(RoundedRectangle::new(200.0, 80.0, 10.0)),
            ShapeStyle::fill(Color::from_rgb(0.3, 0.6, 0.9)),
        ),
        // Polygon (hexagon)
        ShapeRenderable::from_shape(
            600.0, 600.0,
            ShapeKind::Polygon(Polygon::new(polygon_local)),
            ShapeStyle::fill(Color::from_rgb(1.0, 0.0, 0.0)),
        ),
        // Rectangle at (600, 400)
        ShapeRenderable::from_shape(
            600.0, 400.0,
            ShapeKind::Rectangle(Rectangle::new(100.0, 50.0)),
            ShapeStyle::fill(Color::from_rgb(0.0, 1.0, 0.0)),
        ),
        // Outlined rectangle at (270, 50)
        ShapeRenderable::from_shape(
            270.0, 50.0,
            ShapeKind::Rectangle(Rectangle::new(200.0, 80.0)),
            ShapeStyle::stroke(Color::from_rgb(0.2, 0.5, 0.9), 3.0),
        ),
        // Images
        ShapeRenderable::image_with_size(200.0, 540.0, "images/smiley.png", 40.0, 40.0),
        ShapeRenderable::image(400.0, 500.0, "images/bunny.png"),
    ]);

    app.run();
}
