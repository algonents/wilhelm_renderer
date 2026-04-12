mod shaperenderable;

pub use shaperenderable::Anchor;
pub use shaperenderable::ShapeRenderable;
pub use shaperenderable::ShapeRenderableBuilder;
pub use shaperenderable::ShapeStyle;
pub use shaperenderable::clear_font_cache;

#[derive(Clone)]
pub enum ShapeKind {
    Point,
    MultiPoint(MultiPoint),
    Line(Line),
    Polyline(Polyline),
    Triangle(Triangle),
    Rectangle(Rectangle),
    RoundedRectangle(RoundedRectangle),
    Polygon(Polygon),
    Circle(Circle),
    Ellipse(Ellipse),
    Arc(Arc),
    Image(Image),
    Text(Text),
}

#[derive(Clone)]
pub struct Point;
impl Point{
    pub fn new() -> Self{
        Self{}
    }
}

#[derive(Clone)]
pub struct MultiPoint {
    pub points: Vec<(f32, f32)>,
}

impl MultiPoint {
    pub fn new(points: Vec<(f32, f32)>) -> Self {
        Self { points }
    }

    /// Vertex average of all points. Panics on empty input.
    pub fn centroid(&self) -> (f32, f32) {
        let n = self.points.len() as f32;
        let (sx, sy) = self
            .points
            .iter()
            .fold((0.0f32, 0.0f32), |(sx, sy), (x, y)| (sx + x, sy + y));
        (sx / n, sy / n)
    }
}


#[derive(Clone, Copy)]
pub struct Line {
    pub start: (f32, f32),
    pub end: (f32, f32),
}

impl Line {
    pub fn new(start: (f32, f32), end: (f32, f32)) -> Self {
        Self { start, end }
    }

    /// Midpoint of the line.
    pub fn centroid(&self) -> (f32, f32) {
        (
            (self.start.0 + self.end.0) * 0.5,
            (self.start.1 + self.end.1) * 0.5,
        )
    }
}

#[derive(Clone)]
pub struct Polyline {
    pub points: Vec<(f32, f32)>,
}

impl Polyline {
    pub fn new(points: Vec<(f32, f32)>) -> Self {
        Self { points }
    }

    /// Vertex average of the polyline's points. Panics on empty input.
    pub fn centroid(&self) -> (f32, f32) {
        let n = self.points.len() as f32;
        let (sx, sy) = self
            .points
            .iter()
            .fold((0.0f32, 0.0f32), |(sx, sy), (x, y)| (sx + x, sy + y));
        (sx / n, sy / n)
    }
}
#[derive(Clone, Copy)]
pub struct Triangle {
    pub vertices: [(f32, f32); 3],
}

impl Triangle {
    pub fn new(vertices: [(f32, f32); 3]) -> Self {
        Self { vertices }
    }

    /// Geometric centroid: `(v0 + v1 + v2) / 3`.
    pub fn centroid(&self) -> (f32, f32) {
        let [a, b, c] = self.vertices;
        ((a.0 + b.0 + c.0) / 3.0, (a.1 + b.1 + c.1) / 3.0)
    }
}


#[derive(Clone, Copy)]
pub struct Rectangle {
    pub width: f32,
    pub height: f32,
}

impl Rectangle {
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

#[derive(Clone, Copy)]
pub struct RoundedRectangle {
    pub width: f32,
    pub height: f32,
    pub radius: f32,
}

impl RoundedRectangle {
    pub fn new(width: f32, height: f32, radius: f32) -> Self {
        Self { width, height, radius }
    }
}

#[derive(Clone)]
pub struct Polygon {
    pub points: Vec<(f32, f32)>,
}

impl Polygon {
    pub fn new(points: Vec<(f32, f32)>) -> Self {
        Self { points }
    }

    /// Area centroid of the polygon (the geometric "balance point").
    ///
    /// Computed via the shoelace-based centroid formula for a simple polygon.
    /// For concave polygons the centroid may lie outside the polygon itself
    /// (e.g. in the notch of an L-shape) — that is mathematically correct.
    ///
    /// Degenerate inputs (fewer than 3 points, or zero signed area from
    /// collinear points) fall back to the plain vertex average.
    pub fn centroid(&self) -> (f32, f32) {
        let n = self.points.len();
        if n < 3 {
            return Self::vertex_average(&self.points);
        }
        let mut cx = 0.0f32;
        let mut cy = 0.0f32;
        let mut signed_area_sum = 0.0f32;
        for i in 0..n {
            let (x0, y0) = self.points[i];
            let (x1, y1) = self.points[(i + 1) % n];
            let cross = x0 * y1 - x1 * y0;
            cx += (x0 + x1) * cross;
            cy += (y0 + y1) * cross;
            signed_area_sum += cross;
        }
        let area = signed_area_sum * 0.5;
        if area.abs() < f32::EPSILON {
            return Self::vertex_average(&self.points);
        }
        (cx / (6.0 * area), cy / (6.0 * area))
    }

