use super::ModeRenderer;
use crate::{application::mode::ModeData, errors::*};
pub struct ErrorRenderer;

impl ModeRenderer for ErrorRenderer {
    fn render(
        workspace: &mut crate::workspace::Workspace,
        monitor: &mut crate::view::monitor::Monitor,
        mode: &mut super::ModeData,
    ) -> Result<()> {
        if let ModeData::Error(e) = mode {
            panic!("{e:?}");
        }
        todo!()
    }
}
