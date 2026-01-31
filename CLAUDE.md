# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

wilhelm-renderer is a minimalist 2D graphics engine written in Rust with native OpenGL bindings. It provides rendering of 2D shapes and visualization of 2-dimensional data in real time. The project uses OpenGL 3.3 Core Profile with GLFW 3.4 (bundled) for window management.

## Build Commands

```bash
# Build the library
cargo build

# Run examples
cargo run --example shapes
cargo run --example shapes_with_zoom
cargo run --example triangle
cargo run --example instancing

# Run standalone example projects
cd examples/bouncing_balls && cargo run
```

### Build Requirements

- C/C++ compiler and CMake (cmake crate invokes CMake during build)
- Linux: `libgl1-mesa-dev`, `libwayland-dev`, `libxkbcommon-dev`
- GLFW 3.4 is bundled, no external dependency needed

## Architecture

### Three-Layer Design

1. **FFI Layer** (`src/core/engine/`)
   - `opengl.rs`: Raw OpenGL function bindings
   - `glfw.rs`: GLFW window and input wrappers
   - C++ implementation in `cpp/glrenderer.cpp` provides the actual wrappers that link to GLFW and OpenGL

2. **Core Rendering Engine** (`src/core/`)
   - `app.rs`: Main application loop with render callback
   - `renderer.rs`: Mesh drawing, viewport management, zoom
   - `window.rs`: GLFW window creation, event callbacks (resize, scroll, cursor)
   - `geometry.rs`: VAO/VBO management, vertex attributes, instancing support
   - `mesh.rs`: Combines geometry + shader + transform + color/texture
   - `shader.rs`: GLSL compilation wrapper
   - `texture.rs` / `image.rs`: Image loading and GPU texture management

3. **Graphics2D API** (`src/graphics2d/`)
   - `shapes/shaperenderable.rs`: High-level shape rendering (line, polyline, circle, rectangle, polygon, arc, image, etc.)
   - Uses lazy-loaded singleton shaders via thread_local OnceCell
   - Orthographic projection with zoom support

### Key Patterns

- **FFI Wrapper Pattern**: Safe Rust wrappers around C/C++ functions in `src/core/engine/`
- **Interior Mutability**: Window uses `Rc<Cell<>>` for shared state across callbacks
- **Component-Based Meshes**: Mesh = Geometry + Shader + Transform
- **Callback-Driven App Loop**: App uses closures for render logic
- **Box<Window> for FFI Stability**: `Window::new()` returns `Box<Window>` and `App` stores `Box<Window>` because GLFW callbacks receive a raw pointer to the Window (via `glfw_set_window_user_pointer`). The Box ensures a stable heap address that won't invalidate when App is moved.

### Performance Architecture

The current architecture supports scaling to 10,000+ shapes through evolutionary changes:

**Existing Foundation:**
- Direct FFI access to OpenGL instancing (`glVertexAttribDivisor`, `glDrawArraysInstanced`)
- Working instancing in `Geometry` (`enable_instancing_xy`, `update_instance_xy`) - proven with 1000+ shapes
- Singleton shaders via OnceCell - shapes share shaders, minimizing shader switches
- Simple VAO/VBO abstraction without deep hierarchies blocking optimization

**Current Limitation:**
The high-level `ShapeRenderable` API uses 1 draw call per shape, which becomes a CPU bottleneck at high counts. This limitation is in the convenience API, not the core engine.

**Escape Hatch for Power Users:**
`App::on_render()` provides direct `Renderer` access, allowing clients to bypass per-shape rendering and implement custom batching when needed. The existing instancing infrastructure (`Geometry::enable_instancing_xy`, `update_instance_xy`, `update_instance_colors`) is fully functional for manual batching.

**Scaling Strategy (additive, not rewrite):**

1. **Automatic Batching in App::run()** (preferred approach): Group shapes by type and render with instancing. Requires:
   - Normalized (unit) prototype geometries: unit circle (r=1), unit rectangle (1x1)
   - Per-instance scale attribute (location 3 in shader) to transform unit geometry to actual size
   - Grouping logic in `App::run()` to collect (position, scale, color) per shape type
   - Batchable: Circle, Ellipse, Rectangle, Point
   - Non-batchable (unique geometry): Line, Polyline, Polygon, Arc, Text, Image

2. **Extended Instancing**: The shader (`shape.vert`) already supports per-instance position (loc 1) and color (loc 2). Adding per-instance scale (loc 3) is mechanical.

3. **Frustum Culling**: CPU-side viewport bounds check before batching, with optional spatial index (quadtree).

