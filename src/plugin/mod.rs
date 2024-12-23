use dlopen2::wrapper::{Container, WrapperApi};
use held_core::{
    interface::ApplicationInterface, plugin::Plugin, view::render::ContentRenderBuffer,
};

pub mod system;

#[derive(WrapperApi)]
pub struct PluginApi {
    plugin_create: unsafe fn() -> *mut dyn Plugin,
    init_plugin_application: unsafe fn(app: &'static mut dyn ApplicationInterface),
}

#[allow(dead_code)]
pub struct PluginInstance {
    // 顺序不能反，需要确保plugin在container之前销毁
    plugin: Box<dyn Plugin>,
    container: Container<PluginApi>,
}

impl PluginInstance {
    pub fn new(plugin: Box<dyn Plugin>, container: Container<PluginApi>) -> PluginInstance {
        PluginInstance { plugin, container }
    }
}

impl Plugin for PluginInstance {
    fn name(&self) -> &'static str {
        self.plugin.name()
    }

    fn init(&self) {
        self.plugin.init()
    }

    fn deinit(&self) {
        self.plugin.deinit()
    }

    fn on_render_content(&self) -> Vec<ContentRenderBuffer> {
        self.plugin.on_render_content()
    }
}
