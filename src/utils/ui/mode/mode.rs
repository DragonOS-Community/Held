use std::io::Read;
use std::sync::atomic::Ordering;
use std::sync::{Mutex, MutexGuard};
use std::{fmt::Debug, io};

use crate::config::lastline_cmd::LastLineCommand;
use crate::utils::buffer::LineState;
#[cfg(feature = "dragonos")]
use crate::utils::input::KeyEventType;

use crate::utils::terminal::TermManager;

use crate::utils::ui::uicore::{UiCore, APP_INFO, TAB_SIZE};
use crate::utils::ui::{
    event::KeyEventCallback,
    uicore::{CONTENT_WINSIZE, DEF_STYLE},
};

#[cfg(feature = "dragonos")]
use super::reg::REG;
#[cfg(feature = "dragonos")]
use crate::utils::buffer::LineBuffer;
use crate::utils::ui::event::WarpUiCallBackType;
#[cfg(not(feature = "dragonos"))]
use arboard::Clipboard;

pub trait InputMode: KeyEventCallback + Debug {
    fn mode_type(&self) -> ModeType;

    #[cfg(not(feature = "dragonos"))]
    fn event_route(
        &self,
        ui: &mut MutexGuard<UiCore>,
        event: crossterm::event::Event,
    ) -> io::Result<WarpUiCallBackType> {
        match event {
            crossterm::event::Event::FocusGained => todo!(),
            crossterm::event::Event::FocusLost => todo!(),
            crossterm::event::Event::Key(key) => self.key_event_route(ui, key),
            crossterm::event::Event::Mouse(_) => todo!(),
            crossterm::event::Event::Paste(_) => todo!(),
            crossterm::event::Event::Resize(_, _) => todo!(),
        }
    }

    #[cfg(not(feature = "dragonos"))]
    fn key_event_route(
        &self,
        ui: &mut MutexGuard<UiCore>,
        keyev: crossterm::event::KeyEvent,
    ) -> io::Result<WarpUiCallBackType> {
        let callback = match keyev.code {
            crossterm::event::KeyCode::Backspace => self.backspace(ui)?,
            crossterm::event::KeyCode::Enter => self.enter(ui)?,
            crossterm::event::KeyCode::Left => self.left(ui)?,
            crossterm::event::KeyCode::Right => self.right(ui)?,
            crossterm::event::KeyCode::Up => self.up(ui)?,
            crossterm::event::KeyCode::Down => self.down(ui)?,
            crossterm::event::KeyCode::Home => todo!(),
            crossterm::event::KeyCode::End => todo!(),
            crossterm::event::KeyCode::PageUp => todo!(),
            crossterm::event::KeyCode::PageDown => todo!(),
            crossterm::event::KeyCode::Tab => self.tab(ui)?,
            crossterm::event::KeyCode::BackTab => todo!(),
            crossterm::event::KeyCode::Delete => todo!(),
            crossterm::event::KeyCode::Insert => todo!(),
            crossterm::event::KeyCode::F(_) => todo!(),
            crossterm::event::KeyCode::Char(c) => self.input_data(ui, &[c as u8])?,
            crossterm::event::KeyCode::Null => todo!(),
            crossterm::event::KeyCode::Esc => self.esc(ui)?,
            crossterm::event::KeyCode::CapsLock => todo!(),
            crossterm::event::KeyCode::ScrollLock => todo!(),
            crossterm::event::KeyCode::NumLock => todo!(),
            crossterm::event::KeyCode::PrintScreen => todo!(),
            crossterm::event::KeyCode::Pause => todo!(),
            crossterm::event::KeyCode::Menu => todo!(),
            crossterm::event::KeyCode::KeypadBegin => todo!(),
            crossterm::event::KeyCode::Media(_) => todo!(),
            crossterm::event::KeyCode::Modifier(_) => todo!(),
        };

        Ok(callback)
    }

