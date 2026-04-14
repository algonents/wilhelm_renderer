# Primitives

Graphics primitives needed for 2D visualization applications, organized by priority. Requirements derived from **ICAO Annex 4 — Aeronautical Charts** (11th Edition, 2009) and CWP (Controller Working Position) radar display needs.

## Currently Supported

| Primitive | Status | Notes |
|-----------|--------|-------|
| Point | ✓ Complete | |
| MultiPoint | ✓ Complete | |
| Line | ✓ Complete | Solid and dashed |
| Polyline | ✓ Complete | Solid and dashed, miter/bevel joins |
| Arc | ✓ Complete | Solid and dashed |
| Triangle | ✓ Complete | Fill only (no stroke) |
| Rectangle | ✓ Complete | Fill, stroke, fill+stroke, dashed stroke |
| RoundedRectangle | ✓ Complete | Fill only (no stroke) |
| Circle | ✓ Complete | Fill only (no stroke) |
| Ellipse | ✓ Complete | Fill only (no stroke) |
| Polygon | ✓ Complete | Fill only (no stroke) |
| Image | ✓ Complete | |
| Text | ✓ Complete | FreeType, single line, configurable size/color |

## Priority 1 — CWP Core

Primitives required for a functional radar situation display.

| Primitive | Use Case | ICAO Reference | Status |
|-----------|----------|----------------|--------|
| **Circle/Ellipse/Polygon stroke** | Navaid symbols, range rings, airspace boundaries, flyover designation circles | Appendix 2: symbols 84–98 (aerodromes), 99–110 (navaids), 121 (significant points) | Missing |
| **Arrow tips on lines** | Velocity vectors, heading indicators, procedure track direction, missed approach, wind barbs | Appendix 2: Ch. 11 §11.10.6 (approach/missed approach tracks are arrowed solid/dashed/dotted lines) | Missing |
| **Flexible dash patterns (dash array)** | FIR boundaries (dash-rectangle), ATZ (dotted), uncontrolled routes (short-long dash), advisory airspace | Appendix 2: #111 (FIR dash-dot), #112 (ATZ dotted), #114 (uncontrolled short-long), #115 (ADA dashed) | Missing |
| **Hatched/cross-hatched fills** | Restricted, prohibited, and danger areas (P/R/D); glaciers; international boundaries closed to aircraft | Appendix 2: #128 (diagonal hatching, angle/density varies), #129 (cross-hatching) | Missing |
| **Sector / pie slice** | MSA circles, TAA arcs, radar coverage sectors, airspace sectors | Appendix 2: #171 (MSA — circle divided into altitude-labeled sectors), #172 (TAA arcs) | Missing |
| **Ring / annulus** | Range rings with NM labels, distance circles on ATC surveillance charts | Ch. 21 §21.9.3.1e3 (fine dashed distance circles at 20km/10NM intervals) | Missing |
| **Clipping** | Circular radar viewport, sector-based rendering | Ch. 20 (electronic display requirements) | Missing |
| **Racetrack shape** | Holding patterns | Appendix 2: #173 (two semicircles connected by parallel lines with directional arrow) | Missing |

## Priority 2 — Enroute / Area Chart

Primitives needed for enroute and area chart rendering (airways, navaids, airspace).

| Primitive | Use Case | ICAO Reference | Status |
|-----------|----------|----------------|--------|
| **Tick marks along path** | Railroads, levees, fences, telegraph lines | Appendix 2: #51–56 (railroad ticks), #9 (levee), #65 (fence x-x-x), #66 (telegraph T-marks) | Missing |
| **Regular polygon (hexagon)** | VOR/DME symbol | Appendix 2: #103 (hexagon = combined VOR circle + DME square) | Missing |
| **Diamond (4-pointed)** | Waypoint symbol | Appendix 2: #121 (waypoint — outlined = on-request, filled = compulsory) | Missing |
| **Leader lines** | Obstacle labels, label-to-symbol connections | Appendix 2: #136 (obstacle elevation with leader line to triangle) | Missing |
| **Compass rose** | Graduated 360° circle with tick marks, cardinal labels, oriented to magnetic north | Appendix 2: #110 (combined with navaid symbol at center) | Missing |
| **Text along path** | Route identifiers, contour labels, isogonic values along lines | Appendix 2: #138 (isogonic lines with value labels) | Missing |
| **Rotated text** | Labels along airways and routes | Ch. 7–8 (enroute/area charts: bearings, tracks, radials) | Missing |
| **Text decorations (underline/overline)** | Altitude encoding: underline = at-or-above, overline = at-or-below, both = mandatory | Appendix 2: #125 (altitude/flight level text formatting) | Missing |
| **Bezier / smooth curves** | Contour lines, coastlines, rivers, SIDs/STARs | Appendix 2: #1–2 (contours), Ch. 9–10 (SID/STAR tracks) | Missing |

## Priority 3 — Map / Topography Layer

