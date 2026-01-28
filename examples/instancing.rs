extern crate wilhelm_renderer;

use wilhelm_renderer::core::{App, Color, Renderable, Vec2, Window};
use wilhelm_renderer::graphics2d::shapes::{Circle, ShapeKind, ShapeRenderable, ShapeStyle};

const WIDTH: i32 = 1600;
const HEIGHT: i32 = 1000;
const COLS: usize = 100;
const ROWS: usize = 60;
const SPACING: f32 = 15.0;
const ORIGIN_X: f32 = 50.0;
const ORIGIN_Y: f32 = 50.0;
const RADIUS: f32 = 3.0;

// Darker-than-DeepSky: SteelBlue (0..1)
const STEEL_BLUE: (f32, f32, f32) = (0.274510, 0.509804, 0.705882);

fn main() {
    let mut window = Window::new("Instancing Demo", WIDTH, HEIGHT, Color::from_rgb(0.07, 0.13, 0.17));
    window.on_resize(|w, h| println!("Window resized: {}x{}", w, h));

    let mut app = App::new(window);

    // One shape, many instances
    let mut dots = ShapeRenderable::from_shape(
        0.0, 0.0,
        ShapeKind::Circle(Circle::new(RADIUS)),
        ShapeStyle {
            fill: Some(Color::from_rgb(STEEL_BLUE.0, STEEL_BLUE.1, STEEL_BLUE.2)),
            stroke_color: None,
            stroke_width: None,
        },
    );
    let instance_count = COLS * ROWS;
    dots.create_multiple_instances(instance_count);

    // Static base grid with per-instance colors
    let mut base_positions: Vec<Vec2> = Vec::with_capacity(instance_count);
    let mut colors: Vec<Color> = Vec::with_capacity(instance_count);
    for j in 0..ROWS {
        for i in 0..COLS {
            base_positions.push(Vec2::new(
                ORIGIN_X + i as f32 * SPACING,
                ORIGIN_Y + j as f32 * SPACING,
            ));
            let r = i as f32 / COLS as f32;
            let b = j as f32 / ROWS as f32;
            let g = 0.4;
            colors.push(Color::from_rgb(r, g, b));
        }
    }

    let mut positions = base_positions.clone();
    dots.set_instance_positions(&positions);
    dots.set_instance_colors(&colors);

    app.add_shape(dots);

    app.on_pre_render(move |shapes, renderer| {
        let t = renderer.get_time() as f32;
        let wiggle = (t * 2.0).sin() * 3.0;

        for (dst, base) in positions.iter_mut().zip(base_positions.iter()) {
            *dst = Vec2::new(base.x + wiggle, base.y + wiggle);
        }

        shapes[0].set_instance_positions(&positions);
    });

    app.run();
}
