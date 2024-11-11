use held_core::interface::{app::App, ApplicationInterface};

use super::Application;

pub mod app;
pub mod buffer;
pub mod cursor;
pub mod monitor;
pub mod workspace;

impl ApplicationInterface for Application {}
