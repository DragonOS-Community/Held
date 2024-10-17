use crate::application::mode::normal::*;
use crate::application::mode::{ModeData, ModeState};
use crate::application::Application;
use crate::errors::*;

pub fn transition(app: &mut Application) -> Result<()> {
    if let ModeData::Normal(normal_state) = &mut app.mode {
        if let Some(key) = app.monitor.last_key {
            normal_state.transition(&key)?;
        }
    }
    Ok(())
}

pub fn execute(app: &mut Application) -> Result<()> {
    if let ModeData::Normal(normal_state) = &mut app.mode {
        normal_state.cmdchar.inspect(|cmd| {
            match cmd {
                'j' => exec_j_cmd(app),
                'k' => exec_k_cmd(app),
                'd' => exec_d_cmd(app),
                '0' => exec_0_cmd(app),
                'g' => exec_g_cmd(app),
                'G' => exec_G_cmd(app),
                'h' => exec_h_cmd(app),
                'l' => exec_l_cmd(app),
                _ => {}
            };
        });
    }
    Ok(())
}

pub fn reset(app: &mut Application) -> Result<()> {
    if let ModeData::Normal(normal_state) = &mut app.mode {
        normal_state.reset();
    }
    Ok(())
}
