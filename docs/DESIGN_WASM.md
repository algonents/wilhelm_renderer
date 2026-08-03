# WebAssembly Backend — Design

Status: **implementation well underway on `feat/wasm_backend`**
(last updated 2026-08-04; originally assessed 2026-08-02 on `feat/wasm`).
Decision: **one Rust client API, multiple backends behind a frozen FFI
contract**, with the browser (WebGL2) as the second backend, implemented
in pure Rust on `wasm32-unknown-unknown`.

## Status at a glance

Done (all on `feat/wasm_backend`, browser-verified):

- [x] Spike succeeded: shapes render in Chrome via the pure-Rust backend
      (`wilhelm_renderer_sys/src/web/`, contract symbols over ~50 JS imports)
- [x] `App::frame()` extraction; rAF-driven main loop (item 5)
- [x] `build.rs` early-returns on wasm — no C toolchain (item 7)
- [x] Shared JS glue consolidated as `wilhelm_renderer_sys/js/webglrenderer.js`
      (page contract: `WILHELM_WASM` + `WILHELM_ASSETS`)
- [x] Resize + scroll input through the GLFW trampolines
      (`wilhelm_dispatch_resize` / `wilhelm_dispatch_scroll`)
- [x] `GL_R8` font atlas internalformat (item 4)
- [x] MSAA via `antialias: true` context attribute on wasm (item 2)
- [x] `image` crate default-features trimmed to PNG (item 8)
- [x] Byte-based assets: `ImageSource`/`FontSource`, `try_*` loaders,
      `register_font`, network fetch protocol (item 6)
- [x] **FreeType deleted** — text is pure Rust (`ttf-parser` +
      `ab_glyph_rasterizer`) above the contract; ~87 → ~78 symbols
- [x] SDF text (`text_sdf()`, pure-Rust generator, `docs/SDF_FONTS.md`) —
      scale-independent labels, works on wasm
- [x] `Shader::compile` per-stage compile-status checks
- [x] Examples: `wasm/shapes` (incl. images + text), `wasm/shapes_scaled`,
      `wasm/bouncing_balls`, `wasm/text`, `wasm/text_sdf`

Remaining for a stable backend:

- [ ] GL_POINTS retirement → circle-backed Point/MultiPoint,
      remove `Renderer::set_point_size` (item 1 — precursor, still pending)
- [ ] Single-source GLSL ES 3.00 shaders; move the header rewrite from
      webglrenderer.js into the *native* backend (item 3 — currently inverted)
- [ ] Remaining input dispatchers: cursor move, mouse buttons, keys
- [ ] Instancing example on wasm
- [ ] `devicePixelRatio` / content-scale handling (item 9)
- [ ] WebGL context-loss recovery (shader singletons + atlas reset)
- [ ] Drop the `glGetError` per-call check in native `_glTexImage2D` (item 9)
- [ ] Contract specification document + per-backend conformance suite
- [ ] `examples/keyboard` still hand-rolls a blocking loop (item 5 note)

**Emscripten was considered and rejected for certification reasons.** It
would compile the bundled C++/GLFW/FreeType stack to WASM behind a
GL→WebGL translation runtime and GLFW emulation layer — a large body of
third-party code sitting *between* the verified boundary and the screen,
which we neither own nor can qualify. `web-sys`/`wasm-bindgen` were
rejected on the same grounds (large dependency tree vs the minimal-deps
policy, `docs/DESIGN.md`). Instead we own every line on both sides of the
boundary: a Rust backend plus a small hand-written JS glue file.

## Problem

Can wilhelm_renderer run in a browser via WebAssembly? Beyond demos, the
real question is architectural: the project's long-term direction is one
client API over multiple rendering backends, with the certification goal
that **verification of the client-facing crate survives a backend swap**.
The browser differs from native in three fundamental ways: WebGL2
(~OpenGL ES 3.0) instead of desktop GL 3.3 core, no GLFW (canvas + DOM
events), and the browser owning the event loop (no blocking `while` loop,
no filesystem).

## TL;DR

**Feasible, and the architecture is already 90% in place.** The safe Rust
layer contains zero `std::fs`, zero threads, zero `std::time`; shaders are
`include_str!`-embedded; the GL surface is almost entirely ES 3.0-clean;
and the FFI boundary is ~87 underscore-prefixed shim symbols with no
GL/GLFW/FreeType type leakage — i.e. the "backend contract" already exists,
it just isn't documented as one. (Since the FreeType removal the contract
is ~78 symbols: the 9 `_ft_*` entries are gone, text lives above it.)

