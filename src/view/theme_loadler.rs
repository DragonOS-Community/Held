use std::path::PathBuf;

use syntect::highlighting::ThemeSet;

pub struct ThemeLoader {
    path: PathBuf,
}

impl ThemeLoader {
    pub fn new(path: PathBuf) -> ThemeLoader {
        ThemeLoader { path }
    }

    pub fn load(&self) -> ThemeSet {
        todo!()
    }
}
