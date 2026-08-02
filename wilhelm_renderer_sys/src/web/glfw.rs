//! Canvas/DOM implementations of the `_glfw*` contract symbols.
//!
//! A single "window" is the page's canvas element. Creation sets the canvas
//! size and acquires the WebGL2 context; swap/poll are no-ops because the
//! browser presents at requestAnimationFrame boundaries and pushes events.
//! Input callbacks are stored for the glue to dispatch from DOM listeners
//! (not wired in the spike — the shapes demo takes no input).

#![allow(non_snake_case)]

use std::ffi::{c_char, c_double, c_float, c_int, c_void};
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::glfw::{
    GLFW_PLATFORM_NULL, GLFWcursorposfun, GLFWframebuffersizefun, GLFWkeyfun, GLFWmousebuttonfun,
    GLFWscrollfun, GLFWwindow, GLFWwindowsizefun,
};

use super::*;

/// The one canvas-backed window. Any non-null pointer works; the upper
/// crate treats it as opaque and threads it back into these functions.
const CANVAS_WINDOW: *const GLFWwindow = 1 as *const GLFWwindow;

static USER_POINTER: AtomicUsize = AtomicUsize::new(0);

// Stored callback fn-pointers (as usize; 0 = unset) for future DOM wiring.
static CB_FRAMEBUFFER_SIZE: AtomicUsize = AtomicUsize::new(0);
static CB_WINDOW_SIZE: AtomicUsize = AtomicUsize::new(0);
static CB_SCROLL: AtomicUsize = AtomicUsize::new(0);
static CB_CURSOR_POS: AtomicUsize = AtomicUsize::new(0);
static CB_KEY: AtomicUsize = AtomicUsize::new(0);
static CB_MOUSE_BUTTON: AtomicUsize = AtomicUsize::new(0);

fn store_cb<T>(slot: &AtomicUsize, callback: Option<T>) {
    let raw = callback
        .map(|f| {
            // fn pointers are non-null; store as usize
            let p: usize = unsafe { std::mem::transmute_copy(&f) };
            p
        })
        .unwrap_or(0);
    slot.store(raw, Ordering::Relaxed);
}

pub unsafe fn _glfwCreateWindow(
    _title: *const c_char,
    width: c_int,
    height: c_int,
    callback: GLFWframebuffersizefun,
) -> *const GLFWwindow {
    unsafe { js_setup_canvas(width, height) }
    store_cb(&CB_FRAMEBUFFER_SIZE, callback);
    CANVAS_WINDOW
}

pub unsafe fn _glfwCreateFullscreenWindow(
    _title: *const c_char,
    out_width: *mut c_int,
    out_height: *mut c_int,
    callback: GLFWframebuffersizefun,
) -> *const GLFWwindow {
    // The canvas is the "monitor": use its current size.
    let (w, h) = unsafe { (js_canvas_width(), js_canvas_height()) };
    unsafe {
        js_setup_canvas(w, h);
        *out_width = w;
        *out_height = h;
    }
    store_cb(&CB_FRAMEBUFFER_SIZE, callback);
    CANVAS_WINDOW
}

pub unsafe fn _glfwGetWindowContentScale(
    _window: *const GLFWwindow,
    xscale: *mut c_float,
    yscale: *mut c_float,
) {
    // Spike: 1.0. Production: devicePixelRatio via the glue.
    unsafe {
        *xscale = 1.0;
        *yscale = 1.0;
    }
}

pub unsafe fn _glfwWindowHint(_hint: c_int, _value: c_int) {
    // Context/window hints are context-creation attributes in the browser
    // (e.g. MSAA -> { antialias: true }, handled by the glue). No-op.
}

pub unsafe fn _glfwSetWindowUserPointer(_window: *const GLFWwindow, pointer: *const c_void) {
    USER_POINTER.store(pointer as usize, Ordering::Relaxed);
}

pub unsafe fn _glfwGetWindowUserPointer(_window: *const GLFWwindow) -> *const c_void {
    USER_POINTER.load(Ordering::Relaxed) as *const c_void
}

pub unsafe fn _glfwWindowShouldClose(_window: *const GLFWwindow) -> c_int {
    0
}

pub unsafe fn _glfwDestroyWindow(_window: *const GLFWwindow) {}

pub unsafe fn _glfwSwapBuffers(_window: *const GLFWwindow) {
    // The browser presents the canvas at rAF boundaries.
}

pub unsafe fn _glfwPollEvents() {
    // DOM events are pushed by the browser; nothing to poll.
}

pub unsafe fn _glfwTerminate() {}

pub unsafe fn _glfwGetTime() -> c_double {
    unsafe { js_now() }
}

pub unsafe fn _glfwSetFramebufferSizeCallback(
    _window: *const GLFWwindow,
    callback: GLFWframebuffersizefun,
) {
    store_cb(&CB_FRAMEBUFFER_SIZE, callback);
}

pub unsafe fn _glfwSetWindowSizeCallback(_window: *const GLFWwindow, callback: GLFWwindowsizefun) {
    store_cb(&CB_WINDOW_SIZE, callback);
}

pub unsafe fn _glfwSetScrollCallback(_window: *const GLFWwindow, callback: GLFWscrollfun) {
    store_cb(&CB_SCROLL, callback);
}

pub unsafe fn _glfwSetCursorPosCallback(_window: *const GLFWwindow, callback: GLFWcursorposfun) {
    store_cb(&CB_CURSOR_POS, callback);
}

pub unsafe fn _glfwSetKeyCallback(_window: *const GLFWwindow, callback: GLFWkeyfun) {
    store_cb(&CB_KEY, callback);
}

pub unsafe fn _glfwSetMouseButtonCallback(
    _window: *const GLFWwindow,
    callback: GLFWmousebuttonfun,
) {
    store_cb(&CB_MOUSE_BUTTON, callback);
}

pub unsafe fn _glfwGetWindowSize(
    _window: *const GLFWwindow,
    width: *mut c_int,
    height: *mut c_int,
) {
    unsafe {
        *width = js_canvas_width();
        *height = js_canvas_height();
    }
}

pub unsafe fn _glfwGetPlatform() -> c_int {
    GLFW_PLATFORM_NULL
}

/// Called by the JS glue when the canvas is resized (e.g. the browser
/// window changed size). Dispatches the stored GLFW-style callbacks in the
/// same order native GLFW does: framebuffer size first (drives
/// `glViewport`), then window size (updates `Window` state and fires the
/// user's `on_resize`). The callbacks are the upper crate's existing
/// `extern "C"` trampolines — no upper-crate changes needed.
#[no_mangle]
pub extern "C" fn wilhelm_dispatch_resize(width: i32, height: i32) {
    type SizeFn = extern "C" fn(*const GLFWwindow, i32, i32);

    for slot in [&CB_FRAMEBUFFER_SIZE, &CB_WINDOW_SIZE] {
        let raw = slot.load(Ordering::Relaxed);
        if raw != 0 {
            let f: SizeFn = unsafe { std::mem::transmute(raw) };
            f(CANVAS_WINDOW, width, height);
        }
    }
}
