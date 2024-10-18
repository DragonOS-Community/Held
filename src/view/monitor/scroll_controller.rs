use crate::errors::*;
use crate::{buffer::Buffer, view::terminal::Terminal};
use std::sync::Arc;

/// 对于滚动操作的抽象对象
///
/// 外部通过line_offset方法获取滚动后buffer的offset
pub struct ScrollController {
    terminal: Arc<Box<dyn Terminal>>,
    line_offset: usize,
}

impl ScrollController {
    pub fn new(terminal: Arc<Box<dyn Terminal>>, init_line_index: usize) -> ScrollController {
        ScrollController {
            terminal,
            line_offset: init_line_index,
        }
    }

    // 若将buffer指针指向的行滚动到显示区域顶部
    pub fn scroll_into_monitor(&mut self, buffer: &Buffer) -> Result<()> {
        // buffer发生scroll的行数
        let terminal_height = self.terminal.height()? - 2;
        if self.line_offset > buffer.cursor.line {
            self.line_offset = buffer.cursor.line;
        } else if self.line_offset + terminal_height - 1 < buffer.cursor.line {
            self.line_offset = buffer.cursor.line - terminal_height + 1;
        }
        Ok(())
    }

    // 将buffer指针指向的行滚动到显示区域中间区域
    pub fn scroll_to_center(&mut self, buffer: &Buffer) -> Result<()> {
        self.line_offset = buffer
            .cursor
            .line
            .saturating_sub(self.terminal.height()?.saturating_div(2));
        Ok(())
    }

    // 向上滚动n行
    pub fn scroll_up(&mut self, line_count: usize) {
        self.line_offset = self.line_offset.saturating_sub(line_count);
    }

    // 向下滚动n行
    pub fn scroll_down(&mut self, line_count: usize) {
        self.line_offset = self.line_offset.saturating_add(line_count);
    }

    // 返回当前的offset
    pub fn line_offset(&mut self) -> usize {
        self.line_offset
    }
}
