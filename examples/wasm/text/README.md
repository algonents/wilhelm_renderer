# text_wasm — text rendering in the browser

The `text` example in a browser canvas. The font is **fetched over the
network** (no filesystem, nothing embedded): the page lists
`fonts/DejaVuSans.ttf` in `window.WILHELM_ASSETS`, the shared glue copies
the bytes into wasm memory before `wasm_init`, and the app registers them
with `register_font("DejaVuSans", …)` — which the `Text` shapes reference
as their `font_path`.

Glyph rasterization is the pure-Rust text stack (`ttf-parser` +
`ab_glyph_rasterizer`) — identical code and output on native and web.

## Build & run

```bash
# from the repo root
cargo build -p text_wasm --target wasm32-unknown-unknown --release

python3 -m http.server 8000   # serve the repo root — no copy step
# open http://localhost:8000/examples/wasm/text/web/
```

The JS half of the backend is shared by all wasm examples:
`wilhelm_renderer_sys/js/webglrenderer.js` (the page selects the module
via `window.WILHELM_WASM`).
