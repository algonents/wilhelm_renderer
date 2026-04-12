# ShapeRenderable API Review

A comprehensive analysis of inconsistencies, unintuitive behaviors, and improvement opportunities in the ShapeRenderable API.

---

## 1. Anchor Point Inconsistencies

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

### Summary: anchor / rotation pivot / scale origin (corrected)

Because of the shader, all three columns are always identical. What differs is *where that shared point lies on the shape.*

| Shape | Anchor Point | Rotation Pivot | Scale Origin |
|-------|-------------|----------------|--------------|
| Point | The point itself | N/A | N/A |
| MultiPoint | First point | First point | First point |
| Polyline | First point | First point | First point |
| Polygon | First vertex | First vertex | First vertex |
| Circle | Center | Center | Center |
| Ellipse | Center | Center | Center |
| Image | Center | Center | Center |
| Rectangle | Bottom-left corner (top-left visually under Y-down) | Same | Same |
| RoundedRectangle | Same as Rectangle | Same | Same |
| **Line** | **World origin** (vertices baked absolute) | **World origin** | **World origin** |
| **Triangle** | **World origin** (vertices baked absolute) | **World origin** | **World origin** |
| **Arc** | **First perimeter point** (not the arc center) | **First perimeter point** | **First perimeter point** |
| **Text** | **Top-left of text cell** (not the baseline) | N/A | N/A |

### Per-shape anchors (ground truth from the code)

| Shape | Local (0,0) location | Status |
|-------|----------------------|--------|
| Point | The point itself (vertex `[0,0]`) | OK |
| MultiPoint | First point (all points shifted by `-points[0]`, `s.x/y = points[0]`) | OK |
| Polyline | First point (same pattern as MultiPoint) | OK |
| Polygon | First polygon vertex (same pattern) | OK |
| Circle | Center | OK |
| Ellipse | Center | OK |
| Image | Center (geometry built with `-hw..+hw`, `-hh..+hh`) | OK |
| Rectangle | Bottom-left corner of the quad (visually top-left under a Y-down projection) | OK, but differs from Circle/Ellipse/Image |
| RoundedRectangle | Same corner as Rectangle. The fan's center vertex at `(w/2, h/2)` is just a fan seed, **not** the pivot | OK, but differs from Circle/Ellipse/Image |
| **Line** | **World origin** — `line_geometry` bakes absolute `(x1,y1)`/`(x2,y2)` into the vertex buffer; `s.x/y` default to 0 | **Bug** |
| **Triangle** | **World origin** — vertices baked as-is with no normalization; `s.x/y` default to 0 | **Bug** |
| **Arc** | **First perimeter point** — `arc()` builds points around the arc's center, then `polyline_from_points` subtracts `points[0]`, which is the *starting edge point*, not the center | **Bug** |
| **Text** | **Top-left of the text cell**, not the baseline — `baseline_y = font_size`, so local `y=0` is `font_size` pixels above the baseline | Doc was wrong; may also want to revisit |

### Three levels of inconsistency

1. **Closed shapes split three ways.** Circle/Ellipse/Image anchor at the center; Rectangle/RoundedRectangle anchor at a corner; Polygon anchors at its first vertex.
2. **Vertex-defined shapes split two ways.** Polyline and Polygon normalize to their first point (rotating around that point is at least usable). Line and Triangle do **not** normalize at all — they bake world-space vertex coordinates into the mesh, so rotation and scale pivot around the world origin, not the shape. A Triangle positioned near `(100, 100)` rotates around `(0, 0)`, sweeping across the scene.
3. **Arc is broken independently of all of the above.** `set_position(cx, cy)` does not place the arc's center at `(cx, cy)`, and rotation does not spin the arc around its own center — both are offset by the arc's first edge point.

### Impact

