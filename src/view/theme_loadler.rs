use crate::errors::*;
use app_dirs2::{app_dir, AppDataType};
use error_chain::bail;
use std::{
    collections::BTreeMap,
    ffi::OsStr,
    fs::File,
    io::{BufReader, Cursor, Read, Seek},
    path::PathBuf,
};
use syntect::highlighting::{Theme, ThemeSet};

pub struct ThemeLoader {
    path: PathBuf,
    themes: BTreeMap<String, Theme>,
}

impl ThemeLoader {
    pub fn new(path: PathBuf) -> ThemeLoader {
        ThemeLoader {
            path,
            themes: BTreeMap::new(),
        }
    }

    pub fn load(mut self) -> Result<ThemeSet> {
        self.load_default();
        #[cfg(not(feature = "dragonos"))]
        {
            self.load_user()?;
        }

        Ok(ThemeSet {
            themes: self.themes,
        })
    }

    fn load_user(&mut self) -> Result<()> {
        let dir = self.path.read_dir()?;

        let entries = dir
            .filter_map(|f| f.ok())
            .map(|f| f.path())
            .filter(|f| f.is_file())
            .filter(|f| f.extension() == Some(OsStr::new("tmTheme")));

        for entry in entries {
            if let Ok(file) = File::open(&entry) {
                if let Some(name) = entry.file_stem() {
                    if let Some(name) = name.to_str() {
                        self.add_theme(name.into(), file);
                    }
                }
            }
        }

        Ok(())
    }

    fn load_default(&mut self) {
        self.add_theme(
            "solarized_dark".into(),
            Cursor::new(include_str!("../themes/solarized_dark.tmTheme")),
        );
    }

    fn add_theme<D: Read + Seek>(&mut self, name: String, data: D) {
        let mut reader = BufReader::new(data);

        match ThemeSet::load_from_reader(&mut reader) {
            Ok(theme) => {
                self.themes
                    .insert(theme.name.clone().unwrap_or(name).to_owned(), theme);
            }
            Err(e) => {
                // 主题加载出错不必直接退出
                error!("Failed load theme: {name:?},error: {e:?}");
            }
        };
    }
}
