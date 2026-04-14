//! Geometric marker shapes for point symbology.
//!
//! Standard marker shapes inspired by D3's symbol set. Each marker is
//! parameterized by a **bounding radius** (distance from center to the
//! outermost vertex in pixels).
//!
//! All vertices are centered at `(0, 0)` and fit within a circle of the
//! given radius. Concave markers (Cross, Star, Wye) triangulate correctly
//! through the ear-clipping algorithm in [`Polygon`](super::shapes::Polygon).

use std::f32::consts::{FRAC_PI_2, PI, TAU};

/// Standard marker types for point symbology.
///
/// Each variant generates polygon vertices centered at `(0, 0)` for a given
/// bounding radius (distance from center to outermost vertex).
///
/// # Usage
///
/// ```ignore
/// use wilhelm_renderer::graphics2d::markers::MarkerType;
/// use wilhelm_renderer::graphics2d::shapes::{Polygon, ShapeKind, ShapeRenderable, ShapeStyle};
/// use wilhelm_renderer::core::Color;
///
/// let star = ShapeRenderable::from_shape(
///     ShapeKind::Polygon(Polygon::new(MarkerType::Star.vertices(20.0))),
///     ShapeStyle::fill(Color::from_rgb(1.0, 0.78, 0.20)),
/// );
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MarkerType {
    /// A regular polygon approximating a circle (64 sides).
    Circle,
    /// A Greek cross with arms of equal length. Concave (12 vertices).
    Cross,
    /// A tall rhombus (4 vertices). Height/width ratio of approximately 2:1.
    Diamond,
    /// An axis-aligned square (4 vertices).
    Square,
    /// A five-pointed star / pentagram (10 vertices). Concave.
    Star,
    /// An equilateral triangle pointing up (3 vertices).
    Triangle,
    /// A Y-shape (12 vertices). Concave.
    Wye,
}

impl MarkerType {
    /// Generate polygon vertices centered at `(0, 0)`.
    ///
    /// `radius` is the bounding radius: the distance from the center to the
    /// outermost vertex. All vertices lie within a circle of that radius.
    pub fn vertices(&self, radius: f32) -> Vec<(f32, f32)> {
        match self {
            MarkerType::Circle => circle_vertices(radius, 64),
            MarkerType::Cross => cross_vertices(radius),
            MarkerType::Diamond => diamond_vertices(radius),
            MarkerType::Square => square_vertices(radius),
            MarkerType::Star => star_vertices(radius),
            MarkerType::Triangle => triangle_vertices(radius),
            MarkerType::Wye => wye_vertices(radius),
        }
    }
}

/// Regular n-gon approximating a circle.
fn circle_vertices(radius: f32, sides: usize) -> Vec<(f32, f32)> {
    let step = TAU / sides as f32;
    let mut angle = -FRAC_PI_2;
    (0..sides)
        .map(|_| {
            let v = (radius * angle.cos(), radius * angle.sin());
            angle += step;
            v
        })
        .collect()
}

/// Greek cross with arms of equal length and width.
/// Scaled so the farthest vertices (arm tip corners) land exactly on the
/// bounding circle. Arm half-width is 1/3 of the arm half-length.
fn cross_vertices(radius: f32) -> Vec<(f32, f32)> {
    // The farthest vertex is an arm tip corner at (t, a) from center.
    // With t = a/3: dist = sqrt(a² + a²/9) = a * sqrt(10/9).
    // Set that equal to radius: a = radius * 3/sqrt(10).
    let a = radius * 3.0 / 10.0_f32.sqrt(); // arm half-length
    let t = a / 3.0; // arm half-width
    vec![
        (-t, -a),
        ( t, -a),
        ( t, -t),
        ( a, -t),
        ( a,  t),
        ( t,  t),
        ( t,  a),
        (-t,  a),
        (-t,  t),
        (-a,  t),
        (-a, -t),
        (-t, -t),
    ]
}

/// Tall rhombus. The top/bottom vertices touch the bounding circle;
/// the left/right vertices are at radius / sqrt(3) to give a height/width
/// ratio of approximately sqrt(3) (matching D3's diamond proportions).
fn diamond_vertices(radius: f32) -> Vec<(f32, f32)> {
    let half_w = radius / 3.0_f32.sqrt();
    vec![
        (0.0, -radius),
        (half_w, 0.0),
        (0.0, radius),
        (-half_w, 0.0),
    ]
}

/// Axis-aligned square. Corners touch the bounding circle, so the side
/// length is radius * sqrt(2).
fn square_vertices(radius: f32) -> Vec<(f32, f32)> {
    let h = radius * std::f32::consts::FRAC_1_SQRT_2;
    vec![(-h, -h), (h, -h), (h, h), (-h, h)]
}

