use std::{io, sync::MutexGuard, usize};

use crate::utils::{
    terminal::TermManager,
    ui::{
        event::{KeyEventCallback, WarpUiCallBackType},
        uicore::{UiCore, CONTENT_WINSIZE},
    },
};

pub trait CommonOp: KeyEventCallback {
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

    fn locate_next_word(&self, ui: &mut MutexGuard<UiCore>, x: u16, y: u16) -> (u16, u16) {
        let linesize = ui.buffer.get_linesize(y) as usize;
        let abs_y = ui.buffer.offset() as u16 + y;
        let next_word_pos = ui.buffer.search_nextw_begin(x, y);

        if next_word_pos < linesize {
            return (next_word_pos as u16, abs_y);
        }

        if abs_y < ui.buffer.line_count() as u16 - 1 {
            let next_word_pos = ui.buffer.search_nextw_begin(0, y + 1) as u16;
            return (next_word_pos, abs_y + 1);
        }

        return (linesize as u16 - 1, abs_y);
    }

    fn paste(&self, ui: &mut MutexGuard<UiCore>) -> io::Result<()> {
        let x = ui.cursor.x();
        let y = ui.cursor.y();
        if ui.register.text.is_empty() {
            return Ok(());
        }
        if ui.register.is_single_line() {
            // 单行
            ui.buffer.insert_line(y.into(), &ui.register.text[0]);
        } else if ui.register.is_muti_line() {
            // 多行
            for (idx, line) in ui.register.text.iter().enumerate() {
                for (idy, c) in line.data.iter().enumerate() {
                    ui.buffer.insert_char(*c, x + idy as u16, y + idx as u16);
                }
                ui.buffer
                    .input_enter(line.data.len() as u16, y + idx as u16);
            }
        } else {
            // 单词
            let line = &ui.register.text[0];
            for (idx, c) in line.data.iter().enumerate() {
                ui.buffer.insert_char(*c, x + idx as u16, y);
            }
        }
        let rest_line = ui.buffer.line_count() - y as usize - ui.buffer.offset();
        ui.render_content(y, rest_line)?;
        Ok(())
    }
}
