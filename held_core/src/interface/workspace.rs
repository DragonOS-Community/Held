pub trait Workspace {
    fn save_file(&mut self);

    fn undo(&mut self);
}
