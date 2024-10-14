use crossterm::event::{KeyCode, KeyEvent};

use crate::application::Application;
use crate::errors::*;
use crate::util::position::Position;

pub fn insert_char(app: &mut Application) -> Result<()> {
    if let Some(key) = app.monitor.last_key {
        if let KeyCode::Char(c) = key.code {
            app.workspace.current_buffer.as_mut().unwrap().insert(c);
        }
    }
    Ok(())
}

pub fn backspace(app: &mut Application) -> Result<()> {
    if let Some(ref mut buffer) = app.workspace.current_buffer {
        // 在第一行第一列时不执行删除操作
        if buffer.cursor.position != Position::new(0, 0) {
            buffer.cursor.move_left();
            buffer.delete();
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
