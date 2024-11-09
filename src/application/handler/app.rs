use crate::application::mode::command::CommandData;
use crate::application::mode::{ModeData, ModeKey};
use crate::application::Application;
use crate::errors::*;

pub fn exit(app: &mut Application) -> Result<()> {
    app.switch_mode(ModeKey::Exit);
    Ok(())
}

pub fn exit_with_check(app: &mut Application) -> Result<()> {
    if let Some(ref buf) = app.workspace.current_buffer {
        if buf.modified() {
            // 输出提示(todo)
            if let ModeData::Command(ref mut command_data) = app.mode {
                command_data.reset();
            }
            app.switch_mode(ModeKey::Normal);
        } else {
            app.switch_mode(ModeKey::Exit);
        }
    }
    Ok(())
}

pub fn to_insert_mode(app: &mut Application) -> Result<()> {
    app.switch_mode(ModeKey::Insert);
    Ok(())
}

pub fn to_normal_mode(app: &mut Application) -> Result<()> {
    app.switch_mode(ModeKey::Normal);
    Ok(())
}

pub fn to_workspace_mode(app: &mut Application) -> Result<()> {
    app.switch_mode(ModeKey::Workspace);
    Ok(())
}

pub fn to_command_mode(app: &mut Application) -> Result<()> {
    app.switch_mode(ModeKey::Command);
    OK(())
}

pub fn to_search_mode(app: &mut Application) -> Result<()> {
    app.switch_mode(ModeKey::Search);
    OK(())
}

pub fn to_delete_mode(app: &mut Application) -> Result<()> {
    app.switch_mode(ModeKey::Delete);
    Ok(())
}