No architectural blockers exist. The path from current state to high-performance rendering is incremental.

### C++ FFI Build

`build.rs` uses CMake to compile the C++ layer (`cpp/`). Platform-specific linking:
- Linux: Statically links glrenderer, glfw3; dynamically links GL
- macOS: Links Cocoa, CoreFoundation, IOKit, CoreVideo frameworks
- Windows: Links opengl32, gdi32, user32, shell32

### Bundled Dependencies

**FreeType 2.13.2** (text rendering):
- Minimal build: Only TrueType/OpenType support with gzip for WOFF web fonts
- Removed modules: bdf, bzip2, cache, cid, dlg, gxvalid, lzw, otvalid, pcf, pfr, sdf, svg, tools, type1, type42, winfonts
- Removed: docs, tests, build system files, VS/Mac project files
- Config files modified: `CMakeLists.txt` (source list), `include/freetype/config/ftmodule.h` (module registration)
- This reduces crates.io package size from 3.2MB to 2.9MB compressed

**GLFW 3.4** (window management):
- Bundled in full, built via CMake

## Design Decisions

### No Scene Graph / Composite Shapes

The library uses a flat rendering model rather than hierarchical scene graphs. Reasons:

- **Composites work against batching**: If 500 aircraft each "contain" their shapes (symbol + vector + label), you either render per-aircraft (1500 draw calls) or extract shapes to batch anyway
- **Grouping is a domain concept**: "These shapes belong to the same aircraft" is application knowledge, not rendering knowledge
- **Transform propagation is trivial**: A client struct can compute child positions without library support

Logical grouping belongs in client code (e.g., `AircraftTrack` in SkyTracker), not the rendering library. The library focuses on efficiently rendering flat collections of shapes, which enables batching by shape type.

### Performance: Optimize on Demand

Performance optimizations are driven by actual bottlenecks discovered in client code, not speculation. The simple per-shape API is preferred until profiling proves it insufficient. The escape hatch (`App::on_render()` with direct instancing) exists for power users who hit limits before library-level batching is added.

## Key Files

- `src/lib.rs`: Library root, exports `core` and `graphics2d` modules
- `src/core/geometry.rs`: VAO/VBO management and instancing setup
- `src/graphics2d/shapes/shaperenderable.rs`: Main shape rendering implementation (~800 lines)
- `cpp/glrenderer.cpp`: C++ wrapper functions called via FFI
- `build.rs`: CMake integration and platform-specific linking

## Supported Shape Types

Point, MultiPoint, Line, Polyline, Arc, Triangle, Rectangle, RoundedRectangle, Circle, Ellipse, Polygon, Image

## Rotations

Shapes support per-shape rotation via `set_rotation(angle)` where angle is in radians. The rotation is applied in the vertex shader before scaling and translation (rotate → scale → translate).

**Rotation pivot points by shape type:**

| Shape | Geometry Origin | Rotation Pivot | Position Refers To |
|-------|-----------------|----------------|-------------------|
| Image | Centered | Center | Center |
| Circle | Centered | Center | Center |
| Ellipse | Centered | Center | Center |
| Rectangle | (0,0) corner | Top-left | Top-left |
| RoundedRectangle | (0,0) corner | Top-left | Top-left |
| Triangle | User-defined | Depends on vertices | First vertex |
| Polygon | Anchored to first vertex | First vertex | First vertex |
| Polyline | Anchored to first vertex | First vertex | First vertex |
| Line | Anchored to start point | Start point | Start point |

**For center rotation with Rectangle/Polygon:** Use the low-level `Mesh` API with geometry vertices centered at origin, or define Triangle vertices centered at origin.

**Transform order in shaders:** `rotate(u_rotation)` → `scale(u_scale)` → `translate(u_screen_offset + aInstanceXY)` → `project(u_Transform)`

## Platform Notes

- Supports both Wayland and X11 on Linux (GLFW selects backend at runtime)
- OpenGL 3.3 Core Profile for macOS compatibility
- MSAA 4x multisampling enabled by default

## Project Planning

- **ROADMAP.md**: Planned library enhancements (text rendering, projections, interaction, layers, trails) to support SkyTracker and other visualization applications.
- **TODO.md**: Technical debt and improvement areas including resource leaks, FFI issues, performance optimizations, and code style cleanup.
- **RADAR PRIMITIVES.md**: Graphics primitives needed for ATM radar visualization, organized by priority (critical, important, nice to have).
- **CHANGELOG.md**: Record of API changes, improvements, and known limitations.
