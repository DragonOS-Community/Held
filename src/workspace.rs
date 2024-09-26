use std::{
    mem,
    path::{Path, PathBuf},
};

use crate::errors::*;
use syntect::parsing::SyntaxSet;

use crate::buffer::Buffer;

pub struct Workspace {
    pub path: PathBuf,
    buffers: Vec<Buffer>,
    pub current_buffer: Option<Buffer>,
    pub syntax_set: SyntaxSet,
    buffer_ida: usize,
    current_buffer_index: Option<usize>,
}

impl Workspace {
    pub fn new(path: &Path, syntax_definitions: Option<&Path>) -> Result<Workspace> {
        let mut syntax_set = SyntaxSet::load_defaults_newlines();
        if let Some(syntax_definitions) = syntax_definitions {
            let mut builder = syntax_set.into_builder();
            builder.add_from_folder(syntax_definitions, true)?;
            syntax_set = builder.build();
        }

        Ok(Workspace {
            path: path.canonicalize()?,
            buffers: vec![],
            current_buffer: None,
            syntax_set,
            buffer_ida: 0,
            current_buffer_index: None,
        })
    }

    pub fn add_buffer(&mut self, mut buffer: Buffer) {
        buffer.id = Some(self.alloc_buffer_id());

        let target_index = self.current_buffer_index.map(|x| x + 1).unwrap_or(0);

        self.buffers.insert(target_index, buffer);
        self.select_buffer(target_index);

        if let Some(buffer) = self.current_buffer.as_ref() {
            if buffer.syntax_definition.is_none() {
                let _ = self.update_current_syntax();
            }
        }
    }

    fn alloc_buffer_id(&mut self) -> usize {
        self.buffer_ida += 1;
        self.buffer_ida
    }

    fn select_buffer(&mut self, index: usize) -> bool {
        // 将当前buffer放回
        if let Some(buffer) = self.current_buffer.as_mut() {
            mem::swap(
                buffer,
                &mut self.buffers[self.current_buffer_index.unwrap()],
            );
        }

        // 选择新buffer
        if let Some(buffer) = self.buffers.get_mut(index) {
            self.current_buffer = Some(mem::take(buffer));
            self.current_buffer_index.replace(index);

            return true;
        }

        false
    }

    pub fn update_current_syntax(&mut self) -> Result<()> {
        let buffer = self
            .current_buffer
            .as_mut()
            .ok_or(ErrorKind::EmptyWorkspace)?;
        let definition = buffer
            .file_extension()
            .and_then(|ex| self.syntax_set.find_syntax_by_extension(&ex))
            .or_else(|| Some(self.syntax_set.find_syntax_plain_text()))
            .cloned();
        buffer.syntax_definition = definition;

        Ok(())
    }
}
