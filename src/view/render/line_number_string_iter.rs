use crate::buffer::Buffer;

const PADDING_SIZE: usize = 2;
pub struct LineNumberStringIter {
    current: usize,
    max_width: usize,
}

impl LineNumberStringIter {
    pub fn new(buffer: &Buffer, offset: usize) -> LineNumberStringIter {
        let line_count = buffer.line_count();
        LineNumberStringIter {
            current: offset,
            max_width: line_count.to_string().len(),
        }
    }

    pub fn width(&self) -> usize {
        self.max_width + PADDING_SIZE
    }
}

impl Iterator for LineNumberStringIter {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.current += 1;
        Some(format!(
            " {:>width$} ",
            self.current,
            width = self.max_width
        ))
    }
}
