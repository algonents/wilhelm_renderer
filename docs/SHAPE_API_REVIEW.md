# ShapeRenderable API Review

A comprehensive analysis of inconsistencies, unintuitive behaviors, and improvement opportunities in the ShapeRenderable API.

---

## 1. Anchor Points

### How anchors work at the shader level

`shape.vert` applies the transform:

```glsl
rotated = rotate(aPos, u_rotation);
p       = rotated * u_scale + u_screen_offset + aInstanceXY;
```

Consequences:

- **Anchor, rotation pivot, and scale origin are always the same point** — wherever `(0, 0)` sits in each shape's *local* vertex data.
- `set_position(x, y)` writes `u_screen_offset` only; it does not move geometry relative to the shape.
- The behavior of every shape is therefore determined by one question: *where is (0, 0) in the mesh it builds?*

### Default anchors per shape

| Shape | Default Anchor | Rotation Pivot | Scale Origin |
|-------|---------------|----------------|--------------|
| Point | The point itself | N/A | N/A |
| MultiPoint | First point | First point | First point |
| Line | Start point | Start point | Start point |
| Polyline | First point | First point | First point |
| Polygon | First vertex | First vertex | First vertex |
| Triangle | Centroid | Centroid | Centroid |
| Circle | Center | Center | Center |
| Ellipse | Center | Center | Center |
| Image | Center | Center | Center |
| Arc | Circle center | Circle center | Circle center |
| Rectangle | Top-left corner (Y-down screen space) | Same | Same |
| RoundedRectangle | Same as Rectangle | Same | Same |
| Text | Top-left of text cell | N/A | N/A |

### Configurable anchors (v0.11.0)

Users can override the default anchor via the builder API:

```rust
ShapeRenderable::builder(shape, style)
    .anchor(Anchor::Center)
    .build()
```

Available anchors: `Default`, `Center`, `TopLeft`, `TopRight`, `BottomLeft`, `BottomRight`, `Top`, `Bottom`, `Left`, `Right`, `Custom(f32, f32)`.

All variants resolve against the shape's axis-aligned bounding box. `Custom(x, y)` specifies an arbitrary point in local coordinates. `Default` preserves the per-shape natural anchor listed above.

### Remaining inconsistency

Closed shapes still split two ways on their *default*: Circle/Ellipse/Image/Arc default to center; Rectangle/RoundedRectangle default to top-left. This is intentional — both conventions are common and the `Anchor` enum lets users override.

### Centroid helpers

`centroid()` methods are available on `Line` (midpoint), `Triangle` (vertex average), `MultiPoint` (vertex average), `Polyline` (vertex average), and `Polygon` (area centroid via shoelace formula). These can be used with `Anchor::Custom(centroid.0, centroid.1)`.

---

## 2. Redundant Position for Vertex-Defined Shapes

*Resolved in v0.9.0.* The `(x, y)` position parameter was removed from `from_shape()` and `image()` / `image_with_size()`. All shapes default to position (0, 0) and are placed via `set_position()`.

For vertex-defined shapes (Line, Polyline, Polygon, MultiPoint), the constructor normalizes geometry to the default anchor and stores the anchor's input-space position in `s.x/s.y`, so the shape renders in place without requiring `set_position()`. Users who want a different placement call `set_position()` explicitly.

---

## 3. Style Mutation

*Resolved in v0.9.0.* Style mutators added:
- `set_fill_color(color)` — change fill color at any time
- `set_stroke_color(color)` — change stroke color at any time
- `fill_color()` and `stroke_color()` getters

`set_stroke_width()` is not supported because stroke width affects geometry (requires a rebuild).

---

## 4. Position Getters

*Resolved in v0.9.0.* Added `x()`, `y()`, and `position() -> (f32, f32)` getters.

---

## 5. Stroke Handling Inconsistencies

### 5.1 Stroke Width Clamping
- Line geometry: uses `stroke_width.max(MIN_STROKE_WIDTH)` constant
- Polyline geometry: uses `stroke_width.max(1.0)` — hardcoded, not using the constant

