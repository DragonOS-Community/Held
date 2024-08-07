// 为了不影响 Command 模式的功能，参考Vim源码单独实现临时的 Normal 模式用于演示

// Normal模式下的状态机

// 在 input_data() 中处理输入的数据，根据输入的数据进行状态转移，
// 具体而言，根据输入数据及当前状态来更新状态机的参数，如命令字符，重复次数等

// 在 handle() 中处理状态的转移，根据状态的变化执行相应的操作
// handle() 是真正作用于 ui 和 buffer 的地方，可以在这里调用 buffer 的方法，更新 ui 的显示
// 因此，NormalState 提供给 handle() 的参数应该具有足够的一般性，以适应不同的需求

// 在 exit() 中处理状态的退出，清空状态

// 由此为 ndw,ndd 等命令的实现提供了多重字符匹配之外的方案：
// 状态机的下一个状态仅由输入 + 当前状态决定，避免了对输入的全局匹配

// 另：静态 HashMap 比起 match 性能更好是真的吗？后续是否考虑更换为 HashMap？
// 另：在现有框架下，增加一个新功能，需要在 input_data() 中增加一个分支，handle() 中增加一个分支
// 是否有更简便的代码结构？

// 参考：https://github.com/neovim/neovim/blob/master/src/nvim/normal.c#L89

use lazy_static::lazy_static;

use crate::utils::terminal::TermManager;
use crate::utils::ui::event::KeyEventCallback;
use crate::utils::ui::event::WarpUiCallBackType;
use crate::utils::ui::uicore::UiCore;
use crate::utils::ui::uicore::CONTENT_WINSIZE;
use std::io;
use std::sync::{Mutex, MutexGuard};

use super::mode::ModeType;

#[derive(Debug)]
pub enum BufOpArg {
    Around,  // 操作引号内乃至引号的内容
    Inside,  // 操作引号内的内容
    Line,    // 操作整行
    Word,    // 操作单词
    WordEnd, // 操作单词的末尾
    Block,   // 操作块
}

#[derive(Debug)]
pub struct NormalState {
    pub cmdchar: Option<char>,
    pub count: Option<usize>,
    pub count0: bool,
    pub start_pos: Option<(usize, usize)>,
    pub end_pos: Option<(usize, usize)>,
    pub cmdbuf: Vec<u8>,
    pub buf_op_arg: Option<BufOpArg>,
}

lazy_static! {
    static ref NORMALSTATE: Mutex<NormalState> = Mutex::new(NormalState {
        cmdchar: None,       // 命令开头的字符，通常决定了一类功能，如dw,dd系列命令
        count: None,         // 命令的重复次数，如3j,4k
        count0: false,       // 是否将0作为命令的一部分，在normal模式下，0是一个独立的命令，也可能是一个数字的一部分
        start_pos: None,     // 作用区域的起始位置
        end_pos: None,       // 作用区域的结束位置
        cmdbuf: Vec::new(),  // 用于存储输入的命令，可以与状态的显示通用？
        buf_op_arg: None // 用于指定操作的区域，如daw,diw
    });
}

#[derive(Debug)]
pub(crate) struct Normal;

impl KeyEventCallback for Normal {
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
        ui: &mut MutexGuard<UiCore>,
        data: &[u8],
    ) -> io::Result<WarpUiCallBackType> {
        let mut normal_state = NORMALSTATE.lock().unwrap();
        normal_state.cmdbuf.extend_from_slice(data);
        match data {
            b"h" => {
                normal_state.on_h_clicked();
            }
            b"j" => {
                normal_state.on_j_clicked();
            }
            b"k" => {
                normal_state.on_k_clicked();
            }
            b"l" => {
                normal_state.on_l_clicked();
            }
            b"i" => {
                normal_state.on_i_clicked();
            }
            b"d" => {
                normal_state.on_d_clicked();
            }
            [b'1'..=b'9'] => {
                normal_state.on_nonzero_clicked(data);
            }
            b"0" => {
                normal_state.on_zero_clicked();
            }
            b"w" => {
                normal_state.on_w_clicked();
            }
            b"g" => {
                normal_state.on_g_clicked(ui);
            }
            _ => {
                normal_state.reset();
            }
        }
        return normal_state.handle(ui);
    }
}

