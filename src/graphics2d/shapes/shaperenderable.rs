use crate::core::engine::opengl::{
    GL_POINTS, GL_TRIANGLE_FAN, GL_TRIANGLE_STRIP, GL_TRIANGLES, GLfloat, Vec2,
};
use crate::core::{
    Attribute, Color, FontAtlas, Geometry, Mesh, Renderable, Renderer, Shader,
    generate_texture_from_image, load_image,
};
use crate::graphics2d::shapes::{
    Arc as ArcShape, Circle, Ellipse, Image, Line, MultiPoint, Polygon, Polyline, Rectangle,
    RoundedRectangle, ShapeKind, Text, Triangle,
};
use crate::core::math::Mat4;
use std::cell::{OnceCell, RefCell};
use std::collections::HashMap;
use std::f32::consts::PI;
use std::rc::Rc;

const MIN_STROKE_WIDTH: f32 = 1.0;

/// Anchor point used for positioning, rotation, and scaling.
///
/// The anchor is the single point on the shape that:
/// - is placed at the coordinates passed to [`ShapeRenderable::set_position`],
/// - acts as the pivot for rotation,
/// - acts as the origin for scaling.
///
/// `Default` uses each shape's natural anchor. Compass variants resolve
/// against the shape's axis-aligned bounding box (North = top edge,
/// East = right edge, etc. in screen-space Y-down convention).
///
/// `Custom(x, y)` specifies an arbitrary point in the shape's local
/// coordinate space.
///
/// ## Natural defaults
///
/// - `Point`, `Circle`, `Ellipse`, `Image`, `Arc`: center
/// - `Rectangle`, `RoundedRectangle`: north-west corner (top-left)
/// - `Line`, `Polyline`, `Polygon`, `MultiPoint`: first vertex
/// - `Triangle`: centroid
/// - `Text`: north-west corner of the text cell
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Anchor {
    Default,
    Center,
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
    Custom(f32, f32),
}

/// Shared anchor resolution for Rectangle / RoundedRectangle (bbox = (0,0)..(w,h)).
/// Default is NorthWest (0, 0) in Y-down screen space.
fn rectangle_anchor(width: f32, height: f32, anchor: Anchor) -> (f32, f32) {
    resolve_anchor(
        anchor,
        (0.0, 0.0),
        (width, height),
        (0.0, 0.0),
    )
}

/// Axis-aligned bounding box over a slice of points. Panics on empty input.
fn bbox_of_points(points: &[(f32, f32)]) -> ((f32, f32), (f32, f32)) {
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    for &(x, y) in points {
        if x < min_x { min_x = x; }
        if y < min_y { min_y = y; }
        if x > max_x { max_x = x; }
        if y > max_y { max_y = y; }
    }
    ((min_x, min_y), (max_x, max_y))
}

/// Resolve an `Anchor` to a point in the shape's local coordinate space,
/// given the shape's bbox and its natural default.
fn resolve_anchor(
    anchor: Anchor,
    bbox_min: (f32, f32),
    bbox_max: (f32, f32),
    default: (f32, f32),
) -> (f32, f32) {
    let (min_x, min_y) = bbox_min;
    let (max_x, max_y) = bbox_max;
    let cx = (min_x + max_x) * 0.5;
    let cy = (min_y + max_y) * 0.5;
    match anchor {
        Anchor::Default => default,
        Anchor::Center => (cx, cy),
        Anchor::North => (cx, min_y),
        Anchor::NorthEast => (max_x, min_y),
        Anchor::East => (max_x, cy),
        Anchor::SouthEast => (max_x, max_y),
        Anchor::South => (cx, max_y),
        Anchor::SouthWest => (min_x, max_y),
        Anchor::West => (min_x, cy),
        Anchor::NorthWest => (min_x, min_y),
        Anchor::Custom(x, y) => (x, y),
    }
}

#[derive(Clone, Debug)]
pub struct ShapeStyle {
    pub fill: Option<Color>,
    pub stroke_color: Option<Color>,
    pub stroke_width: Option<f32>,
    pub dash_pattern: Option<(f32, f32)>,
}

impl Default for ShapeStyle {
    fn default() -> Self {
        Self {
            fill: Some(Color::from_rgb(1.0, 1.0, 1.0)),
            stroke_color: None,
            stroke_width: None,
            dash_pattern: None,
        }
    }
}

impl ShapeStyle {
    pub fn fill(fill: Color) -> Self {
        Self {
            fill: Some(fill),
            stroke_color: None,
            stroke_width: None,
            dash_pattern: None,
        }
    }

    pub fn stroke(color: Color, width: f32) -> Self {
        Self {
            fill: None,
            stroke_color: Some(color),
            stroke_width: Some(width),
            dash_pattern: None,
        }
    }

    pub fn fill_and_stroke(fill: Color, stroke: Color, width: f32) -> Self {
        Self {
            fill: Some(fill),
            stroke_color: Some(stroke),
            stroke_width: Some(width),
            dash_pattern: None,
        }
    }

    pub fn dashed_stroke(color: Color, width: f32, dash: f32, gap: f32) -> Self {
        Self {
            fill: None,
            stroke_color: Some(color),
            stroke_width: Some(width),
            dash_pattern: Some((dash, gap)),
        }
    }

    pub fn with_dash(mut self, dash: f32, gap: f32) -> Self {
        self.dash_pattern = Some((dash, gap));
        self
    }
}

thread_local! {
    static DEFAULT_SHADER: OnceCell<Rc<Shader>> = OnceCell::new();
}

fn default_shader() -> Rc<Shader> {
    DEFAULT_SHADER.with(|cell| {
        cell.get_or_init(|| {
            let vert_src = include_str!("../shaders/shape.vert");
            let frag_src = include_str!("../shaders/shape.frag");
            Rc::new(
                Shader::compile(vert_src, frag_src, None)
                    .expect("Failed to compile default shader"),
            )
        })
        .clone()
    })
}

thread_local! {
    static DASHED_SHADER: OnceCell<Rc<Shader>> = OnceCell::new();
}

fn dashed_shader() -> Rc<Shader> {
    DASHED_SHADER.with(|cell| {
        cell.get_or_init(|| {
            let vert_src = include_str!("../shaders/dashed.vert");
            let frag_src = include_str!("../shaders/dashed.frag");
            Rc::new(
                Shader::compile(vert_src, frag_src, None)
                    .expect("Failed to compile dashed shader"),
            )
        })
        .clone()
    })
}

thread_local! {
    static POINT_SHADER: OnceCell<Rc<Shader>> = OnceCell::new();
}

fn point_shader() -> Rc<Shader> {
    POINT_SHADER.with(|cell| {
        cell.get_or_init(|| {
            let vert_src = include_str!("../shaders/shape.vert");
            let frag_src = include_str!("../shaders/point.frag");
            Rc::new(
                Shader::compile(vert_src, frag_src, None).expect("Failed to compile point shader"),
            )
        })
        .clone()
    })
}

