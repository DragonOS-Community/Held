use held_core::interface;

use crate::application::Application;

impl interface::cursor::Cursor for Application {
    fn move_left(&mut self) {
        todo!()
    }

    fn move_right(&mut self) {
        todo!()
    }

    fn move_up(&mut self) {
        todo!()
    }

    fn move_down(&mut self) {
        todo!()
    }

    fn move_to_start_of_line(&mut self) {
        todo!()
    }

    fn screen_cursor_position(&self) -> held_core::utils::position::Position {
        self.monitor.
    }
}
