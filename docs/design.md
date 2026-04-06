# Design

## Positioning

wilhelm_renderer is a GPU-accelerated 2D display engine for real-time operational visualization. The primary use case is the SkyTracker ATM radar display (Controller Working Position), but the library is general-purpose for any application rendering real-time 2D positional data with geographic projection.

Its peers are Qt Graphics View, Cairo/GDK, and OpenSceneGraph 2D — but leaner, Rust-native, and purpose-built for streaming positional data.

## Dependency Policy

The library targets certification environments. Dependencies must be minimal and auditable.

**Current dependencies:** `glam` (math), `image` (image loading)
**Bundled:** GLFW 3.4 (window management), FreeType 2.13.2 (text rendering)

Do not add external crates for computational geometry, data processing, or other functionality that can be implemented directly. Every new dependency increases the certification surface.

## Design Principles

1. **Easy client API**: Minimize ceremony and boilerplate for common operations. Hide internal complexity (e.g., `Rc<RefCell<>>`) behind simple methods.
2. **Prevent unnecessary copies**: Prefer references over cloning. Only copy small, `Copy` types (e.g., `Camera2D`, `Vec2`, `Color`).
3. **Optimized for performance**: Minimize per-frame overhead. Avoid allocations in hot paths. Prefer batched operations over per-item work.
4. **Optimize on demand**: Performance optimizations are driven by actual bottlenecks, not speculation. The simple per-shape API is preferred until profiling proves it insufficient.

## Three-Layer Design

```
┌─────────────────────────────────────────────┐
│  Graphics2D API                             │
│  Shape types + ShapeRenderable              │
├─────────────────────────────────────────────┤
│  Core Rendering Engine                      │
│  App, Renderer, Mesh, Geometry, Shader      │
├─────────────────────────────────────────────┤
│  FFI Layer                                  │
│  OpenGL bindings, GLFW wrappers (C++)       │
└─────────────────────────────────────────────┘
```

1. **FFI Layer** (`src/core/engine/`) — Raw OpenGL and GLFW bindings via C++ wrappers in `cpp/glrenderer.cpp`.

2. **Core Rendering Engine** (`src/core/`) — App loop, mesh drawing, VAO/VBO management, shader compilation, texture loading. This layer knows about GPU resources but not about shapes.

3. **Graphics2D API** (`src/graphics2d/`) — Shape types, ShapeRenderable, shaders. This layer provides the user-facing API.

## Shape vs ShapeRenderable

This separation is a core architectural decision.

### Shape (`src/graphics2d/shapes/mod.rs`)

Pure geometry data types: `Polygon`, `Circle`, `Line`, `Rectangle`, etc. No GPU, no rendering, no dependencies on the core engine.

**Responsibilities:**
- Store geometry data (points, radii, dimensions)
- Computational geometry methods (intersection, union, difference)
- Hit testing (contains_point, distance_to_point)
- Geometric queries (area, perimeter, bounding box)

**Design rules:**
- No `use crate::core::*` — shapes must not depend on the rendering layer
- All methods are pure math — testable without a GPU context
- Shapes are cheap to clone and pass around

### ShapeRenderable (`src/graphics2d/shapes/shaperenderable.rs`)

Wraps a Shape and adds everything needed to render it on screen.

**Responsibilities:**
- Convert Shape geometry into GPU buffers (VAO/VBO)
- Store rendering state: position, scale, rotation, z-order, color, style
- Manage fill mesh and optional stroke mesh
- Interface with the Renderer for draw calls

**Relationship:**
```
ShapeRenderable
  ├── position (x, y)
  ├── scale, rotation, z_order
  ├── mesh (Mesh — GPU buffers + shader + color)
  ├── stroke_mesh (Option<Mesh>)
  └── shape (ShapeKind — the underlying Shape data)
```

### Why this separation matters

1. **Hit testing** happens at the Shape level. When a user clicks on the screen, you test against Shape geometry — no GPU involved.

2. **Computational geometry** (polygon intersection, clipping) operates on Shapes and produces new Shapes. The results are then wrapped in ShapeRenderable for display.

3. **Testability.** Shape methods can be unit tested without an OpenGL context.

4. **Certification.** The computational geometry code is pure Rust math with no external dependencies, making it auditable.

## Flat Rendering Model

The library uses flat shape collections rather than hierarchical scene graphs.

**Why:**
- Composites work against batching — extracting shapes to batch is better than rendering per-group
- Grouping is a domain concept (e.g., "these shapes belong to one aircraft") that belongs in client code
- Transform propagation is trivial for clients to implement

Logical grouping belongs in the client application (e.g., `Track` in SkyTracker), not the rendering library.

## Rendering Pipeline

```
ShapeRenderable.render()
  → mesh.set_screen_offset(x, y)
  → mesh.set_scale(scale)
  → mesh.set_rotation(rotation)
  → renderer.draw_mesh(&mesh)
      → bind VAO
      → set uniforms (transform, offset, scale, rotation, color)
      → glDrawArrays
```

**Z-ordering:** Shapes are sorted by `z_order` (stable sort) before rendering each frame.

**Color pipeline:** `Color { r, g, b, a }` → `mesh.color` → `geometryColor` uniform (vec4) → fragment shader.

**Transform order in shaders:** `rotate(u_rotation)` → `scale(u_scale)` → `translate(u_screen_offset + aInstanceXY)` → `project(u_Transform)`

## Instancing

Shapes can be instanced via `create_multiple_instances()`. Per-instance position and color are sent as vertex attributes. The `u_screen_offset` uniform is forced to (0, 0) in instanced mode.

**Attribute locations used by the shape shader:**

| Location | Name | Type | Usage |
|----------|------|------|-------|
| 0 | `aPos` | vec2 | Mesh-local vertex position |
| 1 | `aInstanceXY` | vec2 | Per-instance screen offset (divisor=1) |
| 2 | `aInstanceColor` | vec4 | Per-instance RGBA color (divisor=1) |

**Instance color fallback:** The fragment shader checks `vInstanceColor.a > 0.0` to decide whether to use the per-instance color or fall back to the `geometryColor` uniform. When attrib 2 is disabled, OpenGL reads the generic value `(0,0,0,0)` (reset by `gl_vertex_attrib_4f` before each draw call), so alpha is 0 and the shader uses the uniform color.

**Critical invariant:** Attrib 2 must only be enabled when color data is provided. `enable_instancing_xy` (position-only) must NOT enable attrib 2, or OpenGL reads garbage data causing random color bleeding. The color buffer is lazily initialized on first `update_instance_colors` call.

## Rotations

Shapes support per-shape rotation via `set_rotation(angle)` where angle is in radians.

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

## Client Architecture

Features belong in different places depending on their nature:

| Feature | Where |
|---------|-------|
| Drawing shapes | wilhelm_renderer (ShapeRenderable) |
| Geometric computation (intersection, hit test) | wilhelm_renderer (Shape types) |
| Domain logic (track management, airspace rules) | Client (SkyTracker) |
| Data-to-visual mapping (scales, palettes) | Companion crates (future) |
| Geographic projection | wilhelm_renderer (Camera2D, Mercator) |

## Companion Crates (Future)

Higher-level D3-style functionality will live in separate optional crates:
- `wilhelm_scales` — data domain to visual range mapping
- `wilhelm_geo` — additional geographic projections
- `wilhelm_viz` — axes, legends, color palettes
- `wilhelm_data` — data-to-shape bindings
