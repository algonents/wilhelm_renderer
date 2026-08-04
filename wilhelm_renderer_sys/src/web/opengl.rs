//! WebGL2 implementations of the `_gl*` contract symbols.
//!
//! Signatures mirror the native `extern "C"` declarations in
//! `crate::opengl` exactly; the upper crate compiles unchanged.

#![allow(non_snake_case)]

use std::ffi::CStr;

use crate::opengl::{
    GL_VIEWPORT, GLboolean, GLchar, GLenum, GLfloat, GLint, GLsizei, GLsizeiptr, GLuint, GLvoid,
};

use super::*;

fn as_u32(b: GLboolean) -> u32 {
    matches!(b, GLboolean::TRUE) as u32
}

pub unsafe fn _glClearColor(red: GLfloat, green: GLfloat, blue: GLfloat, alpha: GLfloat) {
    // Contract note: the native shim fuses glClear(GL_COLOR_BUFFER_BIT) into
    // this call; the glue does the same.
    unsafe { js_gl_clear_color(red, green, blue, alpha) }
}

pub unsafe fn _glViewPort(x: GLint, y: GLint, width: GLsizei, height: GLsizei) {
    unsafe { js_gl_viewport(x, y, width, height) }
}

pub unsafe fn _glGetIntegerv(pname: GLenum, data: *mut GLvoid) {
    // GL_VIEWPORT is the only pname the engine queries.
    if pname == GL_VIEWPORT {
        unsafe { js_gl_get_viewport(data as *mut i32) }
    } else {
        console_log("wilhelm wasm backend: _glGetIntegerv pname unsupported");
    }
}

pub unsafe fn _glCreateShader(shaderType: GLenum) -> GLuint {
    unsafe { js_gl_create_shader(shaderType) }
}

pub unsafe fn _glShaderSource(shader: GLuint, source: *const GLchar) {
    let bytes = unsafe { CStr::from_ptr(source) }.to_bytes();
    unsafe { js_gl_shader_source(shader, bytes.as_ptr(), bytes.len()) }
}

pub unsafe fn _glCompileShader(shader: GLuint) {
    unsafe { js_gl_compile_shader(shader) }
}

pub unsafe fn _glDeleteShader(shader: GLuint) {
    unsafe { js_gl_delete_shader(shader) }
}

pub unsafe fn _glGetShaderiv(shader: GLuint, pname: GLenum, params: *mut GLint) {
    let value = unsafe { js_gl_get_shaderiv(shader, pname) };
    unsafe { *params = value }
}

pub unsafe fn _glCreateProgram() -> GLuint {
    unsafe { js_gl_create_program() }
}

pub unsafe fn _glAttachShader(program: GLuint, shader: GLuint) {
    unsafe { js_gl_attach_shader(program, shader) }
}

pub unsafe fn _glLinkProgram(program: GLuint) {
    unsafe { js_gl_link_program(program) }
}

pub unsafe fn _glDeleteProgram(program: GLuint) {
    unsafe { js_gl_delete_program(program) }
}

pub unsafe fn _glUseProgram(program: GLuint) {
    unsafe { js_gl_use_program(program) }
}

pub unsafe fn _glGenBuffer() -> GLuint {
    unsafe { js_gl_gen_buffer() }
}

pub unsafe fn _glGenBuffers(n: GLsizei, buffers: *mut GLuint) {
    for i in 0..n {
        unsafe { *buffers.offset(i as isize) = js_gl_gen_buffer() }
    }
}

pub unsafe fn _glBindBuffer(target: GLenum, buffer: GLuint) {
    unsafe { js_gl_bind_buffer(target, buffer) }
}

pub unsafe fn _glBufferData(target: GLenum, size: GLsizeiptr, data: *const GLvoid, usage: GLenum) {
    unsafe { js_gl_buffer_data(target, data as *const u8, size as i32, usage) }
}

pub unsafe fn _glBufferSubData(
    target: GLenum,
    offset: GLsizeiptr,
    size: GLsizeiptr,
    data: *const GLvoid,
) {
    unsafe { js_gl_buffer_sub_data(target, offset as i32, data as *const u8, size as i32) }
}

pub unsafe fn _glDeleteBuffer(buffer: GLuint) {
    unsafe { js_gl_delete_buffer(buffer) }
}

pub unsafe fn _glGenVertexArray() -> GLuint {
    unsafe { js_gl_gen_vertex_array() }
}

pub unsafe fn _glDeleteVertexArray(vao: GLuint) {
    unsafe { js_gl_delete_vertex_array(vao) }
}

pub unsafe fn _glBindVertexArray(VAO: GLuint) {
    unsafe { js_gl_bind_vertex_array(VAO) }
}

pub unsafe fn _glVertexAttribPointer(
    index: GLuint,
    size: GLint,
    dataType: GLenum,
    normalize: GLboolean,
    stride: GLsizei,
    offset: GLsizei,
) {
    unsafe { js_gl_vertex_attrib_pointer(index, size, dataType, as_u32(normalize), stride, offset) }
}

