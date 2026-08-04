# shapes_wasm — browser backend spike

The `shapes` example rendered in a browser canvas via the pure-Rust
WebAssembly backend (`wasm32-unknown-unknown`, no Emscripten, no
wasm-bindgen). See `docs/DESIGN_WASM.md` for the architecture.

Scene = the native `examples/shapes` minus text and points (out of the
spike's scope by design). Images are included — fetched over the
network, not embedded.

## Build & run

```bash
# from the repo root
cargo build -p shapes_wasm --target wasm32-unknown-unknown --release

python3 -m http.server 8000   # serve the repo root — no copy step
# open http://localhost:8000/examples/wasm/shapes/web/
```

## How it works

- `src/lib.rs` builds the same `App` + `ShapeRenderable` scene as the
  native example and exports `wasm_init` / `wasm_frame`, plus the asset
  protocol (`wasm_alloc` / `wasm_asset_loaded`) used to receive image
  bytes fetched by the page.
- `wilhelm_renderer_sys/js/webglrenderer.js` — the JS half of the
  backend, shared by all wasm examples — supplies the `"wilhelm"` import
  module (WebGL2 calls, canvas setup, clock), fetches the assets listed
  in `window.WILHELM_ASSETS` before `wasm_init`, and drives `wasm_frame`
  from `requestAnimationFrame`.
- The backend proper lives in `wilhelm_renderer_sys/src/web/` — the same
  `_gl*`/`_glfw*` contract symbols as the native C++ shim, implemented
  over the JS imports.
