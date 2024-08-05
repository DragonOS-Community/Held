use std::{
    fmt::Display,
    io::{self, stdout, Write},
    sync::Arc,
};

use crossterm::{
    cursor::{
        Hide, MoveDown, MoveLeft, MoveRight, MoveTo, MoveToColumn, MoveToNextLine,
        MoveToPreviousLine, MoveToRow, MoveUp, RestorePosition, SavePosition, Show,
    },
    ExecutableCommand,
};

use crate::config::appconfig::LineSetting;

use super::{
    buffer::{EditBuffer, LineBuffer},
    style::StyleManager,
    term_io::TermIO,
    terminal::TermManager,
    ui::uicore::{CONTENT_WINSIZE, DEF_STYLE, WINSIZE},
};

struct CursorManager;

#[allow(dead_code)]
impl CursorManager {
    #[inline]
    pub fn move_to(x: u16, y: u16) -> io::Result<()> {
        stdout().execute(MoveTo(x, y)).unwrap().flush()
    }

    #[inline]
    pub fn move_to_nextline(lines: u16) -> io::Result<()> {
        stdout().execute(MoveToNextLine(lines)).unwrap().flush()
    }

    #[inline]
    pub fn move_to_previous_line(lines: u16) -> io::Result<()> {
        stdout().execute(MoveToPreviousLine(lines)).unwrap().flush()
    }

    #[inline]
    pub fn move_to_columu(col: u16) -> io::Result<()> {
        stdout().execute(MoveToColumn(col)).unwrap().flush()
    }

    #[inline]
    pub fn move_to_row(row: u16) -> io::Result<()> {
        stdout().execute(MoveToRow(row)).unwrap().flush()
    }

    #[inline]
    pub fn move_up(count: u16) -> io::Result<()> {
        stdout().execute(MoveUp(count)).unwrap().flush()
    }

    #[inline]
    pub fn move_down(count: u16) -> io::Result<()> {
        stdout().execute(MoveDown(count)).unwrap().flush()
    }

    #[inline]
    pub fn move_left(count: u16) -> io::Result<()> {
        stdout().execute(MoveLeft(count)).unwrap().flush()
    }

    #[inline]
    pub fn move_right(count: u16) -> io::Result<()> {
        stdout().execute(MoveRight(count)).unwrap().flush()
    }

    #[inline]
    pub fn save_position() -> io::Result<()> {
        stdout().execute(SavePosition).unwrap().flush()
    }

    #[inline]
    pub fn restore_position() -> io::Result<()> {
        stdout().execute(RestorePosition).unwrap().flush()
    }

    #[inline]
    pub fn hide() -> io::Result<()> {
        stdout().execute(Hide).unwrap().flush()
    }

    #[inline]
    pub fn show() -> io::Result<()> {
        stdout().execute(Show).unwrap().flush()
    }
}

#[derive(Debug)]
pub struct CursorCrtl {
    x: u16,
    y: u16,

    // 用于处理状态位置
    stored_x: u16,
    stored_y: u16,

    line_prefix_width: u16,
    store_flag: bool,

    // 正文模式会输出前缀，这个标志表示是否需要以正文前缀模式调整坐标
    prefix_mode: bool,

    line_setting: LineSetting,

    buf: Arc<EditBuffer>,
}

#[allow(dead_code)]
impl CursorCrtl {
    pub const PREFIX_COL: u16 = 1;
    pub fn new(buf: Arc<EditBuffer>, line_setting: LineSetting) -> Self {
        Self {
            x: 0,
            y: 0,
            stored_x: 0,
            stored_y: 0,
            store_flag: false,
            line_prefix_width: Self::PREFIX_COL,
            prefix_mode: true,
            line_setting,
            buf,
        }
    }

    pub fn x(&self) -> u16 {
        if self.prefix_mode {
            if self.x < self.line_prefix_width {
                return 0;
            }
            self.x - self.line_prefix_width
        } else {
            self.x
        }
    }

    pub fn y(&self) -> u16 {
        self.y
    }

    pub fn cmd_y(&self) -> u16 {
        if self.store_flag {
            self.stored_y
        } else {
            self.y
        }
    }

    #[inline]
    pub fn set_prefix_mode(&mut self, on: bool) {
        self.prefix_mode = on;
        if on && self.x < self.line_prefix_width {
            self.x = self.line_prefix_width;
            self.move_to_columu(0).unwrap();
        }
    }

    pub fn move_to(&mut self, mut x: u16, y: u16) -> io::Result<()> {
        if self.prefix_mode {
            x += self.line_prefix_width;
        }
        let size = *WINSIZE.read().unwrap();
        CursorManager::move_to(x, y)?;
        self.x = (size.cols - 1).min(x);
        self.y = (size.rows - 1).min(y);
        Ok(())
    }