pub unsafe fn _glActiveTexture(unit: GLenum) {
    unsafe { js_gl_active_texture(unit) }
}

pub unsafe fn _glGenTexture() -> GLuint {
    unsafe { js_gl_gen_texture() }
}

pub unsafe fn _glBindTexture(target: GLenum, texture: GLuint) {
    unsafe { js_gl_bind_texture(target, texture) }
}

pub unsafe fn _glTexParameteri(target: GLenum, pname: GLenum, param: GLint) {
    unsafe { js_gl_tex_parameteri(target, pname, param) }
}

pub unsafe fn _glGenerateMipmap(target: GLenum) {
    unsafe { js_gl_generate_mipmap(target) }
}

pub unsafe fn _glTexImage2D(
    target: GLenum,
    level: GLint,
    internalformat: GLint,
    width: GLsizei,
    height: GLsizei,
    border: GLint,
    format: GLenum,
    dataType: GLenum,
    data: *const GLvoid,
) {
    unsafe {
        js_gl_tex_image_2d(
            target,
            level,
            internalformat,
            width,
            height,
            border,
            format,
            dataType,
            data as *const u8,
        )
    }
}

pub unsafe fn _glTexSubImage2D(
    target: GLenum,
    level: GLint,
    xoffset: GLint,
    yoffset: GLint,
    width: GLsizei,
    height: GLsizei,
    format: GLenum,
    dataType: GLenum,
    data: *const GLvoid,
) {
    unsafe {
        js_gl_tex_sub_image_2d(
            target,
            level,
            xoffset,
            yoffset,
            width,
            height,
            format,
            dataType,
            data as *const u8,
        )
    }
}

pub unsafe fn _glPixelStorei(pname: GLenum, param: GLint) {
    unsafe { js_gl_pixel_storei(pname, param) }
}

pub unsafe fn _glDeleteTexture(texture: GLuint) {
    unsafe { js_gl_delete_texture(texture) }
}

pub unsafe fn _glEnableVertexAttribArray(index: GLuint) {
    unsafe { js_gl_enable_vertex_attrib_array(index) }
}

pub unsafe fn _glDrawArrays(mode: GLenum, first: GLint, count: GLsizei) {
    unsafe { js_gl_draw_arrays(mode, first, count) }
}

pub unsafe fn _glDrawArraysInstanced(
    mode: GLenum,
    first: GLint,
    count: GLsizei,
    instancecount: GLsizei,
) {
    unsafe { js_gl_draw_arrays_instanced(mode, first, count, instancecount) }
}

pub unsafe fn _glVertexAttribDivisor(index: GLuint, divisor: GLuint) {
    unsafe { js_gl_vertex_attrib_divisor(index, divisor) }
}

pub unsafe fn _glVertexAttrib4f(index: GLuint, v0: GLfloat, v1: GLfloat, v2: GLfloat, v3: GLfloat) {
    unsafe { js_gl_vertex_attrib_4f(index, v0, v1, v2, v3) }
}

pub unsafe fn _glDrawElements(mode: GLenum, count: GLsizei, element_type: GLenum, offset: GLuint) {
    unsafe { js_gl_draw_elements(mode, count, element_type, offset) }
}

pub unsafe fn _glGetUniformLocation(program: GLuint, name: *const GLchar) -> GLint {
    let bytes = unsafe { CStr::from_ptr(name) }.to_bytes();
    unsafe { js_gl_get_uniform_location(program, bytes.as_ptr(), bytes.len()) }
}

pub unsafe fn _glUniform1f(location: GLint, v0: GLfloat) {
    unsafe { js_gl_uniform_1f(location, v0) }
}

pub unsafe fn _glUniform2f(location: GLint, v0: GLfloat, v1: GLfloat) {
    unsafe { js_gl_uniform_2f(location, v0, v1) }
}

pub unsafe fn _glUniform3f(location: GLint, v0: GLfloat, v1: GLfloat, v2: GLfloat) {
    unsafe { js_gl_uniform_3f(location, v0, v1, v2) }
}

pub unsafe fn _glUniform4f(location: GLint, v0: GLfloat, v1: GLfloat, v2: GLfloat, v3: GLfloat) {
    unsafe { js_gl_uniform_4f(location, v0, v1, v2, v3) }
}

pub unsafe fn _glUniformMatrix4fv(
    location: GLint,
    count: GLsizei,
    transpose: GLboolean,
    value: *const GLfloat,
) {
    unsafe { js_gl_uniform_matrix_4fv(location, count, as_u32(transpose), value) }
}

pub unsafe fn _glEnable(cap: GLenum) {
    // The glue skips GL_MULTISAMPLE (context-creation attribute in WebGL).
    unsafe { js_gl_enable(cap) }
}

pub unsafe fn _glBlendFunc(sfactor: GLenum, dfactor: GLenum) {
    unsafe { js_gl_blend_func(sfactor, dfactor) }
}