thread_local! {
    static IMAGE_SHADER: OnceCell<Rc<Shader>> = OnceCell::new();
}
fn image_shader() -> Rc<Shader> {
    IMAGE_SHADER.with(|cell| {
        cell.get_or_init(|| {
            let vert_src = include_str!("../shaders/image.vert");
            let frag_src = include_str!("../shaders/image.frag");
            Rc::new(
                Shader::compile(vert_src, frag_src, None).expect("Failed to compile image shader"),
            )
        })
        .clone()
    })
}

thread_local! {
    static TEXT_SHADER: OnceCell<Rc<Shader>> = OnceCell::new();
}
fn text_shader() -> Rc<Shader> {
    TEXT_SHADER.with(|cell| {
        cell.get_or_init(|| {
            let vert_src = include_str!("../shaders/text.vert");
            let frag_src = include_str!("../shaders/text.frag");
            Rc::new(
                Shader::compile(vert_src, frag_src, None).expect("Failed to compile text shader"),
            )
        })
        .clone()
    })
}

/// Font cache key: (font_path, font_size)
type FontCacheKey = (String, u32);

thread_local! {
    /// Global font cache - shares FontAtlas instances across text renderables.
    /// Properly dropped when thread exits, no memory leaks.
    static FONT_CACHE: RefCell<HashMap<FontCacheKey, Rc<RefCell<FontAtlas>>>> = RefCell::new(HashMap::new());
}

/// Get or create a FontAtlas from the cache
fn get_or_create_font_atlas(font_path: &str, font_size: u32) -> Rc<RefCell<FontAtlas>> {
    FONT_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        let key = (font_path.to_string(), font_size);

        if let Some(atlas) = cache.get(&key) {
            return atlas.clone();
        }

        // Create new FontAtlas and cache it
        let atlas = FontAtlas::new(font_path, font_size, 512)
            .expect("Failed to create font atlas");
        let atlas_rc = Rc::new(RefCell::new(atlas));
        cache.insert(key, atlas_rc.clone());
        atlas_rc
    })
}

/// Clear the font cache, releasing all FontAtlas resources.
/// Call this when changing scenes or when fonts are no longer needed.
/// Safe to call at any time - new text will recreate atlases as needed.
pub fn clear_font_cache() {
    FONT_CACHE.with(|cache| {
        cache.borrow_mut().clear();
    });
}

fn ortho_2d(width: f32, height: f32) -> Mat4 {
    Mat4::orthographic_rh_gl(0.0, width, height, 0.0, -1.0, 1.0)
}
pub struct ShapeRenderable {
    x: f32,
    y: f32,
    scale: f32,
    rotation: f32,
    z_order: i32,
    mesh: Mesh,
    stroke_mesh: Option<Mesh>,
    shape: ShapeKind,
}
impl Renderable for ShapeRenderable {
    fn render(&mut self, renderer: &Renderer) {
        let (window_width, window_height) = renderer.window_handle.size();
        let transform = ortho_2d(window_width as f32, window_height as f32);
        self.mesh.set_transform(transform);
        self.mesh.set_scale(self.scale);
        self.mesh.set_rotation(self.rotation);

        if self.mesh.geometry.instance_count() > 0 {
            // instanced: u_offset = (0,0), positions come from attrib 1
            renderer.draw_mesh_instanced(&self.mesh);
        } else {
            // single: use u_offset
            self.mesh.set_screen_offset(self.x, self.y);
            renderer.draw_mesh(&self.mesh);
        }

        // Render stroke on top if present
        if let Some(stroke) = &mut self.stroke_mesh {
            stroke.set_transform(transform);
            stroke.set_scale(self.scale);
            stroke.set_rotation(self.rotation);

            if stroke.geometry.instance_count() > 0 {
                renderer.draw_mesh_instanced(stroke);
            } else {
                stroke.set_screen_offset(self.x, self.y);
                renderer.draw_mesh(stroke);
            }
        }
    }
}

impl ShapeRenderable {
    fn new(mesh: Mesh, shape: ShapeKind) -> Self {
        Self { x: 0.0, y: 0.0, scale: 1.0, rotation: 0.0, z_order: 0, mesh, stroke_mesh: None, shape }
    }

    fn new_with_stroke(mesh: Mesh, stroke_mesh: Mesh, shape: ShapeKind) -> Self {
        Self { x: 0.0, y: 0.0, scale: 1.0, rotation: 0.0, z_order: 0, mesh, stroke_mesh: Some(stroke_mesh), shape }
    }

    pub fn set_position(&mut self, x: f32, y: f32) -> &mut Self {
        self.x = x;
        self.y = y;
        self
    }

    pub fn x(&self) -> f32 {
        self.x
    }

    pub fn y(&self) -> f32 {
        self.y
    }

    pub fn position(&self) -> (f32, f32) {
        (self.x, self.y)
    }

    pub fn set_scale(&mut self, scale: f32) -> &mut Self {
        self.scale = scale;
        self
    }

    pub fn scale(&self) -> f32 {
        self.scale
    }

    pub fn set_rotation(&mut self, angle: f32) -> &mut Self {
        self.rotation = angle;
        self
    }

    pub fn rotation(&self) -> f32 {
        self.rotation
    }

    pub fn set_z_order(&mut self, z_order: i32) -> &mut Self {
        self.z_order = z_order;
        self
    }

    pub fn z_order(&self) -> i32 {
        self.z_order
    }

    pub fn set_fill_color(&mut self, color: Color) -> &mut Self {
        self.mesh.color = Some(color);
        self
    }

    pub fn set_stroke_color(&mut self, color: Color) -> &mut Self {
        if let Some(stroke) = &mut self.stroke_mesh {
            stroke.color = Some(color);
        }
        self
    }

    pub fn fill_color(&self) -> Option<Color> {
        self.mesh.color
    }

    pub fn stroke_color(&self) -> Option<Color> {
        self.stroke_mesh.as_ref().and_then(|s| s.color)
    }
    /// Construct a `ShapeRenderable` with the shape's default anchor.
    ///
    /// Equivalent to `ShapeRenderable::builder(shape, style).build()`.
    pub fn from_shape(shape: ShapeKind, style: ShapeStyle) -> Self {
        Self::builder(shape, style).build()
    }

    /// Start a builder that lets you override the anchor (and, later, other
    /// per-shape parameters) before constructing the `ShapeRenderable`.
    pub fn builder(shape: ShapeKind, style: ShapeStyle) -> ShapeRenderableBuilder {
        ShapeRenderableBuilder {
            shape,
            style,
            anchor: Anchor::Default,
        }
    }

