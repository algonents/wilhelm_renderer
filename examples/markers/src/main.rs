extern crate wilhelm_renderer;

use wilhelm_renderer::core::{App, Color, Window};
use wilhelm_renderer::graphics2d::markers::MarkerType;
use wilhelm_renderer::graphics2d::shapes::{
    Polygon, ShapeKind, ShapeRenderable, ShapeStyle, Text,
};

fn main() {
    let window = Window::new("Symbols", 800, 500, Color::from_rgb(0.07, 0.13, 0.17));
    let mut app = App::new(window);

    let symbols: Vec<(MarkerType, &str, Color)> = vec![
        (MarkerType::Circle,   "Circle",   Color::from_rgb(0.55, 0.85, 0.45)),
        (MarkerType::Cross,    "Cross",    Color::from_rgb(0.95, 0.45, 0.45)),
        (MarkerType::Diamond,  "Diamond",  Color::from_rgb(0.30, 0.78, 0.92)),
        (MarkerType::Square,   "Square",   Color::from_rgb(1.00, 0.78, 0.20)),
        (MarkerType::Star,     "Star",     Color::from_rgb(0.85, 0.55, 0.90)),
        (MarkerType::Triangle, "Triangle", Color::from_rgb(0.95, 0.65, 0.35)),
        (MarkerType::Wye,      "Wye",      Color::from_rgb(0.45, 0.75, 0.95)),
    ];

    let spacing = 800.0 / (symbols.len() as f32 + 1.0);
    let small_radius = 10.0;
    let large_radius = 30.0;

    let mut shapes = Vec::new();
    for (i, (symbol_type, label, color)) in symbols.iter().enumerate() {
        let x = spacing * (i as f32 + 1.0);

        // Row 1: small symbols
        let mut small = ShapeRenderable::from_shape(
            ShapeKind::Polygon(Polygon::new(symbol_type.vertices(small_radius))),
            ShapeStyle::fill(*color),
        );
        small.set_position(x, 100.0);
        shapes.push(small);

        // Row 2: large symbols
        let mut large = ShapeRenderable::from_shape(
            ShapeKind::Polygon(Polygon::new(symbol_type.vertices(large_radius))),
            ShapeStyle::fill(*color),
        );
        large.set_position(x, 260.0);
        shapes.push(large);

        // Label below the large symbol
        let mut label_shape = ShapeRenderable::from_shape(
            ShapeKind::Text(Text::new(*label, "../../fonts/DejaVuSans.ttf", 16)),
            ShapeStyle::fill(Color::from_rgb(0.80, 0.80, 0.80)),
        );
        label_shape.set_position(x - 20.0, 260.0 + large_radius + 40.0);
        shapes.push(label_shape);
    }

    app.add_shapes(shapes);
    app.run();
}
