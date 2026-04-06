# References

## Geometric Tools for Computer Graphics
*Philip Schneider and David Eberly*

Primary reference for computational geometry algorithms implemented in wilhelm_renderer.

### Relevant Chapters

**Immediately useful:**

| Chapter | Topic | wilhelm_renderer Use Case |
|---------|-------|--------------------------|
| 13 | Computational Geometry | Point-in-polygon (hit testing), polygon triangulation (ear clipping), convex hull, polygon clipping |
| 7 | Points, Lines, and Polygons | Polygon representation, winding order, convexity testing |
| 6 | Distance Methods | Point-to-line and point-to-segment distance (hit testing on lines/polylines) |
| 11 | Intersection Methods | Line-line, line-polygon, polygon-polygon intersection (boolean operations) |

**Future (curves and advanced primitives):**

| Chapter | Topic | wilhelm_renderer Use Case |
|---------|-------|--------------------------|
| 10 | Curves | Bezier curves, splines (future primitives) |

**Not needed:**
- 3D chapters (transforms, quaternions, 3D intersection)
- Ray tracing chapters
- Physics/dynamics chapters

---

## OpenGL 2D Rendering Learning Path

### Core 2D Foundation
- Orthographic projection
- Sprite rendering (textured quads)
- Batching (minimize draw calls)
- Texture atlases / sprite sheets
- Instancing for repeated shapes

### Blending & Compositing
- Alpha blending modes (src/dst factors)
- Pre-multiplied alpha
- Order-independent transparency
- Blend equations for effects (additive, multiply, screen)

### Text Rendering
- Bitmap fonts (texture atlas approach)
- Signed Distance Field (SDF) fonts - smooth at any scale
- Subpixel rendering

### Anti-Aliasing
- MSAA
- FXAA / SMAA (post-process)
- Analytical AA for lines/edges in shaders

### Framebuffers & Post-Processing
- Render-to-texture
- Multi-pass rendering
- Blur (Gaussian, box)
- Glow / bloom
- Color grading

### Performance at Scale
- Geometry batching (dynamic VBOs)
- Texture binding reduction
- Shader switching minimization
- Frustum/viewport culling
- Spatial partitioning (quadtree)

### Advanced 2D
- Stencil buffer (masking, clipping regions)
- Signed Distance Fields for shapes
- Bezier curves in shaders
- GPU-driven particle systems
- Compute shaders for physics/simulation

### Resources
- [learnopengl.com](https://learnopengl.com)
- OpenGL SuperBible (book)
- Real-Time Rendering (book)
