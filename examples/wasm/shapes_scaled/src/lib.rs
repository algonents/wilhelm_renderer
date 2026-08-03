//! Browser port of the `shapes_scaled` example: anchored shapes with
//! zoom-on-scroll (no text, images, or points — see docs/DESIGN_WASM.md).
//!
//! First interactive port: the canvas wheel event reaches the engine's
//! existing `Window::on_scroll` closure through the GLFW-style trampolines
//! (`wilhelm_dispatch_scroll` in the sys web backend) — the input plumbing
//! is identical to native above the sys boundary.

use std::cell::{Cell, RefCell};

use wilhelm_renderer::core::{App, Color, Window};
use wilhelm_renderer::graphics2d::shapes::{
    Anchor, Circle, Ellipse, Line, Polygon, Polyline, Rectangle, RoundedRectangle, ShapeKind,
    ShapeRenderable, ShapeStyle,
};

thread_local! {
    static APP: RefCell<Option<App<'static>>> = RefCell::new(None);
    static SCALE_LEVEL: Cell<f32> = Cell::new(1.0);
}

fn stroke_style(color: Color, width: f32) -> ShapeStyle {
    ShapeStyle {
        fill: Some(color.clone()),
        stroke_color: Some(color),
        stroke_width: Some(width),
        ..Default::default()
    }
}

fn fill_style(color: Color) -> ShapeStyle {
    ShapeStyle {
        fill: Some(color),
        ..Default::default()
    }
}

fn build_app() -> App<'static> {
    let mut window = Window::new_fullscreen("Shapes scaled", Color::from_rgb(0.07, 0.13, 0.17));

    window.on_scroll(move |_, y_offset| {
        let scale_step = 1.1;
        let scale_factor = if y_offset > 0.0 {
            scale_step
        } else {
            1.0 / scale_step
        };

        SCALE_LEVEL.with(|s| {
            let new_scale = (s.get() * scale_factor).clamp(0.1, 10.0);
            s.set(new_scale);
        });
    });

    let mut app = App::new(window);

    // Polyline points (relative to first point)
    let polyline_points = vec![(0.0, 0.0), (50.0, 130.0), (100.0, 110.0), (150.0, 160.0)];

    // Polygon points (relative to first point at 600, 600)
    let polygon_points = vec![
        (0.0, 0.0),
        (-25.0, 43.3),
        (-75.0, 43.3),
        (-100.0, 0.0),
        (-75.0, -43.4),
        (-25.0, -43.4),
    ];

    let shape = |pos: (f32, f32), kind: ShapeKind, style: ShapeStyle| -> ShapeRenderable {
        let mut s = ShapeRenderable::from_shape(kind, style);
        s.set_position(pos.0, pos.1);
        s
    };

    let shape_anchored =
        |pos: (f32, f32), kind: ShapeKind, style: ShapeStyle, anchor: Anchor| -> ShapeRenderable {
            let mut s = ShapeRenderable::builder(kind, style).anchor(anchor).build();
            s.set_position(pos.0, pos.1);
            s
        };

    app.add_shapes(vec![
        // Line from (100, 200) to (300, 250)
        shape(
            (100.0, 200.0),
            ShapeKind::Line(Line::new((0.0, 0.0), (200.0, 50.0))),
            stroke_style(Color::from_rgb(0.0, 1.0, 0.0), 1.0),
        ),
        // Polyline starting at (100, 300)
        shape(
            (100.0, 300.0),
            ShapeKind::Polyline(Polyline::new(polyline_points)),
            stroke_style(Color::from_rgb(0.0, 1.0, 0.0), 10.0),
        ),
        // Rectangle at (50, 50) — center anchor
        shape_anchored(
            (150.0, 90.0),
            ShapeKind::Rectangle(Rectangle::new(200.0, 80.0)),
            fill_style(Color::from_rgb(0.2, 0.5, 0.9)),
            Anchor::Center,
        ),
        // Rectangle at (400, 200) — center anchor
        shape_anchored(
            (450.0, 225.0),
            ShapeKind::Rectangle(Rectangle::new(100.0, 50.0)),
            fill_style(Color::from_rgb(1.0, 0.0, 0.0)),
            Anchor::Center,
        ),
        // Circle at (400, 400)
        shape(
            (400.0, 400.0),
            ShapeKind::Circle(Circle::new(50.0)),
            fill_style(Color::from_rgb(0.0, 0.0, 1.0)),
        ),
        // Ellipse at (600, 200) — already centers by default
        shape(
            (600.0, 200.0),
            ShapeKind::Ellipse(Ellipse::new(80.0, 40.0)),
            fill_style(Color::from_rgb(0.5, 0.2, 0.8)),
        ),
        // Rounded rectangle at (100, 600) — center anchor
        shape_anchored(
            (200.0, 640.0),
            ShapeKind::RoundedRectangle(RoundedRectangle::new(200.0, 80.0, 10.0)),
            fill_style(Color::from_rgb(0.3, 0.6, 0.9)),
            Anchor::Center,
        ),
        // Polygon (hexagon) at (600, 600) — center anchor
        shape_anchored(
            (600.0, 600.0),
            ShapeKind::Polygon(Polygon::new(polygon_points)),
            fill_style(Color::from_rgb(1.0, 0.0, 0.0)),
            Anchor::Center,
        ),
        // Rectangle at (600, 400) — NorthWest anchor (scales from top-left corner)
        shape_anchored(
            (600.0, 400.0),
            ShapeKind::Rectangle(Rectangle::new(100.0, 50.0)),
            fill_style(Color::from_rgb(0.0, 1.0, 0.0)),
            Anchor::NorthWest,
        ),
    ]);

    app.on_pre_render(move |shapes, _renderer| {
        let scale = SCALE_LEVEL.with(|s| s.get());
        for shape in shapes.iter_mut() {
            shape.set_scale(scale);
        }
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
