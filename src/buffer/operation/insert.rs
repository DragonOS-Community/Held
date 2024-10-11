use unicode_segmentation::UnicodeSegmentation;

use crate::buffer::{Buffer, Position, Range};

use super::Operation;

#[derive(Clone)]
pub struct Insert {
    content: String,
    position: Position,
}

impl Operation for Insert {
    fn run(&mut self, buffer: &mut Buffer) {
        buffer
            .data
            .borrow_mut()
            .insert(&self.content, &self.position);

        // Run the change callback, if present.
        if let Some(ref callback) = buffer.change_callback {
            callback(self.position)
        }
    }

    fn reverse(&mut self, buffer: &mut Buffer) {
        // The line count of the content tells us the line number for the end of the
        // range (just add the number of new lines to the starting line).
        let line_count = self.content.chars().filter(|&c| c == '\n').count() + 1;
        let end_line = self.position.line + line_count - 1;

        let end_offset = if line_count == 1 {
            // If there's only one line, the range starts and ends on the same line, and so its
            // offset needs to take the original insertion location into consideration.
            self.position.offset + self.content.graphemes(true).count()
        } else {
            // If there are multiple lines, the end of the range doesn't
            // need to consider the original insertion location.
            match self.content.split('\n').last() {
                Some(line) => line.graphemes(true).count(),
                None => return,
            }
        };

        // Now that we have the required info,
        // build the end position and total range.
        let end_position = Position {
            line: end_line,
            offset: end_offset,
        };
        let range = Range::new(self.position, end_position);

        // Remove the content we'd previously inserted.
        buffer.data.borrow_mut().delete(&range);

        // Run the change callback, if present.
        if let Some(ref callback) = buffer.change_callback {
            callback(self.position)
        }
    }

    fn clone_operation(&self) -> Box<dyn Operation> {
        Box::new(self.clone())
    }
}

impl Insert {
    /// Creates a new empty insert operation.
    pub fn new(content: String, position: Position) -> Insert {
        Insert { content, position }
    }
}

impl Buffer {
    /// Inserts `data` into the buffer at the cursor position.
    ///
    /// # Examples
    ///
    /// ```
    /// use scribe::Buffer;
    ///
    /// let mut buffer = Buffer::new();
    /// buffer.insert("scribe");
    /// assert_eq!(buffer.data(), "scribe");
    /// ```
    pub fn insert<T: Into<String>>(&mut self, data: T) {
        // Build and run an insert operation.
        let mut op = Insert::new(data.into(), self.cursor.position);

        op.run(self);

        // Store the operation in the history
        // object so that it can be undone.
        match self.operation_group {
            Some(ref mut group) => group.add(Box::new(op)),
            None => self.history.add(Box::new(op)),
        };
    }
}
