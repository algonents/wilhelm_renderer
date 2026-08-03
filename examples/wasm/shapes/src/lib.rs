//! Browser port of the `shapes` example: geometry and images (no text or
//! points — see docs/DESIGN_WASM.md, spike plan). Images arrive over the
//! network: the page's glue fetches the URLs listed in
//! `window.WILHELM_ASSETS`, copies each response into wasm memory via
//! `wasm_alloc`, and hands it to `wasm_asset_loaded` before calling
//! `wasm_init` — where the scene is built from `ImageSource::Bytes`.
//!
//! The page's JS glue instantiates the wasm module, calls `wasm_init` once,
//! then calls `wasm_frame` from every requestAnimationFrame tick. Scene and
//! API usage are identical to the native example.

use std::cell::RefCell;

use wilhelm_renderer::core::{App, Color, Window};
use wilhelm_renderer::graphics2d::shapes::{
    register_font, Arc, Circle, Ellipse, Line, Polygon, Polyline, Rectangle, RoundedRectangle,
    ShapeKind, ShapeRenderable, ShapeStyle, Text, Triangle,
};

thread_local! {
    static APP: RefCell<Option<App<'static>>> = RefCell::new(None);
    /// Bytes fetched by the page, indexed by position in window.WILHELM_ASSETS.
    static ASSETS: RefCell<Vec<Vec<u8>>> = RefCell::new(Vec::new());
}

const ASSET_SMILEY: usize = 0;
const ASSET_BUNNY: usize = 1;
const ASSET_FONT: usize = 2;

