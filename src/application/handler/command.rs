use std::collections::HashMap;

use crossterm::event::KeyCode;
use smallvec::SmallVec;

use crate::application::handler::{app, workspace};
use crate::application::mode::{ModeData, ModeKey};
use crate::application::Application;
use crate::errors::*;

use lazy_static::lazy_static;

lazy_static! {
    static ref COMMAND: HashMap<String, SmallVec<[fn(&mut Application) -> Result<()>; 4]>> = {
        let mut cmd_map =
            HashMap::<String, SmallVec<[fn(&mut Application) -> Result<()>; 4]>>::new();

        cmd_map.insert(
            "q".to_string(),
            SmallVec::from_vec(vec![
                app::exit_with_check as fn(&mut Application) -> Result<()>,
            ]),
        );
        cmd_map.insert(
            "q!".to_string(),
            SmallVec::from_vec(vec![app::exit as fn(&mut Application) -> Result<()>]),
        );
        cmd_map.insert(
            "w".to_string(),
            SmallVec::from_vec(vec![
                workspace::save_file as fn(&mut Application) -> Result<()>,
            ]),
        );
        cmd_map.insert(
            "wq".to_string(),
            SmallVec::from_vec(vec![
                workspace::save_file as fn(&mut Application) -> Result<()>,
                app::exit as fn(&mut Application) -> Result<()>,
            ]),
        );
        cmd_map
    };
}

pub fn commit_and_execute(app: &mut Application) -> Result<()> {
    let cmd = match app.mode {
        ModeData::Command(ref mut command_data) => command_data.input.clone(),
        _ => String::new(),
    };
    // 匹配命令执行
    match COMMAND.get(&cmd).cloned() {
        Some(fucs) => {
            for fuc in fucs {
                fuc(app)?;
            }
            // 执行完函数清空数据
            if let ModeData::Command(ref mut command_data) = app.mode {
                command_data.reset();
            }
            if app.mode_key != ModeKey::Exit {
                app::to_normal_mode(app)?;
            }
        }
        None => {
            if let ModeData::Command(ref mut command_data) = app.mode {
                command_data.reset();
            }
            app::to_normal_mode(app)?;
        }
    }
    // 匹配完reset
    if let ModeData::Command(ref mut command_data) = app.mode {
        command_data.reset();
    }
    Ok(())
}

pub fn insert_command(app: &mut Application) -> Result<()> {
    if let Some(key) = app.monitor.last_key {
        if let KeyCode::Char(c) = key.code {
            if let ModeData::Command(ref mut command_data) = app.mode {
                command_data.input.insert(command_data.input.len(), c);
            }
        }
    }
    Ok(())
}

pub fn backspace(app: &mut Application) -> Result<()> {
    if let ModeData::Command(ref mut command_data) = app.mode {
        if command_data.input.is_empty() {
            return app::to_normal_mode(app);
        } else {
            command_data.input.remove(command_data.input.len() - 1);
        }
    }
    Ok(())
}

pub fn to_normal_mode(app: &mut Application) -> Result<()> {
    if let ModeData::Command(ref mut command_data) = app.mode {
        command_data.reset();
    }
    app.switch_mode(ModeKey::Normal);
    Ok(())
}
