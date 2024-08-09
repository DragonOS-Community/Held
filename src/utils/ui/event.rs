use std::{io, sync::MutexGuard};

use crate::utils::{
    buffer::LineState, cursor::CursorCrtl, style::StyleManager, terminal::TermManager,
};

use super::{
    mode::mode::ModeType,
    uicore::{UiCore, APP_INFO, CONTENT_WINSIZE, DEF_STYLE, UI_CMD_HEIGHT},
};

pub const TAB_STR: &'static str = "        ";

pub trait KeyEventCallback {
    fn enter(&self, ui: &mut MutexGuard<UiCore>) -> io::Result<WarpUiCallBackType>;
    fn tab(&self, ui: &mut MutexGuard<UiCore>) -> io::Result<WarpUiCallBackType>;
    fn backspace(&self, ui: &mut MutexGuard<UiCore>) -> io::Result<WarpUiCallBackType> {
        if ui.cursor.x() == 0 {
            let y = ui.cursor.y();
            let (merged, linelen) = ui.buffer.merge_line(y);
            if merged {
                // 需要向上翻页
                if ui.cursor.y() == 0 {
                    ui.scroll_down(1)?;
                    ui.cursor.move_to_nextline(1)?;
                }
                // 重新渲染
                ui.cursor.move_up(1)?;

                let y = ui.cursor.y();
                let ret =
                    ui.render_content(y, (CONTENT_WINSIZE.read().unwrap().rows - y + 1) as usize)?;

                // 清除之前显示行
                // 计算需要clear的行号
                let clear_y = if ui.cursor.y() == 0 { y + 1 } else { y };
                let row = clear_y + ret as u16;

                ui.cursor.move_to_row(row)?;

                DEF_STYLE.read().unwrap().set_content_style()?;

                ui.cursor.set_prefix_mode(false);
                StyleManager::reset_color()?;
                ui.cursor.move_to_columu(0)?;
                ui.cursor
                    .write(&TAB_STR[..CursorCrtl::PREFIX_COL as usize])?;
                ui.cursor.set_prefix_mode(true);

                ui.cursor.clear_current_line()?;

                ui.cursor.move_to_row(y)?;
                ui.cursor.move_to_columu(linelen as u16)?;
                ui.cursor.highlight(Some(clear_y))?;
                ui.set_edited();
                return Ok(WarpUiCallBackType::None);
            } else {
                return Ok(WarpUiCallBackType::None);
            }
        }

        let y = ui.cursor.y();
        let x = ui.cursor.x();

        let line = ui.buffer.get_line(y);
        if line.flags.contains(LineState::LOCKED) {
            APP_INFO.lock().unwrap().info = "Row is locked".to_string();
            return Ok(WarpUiCallBackType::None);
        }
        self.left(ui)?;

        ui.buffer.remove_char(x - 1, y);

        let line = ui.buffer.get_line(y);

        ui.cursor.write(format!(
            "{} ",
            String::from_utf8_lossy(&line.data[x as usize..])
        ))?;

        ui.cursor.highlight(None)?;

        ui.cursor.move_to_columu(x - 1)?;

        Ok(WarpUiCallBackType::None)
    }
    fn up(&self, ui: &mut MutexGuard<UiCore>) -> io::Result<WarpUiCallBackType> {
        if ui.cursor.y() == 0 {
            if ui.buffer.offset() == 0 {
                // 上面没有数据
                return Ok(WarpUiCallBackType::None);
            }
            // 向上滚动
            ui.scroll_down(1)?;

            let linesize = ui.buffer.get_linesize(ui.cursor.y());

            // 考虑\n
            if linesize - 1 < ui.cursor.x() {
                ui.cursor.move_to_columu(linesize - 1)?;
            }
            return Ok(WarpUiCallBackType::None);
        }
        let linesize = ui.buffer.get_linesize(ui.cursor.y() - 1);

        if linesize == 0 {
            return Ok(WarpUiCallBackType::None);
        }

        ui.cursor.move_up(1)?;

        // 考虑\n
        if linesize - 1 < ui.cursor.x() {
            ui.cursor.move_to_columu(linesize - 1)?;
        }

        let last_y = ui.cursor.y() + 1;
        ui.cursor.highlight(Some(last_y))?;

        Ok(WarpUiCallBackType::None)
    }
    fn down(&self, ui: &mut MutexGuard<UiCore>) -> io::Result<WarpUiCallBackType> {
        let size = *CONTENT_WINSIZE.read().unwrap();
        let mut linesize = ui.buffer.get_linesize(ui.cursor.y() + 1);

        if linesize == 0 {
            return Ok(WarpUiCallBackType::None);
        }

        if ui.cursor.y() == size.rows - UI_CMD_HEIGHT {
            // 向shang滚动
            ui.scroll_up(1)?;
            if linesize < ui.cursor.x() {
                ui.cursor.move_to_columu(linesize - 1)?;
            }
            return Ok(WarpUiCallBackType::None);
        }

        // \n
        linesize -= 1;

        ui.cursor.move_down(1)?;

        if linesize < ui.cursor.x() {
            ui.cursor.move_to_columu(linesize)?;
        }
        let last_y = ui.cursor.y() - 1;
        ui.cursor.highlight(Some(last_y))?;

        Ok(WarpUiCallBackType::None)
    }
    fn left(&self, ui: &mut MutexGuard<UiCore>) -> io::Result<WarpUiCallBackType> {
        ui.cursor.move_left(1)?;
        Ok(WarpUiCallBackType::None)
    }
    fn right(&self, ui: &mut MutexGuard<UiCore>) -> io::Result<WarpUiCallBackType> {
        ui.cursor.move_right(1)?;
        Ok(WarpUiCallBackType::None)
    }
    fn esc(&self, ui: &mut MutexGuard<UiCore>) -> io::Result<WarpUiCallBackType>;
    fn input_data(
        &self,
        ui: &mut MutexGuard<UiCore>,
        data: &[u8],
    ) -> io::Result<WarpUiCallBackType>;

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

