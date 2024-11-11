use crate::application::Application;
use crate::errors::*;

pub fn scroll_to_cursor(app: &mut Application) -> Result<()> {
    if let Some(ref buffer) = app.workspace.current_buffer {
        app.monitor.scroll_to_cursor(buffer)?;
    }
    Ok(())
}

pub fn scroll_to_center(app: &mut Application) -> Result<()> {
    if let Some(ref buffer) = app.workspace.current_buffer {
        app.monitor.scroll_to_center(buffer)?;
    }
    Ok(())
}

pub fn scroll_to_first_line(app: &mut Application) -> Result<()> {
    if let Some(ref mut buffer) = app.workspace.current_buffer {
        buffer.cursor.move_to_first_line();
        app.monitor.scroll_to_cursor(buffer)?;
    }
    Ok(())
}

pub fn scroll_to_last_line(app: &mut Application) -> Result<()> {
    if let Some(ref mut buffer) = app.workspace.current_buffer {
        buffer.cursor.move_to_last_line();
        app.monitor.scroll_to_cursor(buffer)?;
    }
    Ok(())
}
