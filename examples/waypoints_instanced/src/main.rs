//! Waypoints example with instanced markers.
//!
//! Marker triangles share a single VAO/VBO and are rendered in one
//! instanced draw call. Labels remain individual ShapeRenderables
//! (each has unique text geometry).
//!
//! Run from crate directory: `cd examples/waypoints_instanced && cargo run`
//!
//! - Scroll wheel: zoom in/out (zooms toward cursor)

extern crate wilhelm_renderer;

use std::cell::Cell;
use wilhelm_renderer::core::{
    App, Camera2D, Color, Projection, Renderable, Vec2, Window,
    wgs84_to_mercator,
};
use wilhelm_renderer::graphics2d::shapes::{ShapeKind, ShapeRenderable, ShapeStyle, Text, Triangle};

thread_local! {
    static CAMERA_CENTER: Cell<(f32, f32)> = Cell::new((0.0, 0.0));
    static CAMERA_SCALE: Cell<f32> = Cell::new(1.0);
    static MOUSE_POS: Cell<(f64, f64)> = Cell::new((0.0, 0.0));
}

const FONT_PATH: &str = "../../fonts/DejaVuSans.ttf";
const FONT_SIZE: u32 = 11;
const LABEL_OFFSET_X: f32 = 8.0;
const LABEL_OFFSET_Y: f32 = -(FONT_SIZE as f32) / 2.0;

fn main() {
    let waypoint_data: &[(f32, f32, &str)] = &[
        (6.1432, 46.2044, "Geneva"),
        (6.6323, 46.5197, "Lausanne"),
        (7.4474, 46.9480, "Bern"),
        (8.2457, 46.8959, "Sarnen"),
        (8.5417, 47.3769, "Zurich"),
        (9.8355, 46.4908, "St-Moritz"),
    ];

    let mercator_positions: Vec<Vec2> = waypoint_data
        .iter()
        .map(|(lon, lat, _)| {
            let m = wgs84_to_mercator(Vec2::new(*lon, *lat));
            Vec2::new(m.x, -m.y)
        })
        .collect();

    let mut window = Window::new(
        "Waypoints — Instanced Markers",
        800, 600,
        Color::from_rgb(0.07, 0.13, 0.17),
    );

    // Compute bounding box for initial camera view
    let min_x = mercator_positions.iter().map(|p| p.x).fold(f32::MAX, f32::min);
    let max_x = mercator_positions.iter().map(|p| p.x).fold(f32::MIN, f32::max);
    let min_y = mercator_positions.iter().map(|p| p.y).fold(f32::MAX, f32::min);
    let max_y = mercator_positions.iter().map(|p| p.y).fold(f32::MIN, f32::max);

    let center = Vec2::new((min_x + max_x) / 2.0, (min_y + max_y) / 2.0);
    let range_x = max_x - min_x;
    let range_y = max_y - min_y;
    let initial_scale = (700.0 / range_x).min(500.0 / range_y);

    CAMERA_CENTER.with(|c| c.set((center.x, center.y)));
    CAMERA_SCALE.with(|s| s.set(initial_scale));

    // Scroll to zoom at cursor
    window.on_scroll(move |_, y_offset| {
        let zoom_factor = if y_offset > 0.0 { 1.1 } else { 1.0 / 1.1 };
        let mouse_pos = MOUSE_POS.with(|m| m.get());
        let center = CAMERA_CENTER.with(|c| c.get());
        let scale = CAMERA_SCALE.with(|s| s.get());

        let mut camera = Camera2D::new(
            Vec2::new(center.0, center.1),
            scale,
            Vec2::new(800.0, 600.0),
        );
        camera.zoom_at(zoom_factor, Vec2::new(mouse_pos.0 as f32, mouse_pos.1 as f32));

        let new_scale = camera.scale().clamp(initial_scale * 0.01, initial_scale * 100.0);
        camera.set_scale(new_scale);

        CAMERA_CENTER.with(|c| c.set((camera.center().x, camera.center().y)));
        CAMERA_SCALE.with(|s| s.set(camera.scale()));
    });

    window.on_cursor_position(move |x, y| {
        MOUSE_POS.with(|m| m.set((x, y)));
    });

    let color = Color::from_rgb(0.2, 0.6, 1.0);
    let triangle = Triangle::new([(-4.0, 3.0), (4.0, 3.0), (0.0, -5.0)]);
    let n = waypoint_data.len();

    // One instanced shape for all markers (1 draw call)
    let mut markers = ShapeRenderable::from_shape(
        0.0, 0.0,
        ShapeKind::Triangle(triangle),
        ShapeStyle::fill(color),
    );
    markers.create_multiple_instances(n);

    // Individual labels (each has unique text geometry)
    let mut labels: Vec<ShapeRenderable> = waypoint_data
        .iter()
        .map(|(_, _, name)| {
            ShapeRenderable::from_shape(
                0.0, 0.0,
                ShapeKind::Text(Text::new(*name, FONT_PATH, FONT_SIZE)),
                ShapeStyle::fill(color),
            )
        })
        .collect();

    let mut screen_positions = vec![Vec2::new(0.0, 0.0); n];

    let mut app = App::new(window);

    app.on_render(move |renderer| {
        let center = CAMERA_CENTER.with(|c| c.get());
        let scale = CAMERA_SCALE.with(|s| s.get());

        let camera = Camera2D::new(
            Vec2::new(center.0, center.1),
            scale,
            Vec2::new(800.0, 600.0),
        );

        // Project all waypoints to screen coordinates
        for (i, mercator) in mercator_positions.iter().enumerate() {
            screen_positions[i] = camera.world_to_screen(*mercator);
        }

        // Render all markers in one instanced draw call
        markers.set_instance_positions(&screen_positions);
        markers.render(renderer);

        // Render labels individually
        for (label, pos) in labels.iter_mut().zip(screen_positions.iter()) {
            label.set_position(pos.x + LABEL_OFFSET_X, pos.y + LABEL_OFFSET_Y);
            label.render(renderer);
        }
    });

    println!("Waypoints — Instanced Markers");
    println!("  Scroll: zoom in/out (zooms toward cursor)");
    println!("  Markers: 1 instanced draw call for {} triangles", n);
    println!("  Labels:  {} individual draw calls", n);
    println!();
    println!("Waypoints: Geneva, Lausanne, Bern, Sarnen, Zurich, St-Moritz");

    app.run();
}
