# Changelog

## [0.11.0] - 2026-04-12

### Breaking Changes

- **Line** normalizes its mesh so the start point sits at local (0, 0). Previously start/end were baked as absolute world coordinates, so `set_position` was an additive offset and rotation/scale pivoted around the world origin. Lines constructed with absolute coordinates and then `set_position`'d will render at different positions — migrate by constructing at origin and using `set_position` for placement.
- **Arc** now anchors at its circle center instead of the first perimeter point. `set_position(cx, cy)` places the center at `(cx, cy)`.
- **Triangle** default anchor is now its centroid. Symmetric triangles centered at origin (e.g., waypoint glyphs) are unaffected.
- **`rounded_rectangle_geometry`** and **`image_geometry`** public functions now take `(ox, oy)` offset parameters.

### Added

- **Configurable anchor system.** New `Anchor` enum (Default, Center, North, NorthEast, East, SouthEast, South, SouthWest, West, NorthWest, Custom) and `ShapeRenderableBuilder` let users control which point on a shape is used for positioning, rotation, and scaling:
  ```rust
  ShapeRenderable::builder(shape, style)
      .anchor(Anchor::Center)
      .build()
  ```
  `from_shape()` continues to work unchanged (uses `Anchor::Default`).

- **Centroid helpers** on `Line` (midpoint), `Triangle` (vertex average), `MultiPoint` (vertex average), `Polyline` (vertex average), and `Polygon` (area centroid via shoelace formula).

### Fixed

- Line, Arc, and Triangle rotation/scale now pivot around a meaningful point on the shape instead of the world origin or an arbitrary perimeter point.

### Documentation

- `SHAPE_API_REVIEW.md` Section 1 rewritten with corrected per-shape anchor table, shader-level explanation, and corrected entries for Line, Triangle, Arc, and Text.

## [0.9.0] - 2026-04-05

### Breaking Changes

- **`ShapeRenderable::from_shape()` no longer takes `(x, y)` position parameters.** Shapes default to position (0, 0). Use `set_position(x, y)` after construction. This eliminates anchor point confusion where `(x, y)` meant different things per shape type.
- **`ShapeRenderable::image()` and `image_with_size()` no longer take `(x, y)` parameters.** Same migration: call `set_position()` after construction.

### Added

- **Alpha/opacity support throughout the rendering pipeline.**
  - `Color::from_rgba(r, g, b, a)` constructor for colors with transparency.
  - `Color::from_hsl(h, s, l)` and `Color::from_hsla(h, s, l, a)` constructors for HSL color space.
  - Fragment shaders (`shape.frag`, `point.frag`) now use `vec4` for `geometryColor` uniform, enabling per-shape alpha.
  - Text shader correctly propagates alpha from color.
  - Blending was already enabled; alpha now flows end-to-end.

- **Style mutators for dynamic color changes without rebuilding shapes.**
  - `set_fill_color(color)` — change fill color at any time.
  - `set_stroke_color(color)` — change stroke color at any time.
  - `fill_color()` and `stroke_color()` getters.

- **Position getters:** `x()`, `y()`, `position()` on `ShapeRenderable`.

- **All examples converted to standalone Cargo projects** in a workspace. `cargo build --workspace` builds the library and all examples, catching API breakage.

- **New examples:**
  - `alpha_transparency` — overlapping shapes, opacity gradients, semi-transparent text.
  - `style_mutation` — dynamic color cycling, independent fill/stroke mutation, alpha breathing.

- **Project documentation:**
  - `PRIMITIVES.md` — consolidated primitives spec for maps, radar, and data visualization.
  - `SHAPE_API_REVIEW.md` — API inconsistencies and improvement roadmap.

### Removed

- `RADAR PRIMITIVES.md` — superseded by `PRIMITIVES.md`.
- `bouncing_balls_ws_client` and `bouncing_balls_ws_server` examples.
- Unused shared shaders directory (`examples/shaders/`).

