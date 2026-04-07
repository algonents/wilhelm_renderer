//! Raw FFI bindings and bundled native dependencies for `wilhelm_renderer`.
//!
//! This crate contains only `extern "C"` declarations, C-compatible type
//! definitions, constants, and the CMake build integration for the bundled
//! GLFW 3.4 and FreeType 2.13.2 sources. It exposes no safe Rust API —
//! safe wrappers live in the upper `wilhelm_renderer` crate.

pub mod freetype;
pub mod glfw;
pub mod opengl;
