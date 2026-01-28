extern crate wilhelm_renderer;

use wilhelm_renderer::core::{App, Color, Renderable, Vec2, Window};
use wilhelm_renderer::graphics2d::shapes::{Circle, ShapeKind, ShapeRenderable, ShapeStyle};

use rand::{rngs::ThreadRng, Rng};
use rand::distr::Uniform;

#[derive(Clone, Copy)]
struct Ball {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
}

const BALL_RADIUS: f32 = 5.0;

fn main() {
    let window = Window::new("Bouncing Balls — Instanced", 1280, 720, Color::from_rgb(0.07, 0.13, 0.17));

    let mut balls = initialize_balls(10_000, window.width() as f32, window.height() as f32);

    let mut app = App::new(window);

    let mut dots = ShapeRenderable::from_shape(
        0.0, 0.0,
        ShapeKind::Circle(Circle::new(BALL_RADIUS)),
        ShapeStyle {
            fill: Some(Color::from_rgb(0.254902, 0.411765, 0.882353)),
            stroke_color: None,
            stroke_width: None,
        },
    );
    dots.create_multiple_instances(balls.len());
    {
        let positions: Vec<Vec2> = balls.iter().map(|b| Vec2::new(b.x, b.y)).collect();
        dots.set_instance_positions(&positions);
    }
    {
        let mut rng = rand::rng();
        let colors: Vec<Color> = (0..balls.len())
            .map(|_| Color::from_rgb(rand_f32(&mut rng), rand_f32(&mut rng), rand_f32(&mut rng)))
            .collect();
        dots.set_instance_colors(&colors);
    }

    app.add_shape(dots);

    let mut last_time = app.renderer().get_time();

    app.on_pre_render(move |shapes, renderer| {
        let current_time = renderer.get_time();
        let dt = (current_time - last_time) as f32;
        last_time = current_time;

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

    app.run();
}

fn initialize_balls(n: usize, screen_width: f32, screen_height: f32) -> Vec<Ball> {
    let mut rng = rand::rng();
    let pos_x = Uniform::new(BALL_RADIUS, screen_width - BALL_RADIUS).unwrap();
    let pos_y = Uniform::new(BALL_RADIUS, screen_height - BALL_RADIUS).unwrap();
    let vel = Uniform::new(-150.0, 150.0).unwrap();

    (0..n)
        .map(|_| Ball {
            x: rng.sample(pos_x),
            y: rng.sample(pos_y),
            vx: rng.sample(vel),
            vy: rng.sample(vel),
        })
        .collect()
}

// Random float between 0.0 and 1.0
fn rand_f32(rng: &mut ThreadRng) -> f32 {
    rng.random_range(0.0..1.0)
}
