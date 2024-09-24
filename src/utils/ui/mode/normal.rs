use lazy_static::lazy_static;

use crate::utils::ui::event::KeyEventCallback;
use crate::utils::ui::event::WarpUiCallBackType;
use crate::utils::ui::uicore::UiCore;
use crate::utils::ui::uicore::CONTENT_WINSIZE;
use std::io;
use std::sync::{Mutex, MutexGuard};

use super::common::CommonOp;
use super::matching::find_pair;
use super::matching::PAIRING;
use super::mode::ModeType;
use super::state::StateMachine;
use crate::utils::ui::mode::state::StateCallback;

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

#[derive(Debug)]
pub struct NormalState {
    pub cmdchar: Option<char>,
    pub count: Option<usize>,
    pub count0: bool,
    pub start_pos: Option<(u16, u16)>,
    pub end_pos: Option<(u16, u16)>,
    pub cmdbuf: Vec<u8>,
    pub buf_op_arg: Option<BufOpArg>,
}

impl CommonOp for NormalState {}

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
impl Normal {
    pub fn new() -> Self {
        Self {}
    }
}

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
            b"h" => normal_state.on_h_clicked(),

            b"j" => normal_state.on_j_clicked(),

            b"k" => normal_state.on_k_clicked(),

            b"l" => normal_state.on_l_clicked(),

            b"i" => normal_state.on_i_clicked(),

            b"d" => normal_state.on_d_clicked(),

            [b'1'..=b'9'] => normal_state.on_nonzero_clicked(data),

            b"0" => normal_state.on_zero_clicked(),

            b"w" => normal_state.on_w_clicked(ui),

            b"g" => normal_state.on_g_clicked(),

            b"G" => normal_state.on_G_clicked(ui),

            b"b" => normal_state.on_b_clicked(ui),

            b":" => {
                if normal_state.cmdchar.is_none() {
                    ui.cursor.store_pos();
                    return Ok(WarpUiCallBackType::ChangMode(ModeType::LastLine));
                }
            }
            b"$" => normal_state.on_dollar_clicked(),

            b"e" => normal_state.on_e_clicked(ui),

            b"f" => normal_state.on_f_clicked(),

            b"F" => normal_state.on_F_clicked(),

            b"x" => normal_state.on_x_clicked(),

            b"y" => normal_state.on_y_clicked(),

            b"p" => normal_state.on_p_clicked(),

            b"o" => normal_state.on_o_clicked(),

            b"O" => normal_state.on_O_clicked(),

            b"a" => normal_state.on_a_clicked(),

            b"A" => normal_state.on_A_clicked(),

            b"I" => normal_state.on_I_clicked(),

            b"H" => normal_state.on_H_clicked(),

            b"M" => normal_state.on_M_clicked(),

            _ => {}
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
    pub fn exec_0_cmd(&mut self, ui: &mut MutexGuard<UiCore>) -> io::Result<StateCallback> {
        ui.cursor.move_to_columu(0)?;
        return Ok(StateCallback::Reset);
    }

    pub fn on_h_clicked(&mut self) {
        if self.cmdchar.is_none() {
            self.cmdchar = Some('h');
        }
    }
    /// 向左移动数列
    pub fn exec_h_cmd(&mut self, ui: &mut MutexGuard<UiCore>) -> io::Result<StateCallback> {
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
        return Ok(StateCallback::Reset);
    }

    pub fn on_j_clicked(&mut self) {
        if self.cmdchar.is_none() {
            self.cmdchar = Some('j');
        }
    }
    /// 向下移动数行
    pub fn exec_j_cmd(&mut self, ui: &mut MutexGuard<UiCore>) -> io::Result<StateCallback> {
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
        return Ok(StateCallback::Reset);
    }
    pub fn on_k_clicked(&mut self) {
        if self.cmdchar.is_none() {
            self.cmdchar = Some('k');
        }
    }

    /// 向上移动数行
    pub fn exec_k_cmd(&mut self, ui: &mut MutexGuard<UiCore>) -> io::Result<StateCallback> {
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
        return Ok(StateCallback::Reset);
    }

    pub fn on_l_clicked(&mut self) {
        if self.cmdchar.is_none() {
            self.cmdchar = Some('l');
        }
    }

    /// 向右移动数列
    pub fn exec_l_cmd(&mut self, ui: &mut MutexGuard<UiCore>) -> io::Result<StateCallback> {
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
        return Ok(StateCallback::Reset);
    }

    pub fn on_i_clicked(&mut self) {
        if self.cmdchar.is_none() {
            self.cmdchar = Some('i');
        } else {
            self.buf_op_arg = Some(BufOpArg::Inside);
        }
    }
    pub fn exec_i_cmd(&mut self, _ui: &mut MutexGuard<UiCore>) -> io::Result<StateCallback> {
        return Ok(StateCallback::Exit(ModeType::Insert));
    }

    #[allow(non_snake_case)]
    pub fn on_I_clicked(&mut self) {
        if self.cmdchar.is_none() {
            self.cmdchar = Some('I');
        }
    }

    // 切换Insert模式，从行首开始插入字符
    #[allow(non_snake_case)]
    pub fn exec_I_cmd(&mut self, ui: &mut MutexGuard<UiCore>) -> io::Result<StateCallback> {
        ui.cursor.move_to_columu(0)?;
        return Ok(StateCallback::Exit(ModeType::Insert));
    }

    pub fn on_a_clicked(&mut self) {
        if self.cmdchar.is_none() {
            self.cmdchar = Some('a');
        } else {
            self.buf_op_arg = Some(BufOpArg::Around)
        }
    }

    // 切换Insert模式，从当前位置的下一个字符开始插入
    pub fn exec_a_cmd(&mut self, ui: &mut MutexGuard<UiCore>) -> io::Result<StateCallback> {
        self.right(ui)?;
        return Ok(StateCallback::Exit(ModeType::Insert));
    }

    #[allow(non_snake_case)]
    pub fn on_A_clicked(&mut self) {
        if self.cmdchar.is_none() {
            self.cmdchar = Some('A');
        }
    }

    // 切换Insert模式，从行尾开始插入字符
    #[allow(non_snake_case)]
    pub fn exec_A_cmd(&mut self, ui: &mut MutexGuard<UiCore>) -> io::Result<StateCallback> {
        let line_end = ui.buffer.get_linesize(ui.cursor.y()) - 1;
        ui.cursor.move_to_columu(line_end)?;
        return Ok(StateCallback::Exit(ModeType::Insert));
    }

    pub fn on_o_clicked(&mut self) {
        if self.cmdchar.is_none() {
            self.cmdchar = Some('o');
        }
    }

    // 切换Insert模式，在当前行的下方插入一个新行开始输入文本
    pub fn exec_o_cmd(&mut self, ui: &mut MutexGuard<UiCore>) -> io::Result<StateCallback> {
        let linesize = ui.buffer.get_linesize(ui.cursor.y());
        ui.cursor.move_to_columu(linesize - 1)?;
        ui.buffer.input_enter(ui.cursor.x(), ui.cursor.y());
        ui.cursor.move_to_nextline(1)?;
        return Ok(StateCallback::Exit(ModeType::Insert));
    }

    #[allow(non_snake_case)]
    pub fn on_O_clicked(&mut self) {
        if self.cmdchar.is_none() {
            self.cmdchar = Some('O');
        }
    }

    // 切换Insert模式，在当前行的上方插入一个新行开始输入文本
    #[allow(non_snake_case)]
    pub fn exec_O_cmd(&mut self, ui: &mut MutexGuard<UiCore>) -> io::Result<StateCallback> {
        ui.cursor.move_to_columu(0)?;
        ui.buffer.input_enter(ui.cursor.x(), ui.cursor.y());
        return Ok(StateCallback::Exit(ModeType::Insert));
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
        if !self.count0 && self.cmdchar.is_none() {
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

    /// 处理输入的d
    pub fn on_d_clicked(&mut self) {
        match self.cmdchar {
            None => {
                // 处理d
                self.cmdchar = Some('d');
            }
            Some('d') => {
                // 处理dd
                self.buf_op_arg = Some(BufOpArg::Line);
            }
            _ => {
                self.reset();
            }
        }
    }

    pub fn exec_d_cmd(&mut self, ui: &mut MutexGuard<UiCore>) -> io::Result<StateCallback> {
        let count = match self.count {
            Some(count) => count as u16,
            None => 1,
        };
        match self.buf_op_arg {
            Some(BufOpArg::Line) => {
                // 删除行
                self.remove_n_line(ui, count)?;
                return Ok(StateCallback::Reset);
            }
            Some(BufOpArg::Word) => {
                // 删除单词
                for _ in 0..count {
                    self.remove_word(ui)?;
                }
                return Ok(StateCallback::Reset);
            }
            _ => {
                return Ok(StateCallback::None);
            }
        }
    }

    pub fn on_w_clicked(&mut self, ui: &mut MutexGuard<UiCore>) {
        if self.cmdchar.is_none() {
            // 按单词移动
            self.cmdchar = Some('w');
            let count = match self.count {
                Some(count) => count,
                None => 1,
            };
            let mut pos = (ui.cursor.x(), ui.cursor.y() + ui.buffer.offset() as u16);
            for _ in 0..count {
                pos = self.locate_next_word(ui, pos);
            }
            self.end_pos = Some(pos);
        } else {
            // 按单词操作，具体由self.cmdchar决定
            self.buf_op_arg = Some(BufOpArg::Word);
        }
    }

    pub fn exec_w_cmd(&mut self, ui: &mut MutexGuard<UiCore>) -> io::Result<StateCallback> {
        self.end_pos.map(|pos| {
            self.move_to_line(ui, pos.1).unwrap();
            ui.cursor.move_to_columu(pos.0).unwrap();
        });
        return Ok(StateCallback::Reset);
    }

    fn on_g_clicked(&mut self) {
        if self.cmdchar.is_none() {
            self.cmdchar = Some('g');
        } else {
            self.end_pos = Some((0, 0));
        }
    }

    fn exec_g_cmd(&mut self, ui: &mut MutexGuard<UiCore>) -> io::Result<StateCallback> {
        let rs = self
            .end_pos
            .map(|pos| self.move_to_line(ui, pos.1).unwrap());
        if let None = rs {
            return Ok(StateCallback::None);
        }
        return Ok(StateCallback::Reset);
    }

    #[allow(non_snake_case)]
    fn on_G_clicked(&mut self, _ui: &mut MutexGuard<UiCore>) {
        if self.cmdchar.is_none() {
            self.cmdchar = Some('G');
        }
    }

    #[allow(non_snake_case)]
    fn exec_G_cmd(&mut self, ui: &mut MutexGuard<UiCore>) -> io::Result<StateCallback> {
        let lineidx = match self.count {
            Some(count) => count - 1,
            None => ui.buffer.line_count() - 1,
        };
        self.move_to_line(ui, lineidx as u16)?;
        return Ok(StateCallback::Reset);
    }

    fn on_b_clicked(&mut self, ui: &mut MutexGuard<UiCore>) {
        if self.cmdchar.is_none() {
            self.cmdchar = Some('b');
        } else {
            self.buf_op_arg = Some(BufOpArg::WordBegin);
        }
        let count = match self.count {
            Some(count) => count,
            None => 1,
        };
        let mut pos = (ui.cursor.x(), ui.cursor.y() + ui.buffer.offset() as u16);
        self.start_pos = Some(pos);
        for _ in 0..count {
            pos = self.locate_prevw_begin(ui, pos.0, pos.1);
        }
        self.end_pos = Some(pos);
    }

    fn exec_b_cmd(&mut self, ui: &mut MutexGuard<UiCore>) -> io::Result<StateCallback> {
        let end_pos = self.end_pos.unwrap();
        self.move_to_line(ui, end_pos.1)?;
        ui.cursor.move_to_columu(end_pos.0)?;
        return Ok(StateCallback::Reset);
    }

    fn on_dollar_clicked(&mut self) {
        if self.cmdchar.is_none() {
            self.cmdchar = Some('$');
        }
    }

    fn exec_dollar_cmd(&mut self, ui: &mut MutexGuard<UiCore>) -> io::Result<StateCallback> {
        let line_end = ui.buffer.get_linesize(ui.cursor.y()) as u16 - 1;
        ui.cursor.move_to_columu(line_end)?;
        return Ok(StateCallback::Reset);
    }

    fn on_e_clicked(&mut self, ui: &mut MutexGuard<UiCore>) {
        if self.cmdchar.is_none() {
            self.cmdchar = Some('e');
        } else {
            self.buf_op_arg = Some(BufOpArg::WordEnd);
        }
        let count = match self.count {
            Some(count) => count,
            None => 1,
        };
        let mut pos = (ui.cursor.x(), ui.cursor.y() + ui.buffer.offset() as u16);
        for _ in 0..count {
            pos = self.locate_nextw_ending(ui, pos.0, pos.1);
        }
        self.end_pos = Some(pos);
    }

    fn exec_e_cmd(&mut self, ui: &mut MutexGuard<UiCore>) -> io::Result<StateCallback> {
        let end_pos = self.end_pos;
        if end_pos.is_none() {
            return Ok(StateCallback::None);
        }
        let end_pos = end_pos.unwrap();
        self.move_to_line(ui, end_pos.1)?;
        ui.cursor.move_to_columu(end_pos.0)?;
        return Ok(StateCallback::Reset);
    }

    fn on_f_clicked(&mut self) {
        if self.cmdchar.is_none() {
            self.cmdchar = Some('f');
        }
    }

    fn exec_f_cmd(&mut self, ui: &mut MutexGuard<UiCore>) -> io::Result<StateCallback> {
        if self.cmdbuf.len() < 2 {
            return Ok(StateCallback::None);
        }
        let to_find = self.cmdbuf.last().unwrap();
        let old_x = ui.cursor.x();
        let old_y = ui.cursor.y();
        let line =
            String::from_utf8_lossy(&ui.buffer.get_line(old_y)[old_x as usize..]).to_string();
        let pos = line.find(*to_find as char);
        if pos.is_none() {
            return Ok(StateCallback::None);
        }
        ui.cursor
            .move_to_columu((old_x + pos.unwrap() as u16) as u16)?;
        return Ok(StateCallback::Reset);
    }

    #[allow(non_snake_case)]
    fn on_F_clicked(&mut self) {
        if self.cmdchar.is_none() {
            self.cmdchar = Some('F');
        }
    }

    #[allow(non_snake_case)]
    fn exec_F_cmd(&mut self, ui: &mut MutexGuard<UiCore>) -> io::Result<StateCallback> {
        if self.cmdbuf.len() < 2 {
            return Ok(StateCallback::None);
        }
        let to_find = self.cmdbuf.last().unwrap();
        let old_x = ui.cursor.x();
        let old_y = ui.cursor.y();
        let line =
            String::from_utf8_lossy(&ui.buffer.get_line(old_y)[..old_x as usize]).to_string();
        let pos = line.rfind(*to_find as char);
        if pos.is_none() {
            return Ok(StateCallback::None);
        }
        ui.cursor.move_to_columu(pos.unwrap() as u16)?;
        return Ok(StateCallback::Reset);
    }

    fn on_x_clicked(&mut self) {
        if self.cmdchar.is_none() {
            self.cmdchar = Some('x');
        }
    }

    fn exec_x_cmd(&mut self, ui: &mut MutexGuard<UiCore>) -> io::Result<StateCallback> {
        let y = ui.cursor.y();
        let x = ui.cursor.x();
        if x < ui.buffer.get_linesize(y) - 1 {
            ui.buffer.remove_char(x, y);
            ui.render_content(y, 1)?;
        }
        return Ok(StateCallback::Reset);
    }

    #[allow(non_snake_case)]
    fn on_H_clicked(&mut self) {
        if self.cmdchar.is_none() {
            self.cmdchar = Some('H');
        }
    }

    #[allow(non_snake_case)]
    fn exec_H_cmd(&mut self, ui: &mut MutexGuard<UiCore>) -> io::Result<StateCallback> {
        self.move_to_nlines_of_screen(ui, 0)?;
        return Ok(StateCallback::Reset);
    }
    #[allow(non_snake_case)]
    fn on_M_clicked(&mut self) {
        if self.cmdchar.is_none() {
            self.cmdchar = Some('M');
        }
    }

    #[allow(non_snake_case)]
    fn exec_M_cmd(&mut self, ui: &mut MutexGuard<UiCore>) -> io::Result<StateCallback> {
        let win_size = CONTENT_WINSIZE.read().unwrap().rows as usize;
        self.move_to_nlines_of_screen(ui, win_size / 2)?;
        return Ok(StateCallback::Reset);
    }

    fn on_y_clicked(&mut self) {
        match self.cmdchar {
            Some('y') => {
                self.buf_op_arg = Some(BufOpArg::Line);
            }
            None => {
                self.cmdchar = Some('y');
            }
            _ => {}
        }
    }

    fn exec_y_cmd(&mut self, ui: &mut MutexGuard<UiCore>) -> io::Result<StateCallback> {
        match self.buf_op_arg {
            Some(BufOpArg::Line) => {
                let y = ui.cursor.y();
                let line = ui.buffer.get_line(y);
                ui.register.copy(vec![line]);
                return Ok(StateCallback::Reset);
            }
            Some(BufOpArg::Word) => {
                let curr_pos = (ui.cursor.x(), ui.cursor.y() + ui.buffer.offset() as u16);
                let next_pos = self.locate_next_word(ui, curr_pos);
                let text_copy = ui.buffer.get_range(curr_pos, next_pos);
                ui.register.copy(text_copy);
                return Ok(StateCallback::Reset);
            }
            Some(BufOpArg::Around) => {
                let pat = self.cmdbuf.last().unwrap();
                if PAIRING.contains_key(&(*pat as char)) {
                    find_pair(ui, *pat).map(|paired_pos| {
                        let content = ui.buffer.get_range(paired_pos.start, paired_pos.end);
                        ui.register.copy(content);
                    });
                    return Ok(StateCallback::Reset);
                }
            }
            Some(BufOpArg::Inside) => {
                let pat = self.cmdbuf.last().unwrap();
                if PAIRING.contains_key(&(*pat as char)) {
                    find_pair(ui, *pat).map(|paired_pos| {
                        let start = (paired_pos.start.0 + 1, paired_pos.start.1);
                        let end = (paired_pos.end.0 - 1, paired_pos.end.1);
                        let content = ui.buffer.get_range(start, end);
                        ui.register.copy(content);
                    });
                    return Ok(StateCallback::Reset);
                }
            }
            _ => {}
        }
        return Ok(StateCallback::None);
    }

    fn on_p_clicked(&mut self) {
        if self.cmdchar.is_none() {
            self.cmdchar = Some('p');
        }
    }

    fn exec_p_cmd(&mut self, ui: &mut MutexGuard<UiCore>) -> io::Result<StateCallback> {
        let count = match self.count {
            Some(count) => count,
            None => 1,
        };
        for _ in 0..count {
            self.paste(ui)?;
        }
        return Ok(StateCallback::Reset);
    }
}

impl StateMachine for NormalState {
    fn reset(&mut self) {
        self.cmdchar = None;
        self.count = None;
        self.count0 = false;
        self.start_pos = None;
        self.end_pos = None;
        self.cmdbuf.clear();
        self.buf_op_arg = None;
    }
    fn handle(&mut self, ui: &mut MutexGuard<UiCore>) -> io::Result<WarpUiCallBackType> {
        if self.cmdchar.is_none() {
            return Ok(WarpUiCallBackType::None);
        }
        let state_callback = match self.cmdchar.unwrap() {
            'h' => self.exec_h_cmd(ui),
            'j' => self.exec_j_cmd(ui),
            'k' => self.exec_k_cmd(ui),
            'l' => self.exec_l_cmd(ui),
            'i' => self.exec_i_cmd(ui),
            '0' => self.exec_0_cmd(ui),
            'd' => self.exec_d_cmd(ui),
            'w' => self.exec_w_cmd(ui),
            'g' => self.exec_g_cmd(ui),
            'G' => self.exec_G_cmd(ui),
            'b' => self.exec_b_cmd(ui),
            '$' => self.exec_dollar_cmd(ui),
            'e' => self.exec_e_cmd(ui),
            'f' => self.exec_f_cmd(ui),
            'F' => self.exec_F_cmd(ui),
            'x' => self.exec_x_cmd(ui),
            'y' => self.exec_y_cmd(ui),
            'p' => self.exec_p_cmd(ui),
            'o' => self.exec_o_cmd(ui),
            'O' => self.exec_O_cmd(ui),
            'a' => self.exec_a_cmd(ui),
            'A' => self.exec_A_cmd(ui),
            'I' => self.exec_I_cmd(ui),
            'H' => self.exec_H_cmd(ui),
            'M' => self.exec_M_cmd(ui),
            _ => Ok(StateCallback::None),
        };
        return match state_callback {
            Ok(StateCallback::None) => Ok(WarpUiCallBackType::None),
            Ok(StateCallback::Reset) => {
                self.reset();
                Ok(WarpUiCallBackType::None)
            }
            Ok(StateCallback::Exit(mode)) => self.exit(WarpUiCallBackType::ChangMode(mode)),
            Err(e) => Err(e),
        };
    }

    fn exit(&mut self, callback: WarpUiCallBackType) -> io::Result<WarpUiCallBackType> {
        self.reset();
        Ok(callback)
    }
}