Decisions:

- **Pure-Rust browser backend** — reimplement the sys-crate contract for
  `wasm32-unknown-unknown` in Rust + a small hand-written JS glue file.
  No Emscripten, no `web-sys`/`wasm-bindgen` dependency.
- **Contract floor = OpenGL ES 3.0.** Backends may *implement* the contract
  on any underlying API version; the contract itself promises only
  ES 3.0-expressible behavior. See "Why ES 3.0".
- **Divergence lives below the boundary.** The client crate stays
  single-source; anything target-specific happens inside a backend.
- **Precursors land on master first** (GL_POINTS retirement, `GL_R8`,
  `image` default-features, single shader dialect) — each justified
  independently of WASM.

## The certification argument

The claim to preserve: the safe crate (`src/`, ~4,900 lines) is
**target-independent source, verified once**; each sys backend is
**target-specific, verified against a frozen contract**. Swapping backends
does not invalidate upper-crate verification provided (a) the contract is
unchanged and (b) each backend passes the same conformance suite.

Design rules that follow:

1. **Freeze and document the contract** — the ~87 shim symbols
   (`_glDrawArrays`, `_glfwCreateWindow`, `_ft_new_face`, …) with their
   observable semantics. Today the contract exists structurally
   (`wilhelm_renderer_sys/src/{opengl,glfw,freetype}.rs`) but not as a
   specification.
2. **Converge, don't branch.** Prefer shrinking the contract to the
   GL 3.3 ∩ ES 3.0 intersection over per-target `#ifdef`s. Nearly the
   entire current GL surface already lives in that intersection (see the
   incompatibility list — 4 items, of which the worst is being retired
   anyway).
3. **Tests target the contract, not pixels.** Rasterizers differ across
   drivers, let alone across GL and ANGLE-backed WebGL — pixel-golden
   tests don't transfer. What transfers: upper-crate logic tests (geometry,
   math, atlas packing, anchors — already backend-free) plus a
   **backend conformance suite** driven through the 87 symbols, run once
   per target. A mock/recording backend implementing the same contract
   falls out for free and enables headless upper-crate CI.
4. Each new backend carries its own conformance run — no architecture
   removes that; the win is confining re-verification to a few hundred
   backend lines instead of 4,900 engine lines.

## Why ES 3.0 is the contract floor

The floor is the intersection of committed targets, and three independent
ceilings all land on ES 3.0:

- **The web's ceiling.** WebGL2 is defined against ES 3.0; there is no
  browser exposure of ES 3.1/3.2 (Chrome's "WebGL 2.0 Compute" was
  abandoned in favor of WebGPU). WebKit/Safari supports WebGL2 (Safari 15+,
  via ANGLE-on-Metal) — at ES 3.0 semantics like every other browser.
- **macOS's ceiling.** Apple froze OpenGL at 4.1; ES 3.1 compute maps to
  GL 4.3, unreachable on Macs — the same constraint that pins the engine
  to "GL 3.3 core for macOS compatibility" today.
- **The engine's needs.** ES 3.1/3.2 add compute, geometry, tessellation —
  none on the critical path for instanced 2D shapes and atlas text.

ES 3.2 (2015) is the final ES version; the API family's successor is
Vulkan/WebGPU. **WebGPU is explicitly parked**: it is a different model
(pipelines, bind groups, WGSL — no state machine), so a WebGPU backend
would be a GL emulator, not a thin contract implementation. It buys
compute and lower driver overhead the engine doesn't need, has no
safety-critical profile, and browsers run WebGL2 on ANGLE (D3D/Metal/
Vulkan) anyway. If WebGPU ever becomes a requirement, that is a deliberate
contract-v2 decision, not a backend addition.

## How the backend is implemented

Today, "backend" = whoever provides the 87 symbols to the linker: the
upper crate calls `sys::_glDrawArrays(...)`; the sys crate *declares* it
(`extern "C"`); the CMake-built C++ static lib *provides* it, calling real
libGL/GLFW/FreeType.

On `wasm32-unknown-unknown` the sys crate provides the same symbols as
ordinary Rust functions, selected by `cfg`:

```rust
#[cfg(not(target_arch = "wasm32"))]
extern "C" { pub fn _glDrawArrays(mode: GLenum, first: GLint, count: GLsizei); }

#[cfg(target_arch = "wasm32")]
pub unsafe fn _glDrawArrays(mode: GLenum, first: GLint, count: GLsizei) {
    /* forward to the WebGL2 context via the JS glue imports */
}
```

