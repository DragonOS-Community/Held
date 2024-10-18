use std::{borrow::Cow, fmt::Debug};

use super::{
    colors::{colors::Colors, map::ColorMap},
    monitor::Monitor,
    render::{
        lexeme_mapper::LexemeMapper,
        render_buffer::{Cell, RenderBuffer},
    },
    status_data::StatusLineData,
    style::CharStyle,
};
use crate::{
    application::mode::command::CommandData,
    buffer::Buffer,
    errors::*,
    util::{line_iterator::LineIterator, position::Position, range::Range},
    view::render::renderer::Renderer,
};
use clap::builder::styling::Style;
use syntect::{highlighting::Theme, parsing::SyntaxSet};

pub struct Presenter<'a> {
    view: &'a mut Monitor,
    theme: Theme,
    present_buffer: RenderBuffer<'a>,
    cursor_position: Option<Position>,
}

impl<'a> Presenter<'a> {
    pub fn new(monitor: &mut Monitor) -> Result<Presenter> {
        let theme_name = monitor.perference.borrow().theme_name();
        let mut theme = monitor
            .first_theme()
            .ok_or_else(|| format!("Couldn't find anyone theme"))?;
        if let Some(theme_name) = theme_name {
            theme = monitor
                .get_theme(&theme_name)
                .ok_or_else(|| format!("Couldn't find \"{}\" theme", theme_name))?;
        }
        let present_buffer = RenderBuffer::new(monitor.width()?, monitor.height()?);
        Ok(Presenter {
            view: monitor,
            theme,
            present_buffer,
            cursor_position: None,
        })
    }

    pub fn set_cursor(&mut self, position: Position) {
        self.cursor_position = Some(position);
    }

    pub fn present(&self) -> Result<()> {
        for (position, cell) in self.present_buffer.iter() {
            self.view
                .terminal
                .print(
                    &position,
                    cell.style,
                    self.theme.map_colors(cell.colors),
                    &cell.content,
                )
                .unwrap();
        }

        self.view.terminal.set_cursor(self.cursor_position)?;
        self.view.terminal.present()?;
        Ok(())
    }

    pub fn print_status_line(&mut self, datas: &[StatusLineData]) -> Result<()> {
        let line_width = self.view.terminal.width()?;
        // 在倒数第二行打印status_line
        let line = self.view.terminal.height()? - 2;

        let count = datas.len();
        let mut offset = 0;
        // 从左往右输出，最后一个参数在最后
        for (index, data) in datas.iter().enumerate() {
            let content = match count {
                1 => {
                    format!("{:width$}", data.content, width = line_width)
                }
                _ => {
                    if index == count - 1 {
                        format!(
                            "{:width$}",
                            data.content,
                            width = line_width.saturating_sub(offset)
                        )
                    } else {
                        data.content.to_owned()
                    }
                }
            };

            let len = content.len();
            self.print(&Position { line, offset }, data.style, data.color, content);
            offset += len;
        }

        Ok(())
    }

    // 后续也许要作为一个trait的方法，提供给search和command一起使用(/和:开头)
    pub fn print_last_line(&mut self, data: &CommandData) -> Result<()> {
        // slet line_width = self.view.terminal.width()?;
        // 在倒数第一行打印last_line
        let line = self.view.terminal.height()? - 1;

        let offset = 0;

        let content = format!(":{:}", data.input);
        let cursor_position_offset = content.len();
        self.print(
            &Position { line, offset },
            CharStyle::Default,
            Colors::Default,
            content,
        );
        self.cursor_position = Some(Position::new(line, cursor_position_offset));

        Ok(())
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
        let scroll_offset = self.view.get_scroll_controller(buffer).line_offset();
        let lines = LineIterator::new(&buffer_data);
        self.cursor_position = Renderer::new(
            buffer,
            &mut self.present_buffer,
            &**self.view.terminal,
            &*self.view.perference.borrow(),
            highlights,
            self.view.get_render_cache(buffer),
            &self.theme,
            syntax_set,
            scroll_offset,
        )
        .render(lines, lexeme_mapper)?;
        Ok(())
    }

    pub fn print<C>(&mut self, position: &Position, style: CharStyle, colors: Colors, content: C)
    where
        C: Into<Cow<'a, str>> + Debug,
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