    fn vertex_average(points: &[(f32, f32)]) -> (f32, f32) {
        let n = points.len() as f32;
        let (sx, sy) = points
            .iter()
            .fold((0.0f32, 0.0f32), |(sx, sy), (x, y)| (sx + x, sy + y));
        (sx / n, sy / n)
    }

    /// Signed area of the polygon (shoelace formula).
    ///
    /// Positive when vertices are ordered counter-clockwise (in a Y-up coordinate
    /// system), negative when clockwise. Magnitude equals the polygon area.
    pub fn signed_area(&self) -> f32 {
        let n = self.points.len();
        if n < 3 {
            return 0.0;
        }
        let mut sum = 0.0;
        for i in 0..n {
            let (x0, y0) = self.points[i];
            let (x1, y1) = self.points[(i + 1) % n];
            sum += x0 * y1 - x1 * y0;
        }
        sum * 0.5
    }

    /// Triangulate the (possibly concave) simple polygon using ear clipping.
    ///
    /// Returns a list of triangles as triples of indices into `self.points`.
    /// The polygon is assumed to be simple (non-self-intersecting). The
    /// algorithm normalizes the working order to counter-clockwise internally,
    /// so input may be in either winding.
    ///
    /// Complexity: O(n²). Suitable for typical polygon sizes; for very large
    /// polygons consider a more advanced algorithm.
    ///
    /// Returns an empty Vec for degenerate input (fewer than 3 vertices).
    pub fn triangulate(&self) -> Vec<[usize; 3]> {
        let n = self.points.len();
        if n < 3 {
            return Vec::new();
        }
        if n == 3 {
            return vec![[0, 1, 2]];
        }

        // Work in CCW order so the convexity test has a consistent sign.
        let ccw = self.signed_area() > 0.0;
        let mut indices: Vec<usize> = if ccw {
            (0..n).collect()
        } else {
            (0..n).rev().collect()
        };

        let mut triangles: Vec<[usize; 3]> = Vec::with_capacity(n - 2);
        // Safety bound: a simple polygon needs at most n-2 ears, but we may
        // scan multiple vertices per removal. Cap iterations to avoid infinite
        // loops on malformed (e.g., self-intersecting) input.
        let max_iterations = n * n;
        let mut iterations = 0;

        while indices.len() > 3 {
            iterations += 1;
            if iterations > max_iterations {
                break;
            }

            let m = indices.len();
            let mut ear_index: Option<usize> = None;
            for i in 0..m {
                let a = indices[(i + m - 1) % m];
                let b = indices[i];
                let c = indices[(i + 1) % m];
                if self.is_ear(a, b, c, &indices) {
                    ear_index = Some(i);
                    triangles.push([a, b, c]);
                    break;
                }
            }
            match ear_index {
                Some(i) => {
                    indices.remove(i);
                }
                None => break, // degenerate polygon; bail out
            }
        }

        if indices.len() == 3 {
            triangles.push([indices[0], indices[1], indices[2]]);
        }

        triangles
    }

