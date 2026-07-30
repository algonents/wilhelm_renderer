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

## Integration plan: prototype → framework feature

What it takes for SDF text to stop being a side door and become the
framework's text rendering, in dependency order.

### 1. API design (the gate — everything else hangs off it)

The prototype's `ShapeRenderable::text_sdf()` bypasses the
`from_shape`/builder path, so SDF text has no anchors, no style mutation,
and a bolted-on signature. Production folds the mode into the normal path:

- Put the mode on `Text` (where font path/size already live): a
  `Text::new_sdf(...)` constructor or `.with_sdf()` builder setting a new
  field. Note `Text`'s fields are `pub`, so adding a field breaks
  struct-literal construction — acceptable pre-1.0, but batch it with any
  other `Text` changes (e.g. multi-line) to break once.
- Dispatch inside `ShapeRenderable::from_shape` / `text()`: select
  `(font cache, shader)` by mode. Anchors, `ShapeStyle.fill`, the builder,
  and style mutation then work identically for both modes with no further
  code. Delete `text_sdf()` (or keep it as a thin convenience wrapper).

### 2. One atlas serves all sizes (the actual payoff)

Today the cache key is `(path, size)` even in SDF mode, so two SDF sizes
still build two atlases. Production:

- Rasterize each font once at a canonical SDF size (48px); cache key for
  SDF becomes `(path)` (bitmap keeps `(path, size)`); unify both maps
  into one keyed `(path, size, mode)` with a canonicalized size for SDF.
- Bake the intrinsic factor `requested_size / canonical_size` into the
  vertex positions at build time (in `text_raw_vertices`, which currently
  assumes atlas size == display size), so `u_scale` stays purely the
  user's zoom control and `set_scale` semantics don't change.
- `measure_text` must report metrics at the *requested* size.
- Derive atlas texture size from `canonical_size + 2×spread` and charset
  instead of the hardcoded 512/1024, and decide the atlas-full strategy
  (grow, page, or evict — today it just prints "atlas full").

### 3. Size-based routing (small text)

Below ~14px, hinted bitmaps are sharper than SDF (visible in the demo's
10px row). Add an automatic policy: requested sizes under a threshold use
the bitmap path even when SDF is requested, with an override. Radar labels
at fixed small sizes keep hinting; anything zoomable gets SDF.

### 4. Label effects (why SDF is worth it beyond sharpness)

Halo, outline, and bold are each one uniform plus a few `smoothstep`
thresholds in `text_sdf.frag` — high value for CWP label decluttering.
Needs: a `TextStyle` (or `ShapeStyle` extension) carrying halo
color/width, uniform plumbing in `renderer.rs` (which currently re-looks
up uniform locations by string every draw — cache locations while there,
see PERFORMANCE_ANALYSIS.md), and a demo.

### 5. Rotation

`text.vert` has no `u_rotation`, so rotated text silently doesn't rotate —
a pre-existing defect both text modes share. Rotated labels (leader lines,
map-aligned annotations) need it; add the same rotation handling the shape
shader has.

### 6. Robustness and diagnostics

- Bind `glGetProgramiv`/`glGetProgramInfoLog`; finish link-status checking
  in `Shader::compile` (compile-status was re-enabled on this branch).
- Fix the GLFW init crash found during this work: connect failure to the
  Wayland socket segfaults instead of returning an error.
- Optional: register `ft_bitmap_sdf_renderer_class` (bsdf) if SDF from
  pre-rendered bitmaps is ever wanted; it compiles inside `sdf.c` today
  but is unregistered.

### 7. Batching (design for it, don't block on it)

Text is still one draw call per string (`docs/TODO.md`). A single SDF
atlas per font makes glyph-level batching *more* tractable (all sizes
share one texture, so one instanced draw can cover every label). Not part
of this integration, but the API above shouldn't preclude it — which is
another reason to prefer per-font canonical atlases over per-size ones.

### Done means

- `Text` renders via the builder with anchors and style mutation in both
  modes; one SDF atlas per font serves every display size.
- A zoomable demo (e.g. `waypoints`) shows labels crisp across the full
  zoom range; a halo demo exists.
- Unit tests cover cache keying, `measure_text` at non-canonical sizes,
  and the size-routing threshold.
- Existing bitmap-text behavior is pixel-identical for callers that never
  opt into SDF.

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

## Path to master & crates.io release

The functional work list above makes the feature *good*; this list is what
a merge to master and a release actually require, in order:

1. **API decision before merge**: everything merged to master ships as
   public API on the next release. Either complete integration step 1
   (`Text`/`ShapeStyle` integration) first, or consciously release
   `ShapeRenderable::text_sdf()`, `FontAtlas::new_sdf`/`is_sdf`, and the
   three new freetype wrappers as a documented-experimental API that may
   change. The prototype's rustdoc on `text_sdf()` already flags this.
2. **Version bumps** (two-crate convention, see commit ab14a8a):
   - `wilhelm_renderer_sys` 0.11.0 → **0.12.0** — it gained public FFI
     (`_ft_render_glyph_sdf`, `_ft_set_sdf_spread`, `FT_LOAD_DEFAULT`)
     and bundled C sources.
   - `wilhelm_renderer` 0.13.0 → **0.14.0**, updating the exact pin in
     the root `Cargo.toml` to `version = "=0.12.0"`.
3. **CHANGELOG.md**: entry for 0.14.0 — SDF text (with a usage snippet,
   matching the house style), the restored FreeType sdf module, and the
   behavior change that `Shader::compile` now returns `Err` on compile
   failure instead of silently linking (technically observable by users
   who compile their own shaders).
4. **Doc sweep**: `docs/ROADMAP.md` (the "text labels readable at all
   zoom levels" criterion is now met; add batching note for SDF text —
   same unbatched-draw caveat as bitmap text per `docs/TODO.md`),
   `docs/PRIMITIVES.md` (Text row), `docs/OPENGL.md` (SDF listed as a
   "next step" — done), `CLAUDE.md` (FreeType removed-modules list no
   longer includes sdf).
5. **Release verification**: `cargo package -p wilhelm_renderer_sys`
   *with verify* (the `--no-verify` run measured 2.1 MiB compressed, 516
   files, sdf sources included — no exclude-list change needed); then
   publish sys first, then the main crate. docs.rs is unaffected
   (`build.rs` skips the native build under `DOCS_RS`).
6. **Platform check**: the sdf module is upstream FreeType built via the
   same CMake path on all three platforms, so no linking changes are
   expected — but run the macOS and Windows CI/builds before publishing;
   only Linux has been built and tested on this branch.

## Rough sizing

Prototype: done (this branch). Integration steps 1–5: small-to-moderate —
the heavy lifting (native module, FFI, atlas, shader) is this branch;
step 2 (canonical-size atlases and layout scaling) is the only part with
real design depth, the rest is mechanical Rust API work. Release
mechanics: the checklist above, dominated by the API decision in step 1.
