# Work Log

## 2026-01-26: Camera/Projection System + Per-Instance Colors

Branch: `feat/projection`
PR: https://github.com/algonents/wilhelm-renderer/pull/6

---

### Session 1: Camera2D & Projection System

**Goal:** Add a coordinate projection system and rewrite the waypoints example to use ShapeRenderables with WGS84 projection.

#### New module: `src/core/projection.rs`

- `Projection` trait with `world_to_screen` / `screen_to_world` methods
- `IdentityProjection` — passthrough where world coords equal screen coords
- `Camera2D` — 2D camera with pan/zoom:
  - Fields: `center` (world coords), `scale` (pixels per world unit), `screen_size`
  - `pan()` / `pan_screen()` for panning in world or screen coordinates
  - `zoom()` for center-fixed zoom, `zoom_at()` for zoom-to-cursor
  - `world_bounds()` for visible world region query (frustum culling ready)
  - Implements `Projection` trait
  - 7 unit tests covering roundtrips, offset center, scale, bounds, zoom behavior
- `wgs84_to_mercator()` / `mercator_to_wgs84()` — WGS84 (lon/lat degrees) to Web Mercator (meters) conversion with f64 intermediate precision
  - 3 unit tests: roundtrip accuracy, equator origin, ordering correctness

#### Modified: `src/graphics2d/shapes/mod.rs`

- Added `Clone`/`Copy` derives to all shape types for reuse in camera-projected patterns

#### New example: `examples/camera.rs`

- Demonstrates Camera2D with scroll-to-zoom at cursor position
- Shapes defined in world coordinates, transformed to screen each frame
- Shape sizes stay constant in screen pixels (marker behavior)

#### Rewritten: `examples/waypoints.rs`

- **Before:** Custom vertex/fragment/geometry shaders doing WGS84-to-Mercator-to-NDC in GLSL, rendering GL_POINTS expanded to triangles via geometry shader
- **After:** ShapeRenderable triangles with text labels, projected via Camera2D on the CPU
- Each waypoint is a `Waypoint` struct holding Mercator position + triangle marker + text label
- WGS84 to Mercator conversion at init, Camera2D projects Mercator to screen each frame
- Mercator Y negated for screen-down convention (north appears at top)
- Auto-fit: initial scale/center computed from waypoint bounding box
- Scroll-to-zoom at cursor via `Camera2D::zoom_at()`
- Text labels (11px DejaVuSans) positioned 8px to the right of each marker

#### Exports added to `src/core/mod.rs`

- `Projection`, `IdentityProjection`, `Camera2D`, `wgs84_to_mercator`, `mercator_to_wgs84`

#### Documentation updates

- `ROADMAP.md`: Phase 2 (Coordinate System & Projection) marked complete, milestones updated
- `TODO.md`: Added rendering architecture items to Performance > Architectural section:
  - Per-frame ortho matrix redundancy
  - Camera/callback ergonomics (thread-local Cell pattern)
  - World-scaled vs screen-scaled shape distinction
  - Instancing pipeline and GPU-side projection
  - Text batching limitations
  - Transform composition / matrix stack

---

### Session 2: Per-Instance Colors

**Goal:** Extend the instancing infrastructure to support per-instance RGBA colors.

#### Approach

Separate color VBO at attribute location 2, mirroring the existing position VBO pattern at location 1. Shader fallback: when per-instance color alpha > 0, use it; otherwise fall back to `geometryColor` uniform.

#### Bug discovered and fixed

OpenGL defaults disabled vertex attributes to `(0, 0, 0, 1)`, not `(0, 0, 0, 0)`. This caused all non-instanced shapes to render as opaque black because the shader saw `vInstanceColor.a == 1.0` and used the zero-RGB instance color instead of the uniform.

**Fix:** Added `glVertexAttrib4f(2, 0, 0, 0, 0)` call in the non-instanced `draw_mesh` path to explicitly reset the default.

#### Files modified

| File | Change |
|------|--------|
| `src/core/color.rs` | Added `#[repr(C)]` for safe GPU data transfer |
| `src/core/geometry.rs` | Added `instance_color_vbo` field, `Attribute::instanced_vec4()`, `enable_instancing_color()`, `update_instance_colors()`. `enable_instancing_xy()` now allocates both VBOs. |
| `src/graphics2d/shaders/shape.vert` | Added `aInstanceColor` (location 2) input, `vInstanceColor` output |
| `src/graphics2d/shaders/shape.frag` | Added instance color fallback logic |
| `src/graphics2d/shaders/point.frag` | Same fallback logic (shares vertex shader) |
| `cpp/glrenderer.cpp` | Added `_glVertexAttrib4f` wrapper |
| `src/core/engine/opengl.rs` | Added FFI binding + `gl_vertex_attrib_4f` safe wrapper |
| `src/core/renderer.rs` | Reset instance color to `(0,0,0,0)` in non-instanced path |
| `src/graphics2d/shapes/shaperenderable.rs` | Added `set_instance_colors(&mut self, colors: &[Color])` |
| `examples/instancing.rs` | Updated with per-instance color gradient (red left-to-right, blue top-to-bottom) |

#### API usage

```rust
let mut dots = ShapeRenderable::from_shape(0.0, 0.0, ShapeKind::Circle(Circle::new(3.0)), style);
dots.create_multiple_instances(count);       // allocates position + color VBOs
dots.set_instance_positions(&positions);     // upload positions
dots.set_instance_colors(&colors);           // upload per-instance RGBA colors
dots.render(&renderer);                      // single draw call
```

#### Verification

- All 12 unit tests + 1 doc test pass
- `examples/shapes` — non-instanced shapes render with correct colors (backward compat)
- `examples/instancing` — 6,000 dots with per-instance color gradient
- `examples/waypoints` — WGS84-projected triangles with text labels
- `examples/camera` — world-coordinate shapes with zoom