    fn from_shape_with_anchor(shape: ShapeKind, style: ShapeStyle, anchor: Anchor) -> Self {
        match shape {
            ShapeKind::Point => {
                ShapeRenderable::point(style.fill.unwrap_or(Color::white()), anchor)
            }
            ShapeKind::MultiPoint(mp) => {
                ShapeRenderable::multi_points(mp, style.fill.unwrap_or(Color::white()), anchor)
            }
            ShapeKind::Line(line) => ShapeRenderable::line(
                line,
                style.stroke_color.unwrap_or_else(Color::white),
                style.stroke_width.unwrap_or(1.0),
                anchor,
                style.dash_pattern,
            ),
            ShapeKind::Polyline(poly_line) => ShapeRenderable::polyline(
                poly_line,
                style.stroke_color.unwrap_or(Color::white()),
                style.stroke_width.unwrap_or(1.0),
                anchor,
                style.dash_pattern,
            ),
            ShapeKind::Triangle(triangle) => ShapeRenderable::triangle(
                triangle,
                style.fill.unwrap_or(Color::white()),
                anchor,
            ),
            ShapeKind::Rectangle(rect) => match (style.fill, style.stroke_color) {
                (Some(fill), Some(stroke)) => ShapeRenderable::rectangle_fill_and_stroke(
                    rect,
                    fill,
                    stroke,
                    style.stroke_width.unwrap_or(1.0),
                    anchor,
                    style.dash_pattern,
                ),
                (None, Some(stroke)) => ShapeRenderable::rectangle_outline(
                    rect,
                    stroke,
                    style.stroke_width.unwrap_or(1.0),
                    anchor,
                    style.dash_pattern,
                ),
                (fill, None) => ShapeRenderable::rectangle(
                    rect,
                    fill.unwrap_or(Color::white()),
                    anchor,
                ),
            },
            ShapeKind::RoundedRectangle(rr) => ShapeRenderable::rounded_rectangle(
                rr,
                style.fill.unwrap_or(Color::white()),
                anchor,
            ),
            ShapeKind::Polygon(polygon) => ShapeRenderable::polygon(
                polygon,
                style.fill.unwrap_or(Color::white()),
                anchor,
            ),
            ShapeKind::Circle(circle) => ShapeRenderable::circle(
                circle,
                style.fill.unwrap_or(Color::white()),
                anchor,
            ),
            ShapeKind::Ellipse(ellipse) => ShapeRenderable::ellipse(
                ellipse,
                style.fill.unwrap_or(Color::white()),
                anchor,
            ),
            ShapeKind::Arc(arc) => ShapeRenderable::arc(
                arc,
                style.stroke_color.unwrap_or(Color::white()),
                style.stroke_width.unwrap_or(1.0),
                anchor,
                style.dash_pattern,
            ),
            ShapeKind::Image(_) => {
                unimplemented!("ShapeRenderable::from_shape cannot create Image without path")
            }
            ShapeKind::Text(text) => ShapeRenderable::text(
                text,
                style.fill.unwrap_or(Color::white()),
                anchor,
            ),
        }
    }

    pub fn create_multiple_instances(&mut self, capacity: usize) {
        self.mesh.geometry.enable_instancing_xy(capacity);
        if let Some(stroke) = &mut self.stroke_mesh {
            stroke.geometry.enable_instancing_xy(capacity);
        }
    }

    pub fn set_instance_positions(&mut self, positions: &[Vec2]) -> &mut Self {
        self.mesh.geometry.update_instance_xy(positions);
        if let Some(stroke) = &mut self.stroke_mesh {
            stroke.geometry.update_instance_xy(positions);
        }
        self
    }

    pub fn set_instance_colors(&mut self, colors: &[Color]) -> &mut Self {
        self.mesh.geometry.update_instance_colors(colors);
        self
    }

    pub fn set_instance_stroke_colors(&mut self, colors: &[Color]) -> &mut Self {
        if let Some(stroke) = &mut self.stroke_mesh {
            stroke.geometry.update_instance_colors(colors);
        }
        self
    }

    pub fn clear_instances(&mut self) {
        self.mesh.geometry.clear_instancing();
        if let Some(stroke) = &mut self.stroke_mesh {
            stroke.geometry.clear_instancing();
        }
    }

    fn point(color: Color, _anchor: Anchor) -> Self {
        // Point has only one vertex at (0, 0); anchor is trivially that point
        // for every variant (bbox is degenerate). Ignore the anchor.
        let geometry = ShapeRenderable::point_geometry();
        let mesh = Mesh::with_color(point_shader(), geometry, Some(color));
        ShapeRenderable::new(mesh, ShapeKind::Point)
    }

    fn multi_points(multi_point: MultiPoint, color: Color, anchor: Anchor) -> Self {
        assert!(!multi_point.points.is_empty(), "MultiPoint requires at least one point");

        let (bbox_min, bbox_max) = bbox_of_points(&multi_point.points);
        let default = multi_point.points[0];
        let (ax, ay) = resolve_anchor(anchor, bbox_min, bbox_max, default);

        let rel_points: Vec<(GLfloat, GLfloat)> = multi_point
            .points
            .iter()
            .map(|(px, py)| (px - ax, py - ay))
            .collect();

        let geometry = ShapeRenderable::point_list_geometry(&rel_points);
        let mesh = Mesh::with_color(point_shader(), geometry, Some(color));

        let mut s = ShapeRenderable::new(mesh, ShapeKind::MultiPoint(multi_point));
        s.x = ax;
        s.y = ay;
        s
    }

    fn line(shape: Line, stroke: Color, stroke_width: f32, anchor: Anchor, dash_pattern: Option<(f32, f32)>) -> Self {
        let (x1, y1) = shape.start;
        let (x2, y2) = shape.end;
        let bbox_min = (x1.min(x2), y1.min(y2));
        let bbox_max = (x1.max(x2), y1.max(y2));
        let default = (x1, y1); // start point
        let (ax, ay) = resolve_anchor(anchor, bbox_min, bbox_max, default);

        let (geometry, shader) = if let Some(_) = dash_pattern {
            (ShapeRenderable::line_geometry_dashed(x1 - ax, y1 - ay, x2 - ax, y2 - ay, stroke_width), dashed_shader())
        } else {
            (ShapeRenderable::line_geometry(x1 - ax, y1 - ay, x2 - ax, y2 - ay, stroke_width), default_shader())
        };
        let mut mesh = Mesh::with_color(shader, geometry, Some(stroke));
        if let Some((dash, gap)) = dash_pattern {
            mesh.dash_pattern = Some((dash, gap));
        }

        let mut s = ShapeRenderable::new(mesh, ShapeKind::Line(shape));
        s.x = ax;
        s.y = ay;
        s
    }

    fn polyline(polyline: Polyline, stroke: Color, stroke_width: f32, anchor: Anchor, dash_pattern: Option<(f32, f32)>) -> Self {
        assert!(polyline.points.len() >= 2, "Polyline requires at least two points");

        let (bbox_min, bbox_max) = bbox_of_points(&polyline.points);
        let default = polyline.points[0];
        let (ax, ay) = resolve_anchor(anchor, bbox_min, bbox_max, default);

        let rel_points: Vec<(f32, f32)> = polyline
            .points
            .iter()
            .map(|(px, py)| (px - ax, py - ay))
            .collect();

        let (geometry, shader) = if let Some(_) = dash_pattern {
            (ShapeRenderable::polyline_geometry_dashed(&rel_points, stroke_width), dashed_shader())
        } else {
            (ShapeRenderable::polyline_geometry(&rel_points, stroke_width), default_shader())
        };
        let mut mesh = Mesh::with_color(shader, geometry, Some(stroke));
        if let Some((dash, gap)) = dash_pattern {
            mesh.dash_pattern = Some((dash, gap));
        }

        let mut s = ShapeRenderable::new(mesh, ShapeKind::Polyline(polyline));
        s.x = ax;
        s.y = ay;
        s
    }

