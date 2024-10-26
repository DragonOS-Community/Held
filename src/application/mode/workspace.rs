use std::{collections::HashSet, os::unix::fs::MetadataExt, path::PathBuf};

use crossterm::style::Color;
use error_chain::bail;
use held_core::{
    utils::{position::Position, range::Range},
    view::{colors::Colors, style::CharStyle},
};
use unicode_segmentation::UnicodeSegmentation;
use walkdir::{DirEntry, DirEntryExt, WalkDir};

use super::{ModeData, ModeRenderer};
use crate::{
    buffer::Buffer,
    errors::*,
    view::{monitor::Monitor, status_data::StatusLineData},
    workspace::Workspace,
};
pub struct WorkspaceModeData {
    path: PathBuf,
    selected_index: usize,
    selected_path: PathBuf,
    current_render_index: usize,
    max_index: usize,
    opened_dir_inos: HashSet<u64>,
    buffer_id: usize,
    pub prev_buffer_id: usize,
    highlight_ranges: Vec<(Range, CharStyle, Colors)>,
}

impl WorkspaceModeData {
    pub fn new(workspace: &mut Workspace, monitor: &mut Monitor) -> Result<WorkspaceModeData> {
        if !workspace.path.is_dir() {
            bail!("The workspace must be a directory!");
        }

        let mut opened_dir_inos = HashSet::new();
        opened_dir_inos.insert(workspace.path.metadata()?.ino());

        let prev_buffer_id = workspace.current_buffer.as_ref().unwrap().id()?;

        let buffer = Buffer::new();
        let buffer_id = workspace.add_buffer(buffer);
        monitor.init_buffer(workspace.current_buffer.as_mut().unwrap())?;

        workspace.select_buffer(prev_buffer_id);

        Ok(WorkspaceModeData {
            path: workspace.path.clone(),
            selected_index: 0,
            opened_dir_inos,
            buffer_id: buffer_id,
            prev_buffer_id,
            highlight_ranges: Vec::new(),
            current_render_index: 0,
            max_index: 0,
            selected_path: workspace.path.clone(),
        })
    }

    fn update_max_index(&mut self) {
        self.max_index = self.current_render_index - 1;
    }

    pub(super) fn render_workspace_tree(
        &mut self,
        workspace: &mut crate::workspace::Workspace,
        monitor: &mut crate::view::monitor::Monitor,
    ) -> Result<()> {
        if !workspace.select_buffer(self.buffer_id) {
            bail!("Not Workspace Buffer!");
        }

        self.current_render_index = 0;

        if let Some(ref mut buffer) = workspace.current_buffer {
            buffer.delete_range(Range::new(
                Position { line: 0, offset: 0 },
                Position {
                    line: buffer.line_count(),
                    offset: usize::MAX,
                },
            ));
            buffer.cursor.move_to(Position { line: 0, offset: 0 });
            self.highlight_ranges.resize(0, Default::default());
        }

        let mut depth = 0;
        let root = self.path.clone();
        self.render_dir(workspace, &root, &mut depth);

        if let Some(ref mut buffer) = workspace.current_buffer {
            buffer.cursor.move_to(Position {
                line: self.selected_index,
                offset: 0,
            });
            monitor.scroll_to_cursor(buffer)?;
        }

        let mut presenter = monitor.build_presenter()?;

        let buffer = workspace.current_buffer.as_ref().unwrap();
        let buffer_data = buffer.data();
        presenter.print_buffer(
            buffer,
            &buffer_data,
            &workspace.syntax_set,
            Some(&self.highlight_ranges),
            None,
        )?;

        let mode_name_data = StatusLineData {
            content: " WORKSPACE ".to_string(),
            color: Colors::Inverted,
            style: CharStyle::Bold,
        };
        let workspace_path_data = StatusLineData {
            content: format!(" {}", self.path.display()),
            color: Colors::Focused,
            style: CharStyle::Bold,
        };
        presenter.print_status_line(&[mode_name_data, workspace_path_data])?;
        presenter.present()?;

        monitor.terminal.set_cursor(None)?;
        monitor.terminal.present()?;

        self.update_max_index();
        Ok(())
    }

    fn entry_sort(a: &DirEntry, b: &DirEntry) -> std::cmp::Ordering {
        if a.file_type().is_dir() && b.file_type().is_dir() {
            return a.file_name().cmp(b.file_name());
        }

        if a.file_type().is_dir() {
            return std::cmp::Ordering::Less;
        } else if b.file_type().is_dir() {
            return std::cmp::Ordering::Greater;
        }

        a.file_name().cmp(b.file_name())
    }

