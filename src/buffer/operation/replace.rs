use std::{cell::RefCell, rc::Rc};

use crate::buffer::{cursor::Cursor, Buffer, GapBuffer, Position};

use super::Operation;

#[derive(Clone)]
pub struct Replace {
    old_content: String,
    new_content: String,
}

impl Operation for Replace {
    fn run(&mut self, buffer: &mut Buffer) {
        replace_content(self.new_content.clone(), buffer);
    }

    fn reverse(&mut self, buffer: &mut Buffer) {
        replace_content(self.old_content.clone(), buffer);
    }

    fn clone_operation(&self) -> Box<dyn Operation> {
        Box::new(self.clone())
    }
}

impl Replace {
    /// Creates a new empty insert operation.
    pub fn new(old_content: String, new_content: String) -> Replace {
        Replace {
            old_content,
            new_content,
        }
    }
}

impl Buffer {
    /// Replaces the buffer's contents with the provided data. This method will
    /// make best efforts to retain the full cursor position, then cursor line,
    /// and will ultimately fall back to resetting the cursor to its initial
    /// (0,0) position if these fail. The buffer's ID, syntax definition, and
    /// change callback are always persisted.
    ///
    /// <div class="warning">
    ///   As this is a reversible operation, both the before and after buffer
    ///   contents are kept in-memory, which for large buffers may be relatively
    ///   expensive. To help avoid needless replacements, this method will
    ///   ignore requests that don't actually change content. Despite this, use
    ///   this operation judiciously; it is designed for wholesale replacements
    ///   (e.g. external formatting tools) that cannot be broken down into
    ///   selective delete/insert operations.
    /// </div>
    ///
    /// # Examples
    ///
    /// ```
    /// use scribe::buffer::{Buffer, Position};
    ///
    /// let mut buffer = Buffer::new();
    /// buffer.insert("scribe\nlibrary\n");
    /// buffer.cursor.move_to(Position { line: 1, offset: 1 });
    /// buffer.replace("new\ncontent");
    ///
    /// assert_eq!(buffer.data(), "new\ncontent");
    /// assert_eq!(*buffer.cursor, Position{ line: 1, offset: 1 });
    /// ```
    pub fn replace<T: Into<String> + AsRef<str>>(&mut self, content: T) {
        let old_content = self.data();

        // Ignore replacements that don't change content.
        if content.as_ref() == old_content {
            return;
        }

        // Build and run an insert operation.
        let mut op = Replace::new(self.data(), content.into());
        op.run(self);

        // Store the operation in the history object so that it can be undone.
        match self.operation_group {
            Some(ref mut group) => group.add(Box::new(op)),
            None => self.history.add(Box::new(op)),
        };
    }
}

fn replace_content(content: String, buffer: &mut Buffer) {
    // Create a new gap buffer and associated cursor with the new content.
    let data = Rc::new(RefCell::new(GapBuffer::new(content)));
    let mut cursor = Cursor::new(data.clone(), Position { line: 0, offset: 0 });

    // Try to retain cursor position or line of the current gap buffer.
    if !cursor.move_to(*buffer.cursor) {
        cursor.move_to(Position {
            line: buffer.cursor.line,
            offset: 0,
        });
    }

    // Do the replacement.
    buffer.data = data;
    buffer.cursor = cursor;

    // Run the change callback, if present.
    if let Some(ref callback) = buffer.change_callback {
        callback(Position::default())
    }
}