    fn jump_to_next_word(&self, ui: &mut MutexGuard<UiCore>) -> io::Result<WarpUiCallBackType> {
        let x = ui.cursor.x();
        let y = ui.cursor.y();
        let pos = ui.buffer.search_nextw_begin(x, y);
        let linesize = ui.buffer.get_linesize(y);
        let abs_y = y + ui.buffer.offset() as u16;

        if pos < linesize as usize {
            // 如果下一个单词在当前行，则移动光标到该单词的起始位置
            ui.cursor.move_to_columu(pos as u16)?;
        } else if y as usize + ui.buffer.offset() < ui.buffer.line_count() - 1 {
            // 如果当前行不是最后一行，则移动到下一行的单词起始位置
            let next_word_pos = ui.buffer.search_nextw_begin(0, y + 1) as u16;
            let next_linesize = ui.buffer.get_linesize_abs(abs_y + 1);
            self.down(ui)?;
            ui.cursor
                .move_to_columu(next_word_pos.min(next_linesize - 1))?;
            ui.cursor.highlight(Some(y))?;
        } else {
            // 如果当前行是最后一行，则移动到当前行的末尾
            ui.cursor.move_to_columu(linesize as u16 - 1)?;
        }
        return Ok(WarpUiCallBackType::None);
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

    fn search_pair(&self, ui: &mut MutexGuard<UiCore>, pat: u8) -> Option<(u16, u16)> {
        if self.is_left_bracket(pat) {
            return self.search_pairs_by_left_pat(ui, pat);
        } else if self.is_right_bracket(pat) {
            return self.search_pairs_by_right_pat(ui, pat);
        } else if self.is_paired(pat) {
            return self.search_paired_quotes(ui, pat);
        }
        return None;
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

    fn is_paired(&self, ch: u8) -> bool {
        match ch {
            b'\'' | b'\"' => true,
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

    /// 查找配对的引号
    /// 返回引号的位置，如果未找到则返回None
    fn search_paired_quotes(&self, ui: &mut MutexGuard<UiCore>, pat: u8) -> Option<(u16, u16)> {
        let x = ui.cursor.x();
        let y = ui.cursor.y();
        let line = ui.buffer.get_line(y).data;
        let linesize = ui.buffer.get_linesize(y);
        let mut left = x as i32;
        let mut right = x as i32;
        // 尝试往前找引号
        while left >= 0 && pat != line[left as usize] {
            left -= 1;
        }
        // 未找到引号，尝试往后找引号
        if left < 0 {
            left = x as i32;
            while left <= right && right < linesize as i32 {
                if pat != line[left as usize] {
                    left += 1;
                    right += 1;
                    continue;
                }
                if pat == line[right as usize] && left < right {
                    return Some((left as u16, right as u16));
                }
                right += 1;
            }
            return None;
        } else {
            // 找到引号，尝试往后找引号
            right = left + 1;
            while right < linesize as i32 {
                if pat == line[right as usize] {
                    return Some((left as u16, right as u16));
                }
                right += 1;
            }
            // 未找到引号，尝试继续往前找引号
            right = left;
            left -= 1;
            while left < right && left >= 0 {
                if pat != line[right as usize] {
                    right -= 1;
                    left -= 1;
                    continue;
                }
                if pat == line[left as usize] && left < right {
                    return Some((left as u16, right as u16));
                }
                left -= 1;
            }
            return None;
        }
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

    fn locate_nextw_ending(&self, ui: &mut MutexGuard<UiCore>, x: u16, y: u16) -> (u16, u16) {
        let linesize = ui.buffer.get_linesize(y) as usize;

        // y的绝对位置
        let abs_y = ui.buffer.offset() as u16 + y;
        // 如果光标已经在当前行的末尾或最后一个字符(x + 2)，则尝试移动到下一行的末尾或单词末尾
        if x as usize + 2 >= linesize {
            if abs_y < ui.buffer.line_count() as u16 - 1 {
                let next_end_pos = ui.buffer.search_nextw_end(0, y + 1) as u16;
                return (next_end_pos, abs_y + 1);
            } else {
                // 如果已经是最后一行，则保持光标在当前行的末尾
                let x = if linesize > 0 { linesize - 1 } else { 0 };
                return (x as u16, abs_y);
            }
        }

        let next_end_pos = ui.buffer.search_nextw_end(x, y) as u16;
        // 如果下一个单词的末尾在当前行，则移动光标到该单词的末尾
        return (next_end_pos.min(linesize as u16 - 1), abs_y);
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

    fn locate_prevw_begin(&self, ui: &mut MutexGuard<UiCore>, x: u16, abs_y: u16) -> (u16, u16) {
        // 如果光标已在行首，则尝试移动到上一行的单词首字母
        if x == 0 {
            if abs_y == 0 {
                return (0, 0);
            }
            let last_y = abs_y - 1;
            let end_of_prev_line = ui.buffer.get_linesize_abs(last_y) - 1;
            let prev_word_pos = ui.buffer.search_prevw_begin_abs(end_of_prev_line, last_y);
            return (prev_word_pos as u16, last_y);
        }

        let prev_word_pos = ui.buffer.search_prevw_begin_abs(x, abs_y);

        return (prev_word_pos as u16, abs_y);
    }

    fn move_to_line(&self, ui: &mut MutexGuard<UiCore>, line: u16) -> io::Result<()> {
        let x = ui.cursor.x();
        let y = ui.cursor.y();
        let new_y = ui.buffer.goto_line(line as usize);
        let new_x = x.min(ui.buffer.get_linesize(new_y)) as u16;
        ui.cursor.move_to(new_x, new_y)?;
        ui.render_content(0, CONTENT_WINSIZE.read().unwrap().rows as usize)?;
        ui.cursor.highlight(Some(y))?;
        return Ok(());
    }
}

#[derive(Debug, PartialEq)]
pub enum WarpUiCallBackType {
    ChangMode(ModeType),
    Exit(bool),
    None,
}