Same names, signatures, and semantics — the upper crate compiles unchanged
and cannot tell which body is underneath. `build.rs` does nothing on wasm
targets (same escape-hatch pattern as the existing `DOCS_RS` early-return).

Pieces of the wasm backend:

- **JS glue** (~300–400 lines, hand-written, owned): the wasm module
  declares imports (`#[link(wasm_import_module = "gl")] extern "C" { … }`);
  one small JS file supplies them, calling `WebGL2RenderingContext` on a
  canvas. This replaces `web-sys`/`wasm-bindgen` deliberately — zero crate
  dependencies, every line auditable.
- **Handle table** (~50 lines): the C GL API names objects with integers;
  WebGL returns opaque JS objects (`WebGLBuffer`, …). The backend maps
  integer id → JS object (allocate in `_glGenBuffers`, look up in
  `_glBindBuffer`).
- **GLFW's 22 symbols → canvas/DOM**: `_glfwCreateWindow` → canvas setup;
  `_glfwGetTime` → `performance.now()/1000`; `_glfwSwapBuffers`/
  `_glfwPollEvents` → no-ops (browser presents at rAF boundaries and
  pushes events). Input needs **no upper-crate changes**: `Window` already
  registers `extern "C"` trampolines (`src/core/window.rs:174-210`); the
  backend invokes those same trampolines from its DOM event listeners.
- **FreeType's 9 symbols**: FreeType is C and does not come along. The
  shim surface is tiny (load face / set size / render glyph / read bitmap
  + metrics) and can be satisfied by a pure-Rust rasterizer (`ab_glyph`,
  `fontdue`) behind the same `_ft_*` names — or the first web milestone
  ships shapes-only and text follows.
- **Main loop**: `App::run()`'s body is a pure function of `&mut self`
  (only `last_time` carried across iterations) — extract `fn frame(&mut
  self)`; native keeps the `while`, the wasm entry registers `frame` with
  `requestAnimationFrame` via the glue. Wrinkle: `App<'a>` callbacks are
  non-`'static` (`src/core/app.rs:12-14`); a browser entry point needs
  `App<'static>` (additive, not breaking).

## The complete incompatibility list

Verified against source; full list, not a sample.

### 1. `glPointSize` → retire `GL_POINTS` entirely (precursor, on master)

**PENDING** — `Renderer::set_point_size` and `GL_POINTS` geometry still
exist; the wasm examples simply omit points so far.

`glPointSize` does not exist in ES 3.0. Rather than porting it to a
`gl_PointSize` uniform, **retire point sprites**: keep the `Point`/
`MultiPoint` shape API but back it with (instanced) circle geometry.
Independently justified:

- ES/WebGL only *guarantees* 1px point size (`ALIASED_POINT_SIZE_RANGE`
  is implementation-defined); `GL_POINTS` also clip by center — a point
  vanishes with its whole radius at viewport edges (pop-out artifact,
  bad for radar displays). Circle geometry behaves identically everywhere
  — exactly the "tests survive a backend swap" criterion.
- The global point size is the only draw state configured via `Renderer`
  — hidden global state contradicting the stateless-renderer design; a
  per-shape radius is the better API (mixed sizes per scene).
- Batching survives: `enable_instancing_xy` + per-instance color
  (`vInstanceColor`) already exist; MultiPoint becomes one instanced
  unit-circle draw.
- **Zero downstream impact, verified 2026-08-02**: `set_point_size` /
  `MultiPoint` / `ShapeKind::Point` appear only in three bundled examples
  (`shapes`, `shapes_scaled`, `bouncing_balls`). Grepped clean across
  sky_guard_client, wilhelmos_kiosk, wilhelmos_kiosk_demo, skyui,
  sentrix, wilhelm_renderer_imgui, wilhelm_renderer_symbols.

Removes from the tree: `Renderer::set_point_size` (`src/core/renderer.rs:24`),
the `_glPointSize` shim (`glrenderer.cpp:462`), `point.frag`, `gl_PointCoord`.
Contract shrinks by one symbol.

### 2. `glEnable(GL_MULTISAMPLE)` — invalid enum in ES 3.0

**DONE (by construction):** `webglrenderer.js` requests
`getContext("webgl2", { antialias: true })`; the native enable never runs
on wasm (it lives in the C++ window setup).

`glrenderer.cpp:50`. In WebGL2, MSAA is a context-creation attribute
(`antialias: true`). Native backend keeps the enable; wasm backend requests
the attribute at context creation. Below-the-boundary divergence — fine.

