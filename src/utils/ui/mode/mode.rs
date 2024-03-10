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

use crate::utils::ui::event::WarpUiCallBackType;

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

            b"l" | b"L" => {
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

            b"w" | b"W" => {
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
