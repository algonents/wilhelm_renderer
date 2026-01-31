# TODO

Technical debt and improvement areas identified in code review.

## Resource Leaks

- [x] `shader.rs:53-57` - Delete shader objects after linking (currently commented out)
- [x] `geometry.rs:65` - Implement VAO deletion in `Drop` (currently commented out)
- [x] Add `Drop` impl for `Shader` to delete the GL program
- [x] Add `Drop` impl for `Window` to clean up GLFW resources

## Error Handling

- [ ] `shader.rs:24-37,48-51` - Re-enable shader compilation error checking (currently commented out, failures are silent)

## Bugs

- [ ] `shaperenderable.rs:144` - `points()` panics on empty input (accesses `points[0]` without checking)
- [ ] `shaperenderable.rs:181,183` - Remove duplicate assertion for polyline length
- [ ] `shaperenderable.rs:315-317` - Image loaded twice (once for dimensions, again in `image_with_size`)

## Incomplete Code

- [ ] `mesh.rs:54` - Move `set_uniform_4f` to renderer (comment says "needs to go into renderer!")
- [ ] `shaperenderable.rs:121` - Implement `from_shape` for all shape types (currently `unimplemented!()`)
- [ ] `geometry.rs:54` - Clean up "// NEW" WIP comment

## API Design

- [x] `app.rs:25` - Make clear color configurable (hardcoded to `0.07, 0.13, 0.17`)
- [x] `shaperenderable.rs:11` - Make `SCALE_FACTOR` configurable (now per-shape `scale` field)
- [ ] Rotation pivot points are hardcoded per shape type (Image/Circle/Ellipse rotate around center; Rectangle/Polygon rotate around corner/first vertex). Add configurable pivot via `ShapeRenderable` API (e.g., `set_pivot(Pivot::Center)` or `Pivot::TopLeft`). Implementation is geometry-only — shader always rotates around origin, pivot controls how vertices are generated.

## Performance

### Cost Profile (at N shapes per frame)

| Operation | Calls | Location |
|-----------|-------|----------|
| Shader switches | N | `renderer.rs:39` |
| VAO binds | N | `renderer.rs:40` |
| Uniform lookups | 5N | `renderer.rs:49,59,65,70,78` |
| Uniform sets | 5N | `renderer.rs:51,62,67,73,81` |
| Blend state setup | N (redundant) | `renderer.rs:42-43` |
| Vertex attrib reset | N (redundant) | `renderer.rs:47` |
| Projection matrix compute | N (identical) | `shaperenderable.rs:181` |
| Draw calls | N | `renderer.rs:90` |

At 1,000 shapes: ~13,000 OpenGL state changes per frame.

### Per-Frame Overhead (High Priority)

- [ ] `renderer.rs:49,59,65,70,78` - Cache uniform locations after shader compilation instead of looking up by string every draw call. Currently 5 string lookups per shape per frame.
- [ ] `renderer.rs:42-43` - Set `gl_enable(GL_BLEND)` and `gl_blend_func` once at init, not every draw call
- [ ] `renderer.rs:47` - `gl_vertex_attrib_4f(2, 0.0, 0.0, 0.0, 0.0)` called per draw call to reset instance color attribute. Should be set once before the render loop.
- [ ] `shaperenderable.rs:86` - Use cached window size from `InnerWindow` instead of calling `gl_get_integerv` every frame
- [x] `renderer.rs:43,81` - Remove unnecessary VAO unbind between consecutive draws

### Architectural (Medium Priority)

- [ ] Implement draw call batching for rendering many shapes of the same type
- [ ] Sort draws by shader to minimize shader switches
- [ ] No frustum culling — off-screen shapes go through the full draw pipeline (shader switch, VAO bind, uniform sets, draw call). Add viewport bounds check before issuing draw calls.
- [ ] Geometry duplication — identical shapes (e.g., two circles with the same radius) create separate VAOs/VBOs with identical vertex data. Add a geometry cache keyed by shape type + parameters to share VAOs across identical shapes.
- [ ] Instancing infrastructure exists (`Geometry::enable_instancing_xy`) but is not integrated into default shape creation. `from_shape()` always creates single-instance shapes. Users must manually call `create_multiple_instances()`. Consider automatic batching of identical shapes via instancing.
- [ ] `shaperenderable.rs:283,290` - Scale circle/ellipse segment count based on radius/screen size (currently hardcoded 100/64)
- [ ] `shaperenderable.rs:181` - `ortho_2d()` recomputes the same orthographic matrix for every shape every frame. Cache and reuse when window size hasn't changed.
- [ ] `shaperenderable.rs:182-183` - `set_transform` and `set_scale` called per shape per frame with identical values. Could be set once per frame.
- [ ] Camera state uses thread-local `Cell` in examples because `on_scroll` and `on_render` are separate closures that can't share mutable references. Consider passing a context/state struct into callbacks, or an event queue pattern.
- [ ] No built-in convention for shapes that scale with zoom (e.g., airspace boundaries, range rings) vs shapes that stay fixed in screen pixels (e.g., markers, labels). `set_scale()` exists but must be manually driven from camera state each frame. Consider a `ScaleMode::World` / `ScaleMode::Screen` enum on `ShapeRenderable`.
- [ ] Instanced draw path (`renderer.rs:112-115`) hardcodes `u_screen_offset = (0,0)` and uses vertex attributes for positions. To batch camera-projected shapes via instancing, CPU must pre-project all positions into the instance buffer each frame. An alternative: pass camera view-projection matrix as a uniform and let the GPU project world coordinates directly. Would require shader changes.
- [ ] Text labels have unique geometry per string — cannot use the current instancing path. Batching text requires a different strategy (shared glyph quad geometry with per-instance UV offsets, or a glyph-level instancing approach).
- [ ] No matrix stack or hierarchical transforms. Each shape has an isolated transform (projection + offset). Adding layers, groups, or parent-child transform relationships would require rethinking how `u_Transform` is composed.

