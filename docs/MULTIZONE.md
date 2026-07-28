# Multi-Zone Rendering — Feasibility Assessment

Status: **future design — parked, not scheduled** (2026-07-28). The current
baseline remains a single screen in kiosk fullscreen mode with no zones;
this doc exists so the design thinking isn't lost when zones become current.

History: assessed 2026-07-27. Revised 2026-07-28: FBO/ImGui route re-priced —
shim wrapping is mechanical and cheap, which makes it the *less* invasive
option architecturally. See "Option B".

## Problem

ATC displays are historically 2K×2K. Operations are moving to 3K/4K monitors
(current dev/reference hardware: 2× 3840×2160 under Wayland). We want
wilhelm_renderer to support multiple rendering **zones** — independent
viewport sub-regions of one window — e.g. a main 2048×2048 radar zone plus
auxiliary zones filling the remaining screen real estate.

## TL;DR

Multi-zone is a **small-to-moderate** change either way; the architecture is
unusually well positioned for it. Two viable routes:

- **Option A — direct zones**: draw each zone straight to the default
  framebuffer at a pixel offset, clipped by scissor. Requires wrapping
  `glScissor`/`glDisable` in the shim and a **breaking change** to
  `Renderable::render` (thread a zone parameter through every draw).
- **Option B — FBO zones in ImGui windows**: render each zone to an
  offscreen texture, display via `ImGui::Image`. More shim bindings
  (~10, all mechanical), but **no breaking API change**, clipping for
  free, per-zone clear for free, and dockable/resizable zones for free.

Original assessment priced B at ~2× A by counting binding surface area.
Re-priced: the bindings are mechanical wraps in repos we own; measured by
architectural invasiveness, **B is the smaller change to the renderer core**
and is the recommended direction.

## Why this is easier than it sounds

