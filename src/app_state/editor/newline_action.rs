use crate::app_state::editor::UIState;

impl UIState {
    pub fn add_new_line(&mut self) {
        self.vertical_offset_target = 0;

        // after deleting the selection, we need to insert the newline as usual
        self.delete_selection();

        let current_line_len = self.get_line_len(self.cursor_line - 1);

        let whitespaces = match self.lines.get(self.cursor_line - 1) {
            Some(line) => Self::calculate_whitespace_num(line),
            None => 0,
        };

        if self.cursor_column > current_line_len {
            self.lines.insert(self.cursor_line, vec![' '; whitespaces]);

            self.cursor_column = 1 + whitespaces;
            self.cursor_line += 1;

            self.handle_cursor_scrolling();
        } else {
            match self.lines.get_mut(self.cursor_line - 1) {
                Some(line) => {
                    let new_line = line.split_off(self.cursor_column - 1);

                    let prefixed_line =
                        vec![' '; whitespaces].into_iter().chain(new_line).collect();

                    self.lines.insert(self.cursor_line, prefixed_line);
                    self.cursor_line += 1;
                    self.cursor_column = 1 + whitespaces;

                    self.handle_cursor_scrolling();
                }
                None => {
                    // ???
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_state::app::Config;
    use crate::app_state::undo_redo::UndoRedo;
    use crossterm::event::KeyModifiers;

    #[test]
    fn adds_new_line_correctly() {
        let lines = vec![
            vec!['H', 'e', 'l', 'l', 'o', ' ', 'w', 'o', 'r', 'l', 'd', '!'],
            vec![],
            vec!['D', 'e', 's', 'c', 'r', 'i', 'p', 't', 'i', 'o', 'n'],
        ];
        let mut ui_state = UIState::new(5, lines);
        ui_state.set_editor_offset(30, 0, 50);

        ui_state.cursor_move_right(&KeyModifiers::NONE);
        ui_state.cursor_move_right(&KeyModifiers::NONE);
        ui_state.cursor_move_right(&KeyModifiers::NONE);

        ui_state.add_new_line();

        assert_eq!(ui_state.cursor_column, 1);
        assert_eq!(ui_state.cursor_line, 2);

        assert_eq!(String::from_iter(&ui_state.lines[0]), "Hel");
        assert_eq!(String::from_iter(&ui_state.lines[1]), "lo world!");

        ui_state.cursor_move_line_end(&KeyModifiers::NONE);
        ui_state.add_new_line();

        assert_eq!(ui_state.cursor_column, 1);
        assert_eq!(ui_state.cursor_line, 3);

        assert_eq!(String::from_iter(&ui_state.lines[1]), "lo world!");
        assert_eq!(String::from_iter(&ui_state.lines[4]), "Description");
    }

    #[test]
    fn handles_newline_with_whitespaces() {
        let mut undo_redo = UndoRedo::new();
        let lines = vec![
            vec!['H', 'e', 'l', 'l', 'o', ' ', 'w', 'o', 'r', 'l', 'd', '!'],
            vec!['A', 'n', 'o', 't', 'h', 'e', 'r', ' ', 'l', 'i', 'n', 'e'],
            vec!['D', 'e', 's', 'c', 'r', 'i', 'p', 't', 'i', 'o', 'n'],
        ];
        let mut ui_state = UIState::new(5, lines);
        let config = Config::new();
        ui_state.set_editor_offset(30, 0, 50);

        ui_state.handle_tab_key(&config, &mut undo_redo);

        assert_eq!(ui_state.cursor_column, 5);
        assert_eq!(ui_state.cursor_line, 1);

        assert_eq!(String::from_iter(&ui_state.lines[0]), "    Hello world!");

        ui_state.cursor_move_line_end(&KeyModifiers::NONE);
        ui_state.add_new_line();

        assert_eq!(ui_state.cursor_column, 5);
        assert_eq!(ui_state.cursor_line, 2);

        assert_eq!(String::from_iter(&ui_state.lines[1]), "    ");
    }
}