    fn arc(arc: ArcShape, stroke: Color, stroke_width: f32, anchor: Anchor, dash_pattern: Option<(f32, f32)>) -> Self {
        use std::f32::consts::TAU;

        let segments = 64;

        // Normalize sweep to [0, TAU)
        let mut sweep = arc.end_angle - arc.start_angle;
        if sweep < 0.0 {
            sweep += TAU;
        }

        // Generate points around the arc's center (local origin).
        let mut points = Vec::with_capacity(segments + 1);
        for i in 0..=segments {
            let t = i as f32 / segments as f32;
            let theta = arc.start_angle + t * sweep;
            let px = arc.radius * theta.cos();
            let py = -arc.radius * theta.sin();
            points.push((px, py));
        }

        // Bbox over the curve points; default anchor is the arc's circle center (0, 0).
        let (bbox_min, bbox_max) = bbox_of_points(&points);
        let default = (0.0, 0.0);
        let (ax, ay) = resolve_anchor(anchor, bbox_min, bbox_max, default);

        let shifted: Vec<(f32, f32)> =
            points.iter().map(|(x, y)| (x - ax, y - ay)).collect();
        let (geometry, shader) = if let Some(_) = dash_pattern {
            (ShapeRenderable::polyline_geometry_dashed(&shifted, stroke_width), dashed_shader())
        } else {
            (ShapeRenderable::polyline_geometry(&shifted, stroke_width), default_shader())
        };
        let mut mesh = Mesh::with_color(shader, geometry, Some(stroke));
        if let Some((dash, gap)) = dash_pattern {
            mesh.dash_pattern = Some((dash, gap));
        }

        let mut s = ShapeRenderable::new(mesh, ShapeKind::Arc(arc));
        s.x = ax;
        s.y = ay;
        s
    }

    fn triangle(triangle: Triangle, color: Color, anchor: Anchor) -> Self {
        let [v0, v1, v2] = triangle.vertices;
        let bbox_min = (v0.0.min(v1.0).min(v2.0), v0.1.min(v1.1).min(v2.1));
        let bbox_max = (v0.0.max(v1.0).max(v2.0), v0.1.max(v1.1).max(v2.1));
        let default = triangle.centroid();
        let (ax, ay) = resolve_anchor(anchor, bbox_min, bbox_max, default);

        let shifted = [
            (v0.0 - ax, v0.1 - ay),
            (v1.0 - ax, v1.1 - ay),
            (v2.0 - ax, v2.1 - ay),
        ];
        let geometry = ShapeRenderable::triangle_geometry(&shifted);
        let mesh = Mesh::with_color(default_shader(), geometry, Some(color));

        let mut s = ShapeRenderable::new(mesh, ShapeKind::Triangle(triangle));
        s.x = ax;
        s.y = ay;
        s
    }

    fn rectangle(rect: Rectangle, color: Color, anchor: Anchor) -> Self {
        let (ax, ay) = rectangle_anchor(rect.width, rect.height, anchor);
        let geometry = ShapeRenderable::rectangle_geometry(rect.width, rect.height, ax, ay);
        let mesh = Mesh::with_color(default_shader(), geometry, Some(color));

        let mut s = ShapeRenderable::new(mesh, ShapeKind::Rectangle(rect));
        s.x = ax;
        s.y = ay;
        s
    }

    fn rectangle_outline(rect: Rectangle, stroke: Color, stroke_width: f32, anchor: Anchor, dash_pattern: Option<(f32, f32)>) -> Self {
        let (ax, ay) = rectangle_anchor(rect.width, rect.height, anchor);
        // Closed polyline in the same local frame as the fill, shifted by anchor.
        // The direction hint (6th point) retraces the top edge for a proper miter join.
        // For dashed lines, skip it — the overlap would fill in the gaps on the top edge.
        let mut points = vec![
            (0.0 - ax, 0.0 - ay),
            (rect.width - ax, 0.0 - ay),
            (rect.width - ax, rect.height - ay),
            (0.0 - ax, rect.height - ay),
            (0.0 - ax, 0.0 - ay),                // close the loop
        ];
        if dash_pattern.is_none() {
            points.push((rect.width - ax, 0.0 - ay)); // direction hint for join at closing corner
        }

        let (geometry, shader) = if let Some(_) = dash_pattern {
            (ShapeRenderable::polyline_geometry_dashed(&points, stroke_width), dashed_shader())
        } else {
            (ShapeRenderable::polyline_geometry(&points, stroke_width), default_shader())
        };
        let mut mesh = Mesh::with_color(shader, geometry, Some(stroke));
        if let Some((dash, gap)) = dash_pattern {
            mesh.dash_pattern = Some((dash, gap));
        }

        let mut s = ShapeRenderable::new(mesh, ShapeKind::Rectangle(rect));
        s.x = ax;
        s.y = ay;
        s
    }

    fn rectangle_fill_and_stroke(
        rect: Rectangle,
        fill: Color,
        stroke: Color,
        stroke_width: f32,
        anchor: Anchor,
        dash_pattern: Option<(f32, f32)>,
    ) -> Self {
        let (ax, ay) = rectangle_anchor(rect.width, rect.height, anchor);

        let fill_geometry = ShapeRenderable::rectangle_geometry(rect.width, rect.height, ax, ay);
        let fill_mesh = Mesh::with_color(default_shader(), fill_geometry, Some(fill));

        let mut points = vec![
            (0.0 - ax, 0.0 - ay),
            (rect.width - ax, 0.0 - ay),
            (rect.width - ax, rect.height - ay),
            (0.0 - ax, rect.height - ay),
            (0.0 - ax, 0.0 - ay),
        ];
        if dash_pattern.is_none() {
            points.push((rect.width - ax, 0.0 - ay));
        }
        let (stroke_geometry, shader) = if let Some(_) = dash_pattern {
            (ShapeRenderable::polyline_geometry_dashed(&points, stroke_width), dashed_shader())
        } else {
            (ShapeRenderable::polyline_geometry(&points, stroke_width), default_shader())
        };
        let mut stroke_mesh = Mesh::with_color(shader, stroke_geometry, Some(stroke));
        if let Some((dash, gap)) = dash_pattern {
            stroke_mesh.dash_pattern = Some((dash, gap));
        }

        let mut s = ShapeRenderable::new_with_stroke(
            fill_mesh,
            stroke_mesh,
            ShapeKind::Rectangle(rect),
        );
        s.x = ax;
        s.y = ay;
        s
    }

    fn rounded_rectangle(rr: RoundedRectangle, color: Color, anchor: Anchor) -> Self {
        let (ax, ay) = rectangle_anchor(rr.width, rr.height, anchor);
        let geometry =
            ShapeRenderable::rounded_rectangle_geometry(rr.width, rr.height, rr.radius, 8, ax, ay);
        let mesh = Mesh::with_color(default_shader(), geometry, Some(color));

        let mut s = ShapeRenderable::new(mesh, ShapeKind::RoundedRectangle(rr));
        s.x = ax;
        s.y = ay;
        s
    }

