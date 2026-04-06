extern crate wilhelm_renderer;

use wilhelm_renderer::core::{App, Color, Window};
use wilhelm_renderer::graphics2d::shapes::{
    Polygon, ShapeKind, ShapeRenderable, ShapeStyle,
};

/// Build a regular convex polygon (n sides) centered on (0,0).
fn regular_polygon(radius: f32, sides: usize) -> Vec<(f32, f32)> {
    let mut verts = Vec::with_capacity(sides);
    let step = std::f32::consts::TAU / sides as f32;
    let mut angle = -std::f32::consts::FRAC_PI_2;
    for _ in 0..sides {
        verts.push((radius * angle.cos(), radius * angle.sin()));
        angle += step;
    }
    verts
}

/// Build a star with the given number of points (concave: alternates outer
/// and inner radii). Centered on (0,0).
fn star(outer_r: f32, inner_r: f32, points: usize) -> Vec<(f32, f32)> {
    let mut verts = Vec::with_capacity(points * 2);
    let step = std::f32::consts::PI / points as f32;
    let mut angle = -std::f32::consts::FRAC_PI_2;
    for i in 0..(points * 2) {
        let r = if i % 2 == 0 { outer_r } else { inner_r };
        verts.push((r * angle.cos(), r * angle.sin()));
        angle += step;
    }
    verts
}

fn main() {
    let window = Window::new(
        "Polygons",
        800,
        600,
        Color::from_rgb(0.07, 0.13, 0.17),
    );
    let mut app = App::new(window);

    // Polygons anchor at their first vertex: `set_position(x, y)` places the
    // first vertex at (x, y), not the polygon's visual center. The helper
    // below compensates so that `center` lands on the visual center of a
    // polygon defined in a (0,0)-centered local frame.
    let place = |center: (f32, f32), points: Vec<(f32, f32)>, color: Color| -> ShapeRenderable {
        let (fx, fy) = points[0];
        let mut s = ShapeRenderable::from_shape(
            ShapeKind::Polygon(Polygon::new(points)),
            ShapeStyle::fill(color),
        );
        s.set_position(center.0 + fx, center.1 + fy);
        s
    };

    // Convex: regular hexagon
    let hexagon = regular_polygon(110.0, 6);

    // Concave: five-pointed star (10 vertices)
    let star_5 = star(120.0, 50.0, 5);

    app.add_shapes(vec![
        // Left: convex hexagon
        place((250.0, 300.0), hexagon, Color::from_rgb(0.55, 0.85, 0.45)),
        // Right: concave star
        place((550.0, 300.0), star_5,  Color::from_rgb(1.00, 0.78, 0.20)),
    ]);

    app.run();
}
