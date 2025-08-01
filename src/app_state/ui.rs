use std::collections::HashMap;
use std::io;
use std::path::PathBuf;

use crossterm::{
    cursor::{MoveTo, SetCursorStyle, Show},
    event::KeyModifiers,
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

enum SelectionType {
    Line,
    To(usize),
    From(usize),
    Range(usize, usize),
}

struct Selection {
    start: (usize, usize),
    end: (usize, usize),
    cache: HashMap<usize, SelectionType>,
}

impl Selection {
    fn new(current_cursor_line: usize, current_cursor_col: usize) -> Self {
        Selection {
            start: (current_cursor_col, current_cursor_line),
            end: (current_cursor_col, current_cursor_line),
            cache: HashMap::new(),
        }
    }

    fn set_end(&mut self, current_cursor_line: usize, current_cursor_col: usize) {
        self.end = (current_cursor_col, current_cursor_line);

        let (start_line, start_column) = self.start;

        let direction_forward = (current_cursor_line, current_cursor_col) > (start_line, start_column);

        if start_line == current_cursor_line {
            let entry = if direction_forward {
                SelectionType::Range(
                    start_column.min(current_cursor_col),
                    start_column.max(current_cursor_col - 1),
                )
            } else {
                SelectionType::Range(
                    start_column.min(current_cursor_col - 1),
                    start_column.max(current_cursor_col),
                )
            };

            self.cache
                .entry(start_line)
                .insert_entry(entry);
        } else {
            let min_line = start_line.min(current_cursor_line);
            let max_line = start_line.max(current_cursor_line);

            let (min_line_column, max_line_column) = if start_line < current_cursor_line {
                (start_column, current_cursor_col)
            } else {
                (current_cursor_col, start_column)
            };

            for line_num in min_line..=max_line {
                let entry = match line_num {
                    _ if line_num == min_line => SelectionType::From(min_line_column),
                    _ if line_num == max_line => SelectionType::To(max_line_column),
                    _ => SelectionType::Line,
                };

                self.cache.entry(line_num).insert_entry(entry);
            }
        }
    }

    fn is_char_selected(&self, line: usize, column: usize) -> bool {
        match self.cache.get(&line) {
            Some(selection_type) => match selection_type {
                SelectionType::Line => true,
                SelectionType::To(selected_col) => selected_col >= &column,
                SelectionType::From(selected_col) => selected_col <= &column,
                SelectionType::Range(min_col, max_col) => &column >= min_col && &column <= max_col,
            },
            None => false,
        }
    }
}

pub struct UIState {
    pub cursor_line: usize,
    pub cursor_column: usize,
    pub lines: Vec<Vec<char>>,
    /// horizontal offset on the screen; needed to place the cursor
    editor_offset_x: usize,
    /// vertical offset on the screen; needed to place the cursor
    editor_offset_y: usize,
    /// we don't want to process more lines than visible
    pub editor_lines_num: usize,
    pub editor_scroll_offset: usize,

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

    selection: Option<Selection>,
}

impl UIState {
    pub fn new(prefix_len: usize, lines: Vec<Vec<char>>) -> Self {
        UIState {
            cursor_line: 1,
            cursor_column: 1,
            lines,
            editor_offset_x: 0,
            editor_offset_y: 0,
            // big random number
            editor_lines_num: 1000,
            editor_scroll_offset: 0,
            should_show_cursor: false,
            // 1 character at the beginning, one space at the end
            prefix_len: prefix_len + 2,
            vertical_offset_target: 0,
            selection: None,
        }
    }

    pub fn set_editor_offset(&mut self, x: usize, y: usize, height: usize) {
        // in theory, we only need to set this once. we might need to do again
        // if the file tree is resized, otherwise the offset should be steady.
        // for now, this should work
        if !self.should_show_cursor {
            self.editor_offset_x = x;
            self.editor_offset_y = y;
            // 1 for the border at the top, 1 for the border at the bottom
            self.editor_lines_num = height - 2;
            self.should_show_cursor = true;
        }
    }

    pub fn show_cursor_if_needed(&mut self) {
        if self.should_show_cursor {
            let x = self.cursor_column + self.editor_offset_x + self.prefix_len;
            let y = self.cursor_line + self.editor_offset_y - self.editor_scroll_offset;
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

    /// Start a new selection if necessary, clear existing one if necessary or do nothing if one exists
    pub fn start_selection(&mut self, modifiers: &KeyModifiers) {
        if modifiers.contains(KeyModifiers::SHIFT) {
            if self.selection.is_none() {
                self.selection = Some(Selection::new(self.cursor_line, self.cursor_column));
            }
        } else {
            self.selection = None;
        }
    }

    /// This function only adjust existing selection, but it doesn't clear it based on keyboard modifiers.
    /// However, it will clear the selection if the cursor is at the same spot as it started.
    pub fn adjust_selection(&mut self) {
        if let Some(selection) = &mut self.selection {
            if selection.start.0 == self.cursor_line && selection.start.1 == self.cursor_column {
                self.selection = None;
            } else {
                selection.set_end(self.cursor_line, self.cursor_column);
            }
        }
    }

    pub fn is_char_selected(&self, line: usize, column: usize) -> bool {
        match &self.selection {
            Some(selection) => selection.is_char_selected(line, column),
            None => false,
        }
    }
}
