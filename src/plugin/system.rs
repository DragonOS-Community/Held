use crate::{errors::*, get_application};
use dlopen2::wrapper::Container;
use held_core::{plugin::Plugin, view::render::ContentRenderBuffer};
use std::{collections::HashMap, path::PathBuf, rc::Rc};
use walkdir::WalkDir;

use crate::plugin::PluginApi;

use super::PluginInstance;

pub struct PluginSystem {
    plugins: HashMap<&'static str, Rc<PluginInstance>>,
}

unsafe impl Send for PluginSystem {}
unsafe impl Sync for PluginSystem {}

impl PluginSystem {
    fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    pub fn init_system(pulgin_dir: PathBuf) -> PluginSystem {
        let mut system = PluginSystem::new();
        system.load_pulgins(pulgin_dir);
        system
    }

    pub fn load_pulgins(&mut self, pulgin_dir: PathBuf) {
        for entry in WalkDir::new(pulgin_dir) {
            if let Ok(entry) = entry {
                if entry.file_type().is_file() {
                    let path = entry.into_path();
                    if let Err(e) = unsafe { self.load_pulgin(&path) } {
                        error!("load pulgin: {:?}, load error: {e:?}", path)
                    }
                }
            }
        }
    }

    pub unsafe fn load_pulgin(&mut self, pulgin_path: &PathBuf) -> Result<()> {
        let container: Container<PluginApi> = Container::load(pulgin_path)?;
        let plugin_raw = container.plugin_create();
        let plugin = Box::from_raw(plugin_raw);
        self.plugins.insert(
            plugin.name(),
            Rc::new(PluginInstance::new(plugin, container)),
        );
        Ok(())
    }
}

impl Plugin for PluginSystem {
    fn name(&self) -> &'static str {
        ""
    }

    fn init(&self) {
        for (_, plugin) in self.plugins.iter() {
            unsafe { plugin.container.init_plugin_application(get_application()) };
            plugin.init();
        }
    }

    fn deinit(&self) {
        for (_, plugin) in self.plugins.iter() {
            plugin.deinit();
        }
    }

    fn on_render_content(&self) -> Vec<ContentRenderBuffer> {
        let mut ret = vec![];
        for (_, plugin) in self.plugins.iter() {
            ret.append(&mut plugin.on_render_content());
        }

        ret
    }
}
