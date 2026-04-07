//! Raw OpenGL FFI bindings.
//!
//! All extern declarations, C-compatible types, and OpenGL constants
//! used by `wilhelm_renderer`. Safe Rust wrappers live in the upper
//! crate.

use std::ffi::{c_char, c_float, c_int, c_long, c_uint, c_void};

pub type GLenum = c_uint;
pub type GLsizei = c_int;
pub type GLsizeiptr = c_long;
pub type GLchar = c_char;
pub type GLint = c_int;
pub type GLuint = c_uint;
pub type GLfloat = c_float;
pub type GLvoid = c_void;

/// A 2D vector with guaranteed C-compatible memory layout.
/// Used for uploading vertex data to OpenGL.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[repr(C)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl From<(f32, f32)> for Vec2 {
    fn from((x, y): (f32, f32)) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum GLboolean {
    FALSE = 0,
    TRUE = 1,
}

pub const GL_ARRAY_BUFFER: u32 = 0x8892;
pub const GL_ELEMENT_ARRAY_BUFFER: u32 = 0x8893;

pub const GL_FRAGMENT_SHADER: u32 = 0x8B30;
pub const GL_VERTEX_SHADER: u32 = 0x8B31;
pub const GL_GEOMETRY_SHADER: u32 = 0x8DD9;
pub const GL_COMPILE_STATUS: u32 = 0x8B81;

pub const GL_STATIC_DRAW: u32 = 0x88E4;
pub const GL_DYNAMIC_DRAW: u32 = 0x88E8;
pub const GL_FLOAT: u32 = 0x1406;
pub const GL_UNSIGNED_INT: u32 = 0x1405;
pub const GL_UNSIGNED_BYTE: u32 = 0x1401;
pub const GL_POINTS: u32 = 0x0000;
pub const GL_LINES: u32 = 0x0001;
pub const GL_LINE_STRIP: u32 = 0x0003;
pub const GL_TRIANGLES: u32 = 0x0004;
pub const GL_TRIANGLE_FAN: u32 = 0x0006;
pub const GL_TRIANGLE_STRIP: u32 = 0x0005;
pub const GL_VIEWPORT: u32 = 0x0BA2;
pub const GL_TEXTURE_2D: u32 = 0x0DE1;
pub const GL_RED: u32 = 0x1903;
pub const GL_CULL_FACE: u32 = 0x0B44;
pub const GL_BLEND: u32 = 0x0BE2;
pub const GL_SRC_ALPHA: u32 = 0x0302;
pub const GL_ONE_MINUS_SRC_ALPHA: u32 = 0x0303;
pub const GL_TEXTURE0: u32 = 0x84C0;

pub const GL_TEXTURE_WRAP_S: u32 = 0x2802;
pub const GL_TEXTURE_WRAP_T: u32 = 0x2803;
pub const GL_REPEAT: GLint = 0x2901;
pub const GL_CLAMP_TO_EDGE: GLint = 0x812F;
pub const GL_TEXTURE_MAG_FILTER: u32 = 0x2800;
pub const GL_TEXTURE_MIN_FILTER: u32 = 0x2801;
pub const GL_LINEAR: GLint = 0x2601;
pub const GL_LINEAR_MIPMAP_LINEAR: GLint = 0x2703;
pub const GL_RGB: GLint = 0x1907;
pub const GL_RGBA: GLint = 0x1908;
pub const GL_MULTISAMPLE: GLuint = 0x809D;
pub const GL_SAMPLES: GLuint = 0x80A9;
pub const GL_UNPACK_ALIGNMENT: GLenum = 0x0CF5;

