use held_core::utils::position::Position;

/// 用于记录当前实时更新的状态信息
/// 因为某些插件可能需要获取实时的状态信息，所以所有实时的状态都可能需要更新到这里
#[derive(Debug, Default)]
pub struct ApplicationStateData {
    pub cursor_state: CursorStateData,
}

#[derive(Debug, Default)]
pub struct CursorStateData {
    pub screen_position: Position,
}
