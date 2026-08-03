# shapes_wasm — browser backend spike

The `shapes` example rendered in a browser canvas via the pure-Rust
WebAssembly backend (`wasm32-unknown-unknown`, no Emscripten, no
wasm-bindgen). See `docs/DESIGN_WASM.md` for the architecture.

Scene = the native `examples/shapes` minus text, images, and points
(out of the spike's scope by design).

## Build & run

```bash
# from the repo root
cargo build -p shapes_wasm --target wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/shapes_wasm.wasm examples/wasm/shapes/web/

# serve the page (any static server works)
python3 -m http.server 8000 --directory examples/wasm/shapes/web
# open http://localhost:8000
```

## How it works

- `src/lib.rs` builds the same `App` + `ShapeRenderable` scene as the
  native example and exports `wasm_init` / `wasm_frame`.
- `web/glue.js` supplies the `"wilhelm"` import module (WebGL2 calls,
  canvas setup, clock) and drives `wasm_frame` from
  `requestAnimationFrame`.
- The backend proper lives in `wilhelm_renderer_sys/src/web/` — the same
  `_gl*`/`_glfw*` contract symbols as the native C++ shim, implemented
  over the JS imports.
