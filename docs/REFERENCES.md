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
