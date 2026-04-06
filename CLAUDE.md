# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

wilhelm-renderer is a GPU-accelerated 2D display engine for real-time operational visualization, written in Rust with native OpenGL bindings. See `docs/design.md` for full architecture, design decisions, and dependency policy.

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

`build.rs` uses CMake to compile the C++ layer (`cpp/`). Platform-specific linking:
- Linux: Statically links glrenderer, glfw3; dynamically links GL
- macOS: Links Cocoa, CoreFoundation, IOKit, CoreVideo frameworks
- Windows: Links opengl32, gdi32, user32, shell32

### Bundled Dependencies

**FreeType 2.13.2** (text rendering):
- Minimal build: Only TrueType/OpenType support with gzip for WOFF web fonts
- Removed modules: bdf, bzip2, cache, cid, dlg, gxvalid, lzw, otvalid, pcf, pfr, sdf, svg, tools, type1, type42, winfonts
- Config files modified: `CMakeLists.txt` (source list), `include/freetype/config/ftmodule.h` (module registration)

**GLFW 3.4** (window management):
- Bundled in full, built via CMake

## Implementation Patterns

These patterns are specific to working in this codebase:

- **FFI Wrapper Pattern**: Safe Rust wrappers around C/C++ functions in `src/core/engine/`
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
- `src/core/geometry.rs`: VAO/VBO management and instancing setup
- `src/graphics2d/shapes/shaperenderable.rs`: Main shape rendering implementation
- `src/graphics2d/shapes/mod.rs`: Shape data types (geometry only, no GPU)
- `cpp/glrenderer.cpp`: C++ wrapper functions called via FFI
- `build.rs`: CMake integration and platform-specific linking

## Supported Shape Types

Point, MultiPoint, Line, Polyline, Arc, Triangle, Rectangle, RoundedRectangle, Circle, Ellipse, Polygon, Image, Text

## Platform Notes

- Supports both Wayland and X11 on Linux (GLFW selects backend at runtime)
- OpenGL 3.3 Core Profile for macOS compatibility
- MSAA 4x multisampling enabled by default

## Project Planning

- **docs/design.md**: Architecture and key design decisions (Shape vs ShapeRenderable, dependency policy, rendering pipeline, client architecture).
- **ROADMAP.md**: Planned library enhancements (text rendering, projections, interaction, layers, trails) to support SkyTracker and other visualization applications.
- **TODO.md**: Technical debt and improvement areas including resource leaks, FFI issues, performance optimizations, and code style cleanup.
- **PRIMITIVES.md**: Graphics primitives needed for 2D visualization (maps, radar, data viz), organized by priority (critical, important, nice to have).
- **SHAPE_API_REVIEW.md**: API inconsistencies and improvement roadmap.
- **CHANGELOG.md**: Record of API changes, improvements, and known limitations.