Primitives for background map rendering (terrain, hydrography, culture).

| Primitive | Use Case | ICAO Reference | Status |
|-----------|----------|----------------|--------|
| **Stipple/dot pattern fill** | Built-up areas, sand, tidal flats, non-perennial water, salt lakes | Appendix 2: #7 (sand), #21 (tidal flats), #33 (salt lake), #47 (cities); Appendix 3 (blue stipple, black stipple) | Missing |
| **Polygon with holes** | Lakes within land, exclusion zones, complex coastlines | Appendix 2: #31 (lake outlines within landmass) | Missing |
| **Grid / graticule** | Lat/lon lines, coordinate overlay | Appendix 3: graticule in BLACK | Missing |
| **Hypsometric tints** | Elevation colour ramps (8–10 bands from green through brown to white) | Appendix 4 (two alternative colour ramp systems) | Missing |
| **Hachures** | Short radiating lines for terrain relief depiction | Appendix 2: #3 (relief by hachures — short parallel lines downslope) | Missing |
| **Gradient line** | Altitude-colored routes, heat-mapped paths | — | Missing |

## Priority 4 — Detailed Cartography / Symbology

Specialized symbols, typically handled by a companion symbols crate.

| Primitive | Use Case | ICAO Reference | Status |
|-----------|----------|----------------|--------|
| **Star (N-pointed)** | Aeronautical ground lights | Appendix 2: #143 (open star paper, filled star electronic) | Missing |
| **Y-shape / 3-pointed** | TACAN, VORTAC symbols | Appendix 2: #106 (TACAN), #107 (VORTAC) | Missing |
| **Composite symbols** | Navaid + compass rose, symbol + flyover circle, obstacle + leader line + label | Appendix 2: #110, #121 (flyover = any symbol enclosed in circle) | Missing |
| **Cross / maltese cross** | FAF (final approach fix), charted isolated rock | Appendix 2: #124 (FAF), #44 (rock) | Missing |
| **Pattern fills (grid, crosshatch variants)** | Steel mesh runways, rice fields, salt pans | Appendix 2: #146 (mesh runway), #34 (salt pans), #36 (rice fields) | Missing |
| **Wavy / irregular lines** | Transmission lines, rapids, coral reefs | Appendix 2: #137 (transmission line with T-marks), #22 (coral reefs), #27 (rapids) | Missing |
| **Spline** | Smooth curves through control points (Catmull-Rom, B-spline) | — | Missing |
| **Capsule** | Label backgrounds, UI elements | — | Missing |
| **Scale bar** | Distance reference for maps | — | Missing |
| **Spiral** | Archimedean or logarithmic spiral | — | Missing |

## ICAO Colour Requirements

From Appendix 3 (Colour Guide). Minimum palette for aeronautical chart rendering:

| Colour | Primary Usage |
|--------|-------------|
| **Black** | Culture, outlines, grids, graticules, spot elevations, names/lettering |
| **Brown** | Contours, topographic features |
| **Blue** | Shore lines, rivers, lakes, bathymetric contours, hydrographic names |
| **Blue half-tone** | Open water areas |
| **Blue stipple** | Salt lakes, non-perennial rivers/lakes |
| **Green** | Woods, wooded areas |
| **Yellow** | Built-up areas (alternative to black stipple) |
| **Red** | Highways/roads (optional) |
| **Magenta** | Aeronautical data (preferred for instrument procedures) |
| **Dark blue** | Aeronautical data (alternative) |
| **Golden buff / White** | Unsurveyed areas |

## ICAO Line Style Requirements

From Appendix 2. Distinct line patterns needed:

| Pattern | Visual | Usage |
|---------|--------|-------|
| Solid | `————————` | Contours, shore lines, rivers, runways, approach tracks |
| Dashed (long) | `— — — —` | Approximate contours, international boundaries, ADA/CTR airspace |
| Dashed (short-long) | `- — - —` | Uncontrolled routes (#114) |
| Dotted | `· · · · · ·` | ATZ boundaries (#112), additional procedure tracks |
| Dash-dot | `— · — · —` | FIR boundaries (#111) |
| Heavy solid | Thick `————` | Sector boundary limits for minimum vectoring altitude |
| Fine dashed | Thin `— — —` | Distance circles on ATC surveillance charts |

## Recommended Build Order

Based on CWP impact and dependency chains:

1. **Circle/Ellipse/Polygon stroke** — single most impactful gap; unlocks navaids, range rings, airspace boundaries, flyover circles, and dashed variants of all three
2. **Arrow tips** — velocity vectors, procedure tracks, missed approach
3. **Flexible dash patterns** — FIR dash-dot, ATZ dotted, uncontrolled route patterns
4. **Hatched fills** — restricted/prohibited/danger areas
5. **Sector/pie slice** — MSA circles, radar coverage
6. **Clipping** — radar viewport
7. **Racetrack shape** — holding patterns
8. **Tick marks along path** — railroads, fences, decorations along lines
