
<img src="https://raw.githubusercontent.com/algonents/wilhelm_renderer/master/images/wr_logo_v3.svg" alt="Wilhelm Renderer" width="340">

wilhelm_renderer is a minimalist 2D graphics engine written in Rust with native OpenGL bindings.
Its goal is to provide a robust foundation for rendering 2D shapes and visualizing
2D data and animations in real time.

## Status

*APIs are still evolving — always use the latest release.*

## Features

**Shapes:** Point, MultiPoint, Line, Polyline, Arc, Triangle, Rectangle, RoundedRectangle, Circle, Ellipse, Polygon, Image, Text

**Rendering:**
- Instanced rendering for high-performance scenes (10,000+ shapes)
- Per-shape rotation, scale, and position
- Fill, stroke, and fill+stroke styles
- Alpha/opacity support
- MSAA 4x multisampling

**Text:** FreeType-based rendering with font atlas caching and on-demand glyph loading

**Projection:** Camera2D with world/screen coordinate conversion, pan, zoom, and WGS84/Mercator support

**Bundled dependencies:** GLFW 3.4 and FreeType 2.13.2 are included — no external setup required

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

## Examples

All examples are standalone Cargo projects in the [`examples/`](examples/) directory. Run any example with:

```shell
cd examples/<example> && cargo run
```

Build all examples at once to verify API compatibility:

```shell
cargo build --workspace
```

| | Example | Description |
|---|---------|-------------|
| <a href="examples/triangle"><img width="120" alt="triangle" src="https://raw.githubusercontent.com/algonents/wilhelm_renderer/master/images/triangle.png"></a> | triangle | Low-level: custom shaders and geometry |
| <a href="examples/transforms"><img width="120" alt="transforms" src="https://raw.githubusercontent.com/algonents/wilhelm_renderer/master/images/transforms.png"></a> | transforms | Low-level: matrix transforms and animation |
| <a href="examples/shapes"><img width="120" alt="shapes" src="https://raw.githubusercontent.com/algonents/wilhelm_renderer/master/images/shapes.png"></a> | shapes | All supported shape types |
| <a href="examples/text"><img width="120" alt="text" src="https://raw.githubusercontent.com/algonents/wilhelm_renderer/master/images/text.png"></a> | text | Text rendering with FreeType |
| <a href="examples/rotations"><img width="120" alt="rotations" src="https://raw.githubusercontent.com/algonents/wilhelm_renderer/master/images/rotations.png"></a> | rotations | Per-shape rotation and animation |
| <a href="examples/shapes_scaled"><img width="120" alt="shapes_scaled" src="https://raw.githubusercontent.com/algonents/wilhelm_renderer/master/images/shapes.png"></a> | shapes_scaled | Shapes with scroll-to-zoom scaling |
| <a href="examples/bouncing_balls"><img width="120" alt="bouncing_balls" src="https://raw.githubusercontent.com/algonents/wilhelm_renderer/master/images/bouncing_balls.png"></a> | bouncing_balls | 200 animated balls with per-shape rendering |
| <a href="examples/instancing"><img width="120" alt="instancing" src="https://raw.githubusercontent.com/algonents/wilhelm_renderer/master/images/instancing.png"></a> | instancing | 1,750 instanced circles with per-instance color |
| <a href="examples/bouncing_balls_instanced"><img width="120" alt="bouncing_balls_instanced" src="https://raw.githubusercontent.com/algonents/wilhelm_renderer/master/images/bouncing_balls_instanced.png"></a> | bouncing_balls_instanced | 10,000 animated balls with instanced rendering |
| <a href="examples/alpha_transparency"><img width="120" alt="alpha_transparency" src="https://raw.githubusercontent.com/algonents/wilhelm_renderer/master/images/alpha_transparency.png"></a> | alpha_transparency | Alpha blending and opacity control |
| <a href="examples/style_mutation"><img width="120" alt="style_mutation" src="https://raw.githubusercontent.com/algonents/wilhelm_renderer/master/images/style_mutation.png"></a> | style_mutation | Dynamic color changes and HSL cycling |
| <a href="examples/z_order"><img width="120" alt="z_order" src="https://raw.githubusercontent.com/algonents/wilhelm_renderer/master/images/z_order.png"></a> | z_order | Shape z-ordering independent of insertion order |
| <a href="examples/waypoints"><img width="120" alt="waypoints" src="https://raw.githubusercontent.com/algonents/wilhelm_renderer/master/images/waypoints.png"></a> | waypoints | WGS84 coordinates with Camera2D projection |
| <a href="examples/waypoints_instanced"><img width="120" alt="waypoints_instanced" src="https://raw.githubusercontent.com/algonents/wilhelm_renderer/master/images/waypoints_instanced.png"></a> | waypoints_instanced | Instanced waypoint markers with Camera2D |

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
wilhelm_renderer = "0.9"
```

## IDE Setup (C++ Language Server)

The C++ component uses CMake. To enable clangd support, generate a `compile_commands.json`:

```shell
cmake -S cpp -B build -DCMAKE_EXPORT_COMPILE_COMMANDS=ON
```

The `build/` directory is gitignored. Re-run only when `cpp/CMakeLists.txt` changes.

## Issues

Report issues on [GitHub](https://github.com/algonents/wilhelm-renderer/issues).