- Scaling a Rectangle at `(50, 50)` with `scale = 2` grows it from the corner; scaling a Circle at the same position grows it symmetrically. Users must memorize each shape's anchor to predict transforms.
- Rotating a Line or Triangle is effectively unusable unless the shape is constructed at the world origin and positioned via `set_position()` afterwards (the workaround `sky_guard_client` already uses for other reasons, see Section 2).
- Rotating an Arc around its "center" requires pre-computing the offset from the first edge point and applying it manually — non-obvious and undocumented.

### Recommendation

Treat this in two passes:

1. **Fix the bugs first, independent of any API change.** Line, Triangle, and Arc should normalize their geometry the same way Polyline/Polygon already do (compute a local origin, shift vertices by `-origin`, store the origin in `s.x/s.y`). This alone makes rotation and scale work predictably for those shapes without changing any public API.
2. **Then consider a configurable `Origin` enum** for shapes where users legitimately want a choice (e.g., Rectangle corner vs. center). Default to current post-fix behavior to stay non-breaking; opt in for uniform behavior.

```rust
pub enum Origin {
    Center,
    TopLeft,
    // Current default varies by shape
}
```

---

## 2. Redundant Position for Vertex-Defined Shapes

Line, Triangle, Polygon, and Polyline define their geometry with explicit vertices, making the `(x, y)` position parameter redundant or confusing:

- **Line**: `(x, y)` is added to both `start` and `end` — double bookkeeping
- **Polygon/Polyline**: Points are made absolute with `(x, y)`, then re-anchored to first point
- **Triangle**: `(x, y)` is an offset applied to all vertices

sky_guard_client works around this by creating shapes at `(0.0, 0.0)` and using `set_position()` separately.

**Recommendation:** Document the convention clearly. Consider allowing vertex-defined shapes to accept absolute coordinates with `(x, y)` = `(0, 0)` as the intended usage.

---

## 3. No Style Mutation After Construction

Shapes can only be styled at creation time via `ShapeStyle`. After construction:
- `set_position()` — exists
- `set_scale()` — exists
- `set_rotation()` — exists
- `set_color()` — **missing**
- `set_stroke_color()` — **missing**
- `set_stroke_width()` — **missing**

To change color, the entire shape must be rebuilt.

**Recommendation:** Add style mutators:
```rust
fn set_fill_color(&mut self, color: Color)
fn set_stroke_color(&mut self, color: Color)
fn set_stroke_width(&mut self, width: f32)
```

---

## 4. Missing Position Getters

`scale()` and `rotation()` getters exist, but there are no `x()` or `y()` getters. Users must track position externally.

**Recommendation:** Add `x()`, `y()`, and `position() -> (f32, f32)` getters.

---

## 5. Stroke Handling Inconsistencies

### 5.1 Stroke Width Clamping
- Line geometry: uses `stroke_width.max(MIN_STROKE_WIDTH)` constant
- Polyline geometry: uses `stroke_width.max(1.0)` — hardcoded, not using the constant

**Recommendation:** Use `MIN_STROKE_WIDTH` constant consistently.

### 5.2 Stroke Support Varies by Shape
- **Rectangle**: fill, stroke, fill+stroke — all supported
- **Circle, Ellipse, Polygon, RoundedRectangle, Triangle**: fill only (stroke noted as TODO in ROADMAP.md)
- **Line, Polyline, Arc**: stroke only (inherently stroke-based)

**Recommendation:** Continue stroke rollout per ROADMAP Phase 2.5.

### 5.3 No Style for (None, None)
`ShapeStyle` with `fill: None, stroke_color: None` silently defaults to white fill. No way to create an invisible shape.

---

## 6. Color and Alpha

### 6.1 No Alpha Channel
`Color::from_rgb()` hardcodes alpha to 1.0. No `Color::from_rgba()` exists.

### 6.2 Renderer Color Handling
- `geometryColor` uniform is RGB only (vec3, `gl_uniform_3f`)
- `u_color` uniform is RGBA but alpha is hardcoded to 1.0
- Per-instance color (attribute 2) supports RGBA

**Recommendation:** Add `Color::from_rgba()` and propagate alpha through the uniform pipeline.

