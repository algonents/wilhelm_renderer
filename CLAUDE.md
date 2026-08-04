# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

wilhelm-renderer is a GPU-accelerated 2D display engine for real-time operational visualization, written in Rust with native OpenGL bindings. See `docs/DESIGN.md` for full architecture, design decisions, and dependency policy.

## Crate Structure

The project is split into two published crates (the openssl/openssl-sys convention):

- **`wilhelm_renderer`** (repo root) — the safe Rust API. Contains no C/C++ source and no `build.rs`. Depends on the sys crate via an exact version pin (`version = "=0.10.1"`).
- **`wilhelm_renderer_sys`** (`wilhelm_renderer_sys/`) — raw `extern "C"` bindings plus the bundled GLFW 3.4 C/C++ sources and the CMake `build.rs`. Exposes no safe API.

A Cargo `[workspace]` at the root ties both crates and all examples together (`cargo build --workspace`). The layout is asymmetric: the upper crate lives at the repo root and the sys crate sits in a subdirectory — there is no virtual workspace manifest.

**Why the split:** crates.io perception — bundling ~2 MiB of C source in the main crate made it look like a C project. Keeping the native code in `*-sys` isolates it.

## Build Commands

```bash
# Build the library
cargo build

# Build all examples (catches API breakage)
cargo build --workspace

# Run a standalone example
cd examples/shapes && cargo run
```

### Build Requirements

- C/C++ compiler and CMake (cmake crate invokes CMake during build)
- Linux: `libgl1-mesa-dev`, `libwayland-dev`, `libxkbcommon-dev`
- GLFW 3.4 is bundled, no external dependency needed

### C++ FFI Build

All native build logic lives in the **sys crate**. `wilhelm_renderer_sys/build.rs` uses CMake to compile the C++ layer (`wilhelm_renderer_sys/cpp/`). Platform-specific linking:
- Linux: Statically links glrenderer, glfw3; dynamically links GL and stdc++
- macOS: Statically links glrenderer, glfw3; links Cocoa, CoreFoundation, IOKit, CoreVideo frameworks and c++
- Windows: Statically links glrenderer, glfw3; links opengl32, gdi32, user32, shell32, kernel32

Notes:
- The `Cargo.toml` `links = "wilhelm_renderer"` key plus `cargo:include=...` in `build.rs` publish the bundled GLFW include path downstream as `DEP_WILHELM_RENDERER_INCLUDE`, so direct dependents (e.g. `wilhelm_renderer_imgui`) compile against the exact same GLFW headers.
- `build.rs` skips the native build entirely when `DOCS_RS` is set (docs.rs generation).

### Bundled Dependencies

**GLFW 3.4** (`wilhelm_renderer_sys/cpp/glfw-3.4/`, window management):
- Bundled in full, built via CMake

Text rendering is pure Rust (no bundled C): `ttf-parser` + `ab_glyph_rasterizer`
(both zero-dependency crates) in `src/core/font.rs`, above the sys contract —
the same glyph rasterization runs on every backend, including wasm.

## Implementation Patterns

These patterns are specific to working in this codebase:

- **FFI Wrapper Pattern**: The raw `extern "C"` declarations live in `wilhelm_renderer_sys` (`opengl`, `glfw` modules). The safe wrappers in `src/core/engine/` import them privately via a `use wilhelm_renderer_sys::<mod> as sys;` alias and call through `sys::`. The raw functions are **not** re-exported — only public types and constants are surfaced via explicit `pub use`. When adding a new FFI function: declare it `pub` in the sys crate, then add a safe wrapper in the matching engine module calling through the `sys::` alias; do NOT add the raw function to the `pub use` list.
- **Interior Mutability**: Window uses `Rc<Cell<>>` for shared state across callbacks
- **Component-Based Meshes**: Mesh = Geometry + Shader + Transform
- **Callback-Driven App Loop**: App uses closures for render logic
- **Box<Window> for FFI Stability**: `Window::new()` returns `Box<Window>` and `App` stores `Box<Window>` because GLFW callbacks receive a raw pointer to the Window (via `glfw_set_window_user_pointer`). The Box ensures a stable heap address that won't invalidate when App is moved.
- **Singleton Shaders**: Shape shaders are lazy-loaded via `thread_local` `OnceCell`, shared across all shapes

## Performance Architecture

The current architecture supports scaling to 10,000+ shapes through evolutionary changes:

**Existing Foundation:**
- Direct FFI access to OpenGL instancing (`glVertexAttribDivisor`, `glDrawArraysInstanced`)
- Working instancing in `Geometry` (`enable_instancing_xy`, `update_instance_xy`) - proven with 1000+ shapes
- Singleton shaders via OnceCell - shapes share shaders, minimizing shader switches

**Current Limitation:**
The high-level `ShapeRenderable` API uses 1 draw call per shape, which becomes a CPU bottleneck at high counts. This limitation is in the convenience API, not the core engine.

**Escape Hatch:**
`App::on_render()` provides direct `Renderer` access for custom batching. The instancing infrastructure is fully functional for manual batching.

**Scaling Strategy (additive, not rewrite):**
1. Automatic batching in `App::run()` — group shapes by type, render with instancing
2. Extended instancing — per-instance scale (location 3) is mechanical to add
3. Frustum culling — CPU-side viewport bounds check before batching

## Key Files

- `src/lib.rs`: Library root, exports `core` and `graphics2d` modules
- `src/core/engine/`: Safe FFI wrappers (`opengl.rs`, `glfw.rs`) over the sys crate
- `src/core/font.rs`: Pure-Rust glyph rasterization + font atlas (`ttf-parser`, `ab_glyph_rasterizer`)
- `src/core/geometry.rs`: VAO/VBO management and instancing setup
- `src/graphics2d/shapes/shaperenderable.rs`: Main shape rendering implementation
- `src/graphics2d/shapes/mod.rs`: Shape data types (geometry only, no GPU)
- `wilhelm_renderer_sys/src/lib.rs`: Raw FFI bindings root (`opengl`, `glfw` modules)
- `wilhelm_renderer_sys/cpp/glrenderer.cpp`: C++ wrapper functions called via FFI
- `wilhelm_renderer_sys/build.rs`: CMake integration and platform-specific linking

## Platform Notes

- Supports both Wayland and X11 on Linux (GLFW selects backend at runtime)
- OpenGL 3.3 Core Profile for macOS compatibility
- MSAA 4x multisampling enabled by default

## Project Planning

- **docs/DESIGN.md**: Architecture and key design decisions (Shape vs ShapeRenderable, dependency policy, rendering pipeline, client architecture).
- **docs/ROADMAP.md**: Planned library enhancements (text rendering, projections, interaction, layers, trails) to support SkyTracker and other visualization applications.
- **docs/TODO.md**: Technical debt and improvement areas including resource leaks, FFI issues, performance optimizations, and code style cleanup.
- **docs/PRIMITIVES.md**: Graphics primitives needed for 2D visualization (maps, radar, data viz), organized by priority (critical, important, nice to have).
- **docs/SHAPE_API_REVIEW.md**: API inconsistencies and improvement roadmap.
- **docs/PERFORMANCE_ANALYSIS.md**: Performance analysis of `ShapeKind` (ADT vs trait objects), move/borrow costs, enum size, and rendering hot paths.
- **CHANGELOG.md**: Record of API changes, improvements, and known limitations.
