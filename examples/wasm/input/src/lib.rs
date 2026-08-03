//! Browser input demo: exercises the three input dispatchers added after
//! resize/scroll — cursor position, mouse buttons, and keys.
//!
//! - The circle follows the cursor (`on_cursor_position`).
//! - Holding a mouse button recolors it: left = red, right = blue,
//!   middle = yellow (`on_mouse_button`).
//! - Arrow keys move the square, Space recenters it (`on_key`;
//!   hold an arrow to see GLFW_REPEAT).
//!
//! No assets, no WILHELM_ASSETS — pure input plumbing.

use std::cell::{Cell, RefCell};

use wilhelm_renderer::core::engine::glfw::{
    GLFW_KEY_DOWN, GLFW_KEY_LEFT, GLFW_KEY_RIGHT, GLFW_KEY_SPACE, GLFW_KEY_UP,
    GLFW_MOUSE_BUTTON_LEFT, GLFW_MOUSE_BUTTON_MIDDLE, GLFW_MOUSE_BUTTON_RIGHT, GLFW_PRESS,
    GLFW_RELEASE,
};
use wilhelm_renderer::core::{App, Color, Window};
use wilhelm_renderer::graphics2d::shapes::{
    Circle, Rectangle, ShapeKind, ShapeRenderable, ShapeStyle,
};

thread_local! {
    static APP: RefCell<Option<App<'static>>> = RefCell::new(None);
    static CURSOR: Cell<(f64, f64)> = Cell::new((200.0, 200.0));
    static HELD_BUTTON: Cell<i32> = Cell::new(-1);
    static SQUARE: Cell<(f32, f32)> = Cell::new((400.0, 300.0));
}

const SQUARE_STEP: f32 = 20.0;
const SQUARE_HOME: (f32, f32) = (400.0, 300.0);

fn build_app() -> App<'static> {
    let mut window = Window::new_fullscreen("Input demo", Color::from_rgb(0.07, 0.13, 0.17));

    window.on_cursor_position(|x, y| {
        CURSOR.with(|c| c.set((x, y)));
    });

    window.on_mouse_button(|button, action, _mods| {
        HELD_BUTTON.with(|held| match action {
            GLFW_PRESS => held.set(button),
            GLFW_RELEASE if held.get() == button => held.set(-1),
            _ => {}
        });
    });

    // Press and repeat both move; hold an arrow to glide.
    window.on_key(|key, _scancode, action, _mods| {
        if action == GLFW_RELEASE {
            return;
        }
        SQUARE.with(|s| {
            let (x, y) = s.get();
            match key {
                GLFW_KEY_LEFT => s.set((x - SQUARE_STEP, y)),
                GLFW_KEY_RIGHT => s.set((x + SQUARE_STEP, y)),
                GLFW_KEY_UP => s.set((x, y - SQUARE_STEP)),
                GLFW_KEY_DOWN => s.set((x, y + SQUARE_STEP)),
                GLFW_KEY_SPACE => s.set(SQUARE_HOME),
                _ => {}
            }
        });
    });

    let mut app = App::new(window);

    let mut circle = ShapeRenderable::from_shape(
        ShapeKind::Circle(Circle::new(30.0)),
        ShapeStyle {
            fill: Some(Color::from_rgb(0.3, 0.9, 0.4)),
            ..Default::default()
        },
    );
    circle.set_position(200.0, 200.0);

    let mut square = ShapeRenderable::from_shape(
        ShapeKind::Rectangle(Rectangle::new(80.0, 80.0)),
        ShapeStyle {
            fill: Some(Color::from_rgb(0.2, 0.5, 0.9)),
            ..Default::default()
        },
    );
    square.set_position(SQUARE_HOME.0, SQUARE_HOME.1);

    app.add_shapes(vec![circle, square]);

    app.on_pre_render(move |shapes, _renderer| {
        let (cx, cy) = CURSOR.with(|c| c.get());
        let color = match HELD_BUTTON.with(|h| h.get()) {
            GLFW_MOUSE_BUTTON_LEFT => Color::from_rgb(0.9, 0.2, 0.2),
            GLFW_MOUSE_BUTTON_RIGHT => Color::from_rgb(0.2, 0.4, 0.9),
            GLFW_MOUSE_BUTTON_MIDDLE => Color::from_rgb(0.9, 0.9, 0.2),
            _ => Color::from_rgb(0.3, 0.9, 0.4),
        };
        shapes[0].set_position(cx as f32, cy as f32);
        shapes[0].set_fill_color(color);

        let (sx, sy) = SQUARE.with(|s| s.get());
        shapes[1].set_position(sx, sy);
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
