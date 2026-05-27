use vn_clock_core::models::ClockColor;
use ratatui::style::Color;

pub fn to_ratatui_color(clock_color: ClockColor) -> Color {
    match clock_color {
        ClockColor::Dynamic(id) => {
            let (h, s, l) = ClockColor::to_hsl_dynamic(id);
            hsl_to_ratatui_rgb(h / 360.0, s, l)
        }
        ClockColor::Red => Color::Red,
        ClockColor::White => Color::White,
    }
}

pub fn from_ratatui_color(color: Color) -> ClockColor {
    match color {
        Color::Red => ClockColor::Red,
        Color::White => ClockColor::White,
        _ => ClockColor::White,
    }
}

fn hsl_to_ratatui_rgb(h: f64, s: f64, l: f64) -> Color {
    if s == 0.0 {
        let val = (l * 255.0) as u8;
        return Color::Rgb(val, val, val);
    }

    let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
    let p = 2.0 * l - q;

    let r = hue_to_rgb(p, q, h + 1.0 / 3.0);
    let g = hue_to_rgb(p, q, h);
    let b = hue_to_rgb(p, q, h - 1.0 / 3.0);

    Color::Rgb((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}

fn hue_to_rgb(p: f64, q: f64, mut t: f64) -> f64 {
    if t < 0.0 { t += 1.0; }
    if t > 1.0 { t -= 1.0; }
    if t < 1.0 / 6.0 { return p + (q - p) * 6.0 * t; }
    if t < 1.0 / 2.0 { return q; }
    if t < 2.0 / 3.0 { return p + (q - p) * (2.0 / 3.0 - t) * 6.0; }
    p
}
