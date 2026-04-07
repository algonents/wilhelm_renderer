//! Safe Rust wrappers around the raw GLFW FFI exposed by
//! `wilhelm_renderer_sys`.
//!
//! The raw `extern "C"` functions are imported privately via the `sys`
//! alias and are **not** re-exported — client code cannot reach them
//! through this module.

use std::ffi::CString;
use std::ffi::c_void;
use std::os::raw::c_int;

// Re-export the public GLFW types and constants as part of our API.
pub use wilhelm_renderer_sys::glfw::{
    GLFW_KEY_BACKSPACE, GLFW_KEY_DELETE, GLFW_KEY_DOWN, GLFW_KEY_END, GLFW_KEY_ENTER,
    GLFW_KEY_ESCAPE, GLFW_KEY_F1, GLFW_KEY_F2, GLFW_KEY_F3, GLFW_KEY_F4, GLFW_KEY_F5, GLFW_KEY_F6,
    GLFW_KEY_F7, GLFW_KEY_F8, GLFW_KEY_F9, GLFW_KEY_F10, GLFW_KEY_F11, GLFW_KEY_F12, GLFW_KEY_HOME,
    GLFW_KEY_INSERT, GLFW_KEY_LEFT, GLFW_KEY_LEFT_ALT, GLFW_KEY_LEFT_CONTROL, GLFW_KEY_LEFT_SHIFT,
    GLFW_KEY_LEFT_SUPER, GLFW_KEY_PAGE_DOWN, GLFW_KEY_PAGE_UP, GLFW_KEY_RIGHT, GLFW_KEY_RIGHT_ALT,
    GLFW_KEY_RIGHT_CONTROL, GLFW_KEY_RIGHT_SHIFT, GLFW_KEY_RIGHT_SUPER, GLFW_KEY_SPACE,
    GLFW_KEY_TAB, GLFW_KEY_UP, GLFW_MOD_ALT, GLFW_MOD_CAPS_LOCK, GLFW_MOD_CONTROL,
    GLFW_MOD_NUM_LOCK, GLFW_MOD_SHIFT, GLFW_MOD_SUPER, GLFW_MOUSE_BUTTON_LEFT,
    GLFW_MOUSE_BUTTON_MIDDLE, GLFW_MOUSE_BUTTON_RIGHT, GLFW_PLATFORM_COCOA, GLFW_PLATFORM_NULL,
    GLFW_PLATFORM_WAYLAND, GLFW_PLATFORM_WIN32, GLFW_PLATFORM_X11, GLFW_PRESS, GLFW_RELEASE,
    GLFW_REPEAT, GLFW_SAMPLES, GLFW_SCALE_TO_MONITOR, GLFWcursorposfun, GLFWframebuffersizefun,
    GLFWkeyfun, GLFWmousebuttonfun, GLFWscrollfun, GLFWwindow, GLFWwindowsizefun,
};

// Private alias for the raw FFI. Not re-exported.
use wilhelm_renderer_sys::glfw as sys;

pub fn glfw_get_time() -> f64 {
    unsafe { sys::_glfwGetTime() }
}

pub fn glfw_create_window(
    title: &str,
    width: i32,
    height: i32,
    callback: GLFWframebuffersizefun,
) -> *const GLFWwindow {
    let window_pointer: *const GLFWwindow;
    let title_c_string = CString::new(title).expect("Failed to create title");
    unsafe {
        window_pointer = sys::_glfwCreateWindow(title_c_string.as_ptr(), width, height, callback);
    }
    window_pointer
}

pub fn glfw_get_window_content_scale(window: *const GLFWwindow) -> (f32, f32) {
    unsafe {
        let mut xs: f32 = 0.0;
        let mut ys: f32 = 0.0;
        sys::_glfwGetWindowContentScale(window, &mut xs, &mut ys);
        (xs, ys)
    }
}

pub fn glfw_window_hint(hint: i32, value: i32) {
    unsafe {
        sys::_glfwWindowHint(hint, value);
    }
}

pub fn glfw_set_window_user_pointer(window: *const GLFWwindow, pointer: *mut c_void) {
    unsafe {
        sys::_glfwSetWindowUserPointer(window, pointer);
    }
}

pub fn glfw_get_window_user_pointer(window: *const GLFWwindow) -> *const c_void {
    unsafe { sys::_glfwGetWindowUserPointer(window) }
}

pub fn glfw_window_should_close(window: *const GLFWwindow) -> bool {
    let result: i32;
    unsafe {
        result = sys::_glfwWindowShouldClose(window);
    }
    result != 0
}

pub fn glfw_set_scroll_callback(window: *const GLFWwindow, callback: GLFWscrollfun) {
    unsafe {
        sys::_glfwSetScrollCallback(window, callback);
    }
}

pub fn glfw_set_cursor_pos_callback(window: *const GLFWwindow, callback: GLFWcursorposfun) {
    unsafe {
        sys::_glfwSetCursorPosCallback(window, callback);
    }
}

pub fn glfw_set_key_callback(window: *const GLFWwindow, callback: GLFWkeyfun) {
    unsafe {
        sys::_glfwSetKeyCallback(window, callback);
    }
}

pub fn glfw_set_mouse_button_callback(window: *const GLFWwindow, callback: GLFWmousebuttonfun) {
    unsafe {
        sys::_glfwSetMouseButtonCallback(window, callback);
    }
}

pub fn glfw_set_window_size_callback(window: *const GLFWwindow, callback: GLFWwindowsizefun) {
    unsafe {
        sys::_glfwSetWindowSizeCallback(window, callback);
    }
}

pub fn glfw_get_window_size(window: *const GLFWwindow, width: *mut c_int, height: *mut c_int) {
    unsafe {
        sys::_glfwGetWindowSize(window, width, height);
    }
}

pub fn glfw_poll_events() {
    unsafe { sys::_glfwPollEvents() }
}

pub fn glfw_swap_buffers(window: *const GLFWwindow) {
    unsafe { sys::_glfwSwapBuffers(window) }
}

pub fn glfw_destroy_window(window: *const GLFWwindow) {
    unsafe { sys::_glfwDestroyWindow(window) }
}

pub fn glfw_terminate() {
    unsafe { sys::_glfwTerminate() };
}

/// Returns the currently selected platform.
///
/// Returns one of: `GLFW_PLATFORM_WIN32`, `GLFW_PLATFORM_COCOA`,
/// `GLFW_PLATFORM_WAYLAND`, `GLFW_PLATFORM_X11`, or `GLFW_PLATFORM_NULL`.
///
/// Must be called after GLFW initialization (window creation).
pub fn glfw_get_platform() -> i32 {
    unsafe { sys::_glfwGetPlatform() }
}
