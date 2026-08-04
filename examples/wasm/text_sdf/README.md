# text_sdf_wasm — SDF vs bitmap text under live zoom

Two lines of the same string, both rasterized once at 48px from a font
fetched over the network. **Scroll to zoom**: the bitmap line (top) blurs
as it magnifies; the SDF line (bottom) stays sharp at every scale — the
signed-distance-field fragment shader re-derives a crisp edge per frame
from the same atlas.

## Build & run

```bash
# from the repo root
cargo build -p text_sdf_wasm --target wasm32-unknown-unknown --release

python3 -m http.server 8000   # serve the repo root — no copy step
# open http://localhost:8000/examples/wasm/text_sdf/web/
# scroll on the canvas to zoom
```

The JS half of the backend is shared by all wasm examples:
`wilhelm_renderer_sys/js/webglrenderer.js`.
