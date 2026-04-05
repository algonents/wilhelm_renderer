use wilhelm_renderer::core::{App, Color, Window};
use wilhelm_renderer::graphics2d::shapes::{
    Circle, Ellipse, Rectangle, ShapeKind, ShapeRenderable, ShapeStyle, Text,
};

fn main() {
    let window = Window::new(
        "Style Mutation — Dynamic Color Changes",
        900,
        700,
        Color::from_rgb(0.94, 0.92, 0.84),
    );
    let mut app = App::new(window);

    let shape = |pos: (f32, f32), kind: ShapeKind, style: ShapeStyle| -> ShapeRenderable {
        let mut s = ShapeRenderable::from_shape(kind, style);
        s.set_position(pos.0, pos.1);
        s
    };

    // Title
    let title = shape(
        (20.0, 30.0),
        ShapeKind::Text(Text::new(
            "Style Mutation Demo",
            "../../fonts/DejaVuSans.ttf",
            28,
        )),
        ShapeStyle::fill(Color::from_rgb(0.2, 0.2, 0.2)),
    );

    // Row of circles that cycle through hues
    let circle_y = 150.0;
    let num_circles = 7;
    let circle_r = 35.0;
    let spacing = 110.0;
    let start_x = 100.0;

    let mut circles: Vec<ShapeRenderable> = (0..num_circles)
        .map(|i| {
            shape(
                (start_x + i as f32 * spacing, circle_y),
                ShapeKind::Circle(Circle::new(circle_r)),
                ShapeStyle::fill(Color::from_rgb(0.5, 0.5, 0.5)),
            )
        })
        .collect();

    // Rectangle that pulses between two colors
    let pulse_rect = shape(
        (60.0, 280.0),
        ShapeKind::Rectangle(Rectangle::new(300.0, 80.0)),
        ShapeStyle::fill(Color::from_rgb(0.2, 0.5, 0.9)),
    );

    // Fill+stroke rectangle — stroke and fill change independently
    let dual_rect = shape(
        (450.0, 280.0),
        ShapeKind::Rectangle(Rectangle::new(300.0, 80.0)),
        ShapeStyle::fill_and_stroke(
            Color::from_rgb(0.2, 0.5, 0.9),
            Color::from_rgb(1.0, 1.0, 0.0),
            4.0,
        ),
    );

    // Ellipse with fading alpha
    let fade_ellipse = shape(
        (200.0, 540.0),
        ShapeKind::Ellipse(Ellipse::new(120.0, 60.0)),
        ShapeStyle::fill(Color::from_rgba(1.0, 0.3, 0.3, 1.0)),
    );

    // Text that changes color
    let color_text = shape(
        (450.0, 480.0),
        ShapeKind::Text(Text::new(
            "Color cycling text",
            "../../fonts/DejaVuSans.ttf",
            32,
        )),
        ShapeStyle::fill(Color::from_rgb(0.3, 0.3, 0.3)),
    );

    // Labels
    let label1 = shape(
        (60.0, 90.0),
        ShapeKind::Text(Text::new(
            "Hue cycling circles",
            "../../fonts/DejaVuSans.ttf",
            14,
        )),
        ShapeStyle::fill(Color::from_rgb(0.6, 0.6, 0.6)),
    );

    let label2 = shape(
        (60.0, 250.0),
        ShapeKind::Text(Text::new(
            "Color pulse (fill)",
            "../../fonts/DejaVuSans.ttf",
            14,
        )),
        ShapeStyle::fill(Color::from_rgb(0.6, 0.6, 0.6)),
    );

    let label3 = shape(
        (450.0, 250.0),
        ShapeKind::Text(Text::new(
            "Independent fill + stroke mutation",
            "../../fonts/DejaVuSans.ttf",
            14,
        )),
        ShapeStyle::fill(Color::from_rgb(0.6, 0.6, 0.6)),
    );

    let label4 = shape(
        (60.0, 430.0),
        ShapeKind::Text(Text::new(
            "Alpha breathing",
            "../../fonts/DejaVuSans.ttf",
            14,
        )),
        ShapeStyle::fill(Color::from_rgb(0.6, 0.6, 0.6)),
    );

    let label5 = shape(
        (450.0, 430.0),
        ShapeKind::Text(Text::new(
            "Text color cycling",
            "../../fonts/DejaVuSans.ttf",
            14,
        )),
        ShapeStyle::fill(Color::from_rgb(0.6, 0.6, 0.6)),
    );

    // Collect all static shapes
    app.add_shapes(vec![title, label1, label2, label3, label4, label5]);

    // Add dynamic shapes (indices 6..=13 for circles, then pulse_rect, dual_rect, fade_ellipse, color_text)
    for c in circles.drain(..) {
        app.add_shape(c);
    }
    app.add_shape(pulse_rect);
    app.add_shape(dual_rect);
    app.add_shape(fade_ellipse);
    app.add_shape(color_text);

    // Dynamic shapes start at index 6
    let circles_start = 6;
    let pulse_idx = circles_start + num_circles;
    let dual_idx = pulse_idx + 1;
    let fade_idx = dual_idx + 1;
    let text_idx = fade_idx + 1;

    app.on_pre_render(move |shapes, renderer| {
        let t = renderer.get_time() as f32;

        // Circles: color wave sweeping left to right
        for i in 0..num_circles {
            let phase = i as f32 / num_circles as f32;
            let hue = ((t * 80.0 - phase * 360.0) % 360.0 + 360.0) % 360.0;
            shapes[circles_start + i].set_fill_color(Color::from_hsl(hue, 0.8, 0.55));
        }

        // Pulse rectangle: lerp between blue and orange
        let mix = (t * 2.0).sin() * 0.5 + 0.5;
        let r = 0.2 + mix * 0.8;
        let g = 0.5 - mix * 0.2;
        let b = 0.9 - mix * 0.8;
        shapes[pulse_idx].set_fill_color(Color::from_rgb(r, g, b));

        // Dual rectangle: fill and stroke cycle independently
        let fill_hue = (t * 40.0 % 360.0).abs();
        let stroke_hue = ((t * 80.0 + 180.0) % 360.0).abs();
        shapes[dual_idx].set_fill_color(Color::from_hsl(fill_hue, 0.7, 0.5));
        shapes[dual_idx].set_stroke_color(Color::from_hsl(stroke_hue, 0.9, 0.6));

        // Fade ellipse: breathing alpha
        let alpha = (t * 1.5).sin() * 0.4 + 0.5; // oscillates 0.1..0.9
        shapes[fade_idx].set_fill_color(Color::from_rgba(1.0, 0.3, 0.3, alpha));

        // Text: cycle hue
        let text_hue = (t * 50.0 % 360.0).abs();
        shapes[text_idx].set_fill_color(Color::from_hsl(text_hue, 0.9, 0.65));
    });

    app.run();
}
