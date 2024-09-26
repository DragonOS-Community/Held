use std::{
    cmp::Ordering,
    ops::{Add, AddAssign},
};

use super::distance::Distance;

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Position {
    pub line: usize,
    pub offset: usize,
}

impl Position {
    pub fn new(line: usize, offset: usize) -> Position {
        Position { line, offset }
    }
}

impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Position) -> Option<Ordering> {
        Some(if self.line < other.line {
            Ordering::Less
        } else if self.line > other.line {
            Ordering::Greater
        } else if self.offset < other.offset {
            Ordering::Less
        } else if self.offset > other.offset {
            Ordering::Greater
        } else {
            Ordering::Equal
        })
    }
}

impl Add<Distance> for Position {
    type Output = Position;

    fn add(self, distance: Distance) -> Self::Output {
        let offset = if distance.lines > 0 {
            distance.offset
        } else {
            self.offset + distance.offset
        };

        Position {
            line: self.line + distance.lines,
            offset,
        }
    }
}

impl AddAssign<Distance> for Position {
    fn add_assign(&mut self, distance: Distance) {
        self.line += distance.lines;
        self.offset = if distance.lines > 0 {
            distance.offset
        } else {
            self.offset + distance.offset
        };
    }
}
