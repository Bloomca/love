use super::editor::UIState;
use crossterm::event::KeyModifiers;

impl UIState {
    pub fn cursor_move_left(&mut self, modifiers: &KeyModifiers) {
        if self.should_show_cursor {
            self.vertical_offset_target = 0;

            self.start_selection(modifiers);

            if self.cursor_column == 1 {
                if self.cursor_line == 1 {
                    // we already at the beginning of the file, nothing to do
                    return;
                } else {
                    let new_cursor_line = self.cursor_line - 1;
                    self.cursor_line = new_cursor_line;

                    let line_len = self.get_line_len(new_cursor_line - 1);
                    self.cursor_column = line_len + 1;

                    self.handle_cursor_scrolling();
                }
            } else {
                let new_value = self.cursor_column - 1;
                self.cursor_column = new_value;
            }

            self.adjust_selection();
        }
    }

    pub fn cursor_move_right(&mut self, modifiers: &KeyModifiers) {
        if self.should_show_cursor {
            self.vertical_offset_target = 0;

            self.start_selection(modifiers);

            let line_len = self.get_line_len(self.cursor_line - 1);
            if self.cursor_column > line_len {
                if self.cursor_line >= self.lines.len() {
                    // we are on the last line, do nothing
                    return;
                } else {
                    // need to move to the next line
                    self.cursor_column = 1;
                    self.cursor_line += 1;

                    self.handle_cursor_scrolling();
                }
            } else {
                self.cursor_column += 1;
            }

            self.adjust_selection();
        }
    }

    pub fn cursor_move_up(&mut self, modifiers: &KeyModifiers) {
        if self.should_show_cursor {
            self.start_selection(modifiers);

            if self.cursor_line == 1 {
                if self.vertical_offset_target == 0 {
                    self.vertical_offset_target = self.cursor_column;
                }
                self.cursor_column = 1;
            } else {
                self.cursor_line -= 1;
                self.adjust_cursor_column_after_vertical_nav();

                self.handle_cursor_scrolling();
            }

            self.adjust_selection();
        }
    }

    pub fn cursor_move_down(&mut self, modifiers: &KeyModifiers) {
        if self.should_show_cursor {
            self.start_selection(modifiers);

            if self.cursor_line == self.lines.len() {
                if self.vertical_offset_target == 0 {
                    self.vertical_offset_target = self.cursor_column;
                }

                let line_len = self.get_line_len(self.cursor_line - 1);
                self.cursor_column = line_len + 1;
            } else {
                self.cursor_line += 1;
                self.adjust_cursor_column_after_vertical_nav();

                self.handle_cursor_scrolling();
            }

            self.adjust_selection();
        }
    }

    pub fn cursor_move_line_start(&mut self, modifiers: &KeyModifiers) {
        if self.should_show_cursor {
            self.start_selection(modifiers);

            self.vertical_offset_target = 0;
            // TODO: implement going to the beginning of the line respecting white spaces
            self.cursor_column = 1;

            self.adjust_selection();
        }
    }

    pub fn cursor_move_line_end(&mut self, modifiers: &KeyModifiers) {
        if self.should_show_cursor {
            self.start_selection(modifiers);

            self.vertical_offset_target = 0;
            let line_len = self.get_line_len(self.cursor_line - 1);
            self.cursor_column = line_len + 1;

            self.adjust_selection();
        }
    }

    pub fn handle_cursor_scrolling(&mut self) {
        if self.cursor_line < self.editor_scroll_offset + 1 {
            self.editor_scroll_offset = self.cursor_line - 1;
        } else if self.cursor_line > self.editor_scroll_offset + self.editor_lines_num {
            self.editor_scroll_offset = self.cursor_line - self.editor_lines_num;
        }
    }

    fn adjust_cursor_column_after_vertical_nav(&mut self) {
        // if it is not 0, it means we were pressing up/down before
        if self.vertical_offset_target == 0 {
            self.vertical_offset_target = self.cursor_column;
        }

        let line_len = self.get_line_len(self.cursor_line - 1);

        if self.vertical_offset_target < line_len + 1 {
            self.cursor_column = self.vertical_offset_target;
        } else {
            self.cursor_column = line_len + 1;
        }
    }