**Status:** Open. Use `MIN_STROKE_WIDTH` constant consistently.

### 5.2 Stroke Support Varies by Shape
- **Rectangle**: fill, stroke, fill+stroke — all supported
- **Circle, Ellipse, Polygon, RoundedRectangle, Triangle**: fill only (stroke noted as TODO in ROADMAP.md)
- **Line, Polyline, Arc**: stroke only (inherently stroke-based)

**Status:** Open. Continue stroke rollout per ROADMAP.

### 5.3 No Style for (None, None)
`ShapeStyle` with `fill: None, stroke_color: None` silently defaults to white fill. No way to create an invisible shape.

**Status:** Open.

---

## 6. Color and Alpha

*Resolved in v0.9.0.* `Color::from_rgba()`, `Color::from_hsl()`, and `Color::from_hsla()` added. Alpha propagates through the full rendering pipeline (uniforms, per-instance attributes, fragment shaders).

---

## 7. Instancing Asymmetries

### 7.1 Position Ignored in Instanced Mode
When `instance_count > 0`, the shape's `(self.x, self.y)` is ignored — positions come entirely from `set_instance_positions()`. This is a silent contract.

### 7.2 No Per-Instance Scale or Rotation
All instances share the same `u_scale` and `u_rotation` uniforms. Per-instance scale/rotation would require shader attribute additions (noted in ROADMAP).

### 7.3 Color Updates Are Split
- `set_instance_colors()` — updates fill mesh only
- `set_instance_stroke_colors()` — updates stroke mesh only

Users must call both for fill+stroke shapes and track which shapes have stroke.

### 7.4 Fixed Capacity
`create_multiple_instances(capacity)` sets capacity upfront. No way to grow dynamically.

**Status:** All open.

---

## 8. Geometry Construction Details

### Mixed GL Primitives
| Shape | GL Mode | Vertex Count |
|-------|---------|-------------|
| Point, MultiPoint | GL_POINTS | 1 per point |
| Line, Polyline, Arc | GL_TRIANGLES | 6 per segment (quad) |
| Rectangle | GL_TRIANGLE_STRIP | 4 |
| Circle, Ellipse, RoundedRectangle | GL_TRIANGLE_FAN | varies |
| Polygon | GL_TRIANGLES | 6 per ear-clipped triangle |
| Triangle, Image, Text | GL_TRIANGLES | 6 (2 triangles) |

Not inherently a problem, but affects future batching since GL modes can't be mixed in a single draw call.

### Hardcoded Quality Parameters
- Circle: 100 segments (hardcoded)
- Arc: 64 segments (hardcoded)
- RoundedRectangle: 8 segments per corner (hardcoded)
- Polyline miter limit: 4.0 (hardcoded)

No way for users to control quality vs performance tradeoff.

**Status:** Open.

---

## 9. Summary: Priority Improvements

### Resolved:
- [x] Add position getters: `x()`, `y()`, `position()` — v0.9.0
- [x] Add style mutators: `set_fill_color()`, `set_stroke_color()` — v0.9.0
- [x] Add `Color::from_rgba()`, `Color::from_hsl()`, `Color::from_hsla()` — v0.9.0
- [x] Alpha channel throughout rendering pipeline — v0.9.0
- [x] Standardize anchor points — v0.9.0 (removed `(x, y)` from construction)
- [x] Fix Line/Arc/Triangle anchor bugs — v0.11.0
- [x] Add configurable `Anchor` enum and `ShapeRenderableBuilder` — v0.11.0
- [x] Concave polygon support — ear-clipping triangulation + GL_TRIANGLES

### Open:
- [ ] Use `MIN_STROKE_WIDTH` constant consistently
- [ ] Stroke support for Circle, Ellipse, Polygon, RoundedRectangle, Triangle
- [ ] Document instancing position-ignored contract explicitly
- [ ] Per-instance scale/rotation attributes
- [ ] Dynamic instance capacity
- [ ] Configurable geometry quality (segment counts)
