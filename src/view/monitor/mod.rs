use std::{cell::RefCell, collections::HashMap, rc::Rc, sync::Arc};

use super::{
    render::render_state::RenderState,
    terminal::{cross_terminal::CrossTerminal, Terminal},
    theme_loadler::ThemeLoader,
};
use crate::buffer::Buffer;
use crate::errors::*;
use crate::modules::perferences::Perferences;
use crossterm::event::{Event, KeyEvent};
use scroll_controller::ScrollController;
use syntect::highlighting::{Theme, ThemeSet};

pub mod scroll_controller;

/// 管理所有的显示
pub struct Monitor {
    pub terminal: Arc<dyn Terminal>,
    theme_set: ThemeSet,
    pub perference: Rc<RefCell<dyn Perferences>>,
    scroll_controllers: HashMap<usize, ScrollController>,
    render_cache: HashMap<usize, Rc<RefCell<HashMap<usize, RenderState>>>>,
    last_key: Option<KeyEvent>,
}

impl Monitor {
    pub fn new(perference: Rc<RefCell<dyn Perferences>>) -> Result<Monitor> {
        let terminal = CrossTerminal::new()?;
        let theme_set = ThemeLoader::new(perference.borrow().theme_path()).load();
        Ok(Monitor {
            terminal: Arc::new(terminal),
            theme_set,
            perference,
            scroll_controllers: HashMap::new(),
            render_cache: HashMap::new(),
            last_key: None,
        })
    }

    pub fn init_buffer(&mut self, buffer: &Buffer) -> Result<()> {
        let id = buffer.id()?;
        self.scroll_controllers
            .insert(id, ScrollController::new(self.terminal.clone()));
        self.render_cache
            .insert(id, Rc::new(RefCell::new(HashMap::new())));
        Ok(())
    }

    pub fn deinit_buffer(&mut self, buffer: &Buffer) -> Result<()> {
        let id = buffer.id()?;
        self.scroll_controllers.remove(&id);
        self.render_cache.remove(&id);
        Ok(())
    }

    pub fn listen(&mut self) -> Result<Event> {
        let ev = self.terminal.listen()?;
        if let Event::Key(key) = ev {
            self.last_key.replace(key);
        }
        Ok(ev)
    }

    pub fn width(&self) -> Result<usize> {
        self.terminal.width()
    }

    pub fn height(&self) -> Result<usize> {
        self.terminal.height()
    }

    pub fn get_theme(&self, name: &String) -> Option<Theme> {
        self.theme_set.themes.get(name).cloned()
    }
}