Two deliberate design decisions (`docs/ROADMAP.md:22` — "Renderer is
stateless (no zoom/viewport state)") happen to be exactly what multi-zone
needs:

1. **The camera never touches the GPU.** `Camera2D` (`src/core/camera.rs:58`)
   is a plain `Copy` value type with no globals or statics — N instances can
   exist today. Projection happens on the CPU: the app calls
   `camera.world_to_screen(pos)` and sets shape positions in absolute window
   pixels. There is no view-matrix uniform to fight over.

2. **Because positions are window pixels, zone layout is just an offset.**
   Rendering the radar into a 2048×2048 region is
   `world_to_screen(track) + zone_origin` — no GL viewport switching needed
   for the common case. The camera's own origin math is already zone-local
   (center = `screen_size * 0.5`, `camera.rs:164-178`), not window-local.

The extension point also already exists: wilhelmos_kiosk's
`draw(&mut Context)` hook can drive multiple cameras per frame today.

## Option A — direct zones: what has to change

### 1. Full-window ortho baked into every draw (bulk of the work)

`ShapeRenderable::render` (`src/graphics2d/shapes/shaperenderable.rs:307-309`)
rebuilds a full-window ortho matrix from `window_handle.size()` on every
draw:

```rust
let (window_width, window_height) = renderer.window_handle.size();
let transform = ortho_2d(window_width as f32, window_height as f32);
```

It needs a zone/viewport parameter instead. This is a breaking change to the
`Renderable::render` signature — mechanical, but it touches everything.
Alternative (a mutable "current zone" on `Renderer`) would break the
stateless-renderer design goal.

### 2. Clipping — mandatory for Option A (C++ shim work)

The sys crate wraps **zero** scissor or framebuffer functions. Every GL call
goes through the hand-rolled shim
(`wilhelm_renderer_sys/cpp/glrenderer.{h,cpp}` → `wilhelm_renderer_sys/src/opengl.rs`
→ `src/core/engine/opengl.rs`), and `glScissor` / `glDisable` simply aren't
there. Without scissor, a track symbol at the radar zone's edge draws over
the adjacent zone.

Fix: ~4 new C functions + FFI decls + safe wrappers. Roughly half a day, but
mandatory. Note `gl_enable(cap: u32)` (`src/core/engine/opengl.rs:292-296`)
is already generic, and `gl_viewport` (`opengl.rs:30-34`) is already `pub`.

### 3. Zone-local input

`CameraController` (`src/core/camera.rs:206-216`) assumes camera == window:
cursor positions are raw window coordinates, and `on_resize`
(`camera.rs:304-312`) snaps camera size to window size. Needs zone-local
cursor mapping and per-zone hit-testing so pan/zoom in one zone doesn't move
another. `App` (`src/core/app.rs:15`) and kiosk `Context`
(`wilhelmos_kiosk/src/context.rs:25`) each hold exactly one controller.

Scope note: in real ops the main radar view is effectively static, and aux
zones may not need interactivity at all — the main zone might be the only one
with a controller, which shrinks this item considerably. The existing ImGui
`want_capture_mouse()` gating pattern (`wilhelmos_kiosk/src/app.rs:313-325`)
generalizes to per-zone mouse routing.

### 4. Small stompers

- The GLFW resize callback (`src/core/window.rs:32-34`) unconditionally
  resets the viewport to full window — would stomp any per-zone viewport.
  It is the only `gl_viewport` call in the library.
- `_glClearColor` clears the whole screen as a hidden side effect
  (`wilhelm_renderer_sys/cpp/glrenderer.cpp:210-215`, already flagged in
  `docs/TODO.md:107`) — no per-zone backgrounds until scissor exists.

## Option B — FBO zones in ImGui windows (recommended)

Render each zone into an offscreen framebuffer object (FBO) whose color
attachment is a texture, then display that texture inside an ImGui window
via `ImGui::Image`. An ImGui window is not a render surface — it's a draw
list rasterized over the default framebuffer at end of frame — so embedding
GL content in one *requires* this render-to-texture step.

### Why B avoids the breaking change

The key: `Renderer::viewport_size()` (`src/core/renderer.rs:28-32`) already
queries `GL_VIEWPORT` from the driver. If `ShapeRenderable` built its ortho
from **that** instead of `window_handle.size()`, "which zone am I rendering
into" stops being an API parameter — it's carried by GL state:

```
bind zone FBO → gl_viewport(0, 0, zone_w, zone_h) → draw shapes as today → unbind
```

Every shape automatically gets the right projection. No `Renderable::render`
signature change, no zone threading, and the stateless-renderer design goal
(`ROADMAP.md:22`) survives untouched. Clipping is free (the FBO *is* the
zone boundary — no scissor needed), per-zone clear is free (clear happens
per-FBO), and zones become draggable/resizable ImGui windows.

### Work list

1. **Shim wraps** (mechanical, repos are ours):
   - GL: `glGenFramebuffers`, `glBindFramebuffer`, `glFramebufferTexture2D`,
     `glCheckFramebufferStatus`, `glDeleteFramebuffers`, `glDisable` (~6 fns).
   - Empty-texture allocation path: `gl_tex_image_2d` already accepts a null
     pointer; `generate_texture_from_image` (`src/core/texture.rs`) just
     needs a data-less entry point.
   - ImGui: `Image`, `GetContentRegionAvail`, `GetCursorScreenPos`,
     `IsWindowHovered` (~4 fns in `wilhelm_renderer_imgui`).
2. **Ortho source switch** in `ShapeRenderable`: window size → viewport
   size. Caveat: `window_handle.size()` is logical pixels, `GL_VIEWPORT` is
   physical — this forces resolving the HiDPI debt in `TODO.md:120-140`
   rather than straddling it (a feature, given 4K/Wayland targets).
3. **Kiosk loop restructuring**: zone FBO passes run before the ImGui frame;
   `ctx.render_shapes()` and the hidden `glClear` in `_glClearColor` must
   cooperate.
4. **Resize handling**: reallocate the zone texture when its ImGui window is
   resized (on size-change only, not every frame).
5. **UV flip**: GL FBO textures are vertically flipped relative to ImGui's
   UV convention — flip Vs in the `Image` call.
6. **Input**: `IsWindowHovered` + mapping mouse from window coords to
   image-local coords replaces most of the `CameraController` rework.

### Costs vs Option A

GPU memory per zone texture plus ~one extra screenful of fill rate per frame
(render to texture, then ImGui draws the textured quad). Normally noise, but
measure against the known 30 FPS dip before assuming.

## Watch-items

- **Wayland fractional scaling** (`docs/TODO.md:120-140`): mouse coords are
  logical, framebuffer is physical. Zone rectangles must be defined
  consistently in one space or zones will be mispositioned on HiDPI/4K
  displays. Existing debt that gets sharper with zones.
- **Per-draw uniform lookups**: `gl_get_uniform_location` runs 6–8 times per
  mesh per frame (`src/core/renderer.rs:49-99`, `renderer.rs:128-175`; known
  debt, `docs/ROADMAP.md:147`). More zones → more draw calls → this cost
  multiplies. Worth fixing before or alongside zones given steady-state-perf
  priority.
- **Multi-window/multi-monitor** is a separate, later question: the kiosk is
  hardwired to `Window::new_fullscreen` on the primary monitor
  (`wilhelmos_kiosk/src/app.rs:241`). Zones within one fullscreen window come
  first.

## Rough sizing

| Piece | Effort |
|---|---|
| **A**: scissor bindings in shim | ~0.5 day |
| **A**: `Zone { origin, size, camera }` type + threading through `ShapeRenderable` / kiosk `Context` (breaking change) | a few days |
| **A**: zone-local input (only if aux zones are interactive) | incremental on top |
| **B**: FBO + ImGui bindings (~10 fns, two repos, mechanical) | ~1 day |
| **B**: ortho source switch + HiDPI reconciliation | ~1 day |
| **B**: kiosk loop restructuring, resize/realloc, UV flip, input mapping | 1–2 days |

Both land in the same "a few days" band; B's work is additive bindings and
loop plumbing, A's is a breaking trait change threaded through every
renderable. B recommended.

Related roadmap items that intersect: Layer System (`docs/ROADMAP.md:79-86`),
viewport frustum culling (`docs/ROADMAP.md:153`), and the transform-composition
note in `docs/TODO.md:92` ("No matrix stack or hierarchical transforms…").
