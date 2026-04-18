# Performance

## Memory: ADT vs Trait Objects

This section analyzes the performance characteristics of using Algebraic Data Types (ADT) for `ShapeKind` versus the previous trait object (`Box<dyn Shape>`) approach.

### When Data Moves

During construction, there is a **move chain**:

```rust
// 1. Caller creates ShapeKind and moves it into from_shape
ShapeRenderable::from_shape(x, y, ShapeKind::Triangle(triangle), style)

// 2. from_shape pattern matches, moving inner data OUT of the enum
pub fn from_shape(x: f32, y: f32, shape: ShapeKind, ...) -> Self {
    match shape {
        ShapeKind::Triangle(triangle) => {  // triangle moved out
            ShapeRenderable::triangle(x, y, triangle, ...)  // moved again
        }
    }
}

// 3. Private constructor stores ShapeKind in the final struct
fn triangle(x: f32, y: f32, triangle: Triangle, ...) -> Self {
    // ... build geometry ...
    ShapeRenderable::new(x, y, mesh, ShapeKind::Triangle(triangle))  // final move
}
```

So construction involves 2-3 moves of the shape data. For small structs like `Triangle` (24 bytes) or `Circle` (4 bytes), this is negligible - just a few CPU cycles copying stack memory.

After construction, `ShapeKind` is only **borrowed**:
- SVG export uses `match &self.shape { ... }` - no moves, just references

The rendering path (`render()`) doesn't touch `ShapeKind` at all - it uses the pre-built `Mesh`.

### Enum Size

The largest variants are `Vec`-based (MultiPoint, Polyline, Polygon), but `Vec` is just 24 bytes on the stack (pointer + length + capacity). The actual point data lives on the heap regardless.

So `ShapeKind` is ~32 bytes - small enough that moving it is essentially free (a few register/cache operations).

### ADT vs Box<dyn Shape> Comparison

| Aspect | ADT (ShapeKind) | Box<dyn Shape> |
|--------|-----------------|----------------|
| Stack size | ~32 bytes | 16 bytes |
| Heap alloc | None (for enum itself) | Always |
| Access pattern | Direct | Pointer indirection |
| Cache locality | Better | Worse |
| Branch prediction | Jump table | Vtable dispatch |

### Conclusion

For shapes created once and rendered many times, the ADT approach is actually *faster* due to:
- No heap allocation for the shape descriptor
- Better cache behavior (data is inline)
- Simpler branch prediction (jump table vs vtable dispatch)

The slightly larger stack size (~32 bytes vs 16 bytes) is negligible in practice.

---

## OpenGL Rendering Pipeline

Analysis of the per-frame rendering overhead, bottlenecks, and optimization opportunities.

### Render Path

```
App::run() loop
├─ shapes.sort_by_key(z_order)              O(n log n)
└─ for each shape:
   ├─ compute ortho_2d(width, height)       REDUNDANT — same for all shapes
   ├─ shader.use_program()                  UNTRACKED — may rebind same shader
   ├─ geometry.bind()                       UNTRACKED — binds VAO every time
   ├─ gl_enable(GL_BLEND) + gl_blend_func   REDUNDANT — same every call
   ├─ gl_vertex_attrib_4f(2, 0,0,0,0)       REDUNDANT — reset instance color
   ├─ 8× gl_get_uniform_location(string)    8 GPU queries per shape
   ├─ 8× gl_uniform_*()                     set even if unchanged
   ├─ gl_draw_arrays()                       actual GPU work
   └─ if stroke_mesh: repeat all of above
```

### Cost Profile at N Shapes

| Operation | Calls/Frame | Source |
|-----------|-------------|--------|
| Shader switches | N (untracked) | `renderer.rs:39` |
| VAO binds | N (untracked) | `renderer.rs:40` |
| Uniform location lookups | 8N | `renderer.rs:49–95` |
| Uniform sets | 8N | `renderer.rs` (even if unchanged) |
| Blend state setup | N (redundant) | `renderer.rs:42–43` |
| Attrib reset | N (redundant) | `renderer.rs:47` |
| Projection compute | N (identical) | `shaperenderable.rs:308` |
| Draw calls | N | `renderer.rs:106` |

At **1,000 shapes**: ~18,000 GL state changes per frame.

### Bottlenecks by Impact

#### 1. Uniform location lookups (HIGH)

8+ `gl_get_uniform_location()` calls per shape per frame (`renderer.rs:49–95`). Even with the stack-based string optimization in `opengl.rs`, these are GPU driver queries repeated every draw call.

**Fix:** Cache locations in `Shader` after compilation. One-time cost, zero per-frame.

#### 2. No state tracking (HIGH)

Shader, VAO, texture, and blend state are set unconditionally every draw call. If 50 circles share the same shader, it's rebound 50 times.

**Fix:** `RenderState` struct tracking current shader/VAO/texture. Only change on mismatch.

#### 3. No batching by shader (HIGH)

Shapes sorted only by z-order (`app.rs:160`). Intermixed shape types cause excessive shader/VAO switches.

**Fix:** Sort by `(z_order, shader_id)` to batch same-shader shapes within each z-layer.

#### 4. Projection recomputed per shape (LOW effort, EASY win)

`ortho_2d()` computed N times with identical window dimensions (`shaperenderable.rs:308`).

**Fix:** Compute once per frame in `App::run()`, pass to `render()`.

#### 5. Dual color uniform (MEDIUM)

Both `geometryColor` and `u_color` are looked up and set — two lookups for one value (`renderer.rs:75–88`). Exists because text shader uses a different uniform name.

**Fix:** Unify to a single color uniform name across all shaders.

#### 6. Instance buffer orphaning (MEDIUM for instanced shapes)

`update_instance_xy()` orphans and reallocates the GPU buffer every call (`geometry.rs:234–248`).

**Fix:** Track buffer capacity, only orphan when size changes.

#### 7. Blend state and attrib reset (LOW)

`gl_enable(GL_BLEND)` and `gl_blend_func()` called for every shape (`renderer.rs:42–43`). `gl_vertex_attrib_4f(2, ...)` called for every shape to reset instance color (`renderer.rs:47`).

**Fix:** Set blend state once at frame start. Reset attrib once before the draw loop.

### What's Already Optimized

- **Shader singletons** via `thread_local! OnceCell` — compiled once, shared via `Rc`
- **No heap allocations in the draw path** — uniform name strings are stack-based for < 64 chars
- **Geometry created once** at shape construction, reused every frame
- **Instancing infrastructure** works and is proven at 1000+ shapes
- **Font atlas caching** — glyphs rasterized once per font/size, shared across text shapes

### Estimated Optimization Potential

| Optimization | Estimated Frame Time Reduction | Effort |
|---|---|---|
| Uniform location caching | 15–20% | Low |
| State tracking (shader/VAO/texture) | 10–15% | Medium |
| Shader batching within z-layers | 20–50% (depends on shape mix) | Medium |
| Projection caching | ~1% | Trivial |
| Unify color uniform | ~2% | Low |
| **Combined** | **30–50%** | |

The GPU itself is not the bottleneck — at typical 2D scene complexity, the CPU-side GL state change overhead dominates. The rendering architecture is correct; these are incremental optimizations that become important as shape count scales toward 10,000+.
