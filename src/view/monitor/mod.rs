use std::{cell::RefCell, collections::HashMap, rc::Rc, sync::Arc};

use super::{
    presenter::Presenter,
    render::{render_buffer::CachedRenderBuffer, render_state::RenderState},
    terminal::{cross_terminal::CrossTerminal, Terminal},
    theme_loadler::ThemeLoader,
};
use crate::errors::*;
use crate::modules::perferences::Perferences;
use crate::{buffer::Buffer, plugin::system::PluginSystem};
use crossterm::event::{Event, KeyEvent};
use scroll_controller::ScrollController;
use syntect::highlighting::{Theme, ThemeSet};

pub mod scroll_controller;

/// 管理所有的显示
pub struct Monitor {
    pub terminal: Arc<Box<dyn Terminal>>,
    theme_set: ThemeSet,
    pub perference: Rc<RefCell<dyn Perferences>>,
    scroll_controllers: HashMap<usize, ScrollController>,
    render_caches: HashMap<usize, Rc<RefCell<HashMap<usize, RenderState>>>>,
    pub last_key: Option<KeyEvent>,
    pub cached_render_buffer: Rc<RefCell<CachedRenderBuffer>>,
    pub plugin_system: Rc<RefCell<PluginSystem>>,
}

impl Monitor {
    pub fn new(
        perference: Rc<RefCell<dyn Perferences>>,
        plugin_system: Rc<RefCell<PluginSystem>>,
    ) -> Result<Monitor> {
        let terminal = CrossTerminal::new()?;
        let cached_render_buffer = CachedRenderBuffer::new(terminal.width()?, terminal.height()?);
        let theme_set = ThemeLoader::new(perference.borrow().theme_path()?).load()?;
        Ok(Monitor {
            terminal: Arc::new(Box::new(terminal)),
            theme_set,
            perference,
            scroll_controllers: HashMap::new(),
            render_caches: HashMap::new(),
            last_key: None,
            cached_render_buffer: Rc::new(RefCell::new(cached_render_buffer)),
            plugin_system,
        })
    }

    pub fn init_buffer(&mut self, buffer: &mut Buffer) -> Result<()> {
        let id = buffer.id()?;
        self.scroll_controllers.insert(
            id,
            ScrollController::new(self.terminal.clone(), buffer.cursor.line),
        );
        let render_cache = Rc::new(RefCell::new(HashMap::new()));
        self.render_caches.insert(id, render_cache.clone());

        // 回调清除render_cache
        buffer.change_callback = Some(Box::new(move |change_position| {
            render_cache
                .borrow_mut()
                .retain(|&k, _| k < change_position.line);
        }));
        Ok(())
    }

    pub fn deinit_buffer(&mut self, buffer: &Buffer) -> Result<()> {
        let id = buffer.id()?;
        self.scroll_controllers.remove(&id);
        self.render_caches.remove(&id);
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

    pub fn first_theme(&self) -> Option<Theme> {
        self.theme_set
            .themes
            .first_key_value()
            .map(|(_, v)| v.clone())
    }

    pub fn build_presenter(&mut self) -> Result<Presenter> {
        Presenter::new(self)
    }

    pub fn get_render_cache(&self, buffer: &Buffer) -> &Rc<RefCell<HashMap<usize, RenderState>>> {
        self.render_caches.get(&buffer.id.unwrap()).unwrap()
    }

    pub fn get_scroll_controller(&mut self, buffer: &Buffer) -> &mut ScrollController {
        self.scroll_controllers
            .entry(buffer.id.unwrap())
            .or_insert(ScrollController::new(self.terminal.clone(), 0))
    }

    pub fn scroll_to_cursor(&mut self, buffer: &Buffer) -> Result<()> {
        self.get_scroll_controller(buffer)
            .scroll_into_monitor(buffer)
    }

    pub fn scroll_to_center(&mut self, buffer: &Buffer) -> Result<()> {
        self.get_scroll_controller(buffer).scroll_to_center(buffer)
    }

    pub fn scroll_up(&mut self, buffer: &Buffer, count: usize) {
        self.get_scroll_controller(buffer).scroll_up(count);
    }

    pub fn scroll_down(&mut self, buffer: &Buffer, count: usize) {
        self.get_scroll_controller(buffer).scroll_down(count);
    }
}
