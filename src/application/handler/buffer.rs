use crossterm::event::{KeyCode, KeyEvent};

use crate::application::Application;
use crate::errors::*;

pub fn insert_char(app: &mut Application) -> Result<()> {
    if let Some(key) = app.monitor.last_key {
        match key.code {
            KeyCode::Char(c) => {
                app.workspace.current_buffer.as_mut().unwrap().insert(c);
            }
            KeyCode::Enter => {
                app.workspace.current_buffer.as_mut().unwrap().insert('\n');
            }
            KeyCode::Tab => {
                app.workspace.current_buffer.as_mut().unwrap().insert('\t');
            }
            _ => {}
        }
    }
    Ok(())
}

pub fn delete_char(app: &mut Application) -> Result<()> {
    if let Some(ref mut buffer) = app.workspace.current_buffer {
        buffer.delete();
    }
    Ok(())
}
