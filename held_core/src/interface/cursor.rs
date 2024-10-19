use crate::utils::position::Position;

use super::get_application;

pub trait Cursor {
    fn move_left(&mut self);

    fn move_right(&mut self);

    fn move_up(&mut self);

    fn move_down(&mut self);

    fn move_to_start_of_line(&mut self);

    fn screen_cursor_position(&self) -> Position;
}

pub fn screen_cursor_position() -> Position {
    get_application().screen_cursor_position()
}

pub fn move_down() {
    get_application().move_down()
}
