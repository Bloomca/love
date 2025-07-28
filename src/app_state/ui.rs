use std::path::PathBuf;
use std::collections::HashMap;
use std::fs;
use std::io;

use crossterm::{execute, cursor::{Show, MoveTo, SetCursorStyle}};

pub struct FileEntry {
    pub path: PathBuf
}

pub struct DirEntry {
    pub path: PathBuf,
    file_tree: HashMap<PathBuf, Vec<FileTreeEntry>>,
    expanded: bool
}

pub enum FileTreeEntry {
    File(FileEntry),
    Dir(DirEntry)
}

impl FileTreeEntry {
    pub fn new(path: PathBuf) -> FileTreeEntry {
        if path.is_dir() {
            FileTreeEntry::Dir(DirEntry { path, expanded: false, file_tree: HashMap::new() })
        } else {
            FileTreeEntry::File(FileEntry { path })
        }
    }
}

pub struct UIState {
    pub cursor_line: u16,
    pub cursor_column: u16,
    editor_offset_x: u16,
    editor_offset_y: u16,
    should_show_cursor: bool,
    prefix_len: u16,
}

impl UIState {
    pub fn new(prefix_len: u16) -> Self {
        UIState {
            cursor_line: 1,
            cursor_column: 1,
            editor_offset_x: 0,
            editor_offset_y: 0,
            should_show_cursor: false,
            // 1 character at the beginning, one space at the end
            prefix_len: prefix_len + 2
        }
    }

    pub fn set_editor_offset(&mut self, x: u16, y: u16) {
        // in theory, we only need to set this once. we might need to do again
        // if the file tree is resized, otherwise the offset should be steady.
        // for now, this should work
        if !self.should_show_cursor {
            self.editor_offset_x = x;
            self.editor_offset_y = y;
            self.should_show_cursor = true;
        }
    }

    pub fn show_cursor_if_needed(&mut self) {
        if self.should_show_cursor {
            let x = self.cursor_column + self.editor_offset_x + self.prefix_len;
            let y = self.cursor_line + self.editor_offset_y;
            let result = execute!(
                io::stdout(),
                MoveTo(x, y),
                SetCursorStyle::SteadyBlock,
                Show);

            match result {
                Ok(_) => {
                    // pass
                }
                Err(_) => {
                    // TODO: handle somehow
                }
            }
        }
    }

    pub fn cursor_move_left(&mut self) {
        if self.should_show_cursor && self.cursor_column > 1 {
            let new_value = self.cursor_column - 1;
            self.cursor_column = new_value;
        }
    }

    pub fn cursor_move_right(&mut self) {
        if self.should_show_cursor {
            self.cursor_column = self.cursor_column + 1;
        }
    }

    pub fn cursor_move_up(&mut self) {
        if self.should_show_cursor && self.cursor_line > 1 {
            let new_value = self.cursor_line - 1;
            self.cursor_line = new_value;
        }
    }

    pub fn cursor_move_down(&mut self) {
        if self.should_show_cursor {
            self.cursor_line = self.cursor_line + 1;
        }
    }
}