unsafe extern "C" {
    pub fn _glClearColor(red: GLfloat, green: GLfloat, blue: GLfloat, alpha: GLfloat);
    pub fn _glViewPort(x: GLint, y: GLint, width: GLsizei, height: GLsizei);
    pub fn _glGetIntegerv(pname: GLenum, data: *mut GLvoid);
    pub fn _glCreateShader(shaderType: GLenum) -> GLuint;
    pub fn _glShaderSource(shader: GLuint, source: *const c_char);
    pub fn _glCompileShader(shader: GLuint);
    pub fn _glDeleteShader(shader: GLuint);
    pub fn _glGetShaderiv(shader: GLuint, pname: GLenum, params: *mut GLint);
    pub fn _glCreateProgram() -> GLuint;
    pub fn _glAttachShader(program: GLuint, shader: GLuint);
    pub fn _glLinkProgram(program: GLuint);
    pub fn _glDeleteProgram(program: GLuint);
    pub fn _glUseProgram(program: GLuint);
    pub fn _glGenBuffer() -> GLuint;
    pub fn _glGenBuffers(n: GLsizei, buffers: *mut GLuint);
    pub fn _glBindBuffer(target: GLenum, buffer: GLuint);
    pub fn _glBufferData(target: GLenum, size: GLsizeiptr, data: *const GLvoid, usage: GLenum);
    pub fn _glBufferSubData(
        target: GLenum,
        offset: GLsizeiptr,
        size: GLsizeiptr,
        data: *const GLvoid,
    );
    pub fn _glDeleteBuffer(buffer: GLuint);
    pub fn _glGenVertexArray() -> GLuint;
    pub fn _glDeleteVertexArray(vao: GLuint);
    pub fn _glBindVertexArray(VAO: GLuint);
    pub fn _glVertexAttribPointer(
        index: GLuint,
        size: GLint,
        dataType: GLenum,
        normalize: GLboolean,
        stride: GLsizei,
        offset: GLsizei,
    );
    pub fn _glActiveTexture(unit: GLenum);
    pub fn _glGenTexture() -> GLuint;
    pub fn _glBindTexture(target: GLenum, texture: GLuint);
    pub fn _glTexParameteri(target: GLenum, pname: GLenum, param: GLint);
    pub fn _glGenerateMipmap(target: GLenum);
    pub fn _glTexImage2D(
        target: GLenum,
        level: GLint,
        internalformat: GLint,
        width: GLsizei,
        height: GLsizei,
        border: GLint,
        format: GLenum,
        dataType: GLenum,
        data: *const GLvoid,
    );
    pub fn _glTexSubImage2D(
        target: GLenum,
        level: GLint,
        xoffset: GLint,
        yoffset: GLint,
        width: GLsizei,
        height: GLsizei,
        format: GLenum,
        dataType: GLenum,
        data: *const GLvoid,
    );
    pub fn _glPixelStorei(pname: GLenum, param: GLint);
    pub fn _glDeleteTexture(texture: GLuint);
    pub fn _glEnableVertexAttribArray(index: GLuint);
    pub fn _glDrawArrays(mode: GLenum, first: GLint, count: GLsizei);
    pub fn _glDrawArraysInstanced(
        mode: GLenum,
        first: GLint,
        count: GLsizei,
        instancecount: GLsizei,
    );
    pub fn _glVertexAttribDivisor(index: GLuint, divisor: GLuint);
    pub fn _glVertexAttrib4f(index: GLuint, v0: GLfloat, v1: GLfloat, v2: GLfloat, v3: GLfloat);
    pub fn _glDrawElements(mode: GLenum, count: GLsizei, element_type: GLenum, offset: GLuint);

    pub fn _glGetUniformLocation(program: GLuint, name: *const GLchar) -> GLint;
    pub fn _glUniform1f(location: GLint, v0: GLfloat);
    pub fn _glUniform2f(location: GLint, v0: GLfloat, v1: GLfloat);
    pub fn _glUniform3f(location: GLint, v0: GLfloat, v1: GLfloat, v2: GLfloat);
    pub fn _glUniform4f(location: GLint, v0: GLfloat, v1: GLfloat, v2: GLfloat, v3: GLfloat);
    pub fn _glUniformMatrix4fv(
        location: GLint,
        count: GLsizei,
        transpose: GLboolean,
        value: *const GLfloat,
    );
    pub fn _glPointSize(size: GLfloat);
    pub fn _glEnable(cap: GLenum);
    pub fn _glBlendFunc(sfactor: GLenum, dfactor: GLenum);
}
