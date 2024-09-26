use plugin::{Plugin, PluginRegister};

pub mod control;
pub mod plugin;
pub mod theme;

#[no_mangle]
fn get_plugins() -> Vec<Box<dyn Plugin>> {
    return PluginRegister::get_instance().consume();
}
