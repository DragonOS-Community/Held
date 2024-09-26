use std::{
    cell::RefCell,
    ops::{Deref, DerefMut},
    rc::Rc,
};

use unicode_segmentation::UnicodeSegmentation;

use super::{GapBuffer, Position};

#[derive(Clone)]
pub struct Cursor {
    pub data: Rc<RefCell<GapBuffer>>,
    pub position: Position,
    /// 限制上下移动时，offset不会溢出
    sticky_offset: usize,
}

impl Deref for Cursor {
    type Target = Position;

    fn deref(&self) -> &Position {
        &self.position
    }
}

impl DerefMut for Cursor {
    fn deref_mut(&mut self) -> &mut Position {
        &mut self.position
    }
}

impl Cursor {
    pub fn new(data: Rc<RefCell<GapBuffer>>, position: Position) -> Cursor {
        Cursor {
            data,
            position,
            sticky_offset: position.offset,
        }
    }

    pub fn move_to(&mut self, position: Position) -> bool {
        if self.data.borrow().in_bounds(&position) {
            self.position = position;

            // 缓冲当前offset
            self.sticky_offset = position.offset;

            return true;
        }
        false
    }

    pub fn move_up(&mut self) {
        if self.line == 0 {
            return;
        }

        let target_line = self.line - 1;
        let new_position = Position {
            line: target_line,
            offset: self.sticky_offset,
        };

        if !self.move_to(new_position) {
            let mut target_offset = 0;
            for (line_number, line) in self.data.borrow().to_string().lines().enumerate() {
                if line_number == target_line {
                    target_offset = line.graphemes(true).count();
                    break;
                }
            }
            self.move_to(Position {
                line: target_line,
                offset: target_offset,
            });

            self.sticky_offset = new_position.offset;
        }
    }

    pub fn move_down(&mut self) {
        let target_line = self.line + 1;
        let new_position = Position {
            line: target_line,
            offset: self.sticky_offset,
        };

        if !self.move_to(new_position) {
            let mut target_offset = 0;
            for (line_number, line) in self.data.borrow().to_string().lines().enumerate() {
                if line_number == target_line {
                    target_offset = line.graphemes(true).count();
                }
            }
            self.move_to(Position {
                line: target_line,
                offset: target_offset,
            });

            self.sticky_offset = new_position.offset;
        }
    }

    pub fn move_left(&mut self) {
        if self.offset == 0 {
            return;
        }

        let new_position = Position {
            line: self.line,
            offset: self.offset - 1,
        };
        self.move_to(new_position);
    }

    pub fn move_right(&mut self) {
        let new_position = Position {
            line: self.line,
            offset: self.offset + 1,
        };
        self.move_to(new_position);
    }

    pub fn move_to_start_of_line(&mut self) {
        let new_position = Position {
            line: self.line,
            offset: 0,
        };
        self.move_to(new_position);
    }

    pub fn move_to_end_of_line(&mut self) {
        let data = self.data.borrow().to_string();
        let current_line = data.lines().nth(self.line);
        if let Some(line) = current_line {
            let new_position = Position {
                line: self.line,
                offset: line.graphemes(true).count(),
            };
            self.move_to(new_position);
        }
    }

    pub fn move_to_last_line(&mut self) {
        // Figure out the number and length of the last line.
        let mut line = 0;
        let mut length = 0;
        for c in self.data.borrow().to_string().graphemes(true) {
            if c == "\n" {
                line += 1;
                length = 0;
            } else {
                length += 1;
            }
        }

        let target_position = if length < self.sticky_offset {
            // Current offset is beyond the last line's length; move to the end of it.
            Position {
                line,
                offset: length,
            }
        } else {
            // Current offset is available on the last line; go there.
            Position {
                line,
                offset: self.sticky_offset,
            }
        };
        self.move_to(target_position);
    }

    pub fn move_to_first_line(&mut self) {
        // Figure out the length of the first line.
        let length = self
            .data
            .borrow()
            .to_string()
            .lines()
            .nth(0)
            .map(|line| line.graphemes(true).count())
            .unwrap_or(0);

        let target_position = if length < self.sticky_offset {
            // Current offset is beyond the first line's length; move to the end of it.
            Position {
                line: 0,
                offset: length,
            }
        } else {
            // Current offset is available on the first line; go there.
            Position {
                line: 0,
                offset: self.sticky_offset,
            }
        };
        self.move_to(target_position);
    }
}

