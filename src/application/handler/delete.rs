use held_core::utils::position::Position;
use held_core::utils::range::Range;

use crate::application::mode::motion::locate_next_words_begin;
use crate::application::mode::CMD_COUNTER;
use crate::application::Application;
use crate::errors::*;

use super::normal::{self};

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
    let del_range = Range::new(*current_pos, next_words_pos);
    buf.delete_range(del_range);
    normal::reset(app)?;
    Ok(())
}

pub fn delete_lines(app: &mut Application) -> Result<()> {
    let count = CMD_COUNTER.read().unwrap().clone();
    let buf = app.workspace.current_buffer.as_mut().unwrap();
    let start_pos = Position::new(buf.cursor.line, 0);
    let end_pos = Position::new(
        start_pos.line + count.min(buf.line_count() - start_pos.line).max(1),
        0,
    );
    buf.delete_range(Range::new(start_pos, end_pos));
    normal::reset(app)?;
    Ok(())
}
