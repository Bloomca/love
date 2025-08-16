use crossterm::clipboard::CopyToClipboard;
use crossterm::execute;
use std::io;

use crate::app_state::editor::UIState;
use crate::app_state::undo_redo::{UndoAction, UndoRedo};

impl UIState {
    pub fn handle_paste(&mut self, data: String, undo_redo: &mut UndoRedo) {
        if data.is_empty() {
            return;
        }

        self.vertical_offset_target = 0;

        self.delete_selection();

        let start_position = (self.cursor_line, self.cursor_column);

        // in my iTerm on macOS, newlines are replaced by `\r` by default
        // to be safe, we normalize all possible line endings into '\n'
        let normalized = data.replace("\r\n", "\n").replace("\r", "\n");
        let total_pasted_lines = normalized.lines().count();
        let lines: Vec<(usize, Vec<char>)> = normalized
            .lines()
            .map(|s| s.chars().collect())
            .enumerate()
            .collect();

        let prefix_len = Self::get_common_whitespaces_prefix(&lines);

        let prev_line_whitespaces = match self.lines.get(self.cursor_line - 1) {
            Some(line) => Self::calculate_whitespace_num(line),
            None => 0,
        };

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
                // 1. get the previous line and calculate whitespaces (done before)
                // 2. create a new line with those whitespaces and add new data
                // 3. append that new line in the new index (self.cursor_line + i)
                // 4. if that is the last line (i + 1 == total_pasted_lines), put cursor at the end
                // let whitespaces_num = Self::calculate_whitespace_num(&pasted_line);
                let prefixed_line: Vec<char> = if prev_line_whitespaces >= prefix_len {
                    vec![' '; prev_line_whitespaces - prefix_len]
                        .into_iter()
                        .chain(pasted_line)
                        .collect()
                } else {
                    pasted_line
                        .iter()
                        .cloned()
                        .skip(prefix_len - prev_line_whitespaces)
                        .collect()
                };

                let old_cursor_line = self.cursor_line;
                if total_pasted_lines == i + 1 {
                    self.cursor_line += i;
                    self.cursor_column = prefixed_line.len() + 1;
                }

                self.lines.insert(old_cursor_line - 1 + i, prefixed_line);
            }
        }

        let end_position = (self.cursor_line, self.cursor_column);
        undo_redo.add_undo_action(UndoAction::Paste(start_position, end_position));
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

    fn get_common_whitespaces_prefix(data: &[(usize, Vec<char>)]) -> usize {
        let mut final_whitespaces = 0;
        for (i, line) in data {
            if line.is_empty() {
                continue;
            }

            let whitespaces = Self::calculate_whitespace_num(line);

            if *i == 0 {
                if whitespaces != 0 {
                    return whitespaces;
                } else {
                    final_whitespaces = whitespaces;
                }
            } else if whitespaces == 0 {
                return whitespaces;
            } else if final_whitespaces == 0 || whitespaces < final_whitespaces {
                final_whitespaces = whitespaces;
            }
        }

        final_whitespaces
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn correctly_calculates_whitespaces_without_first_line() {
        let data = vec![
            (0, vec!['c', 'h', 'a', 'r']),
            (1, vec![' ', ' ', 'l', 'i', 'n', 'e']),
            (2, vec![' ', ' ', 'l', 'i', 'n', 'e']),
        ];

        assert_eq!(UIState::get_common_whitespaces_prefix(&data), 2);
    }

    #[test]
    fn respects_whitespaces_first_line() {
        let data = vec![
            (0, vec![' ', ' ', 'c', 'h', 'a', 'r']),
            (1, vec![' ', ' ', ' ', ' ', 'l', 'i', 'n', 'e']),
            (2, vec![' ', ' ', ' ', ' ', 'l', 'i', 'n', 'e']),
        ];

        assert_eq!(UIState::get_common_whitespaces_prefix(&data), 2);
    }

    #[test]
    fn calculates_correctly_with_different_prefix() {
        let data = vec![
            (0, vec!['c', 'h', 'a', 'r']),
            (1, vec![' ', ' ', ' ', ' ', 'l', 'i', 'n', 'e']),
            (2, vec![' ', ' ', 'l', 'i', 'n', 'e']),
        ];

        assert_eq!(UIState::get_common_whitespaces_prefix(&data), 2);
    }

    #[test]
    fn calculates_correctly_with_zero_prefix() {
        let data = vec![
            (0, vec!['c', 'h', 'a', 'r']),
            (1, vec![' ', ' ', ' ', ' ', 'l', 'i', 'n', 'e']),
            (2, vec![' ', ' ', 'l', 'i', 'n', 'e']),
            (3, vec!['l', 'i', 'n', 'e']),
        ];

        assert_eq!(UIState::get_common_whitespaces_prefix(&data), 0);
    }

    #[test]
    fn handles_empty_lines_correctly() {
        let data = vec![
            (0, vec!['c', 'h', 'a', 'r']),
            (1, vec![' ', ' ', ' ', ' ', 'l', 'i', 'n', 'e']),
            (3, vec![]),
            (2, vec![' ', ' ', 'l', 'i', 'n', 'e']),
        ];

        assert_eq!(UIState::get_common_whitespaces_prefix(&data), 2);
    }
}
