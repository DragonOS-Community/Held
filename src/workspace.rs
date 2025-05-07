use std::{
    cell::Ref,
    collections::HashMap,
    env,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};

use crate::{
    errors::*,
    modules::perferences::{Perferences, PerferencesManager},
    view::monitor::Monitor,
};
use syntect::parsing::SyntaxSet;

use crate::buffer::Buffer;

pub struct Workspace {
    pub path: PathBuf,
    buffers: HashMap<usize, Buffer>,
    // ino -> id
    buffers_ino_map: HashMap<u64, usize>,
    pub current_buffer: Option<Buffer>,
    pub syntax_set: SyntaxSet,
    buffer_ida: usize,
}

impl Workspace {
    pub fn create_workspace(
        monitor: &mut Monitor,
        perferences: Ref<dyn Perferences>,
        args: &[String],
    ) -> Result<Workspace> {
        let mut path_args = args.iter().skip(1).peekable();

        let initial_dir = env::current_dir()?;
        // 若第一个参数为dir，则修改工作区
        if let Some(dir) = path_args.peek() {
            let path = Path::new(dir);
            if path.is_dir() {
                env::set_current_dir(path.canonicalize()?)?;
            }
        }
        let workspace_dir = env::current_dir()?;
        #[cfg(feature = "dragonos")]
        let syntax_path: Option<PathBuf> = None;
        #[cfg(not(feature = "dragonos"))]
        let syntax_path = PerferencesManager::user_syntax_path().map(Some)?;
        let mut workspace = Workspace::new(&workspace_dir, syntax_path.as_deref())?;

        if workspace_dir != initial_dir {
            path_args.next();
        }

        for path_str in path_args {
            let path = Path::new(path_str);
            if path.is_dir() {
                continue;
            }

            let syntax_ref = perferences
                .syntax_definition_name(&path)
                .and_then(|name| workspace.syntax_set.find_syntax_by_name(&name).cloned());

            let buffer = if path.exists() {
                let mut buffer = Buffer::from_file(&path)?;
                buffer.syntax_definition = syntax_ref;
                buffer
            } else {
                let mut buffer = Buffer::new();
                buffer.syntax_definition = syntax_ref;

                if path.is_absolute() {
                    buffer.path = Some(path.to_path_buf());
                } else {
                    buffer.path = Some(workspace_dir.join(path))
                }
                buffer
            };

            workspace.add_buffer_with_select(buffer);
            monitor.init_buffer(workspace.current_buffer.as_mut().unwrap())?;
        }

        Ok(workspace)
    }

    fn new(path: &Path, syntax_definitions: Option<&Path>) -> Result<Workspace> {
        let mut syntax_set = SyntaxSet::load_defaults_newlines();
        if let Some(syntax_definitions) = syntax_definitions {
            if syntax_definitions.is_dir() {
                if syntax_definitions.read_dir()?.count() > 0 {
                    let mut builder = syntax_set.into_builder();
                    builder.add_from_folder(syntax_definitions, true)?;
                    syntax_set = builder.build();
                }
            }
        }

        Ok(Workspace {
            path: path.canonicalize()?,
            buffers: HashMap::new(),
            buffers_ino_map: HashMap::new(),
            current_buffer: None,
            syntax_set,
            buffer_ida: 0,
        })
    }

    pub fn add_buffer(&mut self, mut buffer: Buffer) -> usize {
        let id = self.alloc_buffer_id();
        buffer.id = Some(id);

        if let Some(ref path) = buffer.path {
            if let Ok(metadata) = path.metadata() {
                self.buffers_ino_map.insert(metadata.ino(), id);
            }
        }

        self.buffers.insert(id, buffer);
        if let Some(buffer) = self.get_buffer(id) {
            if buffer.syntax_definition.is_none() {
                let _ = self.update_buffer_syntax(id);
            }
        }

        return id;
    }

    pub fn add_buffer_with_select(&mut self, buffer: Buffer) -> usize {
        let id = self.add_buffer(buffer);
        self.select_buffer(id);
        return id;
    }

    fn alloc_buffer_id(&mut self) -> usize {
        self.buffer_ida += 1;
        self.buffer_ida
    }

    pub fn get_buffer(&self, id: usize) -> Option<&Buffer> {
        if let Some(ref buffer) = self.current_buffer {
            if buffer.id.unwrap() == id {
                return Some(buffer);
            }
        }
        return self.buffers.get(&id);
    }

    pub fn get_buffer_mut(&mut self, id: usize) -> Option<&mut Buffer> {
        if let Some(ref mut buffer) = self.current_buffer {
            if buffer.id.unwrap() == id {
                return Some(buffer);
            }
        }
        return self.buffers.get_mut(&id);
    }

    pub fn get_buffer_with_ino(&self, ino: u64) -> Option<&Buffer> {
        if let Some(id) = self.buffers_ino_map.get(&ino) {
            return self.get_buffer(*id);
        }
        None
    }

    pub fn select_buffer(&mut self, id: usize) -> bool {
        // 将当前buffer放回
        if let Some(buffer) = self.current_buffer.take() {
            if buffer.id.unwrap() == id {
                self.current_buffer = Some(buffer);
                return true;
            }
            self.buffers.insert(buffer.id.unwrap(), buffer);
        }

        // 选择新buffer
        if let Some(buffer) = self.buffers.remove(&id) {
            self.current_buffer = Some(buffer);
            return true;
        }

        false
    }

    fn update_buffer_syntax(&mut self, id: usize) -> Result<()> {
        let buffer = self.get_buffer(id).ok_or(ErrorKind::EmptyWorkspace)?;
        let definition = buffer
            .file_extension()
            .and_then(|ex| self.syntax_set.find_syntax_by_extension(&ex))
            .or_else(|| Some(self.syntax_set.find_syntax_plain_text()))
            .cloned();

        drop(buffer);
        self.get_buffer_mut(id)
            .ok_or(ErrorKind::EmptyWorkspace)?
            .syntax_definition = definition;

        Ok(())
    }
}
