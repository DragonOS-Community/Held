pub trait Buffer {
    fn insert_char(&mut self);

    fn new_line(&mut self);

    fn insert_tab(&mut self);
}
