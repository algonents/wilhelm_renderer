# SDF Font Rendering — Feasibility Assessment & Prototype

Status: **prototype working** (2026-07-29, branch `feat/vector-fonts`). Demo
in `examples/text_sdf`; production API integration not yet scheduled.

## Problem

Radar labels must stay legible across continuous zoom, and
`docs/ROADMAP.md` lists "text labels readable at all zoom levels" as an
unmet success criterion. The current text path rasterizes glyphs at a fixed
pixel size into a bitmap atlas; `set_scale` merely magnifies the
`GL_LINEAR`-filtered bitmap, which blurs above 1.0. Rendering at a different
size requires a whole new atlas (the font cache is keyed on `(path, size)`).

## TL;DR

Vector-quality text is **feasible with zero new dependencies**. FreeType
2.13.2 ships its own SDF (signed distance field) renderer; it was removed
from the bundled build only for package size (commit 77d4581) and was
restored verbatim from git history — unmodified upstream code,
certification-friendly. The whole prototype is:

- the restored `src/sdf/` module (+1 CMake line, +1 `ftmodule.h` line,
  ~185 KB source / ~38 KB in the compressed crate — sys crate packages at
  2.1 MiB with it),
- 2 C shims (`_ft_render_glyph_sdf`, `_ft_set_sdf_spread`), 2 raw bindings,
  3 safe wrappers,
- an `sdf` mode on `FontAtlas` (`FontAtlas::new_sdf`) reusing the existing
  packer/upload path unchanged,
- one 10-line fragment shader (`text_sdf.frag`), and
- `ShapeRenderable::text_sdf()` mirroring the bitmap text path.

One 48px SDF atlas serves every zoom level with crisp edges.

## How it works

Instead of `FT_LOAD_RENDER` (coverage bitmap), the glyph is loaded as an
outline (`FT_LOAD_DEFAULT`) and rendered with `FT_Render_Glyph(...,
FT_RENDER_MODE_SDF)`. Each 8-bit texel encodes distance to the nearest
glyph edge: 128 on the edge, >128 inside, mapped over ±`spread` pixels
(default 8). Bilinear filtering interpolates *distances*, which stay
accurate under magnification, and the fragment shader reconstructs a crisp
edge at any scale:

```glsl
float d = texture(u_fontAtlas, TexCoord).r;
float w = fwidth(d);                     // adapts to the current zoom
alpha = smoothstep(0.5 - w, 0.5 + w, d);
```

Key integration fact: FreeType's SDF renderer bakes the spread padding into
the returned bitmap dimensions **and** `bitmap_left`/`bitmap_top`, so the
existing metrics FFI, shelf packer, and `text_raw_vertices()` layout work
unchanged. The SDF bitmap is tightly packed (`pitch == width`, asserted in
`test_render_glyph_sdf`).

## Prototype scope

Built:
- `FontAtlas::new_sdf(path, size, atlas_size)` + `is_sdf()`; bitmap
  constructor untouched.
- `ShapeRenderable::text_sdf(content, font, size, color)` with its own
  shader singleton and font cache (1024 atlas — SDF glyphs carry 8px
  padding per side, so 48px ASCII overflows 512).
- `examples/text_sdf`: bitmap vs SDF columns at effective sizes 10px–192px
  from one 48px atlas.
- Unit test proving the sdf module is linked, registered, and produces a
  real distance field (value distribution check).
- Re-enabled shader compile-status checks in `Shader::compile` (were
  commented out; a new shader with silent failure is undebuggable).
  Link-status check still needs `glGetProgramiv` bindings.

Deliberately not built:
- No change to the public `Text` shape / `ShapeStyle` / builder path.
- No halo/outline/bold effects (near-free follow-ups, see below).
- No unification of the bitmap and SDF font caches.

## Production work list

1. **API integration**: an SDF flag on `Text` (breaking enum/struct change)
   or a `TextStyle`/`ShapeStyle` field, so the builder + anchor path works
   for SDF text. Prototype's `text_sdf()` bypasses anchors.
2. **Cache key**: unify font caches on `(path, size, mode)`.
3. **Atlas policy**: one rasterization size per font (e.g. 48px) reused at
   all display sizes via `u_scale`, replacing per-size atlases; decide
   atlas growth/eviction.
4. **Small text**: below ~14px, hinted bitmap rendering is sharper than
   SDF; keep the bitmap path for small fixed-size text (visible in the
   demo's 10px row).
5. **Label effects**: halo/outline/bold are one uniform + a few shader
   lines each (`dist` thresholds) — high value for radar label
   decluttering.
6. **Shader diagnostics**: bind `glGetProgramiv`/`glGetProgramInfoLog` for
   link-status checking.
7. Optional: register `ft_bitmap_sdf_renderer_class` (bsdf) if SDF from
   pre-rendered bitmaps is ever needed; it compiles inside `sdf.c` today
   but is unregistered.

## Trade-offs & watch-items

- Single-channel SDF slightly rounds sharp corners at extreme
  magnification (multi-channel MSDF would fix this; not worth the
  complexity for radar labels).
- Spread (8px) bounds both the halo width available to shaders and the
  atlas space per glyph; raising it costs atlas area.
- Thin-stroke fonts never reach texel value 255 (interior distance is
  capped by stroke half-width) — harmless for rendering, but relevant for
  anyone writing shader effects that assume the full range.
- Package growth measured: ~38 KB compressed; sys crate at 2.1 MiB total.

## Rough sizing

Prototype: done (this branch). Production integration (work list items
1–5): small — the heavy lifting (native module, FFI, atlas, shader) is
this branch; what remains is Rust API design plus the usual
examples/docs/CHANGELOG pass.
