use crossterm::event::{KeyCode, KeyEvent};
use held_core::view::{colors::Colors, style::CharStyle};

use super::{ModeRenderer, ModeState};
use crate::{
    application::Application,
    errors::*,
    view::status_data::{buffer_status_data, StatusLineData},
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
            warn!("normal buffer id: {}", buffer.id.unwrap());
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
    pub cmdchar: Option<char>,        // 当前命令字符
    pub count: usize,                 // 命令执行次数
    pub buf_op_arg: Option<BufOpArg>, // 缓冲区操作参数
}

impl NormalState {
    pub fn new() -> Self {
        Self {
            cmdchar: None,
            count: 0,
            buf_op_arg: None,
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
            // 0..=9 用于更新执行次数
            '1'..='9' => {
                // 防止溢出
                if self.count < usize::MAX / 10 - 9 {
                    self.count = self.count * 10 + key.to_digit(10).unwrap() as usize;
                }
            }
            '0' => {
                // 防止溢出
                if self.count < usize::MAX / 10 {
                    self.count = self.count * 10;
                }
            }
            // 处理删除的状态参数，待支持正则匹配后弃用
            'd' => {
                if self.cmdchar.is_none() {
                    self.cmdchar = Some(key);
                } else if self.cmdchar == Some('d') {
                    self.buf_op_arg = Some(BufOpArg::Line);
                }
            }
            // 'h' | 'j' | 'k' | 'l' | 'G' => {
            //     if self.cmdchar.is_none() {
            //         self.cmdchar = Some(key);
            //     }
            // }
            _ => {}
        }
        Ok(())
    }
}
