use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Copy, PartialEq)]
pub enum ClockColor {
    Reset,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    Gray,
    DarkGray,
    LightRed,
    LightGreen,
    LightYellow,
    LightBlue,
    LightMagenta,
    LightCyan,
    White,
}

impl Default for ClockColor {
    fn default() -> Self {
        ClockColor::White
    }
}