    /// True if the triangle (a, b, c) is an ear: convex at b (in CCW order)
    /// and contains no other polygon vertex.
    fn is_ear(&self, a: usize, b: usize, c: usize, indices: &[usize]) -> bool {
        let (ax, ay) = self.points[a];
        let (bx, by) = self.points[b];
        let (cx, cy) = self.points[c];

        // Convex at b iff cross product of (b-a) x (c-b) is positive in CCW.
        let cross = (bx - ax) * (cy - by) - (by - ay) * (cx - bx);
        if cross <= 0.0 {
            return false;
        }

        // Reject if any other polygon vertex lies strictly inside the triangle.
        for &idx in indices {
            if idx == a || idx == b || idx == c {
                continue;
            }
            let (px, py) = self.points[idx];
            if point_in_triangle(px, py, ax, ay, bx, by, cx, cy) {
                return false;
            }
        }
        true
    }
}

/// Returns true if point p lies inside the triangle (a, b, c), using the
/// half-plane sign test. Points exactly on edges are treated as inside.
fn point_in_triangle(
    px: f32, py: f32,
    ax: f32, ay: f32,
    bx: f32, by: f32,
    cx: f32, cy: f32,
) -> bool {
    let d1 = (px - bx) * (ay - by) - (ax - bx) * (py - by);
    let d2 = (px - cx) * (by - cy) - (bx - cx) * (py - cy);
    let d3 = (px - ax) * (cy - ay) - (cx - ax) * (py - ay);

    let has_neg = d1 < 0.0 || d2 < 0.0 || d3 < 0.0;
    let has_pos = d1 > 0.0 || d2 > 0.0 || d3 > 0.0;
    !(has_neg && has_pos)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signed_area_ccw_square() {
        // CCW unit square in Y-up coordinates
        let p = Polygon::new(vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]);
        assert!((p.signed_area() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn signed_area_cw_square() {
        let p = Polygon::new(vec![(0.0, 0.0), (0.0, 1.0), (1.0, 1.0), (1.0, 0.0)]);
        assert!((p.signed_area() + 1.0).abs() < 1e-6);
    }

    #[test]
    fn triangulate_triangle() {
        let p = Polygon::new(vec![(0.0, 0.0), (1.0, 0.0), (0.0, 1.0)]);
        let tris = p.triangulate();
        assert_eq!(tris.len(), 1);
    }

    #[test]
    fn triangulate_convex_square() {
        let p = Polygon::new(vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]);
        let tris = p.triangulate();
        assert_eq!(tris.len(), 2);
    }

    #[test]
    fn triangulate_convex_pentagon() {
        let p = Polygon::new(vec![
            (0.0, 1.0),
            (-0.95, 0.31),
            (-0.59, -0.81),
            (0.59, -0.81),
            (0.95, 0.31),
        ]);
        let tris = p.triangulate();
        assert_eq!(tris.len(), 3); // n - 2
    }

    #[test]
    fn triangulate_concave_l_shape() {
        // L-shape: 6 vertices, concave at one corner
        let p = Polygon::new(vec![
            (0.0, 0.0),
            (2.0, 0.0),
            (2.0, 1.0),
            (1.0, 1.0),
            (1.0, 2.0),
            (0.0, 2.0),
        ]);
        let tris = p.triangulate();
        assert_eq!(tris.len(), 4); // n - 2

        // Sanity: total area of triangles equals polygon area (3.0 for the L).
        let mut area = 0.0;
        for [a, b, c] in &tris {
            let (ax, ay) = p.points[*a];
            let (bx, by) = p.points[*b];
            let (cx, cy) = p.points[*c];
            area += ((bx - ax) * (cy - ay) - (by - ay) * (cx - ax)).abs() * 0.5;
        }
        assert!((area - 3.0).abs() < 1e-5, "expected area 3.0, got {}", area);
    }

    #[test]
    fn triangulate_concave_arrow() {
        // Inward-pointing arrow (concave) — 7 vertices
        let p = Polygon::new(vec![
            (0.0, 0.0),
            (4.0, 0.0),
            (4.0, 3.0),
            (3.0, 3.0),
            (2.0, 1.5),
            (1.0, 3.0),
            (0.0, 3.0),
        ]);
        let tris = p.triangulate();
        assert_eq!(tris.len(), 5);
    }

    #[test]
    fn triangulate_handles_clockwise_input() {
        // Same L-shape but in CW order
        let p = Polygon::new(vec![
            (0.0, 0.0),
            (0.0, 2.0),
            (1.0, 2.0),
            (1.0, 1.0),
            (2.0, 1.0),
            (2.0, 0.0),
        ]);
        let tris = p.triangulate();
        assert_eq!(tris.len(), 4);
    }

    #[test]
    fn triangulate_degenerate_returns_empty() {
        let p = Polygon::new(vec![(0.0, 0.0), (1.0, 0.0)]);
        assert!(p.triangulate().is_empty());
    }
}

#[derive(Clone, Copy)]
pub struct Circle {
    pub radius: f32,
}

impl Circle {
    pub fn new(radius: f32) -> Self {
        Self { radius }
    }
}

#[derive(Clone, Copy)]
pub struct Ellipse {
    pub radius_x: f32,
    pub radius_y: f32,
}

impl Ellipse {
    pub fn new(radius_x: f32, radius_y: f32) -> Self {
        Self { radius_x, radius_y }
    }
}

#[derive(Clone, Copy)]
pub struct Image {
    pub width: f32,
    pub height: f32,
}

impl Image {
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

#[derive(Clone, Copy)]
pub struct Arc {
    pub radius: f32,
    pub start_angle: f32,
    pub end_angle: f32,
}

impl Arc {
    pub fn new(radius: f32, start_angle: f32, end_angle: f32) -> Self {
        Self { radius, start_angle, end_angle }
    }
}

#[derive(Clone)]
pub struct Text {
    pub content: String,
    pub font_path: String,
    pub font_size: u32,
}

impl Text {
    pub fn new(content: impl Into<String>, font_path: impl Into<String>, font_size: u32) -> Self {
        Self {
            content: content.into(),
            font_path: font_path.into(),
            font_size,
        }
    }
}