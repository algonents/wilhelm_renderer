# SDF (Vector-Quality) Text Rendering

Status: **implemented on the pure-Rust font stack** (2026-08-04, `feat/wasm_backend`).
Demos: `examples/text_sdf` (native, bitmap vs SDF columns), `examples/wasm/text_sdf`
(browser, live scroll-zoom). Originally prototyped on `feat/vector-fonts` with
FreeType's `sdf` module; that generator died with the FreeType removal and was
replaced by a pure-Rust implementation — the shader, API shape, and analysis
below carry over from the prototype.

## Problem

Radar labels must stay legible across continuous zoom (`docs/ROADMAP.md`:
"text labels readable at all zoom levels"). Bitmap-atlas text rasterizes
glyphs at a fixed pixel size; `set_scale` just magnifies a `GL_LINEAR`
bitmap, so text blurs above 1.0. A different size means a whole new atlas
(cache key includes the size).

## How it works

A signed distance field stores, per texel, the distance to the nearest
glyph edge instead of coverage: **128 = on the edge, >128 inside**, mapped
over ±`SDF_SPREAD` (8) px. Bilinear filtering interpolates *distances*
(which stay meaningful under magnification, unlike interpolated coverage),
and the fragment shader re-derives a crisp edge per frame:

```glsl
float d = texture(u_fontAtlas, TexCoord).r;
float w = fwidth(d);                        // one on-screen pixel
float alpha = smoothstep(0.5 - w, 0.5 + w, d);
```

`fwidth` sizes the anti-aliasing band automatically at any zoom — no
CPU-side plumbing. The vertex shader is shared with bitmap text
(`u_scale` provides the zoom); only the fragment stage differs
(`src/graphics2d/shaders/text_sdf.frag`, which requests `highp` so the
wasm glue does not inject `mediump`).

## The pure-Rust generator (`src/core/font.rs`)

`rasterize_glyph_sdf()` — pure function, headless-testable, same shape as
`rasterize_glyph()`:

1. **Flatten** the glyph outline to line segments (`SegmentCollector`):
   quadratics subdivided by exact max-deviation (`|p0-2c+p2|/4`), cubics by
   Wang's bound, tolerance `FLATTEN_TOL = 0.1` px (the 8-bit SDF encodes
   edge position to ~0.063 px; coarser flattening wobbles under zoom).
2. **Sign** by nonzero-winding scanline crossings against those same
   segments (half-open rule; exact TrueType fill semantics). Using one
   segment list for both sign and distance makes the field the exact SDF
   of a single well-defined shape — no coverage-vs-distance disagreement
   at the edge.
3. **Distance**: per pixel, min point-segment distance over segments
   within the row's ±(spread+0.5) band (row prefilter; brute force within
   the band — <1ms/glyph at 48px, no spatial index needed).
4. **Encode**: `texel = clamp(0.5 + sd/(2·spread), 0, 1)·255`.

**Metrics bake the spread** (as FreeType's renderer did): bitmap dims grow
by `2·SDF_SPREAD`, `bearing_x -= SDF_SPREAD`, `bearing_y += SDF_SPREAD`,
advance unchanged. Because padding rides inside the normal metrics, the
atlas shelf packer, `GlyphInfo`, and `text_raw_vertices()` are untouched —
`AtlasMode::Sdf` only switches which rasterizer `cache_glyph` calls.

## API

```rust
// One 48px atlas serves every display size via scaling:
let mut label = ShapeRenderable::text_sdf("FL350", "DejaVuSans", 48, Color::white());
label.set_scale(size / 48.0);
```

- `font_path` accepts a filesystem path or a `register_font` name (works
  on wasm — see `examples/wasm/text_sdf`).
- SDF atlases allocate 1024 (spread padding overflows 512 at 48px ASCII).
- One unified font cache keyed `(path, size, is_sdf)`.
- `from_shape(ShapeKind::Text)` keeps the bitmap path; SDF is only
  reachable through `text_sdf()` for now (see Future work).

## Trade-offs

- Single-channel SDF rounds sharp corners at extreme magnification
  (multi-channel SDF would fix; judged not worth the complexity).
- `SDF_SPREAD = 8` bounds future halo/outline width and costs atlas area.
- Thin strokes never reach texel 255 (interior distance is capped by the
  stroke half-width) — harmless for rendering, relevant to tooling that
  inspects atlases.
- Below ~14px, hinted-look bitmap text is crisper than SDF; keep using
  bitmap text for very small fixed-size labels.

## Future work (unchanged from the prototype's analysis)

1. **Unified `Text` API** — fold the mode into `Text`/`ShapeStyle` and
   dispatch inside `from_shape`; note `Text`'s fields are `pub`, so adding
   a field is a breaking change.
2. **One canonical atlas for all sizes** — rasterize at 48px only; bake
   `requested/canonical` into vertex positions so `u_scale` stays the
   user's zoom; `measure_text` must report at the requested size.
3. **Size routing** — auto-fallback to bitmap below ~14px with override.
4. **Label effects** — halo/outline/bold are one uniform + extra
   smoothstep thresholds; needs a `TextStyle` and uniform plumbing.
5. **Rotation** — `text.vert` has no `u_rotation`; rotated text silently
   doesn't rotate (pre-existing, shared by both modes).
6. **Link-status check** — `Shader::compile` now checks compile status per
   stage; link status needs `glGetProgramiv`/`glGetProgramInfoLog` bindings.
7. **Batching** — still one draw call per string; a single per-font atlas
   makes glyph-level instancing tractable.
