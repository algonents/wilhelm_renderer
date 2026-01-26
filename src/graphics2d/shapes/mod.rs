mod shaperenderable;

pub use shaperenderable::ShapeRenderable;
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
}


#[derive(Clone, Copy)]
pub struct Line {
    pub x2: f32,
    pub y2: f32,
}

impl Line {
    pub fn new(x2: f32, y2: f32) -> Self {
        Self { x2, y2 }
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
}
#[derive(Clone, Copy)]
pub struct Triangle {
    pub vertices: [(f32, f32); 3],
}

impl Triangle {
    pub fn new(vertices: [(f32, f32); 3]) -> Self {
        Self { vertices }
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