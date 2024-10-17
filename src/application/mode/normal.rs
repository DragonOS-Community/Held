use std::borrow::Borrow;

use crossterm::event::{KeyCode, KeyEvent};
use unicode_segmentation::UnicodeSegmentation;

use super::{ModeData, ModeRenderer, ModeState};
use crate::{
    application::Application,
    errors::*,
    util::{position::Position, range::Range},
    view::{
        colors::colors::Colors,
        status_data::{buffer_status_data, StatusLineData},
        style::CharStyle,
    },
};
pub(super) struct NormalRenderer;

impl ModeRenderer for NormalRenderer {
    fn render(
        workspace: &mut crate::workspace::Workspace,
        monitor: &mut crate::view::monitor::Monitor,
        _mode: &mut super::ModeData,
    ) -> Result<()> {
        let mut presenter = monitor.build_presenter()?;

        if let Some(buffer) = &workspace.current_buffer {
            let data = buffer.data();
            presenter.print_buffer(buffer, &data, &workspace.syntax_set, None, None)?;

            let mode_name_data = StatusLineData {
                content: " NORMAL ".to_string(),
                color: Colors::Inverted,
                style: CharStyle::Bold,
            };
            presenter.print_status_line(&[
                mode_name_data,
                buffer_status_data(&workspace.current_buffer),
            ])?;

            presenter.present()?;
        } else {
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct NormalState {
    pub cmdchar: Option<char>,
    pub count: usize,
    pub count0: bool,
    pub buf_op_arg: Option<BufOpArg>,
    pub gg: bool,
}

impl NormalState {
    pub fn new() -> Self {
        Self {
            cmdchar: None,
            count: 0,
            count0: false,
            buf_op_arg: None,
            gg: false,
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum BufOpArg {
    Around,    // 操作引号内乃至引号的内容
    Inside,    // 操作引号内的内容
    Line,      // 操作整行
    Word,      // 操作单词
    WordEnd,   // 操作单词的末尾
    WordBegin, // 操作单词的开头
    Block,     // 操作块
}

impl ModeState for NormalState {
    fn reset(&mut self) {
        *self = NormalState::new();
    }
    fn transition(&mut self, key: &KeyEvent) -> Result<()> {
        if let KeyCode::Char(key) = key.code {
            self.hanle_char(key)?;
        }
        Ok(())
    }
}

impl NormalState {
    fn hanle_char(&mut self, key: char) -> Result<()> {
        match key {
            '1'..='9' => {
                // 防止溢出
                if self.count < usize::MAX / 10 - 9 {
                    self.count = self.count * 10 + key.to_digit(10).unwrap() as usize;
                }
                self.count0 = true;
            }
            '0' => {
                if self.count0 {
                    // 防止溢出
                    if self.count < usize::MAX / 10 {
                        self.count = self.count * 10;
                    }
                } else {
                    self.cmdchar = Some(key);
                }
            }
            'd' => {
                if self.cmdchar.is_none() {
                    self.cmdchar = Some(key);
                } else if self.cmdchar == Some('d') {
                    self.buf_op_arg = Some(BufOpArg::Line);
                }
            }
            'g' => {
                if self.cmdchar.is_none() {
                    self.cmdchar = Some(key);
                } else if self.cmdchar == Some('g') {
                    self.gg = true;
                }
            }
            'h' | 'j' | 'k' | 'l' | 'G' => {
                if self.cmdchar.is_none() {
                    self.cmdchar = Some(key);
                }
            }
            _ => {}
        }
        Ok(())
    }
}

pub fn exec_j_cmd(app: &mut Application) {
    if let ModeData::Normal(normal_state) = &mut app.mode {
        let count = normal_state.count.max(1);
        if let Some(buffer) = &mut app.workspace.current_buffer {
            for _ in 0..count.min(buffer.line_count() - buffer.cursor.line) {
                buffer.cursor.move_down();
            }
            app.monitor.scroll_to_cursor(buffer).unwrap();
            normal_state.reset();
        }
    }
}

pub fn exec_k_cmd(app: &mut Application) {
    if let ModeData::Normal(normal_state) = &mut app.mode {
        let count = normal_state.count.max(1);
        if let Some(buffer) = &mut app.workspace.current_buffer {
            for _ in 0..count.min(buffer.cursor.line) {
                buffer.cursor.move_up();
            }
            app.monitor.scroll_to_cursor(buffer).unwrap();
            normal_state.reset();
        }
    }
}

pub fn exec_d_cmd(app: &mut Application) {
    if let ModeData::Normal(normal_state) = &mut app.mode {
        let count = normal_state.count.max(1);
        if let Some(buffer) = &mut app.workspace.current_buffer {
            match normal_state.buf_op_arg {
                Some(BufOpArg::Line) => {
                    let line = buffer.cursor.line;
                    buffer.delete_range(Range::new(
                        Position::new(line, 0),
                        Position::new((line + count).min(buffer.line_count()), 0),
                    ));
                    normal_state.reset();
                }
                Some(BufOpArg::Word) => {
                    todo!()
                }
                Some(BufOpArg::WordEnd) => {
                    todo!()
                }
                Some(BufOpArg::WordBegin) => {
                    todo!()
                }
                Some(BufOpArg::Block) => {
                    todo!()
                }
                _ => {}
            }
        }
    }
}

pub fn exec_0_cmd(app: &mut Application) {
    if let ModeData::Normal(normal_state) = &mut app.mode {
        if let Some(buffer) = &mut app.workspace.current_buffer {
            buffer.cursor.move_to_start_of_line();
        }
        normal_state.reset();
    }
}

pub fn exec_g_cmd(app: &mut Application) {
    if let ModeData::Normal(normal_state) = &mut app.mode {
        if let Some(buffer) = &mut app.workspace.current_buffer {
            if normal_state.gg {
                buffer.cursor.move_to_first_line();
                app.monitor.scroll_to_cursor(buffer).unwrap();
                normal_state.reset();
            }
        }
    }
}

#[allow(non_snake_case)]
pub fn exec_G_cmd(app: &mut Application) {
    if let ModeData::Normal(normal_state) = &mut app.mode {
        if let Some(buffer) = &mut app.workspace.current_buffer {
            if normal_state.count == 0 {
                buffer.cursor.move_to_last_line();
            } else {
                let target_line = normal_state.count.min(buffer.line_count());
                let offset = buffer.cursor.offset;
                if !buffer
                    .cursor
                    .move_to(Position::new(target_line - 1, offset))
                {
                    let target_offset = buffer
                        .data()
                        .lines()
                        .nth(target_line - 1)
                        .unwrap()
                        .graphemes(true)
                        .count();
                    buffer
                        .cursor
                        .move_to(Position::new(target_line - 1, target_offset));
                }
            }
            app.monitor.scroll_to_cursor(buffer).unwrap();
            normal_state.reset();
        }
    }
}

pub fn exec_h_cmd(app: &mut Application) {
    if let ModeData::Normal(normal_state) = &mut app.mode {
        let mut count = normal_state.count.max(1);
        if let Some(buffer) = &mut app.workspace.current_buffer {
            let offset = buffer.cursor.offset;
            count = count.min(offset);
            for _ in 0..count {
                buffer.cursor.move_left();
            }
            app.monitor.scroll_to_cursor(buffer).unwrap();
            normal_state.reset();
        }
    }
}

pub fn exec_l_cmd(app: &mut Application) {
    if let ModeData::Normal(normal_state) = &mut app.mode {
        let mut count = normal_state.count.max(1);
        if let Some(buffer) = &mut app.workspace.current_buffer {
            let max_offset = buffer
                .data()
                .lines()
                .nth(buffer.cursor.line)
                .unwrap()
                .graphemes(true)
                .count();
            let offset = buffer.cursor.offset;
            count = count.min(max_offset - offset);
            for _ in 0..count {
                buffer.cursor.move_right();
            }
            app.monitor.scroll_to_cursor(buffer).unwrap();
            normal_state.reset();
        }
    }
}
