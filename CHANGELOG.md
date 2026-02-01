# Changelog

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