    fn polygon(polygon: Polygon, color: Color, anchor: Anchor) -> Self {
        assert!(polygon.points.len() >= 3, "Polygon requires at least 3 points");

        let (bbox_min, bbox_max) = bbox_of_points(&polygon.points);
        let default = polygon.points[0];
        let (ax, ay) = resolve_anchor(anchor, bbox_min, bbox_max, default);

        let rel_points: Vec<(f32, f32)> = polygon
            .points
            .iter()
            .map(|(px, py)| (px - ax, py - ay))
            .collect();
        let triangles = polygon.triangulate();

        let geometry = ShapeRenderable::polygon_geometry(&rel_points, &triangles);
        let mesh = Mesh::with_color(default_shader(), geometry, Some(color));

        let mut s = ShapeRenderable::new(mesh, ShapeKind::Polygon(polygon));
        s.x = ax;
        s.y = ay;
        s
    }

    fn circle(circle: Circle, color: Color, anchor: Anchor) -> Self {
        let r = circle.radius;
        let (ax, ay) = resolve_anchor(anchor, (-r, -r), (r, r), (0.0, 0.0));
        let geometry = ShapeRenderable::circle_geometry(r, 100, ax, ay);
        let mesh = Mesh::with_color(default_shader(), geometry, Some(color));

        let mut s = ShapeRenderable::new(mesh, ShapeKind::Circle(circle));
        s.x = ax;
        s.y = ay;
        s
    }

    fn ellipse(ellipse: Ellipse, color: Color, anchor: Anchor) -> Self {
        let rx = ellipse.radius_x;
        let ry = ellipse.radius_y;
        let (ax, ay) = resolve_anchor(anchor, (-rx, -ry), (rx, ry), (0.0, 0.0));
        let geometry = ShapeRenderable::ellipse_geometry(rx, ry, 64, ax, ay);
        let mesh = Mesh::with_color(default_shader(), geometry, Some(color));

        let mut s = ShapeRenderable::new(mesh, ShapeKind::Ellipse(ellipse));
        s.x = ax;
        s.y = ay;
        s
    }

    fn text(text: Text, color: Color, anchor: Anchor) -> Self {
        let font_atlas = get_or_create_font_atlas(&text.font_path, text.font_size);

        // Generate raw glyph vertices and compute the bbox in one pass.
        let (mut vertices, bbox_min, bbox_max, texture_id) = {
            let mut atlas = font_atlas.borrow_mut();
            let (vs, bmin, bmax) = ShapeRenderable::text_raw_vertices(&text.content, &mut atlas);
            let tex = atlas.texture_id();
            (vs, bmin, bmax, tex)
        };

        // Default anchor for Text is the top-left of the text cell (raw origin).
        let (ax, ay) = resolve_anchor(anchor, bbox_min, bbox_max, (0.0, 0.0));

        // Apply the anchor shift to the position components (x, y at stride 4).
        if ax != 0.0 || ay != 0.0 {
            let stride = 4usize;
            let mut i = 0;
            while i + 1 < vertices.len() {
                vertices[i] -= ax;
                vertices[i + 1] -= ay;
                i += stride;
            }
        }

        let mut geometry = Geometry::new(GL_TRIANGLES);
        geometry.add_buffer(&vertices, 4);
        geometry.add_vertex_attribute(Attribute::new(0, 2, 4, 0));
        geometry.add_vertex_attribute(Attribute::new(1, 2, 4, 2));

        let shader = text_shader();
        let mut mesh = Mesh::with_texture(shader, geometry, Some(texture_id));
        mesh.color = Some(color);

        let mut s = ShapeRenderable::new(mesh, ShapeKind::Text(text));
        s.x = ax;
        s.y = ay;
        s
    }

    pub fn image_with_size(path: &str, width: f32, height: f32) -> ShapeRenderable {
        Self::image_with_size_and_anchor(path, width, height, Anchor::Default)
    }

    fn image_with_size_and_anchor(
        path: &str,
        width: f32,
        height: f32,
        anchor: Anchor,
    ) -> ShapeRenderable {
        let image = load_image(path);
        let texture_id = generate_texture_from_image(&image);

        // Image geometry is built centered on origin, so bbox = (-w/2..w/2, -h/2..h/2)
        let hw = width * 0.5;
        let hh = height * 0.5;
        let (ax, ay) = resolve_anchor(anchor, (-hw, -hh), (hw, hh), (0.0, 0.0));

        let geometry = ShapeRenderable::image_geometry(width, height, ax, ay);
        let shader = image_shader();
        let mesh = Mesh::with_texture(shader, geometry, Some(texture_id));

        let mut s =
            ShapeRenderable::new(mesh, ShapeKind::Image(Image::new(width, height)));
        s.x = ax;
        s.y = ay;
        s
    }

    pub fn image(path: &str) -> Self {
        let image = load_image(path);
        Self::image_with_size(path, image.width as f32, image.height as f32)
    }

    fn point_geometry() -> Geometry {
        let vertex = vec![0.0, 0.0];
        let mut geometry = Geometry::new(GL_POINTS);
        geometry.add_buffer(&vertex, 2);

        geometry.add_vertex_attribute(Attribute::new(0, 2, 2, 0));

        geometry
    }

    fn point_list_geometry(points: &[(GLfloat, GLfloat)]) -> Geometry {
        let mut vertices = Vec::with_capacity(points.len() * 2);

        for &(x, y) in points {
            vertices.push(x);
            vertices.push(y);
        }

        let values_per_vertex = 2;

        let mut geometry = Geometry::new(GL_POINTS);
        geometry.add_buffer(&vertices, values_per_vertex);

        geometry.add_vertex_attribute(Attribute::new(
            0, // position
            values_per_vertex,
            values_per_vertex as usize,
            0,
        ));

        geometry
    }

    fn line_geometry(
        x1: GLfloat,
        y1: GLfloat,
        x2: GLfloat,
        y2: GLfloat,
        stroke_width: f32,
    ) -> Geometry {
        let stroke_width = stroke_width.max(MIN_STROKE_WIDTH);
        let dx = x2 - x1;
        let dy = y2 - y1;
        let length = (dx * dx + dy * dy).sqrt();

        if length == 0.0 {
            return Geometry::new(GL_TRIANGLES);
        }

        // Unit perpendicular vector
        let nx = -dy / length;
        let ny = dx / length;
        let half_thickness = stroke_width / 2.0;

        // Offset vector
        let ox = nx * half_thickness;
        let oy = ny * half_thickness;

        // Four corners of the quad
        let v0 = [x1 - ox, y1 - oy];
        let v1 = [x2 - ox, y2 - oy];
        let v2 = [x2 + ox, y2 + oy];
        let v3 = [x1 + ox, y1 + oy];

        let vertices: Vec<GLfloat> = vec![
            v0[0], v0[1], v1[0], v1[1], v2[0], v2[1], v2[0], v2[1], v3[0], v3[1], v0[0], v0[1],
        ];

        let position_values_per_vertex = 2;

        let mut geometry = Geometry::new(GL_TRIANGLES);
        geometry.add_buffer(&vertices, position_values_per_vertex);
        geometry.add_vertex_attribute(Attribute::new(
            0,
            position_values_per_vertex,
            position_values_per_vertex as usize,
            0,
        ));

        geometry
    }

