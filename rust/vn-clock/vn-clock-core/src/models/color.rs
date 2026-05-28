use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Copy, PartialEq)]
pub enum ClockColor {
    White,
    Red,
    Dynamic(u32),
}

impl ClockColor {
    pub fn from_id(id: u32) -> Self {
        ClockColor::Dynamic(id)
    }

    pub fn to_hex(&self) -> String {
        match self {
            ClockColor::Dynamic(id) => Self::to_hex_dynamic(*id),
            ClockColor::Red => "#ff0000".to_string(),
            ClockColor::White => "#ffffff".to_string(),
        }
    }

    pub fn to_hsl_dynamic(id: u32) -> (f64, f64, f64) {
        let mut hue = 0.0;

        if id > 0 {
            // Van der Corput sequence base 2:
            let mut n = id;
            let mut q = 0.0;
            let mut inv_b = 0.5;
            while n > 0 {
                q += (n % 2) as f64 * inv_b;
                n /= 2;
                inv_b /= 2.0;
            }
            hue = q * 360.0;
        }

        (hue, 1.0, 0.5) // Full saturation, medium lightness for bright colors
    }

    pub fn to_hex_dynamic(id: u32) -> String {
        let (h, s, l) = Self::to_hsl_dynamic(id);
        hsl_to_hex(h, s, l)
    }
}

fn hsl_to_hex(h: f64, s: f64, l: f64) -> String {
    let (r, g, b) = hsl_to_rgb(h / 360.0, s, l);
    format!("#{:02x}{:02x}{:02x}", (r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}

fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (f64, f64, f64) {
    if s == 0.0 {
        return (l, l, l);
    }

    let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
    let p = 2.0 * l - q;

    (
        hue_to_rgb(p, q, h + 1.0 / 3.0),
        hue_to_rgb(p, q, h),
        hue_to_rgb(p, q, h - 1.0 / 3.0),
    )
}

pub fn hue_to_rgb(p: f64, q: f64, mut t: f64) -> f64 {
    if t < 0.0 { t += 1.0; }
    if t > 1.0 { t -= 1.0; }
    if t < 1.0 / 6.0 { return p + (q - p) * 6.0 * t; }
    if t < 1.0 / 2.0 { return q; }
    if t < 2.0 / 3.0 { return p + (q - p) * (2.0 / 3.0 - t) * 6.0; }
    p
}

impl Default for ClockColor {
    fn default() -> Self {
        ClockColor::White
    }
}
