use held_core::view::{colors::Colors, style::CharStyle};

use crate::view::status_data::{buffer_status_data, StatusLineData};

use super::ModeRenderer;

pub(super) struct InsertRenderer;

impl ModeRenderer for InsertRenderer {
    fn render(
        workspace: &mut crate::workspace::Workspace,
        monitor: &mut crate::view::monitor::Monitor,
        _mode: &mut super::ModeData,
    ) -> super::Result<()> {
        let mut presenter = monitor.build_presenter()?;

        if let Some(buffer) = &workspace.current_buffer {
            let data = buffer.data();
            presenter.print_buffer(buffer, &data, &workspace.syntax_set, None, None)?;

            let mode_name_data = StatusLineData {
                content: " INSERT ".to_string(),
                color: Colors::Inverted,
                style: CharStyle::Bold,
            };
            presenter.print_status_line(&[
                mode_name_data,
                buffer_status_data(&workspace.current_buffer),
            ])?;

            presenter.present()?;
        } else {
        }

        Ok(())
    }
}
