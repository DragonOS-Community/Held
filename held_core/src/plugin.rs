use std::sync::Mutex;

static MANAGER: PluginRegister = PluginRegister::new();

pub struct PluginRegister {
    plugins: Mutex<Vec<Box<dyn Plugin>>>,
}

unsafe impl Send for PluginRegister {}
unsafe impl Sync for PluginRegister {}

impl PluginRegister {
    const fn new() -> Self {
        Self {
            plugins: Mutex::new(Vec::new()),
        }
    }
    pub fn get_instance() -> &'static Self {
        return &MANAGER;
    }

    pub fn register_plugin(&self, plugin: Box<dyn Plugin>) {
        self.plugins.lock().unwrap().push(plugin);
    }

    pub(crate) fn consume(&self) -> Vec<Box<dyn Plugin>> {
        let mut guard = self.plugins.lock().unwrap();
        let mut ret = Vec::with_capacity(guard.len());
        ret.append(&mut guard);
        ret
    }
}

pub trait Plugin {
    fn name(&self) -> String;

    fn init(&mut self);

    fn deinit(&mut self);
}
