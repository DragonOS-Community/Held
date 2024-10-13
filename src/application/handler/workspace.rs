use crate::application::Application;
use crate::errors::*;

pub fn save_file(app: &mut Application) -> Result<()> {
    if let Some(ref mut buffer) = app.workspace.current_buffer {
        buffer.save()?;
    }
    Ok(())
}

pub fn undo(app: &mut Application) -> Result<()> {
    if let Some(ref mut buffer) = app.workspace.current_buffer {
        buffer.undo();
    }
    Ok(())
}
