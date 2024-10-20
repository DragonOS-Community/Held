use app::App;
use buffer::Buffer;
use cursor::Cursor;
use monitor::Monitor;
use workspace::Workspace;

pub mod app;
pub mod buffer;
pub mod cursor;
pub mod monitor;
pub mod render;
pub mod terminal;
pub mod workspace;

pub trait ApplicationInterface: App + Buffer + Cursor + Monitor + Workspace {}
pub static mut APPLICATION: Option<&'static mut dyn ApplicationInterface> = None;

pub(crate) fn get_application() -> &'static mut &'static mut dyn ApplicationInterface {
    unsafe {
        APPLICATION
            .as_mut()
            .expect("The application has not been initialized!")
    }
}
