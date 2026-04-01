use wilhelm_renderer::core::engine::glfw::{
    GLFW_KEY_ESCAPE, GLFW_MOD_ALT, GLFW_MOD_CONTROL, GLFW_MOD_SHIFT, GLFW_PRESS, GLFW_RELEASE,
    GLFW_REPEAT,
};
use wilhelm_renderer::core::{Color, Window};

fn main() {
    let mut window = Window::new("Keyboard Demo", 800, 600, Color::from_rgb(0.1, 0.1, 0.1));

    window.on_key(move |key, _scancode, action, mods| {
        let action_str = match action {
            GLFW_PRESS => "pressed",
            GLFW_RELEASE => "released",
            GLFW_REPEAT => "repeat",
            _ => "unknown",
        };

        // Check modifiers
        let ctrl = (mods & GLFW_MOD_CONTROL) != 0;
        let shift = (mods & GLFW_MOD_SHIFT) != 0;
        let alt = (mods & GLFW_MOD_ALT) != 0;

        // For printable keys (A-Z), convert to char
        let key_name = if (65..=90).contains(&key) {
            format!("{}", (key as u8) as char)
        } else if (48..=57).contains(&key) {
            format!("{}", (key as u8) as char)
        } else {
            format!("key {}", key)
        };

        println!(
            "{} {} (ctrl={}, shift={}, alt={})",
            key_name, action_str, ctrl, shift, alt
        );

        // Example: Ctrl+O shortcut
        if action == GLFW_PRESS && ctrl && key == 79 {
            // 'O'
            println!("Ctrl+O pressed - Open file!");
        }

        // Escape to quit (example only - window close handled elsewhere)
        if action == GLFW_PRESS && key == GLFW_KEY_ESCAPE {
            println!("Escape pressed");
        }
    });

    while !window.window_should_close() {
        window.clear_color();
        window.swap_buffers();
        window.poll_events();
    }
}