    /// Polyline triangulation adapted from JVPolyline by Julien Vernay (2025)
    ///
    /// Original C implementation:
    /// https://jvernay.fr/en/blog/polyline-triangulation/
    /// Source: https://git.sr.ht/~jvernay/JV/tree/main/item/src/jv_polyline/jv_polyline.c
    ///
    /// This implementation is based on the original algorithm,
    /// restructured and translated to idiomatic Rust for use in wilhelm_renderer.
    fn polyline_geometry(points: &[(GLfloat, GLfloat)], stroke_width: f32) -> Geometry {
        const MITER_LIMIT: f32 = 4.0; // Equivalent to JV default

        if points.len() < 2 {
            return Geometry::new(GL_TRIANGLES);
        }

        let half_thickness = stroke_width.max(1.0) / 2.0;
        let miter_limit_squared = (stroke_width * MITER_LIMIT).powi(2) / 4.0;
        let mut vertices: Vec<GLfloat> = Vec::new();

        let mut a = points[0];
        let mut b = points[1];

        let mut idx = 1;
        while idx < points.len() && (b.0 - a.0).hypot(b.1 - a.1) == 0.0 {
            idx += 1;
            if idx < points.len() {
                b = points[idx];
            }
        }
        if (b.0 - a.0).hypot(b.1 - a.1) == 0.0 {
            return Geometry::new(GL_TRIANGLES);
        }

        for i in idx + 1..=points.len() {
            let c = if i < points.len() { points[i] } else { a }; // fake point if last

            let ab = (b.0 - a.0, b.1 - a.1);
            let len_ab = (ab.0 * ab.0 + ab.1 * ab.1).sqrt();
            let normal_ab = (
                -ab.1 / len_ab * half_thickness,
                ab.0 / len_ab * half_thickness,
            );

            let a1 = (a.0 + normal_ab.0, a.1 + normal_ab.1);
            let a2 = (a.0 - normal_ab.0, a.1 - normal_ab.1);
            let b1 = (b.0 + normal_ab.0, b.1 + normal_ab.1);
            let b2 = (b.0 - normal_ab.0, b.1 - normal_ab.1);

            // segment quad
            vertices.extend_from_slice(&[
                a1.0, a1.1, a2.0, a2.1, b1.0, b1.1, a2.0, a2.1, b1.0, b1.1, b2.0, b2.1,
            ]);

            let bc = (c.0 - b.0, c.1 - b.1);
            let len_bc = (bc.0 * bc.0 + bc.1 * bc.1).sqrt();
            if len_bc > 0.0 {
                let normal_bc = (
                    -bc.1 / len_bc * half_thickness,
                    bc.0 / len_bc * half_thickness,
                );
                let b3 = (b.0 + normal_bc.0, b.1 + normal_bc.1);
                let b4 = (b.0 - normal_bc.0, b.1 - normal_bc.1);

                // turn direction
                let z = ab.0 * bc.1 - ab.1 * bc.0;

                // bevel join
                if z < 0.0 {
                    vertices.extend_from_slice(&[b.0, b.1, b1.0, b1.1, b3.0, b3.1]);
                } else if z > 0.0 {
                    vertices.extend_from_slice(&[b.0, b.1, b2.0, b2.1, b4.0, b4.1]);
                }

                // optional miter
                if z != 0.0 {
                    let (a_j, b_j, norm_j) = if z < 0.0 { (a1, b3, ab) } else { (a2, b4, ab) };

                    let denom = z;
                    let alpha = (bc.1 * (b_j.0 - a_j.0) + bc.0 * (a_j.1 - b_j.1)) / denom;
                    let mx = a_j.0 + alpha * norm_j.0;
                    let my = a_j.1 + alpha * norm_j.1;

                    let dist2 = (mx - b.0).powi(2) + (my - b.1).powi(2);
                    if dist2 <= miter_limit_squared {
                        if z < 0.0 {
                            vertices.extend_from_slice(&[mx, my, b1.0, b1.1, b3.0, b3.1]);
                        } else {
                            vertices.extend_from_slice(&[mx, my, b2.0, b2.1, b4.0, b4.1]);
                        }
                    }
                }
            }

            a = b;
            b = c;
        }

        let mut geometry = Geometry::new(GL_TRIANGLES);
        geometry.add_buffer(&vertices, 2);
        geometry.add_vertex_attribute(Attribute::new(0, 2, 2, 0));
        geometry
    }

    fn line_geometry_dashed(
        x1: GLfloat,
        y1: GLfloat,
        x2: GLfloat,
        y2: GLfloat,
        stroke_width: f32,
    ) -> Geometry {
        let stroke_width = stroke_width.max(MIN_STROKE_WIDTH);
        let dx = x2 - x1;
        let dy = y2 - y1;
        let length = (dx * dx + dy * dy).sqrt();

        if length == 0.0 {
            return Geometry::new(GL_TRIANGLES);
        }

        let nx = -dy / length;
        let ny = dx / length;
        let half_thickness = stroke_width / 2.0;

        let ox = nx * half_thickness;
        let oy = ny * half_thickness;

        // Four corners with distance: start=0, end=length
        // Layout: x, y, dist per vertex
        let vertices: Vec<GLfloat> = vec![
            x1 - ox, y1 - oy, 0.0,
            x2 - ox, y2 - oy, length,
            x2 + ox, y2 + oy, length,
            x2 + ox, y2 + oy, length,
            x1 + ox, y1 + oy, 0.0,
            x1 - ox, y1 - oy, 0.0,
        ];

        let mut geometry = Geometry::new(GL_TRIANGLES);
        geometry.add_buffer(&vertices, 3);
        geometry.add_vertex_attribute(Attribute::new(0, 2, 3, 0)); // vec2 position
        geometry.add_vertex_attribute(Attribute::new(3, 1, 3, 2)); // float distance
        geometry
    }

