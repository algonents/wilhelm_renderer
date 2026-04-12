extern crate wilhelm_renderer;

use wilhelm_renderer::core::{App, Color, Window};
use wilhelm_renderer::graphics2d::shapes::{
    Anchor, Circle, Rectangle, ShapeKind, ShapeRenderable, ShapeStyle, Text,
};

const RECT_W: f32 = 120.0;
const RECT_H: f32 = 60.0;

/// Background color for the window.
const BG: (f32, f32, f32) = (0.07, 0.13, 0.17);

/// Build a rectangle with a given anchor, position, and color.
fn make_rect(anchor: Anchor, x: f32, y: f32, color: Color) -> ShapeRenderable {
    let mut rect = ShapeRenderable::builder(
        ShapeKind::Rectangle(Rectangle::new(RECT_W, RECT_H)),
        ShapeStyle::fill_and_stroke(color, Color::white(), 1.0),
    )
    .anchor(anchor)
    .build();
    rect.set_position(x, y);
    rect
}

/// Small dot to mark the anchor position.
fn make_dot(x: f32, y: f32) -> ShapeRenderable {
    let mut dot = ShapeRenderable::from_shape(
        ShapeKind::Circle(Circle::new(4.0)),
        ShapeStyle::fill(Color::from_rgb(1.0, 0.2, 0.2)),
    );
    dot.set_position(x, y);
    dot.set_z_order(1);
    dot
}

/// Label placed below each anchor position.
fn make_label(text: &str, x: f32, y: f32) -> ShapeRenderable {
    let mut label = ShapeRenderable::builder(
        ShapeKind::Text(Text::new(text, "../../fonts/DejaVuSans.ttf", 16)),
        ShapeStyle {
            fill: Some(Color::from_rgb(0.7, 0.7, 0.7)),
            ..Default::default()
        },
    )
    .anchor(Anchor::North)
    .build();
    label.set_position(x, y);
    label
}

fn main() {
    let window = Window::new(
        "Anchor Rotations",
        1000,
        700,
        Color::from_rgb(BG.0, BG.1, BG.2),
    );
    let mut app = App::new(window);

    // Layout: 3 columns x 3 rows
    let cols = [200.0_f32, 500.0, 800.0];
    let rows = [150.0_f32, 350.0, 550.0];

    let anchors: [(Anchor, &str); 9] = [
        (Anchor::NorthWest, "NorthWest"),
        (Anchor::North, "North"),
        (Anchor::NorthEast, "NorthEast"),
        (Anchor::West, "West"),
        (Anchor::Center, "Center"),
        (Anchor::East, "East"),
        (Anchor::SouthWest, "SouthWest"),
        (Anchor::South, "South"),
        (Anchor::SouthEast, "SouthEast"),
    ];

    let colors = [
        Color::from_rgb(0.26, 0.52, 0.96), // blue
        Color::from_rgb(0.18, 0.80, 0.44), // green
        Color::from_rgb(0.90, 0.49, 0.13), // orange
        Color::from_rgb(0.61, 0.35, 0.71), // purple
        Color::from_rgb(0.20, 0.74, 0.74), // teal
        Color::from_rgb(0.95, 0.77, 0.06), // yellow
        Color::from_rgb(0.91, 0.30, 0.24), // red
        Color::from_rgb(0.56, 0.70, 0.32), // olive
        Color::from_rgb(0.83, 0.33, 0.61), // pink
    ];

    let mut shapes: Vec<ShapeRenderable> = Vec::new();

    // First, add the 9 rectangles (indices 0..9) so we can rotate them.
    for (i, (anchor, _name)) in anchors.iter().enumerate() {
        let row = i / 3;
        let col = i % 3;
        let x = cols[col];
        let y = rows[row];
        shapes.push(make_rect(*anchor, x, y, colors[i]));
    }

    // Then add dots and labels (static, not rotated).
    for (i, (_anchor, name)) in anchors.iter().enumerate() {
        let row = i / 3;
        let col = i % 3;
        let x = cols[col];
        let y = rows[row];
        shapes.push(make_dot(x, y));
        shapes.push(make_label(name, x, y + 55.0));
    }

    // Title
    let mut title = ShapeRenderable::builder(
        ShapeKind::Text(Text::new(
            "Rectangle Rotation by Anchor Point",
            "../../fonts/DejaVuSans.ttf",
            28,
        )),
        ShapeStyle {
            fill: Some(Color::white()),
            ..Default::default()
        },
    )
    .anchor(Anchor::North)
    .build();
    title.set_position(500.0, 30.0);
    shapes.push(title);

    app.add_shapes(shapes);

    // Rotate only the 9 rectangles (indices 0..9).
    app.on_pre_render(move |shapes, renderer| {
        let time = renderer.get_time() as f32;
        for i in 0..9 {
            if let Some(shape) = shapes.get_mut(i) {
                shape.set_rotation(time * 0.8);
            }
        }
    });

    app.run();
}
