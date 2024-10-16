use std::fmt::Debug;

use crate::errors::*;
use crate::util::position::Position;
use crossterm::event::Event;

use super::{colors::colors::Colors, style::CharStyle};

pub mod cross_terminal;

pub(super) const MIN_WIDTH: u16 = 10;
pub(super) const MIN_HEIGHT: u16 = 10;

pub const TERMINAL_EXECUTE_ERROR: &'static str = "Terminal IO Error";

#[allow(dead_code)]
pub trait Terminal: Send + Sync + Debug {
    fn listen(&self) -> Result<Event>;
    fn clear(&self) -> Result<()>;
    fn present(&self) -> Result<()>;
    fn width(&self) -> Result<usize>;
    fn height(&self) -> Result<usize>;
    fn set_cursor(&self, _: Option<Position>) -> Result<()>;
    fn set_cursor_type(&self, _: crossterm::cursor::SetCursorStyle) -> Result<()>;
    fn print(&self, _: &Position, _: CharStyle, _: Colors, _: &str) -> Result<()>;
    fn suspend(&self);
}
