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
    cursor_line: u16,
    cursor_column: u16,
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

pub struct AppState {
    /// Directory of the entire project, can only be a single one
    /// It is either opened directly (like `love .`) or it is calculated
    /// based on the provided path
    pub working_directory: PathBuf,
    pub file_tree: HashMap<PathBuf, Vec<FileTreeEntry>>,
    pub lines: Vec<Vec<char>>,
    pub ui_state: UIState,
}

impl AppState {
    pub fn new(file_content: String, working_directory: PathBuf) -> AppState {
        let lines_number = file_content.lines().count();
        let lines: Vec<Vec<char>> = file_content.lines().map(|s| s.chars().collect()).collect();
        let mut app_state: AppState = AppState {
            working_directory,
            file_tree: HashMap::new(),
            lines,
            // this is extremely safe, it needs to have 65535 digits to overflow
            ui_state: UIState::new(lines_number.to_string().len() as u16)
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

    pub fn insert_character(&mut self, character: char) {
        let result = self.lines.get_mut((self.ui_state.cursor_line - 1) as usize);

        match result {
            Some(line) => {
                let index = (self.ui_state.cursor_column - 1) as usize;
                if index <= line.len() {
                    line.insert(index, character);
                    self.ui_state.cursor_move_right();
                }
            }
            None => {
                // ????
            }
        }
    }

    pub fn remove_previous_character(&mut self) {
        let index = (self.ui_state.cursor_column - 1) as usize;

        let result = self.lines.get_mut((self.ui_state.cursor_line - 1) as usize);

        match result {
            Some(line) => {
                if index == 0 {
                    // we need to prepend current line to the previous one
                    return;
                } else if index <= line.len() {
                    line.remove(index - 1);
                    self.ui_state.cursor_move_left();
                }
            }
            None => {
                // ????
            }
        }
    }

    // if `delete` is pressed, we delete the next character
    pub fn remove_next_character(&mut self) {
        let index = (self.ui_state.cursor_column - 1) as usize;

        let result = self.lines.get_mut((self.ui_state.cursor_line - 1) as usize);

        match result {
            Some(line) => {
                let line_len = line.len();
                if index == line_len {
                    // we need to get the next line and append it to the current line
                } else if line_len > 0 && index <= line_len - 1 {
                    line.remove(index);
                }
            }
            None => {
                // ????
            }
        }
    }
}