use std::{borrow::Cow, cell::RefCell, rc::Rc};

use unicode_segmentation::UnicodeSegmentation;

use crate::{
    util::position::Position,
    view::{colors::colors::Colors, style::CharStyle},
};

#[derive(Debug, Clone, PartialEq)]
pub struct Cell<'a> {
    pub content: Cow<'a, str>,
    pub colors: Colors,
    pub style: CharStyle,
}

impl<'c> Default for Cell<'c> {
    fn default() -> Self {
        Self {
            content: " ".into(),
            colors: Default::default(),
            style: Default::default(),
        }
    }
}

pub struct RenderBuffer<'a> {
    width: usize,
    height: usize,
    cells: Vec<Cell<'a>>,
}

impl<'a> RenderBuffer<'a> {
    pub fn new(width: usize, height: usize) -> RenderBuffer<'a> {
        RenderBuffer {
            width,
            height,
            cells: vec![Cell::default(); width * height],
        }
    }

    pub fn set_cell(&mut self, position: Position, cell: Cell<'a>) {
        let index = position.line * self.width + position.offset;

        if index < self.cells.len() {
            self.cells[index] = cell;
        }
    }

    pub fn clear(&mut self) {
        self.cells = vec![Cell::default(); self.width * self.height];
    }

    pub fn iter(&self) -> RenderBufferIter {
        RenderBufferIter::new(self)
    }
}

pub struct RenderBufferIter<'a> {
    index: usize,
    width: usize,
    cells: &'a Vec<Cell<'a>>,
}

impl<'a> RenderBufferIter<'a> {
    pub fn new(render_buffer: &'a RenderBuffer) -> RenderBufferIter<'a> {
        RenderBufferIter {
            index: 0,
            width: render_buffer.width,
            cells: &render_buffer.cells,
        }
    }
}

impl<'a> Iterator for RenderBufferIter<'a> {
    type Item = (Position, &'a Cell<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.cells.len() {
            let position = Position {
                line: self.index / self.width,
                offset: self.index % self.width,
            };
            let cell = &self.cells[self.index];
            self.index += cell.content.graphemes(true).count().max(1);

            Some((position, cell))
        } else {
            None
        }
    }
}
