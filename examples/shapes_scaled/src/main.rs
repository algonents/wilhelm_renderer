extern crate wilhelm_renderer;

use wilhelm_renderer::core::{App, Color, Window};
use wilhelm_renderer::graphics2d::shapes::{
    Circle, Ellipse, Line, MultiPoint, Polygon, Polyline, Rectangle, RoundedRectangle, ShapeKind,
    ShapeRenderable, ShapeStyle,
};

use std::cell::Cell;

thread_local! {
    static SCALE_LEVEL: Cell<f32> = Cell::new(1.0);
}

fn stroke_style(color: Color, width: f32) -> ShapeStyle {
    ShapeStyle {
        fill: Some(color.clone()),
        stroke_color: Some(color),
        stroke_width: Some(width),
    }
}

fn fill_style(color: Color) -> ShapeStyle {
    ShapeStyle {
        fill: Some(color),
        stroke_color: None,
        stroke_width: None,
    }
}

fn main() {
    let mut window = Window::new("Shapes", 800, 800, Color::from_rgb(0.07, 0.13, 0.17));

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
            println!("scale level: {}", new_scale);
        });
    });

    let mut app = App::new(window);
    app.renderer().set_point_size(6.0);

    // Polyline points (relative to first point)
    let polyline_points = vec![
        (0.0, 0.0),
        (50.0, 130.0),
        (100.0, 110.0),
        (150.0, 160.0),
    ];

    // MultiPoint points (relative to first point)
    let multipoint_points = vec![
        (0.0, 0.0),   // anchor at (600, 100)
        (20.0, 20.0),
        (-20.0, 20.0),
    ];

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

    let image_sized = |pos: (f32, f32), path: &str, w: f32, h: f32| -> ShapeRenderable {
        let mut s = ShapeRenderable::image_with_size(path, w, h);
        s.set_position(pos.0, pos.1);
        s
    };

    let image = |pos: (f32, f32), path: &str| -> ShapeRenderable {
        let mut s = ShapeRenderable::image(path);
        s.set_position(pos.0, pos.1);
        s
    };

    app.add_shapes(vec![
        // Line from (100, 200) to (300, 250)
        shape((100.0, 200.0),
            ShapeKind::Line(Line::new((0.0, 0.0), (200.0, 50.0))),
            stroke_style(Color::from_rgb(0.0, 1.0, 0.0), 1.0),
        ),
        // Polyline starting at (100, 300)
        shape((100.0, 300.0),
            ShapeKind::Polyline(Polyline::new(polyline_points)),
            stroke_style(Color::from_rgb(0.0, 1.0, 0.0), 10.0),
        ),
        // Rectangle at (50, 50)
        shape((50.0, 50.0),
            ShapeKind::Rectangle(Rectangle::new(200.0, 80.0)),
            fill_style(Color::from_rgb(0.2, 0.5, 0.9)),
        ),
        // Rectangle at (400, 200)
        shape((400.0, 200.0),
            ShapeKind::Rectangle(Rectangle::new(100.0, 50.0)),
            fill_style(Color::from_rgb(1.0, 0.0, 0.0)),
        ),
        // Circle at (400, 400)
        shape((400.0, 400.0),
            ShapeKind::Circle(Circle::new(50.0)),
            fill_style(Color::from_rgb(0.0, 0.0, 1.0)),
        ),
        // Point at (600, 300)
        shape((600.0, 300.0),
            ShapeKind::Point,
            fill_style(Color::from_rgb(1.0, 0.0, 0.0)),
        ),
        // MultiPoint at (600, 100)
        shape((600.0, 100.0),
            ShapeKind::MultiPoint(MultiPoint::new(multipoint_points)),
            fill_style(Color::from_rgb(0.0, 0.0, 1.0)),
        ),
        // Ellipse at (600, 200)
        shape((600.0, 200.0),
            ShapeKind::Ellipse(Ellipse::new(80.0, 40.0)),
            fill_style(Color::from_rgb(0.5, 0.2, 0.8)),
        ),
        // Rounded rectangle at (100, 600)
        shape((100.0, 600.0),
            ShapeKind::RoundedRectangle(RoundedRectangle::new(200.0, 80.0, 10.0)),
            fill_style(Color::from_rgb(0.3, 0.6, 0.9)),
        ),
        // Polygon (hexagon) at (600, 600)
        shape((600.0, 600.0),
            ShapeKind::Polygon(Polygon::new(polygon_points)),
            fill_style(Color::from_rgb(1.0, 0.0, 0.0)),
        ),
        // Rectangle at (600, 400)
        shape((600.0, 400.0),
            ShapeKind::Rectangle(Rectangle::new(100.0, 50.0)),
            fill_style(Color::from_rgb(0.0, 1.0, 0.0)),
        ),
        // Images
        image_sized((200.0, 300.0), "../../images/smiley.png", 40.0, 40.0),
        image((400.0, 500.0), "../../images/bunny.png"),
    ]);

    app.on_pre_render(move |shapes, _renderer| {
        let scale = SCALE_LEVEL.with(|s| s.get());
        for shape in shapes.iter_mut() {
            shape.set_scale(scale);
        }
    });

    app.run();
}