    pub fn move_to_nextline(&mut self, mut lines: u16) -> io::Result<()> {
        let size = *WINSIZE.read().unwrap();
        if self.y + lines >= size.rows {
            // 向上滚动

            // 保存位置
            let pos = self.store_tmp_pos();
            // 计算需要滚动的行数
            let offset = self.buf.offset();
            if offset < lines as usize {
                lines = offset as u16;
            }
            // 重新设置偏移位置
            self.buf.set_offset(offset - lines as usize);
            //翻页并恢复位置
            TermManager::scroll_up(lines)?;
            self.restore_tmp_pos(pos)?;
        }

        CursorManager::move_to_nextline(lines)?;
        if self.prefix_mode {
            self.x = self.line_prefix_width;
            self.move_to_columu(0)?;
        } else {
            self.x = 0;
        }

        self.y += lines;
        Ok(())
    }

    pub fn move_to_previous_line(&mut self, mut lines: u16) -> io::Result<()> {
        if self.y() < lines {
            // 溢出，则向下滚动

            // 保存位置
            let pos = self.store_tmp_pos();
            let offset = self.buf.offset();
            // 计算需要滚动的行数
            let line_count = self.buf.line_count();
            if line_count < offset + lines as usize {
                lines = (line_count - offset) as u16;
            }
            // 重新设置偏移位置
            self.buf.set_offset(offset + lines as usize);
            //翻页并恢复位置
            TermManager::scroll_up(lines)?;
            self.restore_tmp_pos(pos)?;
        }

        CursorManager::move_to_previous_line(lines)?;

        self.y -= lines;
        if self.prefix_mode {
            self.x = self.line_prefix_width;
            self.move_to_columu(0)?;
        } else {
            self.x = 0;
        }
        Ok(())
    }

    pub fn move_to_columu(&mut self, mut col: u16) -> io::Result<()> {
        if self.prefix_mode {
            col += self.line_prefix_width;
        }
        let size = *WINSIZE.read().unwrap();
        CursorManager::move_to_columu(col)?;
        self.x = (size.cols - 1).min(col);
        Ok(())
    }

    pub fn move_to_row(&mut self, row: u16) -> io::Result<()> {
        let size = *WINSIZE.read().unwrap();
        CursorManager::move_to_row(row)?;
        self.y = (size.rows - 1).min(row);
        Ok(())
    }

    pub fn move_up(&mut self, count: u16) -> io::Result<()> {
        CursorManager::move_up(count)?;
        self.y -= count;

        Ok(())
    }

    pub fn move_down(&mut self, count: u16) -> io::Result<()> {
        CursorManager::move_down(count)?;

        self.y += count;
        Ok(())
    }

    pub fn move_left(&mut self, count: u16) -> io::Result<()> {
        let result = match self.x {
            x if x == 0 => Ok(()),
            x if x < count => self.move_to_columu(0),
            x => match self.prefix_mode {
                true if x == self.line_prefix_width - 1 => Ok(()),
                true if x - count < self.line_prefix_width => self.move_to_columu(0),
                _ => {
                    self.x -= count;
                    self.move_to_columu(x - count)
                }
            },
        };

        result
    }

    pub fn move_right(&mut self, count: u16) -> io::Result<()> {
        let mut linesize = self.buf.get_linesize(self.y()) - 1;
        let mut size = *WINSIZE.read().unwrap();
        if self.prefix_mode {
            size.cols -= self.line_prefix_width;
            linesize += self.line_prefix_width;
        }
        if self.x == size.cols - 1 {
            return Ok(());
        }

        if self.x + count > linesize {
            CursorManager::move_to_columu(linesize)?;
            self.x = linesize;
        } else {
            CursorManager::move_right(count)?;
            self.x += count;
        }

        Ok(())
    }

    pub fn write<D: Display>(&mut self, str: D) -> io::Result<()> {
        let str = str.to_string();

        let ss = str.split_terminator(|x| x == '\n').collect::<Vec<&str>>();
        for s in ss {
            self.write_line(s)?;
        }
        Ok(())
    }

    fn write_line(&mut self, str: &str) -> io::Result<()> {
        let len = str.len() as u16;

        let mut size = *WINSIZE.read().unwrap();

        if self.prefix_mode {
            size.cols -= self.line_prefix_width;
        }

        if self.x + len > size.cols {
            let ss = str.split_at((size.cols - self.x) as usize);
            TermIO::write_str(ss.0)?;
            self.move_to_nextline(1)?;
            self.write_line(ss.1)?;
        } else {
            TermIO::write_str(str)?;
            if str.ends_with(|x| x == '\n') {
                self.move_to_nextline(1)?;
            } else {
                self.x += str.len() as u16;
            }
        }

        Ok(())
    }

