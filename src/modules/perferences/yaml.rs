use std::path::PathBuf;

use super::{Perferences, APP_INFO, LINE_WRAPPING_KEY, THEME_KET, THEME_PATH};
use crate::{
    errors::*,
    modules::perferences::{LANGUAGE_KEY, LANGUAGE_SYNTAX_KEY, SYNTAX_PATH},
};
use app_dirs2::{app_dir, AppDataType};
use yaml_rust::Yaml;

pub struct YamlPerferences {
    data: Yaml,
}

impl YamlPerferences {
    pub fn new(yaml: Yaml) -> YamlPerferences {
        YamlPerferences { data: yaml }
    }
}

impl Perferences for YamlPerferences {
    fn load(&mut self) {
        todo!()
    }

    fn line_wrapping(&self) -> bool {
        self.data[LINE_WRAPPING_KEY].as_bool().unwrap_or(true)
    }

    fn tab_width(&self) -> usize {
        todo!()
    }

    fn soft_tab(&self) -> bool {
        todo!()
    }

    fn theme_name(&self) -> Option<String> {
        self.data[THEME_KET].as_str().map(|x| x.to_owned())
    }

    fn syntax_definition_name(&self, path: &std::path::Path) -> Option<String> {
        if let Some(extension) = path.extension().and_then(|f| f.to_str()) {
            if let Some(syntax_definition) =
                self.data[LANGUAGE_KEY][extension][LANGUAGE_SYNTAX_KEY].as_str()
            {
                return Some(syntax_definition.to_string());
            }
        }

        None
    }
}
