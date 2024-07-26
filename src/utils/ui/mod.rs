use std::io;

use crossterm::style::Color;

use super::style::StyleManager;

pub mod event;
pub mod mode;
pub mod uicore;
pub mod undotree;

#[derive(Debug)]
pub struct AppInfo {
    pub level: InfoLevel,
    pub info: String,
}

impl AppInfo {
    pub fn reset(&mut self) {
        self.level = InfoLevel::Info;
        self.info = String::new();
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum InfoLevel {
    Info,
    Warn,
    Error,
}

impl InfoLevel {
    pub fn set_style(&self) -> io::Result<()> {
        match self {
            InfoLevel::Info => {}
            InfoLevel::Warn => {
                StyleManager::set_background_color(Color::DarkYellow)?;
            }
            InfoLevel::Error => {
                StyleManager::set_background_color(Color::DarkRed)?;
            }
        }

        Ok(())
    }
}
