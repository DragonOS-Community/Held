use crate::view::{colors::Colors, style::CharStyle};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Cell {
    pub content: char,
    pub colors: Colors,
    pub style: CharStyle,
}

impl Cell {
    pub fn new(content: char, colors: Colors, style: CharStyle) -> Cell {
        Cell {
            content,
            colors,
            style,
        }
    }
}
