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
