use crossterm::event::KeyCode;
use held_core::utils::position::Position;
use held_core::utils::range::Range;
use unicode_segmentation::UnicodeSegmentation;

use crate::application::mode::motion;
use crate::application::Application;
use crate::errors::*;

pub fn count_cmd(app: &mut Application) -> Result<()> {
    if let Some(key) = app.monitor.last_key {
        if let KeyCode::Char(ch) = key.code {
            if let Some(digit) = ch.to_digit(10) {
                let count = &mut app.cmd_counter;
                *count *= 10;
                *count += digit as usize;
            }
        }
    }
    Ok(())
}

// 可以指定执行次数的命令，数字必须先于命令字符；而命令字符可以在配置文件指定

pub fn move_down_n(app: &mut Application) -> Result<()> {
    let count = app.cmd_counter.max(1);
    if let Some(buffer) = &mut app.workspace.current_buffer {
        for _ in 0..count.min(buffer.line_count() - buffer.cursor.line) {
            buffer.cursor.move_down();
        }
        app.monitor.scroll_to_cursor(buffer).unwrap();
        reset(app)?;
    }
    Ok(())
}

pub fn move_up_n(app: &mut Application) -> Result<()> {
    let count = app.cmd_counter.max(1);
    if let Some(buffer) = &mut app.workspace.current_buffer {
        for _ in 0..count.min(buffer.cursor.line) {
            buffer.cursor.move_up();
        }
        app.monitor.scroll_to_cursor(buffer).unwrap();
        reset(app)?;
    }
    Ok(())
}

pub fn move_to_target_line(app: &mut Application) -> Result<()> {
    if let Some(buffer) = &mut app.workspace.current_buffer {
        let count = app.cmd_counter;
        if count > 0 {
            let target_line = count.min(buffer.line_count());
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
        reset(app)?;
    }
    Ok(())
}

pub fn move_left_n(app: &mut Application) -> Result<()> {
    let mut count = app.cmd_counter.max(1);
    if let Some(buffer) = &mut app.workspace.current_buffer {
        let offset = buffer.cursor.offset;
        count = count.min(offset);
        for _ in 0..count {
            buffer.cursor.move_left();
        }
        app.monitor.scroll_to_cursor(buffer)?;
        reset(app)?;
    }
    Ok(())
}

pub fn move_right_n(app: &mut Application) -> Result<()> {
    let mut count = app.cmd_counter.max(1);
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
        reset(app)?;
    }
    Ok(())
}

pub fn reset(app: &mut Application) -> Result<()> {
    app.cmd_counter = 0;
    Ok(())
}

pub fn move_to_next_words(app: &mut Application) -> Result<()> {
    if let Some(buffer) = &mut app.workspace.current_buffer {
        let current_pos = buffer.cursor.position;
        // 从当前位置向后搜索
        let search_range = if let Some(str) = buffer.read_rest(&current_pos) {
            str
        } else {
            return Ok(());
        };
        let count = app.cmd_counter;
        let next_words_pos =
            motion::locate_next_words_begin(count.max(1), &search_range, &current_pos);
        buffer.cursor.move_to(next_words_pos);
        app.monitor.scroll_to_cursor(buffer)?;
        reset(app)?;
    }
    Ok(())
}

pub fn move_to_prev_words(app: &mut Application) -> Result<()> {
    if let Some(buffer) = &mut app.workspace.current_buffer {
        let current_pos = buffer.cursor.position;
        // + Distance {
        //     lines: 0,
        //     offset: 1,
        // }; // 由于是左闭右开区间，所以需要向后移动一个字符
        // 从当前位置向前搜索
        let search_range =
            if let Some(str) = buffer.read(&Range::new(Position::new(0, 0), current_pos)) {
                str
            } else {
                return Ok(());
            };
        let count = app.cmd_counter;
        let prev_words_pos =
            motion::locate_previous_words(count.max(1), &search_range, &current_pos);
        buffer.cursor.move_to(prev_words_pos);
        app.monitor.scroll_to_cursor(buffer)?;
        reset(app)?;
    }
    Ok(())
}

pub fn move_to_next_words_end(app: &mut Application) -> Result<()> {
    if let Some(buffer) = &mut app.workspace.current_buffer {
        let current_pos = buffer.cursor.position;
        // 从当前位置向后搜索
        let search_range = if let Some(str) = buffer.read_rest(&current_pos) {
            str
        } else {
            return Ok(());
        };
        let count = app.cmd_counter;
        let next_words_pos =
            motion::locate_next_words_end(count.max(1), &search_range, &current_pos);
        buffer.cursor.move_to(next_words_pos);
        app.monitor.scroll_to_cursor(buffer)?;
        reset(app)?;
    }
    Ok(())
}
