pub mod control;
pub mod interface;
pub mod plugin;
pub mod theme;
pub mod utils;
pub mod view;

#[macro_export]
macro_rules! declare_plugin {
    ($app:ty, $constructor:path) => {
        use held_core::interface::ApplicationInterface;
        use held_core::interface::APPLICATION;

        #[no_mangle]
        pub unsafe extern "C" fn init_plugin_application(
            app: &'static mut dyn ApplicationInterface,
        ) {
            APPLICATION = Some(app);
        }

        #[no_mangle]
        pub extern "C" fn plugin_create() -> *mut dyn $crate::plugin::Plugin {
            // 确保构造器正确，所以做了这一步骤，来显示声明签名
            let constructor: fn() -> $app = $constructor;
            let object = constructor();
            let boxed: Box<dyn $crate::plugin::Plugin> = Box::new(object);
            Box::into_raw(boxed)
        }
    };
}
