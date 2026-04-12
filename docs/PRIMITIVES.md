# Primitives

Graphics primitives needed for 2D visualization applications (maps, radar, data visualization), organized by priority.

## Critical (Must Have)

| Primitive | Use Case | Status |
|-----------|----------|--------|
| **Text** | Place names, callsigns, labels, waypoint names, sector labels | ✓ Complete |
| **Rotated shapes** | Aircraft symbols, runway representations, oriented markers | ✓ Complete |
| **Dashed/dotted lines** | Borders, restricted zones, predicted tracks, FIR boundaries, planned routes | ✓ Complete |
| **Arrow** | Direction indicators, velocity vectors, heading, wind barbs, north arrow | Missing |
| **Thick polyline (variable width)** | Roads, rivers, airways, route highlighting | Missing |

## Important (Should Have)

| Primitive | Use Case | Status |
|-----------|----------|--------|
| **Sector (pie slice)** | Radar coverage, airspace sectors, field-of-view cones, pie charts | Missing |
| **Ring / Annulus** | Range rings, distance indicators, status rings | Missing |
| **Annular sector** | Range ring segments, partial coverage areas | Missing |
| **Bezier curve** | Smooth roads, rivers, coastlines, SIDs/STARs, airway depictions | Missing |
| **Polygon with holes** | Lakes within land, exclusion zones, complex boundaries | Missing |
| **Grid / Graticule** | Lat/lon lines, coordinate overlay, reference grids | Missing |
| **Leader lines** | Connecting labels to symbols/markers | Missing |
| **Marker / Pin** | Points of interest, waypoints, named locations | Missing |

## Nice to Have

| Primitive | Use Case | Status |
|-----------|----------|--------|
| **Spline** | Smooth curves through control points (Catmull-Rom, B-spline) | Missing |
| **Regular polygon** | Hexagons, octagons, pentagons (n-sided equilateral) | Missing |
| **Star** | N-pointed star markers, decorative symbols | Missing |
| **Capsule** | Label backgrounds, UI elements, route endpoints | Missing |
| **Cross / Plus** | Data markers, intersection indicators | Missing |
| **Path** | Composite shape from lines + arcs + curves (SVG-like) | Missing |
| **Gradient line** | Altitude-colored routes, heat-mapped paths | Missing |
| **Scale bar** | Distance reference for maps | Missing |
| **Compass rose** | Bearing reference overlay | Missing |
| **Hatched/patterned fills** | Restricted areas, danger zones, prohibited airspace | Missing |
| **Spiral** | Archimedean or logarithmic spiral | Missing |
| **Thick arc** | Holding patterns, procedure turns | Partial (arc has width param) |

## Currently Supported

- Point
- MultiPoint
- Line
- Polyline
- Arc
- Triangle
- Rectangle
- RoundedRectangle
- Circle
- Ellipse
- Polygon
- Image
- Text
