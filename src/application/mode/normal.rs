use super::ModeRenderer;
use crate::{
    errors::*,
    view::{
        colors::colors::Colors,
        status_data::{buffer_status_data, StatusLineData},
        style::CharStyle,
    },
};
pub(super) struct NormalRenderer;

impl ModeRenderer for NormalRenderer {
    fn render(
        workspace: &mut crate::workspace::Workspace,
        monitor: &mut crate::view::monitor::Monitor,
        _mode: &mut super::ModeData,
    ) -> Result<()> {
        let mut presenter = monitor.build_presenter()?;

        if let Some(buffer) = &workspace.current_buffer {
            warn!("normal buffer id: {}", buffer.id.unwrap());
            let data = buffer.data();
            presenter.print_buffer(buffer, &data, &workspace.syntax_set, None, None)?;

            let mode_name_data = StatusLineData {
                content: " NORMAL ".to_string(),
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
