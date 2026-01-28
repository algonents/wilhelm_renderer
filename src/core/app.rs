use crate::core::renderer::{Renderable, Renderer};
use crate::core::Window;

pub struct App<'a> {
    pub window: Box<Window>,
    renderer: Renderer,
    shapes: Vec<Box<dyn Renderable>>,
    pre_render_callback: Option<Box<dyn FnMut(&mut [Box<dyn Renderable>], &Renderer) + 'a>>,
    render_callback: Option<Box<dyn FnMut(&Renderer) + 'a>>,
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
        }
    }

    pub fn renderer(&self) -> &Renderer {
        &self.renderer
    }

    pub fn add_shape(&mut self, shape: impl Renderable + 'static) {
        self.shapes.push(Box::new(shape));
    }

    pub fn add_shapes<R: Renderable + 'static>(&mut self, shapes: Vec<R>) {
        self.shapes.extend(shapes.into_iter().map(|s| Box::new(s) as Box<dyn Renderable>));
    }

    pub fn shapes(&self) -> &[Box<dyn Renderable>] {
        &self.shapes
    }

    pub fn shapes_mut(&mut self) -> &mut [Box<dyn Renderable>] {
        &mut self.shapes
    }

    pub fn on_pre_render<F>(&mut self, callback: F)
    where
        F: FnMut(&mut [Box<dyn Renderable>], &Renderer) + 'a,
    {
        self.pre_render_callback = Some(Box::new(callback));
    }

    pub fn on_render<F>(&mut self, callback: F)
    where
        F: FnMut(&Renderer) + 'a,
    {
        self.render_callback = Some(Box::new(callback));
    }

    pub fn run(mut self) {
        while !self.window.window_should_close() {
            self.window.clear_color();

            if let Some(cb) = self.pre_render_callback.as_mut() {
                cb(&mut self.shapes, &self.renderer);
            }

            for shape in &mut self.shapes {
                shape.render(&self.renderer);
            }

            if let Some(cb) = self.render_callback.as_mut() {
                cb(&self.renderer);
            }

            self.window.swap_buffers();
            self.window.poll_events();
        }
    }
}
