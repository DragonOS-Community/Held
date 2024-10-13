use crossterm::event::{KeyCode, KeyEvent};

use crate::application::Application;
use crate::errors::*;

pub fn insert_char(app: &mut Application) -> Result<()> {
    if let Some(key) = app.monitor.last_key {
        if let KeyCode::Char(c) = key.code {
            app.workspace.current_buffer.as_mut().unwrap().insert(c);
        }
    }
    Ok(())
}
