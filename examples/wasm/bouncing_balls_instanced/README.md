# bouncing_balls_instanced_wasm — instancing in the browser

The `bouncing_balls_instanced` example in a browser canvas: **10,000
balls in a single instanced draw call** — per-instance position and color
via `glVertexAttribDivisor` / `glDrawArraysInstanced`, the first exercise
of the web backend's instancing path. Physics updates all instance
positions every frame through `set_instance_positions`.

As in the other wasm ports, `rand` is replaced by a dependency-free
xorshift PRNG and the canvas is fullscreen.

## Build & run

```bash
# from the repo root
cargo build -p bouncing_balls_instanced_wasm --target wasm32-unknown-unknown --release

python3 -m http.server 8000   # serve the repo root — no copy step
# open http://localhost:8000/examples/wasm/bouncing_balls_instanced/web/
```
