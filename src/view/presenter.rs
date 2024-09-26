use std::borrow::Cow;

use super::{
    colors::colors::Colors,
    monitor::Monitor,
    render::{
        lexeme_mapper::LexemeMapper,
        render_buffer::{Cell, RenderBuffer},
    },
    style::CharStyle,
};
use crate::{
    buffer::Buffer,
    errors::*,
    util::{position::Position, range::Range},
};
use syntect::{highlighting::Theme, parsing::SyntaxSet};

pub struct Presenter<'a> {
    view: &'a mut Monitor,
    theme: Theme,
    present_buffer: RenderBuffer<'a>,
}

impl<'a> Presenter<'a> {
    pub fn new(monitor: &mut Monitor) -> Result<Presenter> {
        let theme_name = monitor.perference.borrow().theme_name();
        let theme = monitor
            .get_theme(&theme_name)
            .ok_or_else(|| format!("Couldn't find \"{}\" theme", theme_name))?;
        let present_buffer = RenderBuffer::new(monitor.width()?, monitor.height()?);
        Ok(Presenter {
            view: monitor,
            theme,
            present_buffer,
        })
    }

    pub fn present(&self) -> Result<()> {
        todo!()
    }

    pub fn print_status_line(&mut self /* some data? */) {
        todo!()
    }

    // 按照预设渲染buffer
    pub fn print_buffer(
        &mut self,
        buffer: &Buffer,
        buffer_data: &'a str,
        syntax_set: &'a SyntaxSet,
        highlights: Option<&[Range]>,
        lexeme_mapper: Option<&'a mut dyn LexemeMapper>,
    ) -> Result<()> {
        todo!()
    }

    pub fn print<C>(&mut self, position: &Position, style: CharStyle, colors: Colors, content: C)
    where
        C: Into<Cow<'a, str>>,
    {
        self.present_buffer.set_cell(
            *position,
            Cell {
                content: content.into(),
                style,
                colors,
            },
        );
    }
}
