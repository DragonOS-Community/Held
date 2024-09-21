use std::{io, sync::MutexGuard};

use crate::utils::ui::{event::WarpUiCallBackType, uicore::UiCore};

use super::mode::ModeType;

pub enum StateCallback {
    None,
    Reset,
    Exit(ModeType),
}

pub trait StateMachine {
    fn handle(&mut self, ui: &mut MutexGuard<UiCore>) -> io::Result<WarpUiCallBackType>;
    fn exit(&mut self, callback: WarpUiCallBackType) -> io::Result<WarpUiCallBackType>;
    fn reset(&mut self);
}
