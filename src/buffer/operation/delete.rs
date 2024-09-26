use crate::buffer::{Buffer, Position, Range};

use super::Operation;

#[derive(Clone)]
pub struct Delete {
    content: Option<String>,
    range: Range,
}

impl Operation for Delete {
    fn run(&mut self, buffer: &mut crate::buffer::Buffer) {
        self.content = buffer.data.borrow().read(&self.range);

        buffer.data.borrow_mut().delete(&self.range);

        if let Some(ref callback) = buffer.change_callback {
            callback(self.range.start());
        }
    }

    fn reverse(&mut self, buffer: &mut crate::buffer::Buffer) {
        if let Some(ref content) = self.content {
            buffer
                .data
                .borrow_mut()
                .insert(content, &self.range.start());

            // Run the change callback, if present.
            if let Some(ref callback) = buffer.change_callback {
                callback(self.range.start())
            }
        }
    }

    fn clone_operation(&self) -> Box<dyn Operation> {
        Box::new(self.clone())
    }
}

impl Delete {
    /// Creates a new empty delete operation.
    pub fn new(range: Range) -> Delete {
        Delete {
            content: None,
            range,
        }
    }
}

impl Buffer {
    // 删除当前cursor指向的字符
    pub fn delete(&mut self) {
        let mut end = Position {
            line: self.cursor.line,
            offset: self.cursor.offset + 1,
        };

        // 下一行的行首
        if !self.data.borrow().in_bounds(&end) {
            end.line += 1;
            end.offset = 0;
        }

        let start = self.cursor.position;
        self.delete_range(Range::new(start, end));
    }

    pub fn delete_range(&mut self, range: Range) {
        // Build and run a delete operation.
        let mut op = Delete::new(range);
        op.run(self);

        // Store the operation in the history
        // object so that it can be undone.
        match self.operation_group {
            Some(ref mut group) => group.add(Box::new(op)),
            None => self.history.add(Box::new(op)),
        };
    }
}
