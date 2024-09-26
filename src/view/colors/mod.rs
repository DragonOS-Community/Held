pub mod colors;
pub mod map;

use crossterm::style::Color;
use syntect::highlighting::Color as RGBAColor;

pub fn to_rgb(highlight_color: RGBAColor) -> Color {
    Color::Rgb {
        r: highlight_color.r,
        g: highlight_color.g,
        b: highlight_color.b,
    }
}
