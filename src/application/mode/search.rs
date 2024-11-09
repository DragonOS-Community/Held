use super::ModeRenderer;
use crate::util::range::Range;
use crate::{
    errors::*,
    view::{
        colors::colors::Colors,
        status_data::{buffer_status_data, StatusLineData},
        style::CharStyle,
    },
};
pub(super) struct SearchRenderer;

impl ModeRenderer for SearchRenderer {
    fn render(
        workspace: &mut crate::workspace::Workspace,
        monitor: &mut crate::view::monitor::Monitor,
        _mode: &mut super::ModeData,
    ) -> Result<()> {
        let mut presenter = monitor.build_presenter()?;

        if let Some(buffer) = &workspace.current_buffer {
            let data = buffer.data();

            if let super::ModeData::Search(ref search_data) = _mode {
                let highlight_search_string = search_data.search_result.clone();

                let highlight_search_string_slice: Option<&[Range]> =
                    if !highlight_search_string.is_empty() {
                        Some(
                            &highlight_search_string[search_data.search_result_index
                                ..search_data.search_result_index + 1],
                        )
                    } else {
                        None
                    };

                presenter.print_buffer(
                    buffer,
                    &data,
                    &workspace.syntax_set,
                    highlight_search_string_slice,
                    None,
                )?;

                let mode_name_data = StatusLineData {
                    content: " Search/".to_string(),
                    color: Colors::Inverted,
                    style: CharStyle::Bold,
                };

                let search_data = StatusLineData {
                    content: search_data.search_string.clone(),
                    color: Colors::Default,
                    style: CharStyle::Default,
                };

                presenter.print_status_line(&[
                    mode_name_data,
                    search_data,
                    buffer_status_data(&workspace.current_buffer),
                ])?;

                presenter.present()?;
            } else {
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct SearchData {
    pub search_string: String,
    pub is_exec_search: bool,
    pub search_result_index: usize,
    pub search_result: Vec<Range>,
}

impl SearchData {
    pub fn new() -> Self {
        Self {
            search_string: String::new(),
            is_exec_search: false,
            search_result_index: 0,
            search_result: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.search_string.clear();
        self.is_exec_search = false;
        self.search_result_index = 0;
        self.search_result.clear();
    }
}
