use std::cell::Cell;
use std::ffi::c_void;
use std::rc::Rc;
use crate::core::Color;
use crate::core::engine::opengl::{gl_clear_color, gl_viewport};
use crate::core::engine::glfw::{GLFWwindow, glfw_create_window, glfw_destroy_window, glfw_get_window_content_scale, glfw_get_window_user_pointer, glfw_poll_events, glfw_set_cursor_pos_callback, glfw_set_key_callback, glfw_set_scroll_callback, glfw_set_window_size_callback, glfw_set_window_user_pointer, glfw_swap_buffers, glfw_window_should_close};


/// Shared inner state that both Window and WindowHandle can access.
struct InnerWindow {
    width: Cell<i32>,
    height: Cell<i32>,
    background_color: Cell<Color>,
}

pub struct Window {
    inner: Rc<InnerWindow>,
    glfw_window: *const GLFWwindow,
    on_resize: Option<Box<dyn FnMut(i32, i32)>>,
    on_scroll: Option<Box<dyn FnMut(f64, f64)>>,
    on_cursor_position: Option<Box<dyn FnMut(f64, f64)>>,
    on_key: Option<Box<dyn FnMut(i32, i32, i32, i32)>>,
}

/// Cheap, cloneable handle to query window state without owning the window.
#[derive(Clone)]
pub struct WindowHandle {
    inner: Rc<InnerWindow>,
}

extern "C" fn _on_viewport_resized(_window: *const GLFWwindow, width: i32, height: i32) {
    gl_viewport(0, 0, width, height);
}

extern "C" fn _on_window_resized_callback(_window: *const GLFWwindow, width: i32, height: i32){
    let user_ptr = glfw_get_window_user_pointer(_window);
    if !user_ptr.is_null() {
        unsafe {
            let window_ref: &mut Window = &mut *(user_ptr as *mut Window);
            window_ref.inner.width.set(width);
            window_ref.inner.height.set(height);
            window_ref._on_resize(width, height);
        }
    }
}

extern "C" fn _on_scroll_callback(_window: *const GLFWwindow, x_offset: f64, y_offset: f64) {
    let user_ptr = glfw_get_window_user_pointer(_window);
    if !user_ptr.is_null() {
        unsafe {
            let window_ref: &mut Window = &mut *(user_ptr as *mut Window);
            window_ref._on_scroll(x_offset, y_offset);
        }
    }
}

extern "C" fn _on_cursor_position_callback(_window: *const GLFWwindow, x_pos: f64, y_pos: f64) {
    let user_ptr = glfw_get_window_user_pointer(_window);
    if !user_ptr.is_null() {
        unsafe {
            let window_ref: &mut Window = &mut *(user_ptr as *mut Window);
            window_ref._on_cursor_position(x_pos, y_pos);
        }
    }
}

extern "C" fn _on_key_callback(
    _window: *const GLFWwindow,
    key: i32,
    scancode: i32,
    action: i32,
    mods: i32,
) {
    let user_ptr = glfw_get_window_user_pointer(_window);
    if !user_ptr.is_null() {
        unsafe {
            let window_ref: &mut Window = &mut *(user_ptr as *mut Window);
            window_ref._on_key(key, scancode, action, mods);
        }
    }
}

impl Window {
    pub fn new(title: &str, width: i32, height: i32, background_color: Color) -> Box<Self> {
        let glfw_window = glfw_create_window(title, width, height, Some(_on_viewport_resized));
        // hook callbacks
        glfw_set_window_size_callback(glfw_window, Some(_on_window_resized_callback));
        glfw_set_scroll_callback(glfw_window, Some(_on_scroll_callback));
        glfw_set_cursor_pos_callback(glfw_window, Some(_on_cursor_position_callback));
        glfw_set_key_callback(glfw_window, Some(_on_key_callback));

        let inner = Rc::new(InnerWindow {
            width: Cell::new(width),
            height: Cell::new(height),
            background_color: Cell::new(background_color),
        });

        let mut window = Box::new(Window {
            glfw_window,
            inner,
            on_resize: None,
            on_scroll: None,
            on_cursor_position: None,
            on_key: None,
        });
        glfw_set_window_user_pointer(glfw_window, &mut *window as *mut _ as *mut c_void);
        gl_clear_color(background_color.red_value(), background_color.green_value(), background_color.blue_value(), 1.0);
        window
    }

    /// Get a cloneable handle of the windows state without owning the window
    pub fn handle(&self) -> WindowHandle {
        WindowHandle {
            inner: Rc::clone(&self.inner),
        }
    }

    pub fn width(&self)->i32{
        self.inner.width.get()
    }

    pub fn height(&self)->i32{
        self.inner.height.get()
    }

    pub fn content_scale(&self)->(f32, f32){
        glfw_get_window_content_scale(self.glfw_window)
    }

    /// Returns the raw GLFW window pointer for FFI integration (e.g., ImGui).
    pub fn glfw_window_ptr(&self) -> *const GLFWwindow {
        self.glfw_window
    }

    pub fn clear_color(&self) {
        gl_clear_color(self.inner.background_color.get().red_value(), self.inner.background_color.get().green_value(), self.inner.background_color.get().blue_value(), 1.0);
    }
    pub fn window_should_close(&self) -> bool {
        glfw_window_should_close(self.glfw_window)
    }
    pub fn swap_buffers(&self) {
        glfw_swap_buffers(self.glfw_window);
    }
    pub fn poll_events(&self) {
        glfw_poll_events();
    }

    pub fn on_resize<F>(&mut self, f: F)
    where
        F: FnMut(i32, i32) + 'static,
    {
        self.on_resize = Some(Box::new(f));
    }

    pub fn on_scroll<F>(&mut self, f: F)
    where
        F: FnMut(f64, f64) + 'static,
    {
        self.on_scroll = Some(Box::new(f));
    }

    pub fn on_cursor_position<F>(&mut self, f: F)
    where
        F: FnMut(f64, f64) + 'static,
    {
        self.on_cursor_position = Some(Box::new(f));
    }

    pub fn on_key<F>(&mut self, f: F)
    where
        F: FnMut(i32, i32, i32, i32) + 'static,
    {
        self.on_key = Some(Box::new(f));
    }

    fn _on_resize(&mut self, width: i32, height: i32) {
        if let Some(callback) = &mut self.on_resize {
            callback(width, height);
        }
    }

    fn _on_scroll(&mut self, x_offset: f64, y_offset: f64) {
        if let Some(callback) = &mut self.on_scroll {
            callback(x_offset, y_offset);
        }
    }
    fn _on_cursor_position(&mut self, x_pos: f64, y_pos: f64) {
        if let Some(callback) = &mut self.on_cursor_position {
            callback(x_pos, y_pos);
        }
    }

    fn _on_key(&mut self, key: i32, scancode: i32, action: i32, mods: i32) {
        if let Some(callback) = &mut self.on_key {
            callback(key, scancode, action, mods);
        }
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        if !self.glfw_window.is_null() {
            glfw_destroy_window(self.glfw_window);
        }
    }
}

impl WindowHandle {
    #[inline]
    pub fn size(&self) -> (i32, i32) {
        (self.inner.width.get(), self.inner.height.get())
    }
    #[inline]
    pub fn width(&self) -> i32 {
        self.inner.width.get()
    }
    #[inline]
    pub fn height(&self) -> i32 {
        self.inner.height.get()
    }
    #[inline]
    pub fn background_color(&self) -> Color {self.inner.background_color.get()}
}