## [0.8.0] - 2026-02-07

### Added

- **`App.enable_camera(camera)`** — creates a `CameraController` internally and wires all window callbacks (scroll, cursor, mouse button, resize). Eliminates `Rc<RefCell<>>` from client code.
- **`App.set_camera_smoothness(value)`** — opt-in exponential interpolation for smooth zoom/pan. Off by default; typical range 5–12.
- **`App.set_camera_zoom_sensitivity(value)`** — configure zoom speed without accessing the controller directly.
- **`CameraController.set_smoothness(value)`** / **`CameraController.update(dt)`** — animation state on the controller for smooth camera transitions.

### Changed

- **`on_render` callback now receives `Option<&Camera2D>`** as a second parameter. Camera examples get the camera directly from the callback instead of capturing shared references.
- **`App.run()` now tracks delta time** and calls `CameraController.update(dt)` each frame when a camera is enabled.

### Removed

- **`App.set_camera_controller()`** — replaced by `App.enable_camera()`.
- **`examples/camera.rs`** — obsolete `thread_local!`-based camera demo, superseded by the waypoints examples.

### Improvements

- Camera setup reduced from ~40 lines of `Rc<RefCell<>>` boilerplate to 3 lines: `enable_camera()`, `set_camera_smoothness()`, and `on_render(|renderer, camera| { ... })`.
- Smooth animation is fully backwards-compatible: when smoothness is 0 (default), all camera updates are instant and `update(dt)` is a no-op.

## [0.5.0] - 2026-02-02

### Added

- **Rectangle stroke support.** Rectangles now support three styles via `ShapeStyle`:
  - `fill(color)`: filled only (existing)
  - `stroke(color, width)`: outlined only
  - `fill_and_stroke(fill, stroke, width)`: both fill and stroke
- **`set_instance_stroke_colors()`** for per-instance stroke colors when using instanced rendering with fill+stroke shapes.
- **Reduced `MIN_STROKE_WIDTH`** from 1.5 to 1.0, allowing 1-pixel thin lines.

### Changed

- **App now owns Renderer and managed shapes internally.** Users no longer create a `Renderer` manually or write the render loop. Static scenes go from ~10 lines of wiring to a single `add_shapes()` call.

- **New `add_shape()` / `add_shapes()` methods** for adding `ShapeRenderable` instances directly to `App`. The app renders them automatically in its run loop.

- **New `on_pre_render(|shapes, renderer|)` callback** runs before managed shapes render, providing mutable access to shapes and the renderer. This enables a clear model-view separation: simulation state lives in the closure's captures, visual state lives in App's managed shapes.

- **`on_render` now receives `&Renderer`** and runs after managed shapes, serving as a post-render hook for raw `Mesh` draws or custom rendering logic.

- **Consistent run loop order:** `clear -> on_pre_render -> render managed shapes -> on_render -> swap`. Previously each example reinvented its own rendering sequence inside a closure.

- **All examples migrated** to the new API. `camera.rs` and `waypoints.rs` dropped their wrapper structs (`WorldShape`, `Waypoint`) in favor of parallel arrays (world state captured by closure, shapes owned by App).

- **README updated** with the new managed-shapes API example.

### Improvements

- Reduced boilerplate for static scenes (no manual Renderer creation, no closure needed).
- Eliminated a class of ownership bugs where users had to move both shapes and renderer into a closure.
- Clearer model-view separation in dynamic examples via `on_pre_render`.

### Known Limitations

- Managed shapes render in insertion order with `on_render` always running after them. Interleaving custom mesh draws between managed shapes requires falling back to `on_render` for everything.
- Shape access is index-based. Inserting a shape shifts all subsequent indices.
- `App` is coupled to `ShapeRenderable` directly. Trait objects (`Box<dyn Renderable>`) were considered but would have hidden shape-specific methods like `set_position()`.
- Single callback slot for `on_pre_render` and `on_render`. Multiple independent systems must share one callback each.