    pub fn write_with_pos<D: Display>(
        &mut self,
        str: D,
        x: u16,
        y: u16,
        stroe: bool,
    ) -> io::Result<()> {
        let mut pos = (0, 0);
        if stroe {
            pos = self.store_tmp_pos();
        }
        self.move_to(x, y)?;
        self.write(str)?;
        if stroe {
            self.restore_tmp_pos(pos)?;
        }
        Ok(())
    }

    pub fn store_pos(&mut self) {
        if self.store_flag {
            panic!("Stored val doesn't restore")
        }
        self.stored_x = self.x;
        self.stored_y = self.y;
        self.store_flag = true;
    }

    pub fn restore_pos(&mut self) -> io::Result<()> {
        if !self.store_flag {
            panic!("No val stored")
        }
        self.x = self.stored_x;
        self.y = self.stored_y;
        self.store_flag = false;
        CursorManager::move_to(self.stored_x, self.stored_y)
    }

    #[inline]
    pub fn store_tmp_pos(&mut self) -> (u16, u16) {
        (self.x(), self.y())
    }

    pub fn restore_tmp_pos(&mut self, pos: (u16, u16)) -> io::Result<()> {
        self.move_to(pos.0, pos.1)
    }

    /// 更新前缀列
    pub fn update_line_prefix(
        &mut self,
        content: &Vec<LineBuffer>,
        start: u16,
        number_len: usize,
    ) -> io::Result<()> {
        let startline = self.buf.offset() + 1;
        let size = *CONTENT_WINSIZE.read().unwrap();
        let max_line = startline + size.rows as usize;

        // 先关闭prefix模式
        self.set_prefix_mode(false);

        // 绝对坐标
        let (x, y) = (self.x(), self.y());

        // 更新第一列flags
        for (num, line) in content.iter().enumerate() {
            // 设置颜色
            StyleManager::set_background_color(self.line_setting.line_num.background)?;
            StyleManager::set_foreground_color(self.line_setting.line_num.frontground)?;
            self.move_to(0, start + num as u16)?;
            let flags = line.flags;
            flags.set_style()?;
            self.write("~")?;
            StyleManager::reset_color()?;
        }

        // 更新页面行号
        if self.line_setting.line_num.enable {
            let len = number_len + 2;
            self.line_prefix_width = len as u16 + Self::PREFIX_COL;

            // 设置颜色
            StyleManager::set_background_color(self.line_setting.line_num.background)?;
            StyleManager::set_foreground_color(self.line_setting.line_num.frontground)?;
            for line in startline..max_line {
                self.move_to(Self::PREFIX_COL, (line - startline) as u16)?;
                let mut prefix = line.to_string();

                prefix.insert(0, ' ');
                unsafe {
                    let data = prefix.as_mut_vec();
                    data.resize(len, ' ' as u8);
                };

                self.write(prefix)?;
            }
            StyleManager::reset_color()?;
        }
        // 恢复绝对坐标
        self.move_to(x, y)?;

        self.set_prefix_mode(true);

        Ok(())
    }

    pub fn clear_current_line(&mut self) -> io::Result<()> {
        if self.prefix_mode {
            let tmp = self.x();
            self.move_to_columu(0)?;
            TermManager::clear_until_new_line()?;
            self.move_to_columu(tmp)
        } else {
            TermManager::clear_current_line()
        }
    }

    pub fn highlight(&mut self, last_line: Option<u16>) -> io::Result<()> {
        if !self.line_setting.highlight.enable {
            return Ok(());
        }
        DEF_STYLE.read().unwrap().set_content_style()?;

        if last_line.is_some() {
            let last_line = last_line.unwrap();
            // 清除上一行高光
            let pos = self.store_tmp_pos();
            self.move_to(0, last_line)?;
            self.clear_current_line()?;
            self.write(String::from_utf8_lossy(&self.buf.get_line(last_line)))?;
            self.restore_tmp_pos(pos)?;
        }

        let pos = self.store_tmp_pos();
        // 设置高光
        StyleManager::set_background_color(self.line_setting.highlight.color)?;
        self.clear_current_line()?;
        self.move_to_columu(0)?;
        self.write(String::from_utf8_lossy(&self.buf.get_line(self.y())))?;
        self.restore_tmp_pos(pos)?;

        Ok(())
    }
}
