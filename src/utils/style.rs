use std::io::{self, stdout, Write};

use crossterm::{style::*, ExecutableCommand};

pub struct StyleManager;

#[allow(dead_code)]
impl StyleManager {
    #[inline]
    pub fn set_foreground_color(color: Color) -> io::Result<()> {
        stdout().execute(SetForegroundColor(color)).unwrap().flush()
    }

    #[inline]
    pub fn set_background_color(color: Color) -> io::Result<()> {
        stdout().execute(SetBackgroundColor(color)).unwrap().flush()
    }

    #[inline]
    pub fn set_underline_color(color: Color) -> io::Result<()> {
        stdout().execute(SetUnderlineColor(color)).unwrap().flush()
    }

    #[inline]
    pub fn set_color(fg: Option<Color>, bg: Option<Color>) -> io::Result<()> {
        stdout()
            .execute(SetColors(Colors {
                foreground: fg,
                background: bg,
            }))
            .unwrap()
            .flush()
    }

    #[inline]
    pub fn set_attr(attr: Attribute) -> io::Result<()> {
        stdout().execute(SetAttribute(attr)).unwrap().flush()
    }

    #[inline]
    pub fn set_attrs(attr: Attributes) -> io::Result<()> {
        stdout().execute(SetAttributes(attr)).unwrap().flush()
    }

    #[inline]
    pub fn set_style(style: ContentStyle) -> io::Result<()> {
        stdout().execute(SetStyle(style)).unwrap().flush()
    }

    #[inline]
    pub fn reset_color() -> io::Result<()> {
        stdout().execute(ResetColor).unwrap().flush()
    }
}
