use crossterm::event::KeyCode;

use crate::errors::*;
use crate::{application::Application, util::position::Position};

use super::cursor;

pub fn insert_char(app: &mut Application) -> Result<()> {
    if let Some(key) = app.monitor.last_key {
        if let KeyCode::Char(c) = key.code {
            app.workspace.current_buffer.as_mut().unwrap().insert(c);
            cursor::move_right(app)?;
        }
    }
    Ok(())
}

pub fn insert_char_on_replace(app: &mut Application) -> Result<()> {
    if let Some(key) = app.monitor.last_key {
        if let KeyCode::Char(c) = key.code {
            app.workspace
                .current_buffer
                .as_mut()
                .unwrap()
                .replace_on_cursor(c.to_string());
            cursor::move_right(app)?;
        }
    }
    Ok(())
}

pub fn new_line(app: &mut Application) -> Result<()> {
    if let Some(ref mut buffer) = app.workspace.current_buffer {
        buffer.insert('\n');
    }
    Ok(())
}

pub fn insert_tab(app: &mut Application) -> Result<()> {
    if let Some(buffer) = app.workspace.current_buffer.as_mut() {
        let tab_len = app.perferences.borrow().tab_width();
        let width = tab_len - (buffer.cursor.offset) % tab_len;
        if app.perferences.borrow().soft_tab() {
            let tab_str = " ".repeat(width);
            buffer.insert(tab_str);
            buffer.cursor.move_to(Position {
                line: buffer.cursor.line,
                offset: buffer.cursor.offset + width,
            });
        } else {
            buffer.insert("\t");
            buffer.cursor.move_to(Position {
                line: buffer.cursor.line,
                offset: buffer.cursor.offset + width,
            });
        }
    }
    Ok(())
}
