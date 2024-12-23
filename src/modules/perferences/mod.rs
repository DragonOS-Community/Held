use crate::{errors::*, utils::ui::uicore::APP_INTERNAL_INFOMATION};
use app_dirs2::{app_dir, AppDataType, AppInfo};
use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    rc::Rc,
};
use yaml::YamlPerferences;
use yaml_rust::{Yaml, YamlLoader};

use super::APP_INFO;

pub mod yaml;

const SYNTAX_PATH: &str = "syntaxes";
const THEME_PATH: &str = "themes";
const INPUT_CONFIG_PATH: &str = "input";
const PLUGINS_PATH: &str = "plugins";
const THEME_KET: &str = "theme";
const LANGUAGE_KEY: &str = "language";
const LANGUAGE_SYNTAX_KEY: &str = "syntax";
const LINE_WRAPPING_KEY: &str = "line_wrapping";
const SOFT_TAB_KEY: &str = "soft_tab";
const TAB_WIDTH_KEY: &str = "tab_width";

pub trait Perferences {
    /// 载入
    fn load(&mut self);

    /// 是否自动换行
    fn line_wrapping(&self) -> bool;

    // tab宽度
    fn tab_width(&self) -> usize;

    // 是否使用空格模拟tab
    fn soft_tab(&self) -> bool;

    // 设置的主题文件路径
    fn theme_path(&self) -> Result<PathBuf> {
        #[cfg(not(feature = "dragonos"))]
        {
            app_dir(AppDataType::UserConfig, &APP_INFO, THEME_PATH)
                .chain_err(|| "Couldn't create a themes directory or build a path tp it")
        }
        #[cfg(feature = "dragonos")]
        Ok(PathBuf::new())
    }

    // 输入映射配置文件路径
    fn input_config_path(&self) -> Result<PathBuf> {
        #[cfg(not(feature = "dragonos"))]
        {
            app_dir(AppDataType::UserConfig, &APP_INFO, INPUT_CONFIG_PATH)
                .chain_err(|| "Couldn't create a themes directory or build a path tp it")
        }
        #[cfg(feature = "dragonos")]
        Ok(PathBuf::new())
    }

    // 插件路径
    fn plugins_path(&self) -> Result<PathBuf> {
        #[cfg(not(feature = "dragonos"))]
        {
            app_dir(AppDataType::UserConfig, &APP_INFO, PLUGINS_PATH)
                .chain_err(|| "Couldn't create a themes directory or build a path tp it")
        }
        #[cfg(feature = "dragonos")]
        Ok(PathBuf::new())
    }

    // 设置的主题名字
    fn theme_name(&self) -> Option<String>;

    // 返回设置的语法定义：例：test.rs -> rs test.cpp -> cpp
    fn syntax_definition_name(&self, path: &Path) -> Option<String>;
}

pub struct PerferencesManager;

impl PerferencesManager {
    pub fn load() -> Result<Rc<RefCell<dyn Perferences>>> {
        match Self::load_extend()? {
            Some(_) => todo!(),
            None => return Ok(Rc::new(RefCell::new(Self::load_default_perferences()?))),
        }
    }

    pub fn user_syntax_path() -> Result<PathBuf> {
        app_dir(AppDataType::UserConfig, &APP_INFO, SYNTAX_PATH)
            .chain_err(|| "Couldn't create syntax directory or build a path to it.")
    }

    fn load_default_perferences() -> Result<YamlPerferences> {
        let yaml = YamlLoader::load_from_str(include_str!("default.yaml"))
            .chain_err(|| "Couldn't parse default config file")?
            .into_iter()
            .next()
            .chain_err(|| "No default preferences document found")?;

        Ok(YamlPerferences::new(yaml))
    }

    fn load_extend() -> Result<Option<Rc<RefCell<dyn Perferences>>>> {
        // 可能涉及加载其他格式的配置文件
        Ok(None)
    }
}

#[cfg(test)]
pub struct DummyPerferences;
#[cfg(test)]
impl Perferences for DummyPerferences {
    fn line_wrapping(&self) -> bool {
        true
    }

    fn tab_width(&self) -> usize {
        2
    }

    fn soft_tab(&self) -> bool {
        true
    }

    fn theme_path(&self) -> Result<PathBuf> {
        todo!()
    }

    fn theme_name(&self) -> Option<String> {
        todo!()
    }

    fn load(&mut self) {
        todo!()
    }

    fn syntax_definition_name(&self, path: &Path) -> Option<String> {
        todo!()
    }

    fn input_config_path(&self) -> Result<PathBuf> {
        todo!()
    }
}
