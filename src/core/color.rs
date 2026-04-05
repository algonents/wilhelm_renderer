#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Color{
    r:f32,
    g:f32,
    b:f32,
    a: f32
}

impl Color{
    pub fn from_rgb(r:f32, g:f32, b:f32)->Self{
        Color {r, g, b, a:1.0}
    }

    pub fn from_rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Color { r, g, b, a }
    }

    /// Create a color from HSL (hue 0-360, saturation 0-1, lightness 0-1).
    pub fn from_hsl(h: f32, s: f32, l: f32) -> Self {
        let (r, g, b) = hsl_to_rgb(h, s, l);
        Color { r, g, b, a: 1.0 }
    }

    /// Create a color from HSLA (hue 0-360, saturation 0-1, lightness 0-1, alpha 0-1).
    pub fn from_hsla(h: f32, s: f32, l: f32, a: f32) -> Self {
        let (r, g, b) = hsl_to_rgb(h, s, l);
        Color { r, g, b, a }
    }

    pub fn red_value(&self)->f32{
        self.r
    }

    pub fn green_value(&self)->f32{
        self.g
    }

    pub fn blue_value(&self)->f32{
        self.b
    }

    pub fn alpha(&self)->f32 { self.a }

    pub fn black() -> Self {
        Color::from_rgb(0.0, 0.0, 0.0)
    }

    pub fn white() -> Self {
        Color::from_rgb(1.0, 1.0, 1.0)
    }

    pub fn red() -> Self {
        Color::from_rgb(1.0, 0.0, 0.0)
    }

    pub fn green() -> Self {
        Color::from_rgb(0.0, 1.0, 0.0)
    }

    pub fn blue() -> Self {
        Color::from_rgb(0.0, 0.0, 1.0)
    }


    pub fn to_hex(&self) -> String {
        format!(
            "#{:02X}{:02X}{:02X}",
            (self.r * 255.0) as u8,
            (self.g * 255.0) as u8,
            (self.b * 255.0) as u8,
        )
    }
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (f32, f32, f32) {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;

    let (r, g, b) = match h as u32 {
        0..=59 => (c, x, 0.0),
        60..=119 => (x, c, 0.0),
        120..=179 => (0.0, c, x),
        180..=239 => (0.0, x, c),
        240..=299 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    (r + m, g + m, b + m)
}