/// Five-pointed star (pentagram). Outer points touch the bounding circle;
/// inner points sit at 38% of the radius (golden-ratio-derived).
fn star_vertices(radius: f32) -> Vec<(f32, f32)> {
    let inner = radius * 0.382;
    let step = PI / 5.0;
    let mut angle = -FRAC_PI_2;
    let mut verts = Vec::with_capacity(10);
    for i in 0..10 {
        let r = if i % 2 == 0 { radius } else { inner };
        verts.push((r * angle.cos(), r * angle.sin()));
        angle += step;
    }
    verts
}

/// Equilateral triangle pointing up. The top vertex touches the bounding
/// circle at (0, -radius).
fn triangle_vertices(radius: f32) -> Vec<(f32, f32)> {
    let mut angle = -FRAC_PI_2;
    let step = TAU / 3.0;
    let mut verts = Vec::with_capacity(3);
    for _ in 0..3 {
        verts.push((radius * angle.cos(), radius * angle.sin()));
        angle += step;
    }
    verts
}

/// Y-shape (wye). Three arms at 120 degree intervals, each arm has a notched
/// profile. Scaled so the arm tip corners land exactly on the bounding circle.
///
/// The vertex order traces the outline continuously (non-self-intersecting):
/// for each pair of adjacent arms, the path goes along one arm's right edge,
/// crosses the inner junction to the next arm's left edge, and continues
/// out to the next arm's tip.
fn wye_vertices(radius: f32) -> Vec<(f32, f32)> {
    let arm = radius * 3.0 / 10.0_f32.sqrt();
    let t = arm / 3.0;

    let angles = [0.0, TAU / 3.0, 2.0 * TAU / 3.0];

    let rotate = |x: f32, y: f32, a: f32| -> (f32, f32) {
        let (sin, cos) = a.sin_cos();
        (x * cos - y * sin, x * sin + y * cos)
    };

    let mut verts = Vec::with_capacity(12);
    for i in 0..3 {
        let next = (i + 1) % 3;
        verts.push(rotate(-t, -arm, angles[i]));
        verts.push(rotate( t, -arm, angles[i]));
        verts.push(rotate( t,   -t, angles[i]));
        verts.push(rotate(-t,   -t, angles[next]));
    }
    verts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn circle_vertex_count() {
        assert_eq!(MarkerType::Circle.vertices(10.0).len(), 64);
    }

    #[test]
    fn cross_vertex_count() {
        assert_eq!(MarkerType::Cross.vertices(10.0).len(), 12);
    }

    #[test]
    fn diamond_vertex_count() {
        assert_eq!(MarkerType::Diamond.vertices(10.0).len(), 4);
    }

    #[test]
    fn square_vertex_count() {
        assert_eq!(MarkerType::Square.vertices(10.0).len(), 4);
    }

    #[test]
    fn star_vertex_count() {
        assert_eq!(MarkerType::Star.vertices(10.0).len(), 10);
    }

    #[test]
    fn triangle_vertex_count() {
        assert_eq!(MarkerType::Triangle.vertices(10.0).len(), 3);
    }

    #[test]
    fn wye_vertex_count() {
        assert_eq!(MarkerType::Wye.vertices(10.0).len(), 12);
    }

    #[test]
    fn vertices_within_bounding_radius() {
        let radius = 15.0;
        for symbol in [
            MarkerType::Circle,
            MarkerType::Cross,
            MarkerType::Diamond,
            MarkerType::Square,
            MarkerType::Star,
            MarkerType::Triangle,
            MarkerType::Wye,
        ] {
            for (x, y) in symbol.vertices(radius) {
                let dist = (x * x + y * y).sqrt();
                assert!(
                    dist <= radius + 1e-4,
                    "{:?} vertex ({}, {}) at distance {} exceeds radius {}",
                    symbol, x, y, dist, radius
                );
            }
        }
    }

    #[test]
    fn vertices_touch_bounding_radius() {
        let radius = 20.0;
        for symbol in [
            MarkerType::Circle,
            MarkerType::Cross,
            MarkerType::Diamond,
            MarkerType::Square,
            MarkerType::Star,
            MarkerType::Triangle,
            MarkerType::Wye,
        ] {
            let max_dist = symbol
                .vertices(radius)
                .iter()
                .map(|(x, y)| (x * x + y * y).sqrt())
                .fold(0.0f32, f32::max);
            assert!(
                (max_dist - radius).abs() < 0.5,
                "{:?} max vertex distance {} doesn't touch radius {}",
                symbol, max_dist, radius
            );
        }
    }
}
