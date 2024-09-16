use mode::ModeType;

pub mod common;
pub mod mode;
pub mod normal;

pub enum StateCallback {
    None,
    Reset,
    Exit(ModeType),
}
