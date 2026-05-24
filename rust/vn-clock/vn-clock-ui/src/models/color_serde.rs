use vn_clock_core::models::ClockColor;
use ratatui::style::Color;

pub fn to_ratatui_color(clock_color: ClockColor) -> Color {
    match clock_color {
        ClockColor::Reset => Color::Reset,
        ClockColor::Black => Color::Black,
        ClockColor::Red => Color::Red,
        ClockColor::Green => Color::Green,
        ClockColor::Yellow => Color::Yellow,
        ClockColor::Blue => Color::Blue,
        ClockColor::Magenta => Color::Magenta,
        ClockColor::Cyan => Color::Cyan,
        ClockColor::Gray => Color::Gray,
        ClockColor::DarkGray => Color::DarkGray,
        ClockColor::LightRed => Color::LightRed,
        ClockColor::LightGreen => Color::LightGreen,
        ClockColor::LightYellow => Color::LightYellow,
        ClockColor::LightBlue => Color::LightBlue,
        ClockColor::LightMagenta => Color::LightMagenta,
        ClockColor::LightCyan => Color::LightCyan,
        ClockColor::White => Color::White,
    }
}

pub fn from_ratatui_color(color: Color) -> ClockColor {
    match color {
        Color::Reset => ClockColor::Reset,
        Color::Black => ClockColor::Black,
        Color::Red => ClockColor::Red,
        Color::Green => ClockColor::Green,
        Color::Yellow => ClockColor::Yellow,
        Color::Blue => ClockColor::Blue,
        Color::Magenta => ClockColor::Magenta,
        Color::Cyan => ClockColor::Cyan,
        Color::Gray => ClockColor::Gray,
        Color::DarkGray => ClockColor::DarkGray,
        Color::LightRed => ClockColor::LightRed,
        Color::LightGreen => ClockColor::LightGreen,
        Color::LightYellow => ClockColor::LightYellow,
        Color::LightBlue => ClockColor::LightBlue,
        Color::LightMagenta => ClockColor::LightMagenta,
        Color::LightCyan => ClockColor::LightCyan,
        Color::White => ClockColor::White,
        _ => ClockColor::White,
    }
}
