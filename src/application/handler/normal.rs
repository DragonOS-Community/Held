use unicode_segmentation::UnicodeSegmentation;

use crate::application::mode::normal::*;
use crate::application::mode::{ModeData, ModeState};
use crate::application::Application;
use crate::errors::*;
use crate::util::position::Position;
use crate::util::range::Range;

pub fn transition(app: &mut Application) -> Result<()> {
    if let ModeData::Normal(normal_state) = &mut app.mode {
        if let Some(key) = app.monitor.last_key {
            normal_state.transition(&key)?;
        }
    }
    Ok(())
}

// 可以指定执行次数的命令，数字必须先于命令字符；而命令字符可以在配置文件指定

pub fn move_down_n(app: &mut Application) -> Result<()> {
    if let ModeData::Normal(normal_state) = &mut app.mode {
        let count = normal_state.count.max(1);
        if let Some(buffer) = &mut app.workspace.current_buffer {
            for _ in 0..count.min(buffer.line_count() - buffer.cursor.line) {
                buffer.cursor.move_down();
            }
            app.monitor.scroll_to_cursor(buffer).unwrap();
            normal_state.reset();
        }
    }
    Ok(())
}

pub fn move_up_n(app: &mut Application) -> Result<()> {
    if let ModeData::Normal(normal_state) = &mut app.mode {
        let count = normal_state.count.max(1);
        if let Some(buffer) = &mut app.workspace.current_buffer {
            for _ in 0..count.min(buffer.cursor.line) {
                buffer.cursor.move_up();
            }
            app.monitor.scroll_to_cursor(buffer).unwrap();
            normal_state.reset();
        }
    }
    Ok(())
}

pub fn delete_some(app: &mut Application) -> Result<()> {
    if let ModeData::Normal(normal_state) = &mut app.mode {
        let count = normal_state.count.max(1);
        if let Some(buffer) = &mut app.workspace.current_buffer {
            match normal_state.buf_op_arg {
                Some(BufOpArg::Line) => {
                    let line = buffer.cursor.line;
                    buffer.delete_range(Range::new(
                        Position::new(line, 0),
                        Position::new((line + count).min(buffer.line_count()), 0),
                    ));
                    normal_state.reset();
                }
                Some(BufOpArg::Word) => {
                    todo!()
                }
                Some(BufOpArg::WordEnd) => {
                    todo!()
                }
                Some(BufOpArg::WordBegin) => {
                    todo!()
                }
                Some(BufOpArg::Block) => {
                    todo!()
                }
                _ => {}
            }
        }
    }
    Ok(())
}

pub fn move_to_target_line(app: &mut Application) -> Result<()> {
    if let ModeData::Normal(normal_state) = &mut app.mode {
        if let Some(buffer) = &mut app.workspace.current_buffer {
            if normal_state.count > 0 {
                let target_line = normal_state.count.min(buffer.line_count());
                let offset = buffer.cursor.offset;
                if !buffer
                    .cursor
                    .move_to(Position::new(target_line - 1, offset))
                {
                    let target_offset = buffer
                        .data()
                        .lines()
                        .nth(target_line - 1)
                        .unwrap()
                        .graphemes(true)
                        .count();
                    buffer
                        .cursor
                        .move_to(Position::new(target_line - 1, target_offset));
                }
            }
            app.monitor.scroll_to_cursor(buffer).unwrap();
            normal_state.reset();
        }
    }
    Ok(())
}

pub fn move_left_n(app: &mut Application) -> Result<()> {
    if let ModeData::Normal(normal_state) = &mut app.mode {
        let mut count = normal_state.count.max(1);
        if let Some(buffer) = &mut app.workspace.current_buffer {
            let offset = buffer.cursor.offset;
            count = count.min(offset);
            for _ in 0..count {
                buffer.cursor.move_left();
            }
            app.monitor.scroll_to_cursor(buffer)?;
            normal_state.reset();
        }
    }
    Ok(())
}

pub fn move_right_n(app: &mut Application) -> Result<()> {
    if let ModeData::Normal(normal_state) = &mut app.mode {
        let mut count = normal_state.count.max(1);
        if let Some(buffer) = &mut app.workspace.current_buffer {
            let max_offset = buffer
                .data()
                .lines()
                .nth(buffer.cursor.line)
                .unwrap()
                .graphemes(true)
                .count();
            let offset = buffer.cursor.offset;
            count = count.min(max_offset - offset);
            for _ in 0..count {
                buffer.cursor.move_right();
            }
            app.monitor.scroll_to_cursor(buffer)?;
            normal_state.reset();
        }
    }
    Ok(())
}

pub fn reset(app: &mut Application) -> Result<()> {
    if let ModeData::Normal(normal_state) = &mut app.mode {
        normal_state.reset();
    }
    Ok(())
}
