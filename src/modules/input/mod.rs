use std::{collections::HashMap, ffi::OsStr, fs::read_to_string, path::PathBuf};

use crate::{
    application::{mode::ModeKey, Application},
    errors::*,
};
use crossterm::event::{Event, KeyEvent, KeyEventKind, KeyModifiers};
use linked_hash_map::LinkedHashMap;

use smallvec::SmallVec;
use strum::IntoEnumIterator;
use yaml_rust::{Yaml, YamlLoader};

const INPUT_CONFIG_NAME: &str = "input.yaml";
pub struct InputLoader;

impl InputLoader {
    pub fn load(
        path: PathBuf,
    ) -> Result<HashMap<String, HashMap<String, SmallVec<[fn(&mut Application) -> Result<()>; 4]>>>>
    {
        #[cfg(not(feature = "dragonos"))]
        let data = Self::load_user(path)?;
        #[cfg(feature = "dragonos")]
        let data = None;
        let default = Self::load_default()?;
        let handle_map = Self::generate_handle_map(
            data,
            default
                .as_hash()
                .ok_or_else(|| "default input config didn't return a hash of key bindings")?,
        )?;
        Ok(handle_map)
    }

    fn generate_handle_map(
        extra_data: Option<LinkedHashMap<Yaml, Yaml>>,
        default: &LinkedHashMap<Yaml, Yaml>,
    ) -> Result<HashMap<String, HashMap<String, SmallVec<[fn(&mut Application) -> Result<()>; 4]>>>>
    {
        let mut handle_map = HashMap::new();
        for mode_key in ModeKey::iter() {
            mode_key.generate_handle_map(&mut handle_map, extra_data.as_ref(), default)?;
        }
        Ok(handle_map)
    }

    fn load_user(path: PathBuf) -> Result<Option<LinkedHashMap<Yaml, Yaml>>> {
        let readdir = path.read_dir()?;
        let mut entries = readdir
            .filter_map(|f| f.ok())
            .map(|f| f.path())
            .filter(|f| f.is_file())
            .filter(|f| {
                f.file_name().is_some() && f.file_name().unwrap() == OsStr::new(INPUT_CONFIG_NAME)
            });

        let entry = entries.next();
        if let Some(entry) = entry {
            let yaml = YamlLoader::load_from_str(&read_to_string(entry.clone())?)
                .chain_err(|| format!("Couldn't parse input config file: {:?}", entry))?
                .into_iter()
                .next()
                .chain_err(|| "No input document found")?;
            let yaml_hash = yaml
                .as_hash()
                .ok_or_else(|| "extra input config didn't return a hash of key bindings")?;

            return Ok(Some(yaml_hash.clone()));
        }

        Ok(None)
    }

    fn load_default() -> Result<Yaml> {
        YamlLoader::load_from_str(include_str!("default.yaml"))
            .chain_err(|| "Couldn't parse default input config file")?
            .into_iter()
            .next()
            .chain_err(|| "No default input document found")
    }
}

pub struct InputMapper;

impl InputMapper {
    pub fn event_map_str(event: Event) -> Option<String> {
        match event {
            Event::FocusGained => None,
            Event::FocusLost => None,
            Event::Key(key_event) => {
                return Some(Self::key_event_map_str(key_event));
            }
            Event::Mouse(_) => None,
            Event::Paste(_) => None,
            Event::Resize(_, _) => None,
        }
    }

    fn key_event_map_str(event: KeyEvent) -> String {
        if let KeyEventKind::Press = event.kind {
            let mut modifier = String::new();
            if event.modifiers.contains(KeyModifiers::CONTROL) {
                modifier.push_str("ctrl-");
            }
            if event.modifiers.contains(KeyModifiers::ALT) {
                modifier.push_str("alt-");
            }
            if event.modifiers.contains(KeyModifiers::SHIFT) {
                modifier.push_str("shift-");
            }
            let keycode_str = match event.code {
                crossterm::event::KeyCode::Backspace => "backspace".into(),
                crossterm::event::KeyCode::Enter => "enter".into(),
                crossterm::event::KeyCode::Left => "left".into(),
                crossterm::event::KeyCode::Right => "right".into(),
                crossterm::event::KeyCode::Up => "up".into(),
                crossterm::event::KeyCode::Down => "down".into(),
                crossterm::event::KeyCode::Home => "home".into(),
                crossterm::event::KeyCode::End => "end".into(),
                crossterm::event::KeyCode::PageUp => "pageup".into(),
                crossterm::event::KeyCode::PageDown => "pagedown".into(),
                crossterm::event::KeyCode::Tab => "tab".into(),
                crossterm::event::KeyCode::BackTab => "backtab".into(),
                crossterm::event::KeyCode::Delete => "delete".into(),
                crossterm::event::KeyCode::Insert => "insert".into(),
                crossterm::event::KeyCode::F(f) => format!("f{f}"),
                crossterm::event::KeyCode::Char(c) => {
                    if c.is_digit(10) {
                        "num".to_string()
                    } else {
                        c.into()
                    }
                }
                crossterm::event::KeyCode::Null => "".into(),
                crossterm::event::KeyCode::Esc => "escape".into(),
                crossterm::event::KeyCode::CapsLock => "caps_lock".into(),
                crossterm::event::KeyCode::ScrollLock => "scroll_lock".into(),
                crossterm::event::KeyCode::NumLock => "num_lock".into(),
                crossterm::event::KeyCode::PrintScreen => "print_screen".into(),
                crossterm::event::KeyCode::Pause => "pause".into(),
                crossterm::event::KeyCode::Menu => "menu".into(),
                crossterm::event::KeyCode::KeypadBegin => "keypad_begin".into(),
                crossterm::event::KeyCode::Media(_) => "".into(),
                crossterm::event::KeyCode::Modifier(modifier_key_code) => match modifier_key_code {
                    crossterm::event::ModifierKeyCode::LeftShift
                    | crossterm::event::ModifierKeyCode::IsoLevel3Shift
                    | crossterm::event::ModifierKeyCode::IsoLevel5Shift
                    | crossterm::event::ModifierKeyCode::RightShift => "shift".into(),
                    crossterm::event::ModifierKeyCode::LeftControl
                    | crossterm::event::ModifierKeyCode::RightControl => "ctrl".into(),
                    crossterm::event::ModifierKeyCode::LeftAlt
                    | crossterm::event::ModifierKeyCode::RightAlt => "alt".into(),
                    crossterm::event::ModifierKeyCode::RightSuper
                    | crossterm::event::ModifierKeyCode::LeftSuper => "super".into(),
                    crossterm::event::ModifierKeyCode::RightHyper
                    | crossterm::event::ModifierKeyCode::LeftHyper => "hyper".into(),
                    crossterm::event::ModifierKeyCode::RightMeta
                    | crossterm::event::ModifierKeyCode::LeftMeta => "meta".into(),
                },
            };

            format!("{}{}", modifier, keycode_str)
        } else {
            "".into()
        }
    }
}
