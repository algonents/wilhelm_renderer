extern crate wilhelm_renderer;

use wilhelm_renderer::core::{App, Color, Window};
use wilhelm_renderer::graphics2d::shapes::{ShapeKind, ShapeRenderable, ShapeStyle, Text};

fn main() {
    let window = Window::new("Text Rendering Example", 800, 600, Color::from_rgb(0.07, 0.13, 0.17));
    let mut app = App::new(window);

    let text = |pos: (f32, f32), kind: ShapeKind, style: ShapeStyle| -> ShapeRenderable {
        let mut s = ShapeRenderable::from_shape(kind, style);
        s.set_position(pos.0, pos.1);
        s
    };

    app.add_shapes(vec![
        // Create text with white color
        text((100.0, 100.0),
            ShapeKind::Text(Text::new("Hello, World!", "../../fonts/DejaVuSans.ttf", 48)),
            ShapeStyle { fill: Some(Color::white()), ..Default::default() },
        ),
        // Red text
        text((100.0, 200.0),
            ShapeKind::Text(Text::new("Red Text", "../../fonts/DejaVuSans.ttf", 36)),
            ShapeStyle { fill: Some(Color::from_rgb(1.0, 0.0, 0.0)), ..Default::default() },
        ),
        // Green text
        text((100.0, 280.0),
            ShapeKind::Text(Text::new("Green Text", "../../fonts/DejaVuSans.ttf", 36)),
            ShapeStyle { fill: Some(Color::from_rgb(0.0, 1.0, 0.0)), ..Default::default() },
        ),
        // Blue text
        text((100.0, 360.0),
            ShapeKind::Text(Text::new("Blue Text", "../../fonts/DejaVuSans.ttf", 36)),
            ShapeStyle { fill: Some(Color::from_rgb(0.0, 0.0, 1.0)), ..Default::default() },
        ),
        // Smaller text
        text((100.0, 450.0),
            ShapeKind::Text(Text::new(
                "The quick brown fox jumps over the lazy dog",
                "../../fonts/DejaVuSans.ttf", 24,
            )),
            ShapeStyle { fill: Some(Color::from_rgb(0.8, 0.8, 0.8)), ..Default::default() },
        ),
    ]);

    app.run();
}
