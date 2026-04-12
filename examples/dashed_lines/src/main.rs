extern crate wilhelm_renderer;

use std::f32::consts::PI;
use wilhelm_renderer::core::{App, Color, Window};
use wilhelm_renderer::graphics2d::shapes::{
    Arc, Line, Polyline, Rectangle, ShapeKind, ShapeRenderable, ShapeStyle,
};

fn main() {
    let window = Window::new(
        "Dashed Lines",
        1000,
        700,
        Color::from_rgb(0.07, 0.13, 0.17),
    );
    let mut app = App::new(window);

    let mut shapes: Vec<ShapeRenderable> = Vec::new();

    // --- Row 1: Lines ---

    // Solid line (reference)
    let mut solid = ShapeRenderable::from_shape(
        ShapeKind::Line(Line::new((0.0, 0.0), (300.0, 0.0))),
        ShapeStyle::stroke(Color::from_rgb(0.5, 0.5, 0.5), 2.0),
    );
    solid.set_position(50.0, 80.0);
    shapes.push(solid);

    // Dashed line
    let mut dashed = ShapeRenderable::from_shape(
        ShapeKind::Line(Line::new((0.0, 0.0), (300.0, 0.0))),
        ShapeStyle::dashed_stroke(Color::from_rgb(0.2, 0.8, 0.4), 2.0, 15.0, 8.0),
    );
    dashed.set_position(50.0, 120.0);
    shapes.push(dashed);

    // Dotted line (short dash, equal gap)
    let mut dotted = ShapeRenderable::from_shape(
        ShapeKind::Line(Line::new((0.0, 0.0), (300.0, 0.0))),
        ShapeStyle::dashed_stroke(Color::from_rgb(0.9, 0.3, 0.3), 3.0, 3.0, 8.0),
    );
    dotted.set_position(50.0, 160.0);
    shapes.push(dotted);

    // Long dash, short gap
    let mut long_dash = ShapeRenderable::from_shape(
        ShapeKind::Line(Line::new((0.0, 0.0), (300.0, 0.0))),
        ShapeStyle::dashed_stroke(Color::from_rgb(0.3, 0.5, 0.9), 2.0, 25.0, 5.0),
    );
    long_dash.set_position(50.0, 200.0);
    shapes.push(long_dash);

    // --- Row 2: Dashed polyline (sine wave) ---

    let num_points = 100;
    let sine_points: Vec<(f32, f32)> = (0..num_points)
        .map(|i| {
            let t = i as f32 / (num_points - 1) as f32;
            let x = t * 400.0;
            let y = 40.0 * (t * 4.0 * PI).sin();
            (x, y)
        })
        .collect();

    let mut sine = ShapeRenderable::from_shape(
        ShapeKind::Polyline(Polyline::new(sine_points)),
        ShapeStyle::dashed_stroke(Color::from_rgb(1.0, 0.7, 0.1), 2.0, 10.0, 6.0),
    );
    sine.set_position(50.0, 320.0);
    shapes.push(sine);

    // --- Row 2 right: Dashed rectangle outline ---

    let mut rect = ShapeRenderable::from_shape(
        ShapeKind::Rectangle(Rectangle::new(160.0, 100.0)),
        ShapeStyle::dashed_stroke(Color::from_rgb(0.2, 0.6, 0.9), 2.0, 12.0, 6.0),
    );
    rect.set_position(550.0, 270.0);
    shapes.push(rect);

    // --- Row 3: Dashed arc ---

    let mut arc = ShapeRenderable::from_shape(
        ShapeKind::Arc(Arc::new(80.0, 0.0, PI * 1.5)),
        ShapeStyle::dashed_stroke(Color::from_rgb(0.9, 0.4, 0.7), 2.0, 8.0, 5.0),
    );
    arc.set_position(150.0, 530.0);
    shapes.push(arc);

    // --- Row 3 right: Diagonal dashed line ---

    let mut diag = ShapeRenderable::from_shape(
        ShapeKind::Line(Line::new((0.0, 0.0), (250.0, 150.0))),
        ShapeStyle::dashed_stroke(Color::from_rgb(0.7, 0.9, 0.3), 2.0, 10.0, 5.0),
    );
    diag.set_position(550.0, 450.0);
    shapes.push(diag);

    // --- Row 3: Filled rect with dashed stroke ---

    let mut fill_dash = ShapeRenderable::from_shape(
        ShapeKind::Rectangle(Rectangle::new(120.0, 80.0)),
        ShapeStyle::fill_and_stroke(
            Color::from_rgb(0.15, 0.25, 0.35),
            Color::from_rgb(0.4, 0.8, 0.8),
            2.0,
        )
        .with_dash(10.0, 5.0),
    );
    fill_dash.set_position(350.0, 490.0);
    shapes.push(fill_dash);

    app.add_shapes(shapes);
    app.run();
}
