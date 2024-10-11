use std::{path::PathBuf, sync::Mutex};

use held_core::plugin::Plugin;
use walkdir::WalkDir;

pub static mut PULGIN_MANAGER: PluginSystem = PluginSystem::new();

pub struct PluginSystem {
    plugins: Mutex<Vec<Box<dyn Plugin>>>,
}

unsafe impl Send for PluginSystem {}
unsafe impl Sync for PluginSystem {}

impl PluginSystem {
    const fn new() -> Self {
        Self {
            plugins: Mutex::new(Vec::new()),
        }
    }

    // #[allow(static_mut_refs)]
    // pub fn get_instance() -> &'static mut Self {
    //     return unsafe { &mut PULGIN_MANAGER };
    // }

    pub fn init_system(pulgin_dir: PathBuf) {
        unsafe {
            PULGIN_MANAGER.load_pulgins(pulgin_dir);
        };
    }

    pub unsafe fn load_pulgins(&mut self, pulgin_dir: PathBuf) {
        for entry in WalkDir::new(pulgin_dir) {
            if let Ok(entry) = entry {
                if entry.file_type().is_file() {
                    let path = entry.into_path();
                    if let Err(e) = self.load_pulgin(&path) {
                        error!("load pulgin: {:?}, load error: {e:?}", path)
                    }
                }
            }
        }
    }

    pub unsafe fn load_pulgin(&mut self, pulgin_path: &PathBuf) -> Result<(), libloading::Error> {
        let lib = libloading::Library::new(pulgin_path)?;
        let get_plugins: libloading::Symbol<extern "Rust" fn() -> Vec<Box<dyn Plugin>>> =
            lib.get(b"get_plugins")?;

        let mut plugins = get_plugins();
        Self::init_plugins(&mut plugins);
        self.plugins.lock().unwrap().append(&mut plugins);
        Ok(())
    }

    fn init_plugins(plugins: &mut Vec<Box<dyn Plugin>>) {
        for plugin in plugins {
            plugin.init();
        }
    }
}
