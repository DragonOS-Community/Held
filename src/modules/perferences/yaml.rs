use super::{Perferences, LINE_WRAPPING_KEY, SOFT_TAB_KEY, TAB_WIDTH_KEY, THEME_KET};
use crate::modules::perferences::{LANGUAGE_KEY, LANGUAGE_SYNTAX_KEY};
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
        self.data[TAB_WIDTH_KEY].as_i64().unwrap_or(4) as usize
    }

    fn soft_tab(&self) -> bool {
        self.data[SOFT_TAB_KEY].as_bool().unwrap_or(true)
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
