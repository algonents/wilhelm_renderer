# bouncing_balls_wasm — animated browser example

The `bouncing_balls` example in a browser canvas: 50 circles with
per-frame physics in `on_pre_render`, driven by the engine clock
(`Renderer::get_time` -> `performance.now()`), bouncing against the
fullscreen canvas bounds.

Differences from native: `rand` replaced by a dependency-free xorshift
PRNG (fixed seed), and the vestigial `set_point_size` call dropped.
See `docs/DESIGN_WASM.md`.

## Build & run

```bash
# from the repo root
cargo build -p bouncing_balls_wasm --target wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/bouncing_balls_wasm.wasm examples/wasm/bouncing_balls/web/

python3 -m http.server 8000 --directory examples/wasm/bouncing_balls/web
# open http://localhost:8000
```

`web/glue.js` is byte-identical to the other wasm examples' glue
(the page selects the module via `window.WILHELM_WASM`).
