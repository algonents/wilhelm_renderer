use wilhelm_renderer::core::{App, Color, Window};
use wilhelm_renderer::graphics2d::shapes::{
    Circle, Ellipse, Polygon, Rectangle, RoundedRectangle, ShapeKind, ShapeRenderable, ShapeStyle,
    Text,
};

fn main() {
    // Eggshell background
    let window = Window::new(
        "Alpha Transparency",
        800,
        800,
        Color::from_rgb(0.94, 0.92, 0.84),
    );
    let mut app = App::new(window);

    let shape = |pos: (f32, f32), kind: ShapeKind, style: ShapeStyle| -> ShapeRenderable {
        let mut s = ShapeRenderable::from_shape(kind, style);
        s.set_position(pos.0, pos.1);
        s
    };

    // Overlapping circles at different opacities
    let circle_r = 80.0;
    let overlap = 50.0;
    let cx = 200.0;
    let cy = 180.0;

    // Three overlapping rectangles with decreasing opacity
    let rect_x = 500.0;
    let rect_y = 80.0;

    // Hexagon points
    let hex_r = 70.0;
    let hex_points: Vec<(f32, f32)> = (0..6)
        .map(|i| {
            let angle = std::f32::consts::PI / 3.0 * i as f32 - std::f32::consts::PI / 6.0;
            (hex_r * angle.cos(), hex_r * angle.sin())
        })
        .collect();

    app.add_shapes(vec![
        // Title
        shape(
            (20.0, 30.0),
            ShapeKind::Text(Text::new(
                "Alpha Transparency Demo",
                "../../fonts/DejaVuSans.ttf",
                28,
            )),
            ShapeStyle::fill(Color::from_rgba(0.2, 0.2, 0.2, 1.0)),
        ),
        // --- Overlapping circles (RGB at 60% opacity) ---
        shape(
            (50.0, 80.0),
            ShapeKind::Text(Text::new(
                "Overlapping circles (60% opacity)",
                "../../fonts/DejaVuSans.ttf",
                14,
            )),
            ShapeStyle::fill(Color::from_rgba(0.3, 0.3, 0.3, 1.0)),
        ),
        shape(
            (cx, cy),
            ShapeKind::Circle(Circle::new(circle_r)),
            ShapeStyle::fill(Color::from_rgba(1.0, 0.0, 0.0, 0.6)),
        ),
        shape(
            (cx + overlap + 30.0, cy),
            ShapeKind::Circle(Circle::new(circle_r)),
            ShapeStyle::fill(Color::from_rgba(0.0, 0.7, 0.0, 0.6)),
        ),
        shape(
            (cx + (overlap + 30.0) / 2.0, cy + overlap),
            ShapeKind::Circle(Circle::new(circle_r)),
            ShapeStyle::fill(Color::from_rgba(0.0, 0.0, 1.0, 0.6)),
        ),
        // --- Rectangles with gradient of opacity ---
        shape(
            (rect_x - 20.0, 80.0),
            ShapeKind::Text(Text::new(
                "Opacity gradient: 80%, 50%, 20%",
                "../../fonts/DejaVuSans.ttf",
                14,
            )),
            ShapeStyle::fill(Color::from_rgba(0.3, 0.3, 0.3, 1.0)),
        ),
        shape(
            (rect_x, rect_y + 30.0),
            ShapeKind::Rectangle(Rectangle::new(180.0, 60.0)),
            ShapeStyle::fill(Color::from_rgba(0.2, 0.4, 0.8, 0.8)),
        ),
        shape(
            (rect_x + 30.0, rect_y + 70.0),
            ShapeKind::Rectangle(Rectangle::new(180.0, 60.0)),
            ShapeStyle::fill(Color::from_rgba(0.2, 0.4, 0.8, 0.5)),
        ),
        shape(
            (rect_x + 60.0, rect_y + 110.0),
            ShapeKind::Rectangle(Rectangle::new(180.0, 60.0)),
            ShapeStyle::fill(Color::from_rgba(0.2, 0.4, 0.8, 0.2)),
        ),
        // --- Semi-transparent polygon (hexagon) ---
        shape(
            (50.0, 370.0),
            ShapeKind::Text(Text::new(
                "Semi-transparent shapes",
                "../../fonts/DejaVuSans.ttf",
                14,
            )),
            ShapeStyle::fill(Color::from_rgba(0.3, 0.3, 0.3, 1.0)),
        ),
        shape(
            (150.0, 480.0),
            ShapeKind::Polygon(Polygon::new(hex_points)),
            ShapeStyle::fill(Color::from_rgba(0.8, 0.2, 0.8, 0.5)),
        ),
        // Ellipse behind/overlapping the hexagon
        shape(
            (200.0, 500.0),
            ShapeKind::Ellipse(Ellipse::new(100.0, 50.0)),
            ShapeStyle::fill(Color::from_rgba(1.0, 0.6, 0.0, 0.4)),
        ),
        // Rounded rectangle
        shape(
            (80.0, 550.0),
            ShapeKind::RoundedRectangle(RoundedRectangle::new(200.0, 80.0, 15.0)),
            ShapeStyle::fill(Color::from_rgba(0.0, 0.7, 0.5, 0.35)),
        ),
        // --- Semi-transparent text ---
        shape(
            (450.0, 370.0),
            ShapeKind::Text(Text::new(
                "Semi-transparent text",
                "../../fonts/DejaVuSans.ttf",
                14,
            )),
            ShapeStyle::fill(Color::from_rgba(0.3, 0.3, 0.3, 1.0)),
        ),
        shape(
            (450.0, 410.0),
            ShapeKind::Text(Text::new(
                "100% opacity",
                "../../fonts/DejaVuSans.ttf",
                24,
            )),
            ShapeStyle::fill(Color::from_rgba(0.8, 0.1, 0.1, 1.0)),
        ),
        shape(
            (450.0, 450.0),
            ShapeKind::Text(Text::new(
                "70% opacity",
                "../../fonts/DejaVuSans.ttf",
                24,
            )),
            ShapeStyle::fill(Color::from_rgba(0.8, 0.1, 0.1, 0.7)),
        ),
        shape(
            (450.0, 490.0),
            ShapeKind::Text(Text::new(
                "40% opacity",
                "../../fonts/DejaVuSans.ttf",
                24,
            )),
            ShapeStyle::fill(Color::from_rgba(0.8, 0.1, 0.1, 0.4)),
        ),
        shape(
            (450.0, 530.0),
            ShapeKind::Text(Text::new(
                "15% opacity",
                "../../fonts/DejaVuSans.ttf",
                24,
            )),
            ShapeStyle::fill(Color::from_rgba(0.8, 0.1, 0.1, 0.15)),
        ),
    ]);

    app.run();
}
