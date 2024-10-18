use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::str::FromStr;

use crate::errors::*;
use crate::modules::perferences::Perferences;
use crate::util::position::Position;
use crate::util::range::Range;
use crate::view::colors::colors::Colors;
use crate::view::colors::to_rgb;
use crate::view::style::CharStyle;
use crate::{buffer::Buffer, util::line_iterator::LineIterator, view::terminal::Terminal};
use crossterm::style::Color;
use syntect::highlighting::{HighlightIterator, Highlighter, Style, Theme};
use syntect::parsing::{ScopeStack, SyntaxSet};

use unicode_segmentation::UnicodeSegmentation;

use super::line_number_string_iter::LineNumberStringIter;
use super::render_buffer::Cell;
use super::render_state::RenderState;
use super::{lexeme_mapper::LexemeMapper, render_buffer::RenderBuffer};

const RENDER_CACHE_FREQUENCY: usize = 100;

pub struct Renderer<'a, 'p> {
    buffer: &'a Buffer,
    render_buffer: &'a mut RenderBuffer<'p>,
    terminal: &'a dyn Terminal,
    theme: &'a Theme,
    highlight_ranges: Option<&'a [Range]>,
    scroll_offset: usize,
    line_number_iter: LineNumberStringIter,
    content_start_of_line: usize,
    cached_render_state: &'a Rc<RefCell<HashMap<usize, RenderState>>>,
    syntax_set: &'a SyntaxSet,
    screen_position: Position,
    buffer_position: Position,
    cursor_position: Option<Position>,
    current_style: Style,
    perferences: &'a dyn Perferences,
}

impl<'a, 'p> Renderer<'a, 'p> {
    pub fn new(
        buffer: &'a Buffer,
        render_buffer: &'a mut RenderBuffer<'p>,
        terminal: &'a dyn Terminal,
        perferences: &'a dyn Perferences,
        highlight_ranges: Option<&'a [Range]>,
        cached_render_state: &'a Rc<RefCell<HashMap<usize, RenderState>>>,
        theme: &'a Theme,
        syntax_set: &'a SyntaxSet,
        scroll_offset: usize,
    ) -> Renderer<'a, 'p> {
        let line_number_iter = LineNumberStringIter::new(buffer, scroll_offset);
        let content_start_of_line = line_number_iter.width() + 1;
        Self {
            buffer,
            render_buffer,
            terminal,
            theme,
            scroll_offset,
            syntax_set,
            cached_render_state,
            screen_position: Position::default(),
            buffer_position: Position::default(),
            cursor_position: None,
            current_style: Style::default(),
            perferences,
            highlight_ranges,
            line_number_iter,
            content_start_of_line,
        }
    }

    pub fn render(
        &mut self,
        lines: LineIterator<'a>,
        mut lexeme_mapper: Option<&mut dyn LexemeMapper>,
    ) -> Result<Option<Position>> {
        self.terminal.set_cursor(None)?;
        self.render_line_number();

        let highlighter = Highlighter::new(&self.theme);
        let syntax_definition = self
            .buffer
            .syntax_definition
            .as_ref()
            .ok_or("Buffer has no syntax definition")?;

        let focused_style = Renderer::mapper_keyword_style(&highlighter);
        let blurred_style = Renderer::mapper_comment_style(&highlighter);

        let (cached_line_num, mut state) = self
            .current_cached_render_state()
            .unwrap_or((0, RenderState::new(&highlighter, syntax_definition)));

        for (line_num, line_data) in lines {
            if line_num >= cached_line_num {
                if line_num % RENDER_CACHE_FREQUENCY == 0 {
                    self.cached_render_state
                        .borrow_mut()
                        .insert(line_num, state.clone());
                }

                if self.before_visible() {
                    self.try_to_advance_to_next_line(&line_data);
                    continue;
                }

                if self.after_visible() {
                    break;
                }

                let events = state
                    .parse
                    .parse_line(&line_data, self.syntax_set)
                    .chain_err(|| "Failed to parse buffer")?;

                let styled_lexemes =
                    HighlightIterator::new(&mut state.highlight, &events, &line_data, &highlighter);

                for (style, lexeme) in styled_lexemes {
                    if let Some(ref mut mapper) = lexeme_mapper {
                        let mapped_lexemes = mapper.map(lexeme, self.buffer_position);
                        for mapped_lexeme in mapped_lexemes {
                            match mapped_lexeme {
                                super::lexeme_mapper::MappedLexeme::Focused(val) => {
                                    self.current_style = focused_style;
                                    self.render_lexeme(val.to_string());
                                }
                                super::lexeme_mapper::MappedLexeme::Blurred(val) => {
                                    self.current_style = blurred_style;
                                    self.render_lexeme(val.to_string());
                                }
                            }
                        }
                    } else {
                        self.current_style = style;
                        self.render_lexeme(lexeme);
                    }
                }
            }

            self.try_to_advance_to_next_line(&line_data);
        }

        Ok(self.cursor_position)
    }

