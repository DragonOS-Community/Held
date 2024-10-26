use std::{collections::HashMap, process::CommandArgs};

use held_core::{
    utils::position::Position,
    view::{colors::Colors, style::CharStyle},
};

use crate::view::status_data::StatusLineData;

use super::{ModeData, ModeRenderer};

const EDITED_NO_STORE: &'static str = "Changes have not been saved";
const NOT_FOUNT_CMD: &'static str = "Command Not Fount";

pub(super) struct CommandRenderer;

impl ModeRenderer for CommandRenderer {
    fn render(
        workspace: &mut crate::workspace::Workspace,
        monitor: &mut crate::view::monitor::Monitor,
        mode: &mut super::ModeData,
    ) -> super::Result<()> {
        let line = monitor.height()? - 1;
        let mut presenter = monitor.build_presenter()?;

        if let Some(buffer) = &workspace.current_buffer {
            let data = buffer.data();
            presenter.print_buffer(buffer, &data, &workspace.syntax_set, None, None)?;

            let mode_name_data = StatusLineData {
                content: " COMMAND ".to_string(),
                color: Colors::Inverted,
                style: CharStyle::Bold,
            };

            let cmd_str = if let ModeData::Command(command_data) = mode {
                command_data.input.clone()
            } else {
                String::new()
            };
            let command_line_str = ":".to_owned() + &cmd_str;
            let command_data = StatusLineData {
                content: command_line_str.clone(),
                color: Colors::Default,
                style: CharStyle::Default,
            };

            presenter.print_status_line(&[mode_name_data, command_data])?;

            let offset = " COMMAND ".len() + command_line_str.len();
            presenter.set_cursor(Position { line, offset });

            presenter.present()?;
        } else {
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct CommandData {
    pub input: String,
}

impl CommandData {
    pub fn new() -> Self {
        CommandData {
            input: String::new(),
        }
    }

    pub fn reset(&mut self) {
        self.input.clear();
    }
}