    // 需要当前buffer为workspace buffer才能调用该方法
    fn render_dir(&mut self, workspace: &mut Workspace, root_path: &PathBuf, depth: &mut usize) {
        let walkdir = WalkDir::new(root_path)
            .max_depth(1)
            .sort_by(Self::entry_sort);

        let mut iter = walkdir.into_iter();

        if let Some(entry) = iter.next() {
            if let Ok(entry) = entry {
                let target_modified = workspace
                    .get_buffer_with_ino(entry.ino())
                    .map(|x| x.modified());

                let buffer = workspace.current_buffer.as_mut().unwrap();
                let ino = entry.ino();
                buffer.cursor.move_down();
                self.print_entry(
                    buffer,
                    entry,
                    *depth,
                    self.opened_dir_inos.contains(&ino),
                    target_modified,
                );
                *depth += 1;
                if !self.opened_dir_inos.contains(&ino) {
                    return;
                }
            }
        }

        for entry in iter {
            if let Ok(entry) = entry {
                self.render_entry(workspace, entry, depth);
            }
        }
    }

    fn render_entry(&mut self, workspace: &mut Workspace, entry: DirEntry, depth: &mut usize) {
        let target_modified = workspace
            .get_buffer_with_ino(entry.ino())
            .map(|x| x.modified());
        let buffer = workspace.current_buffer.as_mut().unwrap();
        if entry.file_type().is_dir() {
            if self.opened_dir_inos.contains(&entry.ino()) {
                self.render_dir(workspace, &entry.path().to_path_buf(), depth);
                *depth -= 1;
            } else {
                self.print_entry(buffer, entry, *depth, false, target_modified);
            }
        } else {
            self.print_entry(buffer, entry, *depth, false, target_modified);
        }
    }

    fn print_entry(
        &mut self,
        buffer: &mut Buffer,
        entry: DirEntry,
        depth: usize,
        is_open: bool,
        target_buffer_modified: Option<bool>,
    ) {
        let prefix = " ".repeat(depth)
            + if entry.file_type().is_dir() {
                if is_open {
                    "- "
                } else {
                    "+ "
                }
            } else {
                "| "
            };

        buffer.insert(&prefix);
        buffer.cursor.move_to(Position {
            line: buffer.cursor.line,
            offset: buffer.cursor.offset + prefix.graphemes(true).count(),
        });

        let start = buffer.cursor.position;
        let file_name = entry.file_name().to_str().unwrap_or_default();

        buffer.insert(file_name);
        buffer.cursor.move_to(Position {
            line: buffer.cursor.line,
            offset: buffer.cursor.offset + file_name.graphemes(true).count(),
        });
        let end = buffer.cursor.position;

        buffer.insert("\n");
        buffer.cursor.move_down();

        if self.selected_index == self.current_render_index {
            self.highlight_ranges.push((
                Range::new(start, end),
                CharStyle::Bold,
                Colors::CustomForeground(Color::Cyan),
            ));

            self.selected_path = entry.into_path();
        } else if let Some(modified) = target_buffer_modified {
            if modified {
                self.highlight_ranges.push((
                    Range::new(start, end),
                    CharStyle::Bold,
                    Colors::CustomForeground(Color::Yellow),
                ));
            } else {
                self.highlight_ranges.push((
                    Range::new(start, end),
                    CharStyle::Bold,
                    Colors::CustomForeground(Color::Green),
                ));
            }
        }

        self.current_render_index += 1;
    }

    /// 打开当前选择的节点，若返回true则表示打开的是文件而非dir，需要回退normal模式
    pub fn open(&mut self, workspace: &mut Workspace, monitor: &mut Monitor) -> Result<bool> {
        if self.selected_path.is_dir() {
            let ino = self.selected_path.metadata()?.ino();
            if self.opened_dir_inos.contains(&ino) {
                self.opened_dir_inos.remove(&ino);
            } else {
                self.opened_dir_inos.insert(ino);
            }

            Ok(false)
        } else {
            let buffer = Buffer::from_file(&self.selected_path)?;
            let id = workspace.add_buffer(buffer);
            workspace.select_buffer(id);
            monitor.init_buffer(workspace.current_buffer.as_mut().unwrap())?;
            self.prev_buffer_id = id;
            Ok(true)
        }
    }

    pub fn move_down(&mut self) {
        if self.selected_index == self.max_index {
            return;
        }
        self.selected_index += 1;
    }

    pub fn move_up(&mut self) {
        if self.selected_index == 0 {
            return;
        }
        self.selected_index -= 1;
    }
}

pub struct WorkspaceRender;

impl ModeRenderer for WorkspaceRender {
    fn render(
        workspace: &mut crate::workspace::Workspace,
        monitor: &mut crate::view::monitor::Monitor,
        mode: &mut super::ModeData,
    ) -> super::Result<()> {
        if let ModeData::Workspace(mode_data) = mode {
            return mode_data.render_workspace_tree(workspace, monitor);
        } else {
            bail!("Workspace mode cannot receive data other than WorkspaceModeData")
        }
    }
}
