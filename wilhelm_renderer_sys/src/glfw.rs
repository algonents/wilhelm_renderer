//! Raw GLFW FFI bindings.
//!
//! All extern declarations, types, and constants used by `wilhelm_renderer`.
//! Safe Rust wrappers live in the upper crate.

use std::ffi::c_float;
use std::ffi::c_void;
use std::os::raw::c_char;
use std::os::raw::c_double;
use std::os::raw::c_int;

pub const GLFW_SAMPLES: i32 = 0x0002100D;
pub const GLFW_SCALE_TO_MONITOR: i32 = 0x0002200C;

pub const GLFW_PLATFORM_WIN32: i32 = 0x00060001;
pub const GLFW_PLATFORM_COCOA: i32 = 0x00060002;
pub const GLFW_PLATFORM_WAYLAND: i32 = 0x00060003;
pub const GLFW_PLATFORM_X11: i32 = 0x00060004;
pub const GLFW_PLATFORM_NULL: i32 = 0x00060005;

pub enum GLFWwindow {}

pub type GLFWframebuffersizefun =
    Option<extern "C" fn(window: *const GLFWwindow, width: i32, height: i32)>;

pub type GLFWwindowsizefun =
    Option<extern "C" fn(window: *const GLFWwindow, width: i32, height: i32)>;

pub type GLFWscrollfun =
    Option<extern "C" fn(window: *const GLFWwindow, xoffset: f64, yoffset: f64)>;

pub type GLFWcursorposfun =
    Option<extern "C" fn(window: *const GLFWwindow, xpos: f64, ypos: f64)>;

pub type GLFWkeyfun = Option<
    extern "C" fn(window: *const GLFWwindow, key: i32, scancode: i32, action: i32, mods: i32),
>;

pub type GLFWmousebuttonfun =
    Option<extern "C" fn(window: *const GLFWwindow, button: i32, action: i32, mods: i32)>;

// Mouse buttons
pub const GLFW_MOUSE_BUTTON_LEFT: i32 = 0;
pub const GLFW_MOUSE_BUTTON_RIGHT: i32 = 1;
pub const GLFW_MOUSE_BUTTON_MIDDLE: i32 = 2;

// Key actions
pub const GLFW_RELEASE: i32 = 0;
pub const GLFW_PRESS: i32 = 1;
pub const GLFW_REPEAT: i32 = 2;

// Modifier flags (bitfield)
pub const GLFW_MOD_SHIFT: i32 = 0x0001;
pub const GLFW_MOD_CONTROL: i32 = 0x0002;
pub const GLFW_MOD_ALT: i32 = 0x0004;
pub const GLFW_MOD_SUPER: i32 = 0x0008;
pub const GLFW_MOD_CAPS_LOCK: i32 = 0x0010;
pub const GLFW_MOD_NUM_LOCK: i32 = 0x0020;

// Common keys
pub const GLFW_KEY_SPACE: i32 = 32;
pub const GLFW_KEY_ESCAPE: i32 = 256;
pub const GLFW_KEY_ENTER: i32 = 257;
pub const GLFW_KEY_TAB: i32 = 258;
pub const GLFW_KEY_BACKSPACE: i32 = 259;
pub const GLFW_KEY_INSERT: i32 = 260;
pub const GLFW_KEY_DELETE: i32 = 261;
pub const GLFW_KEY_RIGHT: i32 = 262;
pub const GLFW_KEY_LEFT: i32 = 263;
pub const GLFW_KEY_DOWN: i32 = 264;
pub const GLFW_KEY_UP: i32 = 265;
pub const GLFW_KEY_PAGE_UP: i32 = 266;
pub const GLFW_KEY_PAGE_DOWN: i32 = 267;
pub const GLFW_KEY_HOME: i32 = 268;
pub const GLFW_KEY_END: i32 = 269;
pub const GLFW_KEY_F1: i32 = 290;
pub const GLFW_KEY_F2: i32 = 291;
pub const GLFW_KEY_F3: i32 = 292;
pub const GLFW_KEY_F4: i32 = 293;
pub const GLFW_KEY_F5: i32 = 294;
pub const GLFW_KEY_F6: i32 = 295;
pub const GLFW_KEY_F7: i32 = 296;
pub const GLFW_KEY_F8: i32 = 297;
pub const GLFW_KEY_F9: i32 = 298;
pub const GLFW_KEY_F10: i32 = 299;
pub const GLFW_KEY_F11: i32 = 300;
pub const GLFW_KEY_F12: i32 = 301;
pub const GLFW_KEY_LEFT_SHIFT: i32 = 340;
pub const GLFW_KEY_LEFT_CONTROL: i32 = 341;
pub const GLFW_KEY_LEFT_ALT: i32 = 342;
pub const GLFW_KEY_LEFT_SUPER: i32 = 343;
pub const GLFW_KEY_RIGHT_SHIFT: i32 = 344;
pub const GLFW_KEY_RIGHT_CONTROL: i32 = 345;
pub const GLFW_KEY_RIGHT_ALT: i32 = 346;
pub const GLFW_KEY_RIGHT_SUPER: i32 = 347;

unsafe extern "C" {
    pub fn _glfwCreateWindow(
        title: *const c_char,
        width: c_int,
        height: c_int,
        callback: GLFWframebuffersizefun,
    ) -> *const GLFWwindow;

    pub fn _glfwGetWindowContentScale(
        window: *const GLFWwindow,
        xscale: *mut c_float,
        yscale: *mut c_float,
    );

    pub fn _glfwWindowHint(hint: c_int, value: c_int);

    pub fn _glfwSetWindowUserPointer(window: *const GLFWwindow, pointer: *const c_void);
    pub fn _glfwGetWindowUserPointer(window: *const GLFWwindow) -> *const c_void;

    pub fn _glfwWindowShouldClose(window: *const GLFWwindow) -> c_int;
    pub fn _glfwDestroyWindow(window: *const GLFWwindow);
    pub fn _glfwSwapBuffers(window: *const GLFWwindow);

    pub fn _glfwPollEvents();
    pub fn _glfwTerminate();

    pub fn _glfwGetTime() -> c_double;
    pub fn _glfwSetFramebufferSizeCallback(
        window: *const GLFWwindow,
        callback: GLFWframebuffersizefun,
    );
    pub fn _glfwSetWindowSizeCallback(window: *const GLFWwindow, callback: GLFWwindowsizefun);
    pub fn _glfwSetScrollCallback(window: *const GLFWwindow, callback: GLFWscrollfun);
    pub fn _glfwSetCursorPosCallback(window: *const GLFWwindow, callback: GLFWcursorposfun);
    pub fn _glfwSetKeyCallback(window: *const GLFWwindow, callback: GLFWkeyfun);
    pub fn _glfwSetMouseButtonCallback(window: *const GLFWwindow, callback: GLFWmousebuttonfun);
    pub fn _glfwGetWindowSize(window: *const GLFWwindow, width: *mut c_int, height: *mut c_int);

    pub fn _glfwGetPlatform() -> c_int;
}