    fn polyline_geometry_dashed(points: &[(GLfloat, GLfloat)], stroke_width: f32) -> Geometry {
        const MITER_LIMIT: f32 = 4.0;

        if points.len() < 2 {
            return Geometry::new(GL_TRIANGLES);
        }

        let half_thickness = stroke_width.max(1.0) / 2.0;
        let miter_limit_squared = (stroke_width * MITER_LIMIT).powi(2) / 4.0;
        let mut vertices: Vec<GLfloat> = Vec::new();

        // Pre-compute cumulative distances
        let mut cum_dist = vec![0.0f32; points.len()];
        for i in 1..points.len() {
            let dx = points[i].0 - points[i - 1].0;
            let dy = points[i].1 - points[i - 1].1;
            cum_dist[i] = cum_dist[i - 1] + (dx * dx + dy * dy).sqrt();
        }

        let mut a = points[0];
        let mut b = points[1];
        let mut a_idx = 0usize;
        let mut b_idx = 1usize;

        let mut idx = 1;
        while idx < points.len() && (b.0 - a.0).hypot(b.1 - a.1) == 0.0 {
            idx += 1;
            if idx < points.len() {
                b = points[idx];
                b_idx = idx;
            }
        }
        if (b.0 - a.0).hypot(b.1 - a.1) == 0.0 {
            return Geometry::new(GL_TRIANGLES);
        }

        for i in idx + 1..=points.len() {
            let c = if i < points.len() { points[i] } else { a };

            let ab = (b.0 - a.0, b.1 - a.1);
            let len_ab = (ab.0 * ab.0 + ab.1 * ab.1).sqrt();
            let normal_ab = (
                -ab.1 / len_ab * half_thickness,
                ab.0 / len_ab * half_thickness,
            );

            let a1 = (a.0 + normal_ab.0, a.1 + normal_ab.1);
            let a2 = (a.0 - normal_ab.0, a.1 - normal_ab.1);
            let b1 = (b.0 + normal_ab.0, b.1 + normal_ab.1);
            let b2 = (b.0 - normal_ab.0, b.1 - normal_ab.1);

            let da = cum_dist[a_idx];
            let db = cum_dist[b_idx];

            // segment quad with distance
            vertices.extend_from_slice(&[
                a1.0, a1.1, da, a2.0, a2.1, da, b1.0, b1.1, db,
                a2.0, a2.1, da, b1.0, b1.1, db, b2.0, b2.1, db,
            ]);

            let bc = (c.0 - b.0, c.1 - b.1);
            let len_bc = (bc.0 * bc.0 + bc.1 * bc.1).sqrt();
            if len_bc > 0.0 {
                let normal_bc = (
                    -bc.1 / len_bc * half_thickness,
                    bc.0 / len_bc * half_thickness,
                );
                let b3 = (b.0 + normal_bc.0, b.1 + normal_bc.1);
                let b4 = (b.0 - normal_bc.0, b.1 - normal_bc.1);

                let z = ab.0 * bc.1 - ab.1 * bc.0;

                // bevel join — all vertices at junction distance
                if z < 0.0 {
                    vertices.extend_from_slice(&[b.0, b.1, db, b1.0, b1.1, db, b3.0, b3.1, db]);
                } else if z > 0.0 {
                    vertices.extend_from_slice(&[b.0, b.1, db, b2.0, b2.1, db, b4.0, b4.1, db]);
                }

                // optional miter
                if z != 0.0 {
                    let (a_j, b_j, norm_j) = if z < 0.0 { (a1, b3, ab) } else { (a2, b4, ab) };

                    let denom = z;
                    let alpha = (bc.1 * (b_j.0 - a_j.0) + bc.0 * (a_j.1 - b_j.1)) / denom;
                    let mx = a_j.0 + alpha * norm_j.0;
                    let my = a_j.1 + alpha * norm_j.1;

                    let dist2 = (mx - b.0).powi(2) + (my - b.1).powi(2);
                    if dist2 <= miter_limit_squared {
                        if z < 0.0 {
                            vertices.extend_from_slice(&[mx, my, db, b1.0, b1.1, db, b3.0, b3.1, db]);
                        } else {
                            vertices.extend_from_slice(&[mx, my, db, b2.0, b2.1, db, b4.0, b4.1, db]);
                        }
                    }
                }
            }

            a = b;
            a_idx = b_idx;
            b = c;
            if i < points.len() {
                b_idx = i;
            }
        }

        let mut geometry = Geometry::new(GL_TRIANGLES);
        geometry.add_buffer(&vertices, 3);
        geometry.add_vertex_attribute(Attribute::new(0, 2, 3, 0)); // vec2 position
        geometry.add_vertex_attribute(Attribute::new(3, 1, 3, 2)); // float distance
        geometry
    }

    fn triangle_geometry(vertices: &[(f32, f32); 3]) -> Geometry {
        let mut geometry = Geometry::new(GL_TRIANGLES);
        let flattened: Vec<f32> = vertices.iter().flat_map(|(x, y)| [*x, *y]).collect();

        geometry.add_buffer(&flattened, 2);
        geometry.add_vertex_attribute(Attribute::new(0, 2, 2, 0));

        geometry
    }

    fn rectangle_geometry(width: GLfloat, height: GLfloat, ox: GLfloat, oy: GLfloat) -> Geometry {
        let vertices: Vec<GLfloat> = vec![
            0.0 - ox, 0.0 - oy,
            width - ox, 0.0 - oy,
            0.0 - ox, height - oy,
            width - ox, height - oy,
        ];

        let values_per_vertex = 2;
        let mut geometry = Geometry::new(GL_TRIANGLE_STRIP);
        geometry.add_buffer(&vertices, values_per_vertex);
        geometry.add_vertex_attribute(Attribute::new(0, 2, values_per_vertex as usize, 0));
        geometry
    }

    fn circle_geometry(radius: GLfloat, segments: usize, ox: GLfloat, oy: GLfloat) -> Geometry {
        let mut vertices: Vec<GLfloat> = Vec::with_capacity((segments + 2) * 2);

        // Center of the circle
        vertices.extend_from_slice(&[0.0 - ox, 0.0 - oy]);

        for i in 0..=segments {
            let theta = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let x = radius * theta.cos() - ox;
            let y = radius * theta.sin() - oy;
            vertices.extend_from_slice(&[x, y]);
        }

        let values_per_vertex = 2;
        let mut geometry = Geometry::new(GL_TRIANGLE_FAN);
        geometry.add_buffer(&vertices, values_per_vertex);
        geometry.add_vertex_attribute(Attribute::new(0, 2, values_per_vertex as usize, 0));
        geometry
    }

    fn ellipse_geometry(rx: f32, ry: f32, segments: usize, ox: f32, oy: f32) -> Geometry {
        use std::f32::consts::PI;

        let mut vertices: Vec<GLfloat> = Vec::with_capacity((segments + 2) * 2);

        vertices.extend_from_slice(&[0.0 - ox, 0.0 - oy]);

        for i in 0..=segments {
            let angle = 2.0 * PI * (i as f32) / (segments as f32);
            let x = rx * angle.cos() - ox;
            let y = ry * angle.sin() - oy;
            vertices.extend_from_slice(&[x, y]);
        }

        let values_per_vertex = 2;
        let mut geometry = Geometry::new(GL_TRIANGLE_FAN);
        geometry.add_buffer(&vertices, values_per_vertex);
        geometry.add_vertex_attribute(Attribute::new(0, 2, values_per_vertex as usize, 0));
        geometry
    }

