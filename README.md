
<img src="https://raw.githubusercontent.com/algonents/wilhelm_renderer/master/images/wr_logo_v3.svg" alt="Wilhelm Renderer" width="340">

wilhelm_renderer is a minimalist 2D graphics engine written in Rust with native OpenGL bindings.
Its goal is to provide a robust foundation for rendering 2D shapes and visualizing
2D data and animations in real time.

## Status

*APIs are still evolving — always use the latest release.*

## Examples

<table>
<tr>
<td align="center"><a href="examples/shapes/src/main.rs"><img width="180" alt="shapes" title="Shapes" src="https://raw.githubusercontent.com/algonents/wilhelm_renderer/master/images/shapes.png"></a><br><sub>Shapes</sub></td>
<td align="center"><a href="examples/bouncing_balls_instanced/src/main.rs"><img width="180" alt="bouncing_balls_instanced" title="Bouncing Balls (Instanced)" src="https://raw.githubusercontent.com/algonents/wilhelm_renderer/master/images/bouncing_balls_instanced.png"></a><br><sub>Bouncing Balls (Instanced)</sub></td>
<td align="center"></td>
</tr>
</table>

All examples are standalone Cargo projects in the [`examples/`](examples/) directory. Run any example with:

```shell
cd examples/shapes && cargo run
```

Build all examples at once to verify API compatibility:

```shell
cargo build --workspace
```

| Example | Description |
|---------|-------------|
| [shapes](examples/shapes/src/main.rs) | All supported shape types |
| [shapes_scaled](examples/shapes_scaled/src/main.rs) | Shapes with scroll-to-zoom scaling |
| [rotations](examples/rotations/src/main.rs) | Per-shape rotation and animation |
| [text](examples/text/src/main.rs) | Text rendering with FreeType |
| [instancing](examples/instancing/src/main.rs) | 6,000 instanced circles with per-instance color |
| [bouncing_balls](examples/bouncing_balls/src/main.rs) | 50 animated balls with per-shape rendering |
| [bouncing_balls_instanced](examples/bouncing_balls_instanced/src/main.rs) | 10,000 animated balls with instanced rendering |
| [waypoints](examples/waypoints/src/main.rs) | WGS84 coordinates with Camera2D projection |
| [waypoints_instanced](examples/waypoints_instanced/src/main.rs) | Instanced waypoint markers with Camera2D |
| [triangle](examples/triangle/src/main.rs) | Low-level: custom shaders and geometry |
| [transforms](examples/transforms/src/main.rs) | Low-level: matrix transforms and animation |

## Quick Start

```rust
use wilhelm_renderer::core::{App, Color, Window};
use wilhelm_renderer::graphics2d::shapes::{
    Circle, Rectangle, ShapeKind, ShapeRenderable, ShapeStyle, Text,
};

fn main() {
    let window = Window::new("Shapes", 800, 800, Color::from_rgb(0.07, 0.13, 0.17));
    let mut app = App::new(window);

    let shape = |pos: (f32, f32), kind: ShapeKind, style: ShapeStyle| {
        let mut s = ShapeRenderable::from_shape(kind, style);
        s.set_position(pos.0, pos.1);
        s
    };

    app.add_shapes(vec![
        shape((160.0, 280.0),
            ShapeKind::Text(Text::new("Hello!", "fonts/DejaVuSans.ttf", 48)),
            ShapeStyle::fill(Color::from_rgb(0.94, 0.91, 0.78)),
        ),
        shape((50.0, 50.0),
            ShapeKind::Rectangle(Rectangle::new(200.0, 80.0)),
            ShapeStyle::fill(Color::from_rgb(0.2, 0.5, 0.9)),
        ),
        shape((400.0, 400.0),
            ShapeKind::Circle(Circle::new(50.0)),
            ShapeStyle::fill(Color::from_rgb(0.0, 0.0, 1.0)),
        ),
    ]);

    app.run();
}
```

## Features

**Shapes:** Point, MultiPoint, Line, Polyline, Arc, Triangle, Rectangle, RoundedRectangle, Circle, Ellipse, Polygon, Image, Text

**Rendering:**
- Instanced rendering for high-performance scenes (10,000+ shapes)
- Per-shape rotation, scale, and position
- Fill, stroke, and fill+stroke styles
- MSAA 4x multisampling

**Text:** FreeType-based rendering with font atlas caching and on-demand glyph loading

**Projection:** Camera2D with world/screen coordinate conversion, pan, zoom, and WGS84/Mercator support

**Bundled dependencies:** GLFW 3.4 and FreeType 2.13.2 are included — no external setup required

## Installation

### Linux

```shell
sudo apt-get install libgl1-mesa-dev
sudo apt install libwayland-dev libxkbcommon-dev xorg-dev
```

### Windows

Ensure that Visual C++ Build Tools and CMake 3.5 or later are installed.

### macOS

Ensure that the Xcode command-line tools and CMake 3.5 or later are installed.

Then add to your `Cargo.toml`:

```toml
[dependencies]
wilhelm_renderer = "0.8"
```

## IDE Setup (C++ Language Server)

The C++ component uses CMake. To enable clangd support, generate a `compile_commands.json`:

```shell
cmake -S cpp -B build -DCMAKE_EXPORT_COMPILE_COMMANDS=ON
```

The `build/` directory is gitignored. Re-run only when `cpp/CMakeLists.txt` changes.

## Issues

Report issues on [GitHub](https://github.com/algonents/wilhelm-renderer/issues).
