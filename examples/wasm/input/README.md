# input_wasm — browser input demo

Exercises the full input dispatcher set of the wasm backend
(`wilhelm_dispatch_cursor_pos` / `_mouse_button` / `_key`, alongside the
earlier resize/scroll): the circle follows the cursor, holding a mouse
button recolors it (left = red, right = blue, middle = yellow), arrow keys
move the square (hold to repeat), Space recenters it. The engine-side
callbacks (`Window::on_cursor_position` / `on_mouse_button` / `on_key`)
are identical to native — the glue feeds the same GLFW-style trampolines
from DOM listeners.

## Build & run

```bash
# from the repo root
cargo build -p input_wasm --target wasm32-unknown-unknown --release

python3 -m http.server 8000   # serve the repo root — no copy step
# open http://localhost:8000/examples/wasm/input/web/
```
