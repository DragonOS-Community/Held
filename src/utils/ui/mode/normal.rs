use lazy_static::lazy_static;

use crate::utils::ui::event::KeyEventCallback;
use crate::utils::ui::event::WarpUiCallBackType;
use crate::utils::ui::uicore::UiCore;
use std::io;
use std::sync::{Mutex, MutexGuard};

use super::mode::ModeType;

#[derive(Debug)]
pub struct NormalState {
    pub cmdchar: Option<char>,
    pub count: Option<usize>,
    pub count0: bool,
    pub start_pos: Option<(usize, usize)>,
    pub end_pos: Option<(usize, usize)>,
    pub cmdbuf: Vec<u8>,
}

lazy_static! {
    static ref NORMALSTATE: Mutex<NormalState> = Mutex::new(NormalState {
        cmdchar: None,       // 命令开头的字符，通常决定了一类功能，如dw,dd系列命令
        count: None,         // 命令的重复次数，如3j,4k
        count0: false,       // 是否将0作为命令的一部分，在normal模式下，0是一个独立的命令，也可能是一个数字的一部分
        start_pos: None,     // 作用区域的起始位置
        end_pos: None,       // 作用区域的结束位置
        cmdbuf: Vec::new(),  // 用于存储输入的命令，可以与状态的显示通用？
    });
}

#[derive(Debug)]
pub(crate) struct Normal;

impl KeyEventCallback for Normal {
    fn backspace(&self, _ui: &mut MutexGuard<UiCore>) -> io::Result<WarpUiCallBackType> {
        return Ok(WarpUiCallBackType::None);
    }
    fn esc(&self, _ui: &mut MutexGuard<UiCore>) -> io::Result<WarpUiCallBackType> {
        return Ok(WarpUiCallBackType::None);
    }

    fn enter(&self, _ui: &mut MutexGuard<UiCore>) -> io::Result<WarpUiCallBackType> {
        return Ok(WarpUiCallBackType::None);
    }
    fn tab(&self, _ui: &mut MutexGuard<UiCore>) -> io::Result<WarpUiCallBackType> {
        return Ok(WarpUiCallBackType::None);
    }
    fn input_data(
        &self,
        ui: &mut MutexGuard<UiCore>,
        data: &[u8],
    ) -> io::Result<WarpUiCallBackType> {
        let mut normal_state = NORMALSTATE.lock().unwrap();
        normal_state.cmdbuf.extend_from_slice(data);
        match data {
            b"j" => {
                normal_state.on_j_clicked();
            }
            b"k" => {
                normal_state.on_k_clicked();
            }
            b"i" => {
                normal_state.on_i_clicked();
            }
            [b'1'..=b'9'] => {
                normal_state.on_nonzero_clicked(data);
            }
            b"0" => {
                normal_state.on_zero_clicked();
            }
            _ => {
                normal_state.reset();
            }
        }
        return normal_state.handle(ui);
    }
}

impl KeyEventCallback for NormalState {
    fn backspace(&self, _ui: &mut MutexGuard<UiCore>) -> io::Result<WarpUiCallBackType> {
        return Ok(WarpUiCallBackType::None);
    }
    fn esc(&self, _ui: &mut MutexGuard<UiCore>) -> io::Result<WarpUiCallBackType> {
        return Ok(WarpUiCallBackType::ChangMode(ModeType::Command));
    }

    fn enter(&self, _ui: &mut MutexGuard<UiCore>) -> io::Result<WarpUiCallBackType> {
        return Ok(WarpUiCallBackType::None);
    }
    fn tab(&self, _ui: &mut MutexGuard<UiCore>) -> io::Result<WarpUiCallBackType> {
        return Ok(WarpUiCallBackType::None);
    }
    fn input_data(
        &self,
        _ui: &mut MutexGuard<UiCore>,
        _data: &[u8],
    ) -> io::Result<WarpUiCallBackType> {
        return Ok(WarpUiCallBackType::None);
    }
}
impl NormalState {
    pub fn on_j_clicked(&mut self) {
        self.cmdchar = Some('j');
    }
    pub fn exec_j_cmd(&mut self, ui: &mut MutexGuard<UiCore>) -> io::Result<WarpUiCallBackType> {
        let exec_count = match self.count {
            Some(count) => {
                count.min(ui.buffer.line_count() - ui.cursor.y() as usize - 1 - ui.buffer.offset())
            }
            None => 1,
        };
        let old_y = ui.cursor.y();
        let new_y = ui.buffer.goto_line(old_y as usize + exec_count);
        ui.cursor.move_to_row(new_y)?;
        ui.cursor.highlight(Some(old_y))?;
        self.reset();
        return Ok(WarpUiCallBackType::None);
    }
    pub fn on_k_clicked(&mut self) {
        self.cmdchar = Some('k');
    }

    pub fn exec_k_cmd(&mut self, ui: &mut MutexGuard<UiCore>) -> io::Result<WarpUiCallBackType> {
        let exec_count = match self.count {
            Some(count) => count.min(ui.cursor.y() as usize + ui.buffer.offset()),
            None => 1,
        };
        let old_y = ui.cursor.y();
        let new_y = ui.buffer.goto_line(old_y as usize - exec_count);
        ui.cursor.move_to_row(new_y)?;
        ui.cursor.highlight(Some(old_y))?;
        self.reset();
        return Ok(WarpUiCallBackType::None);
    }

    pub fn on_i_clicked(&mut self) {
        self.cmdchar = Some('i');
    }
    pub fn exec_i_cmd(&mut self, _ui: &mut MutexGuard<UiCore>) -> io::Result<WarpUiCallBackType> {
        self.exit()?;
        return Ok(WarpUiCallBackType::ChangMode(ModeType::Insert));
    }

    pub fn on_nonzero_clicked(&mut self, data: &[u8]) {
        let count = self.count;
        if count.is_none() {
            let count = data[0] - b'0';
            self.count = Some(count as usize);
        } else {
            let mut count = count.unwrap();
            count = count * 10 + (data[0] - b'0') as usize;
            self.count = Some(count);
        }
        self.count0 = true;
    }

    pub fn on_zero_clicked(&mut self) {
        if !self.count0 {
            self.cmdchar = Some('0');
            self.count0 = true;
        }
        let count = self.count;
        if count.is_none() {
            self.count = Some(0);
        } else {
            let mut count = count.unwrap();
            count = count * 10;
            self.count = Some(count);
        }
    }

    pub fn reset(&mut self) {
        self.cmdchar = None;
        self.count = None;
        self.count0 = false;
        self.start_pos = None;
        self.end_pos = None;
        self.cmdbuf.clear();
    }
}

pub trait StateMachine {
    fn handle(&mut self, ui: &mut MutexGuard<UiCore>) -> io::Result<WarpUiCallBackType>;
    fn exit(&mut self) -> io::Result<()>;
}

impl StateMachine for NormalState {
    fn handle(&mut self, ui: &mut MutexGuard<UiCore>) -> io::Result<WarpUiCallBackType> {
        if self.cmdchar.is_none() {
            return Ok(WarpUiCallBackType::None);
        }
        match self.cmdchar.unwrap() {
            'j' => self.exec_j_cmd(ui),
            'k' => self.exec_k_cmd(ui),
            'i' => self.exec_i_cmd(ui),
            _ => return Ok(WarpUiCallBackType::None),
        }
    }

    fn exit(&mut self) -> io::Result<()> {
        self.reset();
        Ok(())
    }
}
