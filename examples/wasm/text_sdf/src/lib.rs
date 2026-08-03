//! SDF vs bitmap text under live zoom, in the browser.
//!
//! Scroll to zoom both lines: the bitmap line blurs as it magnifies, the
//! SDF line stays sharp at every scale — the same 48px atlas serves the
//! whole zoom range. The font arrives over the network (WILHELM_ASSETS)
//! and is registered by name; both atlases build from the same bytes.

use std::cell::{Cell, RefCell};

use wilhelm_renderer::core::{App, Color, Window};
use wilhelm_renderer::graphics2d::shapes::{
    register_font, ShapeKind, ShapeRenderable, ShapeStyle, Text,
};

thread_local! {
    static APP: RefCell<Option<App<'static>>> = RefCell::new(None);
    static ASSETS: RefCell<Vec<Vec<u8>>> = RefCell::new(Vec::new());
    static SCALE_LEVEL: Cell<f32> = Cell::new(1.0);
}

const ASSET_FONT: usize = 0;
const FONT: &str = "DejaVuSans";
const FONT_SIZE: u32 = 48;
const SAMPLE: &str = "FL350 KLM1874";

#[no_mangle]
pub extern "C" fn wasm_alloc(len: usize) -> *mut u8 {
    let mut buf = vec![0u8; len];
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

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
    let mut window = Window::new_fullscreen("SDF text zoom", Color::from_rgb(0.07, 0.13, 0.17));

    window.on_scroll(move |_, y_offset| {
        let scale_step = 1.1;
        let scale_factor = if y_offset > 0.0 {
            scale_step
        } else {
            1.0 / scale_step
        };
        SCALE_LEVEL.with(|s| {
            let new_scale = (s.get() * scale_factor).clamp(0.1, 20.0);
            s.set(new_scale);
        });
    });

    let mut app = App::new(window);

    ASSETS.with(|assets| {
        let assets = assets.borrow();
        if let Some(font_bytes) = assets.get(ASSET_FONT) {
            register_font(FONT, font_bytes.clone());
        }
    });

    let bitmap_text = |pos: (f32, f32), content: &str, size: u32| -> ShapeRenderable {
        let mut s = ShapeRenderable::from_shape(
            ShapeKind::Text(Text::new(content, FONT, size)),
            ShapeStyle {
                fill: Some(Color::white()),
                ..Default::default()
            },
        );
        s.set_position(pos.0, pos.1);
        s
    };

    let mut sdf = ShapeRenderable::text_sdf(SAMPLE, FONT, FONT_SIZE, Color::white());
    sdf.set_position(60.0, 320.0);

    let mut bitmap = bitmap_text((60.0, 120.0), SAMPLE, FONT_SIZE);
    bitmap.set_position(60.0, 120.0);

    app.add_shapes(vec![bitmap, sdf]);

    // Scroll-to-zoom: rescale both lines every frame from the shared level.
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
