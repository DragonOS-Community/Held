use lazy_static::lazy_static;

use crate::utils::buffer::LineBuffer;
use std::sync::RwLock;
pub struct Register {
    data: RwLock<Vec<LineBuffer>>,
}

impl Register {
    pub fn new() -> Self {
        Register {
            data: RwLock::new(Vec::new()),
        }
    }

    pub fn push(&self, buffer: LineBuffer) {
        self.data.write().unwrap().push(buffer);
    }

    pub fn pop(&self) -> Option<LineBuffer> {
        self.data.write().unwrap().pop()
    }
}

lazy_static! (
    pub static ref REG: Register = Register::new();
);
