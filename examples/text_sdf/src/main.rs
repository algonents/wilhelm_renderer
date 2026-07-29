//! Side-by-side comparison of bitmap text and SDF text under scaling.
//!
//! Both columns rasterize DejaVuSans at 48px and draw the same string at
//! several scale factors. The bitmap column blurs as the scale grows; the
//! SDF column stays sharp at every scale.

extern crate wilhelm_renderer;

use wilhelm_renderer::core::{App, Color, Window};
use wilhelm_renderer::graphics2d::shapes::{ShapeKind, ShapeRenderable, ShapeStyle, Text};

const FONT: &str = "../../fonts/DejaVuSans.ttf";
const FONT_SIZE: u32 = 48;
const SAMPLE: &str = "FL350";
/// Effective on-screen text sizes; each row scales the 48px atlas to hit one.
const SIZES: [f32; 5] = [10.0, 24.0, 48.0, 96.0, 192.0];

fn bitmap_text(content: &str, size: u32, color: Color) -> ShapeRenderable {
    ShapeRenderable::from_shape(
        ShapeKind::Text(Text::new(content, FONT, size)),
        ShapeStyle {
            fill: Some(color),
            ..Default::default()
        },
    )
}

fn main() {
    let window = Window::new(
        "SDF vs Bitmap Text",
        1440,
        800,
        Color::from_rgb(0.07, 0.13, 0.17),
    );
    let mut app = App::new(window);

    let mut shapes = Vec::new();

    // Column headers
    let mut header = bitmap_text("bitmap (48px atlas)", 24, Color::from_rgb(0.6, 0.7, 0.8));
    header.set_position(40.0, 20.0);
    shapes.push(header);

    let mut header = bitmap_text("SDF (48px atlas)", 24, Color::from_rgb(0.6, 0.7, 0.8));
    header.set_position(760.0, 20.0);
    shapes.push(header);

    let mut y = 90.0;
    for &size in &SIZES {
        let scale = size / FONT_SIZE as f32;
        let mut label = bitmap_text(
            &format!("{}px", size as u32),
            24,
            Color::from_rgb(0.4, 0.5, 0.6),
        );
        label.set_position(40.0, y);
        shapes.push(label);

        let mut bitmap = bitmap_text(SAMPLE, FONT_SIZE, Color::white());
        bitmap.set_scale(scale);
        bitmap.set_position(120.0, y);
        shapes.push(bitmap);

        let mut sdf = ShapeRenderable::text_sdf(SAMPLE, FONT, FONT_SIZE, Color::white());
        sdf.set_scale(scale);
        sdf.set_position(760.0, y);
        shapes.push(sdf);

        y += FONT_SIZE as f32 * scale + 30.0;
    }

    app.add_shapes(shapes);
    app.run();
}