### 3. Shader dialect: single-source GLSL ES 3.00 (precursor-adjacent)

**PENDING — currently inverted vs this design:** sources are still
`#version 330 core` and `webglrenderer.js` rewrites them to `300 es` (plus
injecting a default fragment precision when absent). The decided end state
flips this: author in `300 es`, native backend rewrites.

All 13 shaders are `#version 330 core` (9 in `src/graphics2d/shaders/`,
4 in examples). Rule 2 (converge, don't branch) applied: **author shaders
once in GLSL ES 3.00** (`#version 300 es` + precision qualifiers — legal
no-ops in desktop GLSL), and the *native backend* rewrites the version
header inside `_glShaderSource`. The client crate hands over identical
source on every target; no target-conditional code above the boundary.
(Rewriting headers in `Shader::compile` is rejected — it would put target
branching in the verified layer.)

### 4. Unsized `GL_RED` internalformat in the font atlas (precursor, on master)

**DONE (2026-08-03, on feat/wasm_backend):** the atlas allocates with `GL_R8`.

`src/core/font.rs:84-92` allocates the atlas with internalformat `GL_RED`;
WebGL2 rejects unsized `RED` — must be **`GL_R8`** (0x8229). `GL_R8` is
equally valid on desktop GL 3.3, so change it unconditionally (converge).
The `glTexSubImage2D` upload path (`GL_RED` as *format*) is already correct.

### 5. Blocking main loop

**DONE:** `App::frame()` extracted; wasm entry points drive it from
`requestAnimationFrame`. Still open: `examples/keyboard` hand-rolls its
own blocking loop and would need rewriting onto `App`.

`src/core/app.rs:142-176` — see "How the backend is implemented" above for
the `frame()` extraction.

### 6. Path-based asset loading

**DONE (2026-08-03, on feat/wasm_backend), beyond the plan below:**
`ImageSource{Path,Bytes}` / `FontSource{Path,Bytes}` converge every loader
on bytes; `register_font(name, bytes)` lets `Text::font_path` name
registered bytes; `try_*` variants make load failures recoverable. Fonts
went further than the `_ft_*`-shim plan: FreeType was deleted outright and
glyph rasterization is pure Rust (`ttf-parser` + `ab_glyph_rasterizer`) in
the safe crate — text now lives *above* the contract (~87 → ~78 symbols)
and is target-independent. The page-side transport is
`window.WILHELM_ASSETS` → `wasm_alloc` / `wasm_asset_loaded` (see
`wilhelm_renderer_sys/js/webglrenderer.js`).

- Fonts: `FT_New_Face(path)` (`glrenderer.cpp:498`, from
  `FontAtlas::new(font_path, …)`, `src/core/font.rs:59`).
- Images: `ImageReader::open(path)` (`src/core/image.rs:13`).

The wasm backend has no filesystem: assets arrive as bytes (fetched by the
page, or embedded). Fix: `FontAtlas::from_bytes` / image-from-bytes
constructors; downstream layers are already byte-oriented (`Image` is
`{width, height, pixels: Vec<u8>}`; `generate_texture_from_image` takes a
reference). On native, `FT_New_Memory_Face` is the byte-slice twin of
`FT_New_Face` (~10-line shim addition); on wasm the `_ft_*` backend is
byte-based from the start.

Related: asset-load failures currently `.expect()`/panic
(`shaperenderable.rs:277`, `image.rs:14-16`) — in WASM a panic aborts the
module; these want to become recoverable.

### 7. Build system

**DONE:** `build.rs` early-returns on wasm targets — no CMake, no C
toolchain required.

`wilhelm_renderer_sys/build.rs` dispatches on linux/apple/windows only.
The wasm branch is trivial: **skip CMake entirely** (no C is built), same
pattern as the `DOCS_RS` early-return (`build.rs:8-11`).

### 8. `image` crate default features (precursor, on master)

**DONE (on feat/wasm_backend):** `default-features = false, features =
["png"]`. Semver note for release: native loses non-PNG decoding.

`image = "0.25.6"` with defaults drags in **133 lockfile packages**,
including `rayon` (broken on `wasm32-unknown-unknown` without threads) and
the `ravif`/`rav1e` AV1 encoder. Fix regardless of WASM, on certification
grounds: `image = { version = "0.25", default-features = false,
features = ["png"] }`; consider making it optional entirely (a browser
page can decode via `createImageBitmap` and hand raw RGBA across).
(Noted in passing: `docs/DESIGN.md` still lists `glam` as a dependency;
math is hand-rolled in `src/core/math.rs` now.)

### 9. Minor items

- `_glTexImage2D` does `glGetError()` + stdout print per call
  (`glrenderer.cpp:311-319`) — `glGetError` forces a synchronous pipeline
  flush in WebGL; remove (arguably for native too).
- `glGetIntegerv(GL_VIEWPORT)` in `Renderer::viewport_size()`
  (`src/core/renderer.rs:30`) — valid, but a synchronous round-trip in
  WebGL; perf note.
- Content scale on the web comes from `devicePixelRatio`; the
  fullscreen/monitor path (`glrenderer.cpp:88-127`) maps to the browser
  Fullscreen API. Neither blocks the spike.

Explicitly checked and **absent** (each would have been a real problem):
`glPolygonMode`, `glLineWidth`, geometry/tessellation shader use,
`glGetTexImage`, `glMapBuffer`, client-side vertex arrays, `glMultiDraw*`,
sRGB enums, double-precision uniforms, UBOs, transform feedback. All
vertex attributes go through VBOs. Multiple windows, clipboard, gamma,
cursors, joystick: unused.

## Companion crate: wilhelm_renderer_imgui

`wilhelm_renderer_imgui` is **native-only by construction and stays that
way**. It bundles the Dear ImGui C++ sources with the `imgui_impl_glfw` /
`imgui_impl_opengl3` backends, which call the real GLFW window and real GL
context directly — it consumes the native backend's implementation, not
the 87-symbol contract, and sits outside the certification boundary by
design (its library code doesn't use the safe API). Nothing in this plan
affects it on desktop. On the web there is no equivalent and none is
needed: **browser UI is handled by the web layer** — HTML/DOM around the
canvas — not by porting an immediate-mode C++ UI into the wasm module.

## Proof-of-concept spike plan (EXECUTED — succeeded, then surpassed)

The spike (steps 2–6 below) succeeded on 2026-08-02: shapes rendered in
Chrome, 136 KB module, zero console errors, no upper-crate `cfg`s other
than the entry point. The branch has since gone well past the spike goal:
interaction (scroll-zoom), animation (bouncing balls), live resize,
network-loaded images, text (pure-Rust rasterizer — FreeType deleted
rather than shimmed, superseding step 6's `_ft_*` plan), and SDF text.
Of the step-1 precursors, `GL_R8` and `image` default-features are done
(landed on the branch, not master); GL_POINTS retirement and the shader
dialect flip remain.

Original plan, for the record:

1. Land the precursors on master: GL_POINTS retirement (item 1), `GL_R8`
   (item 4), `image` default-features (item 8), shader dialect flip
   (item 3: sources → `300 es`, native backend rewrites the header).
2. `App`: extract `frame(&mut self)` (item 5) — native behavior unchanged.
3. Sys crate: `cfg`-gate the extern declarations; implement the ~20 GL
   symbols + ~8 GLFW symbols the shapes example actually exercises, with
   the handle table; `build.rs` early-returns on wasm.
4. Write the JS glue (imports + canvas/context setup + rAF driver + DOM
   event → trampoline wiring).
5. Static page: load the `.wasm`, instantiate with the glue, run.
6. Verify visually against the native demo; then grow symbol coverage
   (remaining shapes, instancing, then text via the `_ft_*`-over-Rust-
   rasterizer decision).

## Open questions

Still open:

- **WebGL context loss**: shader singletons and the font atlas hold GL
  object IDs in `thread_local` `OnceCell`s with no reset path.
  Lost-context recovery needs `clear_font_cache()` plus a shader-cache
  reset that doesn't exist today. Real for production.
- **Contract specification**: where the (now ~78) symbol semantics get
  written down (a `docs/BACKEND_CONTRACT.md`?), and what the conformance
  suite runs on for the browser target (headless Chrome?).
- **WebGPU**: parked; revisit only as a deliberate contract-v2 decision
  if a hard requirement appears.

Resolved since first writing:

- ~~**Async assets vs sync constructors**~~ — resolved: the page fetches
  before `wasm_init` (`WILHELM_ASSETS` → `wasm_alloc`/`wasm_asset_loaded`),
  so construction stays synchronous; truly late-arriving assets can use
  the same protocol post-init.
- ~~**Text backend for wasm**~~ — resolved beyond the question's framing:
  `ttf-parser` + `ab_glyph_rasterizer` on *every* backend; FreeType is
  deleted, not merely non-authoritative on wasm. Text (bitmap and SDF)
  lives above the contract.
