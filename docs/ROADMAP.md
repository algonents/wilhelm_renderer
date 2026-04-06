# Roadmap: wilhelm-renderer Library Enhancements

Features to add to the wilhelm-renderer library to support interactive 2D visualization applications (including **SkyTracker**, the ATM radar visualization application).

> **Note**: Domain-specific ATM features (aircraft symbols, track management, airspace boundaries, etc.) belong in the separate closed-source **SkyTracker** repository.

---

## Phase 1: Text Rendering ✓ Complete

- [x] Integrate font rasterization (FreeType FFI - encapsulated wrapper)
- [x] Font atlas with on-demand glyph caching (lazy loading)
- [x] Store glyph metrics and UV coordinates
- [x] Create text shader (instanced quads with texture sampling)
- [x] Implement `Text` struct with API: `Text::new(x, y, "label", font_size, color)`
- [x] Support text anchoring (left, center, right)
- [ ] Batch multiple text draws into single draw call (future optimization)

## Phase 2: Coordinate System & Projection ✓ Complete

### Foundation ✓ Complete
- [x] Renderer is stateless (no zoom/viewport state)
- [x] ShapeRenderable works in screen/pixel coordinates
- [x] Per-shape `scale` property for intrinsic size adjustment (scales around shape center)
- [x] Simple orthographic projection: screen coordinates → NDC

### Projection System ✓ Complete
- [x] Define `Projection` trait for world-to-screen coordinate transforms
- [x] Implement `IdentityProjection` (screen coordinates passthrough)
- [x] `Camera2D` struct with center, scale (pixels per world unit), screen size
- [x] `world_to_screen` and `screen_to_world` conversion
- [x] Pan (world and screen delta), zoom (center and zoom-at-point)
- [x] `world_bounds()` for visible region query (frustum culling ready)
- [x] Unit tests for projection accuracy (7 tests: roundtrip, offset, scale, bounds, zoom)

### Geographic Projection ✓ Complete
- [x] `wgs84_to_mercator` / `mercator_to_wgs84` coordinate conversion (f64 intermediate precision)
- [x] Unit tests for Mercator conversion (roundtrip, origin, ordering)
- [x] Waypoints example rewritten: ShapeRenderable triangles + text labels, Camera2D projection from WGS84
- [x] Camera example: zoom-to-cursor with shapes in world coordinates

### Remaining
- [ ] Mouse drag to pan (currently zoom-only in examples)
- [ ] Batch multiple text draws into single draw call (from Phase 1)

> **SkyTracker**: Stereographic projection, lat/lon viewport, nautical mile units

## Phase 2.5: Shape Stroke Support

Add stroke (outline) rendering to filled shapes. Currently, shapes are either filled or stroked (lines, polylines, arcs). Goal: support fill-only, stroke-only, and fill+stroke for applicable shapes.

### Completed
- [x] Rectangle: fill, stroke, fill+stroke (uses polyline geometry for stroke)
- [x] Instancing support for fill+stroke (separate stroke colors via `set_instance_stroke_colors`)

### Remaining
- [ ] Circle: stroke and fill+stroke
- [ ] Ellipse: stroke and fill+stroke
- [ ] Polygon: stroke and fill+stroke
- [ ] RoundedRectangle: stroke and fill+stroke
- [ ] Triangle: stroke and fill+stroke

> **Note**: Line, Polyline, and Arc are inherently stroke-based shapes.

## Phase 3: Interaction

### Picking/Selection
- [ ] Screen-to-world coordinate conversion using projection
- [ ] Spatial index for efficient hit testing (grid or quadtree)
- [ ] Click to select/deselect entities
- [ ] Multi-select support (shift+click or box select)
- [ ] Selection callback/event system

### Pan/Zoom Controls
- [ ] Mouse drag to pan (update viewport center)
- [x] Scroll wheel to zoom (Camera2D zoom_at with cursor position)
- [ ] Keyboard shortcuts (arrow keys for pan, +/- for zoom)
- [ ] Zoom-to-fit selected entities
- [x] Min/max zoom limits (clamp in examples)

## Phase 4: Layer System

- [ ] `Layer` struct with z-order, visibility, opacity
- [ ] Layer visibility toggles
- [ ] Per-layer rendering with proper depth ordering
- [ ] Layer-based draw call batching

> **SkyTracker**: Predefined layers (background, map, routes, aircraft, labels, selection)

## Phase 5: Trail Rendering

- [ ] Circular buffer storing N past positions per entity
- [ ] Configurable trail length and decay (fade older positions)
- [ ] Efficient rendering via instancing or line strips

## Phase 6: Performance & Scalability

### Current Limitations
The current architecture uses 1 draw call per shape, which becomes a CPU bottleneck at high entity counts. For 10,000+ shapes, architectural changes are needed.