#[cfg(test)]
mod tests {
    use crate::buffer::{Cursor, GapBuffer, Position};
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn move_up_goes_to_eol_if_offset_would_be_out_of_range() {
        let buffer = Rc::new(RefCell::new(GapBuffer::new(
            "This is a test.\nAnother line that is longer.".to_string(),
        )));
        let mut cursor = Cursor::new(
            buffer,
            Position {
                line: 1,
                offset: 20,
            },
        );
        cursor.move_up();
        assert_eq!(cursor.line, 0);
        assert_eq!(cursor.offset, 15);
    }

    #[test]
    fn move_down_goes_to_eol_if_offset_would_be_out_of_range() {
        let buffer = Rc::new(RefCell::new(GapBuffer::new(
            "Another line that is longer.\nThis is a test.".to_string(),
        )));
        let mut cursor = Cursor::new(
            buffer,
            Position {
                line: 0,
                offset: 20,
            },
        );
        cursor.move_down();
        assert_eq!(cursor.line, 1);
        assert_eq!(cursor.offset, 15);
    }

    #[test]
    fn move_up_counts_graphemes_as_a_single_offset() {
        let buffer = Rc::new(RefCell::new(GapBuffer::new(
            "First नी\nSecond line".to_string(),
        )));
        let mut cursor = Cursor::new(
            buffer,
            Position {
                line: 1,
                offset: 11,
            },
        );
        cursor.move_up();
        assert_eq!(cursor.line, 0);
        assert_eq!(cursor.offset, 7);
    }

    #[test]
    fn move_down_counts_graphemes_as_a_single_offset() {
        let buffer = Rc::new(RefCell::new(GapBuffer::new(
            "First line\nSecond नी".to_string(),
        )));
        let mut cursor = Cursor::new(
            buffer,
            Position {
                line: 0,
                offset: 10,
            },
        );
        cursor.move_down();
        assert_eq!(cursor.line, 1);
        assert_eq!(cursor.offset, 8);
    }

    #[test]
    fn move_up_persists_offset_across_shorter_lines() {
        let buffer = Rc::new(RefCell::new(GapBuffer::new(
            "First line that is longer.\nThis is a test.\nAnother line that is longer.".to_string(),
        )));
        let mut cursor = Cursor::new(
            buffer,
            Position {
                line: 2,
                offset: 20,
            },
        );
        cursor.move_up();
        cursor.move_up();
        assert_eq!(cursor.line, 0);
        assert_eq!(cursor.offset, 20);
    }

    #[test]
    fn move_down_persists_offset_across_shorter_lines() {
        let buffer = Rc::new(RefCell::new(GapBuffer::new(
            "First line that is longer.\nThis is a test.\nAnother line that is longer.".to_string(),
        )));
        let mut cursor = Cursor::new(
            buffer,
            Position {
                line: 0,
                offset: 20,
            },
        );
        cursor.move_down();
        cursor.move_down();
        assert_eq!(cursor.line, 2);
        assert_eq!(cursor.offset, 20);
    }

    #[test]
    fn move_to_sets_persisted_offset() {
        let buffer = Rc::new(RefCell::new(GapBuffer::new(
            "First line that is longer.\nThis is a test.\nAnother line that is longer.".to_string(),
        )));
        let mut cursor = Cursor::new(
            buffer,
            Position {
                line: 0,
                offset: 20,
            },
        );
        cursor.move_to(Position { line: 1, offset: 5 });
        cursor.move_down();
        assert_eq!(cursor.line, 2);
        assert_eq!(cursor.offset, 5);
    }

    #[test]
    fn move_to_start_of_line_sets_offset_to_zero() {
        let buffer = Rc::new(RefCell::new(GapBuffer::new(
            "This is a test.\nAnother line.".to_string(),
        )));
        let mut cursor = Cursor::new(buffer, Position { line: 1, offset: 5 });
        cursor.move_to_start_of_line();
        assert_eq!(cursor.line, 1);
        assert_eq!(cursor.offset, 0);
    }

    #[test]
    fn move_to_end_of_line_counts_graphemes_as_a_single_offset() {
        let buffer = Rc::new(RefCell::new(GapBuffer::new("First नी".to_string())));
        let mut cursor = Cursor::new(buffer, Position { line: 0, offset: 0 });
        cursor.move_to_end_of_line();
        assert_eq!(cursor.line, 0);
        assert_eq!(cursor.offset, 7);
    }