    pub(super) fn get_line_len(&self, index: usize) -> usize {
        // this will break if index is higher than 65535, which is not impossible
        // TODO: switch to `usize` everywhere, and only use `u16` for actual terminal
        match self.lines.get(index) {
            Some(line) => line.len(),
            None => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cannot_move_negative() {
        let lines = vec![
            vec!['H', 'e', 'l', 'l', 'o', ' ', 'w', 'o', 'r', 'l', 'd', '!'],
            vec![],
            vec!['D', 'e', 's', 'c', 'r', 'i', 'p', 't', 'i', 'o', 'n'],
        ];
        let mut ui_state = UIState::new(5, lines);
        ui_state.set_editor_offset(30, 0, 50);

        ui_state.cursor_move_left(&KeyModifiers::NONE);
        ui_state.cursor_move_left(&KeyModifiers::NONE);
        ui_state.cursor_move_left(&KeyModifiers::NONE);

        ui_state.cursor_move_up(&KeyModifiers::NONE);
        ui_state.cursor_move_up(&KeyModifiers::NONE);
        ui_state.cursor_move_up(&KeyModifiers::NONE);

        assert_eq!(ui_state.cursor_column, 1);
        assert_eq!(ui_state.cursor_line, 1);
    }

    #[test]
    fn moves_correctly() {
        let lines = vec![
            vec!['H', 'e', 'l', 'l', 'o', ' ', 'w', 'o', 'r', 'l', 'd', '!'],
            vec![],
            vec!['D', 'e', 's', 'c', 'r', 'i', 'p', 't', 'i', 'o', 'n'],
        ];
        let mut ui_state = UIState::new(5, lines);
        ui_state.set_editor_offset(30, 0, 50);

        ui_state.cursor_move_right(&KeyModifiers::NONE);
        ui_state.cursor_move_right(&KeyModifiers::NONE);

        assert_eq!(ui_state.cursor_column, 3);

        ui_state.cursor_move_left(&KeyModifiers::NONE);

        assert_eq!(ui_state.cursor_column, 2);

        ui_state.cursor_move_down(&KeyModifiers::NONE);
        ui_state.cursor_move_down(&KeyModifiers::NONE);

        assert_eq!(ui_state.cursor_line, 3);

        ui_state.cursor_move_up(&KeyModifiers::NONE);

        assert_eq!(ui_state.cursor_line, 2);
    }

    #[test]
    fn move_right_goes_to_next_line() {
        let lines = vec![
            vec!['H', 'e', 'l', 'l', 'o'],
            vec![],
            vec!['D', 'e', 's', 'c', 'r', 'i', 'p', 't', 'i', 'o', 'n'],
        ];
        let mut ui_state = UIState::new(5, lines);
        ui_state.set_editor_offset(30, 0, 50);

        for _ in 0..5 {
            ui_state.cursor_move_right(&KeyModifiers::NONE);
        }

        assert_eq!(ui_state.cursor_column, 6);
        assert_eq!(ui_state.cursor_line, 1);

        // should move to the next line
        ui_state.cursor_move_right(&KeyModifiers::NONE);

        assert_eq!(ui_state.cursor_column, 1);
        assert_eq!(ui_state.cursor_line, 2);

        // should move to the next line
        ui_state.cursor_move_right(&KeyModifiers::NONE);

        assert_eq!(ui_state.cursor_column, 1);
        assert_eq!(ui_state.cursor_line, 3);

        // there is no lines after this one
        for _ in 0..15 {
            ui_state.cursor_move_right(&KeyModifiers::NONE);
        }

        assert_eq!(ui_state.cursor_column, 12);
        assert_eq!(ui_state.cursor_line, 3);
    }

    #[test]
    fn move_left_goes_to_previous_line() {
        let lines = vec![
            vec!['H', 'e', 'l', 'l', 'o'],
            vec![],
            vec!['D', 'e', 's', 'c', 'r', 'i', 'p', 't', 'i', 'o', 'n'],
        ];
        let mut ui_state = UIState::new(5, lines);
        ui_state.set_editor_offset(30, 0, 50);

        ui_state.cursor_move_down(&KeyModifiers::NONE);
        ui_state.cursor_move_down(&KeyModifiers::NONE);

        assert_eq!(ui_state.cursor_column, 1);
        assert_eq!(ui_state.cursor_line, 3);

        // should go to the previous line
        ui_state.cursor_move_left(&KeyModifiers::NONE);

        assert_eq!(ui_state.cursor_column, 1);
        assert_eq!(ui_state.cursor_line, 2);

        // should go to the previous line
        ui_state.cursor_move_left(&KeyModifiers::NONE);

        assert_eq!(ui_state.cursor_column, 6);
        assert_eq!(ui_state.cursor_line, 1);
    }

    #[test]
    fn move_up_down_handles_end_of_file() {
        let lines = vec![
            vec!['H', 'e', 'l', 'l', 'o'],
            vec![],
            vec!['D', 'e', 's', 'c', 'r', 'i', 'p', 't', 'i', 'o', 'n'],
        ];
        let mut ui_state = UIState::new(5, lines);
        ui_state.set_editor_offset(30, 0, 50);

        ui_state.cursor_move_right(&KeyModifiers::NONE);
        ui_state.cursor_move_right(&KeyModifiers::NONE);

        assert_eq!(ui_state.cursor_column, 3);
        assert_eq!(ui_state.cursor_line, 1);

        ui_state.cursor_move_up(&KeyModifiers::NONE);

        assert_eq!(ui_state.cursor_column, 1);
        assert_eq!(ui_state.cursor_line, 1);

        ui_state.cursor_move_down(&KeyModifiers::NONE);

        assert_eq!(ui_state.cursor_column, 1);
        assert_eq!(ui_state.cursor_line, 2);

        ui_state.cursor_move_down(&KeyModifiers::NONE);

        assert_eq!(ui_state.cursor_column, 3);
        assert_eq!(ui_state.cursor_line, 3);

        ui_state.cursor_move_down(&KeyModifiers::NONE);

        assert_eq!(ui_state.cursor_column, 12);
        assert_eq!(ui_state.cursor_line, 3);

        ui_state.cursor_move_up(&KeyModifiers::NONE);
        ui_state.cursor_move_up(&KeyModifiers::NONE);

        assert_eq!(ui_state.cursor_column, 3);
        assert_eq!(ui_state.cursor_line, 1);
    }

    #[test]
    fn move_start_end_line_correctly() {
        let lines = vec![
            vec!['H', 'e', 'l', 'l', 'o'],
            vec![],
            vec!['D', 'e', 's', 'c', 'r', 'i', 'p', 't', 'i', 'o', 'n'],
        ];

        let mut ui_state = UIState::new(5, lines);
        ui_state.set_editor_offset(30, 0, 50);

        ui_state.cursor_move_line_end(&KeyModifiers::NONE);
        assert_eq!(ui_state.cursor_column, 6);
        assert_eq!(ui_state.cursor_line, 1);

        ui_state.cursor_move_down(&KeyModifiers::NONE);
        ui_state.cursor_move_down(&KeyModifiers::NONE);

        ui_state.cursor_move_line_start(&KeyModifiers::NONE);
        assert_eq!(ui_state.cursor_column, 1);
        assert_eq!(ui_state.cursor_line, 3);

        ui_state.cursor_move_line_end(&KeyModifiers::NONE);
        assert_eq!(ui_state.cursor_column, 12);
        assert_eq!(ui_state.cursor_line, 3);
    }

    #[test]
    fn moving_scrolls_correctly() {
        let lines = vec![
            vec!['H', 'e', 'l', 'l', 'o'],
            vec![],
            vec!['D', 'e', 's', 'c', 'r', 'i', 'p', 't', 'i', 'o', 'n'],
            vec!['T', 'i', 't', 'l', 'e'],
            vec!['L', 'i', 'n', 'e'],
            vec![],
            vec!['E', 'n', 'd'],
        ];
        let mut ui_state = UIState::new(5, lines);
        ui_state.set_editor_offset(30, 0, 5);

        assert_eq!(ui_state.editor_lines_num, 3);

        ui_state.cursor_move_down(&KeyModifiers::NONE);

        assert_eq!(ui_state.editor_scroll_offset, 0);

        ui_state.cursor_move_down(&KeyModifiers::NONE);

        assert_eq!(ui_state.editor_scroll_offset, 0);

        ui_state.cursor_move_down(&KeyModifiers::NONE);

        assert_eq!(ui_state.editor_scroll_offset, 1);

        ui_state.cursor_move_line_end(&KeyModifiers::NONE);
        ui_state.cursor_move_right(&KeyModifiers::NONE);

        assert_eq!(ui_state.editor_scroll_offset, 2);

        ui_state.cursor_move_up(&KeyModifiers::NONE);
        ui_state.cursor_move_up(&KeyModifiers::NONE);

        assert_eq!(ui_state.editor_scroll_offset, 2);

        assert_eq!(ui_state.cursor_column, 1);
        assert_eq!(ui_state.cursor_line, 3);

        ui_state.cursor_move_left(&KeyModifiers::NONE);

        assert_eq!(ui_state.editor_scroll_offset, 1);

        assert_eq!(ui_state.cursor_column, 1);
        assert_eq!(ui_state.cursor_line, 2);
    }
}
