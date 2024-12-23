use crate::buffer::Buffer;

use super::Operation;

pub struct OperationGroup {
    operations: Vec<Box<dyn Operation>>,
}

impl Operation for OperationGroup {
    fn run(&mut self, buffer: &mut Buffer) {
        for operation in &mut self.operations {
            operation.run(buffer);
        }
    }

    fn reverse(&mut self, buffer: &mut Buffer) {
        for operation in &mut self.operations.iter_mut().rev() {
            operation.reverse(buffer);
        }
    }

    fn clone_operation(&self) -> Box<dyn Operation> {
        Box::new(OperationGroup {
            operations: self
                .operations
                .iter()
                .map(|o| (*o).clone_operation())
                .collect(),
        })
    }
}

impl OperationGroup {
    pub fn new() -> OperationGroup {
        OperationGroup {
            operations: Vec::new(),
        }
    }

    pub fn add(&mut self, operation: Box<dyn Operation>) {
        self.operations.push(operation);
    }

    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }
}

impl Buffer {
    // 开启一个操作组
    pub fn start_operation_group(&mut self) {
        // Create an operation group, if one doesn't already exist.
        match self.operation_group {
            Some(_) => (),
            None => {
                self.operation_group = Some(OperationGroup::new());
            }
        }
    }

    pub fn end_operation_group(&mut self) {
        // Push an open operation group on to the history stack, if one exists.
        if let Some(group) = self.operation_group.take() {
            if !group.is_empty() {
                self.history.add(Box::new(group))
            }
        }
    }
}