    #[cfg(feature = "dragonos")]
    fn key_event_route(
        &self,
        ui: &mut MutexGuard<UiCore>,
        key: KeyEventType,
    ) -> io::Result<WarpUiCallBackType> {
        match key {
            KeyEventType::Common(c) => self.input_data(ui, &[c]),
            KeyEventType::Up => self.up(ui),
            KeyEventType::Down => self.down(ui),
            KeyEventType::Right => self.right(ui),
            KeyEventType::Left => self.left(ui),
            KeyEventType::Enter => self.enter(ui),
            KeyEventType::Tab => self.tab(ui),
            KeyEventType::Backspace => self.backspace(ui),
            KeyEventType::Esc => self.esc(ui),
            KeyEventType::Unknown(_) => {
                ui.update_bottom_state_bar()?;
                Ok(WarpUiCallBackType::None)
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ModeType {
    Command,
    LastLine,
    Insert,
}

impl InputMode for Command {
    fn mode_type(&self) -> ModeType {
        ModeType::Command
    }
}
impl InputMode for LastLine {
    fn mode_type(&self) -> ModeType {
        ModeType::LastLine
    }
}
impl InputMode for Insert {
    fn mode_type(&self) -> ModeType {
        ModeType::Insert
    }
}

pub trait GetClipboard {
    fn get_clipboard(&self) -> String;
    fn set_clipboard(&self, content: &str);
}

impl GetClipboard for Command {
    #[cfg(not(feature = "dragonos"))]
    fn get_clipboard(&self) -> String {
        let mut ctx = Clipboard::new().unwrap();
        let content = ctx.get_text().unwrap();
        return content;
    }
    #[cfg(feature = "dragonos")]
    fn get_clipboard(&self) -> String {
        let linebuf = match REG.pop() {
            Some(buf) => buf,
            None => return String::new(),
        };

        let content = String::from_utf8_lossy(&linebuf.data).to_string();
        return content;
    }
    #[cfg(not(feature = "dragonos"))]
    fn set_clipboard(&self, content: &str) {
        let mut ctx = Clipboard::new().unwrap();
        ctx.set_text(content.to_string()).unwrap();
    }
    #[cfg(feature = "dragonos")]
    fn set_clipboard(&self, content: &str) {
        let mut linebuf = LineBuffer::new(Vec::new());
        linebuf.data.extend_from_slice(content.as_bytes());
        REG.push(linebuf);
    }
}

#[derive(Debug)]
pub struct Command;

impl Command {
    pub fn jump_to_next_flag(
        &self,
        ui: &mut MutexGuard<UiCore>,
        flags: LineState,
    ) -> io::Result<()> {
        let offset = ui.buffer.offset();
        let y = ui.cursor.y() as usize;

        let start_line_number = offset + y + 1;
        if start_line_number >= ui.buffer.line_count() {
            return Ok(());
        }

        let content = &ui.buffer.all_buffer()[start_line_number..];

        // 下一个flaged位置
        let idx = content.iter().position(|x| x.flags.contains(flags));

        if idx.is_some() {
            // y + idx
            let line_number = start_line_number + idx.unwrap();
            let new_y = ui.buffer.goto_line(line_number);
            ui.render_content(0, CONTENT_WINSIZE.read().unwrap().rows as usize)?;
            ui.cursor.move_to_row(new_y)?;
            ui.cursor.highlight(Some(y as u16))?;
        }

        Ok(())
    }

    pub fn jump_to_previous_flag(
        &self,
        ui: &mut MutexGuard<UiCore>,
        flags: LineState,
    ) -> io::Result<()> {
        let offset = ui.buffer.offset();
        let y = ui.cursor.y() as usize;
        if offset == 0 && y == 0 {
            return Ok(());
        }
        let end_linenumber = offset + y - 1;

        let content = &ui.buffer.all_buffer()[0..end_linenumber];

        // 下一个flaged位置
        let idx = content.iter().rposition(|x| x.flags.contains(flags));

        if idx.is_some() {
            // y + idx
            let new_y = ui.buffer.goto_line(idx.unwrap());
            ui.render_content(0, CONTENT_WINSIZE.read().unwrap().rows as usize)?;
            ui.cursor.move_to_row(new_y)?;
            ui.cursor.highlight(Some(y as u16))?;
        }

        Ok(())
    }

    fn jump_to_first_char(&self, ui: &mut MutexGuard<UiCore>) -> io::Result<WarpUiCallBackType> {
        // 移动到行第一个单词的首字母
        let first_char_pos = {
            let line = ui.buffer.get_line(ui.cursor.y()).data;
            let mut idx = 0;
            for char in line {
                if char == b" "[0] {
                    idx += 1;
                } else if char == b"\t"[0] {
                    idx += 4;
                } else {
                    break;
                }
            }
            idx
        };

        ui.cursor.move_to_columu(first_char_pos as u16)?;
        return Ok(WarpUiCallBackType::None);
    }

    fn do_delete_on_d_clicked(
        &self,
        ui: &mut MutexGuard<UiCore>,
    ) -> io::Result<WarpUiCallBackType> {
        let buf: &mut [u8] = &mut [0; 8];
        let _ = io::stdin().read(buf)?;

        match buf[0] {
            b'd' => self.remove_line(ui)?,

            b'0' => {
                let x = ui.cursor.x() as usize;
                let y = ui.cursor.y() as usize;
                match ui.buffer.delete_until_line_beg(x, y) {
                    Some(..) => {
                        // 文本变动重新渲染
                        ui.cursor.move_to_columu(0)?;
                        ui.render_content(y as u16, 1)?;
                    }
                    None => {}
                };
            }
            b'$' => {
                let x = ui.cursor.x() as usize;
                let y = ui.cursor.y() as usize;
                match ui.buffer.delete_until_endl(x, y) {
                    Some(..) => {
                        ui.cursor.move_left(1)?;
                        ui.render_content(y as u16, 1)?;
                    }
                    None => {}
                }
            }

            b'w' | b'e' => self.remove_word(ui)?,

            b'b' => {
                todo!();
                // let old_x = ui.cursor.x();
                // let old_y = ui.cursor.y();

                // self.jump_to_prevw_beg(ui)?;

                // let x = ui.cursor.x();
                // let y = ui.cursor.y();
                // if old_y == y {
                //     ui.buffer.remove_str(x, y, old_x as usize - x as usize);
                //     ui.render_content(y, 1)?;
                // } else {
                //     ui.buffer.delete_until_endl(x as usize, y as usize);
                //     ui.buffer
                //         .delete_until_line_beg(old_x as usize, old_y as usize);
                //     ui.buffer.merge_line(old_y);
                //     let linecount = ui.buffer.line_count();
                //     TermManager::clear_under_cursor()?;
                //     ui.render_content(y, linecount - y as usize - 1)?;
                // }
            }
            _ => {}
        }
        return Ok(WarpUiCallBackType::None);
    }

    fn jump_to_next_word(&self, ui: &mut MutexGuard<UiCore>) -> io::Result<WarpUiCallBackType> {
        let x = ui.cursor.x();
        let y = ui.cursor.y();
        let pos = ui.buffer.search_nextw_begin(x, y);
        let linesize = ui.buffer.get_linesize(y);

        if pos < linesize as usize {
            // 如果下一个单词在当前行，则移动光标到该单词的起始位置
            ui.cursor.move_to_columu(pos as u16)?;
        } else if y as usize + ui.buffer.offset() < ui.buffer.line_count() - 1 {
            // 如果当前行不是最后一行，则移动到下一行的单词起始位置
            let next_word_pos = ui.buffer.search_nextw_begin(0, y + 1) as u16;
            self.down(ui)?;
            ui.cursor.move_to_columu(next_word_pos)?;
            ui.cursor.highlight(Some(y))?;
        } else {
            // 如果当前行是最后一行，则移动到当前行的末尾
            ui.cursor.move_to_columu(linesize as u16 - 1)?;
        }
        return Ok(WarpUiCallBackType::None);
    }

    fn jump_to_nextw_ending(&self, ui: &mut MutexGuard<UiCore>) -> io::Result<WarpUiCallBackType> {
        let x = ui.cursor.x();
        let y = ui.cursor.y();
        let linesize = ui.buffer.get_linesize(y) as usize;

        // 如果光标已经在当前行的末尾或最后一个字符(x + 2)，则尝试移动到下一行的末尾或单词末尾
        if x as usize + 2 >= linesize {
            // y的绝对位置
            let abs_y = ui.buffer.offset() + y as usize;
            if abs_y < ui.buffer.line_count() - 1 {
                let next_end_pos = ui.buffer.search_nextw_end(0, y + 1) as u16;
                self.down(ui)?;
                ui.cursor.move_to_columu(next_end_pos)?;
                ui.cursor.highlight(Some(y))?;
            } else {
                // 如果已经是最后一行，则保持光标在当前行的末尾
                ui.cursor.move_to_columu(linesize as u16 - 1)?;
            }
            return Ok(WarpUiCallBackType::None);
        }

        let next_end_pos = ui.buffer.search_nextw_end(x, y) as u16;
        // 如果下一个单词的末尾在当前行，则移动光标到该单词的末尾
        ui.cursor
            .move_to_columu(next_end_pos.min(linesize as u16 - 2))?;
        return Ok(WarpUiCallBackType::None);
    }

    fn jump_to_prevw_beg(&self, ui: &mut MutexGuard<UiCore>) -> io::Result<WarpUiCallBackType> {
        let x = ui.cursor.x();
        let y = ui.cursor.y();

        // 如果光标已在行首，则尝试移动到上一行的单词首字母
        if x == 0 {
            if y > 0 {
                let end_of_prev_line = ui.buffer.get_linesize(y - 1) - 1;
                let prev_word_pos = match ui.buffer.search_prevw_begin(end_of_prev_line, y - 1) {
                    Some(pos) => pos,
                    None => 0,
                };
                ui.cursor.move_to(prev_word_pos as u16, y - 1)?;
                ui.cursor.highlight(Some(y))?;
            } else {
                // 如果已经是第一行，则保持光标在当前行的起始位置
                ui.cursor.move_to_columu(0)?;
            }
            return Ok(WarpUiCallBackType::None);
        }

        let prev_word_pos = match ui.buffer.search_prevw_begin(x, y) {
            Some(pos) => pos,
            None => 0,
        };

        ui.cursor.move_to(prev_word_pos as u16, y)?;
        return Ok(WarpUiCallBackType::None);
    }

    fn remove_word(&self, ui: &mut MutexGuard<UiCore>) -> io::Result<()> {
        let x = ui.cursor.x();
        let y = ui.cursor.y();
        let next_word_pos = ui.buffer.search_nextw_begin(x, y);
        let linesize = ui.buffer.get_linesize(y);

        // 如果下一个单词在当前行，则删除当前单词
        if next_word_pos < linesize.into() {
            ui.buffer.remove_str(x, y, next_word_pos - x as usize);
        } else {
            // 如果下一个单词在下一行，则删除当前行剩余部分
            self.left(ui)?;
            ui.buffer.delete_until_endl(x.into(), y.into());
        }
        ui.render_content(y, 1)?;
        return Ok(());
    }

    fn remove_n_word(&self, ui: &mut MutexGuard<UiCore>, n: u16) -> io::Result<()> {
        let old_x = ui.cursor.x();
        let old_y = ui.cursor.y();
        TermManager::clear_current_line()?;
        TermManager::clear_under_cursor()?;
        for _ in 0..n {
            self.jump_to_next_word(ui)?;
        }
        let x = ui.cursor.x();
        let y = ui.cursor.y();
        if old_y == y {
            ui.buffer
                .remove_str(old_x, old_y, x as usize - old_x as usize);
            ui.render_content(y, 1)?;
        } else {
            ui.buffer.delete_until_endl(old_x as usize, old_y as usize);
            ui.buffer.delete_until_line_beg(x as usize + 1, y as usize);
            if y - old_y > 1 {
                ui.buffer.delete_lines(old_y as usize + 1, y as usize - 1);
            }
            let linecount = ui.buffer.line_count();
            ui.render_content(y, linecount - y as usize - 1)?;
        }
        ui.cursor.move_to(old_x, old_y)?;
        Ok(())
    }

    fn remove_line(&self, ui: &mut MutexGuard<UiCore>) -> io::Result<()> {
        TermManager::clear_current_line()?;
        TermManager::clear_under_cursor()?;
        let y = ui.cursor.y() as usize;
        let old_line_count = ui.buffer.line_count();

        let count = old_line_count - y as usize;
        ui.buffer.delete_line(y);
        ui.render_content(y as u16, count.max(1))?;

        if y == old_line_count - 1 {
            self.up(ui)?;
        }

        if old_line_count == 1 {
            ui.cursor.move_to_columu(0)?;
            ui.buffer.insert_char('\n' as u8, 0, 0);
            ui.render_content(0, 1)?;
        }

        Ok(())
    }

    fn remove_n_line(&self, ui: &mut MutexGuard<UiCore>, n: u16) -> io::Result<()> {
        let linecount = ui.buffer.line_count() as u16;
        let y = ui.cursor.y();

        // 实际能删除的行数
        let to_delete = n.min(linecount - y);
        for _ in 0..to_delete {
            self.remove_line(ui)?;
        }
        Ok(())
    }

    /// 移动到最后一行
    fn jump_to_last_line(&self, ui: &mut MutexGuard<UiCore>) -> io::Result<()> {
        let line_count = ui.buffer.line_count() as u16;
        let y = ui.cursor.y();
        let new_y = ui.buffer.goto_line(line_count as usize - 1);
        ui.render_content(0, CONTENT_WINSIZE.read().unwrap().rows as usize)?;
        ui.cursor.move_to_row(new_y)?;
        ui.cursor.highlight(Some(y))?;
        return Ok(());
    }

    /// 移动到第一行
    fn jump_to_first_line(&self, ui: &mut MutexGuard<UiCore>) -> io::Result<()> {
        let buf: &mut [u8] = &mut [0; 8];
        let _ = io::stdin().read(buf)?;
        if buf[0] as char == 'g' {
            let old_y = ui.cursor.y();
            let y = ui.buffer.goto_line(0);
            ui.cursor.move_to_row(y)?;
            ui.render_content(0, CONTENT_WINSIZE.read().unwrap().rows as usize)?;
            ui.cursor.highlight(Some(old_y))?;
        }
        Ok(())
    }

    // #[cfg(not(feature = "dragonos"))]
    fn do_yank(&self, ui: &mut MutexGuard<UiCore>) -> io::Result<()> {
        let buf = &mut [0; 8];
        let _ = io::stdin().read(buf)?;
        match buf[0] as char {
            'y' => self.yank_line(ui)?,
            'w' => self.yank_word(ui)?,
            'a' => {
                let buf = &mut [0; 8];
                let _ = io::stdin().read(buf)?;
                let pat = buf[0];
                if !self.is_left_bracket(pat) && !self.is_right_bracket(pat) {
                    return Ok(());
                }
                if let Some((left, right)) = self.search_pair(ui, pat) {
                    let mut content: Vec<u8> = Vec::new();
                    content.extend_from_slice(
                        &ui.buffer.get_line(ui.cursor.y()).data[left as usize..right as usize + 1],
                    );
                    // let mut ctx = Clipboard::new().unwrap();
                    // ctx.set_text(String::from_utf8_lossy(&content).to_string())
                    //     .unwrap();
                    self.set_clipboard(&String::from_utf8_lossy(&content).to_string());
                }
            }
            _ => {}
        }
        Ok(())
    }

    // #[cfg(not(feature = "dragonos"))]
    fn yank_word(&self, ui: &mut MutexGuard<UiCore>) -> io::Result<()> {
        let old_x = ui.cursor.x();
        let old_y = ui.cursor.y();
        self.jump_to_nextw_ending(ui)?;
        let x = ui.cursor.x();
        let y = ui.cursor.y();
        ui.cursor.move_to(old_x, old_y)?;
        // 复制(old_x, old_y)到(x, y)之间的内容
        let mut content: Vec<u8> = Vec::new();
        if old_y == y {
            content.extend_from_slice(&ui.buffer.get_line(y).data[old_x as usize..x as usize + 1]);
        } else {
            content.extend_from_slice(&ui.buffer.get_line(old_y).data[old_x as usize..]);
            for i in old_y + 1..y {
                content.extend_from_slice(&ui.buffer.get_line(i).data);
            }
            content.extend_from_slice(&ui.buffer.get_line(y).data[..x as usize + 1]);
        }
        // let mut ctx = Clipboard::new().unwrap();
        // ctx.set_text(String::from_utf8_lossy(&content).to_string())
        //     .unwrap();
        self.set_clipboard(&String::from_utf8_lossy(&content).to_string());
        Ok(())
    }

    /// 复制当前行到剪切板
    // #[cfg(not(feature = "dragonos"))]
    fn yank_line(&self, ui: &mut MutexGuard<UiCore>) -> io::Result<()> {
        let y = ui.cursor.y();
        let line = ui.buffer.get_line(y);
        // 将当前行的内容复制到剪切板
        // let mut ctx = Clipboard::new().unwrap();
        // ctx.set_text(String::from_utf8_lossy(&line.data).to_string())
        //     .unwrap();
        self.set_clipboard(&String::from_utf8_lossy(&line.data).to_string());
        Ok(())
    }

    /// 粘贴剪切板内容
    // #[cfg(not(feature = "dragonos"))]
    fn paste(&self, ui: &mut MutexGuard<UiCore>, content: &str, x: u16, y: u16) -> io::Result<()> {
        for (idx, ch) in content.as_bytes().iter().enumerate() {
            ui.buffer.insert_char(*ch, x + idx as u16, y);
        }

        ui.render_content(y, crossterm::terminal::size()?.0 as usize)?;
        ui.cursor.move_to_columu(x + content.len() as u16)?;
        Ok(())
    }

    /// 向下粘贴一行
    // #[cfg(not(feature = "dragonos"))]
    fn paste_line(
        &self,
        ui: &mut MutexGuard<UiCore>,
        content: &str,
        x: u16,
        y: u16,
    ) -> io::Result<()> {
        ui.buffer.input_enter(ui.buffer.get_linesize(y) - 1, y);
        for (idx, ch) in content.as_bytes().iter().enumerate() {
            ui.buffer.insert_char(*ch, x + idx as u16, y + 1);
        }
        ui.render_content(y, crossterm::terminal::size()?.0 as usize)?;
        self.down(ui)?;
        Ok(())
    }

    // #[cfg(not(feature = "dragonos"))]
    fn do_paste(&self, ui: &mut MutexGuard<UiCore>) -> io::Result<()> {
        let x = ui.cursor.x();
        let y = ui.cursor.y();
        let lineflag = ui.buffer.line_flags(y);
        if lineflag.contains(LineState::LOCKED) {
            APP_INFO.lock().unwrap().info = "Row is locked".to_string();
            return Err(io::Error::new(io::ErrorKind::Other, "Row is locked"));
        }
        // let mut ctx = Clipboard::new().unwrap();
        // let content = ctx.get_text().unwrap();
        let content = self.get_clipboard();

        if content.ends_with('\n') {
            self.paste_line(ui, &content, x, y)?;
        } else {
            self.paste(ui, &content, x, y)?;
        }
        Ok(())
    }

    /// 定位最近的一对括号，根据左括号定位右括号
    /// 如果找到则返回括号的位置，否则返回None
    fn search_pairs_by_left_pat(
        &self,
        ui: &mut MutexGuard<UiCore>,
        left_pat: u8,
    ) -> Option<(u16, u16)> {
        let x = ui.cursor.x();
        let y = ui.cursor.y();
        let mut left = x as i32;
        let mut right = x as i32;
        let line = ui.buffer.get_line(y).data;
        let linesize = ui.buffer.get_linesize(y);
        let right_pat = self.get_right_pair(left_pat);
        // 尝试往前找到左括号
        while left >= 0 && line[left as usize] != left_pat {
            left -= 1;
        }
        // 未找到左括号，尝试往后找左括号
        if left < 0 {
            left = x as i32;
            while left <= right && right < linesize as i32 {
                if line[left as usize] != left_pat {
                    left += 1;
                    right += 1;
                    continue;
                }
                if right_pat == line[right as usize] {
                    break;
                }
                right += 1;
            }
        } else {
            // 找到左括号，尝试往后找右括号
            right = left + 1;
            while right < linesize as i32 {
                if line[right as usize] == right_pat {
                    break;
                }
                right += 1;
            }
        }
        // 匹配失败
        if right >= linesize.into() {
            return None;
        }
        return Some((left as u16, right as u16));
    }

    /// 定位最近的一对括号，根据右括号定位左括号
    /// 返回左括号和右括号的位置
    fn search_pairs_by_right_pat(
        &self,
        ui: &mut MutexGuard<UiCore>,
        right_pat: u8,
    ) -> Option<(u16, u16)> {
        let x = ui.cursor.x();
        let y = ui.cursor.y();
        let mut left = x as i32;
        let mut right = x as i32;
        let line = ui.buffer.get_line(y).data;
        let linesize = ui.buffer.get_linesize(y);
        // 尝试往后找到右括号
        while right < linesize as i32 && line[right as usize] != right_pat {
            right += 1;
        }
        // 未找到右括号，尝试往前找右括号
        if right >= linesize as i32 {
            right = x as i32;
            while right >= left && left >= 0 {
                if line[right as usize] != right_pat {
                    right -= 1;
                    left -= 1;
                    continue;
                }
                if line[left as usize] == self.get_left_pair(right_pat) {
                    break;
                }
                left -= 1;
            }
        } else {
            // 找到右括号，尝试往前找左括号
            left = right - 1;
            while left >= 0 {
                if line[left as usize] == self.get_left_pair(right_pat) {
                    break;
                }
                left -= 1;
            }
        }
        // 匹配失败
        if left < 0 {
            return None;
        }
        return Some((left as u16, right as u16));
    }

    fn is_left_bracket(&self, ch: u8) -> bool {
        match ch {
            b'(' | b'[' | b'{' | b'<' => true,
            _ => false,
        }
    }

    fn is_right_bracket(&self, ch: u8) -> bool {
        match ch {
            b')' | b']' | b'}' | b'>' => true,
            _ => false,
        }
    }

    fn get_right_pair(&self, ch: u8) -> u8 {
        match ch {
            b'(' => b')',
            b'[' => b']',
            b'{' => b'}',
            b'<' => b'>',
            _ => 0,
        }
    }

    fn get_left_pair(&self, ch: u8) -> u8 {
        match ch {
            b')' => b'(',
            b']' => b'[',
            b'}' => b'{',
            b'>' => b'<',
            _ => 0,
        }
    }

    fn search_pair(&self, ui: &mut MutexGuard<UiCore>, pat: u8) -> Option<(u16, u16)> {
        if self.is_left_bracket(pat) {
            return self.search_pairs_by_left_pat(ui, pat);
        } else if self.is_right_bracket(pat) {
            return self.search_pairs_by_right_pat(ui, pat);
        }
        return None;
    }
}

impl KeyEventCallback for Command {
    fn backspace(&self, _ui: &mut MutexGuard<UiCore>) -> io::Result<WarpUiCallBackType> {
        Ok(WarpUiCallBackType::None)
    }
    fn enter(&self, _ui: &mut MutexGuard<UiCore>) -> io::Result<WarpUiCallBackType> {
        Ok(WarpUiCallBackType::None)
    }

    fn tab(&self, _ui: &mut MutexGuard<UiCore>) -> io::Result<WarpUiCallBackType> {
        Ok(WarpUiCallBackType::None)
    }

    fn esc(&self, _ui: &mut MutexGuard<UiCore>) -> io::Result<WarpUiCallBackType> {
        Ok(WarpUiCallBackType::None)
    }

    fn input_data(
        &self,
        ui: &mut MutexGuard<UiCore>,
        data: &[u8],
    ) -> io::Result<WarpUiCallBackType> {
        match data {
            b":" => {
                // 保存位置
                ui.cursor.store_pos();
                return Ok(WarpUiCallBackType::ChangMode(ModeType::LastLine));
            }

            b"i" | b"I" => {
                // 切换Insert模式
                return Ok(WarpUiCallBackType::ChangMode(ModeType::Insert));
            }

            // hjkl 与 Vim 的效果一致
            b"h" => self.left(ui),

            // 向下
            b"j" => self.down(ui),

            // 向上
            b"k" => self.up(ui),

            //  向右
            b"l" => self.right(ui),

            b"L" => {
                // 设置当前行lock
                let flag = ui.buffer.line_flags(ui.cursor.y());
                let offset = ui.buffer.offset();
                if flag.contains(LineState::LOCKED) {
                    ui.buffer
                        .remove_line_flags(offset + ui.cursor.y() as usize, LineState::LOCKED);
                } else {
                    ui.buffer
                        .add_line_flags(offset + ui.cursor.y() as usize, LineState::LOCKED);
                }
                let y = ui.cursor.y();
                ui.render_content(y, 1)?;
                return Ok(WarpUiCallBackType::None);
            }

            b"f" | b"F" => {
                // 设置当前行flag
                let flag = ui.buffer.line_flags(ui.cursor.y());
                let offset = ui.buffer.offset();
                if flag.contains(LineState::FLAGED) {
                    ui.buffer
                        .remove_line_flags(offset + ui.cursor.y() as usize, LineState::FLAGED);
                } else {
                    ui.buffer
                        .add_line_flags(offset + ui.cursor.y() as usize, LineState::FLAGED);
                }

                let y = ui.cursor.y();
                ui.render_content(y, 1)?;
                return Ok(WarpUiCallBackType::None);
            }

            b"q" | b"Q" => {
                // 跳转到上一个flag行
                self.jump_to_previous_flag(ui, LineState::FLAGED)?;
                return Ok(WarpUiCallBackType::None);
            }

            b"w" => self.jump_to_next_word(ui),

            b"e" => self.jump_to_nextw_ending(ui),

            b"b" => self.jump_to_prevw_beg(ui),

            b"W" => {
                // 跳转到下一个flag行
                self.jump_to_next_flag(ui, LineState::FLAGED)?;
                return Ok(WarpUiCallBackType::None);
            }

            b"a" | b"A" => {
                self.jump_to_previous_flag(ui, LineState::LOCKED)?;
                return Ok(WarpUiCallBackType::None);
            }

            b"s" | b"S" => {
                self.jump_to_next_flag(ui, LineState::LOCKED)?;
                return Ok(WarpUiCallBackType::None);
            }

            b"0" => {
                // 移动到行首
                ui.cursor.move_to_columu(0)?;
                return Ok(WarpUiCallBackType::None);
            }

            b"^" => self.jump_to_first_char(ui),

            b"$" => {
                // 移动到行末
                let line_end = ui.buffer.get_linesize(ui.cursor.y()) - 1;
                ui.cursor.move_to_columu(line_end)?;
                return Ok(WarpUiCallBackType::None);
            }

            b"d" => self.do_delete_on_d_clicked(ui),

            b"x" => {
                let y = ui.cursor.y();
                let x = ui.cursor.x();
                if x < ui.buffer.get_linesize(y) - 1 {
                    ui.buffer.remove_char(x, y);
                    ui.render_content(y, 1)?;
                }
                return Ok(WarpUiCallBackType::None);
            }

            b"G" => {
                self.jump_to_last_line(ui)?;
                return Ok(WarpUiCallBackType::None);
            }

            b"g" => {
                self.jump_to_first_line(ui)?;
                return Ok(WarpUiCallBackType::None);
            }

            b"y" => {
                self.do_yank(ui)?;
                return Ok(WarpUiCallBackType::None);
            }

            b"p" => {
                self.do_paste(ui)?;
                return Ok(WarpUiCallBackType::None);
            }

            _ => {
                return Ok(WarpUiCallBackType::None);
            }
        }
    }
}

#[derive(Debug)]
pub struct Insert;
impl KeyEventCallback for Insert {
    fn enter(&self, ui: &mut MutexGuard<UiCore>) -> io::Result<WarpUiCallBackType> {
        let line_idx = ui.cursor.y();
        let col = ui.cursor.x();

        let line = ui.buffer.get_line(line_idx);
        if line.flags.contains(LineState::LOCKED) {
            APP_INFO.lock().unwrap().info = "Row is locked".to_string();
            return Ok(WarpUiCallBackType::None);
        }
        ui.buffer.input_enter(col, line_idx);

        DEF_STYLE.read().unwrap().set_content_style()?;
        // 清空改行光标后的内容
        TermManager::clear_until_new_line()?;

        // 执行渲染后续文本
        ui.cursor.move_to_nextline(1)?;
        ui.cursor.clear_current_line()?;

        let ret = ui.render_content(
            line_idx + 1,
            (CONTENT_WINSIZE.read().unwrap().rows - line_idx) as usize,
        )?;

        if ret == 0 {
            ui.scroll_up(1)?;
            ui.render_content(
                line_idx + 1,
                (CONTENT_WINSIZE.read().unwrap().rows - line_idx) as usize,
            )?;

            ui.cursor.move_up(1)?;
        }

        let last = ui.cursor.y() - 1;
        ui.cursor.highlight(Some(last))?;
        ui.set_edited();
        Ok(WarpUiCallBackType::None)
    }

    fn tab(&self, ui: &mut MutexGuard<UiCore>) -> io::Result<WarpUiCallBackType> {
        ui.set_edited();
        let x = ui.cursor.x();

        let tab_size = TAB_SIZE.load(Ordering::SeqCst);
        let space_size = tab_size - (x % tab_size);

        for _ in 0..space_size {
            ui.buffer
                .insert_char(' ' as u8, ui.cursor.x(), ui.cursor.y());
        }

        let y = ui.cursor.y();
        ui.render_content(y, 1)?;

        ui.cursor.move_right(space_size)?;

        Ok(WarpUiCallBackType::None)
    }

    fn esc(&self, _ui: &mut MutexGuard<UiCore>) -> io::Result<WarpUiCallBackType> {
        Ok(WarpUiCallBackType::ChangMode(ModeType::Command))
    }

    fn input_data(
        &self,
        ui: &mut MutexGuard<UiCore>,
        data: &[u8],
    ) -> io::Result<WarpUiCallBackType> {
        let x = ui.cursor.x();
        let y = ui.cursor.y();

        let line = ui.buffer.get_line(y);
        if line.flags.contains(LineState::LOCKED) {
            APP_INFO.lock().unwrap().info = "Row is locked".to_string();
            return Ok(WarpUiCallBackType::None);
        }

        for (idx, ch) in data.iter().enumerate() {
            ui.buffer.insert_char(*ch, x + idx as u16, y);
        }

        let line_data = ui.buffer.get_line(y);

        // 考虑长度包含\n,所以要减1
        ui.cursor.write(String::from_utf8_lossy(
            &line_data.data[x as usize..(line_data.size() - 1)],
        ))?;

        ui.cursor.move_to_columu(x + data.len() as u16)?;
        ui.set_edited();
        ui.cursor.highlight(None)?;
        Ok(WarpUiCallBackType::None)
    }
}

#[derive(Debug)]
pub struct LastLine {
    buf: Mutex<Vec<u8>>,
}

impl LastLine {
    pub fn new() -> Self {
        Self {
            buf: Mutex::new(vec![':' as u8]),
        }
    }

    pub fn reset(&self) {
        self.buf.lock().unwrap().resize(1, ':' as u8);
    }
}

impl KeyEventCallback for LastLine {
    fn enter(&self, ui: &mut MutexGuard<UiCore>) -> io::Result<WarpUiCallBackType> {
        let mut buf = self.buf.lock().unwrap();
        let cmd = String::from_utf8_lossy(&buf).to_string();

        let ret = LastLineCommand::process(ui, cmd);

        ui.cursor.move_to(1, u16::MAX - 1)?;
        // ui.cursor.move_to_columu(1)?;
        TermManager::clear_until_new_line()?;
        ui.cursor.move_to(1, u16::MAX - 1)?;

        buf.resize(1, 0);
        if ret == WarpUiCallBackType::None {
            ui.cursor.restore_pos()?;
            return Ok(WarpUiCallBackType::ChangMode(ModeType::Command));
        }

        Ok(ret)
    }

    fn tab(&self, _ui: &mut MutexGuard<UiCore>) -> io::Result<WarpUiCallBackType> {
        Ok(WarpUiCallBackType::None)
    }

    fn backspace(&self, ui: &mut MutexGuard<UiCore>) -> io::Result<WarpUiCallBackType> {
        if ui.cursor.x() == 1 {
            return Ok(WarpUiCallBackType::None);
        }

        self.left(ui)?;
        self.buf.lock().unwrap().remove(ui.cursor.x() as usize);

        ui.cursor.write(' ')?;
        self.left(ui)?;

        Ok(WarpUiCallBackType::None)
    }

    fn up(&self, _ui: &mut MutexGuard<UiCore>) -> io::Result<WarpUiCallBackType> {
        Ok(WarpUiCallBackType::None)
    }

    fn down(&self, _ui: &mut MutexGuard<UiCore>) -> io::Result<WarpUiCallBackType> {
        Ok(WarpUiCallBackType::None)
    }

    fn esc(&self, ui: &mut MutexGuard<UiCore>) -> io::Result<WarpUiCallBackType> {
        ui.cursor.restore_pos()?;
        Ok(WarpUiCallBackType::ChangMode(ModeType::Command))
    }

    fn input_data(
        &self,
        ui: &mut MutexGuard<UiCore>,
        data: &[u8],
    ) -> io::Result<WarpUiCallBackType> {
        let mut buf = self.buf.lock().unwrap();

        if ui.cursor.x() == buf.len() as u16 {
            buf.extend(data);
        } else {
            let index = ui.cursor.x() as usize;
            for (i, &item) in data.iter().enumerate() {
                buf.insert(index + i, item);
            }
        }

        ui.cursor.write(String::from_utf8_lossy(&data))?;

        Ok(WarpUiCallBackType::None)
    }
}