    fn mapper_keyword_style(highlighter: &Highlighter) -> Style {
        highlighter.style_for_stack(
            ScopeStack::from_str("keyword")
                .unwrap_or_default()
                .as_slice(),
        )
    }

    fn mapper_comment_style(highlighter: &Highlighter) -> Style {
        highlighter.style_for_stack(
            ScopeStack::from_str("keyword")
                .unwrap_or_default()
                .as_slice(),
        )
    }

    fn current_cached_render_state(&self) -> Option<(usize, RenderState)> {
        self.cached_render_state
            .borrow()
            .iter()
            .filter(|(k, _)| **k < self.scroll_offset)
            .max_by(|a, b| a.0.cmp(b.0))
            .map(|x| (*x.0, x.1.clone()))
    }

    fn after_visible(&self) -> bool {
        // 腾出底下两行
        self.screen_position.line >= (self.terminal.height().unwrap() - 2)
    }

    fn before_visible(&self) -> bool {
        self.buffer_position.line < self.scroll_offset
    }

    fn inside_visible(&self) -> bool {
        !self.before_visible() && !self.after_visible()
    }

    fn set_cursor(&mut self) {
        if self.inside_visible() && *self.buffer.cursor == self.buffer_position {
            self.cursor_position = Some(self.screen_position);
        }
    }

    fn on_cursor_line(&self) -> bool {
        self.buffer.cursor.line == self.buffer_position.line
    }

    fn try_to_advance_to_next_line(&mut self, line: &str) {
        if line.chars().last().map(|x| x == '\n').unwrap_or(false) {
            self.advance_to_next_line();
        }
    }

    fn advance_to_next_line(&mut self) {
        if self.inside_visible() {
            self.set_cursor();
            self.render_rest_of_line();
            self.screen_position.line += 1;
        }

        self.buffer_position.line += 1;
        self.buffer_position.offset = 0;
        self.render_line_number();
    }

    fn render_rest_of_line(&mut self) {
        let on_cursor_line = self.on_cursor_line();
        for offset in self.screen_position.offset..self.terminal.width().unwrap() {
            let colors = if on_cursor_line {
                Colors::Focused
            } else {
                Colors::Default
            };

            self.render_cell(
                Position {
                    line: self.screen_position.line,
                    offset,
                },
                CharStyle::Default,
                colors,
                " ",
            );
        }
    }

    fn render_line_number(&mut self) {
        if !self.inside_visible() {
            return;
        }
        let line_number = self.line_number_iter.next().unwrap();
        let is_on_cursor_line = self.on_cursor_line();

        let style = if is_on_cursor_line {
            CharStyle::Bold
        } else {
            CharStyle::Default
        };
        // 渲染行号
        self.render_cell(
            Position {
                line: self.screen_position.line,
                offset: 0,
            },
            style,
            Colors::Focused,
            line_number,
        );

        // 行号后的gap
        let gap_color = if is_on_cursor_line {
            Colors::Focused
        } else {
            Colors::Default
        };
        self.render_cell(
            Position {
                line: self.screen_position.line,
                offset: self.line_number_iter.width(),
            },
            style,
            gap_color,
            " ",
        );

        self.screen_position.offset = self.line_number_iter.width() + 1;
    }

