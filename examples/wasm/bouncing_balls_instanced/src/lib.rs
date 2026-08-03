//! Browser port of `bouncing_balls_instanced`: 10,000 balls in ONE
//! instanced draw call — first exercise of the instancing path
//! (`glVertexAttribDivisor` / `glDrawArraysInstanced`) on the web backend.
//!
//! Physics is identical to the native example; as in the other wasm ports,
//! the `rand` crate is replaced by a dependency-free xorshift PRNG and the
//! canvas is fullscreen.

use std::cell::RefCell;

use wilhelm_renderer::core::{App, Color, Vec2, Window};
use wilhelm_renderer::graphics2d::shapes::{Circle, ShapeKind, ShapeRenderable, ShapeStyle};

thread_local! {
    static APP: RefCell<Option<App<'static>>> = RefCell::new(None);
}

#[derive(Clone, Copy)]
struct Ball {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
}

const BALL_RADIUS: f32 = 5.0;
const BALL_COUNT: usize = 10_000;

/// Minimal xorshift32 PRNG — enough randomness for a demo, no dependencies.
struct XorShift(u32);

impl XorShift {
    fn next_f32(&mut self) -> f32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        (x as f32) / (u32::MAX as f32)
    }

    fn range(&mut self, lo: f32, hi: f32) -> f32 {
        lo + self.next_f32() * (hi - lo)
    }
}

fn build_app() -> App<'static> {
    let window = Window::new_fullscreen(
        "Bouncing Balls — Instanced",
        Color::from_rgb(0.07, 0.13, 0.17),
    );
    let (w, h) = (window.width() as f32, window.height() as f32);
    let mut app = App::new(window);

    let mut rng = XorShift(0x57_11_4E_1D);
    let mut balls: Vec<Ball> = (0..BALL_COUNT)
        .map(|_| Ball {
            x: rng.range(BALL_RADIUS, w - BALL_RADIUS),
            y: rng.range(BALL_RADIUS, h - BALL_RADIUS),
            vx: rng.range(-150.0, 150.0),
            vy: rng.range(-150.0, 150.0),
        })
        .collect();

    let mut dots = ShapeRenderable::from_shape(
        ShapeKind::Circle(Circle::new(BALL_RADIUS)),
        ShapeStyle {
            fill: Some(Color::from_rgb(0.254902, 0.411765, 0.882353)),
            ..Default::default()
        },
    );
    dots.create_multiple_instances(balls.len());
    {
        let positions: Vec<Vec2> = balls.iter().map(|b| Vec2::new(b.x, b.y)).collect();
        dots.set_instance_positions(&positions);
    }
    {
        let colors: Vec<Color> = (0..balls.len())
            .map(|_| Color::from_rgb(rng.next_f32(), rng.next_f32(), rng.next_f32()))
            .collect();
        dots.set_instance_colors(&colors);
    }

    app.add_shape(dots);

    let mut last_time: Option<f64> = None;

    app.on_pre_render(move |shapes, renderer| {
        let current_time = renderer.get_time();
        let dt = (current_time - last_time.unwrap_or(current_time)) as f32;
        last_time = Some(current_time);

        let (w, h) = renderer.window_handle.size();
        let w = w as f32;
        let h = h as f32;

        for ball in balls.iter_mut() {
            ball.x += ball.vx * dt;
            ball.y += ball.vy * dt;

            if ball.x - BALL_RADIUS < 0.0 || ball.x + BALL_RADIUS > w {
                ball.vx = -ball.vx;
                ball.x = ball.x.clamp(BALL_RADIUS, w - BALL_RADIUS);
            }
            if ball.y - BALL_RADIUS < 0.0 || ball.y + BALL_RADIUS > h {
                ball.vy = -ball.vy;
                ball.y = ball.y.clamp(BALL_RADIUS, h - BALL_RADIUS);
            }
        }

        let positions: Vec<Vec2> = balls.iter().map(|b| Vec2::new(b.x, b.y)).collect();
        shapes[0].set_instance_positions(&positions);
    });

    app
}

#[no_mangle]
pub extern "C" fn wasm_init() {
    APP.with(|slot| {
        *slot.borrow_mut() = Some(build_app());
    });
}

#[no_mangle]
pub extern "C" fn wasm_frame() {
    APP.with(|slot| {
        if let Some(app) = slot.borrow_mut().as_mut() {
            app.frame();
        }
    });
}
