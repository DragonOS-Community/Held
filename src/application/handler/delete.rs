use std::ops::{AddAssign, MulAssign};

use crossterm::event::KeyCode;
use held_core::utils::range::Range;

use crate::application::mode::motion::locate_next_words_begin;
use crate::application::mode::CMD_COUNTER;
use crate::application::Application;
use crate::errors::*;

pub fn delete_words(app: &mut Application) -> Result<()> {
    let count = CMD_COUNTER.read().unwrap().clone();
    let buf = app.workspace.current_buffer.as_mut().unwrap();
    let current_pos = &buf.cursor.position;
    let search_range = if let Some(str) = buf.read_rest(&current_pos) {
        str
    } else {
        return Ok(());
    };
    let next_words_pos = locate_next_words_begin(count, &search_range, current_pos);
    if let Some(next_words_pos) = next_words_pos {
        let del_range = Range::new(*current_pos, next_words_pos);
        buf.delete_range(del_range);
    }
    Ok(())
}

pub fn handle_num(app: &mut Application) -> Result<()> {
    if let Some(key) = app.monitor.last_key {
        if let KeyCode::Char(ch) = key.code {
            let mut count = CMD_COUNTER.write().unwrap();
            count.mul_assign(10);
            count.add_assign(ch.to_digit(10).unwrap() as usize);
        }
    }
    Ok(())
}
