use crate::app_state::app::Config;
use crate::app_state::editor::UIState;
use crate::app_state::undo_redo::UndoRedo;

impl UIState {
    pub fn handle_tab_key(&mut self, config: &Config, undo_redo: &mut UndoRedo) {
        self.vertical_offset_target = 0;
        if config.tabs_to_spaces {
            if let Some(selection) = &mut self.selection {
                let (start_line, start_column) = selection.start;
                let (end_line, end_column) = selection.end;

                for line_num in start_line.min(end_line)..=start_line.max(end_line) {
                    if let Some(line) = self.lines.get_mut(line_num - 1) {
                        line.splice(0..0, vec![' '; config.whitespaces_amount]);
                    }
                }

                selection.start = (start_line, start_column + config.whitespaces_amount);
                selection.set_end(end_line, end_column + config.whitespaces_amount);
                self.cursor_column += config.whitespaces_amount;
            } else {
                for _ in 0..config.whitespaces_amount {
                    self.insert_character(' ', undo_redo);
                }
            }
        } else {
            self.insert_character('\t', undo_redo);
        }
    }

    pub fn calculate_whitespace_num(line: &[char]) -> usize {
        line.iter().take_while(|c| c.is_whitespace()).count()
    }

    pub fn handle_backtab_key(&mut self, config: &Config) {
        if config.tabs_to_spaces {
            if let Some(selection) = &mut self.selection {
                let (start_line, start_column) = selection.start;
                let (end_line, end_column) = selection.end;
                let mut start_line_removed = 0;
                let mut end_line_removed = 0;

                for line_num in start_line.min(end_line)..=start_line.max(end_line) {
                    if let Some(line) = self.lines.get_mut(line_num - 1) {
                        let whitespaces = Self::calculate_whitespace_num(line);
                        let remove_num = whitespaces.min(config.whitespaces_amount);

                        if remove_num != 0 {
                            line.drain(0..remove_num);

                            if line_num == start_line {
                                start_line_removed = remove_num;
                            }

                            if line_num == self.cursor_line {
                                if self.cursor_column <= remove_num {
                                    self.cursor_column = 1;
                                } else {
                                    self.cursor_column -= remove_num;
                                }
                            }

                            if line_num == end_line {
                                end_line_removed = remove_num;
                            }
                        }
                    }
                }

                if start_line_removed != 0 {
                    if start_column <= start_line_removed {
                        selection.start = (start_line, 1);
                    } else {
                        selection.start = (start_line, start_column - start_line_removed);
                    }
                }

                if end_line_removed != 0 {
                    if end_column <= end_line_removed {
                        selection.set_end(end_line, 1);
                    } else {
                        selection.set_end(end_line, end_column - end_line_removed);
                    }
                } else if start_line_removed != 0 {
                    selection.set_end(end_line, end_column);
                }
            } else if let Some(line) = self.lines.get_mut(self.cursor_line - 1) {
                let whitespaces = Self::calculate_whitespace_num(line);
                let remove_num = whitespaces.min(config.whitespaces_amount);

                if remove_num != 0 {
                    line.drain(0..remove_num);

                    if self.cursor_column <= remove_num {
                        self.cursor_column = 1;
                    } else {
                        self.cursor_column -= remove_num;
                    }
                }
            }
        } else {
            // pass for now, handle tabs later
        }
    }
}

#[cfg(test)]
mod tests {
    // use super::*;

    // TODO: write tests for all tab actions
}
