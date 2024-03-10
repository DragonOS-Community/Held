use std::io::{self, stdout, Write};

use crossterm::{terminal::*, ExecutableCommand};

use super::ui::uicore::DEF_STYLE;

pub struct TermManager;

#[allow(dead_code)]
impl TermManager {
    pub fn init_term() -> io::Result<()> {
        DEF_STYLE.read().unwrap().set_content_style()?;
        Self::clear_all()
    }

    #[inline]
    pub fn disable_line_warp() -> io::Result<()> {
        stdout().execute(DisableLineWrap).unwrap().flush()
    }

    #[inline]
    pub fn enable_line_warp() -> io::Result<()> {
        stdout().execute(EnableLineWrap).unwrap().flush()
    }

    #[inline]
    pub fn leave_alternate_screen() -> io::Result<()> {
        stdout().execute(LeaveAlternateScreen).unwrap().flush()
    }

    #[inline]
    pub fn enter_alternate_screen() -> io::Result<()> {
        stdout().execute(EnterAlternateScreen).unwrap().flush()
    }

    #[inline]
    pub fn scroll_up(lines: u16) -> io::Result<()> {
        stdout().execute(ScrollUp(lines)).unwrap().flush()
    }

    #[inline]
    pub fn scroll_down(lines: u16) -> io::Result<()> {
        stdout().execute(ScrollDown(lines)).unwrap().flush()
    }

    #[inline]
    pub fn clear_all() -> io::Result<()> {
        stdout().execute(Clear(ClearType::All)).unwrap().flush()
    }

    #[inline]
    pub fn clear_purge() -> io::Result<()> {
        stdout().execute(Clear(ClearType::Purge)).unwrap().flush()
    }

    #[inline]
    pub fn clear_under_cursor() -> io::Result<()> {
        stdout()
            .execute(Clear(ClearType::FromCursorDown))
            .unwrap()
            .flush()
    }

    #[inline]
    pub fn clear_up_cursor() -> io::Result<()> {
        stdout()
            .execute(Clear(ClearType::FromCursorUp))
            .unwrap()
            .flush()
    }

    #[inline]
    pub fn clear_current_line() -> io::Result<()> {
        stdout()
            .execute(Clear(ClearType::CurrentLine))
            .unwrap()
            .flush()
    }

    #[inline]
    pub fn clear_until_new_line() -> io::Result<()> {
        stdout()
            .execute(Clear(ClearType::UntilNewLine))
            .unwrap()
            .flush()
    }
}
