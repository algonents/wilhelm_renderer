extern crate wilhelm_renderer;

use wilhelm_renderer::core::{App, Color, Renderable, Renderer, Window};
use wilhelm_renderer::graphics2d::shapes::{ShapeKind, ShapeRenderable, ShapeStyle, Text};

fn main() {
    let window = Window::new("Text Rendering Example", 800, 600);
    let renderer = Renderer::new(window.handle());
    let mut app = App::new(window);

    // Create text with white color
    let text = ShapeRenderable::from_shape(
        100.0,
        100.0,
        ShapeKind::Text(Text::new("Hello, World!", "fonts/DejaVuSans.ttf", 48)),
        ShapeStyle {
            fill: Some(Color::white()),
            ..Default::default()
        },
    );

    // Create more text in different colors
    let red_text = ShapeRenderable::from_shape(
        100.0,
        200.0,
        ShapeKind::Text(Text::new("Red Text", "fonts/DejaVuSans.ttf", 36)),
        ShapeStyle {
            fill: Some(Color::from_rgb(1.0, 0.0, 0.0)),
            ..Default::default()
        },
    );

    let green_text = ShapeRenderable::from_shape(
        100.0,
        280.0,
        ShapeKind::Text(Text::new("Green Text", "fonts/DejaVuSans.ttf", 36)),
        ShapeStyle {
            fill: Some(Color::from_rgb(0.0, 1.0, 0.0)),
            ..Default::default()
        },
    );

    let blue_text = ShapeRenderable::from_shape(
        100.0,
        360.0,
        ShapeKind::Text(Text::new("Blue Text", "fonts/DejaVuSans.ttf", 36)),
        ShapeStyle {
            fill: Some(Color::from_rgb(0.0, 0.0, 1.0)),
            ..Default::default()
        },
    );

    // Smaller text
    let small_text = ShapeRenderable::from_shape(
        100.0,
        450.0,
        ShapeKind::Text(Text::new(
            "The quick brown fox jumps over the lazy dog",
            "fonts/DejaVuSans.ttf",
            24,
        )),
        ShapeStyle {
            fill: Some(Color::from_rgb(0.8, 0.8, 0.8)),
            ..Default::default()
        },
    );

    let mut shapes = vec![text, red_text, green_text, blue_text, small_text];

    app.on_render(move || {
        for shape in &mut shapes {
            shape.render(&renderer);
        }
    });
    app.run();
}
