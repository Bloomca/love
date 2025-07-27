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
    fn new(path: PathBuf) -> FileTreeEntry {
        if path.is_dir() {
            FileTreeEntry::Dir(DirEntry { path, expanded: false, file_tree: HashMap::new() })
        } else {
            FileTreeEntry::File(FileEntry { path })
        }
    }
}

pub struct UIState {
    cursor_x: u16,
    cursor_y: u16,
    editor_offset_x: u16,
    editor_offset_y: u16,
    should_show_cursor: bool
}

impl UIState {
    pub fn new() -> Self {
        UIState {
            cursor_x: 0,
            cursor_y: 0,
            editor_offset_x: 0,
            editor_offset_y: 0,
            should_show_cursor: false
        }
    }

    pub fn set_editor_offset(&mut self, x: u16, y: u16) {
        // in theory, we only need to set this once. we might need to do again
        // if the file tree is resized, otherwise the offset should be steady.
        // for now, this should work
        if !self.should_show_cursor {
            self.editor_offset_x = x;
            self.editor_offset_y = y;
            self.cursor_x = self.editor_offset_x + 1;
            self.cursor_y = self.editor_offset_y + 1;
            self.should_show_cursor = true;
        }
    }

    pub fn show_cursor_if_needed(&mut self) {
        if self.should_show_cursor {
            let result = execute!(
                io::stdout(),
                MoveTo(self.cursor_x, self.cursor_y),
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
        if self.should_show_cursor && self.cursor_x > 0 {
            let new_value = self.cursor_x - 1;

            if new_value > self.editor_offset_x {
                self.cursor_x = new_value;
            }
        }
    }

    pub fn cursor_move_right(&mut self) {
        if self.should_show_cursor {
            self.cursor_x = self.cursor_x + 1;
        }
    }

    pub fn cursor_move_up(&mut self) {
        if self.should_show_cursor && self.cursor_y > 0 {
            let new_value = self.cursor_y - 1;

            if new_value > self.editor_offset_y {
                self.cursor_y = new_value;
            }
        }
    }

    pub fn cursor_move_down(&mut self) {
        if self.should_show_cursor {
            self.cursor_y = self.cursor_y + 1;
        }
    }
}

pub struct AppState {
    /// Directory of the entire project, can only be a single one
    /// It is either opened directly (like `love .`) or it is calculated
    /// based on the provided path
    pub working_directory: PathBuf,
    pub file_tree: HashMap<PathBuf, Vec<FileTreeEntry>>,
    pub file_content: String,
    pub ui_state: UIState,
}

impl AppState {
    pub fn new(file_content: String, working_directory: PathBuf) -> AppState {
        let mut app_state: AppState = AppState {
            working_directory,
            file_tree: HashMap::new(),
            file_content,
            ui_state: UIState::new()
        };

        app_state.read_directory(app_state.working_directory.clone());

        app_state
    }

    pub fn read_directory(&mut self, path: PathBuf) {
        // TODO: save the error as a directory state
        let dir_entries = fs::read_dir(&path).expect("Could not read directory");

        let values: Vec<FileTreeEntry> = dir_entries
            .filter_map(|result| result.ok())
            .map(|entry| FileTreeEntry::new(entry.path()))
            .collect();

        self.file_tree.insert(path, values);
    }
}