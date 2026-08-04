//! Browser port of the `text` example. The font arrives over the network:
//! the page fetches DejaVuSans.ttf (window.WILHELM_ASSETS), hands the bytes
//! to `wasm_asset_loaded`, and `wasm_init` registers them under the name
//! "DejaVuSans" — which the `Text` shapes then use as their `font_path`.
//! Glyph rasterization is the same pure-Rust code as native.

use std::cell::RefCell;

use wilhelm_renderer::core::{App, Color, Window};
use wilhelm_renderer::graphics2d::shapes::{
    register_font, ShapeKind, ShapeRenderable, ShapeStyle, Text,
};

thread_local! {
    static APP: RefCell<Option<App<'static>>> = RefCell::new(None);
    /// Bytes fetched by the page, indexed by position in window.WILHELM_ASSETS.
    static ASSETS: RefCell<Vec<Vec<u8>>> = RefCell::new(Vec::new());
}

const ASSET_FONT: usize = 0;
const FONT: &str = "DejaVuSans";

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

fn build_app() -> App<'static> {
    let window = Window::new_fullscreen("Text Rendering", Color::from_rgb(0.07, 0.13, 0.17));
    let mut app = App::new(window);

    ASSETS.with(|assets| {
        let assets = assets.borrow();
        if let Some(font_bytes) = assets.get(ASSET_FONT) {
            register_font(FONT, font_bytes.clone());
        }
    });

    let text = |pos: (f32, f32), kind: ShapeKind, style: ShapeStyle| -> ShapeRenderable {
        let mut s = ShapeRenderable::from_shape(kind, style);
        s.set_position(pos.0, pos.1);
        s
    };

    app.add_shapes(vec![
        // Create text with white color
        text(
            (100.0, 100.0),
            ShapeKind::Text(Text::new("Hello, World!", FONT, 48)),
            ShapeStyle { fill: Some(Color::white()), ..Default::default() },
        ),
        // Red text
        text(
            (100.0, 200.0),
            ShapeKind::Text(Text::new("Red Text", FONT, 36)),
            ShapeStyle { fill: Some(Color::from_rgb(1.0, 0.0, 0.0)), ..Default::default() },
        ),
        // Green text
        text(
            (100.0, 280.0),
            ShapeKind::Text(Text::new("Green Text", FONT, 36)),
            ShapeStyle { fill: Some(Color::from_rgb(0.0, 1.0, 0.0)), ..Default::default() },
        ),
        // Blue text
        text(
            (100.0, 360.0),
            ShapeKind::Text(Text::new("Blue Text", FONT, 36)),
            ShapeStyle { fill: Some(Color::from_rgb(0.0, 0.0, 1.0)), ..Default::default() },
        ),
        // Smaller text
        text(
            (100.0, 450.0),
            ShapeKind::Text(Text::new(
                "The quick brown fox jumps over the lazy dog",
                FONT,
                24,
            )),
            ShapeStyle { fill: Some(Color::from_rgb(0.8, 0.8, 0.8)), ..Default::default() },
        ),
    ]);

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