---

## 7. Instancing Asymmetries

### 7.1 Position Ignored in Instanced Mode
When `instance_count > 0`, the shape's `(self.x, self.y)` is ignored — positions come entirely from `set_instance_positions()`. This is a silent contract.

### 7.2 No Per-Instance Scale or Rotation
All instances share the same `u_scale` and `u_rotation` uniforms. Per-instance scale/rotation would require shader attribute additions (noted in ROADMAP Phase 6).

### 7.3 Color Updates Are Split
- `set_instance_colors()` — updates fill mesh only
- `set_instance_stroke_colors()` — updates stroke mesh only

Users must call both for fill+stroke shapes and track which shapes have stroke.

### 7.4 Fixed Capacity
`create_multiple_instances(capacity)` sets capacity upfront. No way to grow dynamically.

---

## 8. SVG Export Bugs

### Circle and Ellipse SVG positions are wrong:
```rust
// Circle SVG (line 1136) — adds radius to position
cx = self.x + circle.radius
cy = self.y + circle.radius
```

But rendering treats `(self.x, self.y)` as center, not top-left. The SVG output places the circle at the wrong position.

Same issue for Ellipse.

### Missing SVG implementations:
- Image: unimplemented
- Arc: unimplemented

**Recommendation:** Fix Circle/Ellipse SVG to use `self.x, self.y` directly as center. Implement missing SVG exports.

---

## 9. Geometry Construction Details

### Mixed GL Primitives
| Shape | GL Mode | Vertex Count |
|-------|---------|-------------|
| Point, MultiPoint | GL_POINTS | 1 per point |
| Line, Polyline, Arc | GL_TRIANGLES | 6 per segment (quad) |
| Rectangle | GL_TRIANGLE_STRIP | 4 |
| Circle, Ellipse, RoundedRectangle, Polygon | GL_TRIANGLE_FAN | varies |
| Triangle, Image, Text | GL_TRIANGLES | 6 (2 triangles) |

Not inherently a problem, but affects future batching (Strategy B in ROADMAP) since GL modes can't be mixed in a single draw call.

### Polygon only supports convex shapes
`Polygon` uses `GL_TRIANGLE_FAN`, which only renders correctly for convex polygons. Concave polygons produce incorrect geometry because triangle fan assumes all triangles share a common vertex. Supporting concave polygons requires triangulation (e.g., ear clipping algorithm) before uploading vertices to the GPU.

### Hardcoded Quality Parameters
- Circle: 64 segments (hardcoded)
- Arc: 64 segments (hardcoded)
- RoundedRectangle: 8 segments per corner (hardcoded)
- Polyline miter limit: 4.0 (hardcoded)

No way for users to control quality vs performance tradeoff.

---

## 10. Summary: Priority Improvements

### Non-Breaking (additive):
- [x] Add position getters: `x()`, `y()`, `position()`
- [x] Add style mutators: `set_fill_color()`, `set_stroke_color()` — done (stroke_width requires geometry rebuild, not added)
- [x] Add `Color::from_rgba()` — done, plus `Color::from_hsl()` and `Color::from_hsla()`
- [ ] Add `Origin` enum for configurable anchor points (default to current behavior)
- [ ] Fix Circle/Ellipse SVG export
- [ ] Use `MIN_STROKE_WIDTH` constant consistently

### Behavioral (needs migration consideration):
- [x] Standardize anchor points for new primitives — resolved by removing `(x, y)` from construction; all shapes now use `set_position()`
- [ ] Document `(x, y)` semantics clearly per shape type
- [ ] Make instancing position-ignored contract explicit

### Future (larger changes):
- [ ] Per-instance scale/rotation attributes
- [ ] Dynamic instance capacity
- [ ] Configurable geometry quality (segment counts)
- [x] Alpha channel throughout rendering pipeline — done in v0.9.0
- [ ] Concave polygon support (requires triangulation, currently convex only via GL_TRIANGLE_FAN)