### Memory (Low Priority)

- [ ] `geometry.rs:196-197` - Use single `gl_buffer_data` with data instead of `gl_buffer_data_empty` + `gl_buffer_sub_data`

## FFI Layer (C++)

### Bugs

- [x] `glrenderer.cpp:30-35` - Window creation continues after failure (missing `return nullptr`), will crash on `glfwMakeContextCurrent(nullptr)`
- [x] `glrenderer.cpp:40-41` - GLAD initialized twice, first call result ignored

### Design Issues

- [ ] `glrenderer.cpp:113-118` - `_glClearColor` also calls `glClear()` - surprising hidden side effect, should be separate functions
- [ ] `glrenderer.cpp:216` - Debug print on every texture upload, should be `#ifndef NDEBUG` guarded

### Missing Wrappers

- [x] Add `_glDeleteVertexArray` wrapper (needed for VAO cleanup)
- [x] Add `_glDeleteShader` wrapper (needed for shader cleanup)
- [x] Add `_glDeleteProgram` wrapper (needed for program cleanup)
- [x] Add `_glGetShaderiv` wrapper (for error reporting to Rust side)
- [ ] Add `_glGetShaderInfoLog` wrapper (for shader compilation error messages)
- [ ] Add `_glGetProgramiv` / `_glGetProgramInfoLog` wrappers (for link error reporting)

## Wayland / HiDPI Scaling

### The Problem

On HiDPI displays (common on Wayland), window coordinates and framebuffer coordinates differ:

| Concept | X11 (no scaling) | Wayland @ 2x scale |
|---------|------------------|-------------------|
| Window size | 800×600 | 800×600 (logical) |
| Framebuffer size | 800×600 | 1600×1200 (physical) |
| Mouse coordinates | 0-800, 0-600 | 0-800, 0-600 (logical) |
| OpenGL viewport | 800×600 | 1600×1200 |

Mouse coordinates are in **window/logical** space, but rendering is in **framebuffer/physical** space. Mouse-to-world mapping will be off by the scale factor without correction.

### Current State

- Rendering correctly uses framebuffer size (via `glfwSetFramebufferSizeCallback` and `glfwGetFramebufferSize`)
- Mouse coordinates come in window/logical space (uncorrected)

### Fixed

- [x] Bug: `shaperenderable.rs:86-87` uses framebuffer size for orthographic projection, but shapes are positioned in logical coordinates. On scaled displays, shapes appear at wrong positions/sizes.
  - **Fix**: Use logical window size directly from `WindowHandle` instead of framebuffer size. `Renderer` now takes `WindowHandle` and uses `window_handle.size()` for orthographic projection.
- [x] Add `_glfwGetWindowContentScale` wrapper to query current scale factor
- [x] Expose content scale to Rust `Window` via `Window::content_scale()`

### Additional Wrappers (Lower Priority)

- [ ] Add `_glfwSetWindowContentScaleCallback` wrapper to detect scale changes (window moved between monitors)
- [ ] Document or provide helper for mouse-to-world coordinate conversion

## FFI Layer (Rust)

### Bugs

- [x] `opengl.rs:261-268` - `gl_buffer_sub_data_vec2` assumes `(f32, f32)` is tightly packed (potential UB), use `#[repr(C)]` struct instead
- [x] `opengl.rs:298` - Typo: `instance_cout` should be `instance_count`
- [x] `glfw.rs:69` - `glfw_set_window_user_pointer` takes `*const c_void` but should be `*mut c_void`

### Performance

- [x] `opengl.rs:315-317` - `gl_get_uniform_location` allocates CString on every call, contributes to per-frame heap allocations

### Dead Code

- [ ] `glfw.rs:38` - `_glfwSetFramebufferSizeCallback` declared but never used

### API Consistency

- [ ] `opengl.rs:189` - `gl_gen_buffers` takes `&mut Vec<GLuint>` instead of `&mut [GLuint]` (unusual API)

## Code Style

- [ ] Run `rustfmt` to fix inconsistent spacing (`zoom_level:1.0` vs `zoom_level: 1.0`, return arrows, etc.)
- [ ] Run `cargo clippy` and address warnings
- [ ] Clean up unused imports (`PI` imported but `TAU` used)
- [ ] Add doc comments to public API functions