    fn render_lexeme<T: Into<Cow<'a, str>>>(&mut self, lexeme: T) {
        for character in lexeme.into().graphemes(true) {
            if character == "\n" {
                continue;
            }

            self.set_cursor();

            let token_color = to_rgb(self.current_style.foreground);
            let (style, color) = self.current_char_style(token_color);

            if self.perferences.line_wrapping()
                && self.screen_position.offset == self.terminal.width().unwrap()
            {
                todo!()
            } else if character == "\t" {
                todo!()
            } else {
                self.render_cell(self.screen_position, style, color, character.to_string());
                self.screen_position.offset += 1;
                self.buffer_position.offset += 1;
            }

            // 退出循环前更新
            self.set_cursor();
        }
    }

    fn render_cell<C: Into<Cow<'p, str>>>(
        &mut self,
        position: Position,
        style: CharStyle,
        colors: Colors,
        content: C,
    ) {
        self.render_buffer.set_cell(
            position,
            Cell {
                content: content.into(),
                colors,
                style,
            },
        );
    }

    fn current_char_style(&self, token_color: Color) -> (CharStyle, Colors) {
        let (style, colors) = match self.highlight_ranges {
            Some(highlight_ranges) => {
                for range in highlight_ranges {
                    if range.includes(&self.buffer_position) {
                        // We're inside of one of the highlighted areas.
                        // Return early with highlight colors.
                        if range.includes(&self.buffer.cursor) {
                            return (CharStyle::Bold, Colors::SelectMode);
                        } else {
                            return (CharStyle::Reverse, Colors::Default);
                        }
                    }
                }

                // We aren't inside one of the highlighted areas.
                // Fall back to other styling considerations.
                if self.on_cursor_line() {
                    (
                        CharStyle::Default,
                        Colors::CustomFocusedForeground(token_color),
                    )
                } else {
                    (CharStyle::Default, Colors::CustomForeground(token_color))
                }
            }
            None => {
                if self.on_cursor_line() {
                    (
                        CharStyle::Default,
                        Colors::CustomFocusedForeground(token_color),
                    )
                } else {
                    (CharStyle::Default, Colors::CustomForeground(token_color))
                }
            }
        };

        (style, colors)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        cell::RefCell,
        collections::HashMap,
        io::{BufReader, Cursor},
        path::Path,
        rc::Rc,
    };

    use syntect::{
        highlighting::{Theme, ThemeSet},
        parsing::SyntaxSet,
    };

    use crate::{
        buffer::Buffer,
        modules::perferences::DummyPerferences,
        util::line_iterator::LineIterator,
        view::{
            colors::map::ColorMap,
            render::render_buffer::RenderBuffer,
            terminal::{cross_terminal::CrossTerminal, Terminal},
        },
    };

    use super::Renderer;

    #[test]
    fn test_display() {
        let terminal = CrossTerminal::new().unwrap();
        let mut buffer = Buffer::from_file(Path::new("src/main.rs")).unwrap();
        let mut render_buffer =
            RenderBuffer::new(terminal.width().unwrap(), terminal.height().unwrap());
        let perferences = DummyPerferences;
        let cached_render_state = Rc::new(RefCell::new(HashMap::new()));

        let mut reader = BufReader::new(Cursor::new(include_str!(
            "../../themes/solarized_dark.tmTheme"
        )));
        let theme = ThemeSet::load_from_reader(&mut reader).unwrap();
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let definition = buffer
            .file_extension()
            .and_then(|ex| syntax_set.find_syntax_by_extension(&ex))
            .or_else(|| Some(syntax_set.find_syntax_plain_text()))
            .cloned();

        buffer.syntax_definition = definition;
        let binding = buffer.data();
        {
            let mut renderer = Renderer::new(
                &buffer,
                &mut render_buffer,
                &terminal,
                &perferences,
                None,
                &cached_render_state,
                &theme,
                &syntax_set,
                0,
            );
            renderer.render(LineIterator::new(&binding), None).unwrap();
        }

        for (position, cell) in render_buffer.iter() {
            terminal
                .print(
                    &position,
                    cell.style,
                    theme.map_colors(cell.colors),
                    &cell.content,
                )
                .unwrap();
        }

        terminal.present().unwrap();
    }
}
