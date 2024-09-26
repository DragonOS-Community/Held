use std::io::{stdout, Write};

use crossterm::{
    event::Event,
    terminal::{self, disable_raw_mode},
    ExecutableCommand,
};

use super::{Terminal, MIN_HEIGHT, MIN_WIDTH, TERMINAL_EXECUTE_ERROR};
use crate::errors::*;

#[derive(Debug)]
pub struct CrossTerminal;

unsafe impl Send for CrossTerminal {}
unsafe impl Sync for CrossTerminal {}

impl CrossTerminal {
    pub fn new() -> Result<CrossTerminal> {
        crossterm::terminal::enable_raw_mode()?;
        Ok(CrossTerminal)
    }

    fn update_style(
        &self,
        char_style: crate::view::style::CharStyle,
        colors: crate::view::colors::colors::Colors,
    ) -> Result<()> {
        stdout().execute(crossterm::style::SetAttribute(
            crossterm::style::Attribute::Reset,
        ))?;

        match char_style {
            crate::view::style::CharStyle::Default => {}
            crate::view::style::CharStyle::Bold => {
                stdout().execute(crossterm::style::SetAttribute(
                    crossterm::style::Attribute::Bold,
                ))?;
            }
            crate::view::style::CharStyle::Reverse => {
                stdout().execute(crossterm::style::SetAttribute(
                    crossterm::style::Attribute::Reverse,
                ))?;
            }
            crate::view::style::CharStyle::Italic => {
                stdout().execute(crossterm::style::SetAttribute(
                    crossterm::style::Attribute::Italic,
                ))?;
            }
        }

        match colors {
            crate::view::colors::colors::Colors::Default => {}
            crate::view::colors::colors::Colors::CustomForeground(color) => {
                stdout().execute(crossterm::style::SetForegroundColor(color))?;
            }
            crate::view::colors::colors::Colors::Custom(fg, bg) => {
                stdout().execute(crossterm::style::SetForegroundColor(fg))?;
                stdout().execute(crossterm::style::SetBackgroundColor(bg))?;
            }
            _ => {
                unreachable!();
            }
        }

        Ok(())
    }
}

impl Terminal for CrossTerminal {
    fn listen(&self) -> Result<Event> {
        crossterm::event::read().chain_err(|| "Handle event io error")
    }

    fn clear(&self) -> Result<()> {
        stdout()
            .execute(terminal::Clear(terminal::ClearType::All))
            .chain_err(|| TERMINAL_EXECUTE_ERROR)
            .map(|_| ())
    }

    fn present(&self) -> Result<()> {
        stdout()
            .flush()
            .chain_err(|| TERMINAL_EXECUTE_ERROR)
            .map(|_| ())
    }

    fn width(&self) -> Result<usize> {
        let (width, _) = crossterm::terminal::size()?;
        return Ok(width.max(MIN_WIDTH).into());
    }

    fn height(&self) -> Result<usize> {
        let (_, height) = crossterm::terminal::size()?;
        return Ok(height.max(MIN_HEIGHT).into());
    }

    fn set_cursor(&self, position: Option<crate::util::position::Position>) -> Result<()> {
        match position {
            Some(position) => {
                stdout()
                    .execute(crossterm::cursor::MoveTo(
                        position.offset as u16,
                        position.line as u16,
                    ))
                    .unwrap()
                    .execute(crossterm::cursor::Show)
                    .chain_err(|| TERMINAL_EXECUTE_ERROR)?;
            }
            None => {
                stdout()
                    .execute(crossterm::cursor::Hide)
                    .chain_err(|| TERMINAL_EXECUTE_ERROR)?;
            }
        }

        Ok(())
    }

    fn set_cursor_type(&self, cursor_type: crossterm::cursor::SetCursorStyle) -> Result<()> {
        stdout()
            .execute(cursor_type)
            .chain_err(|| TERMINAL_EXECUTE_ERROR)
            .map(|_| ())
    }

    fn print(
        &self,
        position: &crate::util::position::Position,
        char_style: crate::view::style::CharStyle,
        colors: crate::view::colors::colors::Colors,
        content: &str,
    ) -> Result<()> {
        self.update_style(char_style, colors)?;
        self.set_cursor(Some(*position))?;
        stdout().write(content.as_bytes())?;
        Ok(())
    }

    fn suspend(&self) {}
}

impl Drop for CrossTerminal {
    fn drop(&mut self) {
        self.suspend();
        let _ = disable_raw_mode();
    }
}