/// Called by the glue to reserve space for one fetched asset.
#[no_mangle]
pub extern "C" fn wasm_alloc(len: usize) -> *mut u8 {
    let mut buf = vec![0u8; len];
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

/// Called by the glue once it has copied a fetched asset into the buffer
/// returned by `wasm_alloc`.
#[no_mangle]
pub extern "C" fn wasm_asset_loaded(index: usize, ptr: *mut u8, len: usize) {
    let bytes = unsafe { Vec::from_raw_parts(ptr, len, len) };
    ASSETS.with(|assets| {
        let mut assets = assets.borrow_mut();
        if assets.len() <= index {
            assets.resize(index + 1, Vec::new());
        }
        assets[index] = bytes;
    });
}

fn create_equilateral_triangle() -> [(f32, f32); 3] {
    let side = 20.0;
    let height = (3.0f32).sqrt() / 2.0 * side;

    [
        (0.0, 2.0 * -height / 3.0),  // Top vertex
        (-0.5 * side, height / 3.0), // Bottom left
        (0.5 * side, height / 3.0),  // Bottom right
    ]
}

fn build_app() -> App<'static> {
    // Fullscreen: the canvas is the "monitor" — the backend sizes the GL
    // context to the canvas, which the glue keeps matched to the browser
    // window (including live resize via wilhelm_dispatch_resize).
    let window = Window::new_fullscreen("Shapes", Color::from_rgb(0.07, 0.13, 0.17));
    let mut app = App::new(window);

    let polyline_points = vec![(0.0, 0.0), (50.0, 130.0), (100.0, 110.0), (100.0, 200.0)];

    let polygon_local: Vec<(f32, f32)> = vec![
        (0.0, 0.0), // anchor vertex
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

    app.add_shapes(vec![
        // Line from (100, 200) to (300, 250)
        shape(
            (100.0, 200.0),
            ShapeKind::Line(Line::new((0.0, 0.0), (200.0, 50.0))),
            ShapeStyle::stroke(Color::from_rgb(0.0, 1.0, 0.0), 5.0),
        ),
        // Polyline starting at (100, 300)
        shape(
            (100.0, 300.0),
            ShapeKind::Polyline(Polyline::new(polyline_points)),
            ShapeStyle::stroke(Color::from_rgb(1.0, 0.0, 0.0), 10.0),
        ),
        // Arc centered at (700, 600)
        shape(
            (700.0, 600.0),
            ShapeKind::Arc(Arc::new(70.0, 0.0, std::f32::consts::PI / 2.0)),
            ShapeStyle::stroke(Color::from_rgb(0.0, 0.0, 1.0), 10.0),
        ),
        // Rectangle at (50, 50)
        shape(
            (50.0, 50.0),
            ShapeKind::Rectangle(Rectangle::new(200.0, 80.0)),
            ShapeStyle::fill(Color::from_rgb(0.2, 0.5, 0.9)),
        ),
        // Triangle at (50, 50)
        shape(
            (50.0, 50.0),
            ShapeKind::Triangle(Triangle::new(create_equilateral_triangle())),
            ShapeStyle::fill(Color::from_rgb(1.0, 0.0, 0.0)),
        ),
        // Rectangle at (400, 200)
        shape(
            (400.0, 200.0),
            ShapeKind::Rectangle(Rectangle::new(100.0, 50.0)),
            ShapeStyle::fill(Color::from_rgb(1.0, 0.0, 0.0)),
        ),
        // Circle at (400, 400)
        shape(
            (400.0, 400.0),
            ShapeKind::Circle(Circle::new(50.0)),
            ShapeStyle::fill(Color::from_rgb(0.0, 0.0, 1.0)),
        ),
        // Ellipse at (600, 200)
        shape(
            (600.0, 200.0),
            ShapeKind::Ellipse(Ellipse::new(80.0, 40.0)),
            ShapeStyle::fill(Color::from_rgb(0.5, 0.2, 0.8)),
        ),
        // Rounded rectangle at (100, 600)
        shape(
            (100.0, 600.0),
            ShapeKind::RoundedRectangle(RoundedRectangle::new(200.0, 80.0, 10.0)),
            ShapeStyle::fill(Color::from_rgb(0.3, 0.6, 0.9)),
        ),
        // Polygon (hexagon)
        shape(
            (600.0, 600.0),
            ShapeKind::Polygon(Polygon::new(polygon_local)),
            ShapeStyle::fill(Color::from_rgb(1.0, 0.0, 0.0)),
        ),
        // Rectangle at (600, 400)
        shape(
            (600.0, 400.0),
            ShapeKind::Rectangle(Rectangle::new(100.0, 50.0)),
            ShapeStyle::fill(Color::from_rgb(0.0, 1.0, 0.0)),
        ),
        // Outlined rectangle at (270, 50)
        shape(
            (270.0, 50.0),
            ShapeKind::Rectangle(Rectangle::new(200.0, 80.0)),
            ShapeStyle::stroke(Color::from_rgb(0.2, 0.5, 0.9), 3.0),
        ),
        // Fill and stroke rectangle at (490, 50)
        shape(
            (490.0, 50.0),
            ShapeKind::Rectangle(Rectangle::new(200.0, 80.0)),
            ShapeStyle::fill_and_stroke(
                Color::from_rgb(0.2, 0.5, 0.9),
                Color::from_rgb(1.0, 1.0, 0.0),
                3.0,
            ),
        ),
    ]);

    // Images (network bytes, same positions as the native example). The
    // bytes came over the wire, so decode failures are handled instead of
    // panicking — on wasm a panic aborts the whole module.
    ASSETS.with(|assets| {
        let assets = assets.borrow();

        let image_sized = |idx: usize, w: f32, h: f32| -> Option<ShapeRenderable> {
            let bytes = assets.get(idx)?;
            ShapeRenderable::try_image_with_size(bytes.as_slice(), w, h).ok()
        };
        let image = |idx: usize| -> Option<ShapeRenderable> {
            let bytes = assets.get(idx)?;
            ShapeRenderable::try_image(bytes.as_slice()).ok()
        };

        let mut images = Vec::new();
        if let Some(mut s) = image_sized(ASSET_SMILEY, 40.0, 40.0) {
            s.set_position(200.0, 540.0);
            images.push(s);
        }
        if let Some(mut s) = image(ASSET_BUNNY) {
            s.set_position(400.0, 500.0);
            images.push(s);
        }

        // Text: register the fetched font bytes under a name, then reference
        // that name as the Text "font_path" — no filesystem involved.
        if let Some(font_bytes) = assets.get(ASSET_FONT) {
            register_font("DejaVuSans", font_bytes.clone());
            let mut s = ShapeRenderable::from_shape(
                ShapeKind::Text(Text::new("Hello from the browser!", "DejaVuSans", 48)),
                ShapeStyle::fill(Color::from_rgb(1.0, 1.0, 1.0)),
            );
            s.set_position(750.0, 80.0);
            images.push(s);
        }
        app.add_shapes(images);
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
