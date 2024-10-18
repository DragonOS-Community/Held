use std::{collections::HashMap, process::CommandArgs};

use super::{ModeData, ModeRenderer};
use crate::errors::Error;
use crate::{
    application::Application,
    view::{
        colors::colors::Colors,
        status_data::{buffer_status_data, StatusLineData},
        style::CharStyle,
    },
};

const EDITED_NO_STORE: &'static str = "Changes have not been saved";
const NOT_FOUNT_CMD: &'static str = "Command Not Fount";

pub(super) struct CommandRenderer;

impl ModeRenderer for CommandRenderer {
    fn render(
        workspace: &mut crate::workspace::Workspace,
        monitor: &mut crate::view::monitor::Monitor,
        mode: &mut super::ModeData,
    ) -> super::Result<()> {
        let mut presenter = monitor.build_presenter()?;

        if let Some(buffer) = &workspace.current_buffer {
            let data = buffer.data();
            presenter.print_buffer(buffer, &data, &workspace.syntax_set, None, None)?;

            let mode_name_data = StatusLineData {
                content: " COMMAND ".to_string(),
                color: Colors::Inverted,
                style: CharStyle::Bold,
            };
            presenter.print_status_line(&[
                mode_name_data,
                buffer_status_data(&workspace.current_buffer),
            ])?;
            if let ModeData::Command(ref command_data) = mode {
                presenter.print_last_line(command_data)?;
            };

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
