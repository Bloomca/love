use crossterm::clipboard::CopyToClipboard;
use crossterm::execute;
use std::io;

use crate::app_state::editor::UIState;

impl UIState {
    pub fn handle_paste(&mut self, data: String) {
        self.vertical_offset_target = 0;

        self.delete_selection();

        // in my iTerm on macOS, newlines are replaced by `\r` by default
        // to be safe, we normalize all possible line endings into '\n'
        let normalized = data.replace("\r\n", "\n").replace("\r", "\n");
        let total_pasted_lines = normalized.lines().count();
        let lines: Vec<(usize, Vec<char>)> = normalized
            .lines()
            .map(|s| s.chars().collect())
            .enumerate()
            .collect();

        for (i, pasted_line) in lines {
            if i == 0 {
                // 1. get the current line
                // 2. append the new line to it
                // 3. if there is only one line, move cursor to it
                if let Some(line) = self.lines.get_mut(self.cursor_line - 1) {
                    // we need to calculate index first, as we might change cursor next
                    let index = self.cursor_column - 1;
                    if total_pasted_lines == 1 {
                        self.cursor_column += pasted_line.len();
                    }
                    line.splice(index..index, pasted_line);
                }
            } else {
                // 1. get the previous line and calculate whitespaces
                // 2. create a new line with those whitespaces and add new data
                // 3. append that new line in the new index (self.cursor_line + i)
                // 4. if that is the last line (i + 1 == total_pasted_lines), put cursor at the end
                if let Some(prev_line) = self.lines.get(self.cursor_line - 1 + i - 1) {
                    let prev_line_whitespaces = Self::calculate_whitespace_num(prev_line);

                    let prefixed_line: Vec<char> = vec![' '; prev_line_whitespaces]
                        .into_iter()
                        .chain(pasted_line)
                        .collect();

                    if total_pasted_lines == i + 1 {
                        self.cursor_line += i - 1;
                        self.cursor_column = prefixed_line.len();
                    }

                    self.lines.insert(self.cursor_line - 1 + i, prefixed_line);
                }
            }
        }
    }

    pub fn handle_copy(&self) {
        // 1. read selection, if none do nothing
        // 2. copy selection (join all strings with '\n' character)
        // 3. Execute a crossterm command: https://docs.rs/crossterm/0.29.0/crossterm/clipboard/struct.CopyToClipboard.html
        if let Some(selection) = &self.selection {
            let (start_line, start_column) = selection.start;
            let (end_line, end_column) = selection.end;

            if start_line == end_line {
                if let Some(line) = self.lines.get(start_line - 1) {
                    let start_index = start_column.min(end_column) - 1;
                    let end_index = start_column.max(end_column);
                    let text = line[start_index..end_index].iter().collect::<String>();
                    Self::execute_terminal_copy(text);
                }
            } else {
                let mut result = vec![];

                let start_line_index = start_line.min(end_line);
                let end_line_index = start_line.max(end_line);
                let first_line_column = if start_line < end_line {
                    start_column
                } else {
                    end_column
                };
                let last_line_column = if start_line > end_line {
                    start_column
                } else {
                    end_column
                };
                for i in start_line_index..=end_line_index {
                    if i != start_line {
                        result.push('\n');
                    }

                    if i == start_line_index {
                        if let Some(line) = self.lines.get(i - 1) {
                            let start_index = first_line_column - 1;
                            result.extend_from_slice(&line[start_index..]);
                        }
                    }

                    if i == end_line_index {
                        if let Some(line) = self.lines.get(i - 1) {
                            let end_index = last_line_column - 1;
                            result.extend_from_slice(&line[..end_index]);
                        }
                    }

                    if i != start_line_index && i != end_line_index {
                        if let Some(line) = self.lines.get(i - 1) {
                            result.extend_from_slice(line);
                        }
                    }
                }

                let text = result.iter().collect::<String>();
                Self::execute_terminal_copy(text);
            }
        } else {
            // do nothing
        }
    }

    fn execute_terminal_copy(data: String) {
        match execute!(io::stdout(), CopyToClipboard::to_clipboard_from(data)) {
            Ok(_) => {
                // pass
            }
            Err(_) => {
                // pass
            }
        }
    }
}
