use super::ui::UIState;

impl UIState {
    pub fn cursor_move_left(&mut self) {
        if self.should_show_cursor && self.cursor_column > 1 {
            let new_value = self.cursor_column - 1;
            self.cursor_column = new_value;
        }
    }

    pub fn cursor_move_right(&mut self) {
        if self.should_show_cursor {
            self.cursor_column += 1;
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
            self.cursor_line += 1;
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
        ui_state.set_editor_offset(30, 0);

        ui_state.cursor_move_left();
        ui_state.cursor_move_left();
        ui_state.cursor_move_left();

        ui_state.cursor_move_up();
        ui_state.cursor_move_up();
        ui_state.cursor_move_up();

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
        ui_state.set_editor_offset(30, 0);

        ui_state.cursor_move_right();
        ui_state.cursor_move_right();

        assert_eq!(ui_state.cursor_column, 3);

        ui_state.cursor_move_left();

        assert_eq!(ui_state.cursor_column, 2);

        ui_state.cursor_move_down();
        ui_state.cursor_move_down();

        assert_eq!(ui_state.cursor_line, 3);

        ui_state.cursor_move_up();

        assert_eq!(ui_state.cursor_line, 2);
    }
}
