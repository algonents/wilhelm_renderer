//! Safe Rust wrappers around the raw OpenGL FFI exposed by
//! `wilhelm_renderer_sys`.
//!
//! The raw `extern "C"` functions are imported privately via the `sys`
//! alias and are **not** re-exported — client code cannot reach them
//! through this module. Clients that need direct FFI access must depend
//! on `wilhelm_renderer_sys` explicitly.

use std::ffi::{CString, c_char};

// Re-export the public OpenGL types and constants as part of our API.
pub use wilhelm_renderer_sys::opengl::{
    GL_ARRAY_BUFFER, GL_BLEND, GL_CLAMP_TO_EDGE, GL_COMPILE_STATUS, GL_CULL_FACE, GL_DYNAMIC_DRAW,
    GL_ELEMENT_ARRAY_BUFFER, GL_FLOAT, GL_FRAGMENT_SHADER, GL_GEOMETRY_SHADER, GL_LINEAR,
    GL_LINEAR_MIPMAP_LINEAR, GL_LINES, GL_LINE_STRIP, GL_MULTISAMPLE, GL_ONE_MINUS_SRC_ALPHA,
    GL_POINTS, GL_RED, GL_REPEAT, GL_RGB, GL_RGBA, GL_SAMPLES, GL_SRC_ALPHA, GL_STATIC_DRAW,
    GL_TEXTURE0, GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_TEXTURE_MIN_FILTER, GL_TEXTURE_WRAP_S,
    GL_TEXTURE_WRAP_T, GL_TRIANGLES, GL_TRIANGLE_FAN, GL_TRIANGLE_STRIP, GL_UNPACK_ALIGNMENT,
    GL_UNSIGNED_BYTE, GL_UNSIGNED_INT, GL_VERTEX_SHADER, GL_VIEWPORT, GLboolean, GLchar, GLenum,
    GLfloat, GLint, GLsizei, GLsizeiptr, GLuint, GLvoid, Vec2,
};

// Private alias for the raw FFI. Not re-exported.
use wilhelm_renderer_sys::opengl as sys;

pub fn gl_clear_color(red: GLfloat, green: GLfloat, blue: GLfloat, alpha: GLfloat) {
    unsafe { sys::_glClearColor(red, green, blue, alpha) }
}

pub fn gl_viewport(x: GLint, y: GLint, width: GLsizei, height: GLsizei) {
    unsafe {
        sys::_glViewPort(x, y, width, height);
    }
}

pub fn gl_get_integerv(pname: GLenum, data: *mut GLvoid) {
    unsafe {
        sys::_glGetIntegerv(pname, data);
    }
}

pub fn gl_create_fragment_shader() -> GLuint {
    unsafe { sys::_glCreateShader(GL_FRAGMENT_SHADER) }
}

pub fn gl_create_vertex_shader() -> GLuint {
    unsafe { sys::_glCreateShader(GL_VERTEX_SHADER) }
}

pub fn gl_create_geometry_shader() -> GLuint {
    unsafe { sys::_glCreateShader(GL_GEOMETRY_SHADER) }
}

pub fn gl_shader_source(shader: GLuint, source: &str) {
    let c_string = CString::new(source).expect("CString::new failed");
    unsafe { sys::_glShaderSource(shader, c_string.as_ptr()) }
}

pub fn gl_compile_shader(shader: GLuint) {
    unsafe { sys::_glCompileShader(shader) }
}

pub fn gl_delete_shader(shader: GLuint) {
    unsafe { sys::_glDeleteShader(shader) }
}

pub fn gl_get_shaderiv(shader: GLuint, pname: GLenum, params: &mut GLint) {
    unsafe { sys::_glGetShaderiv(shader, pname, params as *mut GLint) }
}

pub fn gl_create_program() -> GLuint {
    unsafe { sys::_glCreateProgram() }
}

pub fn gl_attach_shader(program: GLuint, shader: GLuint) {
    unsafe { sys::_glAttachShader(program, shader) }
}

pub fn gl_link_program(program: GLuint) {
    unsafe {
        sys::_glLinkProgram(program);
    }
}

pub fn gl_delete_program(program: GLuint) {
    unsafe { sys::_glDeleteProgram(program) }
}

pub fn gl_use_program(program: GLuint) {
    unsafe {
        sys::_glUseProgram(program);
    }
}

