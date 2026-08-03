# shapes_scaled_wasm — interactive browser example

The `shapes_scaled` example in a browser canvas: anchored shapes with
**zoom on mouse wheel**. First interactive port — canvas wheel events
reach the engine's `Window::on_scroll` closure through the GLFW-style
trampolines (`wilhelm_dispatch_scroll`), identically to native.

Scene = the native `examples/shapes_scaled` minus text, images, and
points (out of the spike's scope by design). See `docs/DESIGN_WASM.md`.

## Build & run

```bash
# from the repo root
cargo build -p shapes_scaled_wasm --target wasm32-unknown-unknown --release

python3 -m http.server 8000   # serve the repo root — no copy step
# open http://localhost:8000/examples/wasm/shapes_scaled/web/
# scroll on the canvas to zoom
```

The JS half of the backend is shared by all wasm examples:
`wilhelm_renderer_sys/js/webglrenderer.js` (the page selects the module
via `window.WILHELM_WASM`).
