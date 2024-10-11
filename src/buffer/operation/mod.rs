pub use self::group::OperationGroup;
use crate::buffer::Buffer;

mod delete;
pub mod group;
pub mod history;
mod insert;
mod replace;

pub trait Operation {
    fn run(&mut self, buffer: &mut Buffer);
    fn reverse(&mut self, buffer: &mut Buffer);
    fn clone_operation(&self) -> Box<dyn Operation>;
}