pub fn gl_gen_buffer() -> GLuint {
    unsafe { sys::_glGenBuffer() }
}
pub fn gl_delete_buffer(buffer: GLuint) {
    unsafe { sys::_glDeleteBuffer(buffer) }
}

pub fn gl_gen_buffers(buffers: &mut Vec<GLuint>) {
    unsafe {
        sys::_glGenBuffers(buffers.len().try_into().unwrap(), buffers.as_mut_ptr());
    }
}

pub fn gl_bind_buffer(target: GLuint, buffer: GLuint) {
    unsafe {
        sys::_glBindBuffer(target, buffer);
    }
}

pub fn gl_gen_texture() -> GLuint {
    unsafe { sys::_glGenTexture() }
}

pub fn gl_bind_texture(target: GLenum, texture: GLuint) {
    unsafe { sys::_glBindTexture(target, texture) }
}

pub fn gl_gen_vertex_array() -> GLuint {
    unsafe { sys::_glGenVertexArray() }
}

pub fn gl_delete_vertex_array(vao: GLuint) {
    unsafe { sys::_glDeleteVertexArray(vao) }
}

pub fn gl_bind_vertex_array(array: GLuint) {
    unsafe {
        sys::_glBindVertexArray(array);
    }
}

pub fn gl_buffer_data<T>(target: GLenum, data: &[T]) {
    unsafe {
        sys::_glBufferData(
            target,
            std::mem::size_of_val(data) as GLsizeiptr,
            data.as_ptr() as *const GLvoid,
            GL_STATIC_DRAW,
        )
    }
}

// 1) Exact orphan/allocate helper — NULL data pointer
pub fn gl_buffer_data_empty_with_usage(target: GLenum, size_bytes: GLsizeiptr, usage: GLenum) {
    unsafe {
        sys::_glBufferData(target, size_bytes, std::ptr::null::<GLvoid>(), usage);
    }
}

// 2) Convenience: dynamic by default (perfect for instance positions updated each frame)
pub fn gl_buffer_data_empty(target: GLenum, size_bytes: GLsizeiptr) {
    gl_buffer_data_empty_with_usage(target, size_bytes, GL_DYNAMIC_DRAW);
}

// 3) (Optional) If you often allocate for vec2<f32> instance arrays:
pub fn gl_buffer_data_empty_vec2(target: GLenum, count_instances: usize) {
    let size_bytes = (count_instances * 2 * std::mem::size_of::<f32>()) as GLsizeiptr;
    gl_buffer_data_empty(target, size_bytes);
}

pub fn gl_buffer_sub_data<T>(target: GLenum, offset: GLsizeiptr, data: &[T]) {
    unsafe {
        sys::_glBufferSubData(
            target,
            offset,
            std::mem::size_of_val(data) as GLsizeiptr,
            data.as_ptr() as *const GLvoid,
        );
    }
}

pub fn gl_buffer_sub_data_vec2(target: GLenum, xy: &[Vec2]) {
    // SAFETY: Vec2 is #[repr(C)] with two f32 fields, guaranteeing tightly packed layout
    let ptr = xy.as_ptr() as *const GLvoid;
    let size_bytes = (xy.len() * std::mem::size_of::<Vec2>()) as GLsizeiptr;
    unsafe {
        sys::_glBufferSubData(target, 0 as GLsizeiptr, size_bytes, ptr);
    }
}

pub fn gl_enable_vertex_attrib_array(index: GLuint) {
    unsafe {
        sys::_glEnableVertexAttribArray(index);
    }
}

pub fn gl_vertex_attrib_pointer_float(
    index: GLuint,
    size: GLint,
    normalize: GLboolean,
    stride: GLsizei,
    offset: GLsizei,
) {
    unsafe {
        sys::_glVertexAttribPointer(index, size, GL_FLOAT, normalize, stride, offset);
    }
}

pub fn gl_draw_arrays(mode: GLenum, first: GLint, count: GLsizei) {
    unsafe {
        sys::_glDrawArrays(mode, first, count);
    }
}

pub fn gl_draw_arrays_instanced(
    mode: GLenum,
    first: GLint,
    count: GLsizei,
    instance_count: GLsizei,
) {
    unsafe {
        sys::_glDrawArraysInstanced(mode, first, count, instance_count);
    }
}

pub fn gl_vertex_attrib_divisor(index: GLuint, divisor: GLuint) {
    unsafe {
        sys::_glVertexAttribDivisor(index, divisor);
    }
}