    #[test]
    fn move_to_end_of_line_sets_offset_the_line_length() {
        let buffer = Rc::new(RefCell::new(GapBuffer::new(
            "This is a test.\nAnother line.".to_string(),
        )));
        let mut cursor = Cursor::new(buffer, Position { line: 0, offset: 5 });
        cursor.move_to_end_of_line();
        assert_eq!(cursor.line, 0);
        assert_eq!(cursor.offset, 15);
    }

    #[test]
    fn move_up_does_nothing_if_at_the_start_of_line() {
        let buffer = Rc::new(RefCell::new(GapBuffer::new("This is a test.".to_string())));
        let mut cursor = Cursor::new(buffer, Position { line: 0, offset: 0 });
        cursor.move_up();
        assert_eq!(cursor.line, 0);
        assert_eq!(cursor.offset, 0);
    }

    #[test]
    fn move_left_does_nothing_if_at_the_start_of_line() {
        let buffer = Rc::new(RefCell::new(GapBuffer::new("This is a test.".to_string())));
        let mut cursor = Cursor::new(buffer, Position { line: 0, offset: 0 });
        cursor.move_left();
        assert_eq!(cursor.line, 0);
        assert_eq!(cursor.offset, 0);
    }

    #[test]
    fn move_to_last_line_counts_graphemes_as_a_single_offset() {
        let buffer = Rc::new(RefCell::new(GapBuffer::new(
            "First line\nLast नी".to_string(),
        )));
        let mut cursor = Cursor::new(
            buffer,
            Position {
                line: 0,
                offset: 10,
            },
        );
        cursor.move_to_last_line();
        assert_eq!(cursor.line, 1);
        assert_eq!(cursor.offset, 6);
    }

    #[test]
    fn move_to_last_line_moves_to_same_offset_on_last_line() {
        let buffer = Rc::new(RefCell::new(GapBuffer::new(
            "first\nsecond\nlast".to_string(),
        )));
        let mut cursor = Cursor::new(buffer, Position { line: 0, offset: 2 });
        cursor.move_to_last_line();
        assert_eq!(cursor.line, 2);
        assert_eq!(cursor.offset, 2);
    }

    #[test]
    fn move_to_last_line_moves_to_end_of_last_line_if_offset_would_be_out_of_range() {
        let buffer = Rc::new(RefCell::new(GapBuffer::new(
            "first\nsecond\nlast".to_string(),
        )));
        let mut cursor = Cursor::new(buffer, Position { line: 0, offset: 5 });
        cursor.move_to_last_line();
        assert_eq!(cursor.line, 2);
        assert_eq!(cursor.offset, 4);
    }

    #[test]
    fn move_to_last_line_moves_last_line_when_it_is_a_trailing_newline() {
        let buffer = Rc::new(RefCell::new(GapBuffer::new(
            "first\nsecond\nlast\n".to_string(),
        )));
        let mut cursor = Cursor::new(buffer, Position { line: 0, offset: 2 });
        cursor.move_to_last_line();
        assert_eq!(cursor.line, 3);
        assert_eq!(cursor.offset, 0);
    }

    #[test]
    fn move_to_first_line_counts_graphemes_as_a_single_offset() {
        let buffer = Rc::new(RefCell::new(GapBuffer::new(
            "First नी\nLast line".to_string(),
        )));
        let mut cursor = Cursor::new(buffer, Position { line: 0, offset: 9 });
        cursor.move_to_first_line();
        assert_eq!(cursor.line, 0);
        assert_eq!(cursor.offset, 7);
    }

    #[test]
    fn move_to_first_line_moves_to_same_offset_on_first_line() {
        let buffer = Rc::new(RefCell::new(GapBuffer::new(
            "first\nsecond\nlast".to_string(),
        )));
        let mut cursor = Cursor::new(buffer, Position { line: 1, offset: 2 });
        cursor.move_to_first_line();
        assert_eq!(cursor.line, 0);
        assert_eq!(cursor.offset, 2);
    }

    #[test]
    fn move_to_first_line_moves_to_end_of_first_line_if_offset_would_be_out_of_range() {
        let buffer = Rc::new(RefCell::new(GapBuffer::new(
            "first\nsecond\nlast".to_string(),
        )));
        let mut cursor = Cursor::new(buffer, Position { line: 1, offset: 6 });
        cursor.move_to_first_line();
        assert_eq!(cursor.line, 0);
        assert_eq!(cursor.offset, 5);
    }
}
