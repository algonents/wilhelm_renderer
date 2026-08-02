//! Browser (WebAssembly) backend.
//!
//! Provides the same `_gl*` / `_glfw*` / `_ft_*` symbols as the native
//! C++ backend, implemented as ordinary Rust functions that forward to a
//! small hand-written JS glue file (see `examples/shapes_wasm/web/glue.js`)
//! through wasm imports in the `"wilhelm"` module. The upper crate compiles
//! against the identical API on every target and cannot tell which backend
//! is underneath.
//!
//! GL object handles: the C API names objects with integers, WebGL with
//! opaque JS objects. The integer→object table lives on the JS side; this
//! module only ever sees the integers, so the contract semantics match the
//! native backend. Id 0 maps to `null` (unbind), matching GL.

pub mod freetype;
pub mod glfw;
pub mod opengl;

// The complete import surface of the backend. Supplied by the JS glue at
// instantiation under the "wilhelm" import module.
#[link(wasm_import_module = "wilhelm")]
unsafe extern "C" {
    // canvas / environment
    pub(crate) fn js_setup_canvas(width: i32, height: i32);
    pub(crate) fn js_canvas_width() -> i32;
    pub(crate) fn js_canvas_height() -> i32;
    pub(crate) fn js_now() -> f64;
    pub(crate) fn js_log(ptr: *const u8, len: usize);

    // GL state
    pub(crate) fn js_gl_clear_color(r: f32, g: f32, b: f32, a: f32);
    pub(crate) fn js_gl_viewport(x: i32, y: i32, w: i32, h: i32);
    pub(crate) fn js_gl_get_viewport(out4: *mut i32);
    pub(crate) fn js_gl_enable(cap: u32);
    pub(crate) fn js_gl_blend_func(sfactor: u32, dfactor: u32);

    // shaders / programs
    pub(crate) fn js_gl_create_shader(shader_type: u32) -> u32;
    pub(crate) fn js_gl_shader_source(shader: u32, ptr: *const u8, len: usize);
    pub(crate) fn js_gl_compile_shader(shader: u32);
    pub(crate) fn js_gl_delete_shader(shader: u32);
    pub(crate) fn js_gl_get_shaderiv(shader: u32, pname: u32) -> i32;
    pub(crate) fn js_gl_create_program() -> u32;
    pub(crate) fn js_gl_attach_shader(program: u32, shader: u32);
    pub(crate) fn js_gl_link_program(program: u32);
    pub(crate) fn js_gl_delete_program(program: u32);
    pub(crate) fn js_gl_use_program(program: u32);

    // buffers / vertex arrays
    pub(crate) fn js_gl_gen_buffer() -> u32;
    pub(crate) fn js_gl_bind_buffer(target: u32, buffer: u32);
    pub(crate) fn js_gl_buffer_data(target: u32, ptr: *const u8, size: i32, usage: u32);
    pub(crate) fn js_gl_buffer_sub_data(target: u32, offset: i32, ptr: *const u8, size: i32);
    pub(crate) fn js_gl_delete_buffer(buffer: u32);
    pub(crate) fn js_gl_gen_vertex_array() -> u32;
    pub(crate) fn js_gl_bind_vertex_array(vao: u32);
    pub(crate) fn js_gl_delete_vertex_array(vao: u32);
    pub(crate) fn js_gl_vertex_attrib_pointer(
        index: u32,
        size: i32,
        data_type: u32,
        normalized: u32,
        stride: i32,
        offset: i32,
    );
    pub(crate) fn js_gl_enable_vertex_attrib_array(index: u32);
    pub(crate) fn js_gl_vertex_attrib_divisor(index: u32, divisor: u32);
    pub(crate) fn js_gl_vertex_attrib_4f(index: u32, v0: f32, v1: f32, v2: f32, v3: f32);

    // textures
    pub(crate) fn js_gl_active_texture(unit: u32);
    pub(crate) fn js_gl_gen_texture() -> u32;
    pub(crate) fn js_gl_bind_texture(target: u32, texture: u32);
    pub(crate) fn js_gl_tex_parameteri(target: u32, pname: u32, param: i32);
    pub(crate) fn js_gl_generate_mipmap(target: u32);
    pub(crate) fn js_gl_pixel_storei(pname: u32, param: i32);
    pub(crate) fn js_gl_delete_texture(texture: u32);
    pub(crate) fn js_gl_tex_image_2d(
        target: u32,
        level: i32,
        internalformat: i32,
        width: i32,
        height: i32,
        border: i32,
        format: u32,
        data_type: u32,
        ptr: *const u8,
    );
    pub(crate) fn js_gl_tex_sub_image_2d(
        target: u32,
        level: i32,
        xoffset: i32,
        yoffset: i32,
        width: i32,
        height: i32,
        format: u32,
        data_type: u32,
        ptr: *const u8,
    );

    // draws
    pub(crate) fn js_gl_draw_arrays(mode: u32, first: i32, count: i32);
    pub(crate) fn js_gl_draw_arrays_instanced(mode: u32, first: i32, count: i32, instances: i32);
    pub(crate) fn js_gl_draw_elements(mode: u32, count: i32, element_type: u32, offset: u32);

    // uniforms
    pub(crate) fn js_gl_get_uniform_location(program: u32, ptr: *const u8, len: usize) -> i32;
    pub(crate) fn js_gl_uniform_1f(location: i32, v0: f32);
    pub(crate) fn js_gl_uniform_2f(location: i32, v0: f32, v1: f32);
    pub(crate) fn js_gl_uniform_3f(location: i32, v0: f32, v1: f32, v2: f32);
    pub(crate) fn js_gl_uniform_4f(location: i32, v0: f32, v1: f32, v2: f32, v3: f32);
    pub(crate) fn js_gl_uniform_matrix_4fv(location: i32, count: i32, transpose: u32, ptr: *const f32);
}

/// Log a message to the browser console (backend-internal diagnostics).
pub(crate) fn console_log(msg: &str) {
    unsafe { js_log(msg.as_ptr(), msg.len()) }
}