pub fn gl_vertex_attrib_4f(index: GLuint, v0: GLfloat, v1: GLfloat, v2: GLfloat, v3: GLfloat) {
    unsafe {
        sys::_glVertexAttrib4f(index, v0, v1, v2, v3);
    }
}

pub fn gl_draw_elements(mode: GLenum, count: GLsizei, element_type: GLenum, offset: GLuint) {
    unsafe { sys::_glDrawElements(mode, count, element_type, offset) }
}

pub fn gl_get_uniform_location(program: GLuint, name: &str) -> GLint {
    const MAX_STACK_LEN: usize = 63;

    debug_assert!(!name.contains('\0'), "Uniform name contains null byte");

    if name.len() <= MAX_STACK_LEN {
        // Stack-allocate for typical uniform names (avoids heap allocation)
        let mut buf = [0u8; MAX_STACK_LEN + 1];
        buf[..name.len()].copy_from_slice(name.as_bytes());
        unsafe { sys::_glGetUniformLocation(program, buf.as_ptr() as *const c_char) }
    } else {
        // Fallback to heap for unusually long names
        let c_string = CString::new(name).expect("CString::new failed");
        unsafe { sys::_glGetUniformLocation(program, c_string.as_ptr()) }
    }
}

pub fn gl_uniform_1f(location: GLint, v0: GLfloat) {
    unsafe {
        sys::_glUniform1f(location, v0);
    }
}

pub fn gl_uniform_2f(location: GLint, v0: GLfloat, v1: GLfloat) {
    unsafe {
        sys::_glUniform2f(location, v0, v1);
    }
}

pub fn gl_uniform_3f(location: GLint, v0: GLfloat, v1: GLfloat, v2: GLfloat) {
    unsafe {
        sys::_glUniform3f(location, v0, v1, v2);
    }
}

pub fn gl_uniform_4f(location: GLint, v0: GLfloat, v1: GLfloat, v2: GLfloat, v3: GLfloat) {
    unsafe {
        sys::_glUniform4f(location, v0, v1, v2, v3);
    }
}

pub fn gl_uniform_matrix_4fv(
    location: GLint,
    count: GLsizei,
    transpose: GLboolean,
    value: *const GLfloat,
) {
    unsafe {
        sys::_glUniformMatrix4fv(location, count, transpose, value);
    }
}

pub fn gl_point_size(size: GLfloat) {
    unsafe { sys::_glPointSize(size) }
}

pub fn gl_enable(cap: u32) {
    unsafe {
        sys::_glEnable(cap);
    }
}

pub fn gl_blend_func(sfactor: GLenum, dfactor: GLenum) {
    unsafe { sys::_glBlendFunc(sfactor, dfactor) }
}

pub fn gl_active_texture(unit: GLenum) {
    unsafe {
        sys::_glActiveTexture(unit);
    }
}

pub fn gl_tex_parameteri(target: GLenum, pname: GLenum, param: GLint) {
    unsafe {
        sys::_glTexParameteri(target, pname, param);
    }
}

pub fn gl_generate_mipmap(target: GLenum) {
    unsafe {
        sys::_glGenerateMipmap(target);
    }
}

pub fn gl_tex_image_2d(
    target: GLenum,
    level: GLint,
    internalformat: GLint,
    width: GLsizei,
    height: GLsizei,
    border: GLint,
    format: GLenum,
    data_type: GLenum,
    data: *const GLvoid,
) {
    unsafe {
        sys::_glTexImage2D(
            target,
            level,
            internalformat,
            width,
            height,
            border,
            format,
            data_type,
            data,
        );
    }
}

pub fn gl_tex_sub_image_2d(
    target: GLenum,
    level: GLint,
    xoffset: GLint,
    yoffset: GLint,
    width: GLsizei,
    height: GLsizei,
    format: GLenum,
    data_type: GLenum,
    data: *const GLvoid,
) {
    unsafe {
        sys::_glTexSubImage2D(
            target,
            level,
            xoffset,
            yoffset,
            width,
            height,
            format,
            data_type,
            data,
        );
    }
}

pub fn gl_pixel_storei(pname: GLenum, param: GLint) {
    unsafe {
        sys::_glPixelStorei(pname, param);
    }
}

pub fn gl_delete_texture(texture: GLuint) {
    unsafe {
        sys::_glDeleteTexture(texture);
    }
}
