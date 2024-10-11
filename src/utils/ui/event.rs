use std::{io, sync::MutexGuard};

use crate::utils::{buffer::LineState, cursor::CursorCrtl, style::StyleManager};

use super::{
    mode::mode::ModeType,
    uicore::{UiCore, APP_INTERNAL_INFOMATION, CONTENT_WINSIZE, DEF_STYLE, UI_CMD_HEIGHT},
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
            APP_INTERNAL_INFOMATION.lock().unwrap().info = "Row is locked".to_string();
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
}

#[derive(Debug, PartialEq)]
pub enum WarpUiCallBackType {
    ChangMode(ModeType),
    Exit(bool),
    None,
}