### Instancing Enhancements (High Priority)
- [x] Per-instance position attribute (vec2)
- [x] Per-instance color attribute (vec4)
- [x] Per-shape scale via `u_scale` uniform (shared across instances)
- Remaining items covered in Draw Call Batching → Strategy A below

### Draw Call Batching

Two complementary strategies, both additive to the existing per-shape path:

**Strategy A: Extended instancing (identical shapes)**

For N shapes with the same geometry (e.g., 1,000 circles), one VAO/VBO with per-instance attributes. Already works for position and color. Extend with:

- [ ] Per-instance scale attribute
- [ ] Per-instance rotation attribute
- [ ] Generic `InstancedShape` API supporting position + rotation + color + scale

**Strategy B: Dynamic geometry batching (mixed shapes per shader)**

For heterogeneous shapes sharing a shader, merge pre-transformed vertices into a single VBO per shader type. Reduces N draw calls to ~4 (one per shader: default, point, text, image).

- [ ] Add per-vertex color attribute to shape shader (currently color is a uniform — one draw call = one color)
- [ ] Convert TRIANGLE_FAN/STRIP shapes to plain TRIANGLES for merging (drawing modes differ per shape type)
- [ ] Retain CPU-side vertex data in `Geometry` or expose geometry-building functions (currently `add_buffer()` uploads to GPU and discards the data)
- [ ] Implement `BatchRenderer` that collects shapes per shader, merges transformed vertices, issues one draw call per shader
- [ ] Separate batches for static vs dynamic geometry (static: upload once; dynamic: rebuild per frame)

**Architecture:**

```
BatchRenderer
  └─ batches: HashMap<ShaderType, Batch>
       └─ Batch
            ├─ shader: Rc<Shader>
            ├─ geometry: Geometry (single merged VBO)
            └─ vertices: Vec<f32> (CPU-side, rebuilt when dirty)
```

**Implementation order:**

1. Per-vertex color in the shader (prerequisite for both strategies)
2. `BatchRenderer` for same-shader shapes (the big win: N draw calls → ~4)
3. Auto-grouping: sort shapes by shader, batch automatically

`BatchRenderer` coexists with `ShapeRenderable` — opt-in for performance-critical paths. The existing per-shape rendering stays as-is for simple cases.

### Render State Optimization
- [ ] Cache uniform locations after shader compilation (from TODO.md)
- [ ] Set GL state (blend, depth) once at init, not per draw (from TODO.md)
- [x] Minimize VAO binds between draws (removed unnecessary unbind calls)
- [ ] Sort draws by shader to reduce shader switches

### Culling
- [ ] Viewport frustum culling (skip shapes outside visible area)
- [ ] Spatial index (quadtree or grid) for efficient culling and picking

### Performance Targets
| Scenario | Target |
|----------|--------|
| 10,000 static shapes | <2ms frame time |
| 1,000 dynamic shapes @ 4Hz | <16ms frame time |
| Pan/zoom responsiveness | <16ms frame time |

### Stability
- [x] Resource cleanup on shutdown (Drop impls added)
- [ ] Error handling improvements (from TODO.md)
- [ ] Memory leak detection/validation

### Utilities
- [ ] Distance measuring tool (generic, pixel/world units)
- [ ] Screenshot/export capability
- [ ] Compass rose rendering

---

## Dependencies

External dependencies:
- FreeType library - font rasterization via encapsulated FFI wrapper
- `image` - already used, for font atlas and image loading

## Milestones

| Milestone | Deliverable | Status |
|-----------|-------------|--------|
| M1 | Text rendering working | ✓ Complete |
| M2 | Projection trait + Camera2D + Mercator | ✓ Complete |
| M3 | Picking/selection, pan/zoom controls | Pending |
| M4 | Layer system | Pending |
| M5 | Trail rendering, performance validated | Pending |

---

## Out of Scope (SkyTracker Repository)

The following features belong in the closed-source SkyTracker application:

### Radar-Specific Projections
- Stereographic projection implementation
- Viewport with lat/lon center and nautical mile range

### Aircraft Visualization
- Aircraft symbol (rotatable icon with heading)
- Velocity vector line
- Label block (callsign, altitude, speed)
- Selection state highlighting
- Coasting indicator (stale track)

### Aviation Map Elements
- Range rings with NM labels
- Waypoints with labels
- Airways/routes as labeled polylines
- Sector/airspace boundaries

### Track Management
- `TrackManager` for create/update/delete operations
- Track ID mapping
- Batch position updates
- Update rate handling (1-4 Hz)
- Track timeout/deletion

### ATM-Specific Features
- Conflict visualization (aircraft pair connecting lines)
- Altitude/speed/heading filters
- Aviation-specific distance measuring (NM)

### SkyTracker Success Criteria
- Render 500+ aircraft tracks at 4 Hz update rate
- Text labels readable at all zoom levels
- Pan/zoom responsive (<16ms frame time)
- Select track by clicking
- Show/hide layers independently
