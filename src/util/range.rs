use crate::util::position::Position;

/// Range: 左开右闭区间
#[derive(Clone, Debug, PartialEq)]
pub struct Range {
    start: Position,
    end: Position,
}

impl Range {
    pub fn new(start: Position, end: Position) -> Range {
        if start > end {
            Range {
                start: end,
                end: start,
            }
        } else {
            Range { start, end }
        }
    }

    pub fn start(&self) -> Position {
        self.start
    }

    pub fn end(&self) -> Position {
        self.end
    }

    /// Whether or not the range includes the specified position.
    /// The range is exclusive, such that its ending position is not included.
    ///
    /// # Examples
    ///
    /// ```
    /// use scribe::buffer::{Position, Range};
    ///
    /// // Builder a range.
    /// let range = Range::new(
    ///     Position{ line: 0, offset: 0 },
    ///     Position{ line: 1, offset: 5 }
    /// );
    ///
    /// assert!(range.includes(
    ///     &Position{ line: 1, offset: 0 }
    /// ));
    ///
    /// assert!(range.includes(
    ///     &Position{ line: 1, offset: 4 }
    /// ));
    ///
    /// assert!(!range.includes(
    ///     &Position{ line: 1, offset: 5 }
    /// ));
    /// ```
    pub fn includes(&self, position: &Position) -> bool {
        position >= &self.start() && position < &self.end()
    }
}
