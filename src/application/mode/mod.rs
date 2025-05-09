use std::collections::HashMap;

use crate::errors::*;
use crate::{view::monitor::Monitor, workspace::Workspace};
use command::{CommandData, CommandRenderer};
use delete::DeleteRenderer;
use error::ErrorRenderer;
use error_chain::bail;
use insert::InsertRenderer;
use linked_hash_map::LinkedHashMap;
use normal::NormalRenderer;
use replace::ReplaceRenderer;
use search::{SearchData, SearchRenderer};
use smallvec::SmallVec;
use strum::EnumIter;
use workspace::{WorkspaceModeData, WorkspaceRender};
use yaml_rust::Yaml;

use super::handler::handle_map;
use super::Application;

pub mod command;
pub mod delete;
pub mod error;
mod insert;
pub mod motion;
pub mod normal;
mod replace;
pub mod search;
pub mod workspace;

pub enum ModeData {
    Normal,
    Error(Error),
    Exit,
    Insert,
    Command(CommandData),
    Workspace(WorkspaceModeData),
    Search(SearchData),
    Delete,
    Replace, // Other(OtherData)
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, EnumIter)]
pub enum ModeKey {
    Normal,
    Error,
    Exit,
    Insert,
    Command,
    Workspace,
    Search,
    Delete,
    Replace,
}

impl ModeKey {
    pub fn to_string(&self) -> Option<String> {
        match self {
            ModeKey::Normal => Some("normal".into()),
            ModeKey::Insert => Some("insert".into()),
            ModeKey::Command => Some("command".into()),
            ModeKey::Workspace => Some("workspace".into()),
            ModeKey::Search => Some("search".into()),
            ModeKey::Delete => Some("delete".into()),
            ModeKey::Replace => Some("replace".into()),
            _ => None,
        }
    }

    pub fn generate_handle_map(
        &self,
        mode_map: &mut HashMap<
            String,
            HashMap<String, SmallVec<[fn(&mut Application) -> Result<()>; 4]>>,
        >,
        extra: Option<&LinkedHashMap<Yaml, Yaml>>,
        default: &LinkedHashMap<Yaml, Yaml>,
    ) -> Result<()> {
        let handle_map = handle_map();
        let mut command_map =
            HashMap::<String, SmallVec<[fn(&mut Application) -> Result<()>; 4]>>::new();
        if let Some(mode) = self.to_string() {
            if let Some(yaml) = default.get(&Yaml::String(mode.clone())) {
                if let Some(keys) = yaml.as_hash() {
                    self.parse_mode_keybindings(keys, &handle_map, &mut command_map)?;
                }
            }

            if let Some(extra) = extra {
                if let Some(yaml) = extra.get(&Yaml::String(mode.clone())) {
                    if let Some(keys) = yaml.as_hash() {
                        self.parse_mode_keybindings(keys, &handle_map, &mut command_map)?;
                    }
                }
            }
            mode_map.insert(mode, command_map);
        }

        Ok(())
    }

    fn parse_mode_keybindings(
        &self,
        keybindings: &LinkedHashMap<Yaml, Yaml>,
        handle_map: &HashMap<&str, fn(&mut Application) -> Result<()>>,
        result: &mut HashMap<String, SmallVec<[fn(&mut Application) -> Result<()>; 4]>>,
    ) -> Result<()> {
        for (key, handle) in keybindings {
            if let Some(key) = key.as_str() {
                let mut closures = SmallVec::new();

                match handle {
                    Yaml::String(command_key) => {
                        closures.push(
                            *handle_map
                                .get(command_key.as_str())
                                .ok_or_else(|| format!("command \"{command_key:?}\" not found"))?,
                        );
                    }
                    Yaml::Array(commands) => {
                        for command in commands {
                            let command_key = command.as_str().ok_or_else(|| {
                                format!(
                                    "Keymap command \"{:?}\" couldn't be parsed as a string",
                                    command
                                )
                            })?;

                            closures.push(
                                *handle_map.get(command_key).ok_or_else(|| {
                                    format!("command \"{command_key:?}\" not found")
                                })?,
                            );
                        }
                    }
                    _ => {
                        bail!(format!("conmand: \"{handle:?}\" couldn't be parsed"));
                    }
                }

                result.insert(key.to_string(), closures);
            }
        }

        Ok(())
    }
}

pub trait ModeRenderer {
    fn render(workspace: &mut Workspace, monitor: &mut Monitor, mode: &mut ModeData) -> Result<()>;
}

pub struct ModeRouter;

impl ModeRenderer for ModeRouter {
    fn render(workspace: &mut Workspace, monitor: &mut Monitor, mode: &mut ModeData) -> Result<()> {
        match mode {
            ModeData::Normal => NormalRenderer::render(workspace, monitor, mode),
            ModeData::Error(_) => ErrorRenderer::render(workspace, monitor, mode),
            ModeData::Insert => InsertRenderer::render(workspace, monitor, mode),
            ModeData::Command(_) => CommandRenderer::render(workspace, monitor, mode),
            ModeData::Workspace(_) => WorkspaceRender::render(workspace, monitor, mode),
            ModeData::Search(_) => SearchRenderer::render(workspace, monitor, mode),
            ModeData::Replace => ReplaceRenderer::render(workspace, monitor, mode),
            ModeData::Exit => todo!(),
            ModeData::Delete => DeleteRenderer::render(workspace, monitor, mode),
        }
    }
}