    pub fn rounded_rectangle_geometry(
        width: f32,
        height: f32,
        radius: f32,
        segments: usize,
        ox: f32,
        oy: f32,
    ) -> Geometry {
        assert!(radius * 2.0 <= width && radius * 2.0 <= height);

        let mut vertices: Vec<GLfloat> = Vec::new();

        // 1. Add center point for triangle fan
        let center_x = width / 2.0 - ox;
        let center_y = height / 2.0 - oy;
        vertices.push(center_x);
        vertices.push(center_y);

        // 2. Define arcs for each corner: (cx, cy, start_angle, end_angle)
        let corners = [
            (radius, radius, PI, 1.5 * PI),
            (width - radius, radius, 1.5 * PI, 2.0 * PI),
            (width - radius, height - radius, 0.0, 0.5 * PI),
            (radius, height - radius, 0.5 * PI, PI),
        ];

        let mut first_arc_x = 0.0;
        let mut first_arc_y = 0.0;
        let mut is_first = true;

        for &(cx, cy, start_angle, end_angle) in &corners {
            for i in 0..=segments {
                let theta =
                    start_angle + (end_angle - start_angle) * (i as f32) / (segments as f32);
                let x = cx + radius * theta.cos() - ox;
                let y = cy + radius * theta.sin() - oy;

                if is_first {
                    first_arc_x = x;
                    first_arc_y = y;
                    is_first = false;
                }

                vertices.push(x);
                vertices.push(y);
            }
        }

        vertices.push(first_arc_x);
        vertices.push(first_arc_y);

        let values_per_vertex = 2;
        let mut geometry = Geometry::new(GL_TRIANGLE_FAN);
        geometry.add_buffer(&vertices, values_per_vertex);
        geometry.add_vertex_attribute(Attribute::new(0, 2, values_per_vertex as usize, 0));
        geometry
    }

    fn polygon_geometry(
        points: &[(GLfloat, GLfloat)],
        triangles: &[[usize; 3]],
    ) -> Geometry {
        assert!(points.len() >= 3, "Polygon requires at least 3 points");
        assert!(
            !triangles.is_empty(),
            "Polygon triangulation produced no triangles"
        );

        // Expand the index list into a flat vertex buffer of triangles so the
        // geometry can be drawn with GL_TRIANGLES (works for concave polygons,
        // unlike GL_TRIANGLE_FAN which only renders convex shapes correctly).
        let mut vertices = Vec::with_capacity(triangles.len() * 3 * 2);
        for &[a, b, c] in triangles {
            let (ax, ay) = points[a];
            let (bx, by) = points[b];
            let (cx, cy) = points[c];
            vertices.extend_from_slice(&[ax, ay, bx, by, cx, cy]);
        }

        let values_per_vertex = 2;
        let mut geometry = Geometry::new(GL_TRIANGLES);
        geometry.add_buffer(&vertices, values_per_vertex);
        geometry.add_vertex_attribute(Attribute::new(0, 2, values_per_vertex as usize, 0));
        geometry
    }

    pub fn image_geometry(width: f32, height: f32, ox: f32, oy: f32) -> Geometry {
        // Vertex format: [x, y, u, v]. Geometry built centered at origin,
        // then shifted by (-ox, -oy) so the resolved anchor sits at local (0, 0).
        let hw = width / 2.0;
        let hh = height / 2.0;
        let vertices: Vec<f32> = vec![
            // Triangle 1
            -hw - ox, -hh - oy, 0.0, 0.0, // bottom-left
             hw - ox, -hh - oy, 1.0, 0.0, // bottom-right
             hw - ox,  hh - oy, 1.0, 1.0, // top-right
            // Triangle 2
            -hw - ox, -hh - oy, 0.0, 0.0, // bottom-left
             hw - ox,  hh - oy, 1.0, 1.0, // top-right
            -hw - ox,  hh - oy, 0.0, 1.0, // top-left
        ];

        let values_per_vertex = 4;

        let mut geometry = Geometry::new(GL_TRIANGLES);
        geometry.add_buffer(&vertices, values_per_vertex);
        geometry.add_vertex_attribute(Attribute::new(0, 2, values_per_vertex as usize, 0));
        geometry.add_vertex_attribute(Attribute::new(1, 2, values_per_vertex as usize, 2));

        geometry
    }

    /// Build raw textured-quad vertices for a string of text and compute the
    /// bounding box over all glyph quads. Returns `(vertices, bbox_min, bbox_max)`.
    /// Local origin is the top-left of the text cell (cursor_x = 0, y = 0).
    fn text_raw_vertices(
        text: &str,
        font_atlas: &mut FontAtlas,
    ) -> (Vec<f32>, (f32, f32), (f32, f32)) {
        let mut vertices: Vec<f32> = Vec::new();
        let mut cursor_x: f32 = 0.0;
        let baseline_y: f32 = font_atlas.font_size() as f32;

        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut max_y = f32::NEG_INFINITY;

        for ch in text.chars() {
            if let Some(glyph) = font_atlas.get_glyph(ch) {
                if glyph.width == 0 || glyph.height == 0 {
                    cursor_x += glyph.advance;
                    continue;
                }

                let x0 = cursor_x + glyph.bearing_x as f32;
                let y0 = baseline_y - glyph.bearing_y as f32;
                let x1 = x0 + glyph.width as f32;
                let y1 = y0 + glyph.height as f32;

                if x0 < min_x { min_x = x0; }
                if y0 < min_y { min_y = y0; }
                if x1 > max_x { max_x = x1; }
                if y1 > max_y { max_y = y1; }

                let u0 = glyph.uv_x;
                let v0 = glyph.uv_y;
                let u1 = glyph.uv_x + glyph.uv_width;
                let v1 = glyph.uv_y + glyph.uv_height;

                // Triangle 1: bottom-left, bottom-right, top-right
                vertices.extend_from_slice(&[
                    x0, y1, u0, v1,
                    x1, y1, u1, v1,
                    x1, y0, u1, v0,
                ]);
                // Triangle 2: bottom-left, top-right, top-left
                vertices.extend_from_slice(&[
                    x0, y1, u0, v1,
                    x1, y0, u1, v0,
                    x0, y0, u0, v0,
                ]);

                cursor_x += glyph.advance;
            }
        }

        let (bbox_min, bbox_max) = if min_x.is_finite() {
            ((min_x, min_y), (max_x, max_y))
        } else {
            ((0.0, 0.0), (0.0, 0.0))
        };

        (vertices, bbox_min, bbox_max)
    }

}

/// Builder for `ShapeRenderable` that lets callers override the anchor point
/// before the mesh is created. Obtained via [`ShapeRenderable::builder`].
///
/// ```ignore
/// let rect = ShapeRenderable::builder(
///     ShapeKind::Rectangle(Rectangle::new(6.0, 6.0)),
///     ShapeStyle::fill(Color::white()),
/// )
/// .anchor(Anchor::Center)
/// .build();
/// ```
pub struct ShapeRenderableBuilder {
    shape: ShapeKind,
    style: ShapeStyle,
    anchor: Anchor,
}

impl ShapeRenderableBuilder {
    /// Set the anchor for the shape. Defaults to `Anchor::Default` if not called.
    pub fn anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = anchor;
        self
    }

    /// Build the `ShapeRenderable`, consuming the builder.
    pub fn build(self) -> ShapeRenderable {
        ShapeRenderable::from_shape_with_anchor(self.shape, self.style, self.anchor)
    }
}
