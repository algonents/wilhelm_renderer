extern crate wilhelm_renderer;

use std::f32::consts::PI;
use wilhelm_renderer::core::{App, Color, Window};
use wilhelm_renderer::graphics2d::shapes::{
    Arc, Circle, Ellipse, Line, Polygon, Polyline, Rectangle, ShapeKind, ShapeRenderable,
    ShapeStyle, Triangle,
};

fn main() {
    let window = Window::new(
        "Dashed Lines & Shapes",
        1200,
        900,
        Color::from_rgb(0.07, 0.13, 0.17),
    );
    let mut app = App::new(window);

    let mut shapes: Vec<ShapeRenderable> = Vec::new();

    // =============================================
    // Left column: Dashed lines and polyline
    // =============================================

    // Solid line (reference)
    let mut solid = ShapeRenderable::from_shape(
        ShapeKind::Line(Line::new((0.0, 0.0), (300.0, 0.0))),
        ShapeStyle::stroke(Color::from_rgb(0.5, 0.5, 0.5), 2.0),
    );
    solid.set_position(50.0, 60.0);
    shapes.push(solid);

    // Dashed line
    let mut dashed = ShapeRenderable::from_shape(
        ShapeKind::Line(Line::new((0.0, 0.0), (300.0, 0.0))),
        ShapeStyle::dashed_stroke(Color::from_rgb(0.2, 0.8, 0.4), 2.0, 15.0, 8.0),
    );
    dashed.set_position(50.0, 100.0);
    shapes.push(dashed);

    // Dotted line
    let mut dotted = ShapeRenderable::from_shape(
        ShapeKind::Line(Line::new((0.0, 0.0), (300.0, 0.0))),
        ShapeStyle::dashed_stroke(Color::from_rgb(0.9, 0.3, 0.3), 3.0, 3.0, 8.0),
    );
    dotted.set_position(50.0, 140.0);
    shapes.push(dotted);

    // Long dash
    let mut long_dash = ShapeRenderable::from_shape(
        ShapeKind::Line(Line::new((0.0, 0.0), (300.0, 0.0))),
        ShapeStyle::dashed_stroke(Color::from_rgb(0.3, 0.5, 0.9), 2.0, 25.0, 5.0),
    );
    long_dash.set_position(50.0, 180.0);
    shapes.push(long_dash);

    // Dashed polyline (sine wave)
    let num_points = 100;
    let sine_points: Vec<(f32, f32)> = (0..num_points)
        .map(|i| {
            let t = i as f32 / (num_points - 1) as f32;
            (t * 350.0, 35.0 * (t * 4.0 * PI).sin())
        })
        .collect();
    let mut sine = ShapeRenderable::from_shape(
        ShapeKind::Polyline(Polyline::new(sine_points)),
        ShapeStyle::dashed_stroke(Color::from_rgb(1.0, 0.7, 0.1), 2.0, 10.0, 6.0),
    );
    sine.set_position(50.0, 260.0);
    shapes.push(sine);

    // Diagonal dashed line
    let mut diag = ShapeRenderable::from_shape(
        ShapeKind::Line(Line::new((0.0, 0.0), (200.0, 100.0))),
        ShapeStyle::dashed_stroke(Color::from_rgb(0.7, 0.9, 0.3), 2.0, 10.0, 5.0),
    );
    diag.set_position(50.0, 340.0);
    shapes.push(diag);

    // Dashed arc
    let mut arc = ShapeRenderable::from_shape(
        ShapeKind::Arc(Arc::new(60.0, 0.0, PI * 1.5)),
        ShapeStyle::dashed_stroke(Color::from_rgb(0.9, 0.4, 0.7), 2.0, 8.0, 5.0),
    );
    arc.set_position(200.0, 530.0);
    shapes.push(arc);

    // =============================================
    // Middle column: Stroke-only shapes
    // =============================================

    // Dashed rectangle outline
    let mut rect = ShapeRenderable::from_shape(
        ShapeKind::Rectangle(Rectangle::new(140.0, 80.0)),
        ShapeStyle::dashed_stroke(Color::from_rgb(0.2, 0.6, 0.9), 2.0, 12.0, 6.0),
    );
    rect.set_position(450.0, 50.0);
    shapes.push(rect);

    // Dashed circle
    let mut dashed_circle = ShapeRenderable::from_shape(
        ShapeKind::Circle(Circle::new(45.0)),
        ShapeStyle::dashed_stroke(Color::from_rgb(0.3, 0.8, 0.9), 2.0, 8.0, 5.0),
    );
    dashed_circle.set_position(520.0, 230.0);
    shapes.push(dashed_circle);

    // Dashed ellipse
    let mut dashed_ellipse = ShapeRenderable::from_shape(
        ShapeKind::Ellipse(Ellipse::new(65.0, 35.0)),
        ShapeStyle::dashed_stroke(Color::from_rgb(0.9, 0.6, 0.2), 2.0, 10.0, 5.0),
    );
    dashed_ellipse.set_position(520.0, 380.0);
    shapes.push(dashed_ellipse);

    // Dashed triangle
    let tri_r = 40.0;
    let mut dashed_tri = ShapeRenderable::from_shape(
        ShapeKind::Triangle(Triangle::new([
            (0.0, -tri_r),
            (tri_r * 0.866, tri_r * 0.5),
            (-tri_r * 0.866, tri_r * 0.5),
        ])),
        ShapeStyle::dashed_stroke(Color::from_rgb(0.95, 0.45, 0.45), 2.0, 8.0, 4.0),
    );
    dashed_tri.set_position(520.0, 530.0);
    shapes.push(dashed_tri);

    // Dashed polygon (hexagon)
    let hex_points: Vec<(f32, f32)> = (0..6)
        .map(|i| {
            let angle = (i as f32) * PI / 3.0 - PI / 2.0;
            (40.0 * angle.cos(), 40.0 * angle.sin())
        })
        .collect();
    let mut dashed_hex = ShapeRenderable::from_shape(
        ShapeKind::Polygon(Polygon::new(hex_points)),
        ShapeStyle::dashed_stroke(Color::from_rgb(0.5, 0.9, 0.5), 2.0, 6.0, 4.0),
    );
    dashed_hex.set_position(520.0, 700.0);
    shapes.push(dashed_hex);

    // =============================================
    // Right column: Fill + dashed stroke
    // =============================================

    // Filled rect with dashed stroke
    let mut fill_rect = ShapeRenderable::from_shape(
        ShapeKind::Rectangle(Rectangle::new(140.0, 80.0)),
        ShapeStyle::fill_and_stroke(
            Color::from_rgb(0.15, 0.25, 0.35),
            Color::from_rgb(0.4, 0.8, 0.8),
            2.0,
        )
        .with_dash(10.0, 5.0),
    );
    fill_rect.set_position(830.0, 50.0);
    shapes.push(fill_rect);

    // Filled circle with dashed stroke
    let mut fill_circle = ShapeRenderable::from_shape(
        ShapeKind::Circle(Circle::new(45.0)),
        ShapeStyle::fill_and_stroke(
            Color::from_rgb(0.15, 0.20, 0.35),
            Color::from_rgb(0.9, 0.5, 0.2),
            2.0,
        )
        .with_dash(8.0, 4.0),
    );
    fill_circle.set_position(900.0, 230.0);
    shapes.push(fill_circle);

    // Filled ellipse with dashed stroke
    let mut fill_ellipse = ShapeRenderable::from_shape(
        ShapeKind::Ellipse(Ellipse::new(65.0, 35.0)),
        ShapeStyle::fill_and_stroke(
            Color::from_rgb(0.20, 0.15, 0.30),
            Color::from_rgb(0.9, 0.6, 0.2),
            2.0,
        )
        .with_dash(10.0, 5.0),
    );
    fill_ellipse.set_position(900.0, 380.0);
    shapes.push(fill_ellipse);

    // Filled triangle with dashed stroke
    let mut fill_tri = ShapeRenderable::from_shape(
        ShapeKind::Triangle(Triangle::new([
            (0.0, -tri_r),
            (tri_r * 0.866, tri_r * 0.5),
            (-tri_r * 0.866, tri_r * 0.5),
        ])),
        ShapeStyle::fill_and_stroke(
            Color::from_rgb(0.25, 0.15, 0.20),
            Color::from_rgb(0.85, 0.55, 0.90),
            2.0,
        )
        .with_dash(6.0, 4.0),
    );
    fill_tri.set_position(900.0, 530.0);
    shapes.push(fill_tri);

    // Filled hexagon with dashed stroke
    let hex_points2: Vec<(f32, f32)> = (0..6)
        .map(|i| {
            let angle = (i as f32) * PI / 3.0 - PI / 2.0;
            (40.0 * angle.cos(), 40.0 * angle.sin())
        })
        .collect();
    let mut fill_hex = ShapeRenderable::from_shape(
        ShapeKind::Polygon(Polygon::new(hex_points2)),
        ShapeStyle::fill_and_stroke(
            Color::from_rgb(0.15, 0.25, 0.15),
            Color::from_rgb(0.5, 0.9, 0.5),
            2.0,
        )
        .with_dash(6.0, 4.0),
    );
    fill_hex.set_position(900.0, 700.0);
    shapes.push(fill_hex);

    app.add_shapes(shapes);
    app.run();
}