impl KeyEventCallback for NormalState {
    fn backspace(&self, ui: &mut MutexGuard<UiCore>) -> io::Result<WarpUiCallBackType> {
        ui.cursor.move_left(1)?;
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
    pub fn reset(&mut self) {
        self.cmdchar = None;
        self.count = None;
        self.count0 = false;
        self.start_pos = None;
        self.end_pos = None;
        self.cmdbuf.clear();
        self.buf_op_arg = None;
    }

    pub fn exec_0_cmd(&mut self, ui: &mut MutexGuard<UiCore>) -> io::Result<WarpUiCallBackType> {
        ui.cursor.move_to_columu(0)?;
        self.reset();
        return Ok(WarpUiCallBackType::None);
    }

    pub fn on_h_clicked(&mut self) {
        self.cmdchar = Some('h');
    }
    /// 向左移动数列
    pub fn exec_h_cmd(&mut self, ui: &mut MutexGuard<UiCore>) -> io::Result<WarpUiCallBackType> {
        let old_x = ui.cursor.x();
        let exec_count = match self.count {
            Some(count) => count.min(old_x as usize),
            None => {
                if old_x == 0 {
                    0
                } else {
                    1
                }
            } // 如果在第一列，不再向左移动，防止溢出
        };
        let new_x = old_x - exec_count as u16;
        ui.cursor.move_to_columu(new_x)?;
        self.reset();
        return Ok(WarpUiCallBackType::None);
    }

    pub fn on_j_clicked(&mut self) {
        self.cmdchar = Some('j');
    }
    /// 向下移动数行
    pub fn exec_j_cmd(&mut self, ui: &mut MutexGuard<UiCore>) -> io::Result<WarpUiCallBackType> {
        let old_y = ui.cursor.y();
        let old_abs_y = old_y + ui.buffer.offset() as u16;
        // 限制最大移动行数
        let exec_count = match self.count {
            Some(count) => count.min(ui.buffer.line_count() - old_abs_y as usize - 1),
            None => 1, // goto_line 会自动处理最大值
        };
        let old_offset = ui.buffer.offset();
        let new_y = ui.buffer.goto_line(old_abs_y as usize + exec_count);
        let new_linesize = ui.buffer.get_linesize(new_y);
        let new_x = if new_linesize < ui.cursor.x() {
            // 如果新行的长度小于原来的x坐标，将光标移动到新行的最后一个字符
            new_linesize - 1
        } else {
            ui.cursor.x()
        };
        ui.cursor.move_to(new_x, new_y)?;
        ui.cursor.highlight(Some(old_y))?;
        // 如果移动后，buffer的offset发生了变化，需要重新渲染
        if ui.buffer.offset() != old_offset {
            ui.render_content(0, CONTENT_WINSIZE.read().unwrap().rows as usize)?;
        }
        self.reset();
        return Ok(WarpUiCallBackType::None);
    }
    pub fn on_k_clicked(&mut self) {
        self.cmdchar = Some('k');
    }

    /// 向上移动数行
    pub fn exec_k_cmd(&mut self, ui: &mut MutexGuard<UiCore>) -> io::Result<WarpUiCallBackType> {
        let old_y = ui.cursor.y();
        let old_abs_y = old_y + ui.buffer.offset() as u16;
        // 限制最大移动行数
        let exec_count = match self.count {
            Some(count) => count.min(old_y as usize + ui.buffer.offset()),
            None => {
                if old_abs_y == 0 {
                    0
                } else {
                    1
                }
            } // 如果在第一行，不再向上移动，防止溢出
        };
        let to_line = old_abs_y as usize - exec_count;
        let old_offset = ui.buffer.offset();
        let new_y = ui.buffer.goto_line(to_line);
        let new_linesize = ui.buffer.get_linesize(new_y);
        let new_x = if new_linesize < ui.cursor.x() {
            // 如果新行的长度小于原来的x坐标，将光标移动到新行的最后一个字符
            new_linesize - 1
        } else {
            ui.cursor.x()
        };
        ui.cursor.move_to(new_x, new_y)?;
        ui.cursor.highlight(Some(old_y))?;
        // 如果移动后，buffer的offset发生了变化，需要重新渲染
        if old_offset != ui.buffer.offset() {
            ui.render_content(0, CONTENT_WINSIZE.read().unwrap().rows as usize)?;
        }
        self.reset();
        return Ok(WarpUiCallBackType::None);
    }

    pub fn on_l_clicked(&mut self) {
        self.cmdchar = Some('l');
    }

    /// 向右移动数列
    pub fn exec_l_cmd(&mut self, ui: &mut MutexGuard<UiCore>) -> io::Result<WarpUiCallBackType> {
        let old_x = ui.cursor.x();
        let linesize = ui.buffer.get_linesize(ui.cursor.y()) as usize;
        let max_count = linesize - old_x as usize - 1;
        let exec_count = match self.count {
            Some(count) => count.min(max_count),
            None => {
                if old_x == linesize as u16 - 1 {
                    0
                } else {
                    1
                }
            }
        };
        let new_x = old_x + exec_count as u16;
        ui.cursor.move_to_columu(new_x)?;
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

    /// 处理输入的非零数字
    pub fn on_nonzero_clicked(&mut self, data: &[u8]) {
        let count = self.count;
        if count.is_none() {
            // 如果count为空，将第一个输入的数字作为count
            let count = data[0] - b'0';
            self.count = Some(count as usize);
        } else {
            // 如果count不为空，将输入的数字添加到count的末尾
            let mut count = count.unwrap();
            count = count * 10 + (data[0] - b'0') as usize;
            self.count = Some(count);
        }
        self.count0 = true; // 将后续输入的0作为执行次数的一部分
    }

    /// 处理输入的0
    pub fn on_zero_clicked(&mut self) {
        // 如果0是命令的一部分，不再处理
        if !self.count0 {
            self.cmdchar = Some('0');
            self.count0 = true;
        }
        let count = self.count;
        // 如果输入的是0，且count不为空，将count扩大10倍
        if count.is_some() {
            let mut count = count.unwrap();
            count = count * 10;
            self.count = Some(count);
        }
    }

    pub fn on_d_clicked(&mut self) {
        match self.cmdchar {
            None => {
                self.cmdchar = Some('d');
            }
            Some('d') => {
                self.buf_op_arg = Some(BufOpArg::Line);
            }
            _ => {
                self.reset();
            }
        }
    }

    fn remove_line(&self, ui: &mut MutexGuard<UiCore>) -> io::Result<()> {
        TermManager::clear_current_line()?;
        TermManager::clear_under_cursor()?;
        let y = ui.cursor.y() as usize;
        let old_line_count = ui.buffer.line_count();
        let old_offset = ui.buffer.offset();

        let count = old_line_count - y as usize;
        ui.buffer.delete_line(y + ui.buffer.offset() as usize);
        ui.render_content(y as u16, count.max(1))?;

        if y + old_offset == old_line_count - 1 {
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

    pub fn exec_d_cmd(&mut self, ui: &mut MutexGuard<UiCore>) -> io::Result<WarpUiCallBackType> {
        let count = match self.count {
            Some(count) => count as u16,
            None => 1,
        };
        match self.buf_op_arg {
            Some(BufOpArg::Line) => {
                let result = self
                    .remove_n_line(ui, count)
                    .map(|_| WarpUiCallBackType::None);
                self.reset();
                return result;
            }
            Some(BufOpArg::Word) => {
                for _ in 0..count {
                    self.remove_word(ui)?;
                }
                self.reset();
                return Ok(WarpUiCallBackType::None);
            }
            _ => {
                return Ok(WarpUiCallBackType::None);
            }
        }
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
            ui.buffer.delete_line(y.into());
            self.down(ui)?;
        }
        ui.render_content(y, 1)?;
        return Ok(());
    }

    pub fn on_w_clicked(&mut self) {
        if self.cmdchar.is_none() {
            self.cmdchar = Some('w');
        } else {
            self.buf_op_arg = Some(BufOpArg::Word);
        }
    }

    pub fn exec_w_cmd(&mut self, ui: &mut MutexGuard<UiCore>) -> io::Result<WarpUiCallBackType> {
        let count = match self.count {
            Some(count) => count,
            None => 1,
        };
        for _ in 0..count {
            self.jump_to_next_word(ui)?;
        }
        self.reset();
        return Ok(WarpUiCallBackType::None);
    }

    // fn locate_next_word(&self, ui: &mut MutexGuard<UiCore>, x: u16, y: u16) -> (u16, u16) {
    //     let pos = ui.buffer.search_nextw_begin(x, y);
    //     let linesize = ui.buffer.get_linesize(y);

    //     if pos < linesize as usize {
    //         // 如果下一个单词在当前行，则移动光标到该单词的起始位置
    //         return (pos as u16, y);
    //     } else if y as usize + ui.buffer.offset() < ui.buffer.line_count() - 1 {
    //         // 如果当前行不是最后一行，则移动到下一行的单词起始位置
    //         let next_word_pos = ui.buffer.search_nextw_begin(0, y + 1) as u16;
    //         let next_line_size = ui.buffer.get_linesize(y + 1);
    //         let next_word_pos = next_word_pos.min(next_line_size) as u16;
    //         return (next_word_pos, y + 1);
    //     } else {
    //         // 如果当前行是最后一行，则移动到当前行的末尾
    //         return (linesize as u16 - 1, y);
    //     }
    // }

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

    fn on_g_clicked(&mut self, ui: &mut MutexGuard<UiCore>) {
        if self.cmdchar.is_none() {
            self.cmdchar = Some('g');
        } else {
            let first_line_size = ui.buffer.get_linesize(0);
            self.end_pos = Some((ui.cursor.x().min(first_line_size - 1) as usize, 0));
        }
    }

    fn exec_g_cmd(&mut self, ui: &mut MutexGuard<UiCore>) -> io::Result<WarpUiCallBackType> {
        let end_pos = self.end_pos;
        if end_pos.is_none() {
            return Ok(WarpUiCallBackType::None);
        }
        let old_y = ui.cursor.y();
        let (x, y) = end_pos.unwrap();
        let y = ui.buffer.goto_line(y);
        ui.cursor.move_to(x as u16, y as u16)?;
        ui.render_content(y, CONTENT_WINSIZE.read().unwrap().rows as usize)?;
        ui.cursor.highlight(Some(old_y))?;
        self.reset();
        return Ok(WarpUiCallBackType::None);
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
            'h' => self.exec_h_cmd(ui),
            'j' => self.exec_j_cmd(ui),
            'k' => self.exec_k_cmd(ui),
            'l' => self.exec_l_cmd(ui),
            'i' => self.exec_i_cmd(ui),
            '0' => self.exec_0_cmd(ui),
            'd' => self.exec_d_cmd(ui),
            'w' => self.exec_w_cmd(ui),
            'g' => self.exec_g_cmd(ui),
            _ => return Ok(WarpUiCallBackType::None),
        }
    }

    fn exit(&mut self) -> io::Result<()> {
        self.reset();
        Ok(())
    }
}
