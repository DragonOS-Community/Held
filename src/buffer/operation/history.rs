use super::Operation;

pub struct History {
    previous: Vec<Box<dyn Operation>>,
    next: Vec<Box<dyn Operation>>,
    marked_position: Option<usize>,
}

impl History {
    /// Creates a new empty operation history.
    pub fn new() -> History {
        History {
            previous: Vec::new(),
            next: Vec::new(),
            marked_position: None,
        }
    }

    /// Store an operation that has already been run.
    pub fn add(&mut self, operation: Box<dyn Operation>) {
        self.previous.push(operation);
        self.next.clear();

        // Clear marked position if we've replaced a prior operation.
        if let Some(position) = self.marked_position {
            if position >= self.previous.len() {
                self.marked_position = None
            }
        }
    }

    /// Navigate the history backwards.
    pub fn previous(&mut self) -> Option<Box<dyn Operation>> {
        match self.previous.pop() {
            Some(operation) => {
                // We've found a previous operation. Before we return it, store a
                // clone of it so that it can be re-applied as a redo operation.
                self.next.push(operation.clone_operation());
                Some(operation)
            }
            None => None,
        }
    }

    /// Navigate the history forwards.
    pub fn next(&mut self) -> Option<Box<dyn Operation>> {
        match self.next.pop() {
            Some(operation) => {
                // We've found a subsequent operation. Before we return it, store a
                // clone of it so that it can be re-applied as an undo operation, again.
                self.previous.push(operation.clone_operation());
                Some(operation)
            }
            None => None,
        }
    }

    pub fn mark(&mut self) {
        self.marked_position = Some(self.previous.len())
    }

    pub fn at_mark(&self) -> bool {
        if let Some(position) = self.marked_position {
            self.previous.len() == position
        } else {
            false
        }
    }
}
