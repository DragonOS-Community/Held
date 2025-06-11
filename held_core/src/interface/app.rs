use super::get_application;

pub trait App {
    fn exit(&mut self);

    fn to_insert_mode(&mut self);

    fn to_normal_mode(&mut self);
}

pub fn exit() {
    get_application().exit();
}

pub fn to_insert_mode() {
    get_application().to_insert_mode();
}

pub fn to_normal_mode() {
    get_application().to_normal_mode();
}
