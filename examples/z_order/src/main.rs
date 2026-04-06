use wilhelm_renderer::core::{App, Color, Window};
use wilhelm_renderer::graphics2d::shapes::{
    Circle, Rectangle, ShapeKind, ShapeRenderable, ShapeStyle, Text,
};

fn main() {
    let window = Window::new("Z-Order", 800, 600, Color::from_rgb(0.07, 0.13, 0.17));
    let mut app = App::new(window);

    let shape = |pos: (f32, f32), kind: ShapeKind, style: ShapeStyle, z: i32| -> ShapeRenderable {
        let mut s = ShapeRenderable::from_shape(kind, style);
        s.set_position(pos.0, pos.1);
        s.set_z_order(z);
        s
    };

    app.add_shapes(vec![
        // Label
        shape((310.0, 30.0),
            ShapeKind::Text(Text::new("Z-Order Demo", "../../fonts/DejaVuSans.ttf", 28)),
            ShapeStyle::fill(Color::from_rgb(0.8, 0.8, 0.8)),
            10,
        ),

        // Three overlapping rectangles — inserted in order red, green, blue
        // but z-order puts blue behind, green in middle, red on top
        shape((200.0, 150.0),
            ShapeKind::Rectangle(Rectangle::new(200.0, 150.0)),
            ShapeStyle::fill(Color::from_rgba(0.2, 0.4, 0.9, 0.9)),
            0, // blue: back
        ),
        shape((250.0, 200.0),
            ShapeKind::Rectangle(Rectangle::new(200.0, 150.0)),
            ShapeStyle::fill(Color::from_rgba(0.2, 0.8, 0.2, 0.9)),
            1, // green: middle
        ),
        shape((300.0, 250.0),
            ShapeKind::Rectangle(Rectangle::new(200.0, 150.0)),
            ShapeStyle::fill(Color::from_rgba(0.9, 0.2, 0.2, 0.9)),
            2, // red: front
        ),

        // Two overlapping circles — orange inserted first but has higher z-order
        shape((600.0, 280.0),
            ShapeKind::Circle(Circle::new(80.0)),
            ShapeStyle::fill(Color::from_rgba(1.0, 0.6, 0.0, 0.9)),
            2, // orange: front (despite being added first)
        ),
        shape((650.0, 320.0),
            ShapeKind::Circle(Circle::new(80.0)),
            ShapeStyle::fill(Color::from_rgba(0.6, 0.0, 1.0, 0.9)),
            1, // purple: behind orange
        ),

        // Labels for each group
        shape((240.0, 430.0),
            ShapeKind::Text(Text::new("z: 0, 1, 2", "../../fonts/DejaVuSans.ttf", 16)),
            ShapeStyle::fill(Color::from_rgb(0.6, 0.6, 0.6)),
            10,
        ),
        shape((580.0, 430.0),
            ShapeKind::Text(Text::new("insertion order reversed", "../../fonts/DejaVuSans.ttf", 16)),
            ShapeStyle::fill(Color::from_rgb(0.6, 0.6, 0.6)),
            10,
        ),
    ]);

    app.run();
}
