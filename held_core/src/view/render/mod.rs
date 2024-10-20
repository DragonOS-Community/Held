use std::str::Chars;

use cell::Cell;

use crate::utils::{position::Position, rectangle::Rectangle};

use super::{colors::Colors, style::CharStyle};

pub mod cell;
pub struct ContentRenderBuffer {
    pub rectangle: Rectangle,
    pub cells: Vec<Option<Cell>>,
}

impl ContentRenderBuffer {
    pub fn new(rectangle: Rectangle) -> ContentRenderBuffer {
        let cells = vec![None; rectangle.height * rectangle.width];
        ContentRenderBuffer { rectangle, cells }
    }

    pub fn set_cell(&mut self, position: Position, cell: Option<Cell>) {
        let index = position.line * self.rectangle.width + position.offset;
        if index < self.cells.len() {
            self.cells[index] = cell;
        }
    }

    pub fn put_buffer(
        &mut self,
        position: Position,
        buffer: String,
        style: CharStyle,
        colors: Colors,
    ) {
        let mut line = position.line;
        let mut offset = position.offset;
        for c in buffer.chars() {
            let index = line * self.rectangle.width + offset;
            if index < self.cells.len() {
                let cell = Cell {
                    content: c,
                    colors,
                    style,
                };
                self.cells[index] = Some(cell);
                offset += 1;
                if offset == self.rectangle.width {
                    line += 1;
                    offset = 0;
                }
            } else {
                break;
            }
        }
    }
}
