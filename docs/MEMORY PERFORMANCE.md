# Memory & Performance: ADT vs Trait Objects

This document analyzes the performance characteristics of using Algebraic Data Types (ADT) for `ShapeKind` versus the previous trait object (`Box<dyn Shape>`) approach.

## When Data Moves

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

## Enum Size

The largest variants are `Vec`-based (MultiPoint, Polyline, Polygon), but `Vec` is just 24 bytes on the stack (pointer + length + capacity). The actual point data lives on the heap regardless.

So `ShapeKind` is ~32 bytes - small enough that moving it is essentially free (a few register/cache operations).

## ADT vs Box<dyn Shape> Comparison

| Aspect | ADT (ShapeKind) | Box<dyn Shape> |
|--------|-----------------|----------------|
| Stack size | ~32 bytes | 16 bytes |
| Heap alloc | None (for enum itself) | Always |
| Access pattern | Direct | Pointer indirection |
| Cache locality | Better | Worse |
| Branch prediction | Jump table | Vtable dispatch |

## Conclusion

For shapes created once and rendered many times, the ADT approach is actually *faster* due to:
- No heap allocation for the shape descriptor
- Better cache behavior (data is inline)
- Simpler branch prediction (jump table vs vtable dispatch)

The slightly larger stack size (~32 bytes vs 16 bytes) is negligible in practice.
