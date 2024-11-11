use crate::application::mode::{ModeData, ModeKey};
use crate::application::Application;
use crate::errors::*;

pub fn to_normal_mode(app: &mut Application) -> Result<()> {
    if let ModeData::Workspace(ref mode) = app.mode {
        app.workspace.select_buffer(mode.prev_buffer_id);
    }
    app.switch_mode(ModeKey::Normal);
    Ok(())
}

pub fn move_down(app: &mut Application) -> Result<()> {
    if let ModeData::Workspace(ref mut mode) = app.mode {
        mode.move_down();
    }
    Ok(())
}

pub fn move_up(app: &mut Application) -> Result<()> {
    if let ModeData::Workspace(ref mut mode) = app.mode {
        mode.move_up();
    }
    Ok(())
}

pub fn enter(app: &mut Application) -> Result<()> {
    if let ModeData::Workspace(ref mut mode) = app.mode {
        if mode.open(&mut app.workspace, &mut app.monitor)? {
            to_normal_mode(app)?;
        }
    }
    Ok(())
}

pub fn save_file(_app: &mut Application) -> Result<()> {
    todo!();
}
