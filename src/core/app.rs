use std::cell::RefCell;
use std::rc::Rc;

use crate::core::camera::{Camera2D, CameraController};
use crate::core::engine::opengl::Vec2;
use crate::core::renderer::{Renderable, Renderer};
use crate::core::Window;
use crate::graphics2d::shapes::ShapeRenderable;

pub struct App<'a> {
    pub window: Box<Window>,
    renderer: Renderer,
    shapes: Vec<ShapeRenderable>,
    pre_render_callback: Option<Box<dyn FnMut(&mut [ShapeRenderable], &Renderer) + 'a>>,
    render_callback: Option<Box<dyn FnMut(&Renderer, Option<&Camera2D>) + 'a>>,
    camera_controller: Option<Rc<RefCell<CameraController>>>,
}

impl<'a> App<'a> {
    pub fn new(window: Box<Window>) -> Self {
        let renderer = Renderer::new(window.handle());
        Self {
            window,
            renderer,
            shapes: Vec::new(),
            pre_render_callback: None,
            render_callback: None,
            camera_controller: None,
        }
    }

    pub fn renderer(&self) -> &Renderer {
        &self.renderer
    }

    pub fn add_shape(&mut self, shape: ShapeRenderable) {
        self.shapes.push(shape);
    }

    pub fn add_shapes(&mut self, shapes: Vec<ShapeRenderable>) {
        self.shapes.extend(shapes);
    }

    pub fn shapes(&self) -> &[ShapeRenderable] {
        &self.shapes
    }

    pub fn shapes_mut(&mut self) -> &mut [ShapeRenderable] {
        &mut self.shapes
    }

    pub fn on_pre_render<F>(&mut self, callback: F)
    where
        F: FnMut(&mut [ShapeRenderable], &Renderer) + 'a,
    {
        self.pre_render_callback = Some(Box::new(callback));
    }

    pub fn on_render<F>(&mut self, callback: F)
    where
        F: FnMut(&Renderer, Option<&Camera2D>) + 'a,
    {
        self.render_callback = Some(Box::new(callback));
    }

    /// Enable camera-controlled pan and zoom.
    ///
    /// Creates a [`CameraController`] and wires scroll, cursor, mouse button,
    /// and resize callbacks on the window. The camera is passed to the
    /// `on_render` callback each frame as `Option<&Camera2D>`.
    ///
    /// Use [`set_camera_smoothness`](Self::set_camera_smoothness) to enable
    /// smooth animation after calling this method.
    pub fn enable_camera(&mut self, camera: Camera2D) {
        let controller = Rc::new(RefCell::new(CameraController::new(camera)));

        let ctrl = Rc::clone(&controller);
        self.window.on_mouse_button(move |button, action, _| {
            ctrl.borrow_mut().on_mouse_button(button, action);
        });

        let ctrl = Rc::clone(&controller);
        self.window.on_cursor_position(move |x, y| {
            ctrl.borrow_mut().on_cursor_move(x, y);
        });

        let ctrl = Rc::clone(&controller);
        self.window.on_scroll(move |_, y_offset| {
            ctrl.borrow_mut().on_scroll(y_offset);
        });

        let ctrl = Rc::clone(&controller);
        self.window.on_resize(move |width, height| {
            ctrl.borrow_mut()
                .camera_mut()
                .set_screen_size(Vec2::new(width as f32, height as f32));
        });

        self.camera_controller = Some(controller);
    }

    /// Set camera smoothness for animated interpolation.
    ///
    /// - `0.0` (default): instant camera updates
    /// - `> 0.0`: exponential decay rate; typical range 5–12
    ///
    /// No-op if [`enable_camera`](Self::enable_camera) has not been called.
    pub fn set_camera_smoothness(&mut self, value: f32) {
        if let Some(ctrl) = &self.camera_controller {
            ctrl.borrow_mut().set_smoothness(value);
        }
    }

    /// Set camera zoom sensitivity. Default is 1.1 (10% per scroll tick).
    ///
    /// No-op if [`enable_camera`](Self::enable_camera) has not been called.
    pub fn set_camera_zoom_sensitivity(&mut self, value: f32) {
        if let Some(ctrl) = &self.camera_controller {
            ctrl.borrow_mut().set_zoom_sensitivity(value);
        }
    }

    pub fn run(mut self) {
        let mut last_time = self.renderer.get_time();

        while !self.window.window_should_close() {
            let now = self.renderer.get_time();
            let dt = (now - last_time) as f32;
            last_time = now;

            if let Some(ctrl) = &self.camera_controller {
                ctrl.borrow_mut().update(dt);
            }

            self.window.clear_color();

            if let Some(cb) = self.pre_render_callback.as_mut() {
                cb(&mut self.shapes, &self.renderer);
            }

            for shape in &mut self.shapes {
                shape.render(&self.renderer);
            }

            if let Some(cb) = self.render_callback.as_mut() {
                let camera = self.camera_controller.as_ref().map(|ctrl| {
                    *ctrl.borrow().camera()
                });
                cb(&self.renderer, camera.as_ref());
            }

            self.window.swap_buffers();
            self.window.poll_events();
        }
    }
}
