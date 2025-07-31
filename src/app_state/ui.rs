use std::io;
use std::path::PathBuf;

use crossterm::{
    cursor::{MoveTo, SetCursorStyle, Show},
    execute,
};

pub struct FileEntry {
    pub path: PathBuf,
}

pub struct DirEntry {
    pub path: PathBuf,
    // file_tree: HashMap<PathBuf, Vec<FileTreeEntry>>,
    // expanded: bool,
}

pub enum FileTreeEntry {
    File(FileEntry),
    Dir(DirEntry),
}

impl FileTreeEntry {
    pub fn new(path: PathBuf) -> FileTreeEntry {
        if path.is_dir() {
            FileTreeEntry::Dir(DirEntry {
                path,
                // expanded: false,
                // file_tree: HashMap::new(),
            })
        } else {
            FileTreeEntry::File(FileEntry { path })
        }
    }
}

pub struct UIState {
    pub cursor_line: usize,
    pub cursor_column: usize,
    pub lines: Vec<Vec<char>>,
    editor_offset_x: usize,
    editor_offset_y: usize,

    /// There are multiple widgets which can be focused, plus we might
    /// not even have a valid editor open (e.g. if all files are closed)
    pub(super) should_show_cursor: bool,

    /// Each line is prefixed with the line number. To have a consistent
    /// prefix width, we take the highest number, take number of digits,
    /// and add `|` symbol and a space afterwards.
    prefix_len: usize,

    /// When we navigate using up/down directions, we ideally want to stay
    /// on the same column vertically. It is not always possible, because
    /// the next line might have fewer characters. But the line after might
    /// have enough! So in order to preserve that, we need to store the
    /// "target" column. It is invalidated the moment we navigate left or right,
    /// or insert a new character.
    pub(super) vertical_offset_target: usize,
}

impl UIState {
    pub fn new(prefix_len: usize, lines: Vec<Vec<char>>) -> Self {
        UIState {
            cursor_line: 1,
            cursor_column: 1,
            lines,
            editor_offset_x: 0,
            editor_offset_y: 0,
            should_show_cursor: false,
            // 1 character at the beginning, one space at the end
            prefix_len: prefix_len + 2,
            vertical_offset_target: 0,
        }
    }

    pub fn set_editor_offset(&mut self, x: usize, y: usize) {
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
                MoveTo(x as u16, y as u16),
                SetCursorStyle::SteadyBlock,
                Show
            );

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
}
