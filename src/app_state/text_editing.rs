use super::ui::UIState;

impl UIState {
    pub fn insert_character(&mut self, character: char) {
        let result = self.lines.get_mut((self.cursor_line - 1) as usize);

        match result {
            Some(line) => {
                let index = (self.cursor_column - 1) as usize;
                if index <= line.len() {
                    line.insert(index, character);
                    self.cursor_move_right();
                }
            }
            None => {
                // ????
            }
        }
    }

    pub fn remove_previous_character(&mut self) {
        let index = (self.cursor_column - 1) as usize;

        let result = self.lines.get_mut((self.cursor_line - 1) as usize);

        match result {
            Some(line) => {
                if index == 0 {
                    // we need to prepend current line to the previous one
                } else if index <= line.len() {
                    line.remove(index - 1);
                    self.cursor_move_left();
                }
            }
            None => {
                // ????
            }
        }
    }

    // if `delete` is pressed, we delete the next character
    pub fn remove_next_character(&mut self) {
        let index = (self.cursor_column - 1) as usize;

        let result = self.lines.get_mut((self.cursor_line - 1) as usize);

        match result {
            Some(line) => {
                let line_len = line.len();
                if index == line_len {
                    // we need to get the next line and append it to the current line
                } else if line_len > 0 && index < line_len {
                    line.remove(index);
                }
            }
            None => {
                // ????
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inserts_new_character_correctly() {
        let lines = vec![
            vec!['H', 'e', 'l', 'l', 'o', ' ', 'w', 'o', 'r', 'l', 'd', '!'],
            vec![],
            vec!['D', 'e', 's', 'c', 'r', 'i', 'p', 't', 'i', 'o', 'n'],
        ];
        let mut ui_state = UIState::new(5, lines);
        ui_state.set_editor_offset(30, 0);

        for _ in 0..6 {
            ui_state.cursor_move_right();
        }

        ui_state.insert_character('m');
        ui_state.insert_character('y');
        ui_state.insert_character(' ');

        assert_eq!(String::from_iter(&ui_state.lines[0]), "Hello my world!")
    }

    #[test]
    fn deletes_previous_character_correctly() {
        let lines = vec![
            vec!['H', 'e', 'l', 'l', 'o', ' ', 'w', 'o', 'r', 'l', 'd', '!'],
            vec![],
            vec!['D', 'e', 's', 'c', 'r', 'i', 'p', 't', 'i', 'o', 'n'],
        ];
        let mut ui_state = UIState::new(5, lines);
        ui_state.set_editor_offset(30, 0);

        for _ in 0..6 {
            ui_state.cursor_move_right();
        }

        // intentionally delete more than 6 characters
        for _ in 0..15 {
            ui_state.remove_previous_character();
        }

        assert_eq!(String::from_iter(&ui_state.lines[0]), "world!");
        assert_eq!(ui_state.cursor_column, 1);
    }

    #[test]
    fn deletes_next_character_correctly() {
        let lines = vec![
            vec!['H', 'e', 'l', 'l', 'o', ' ', 'w', 'o', 'r', 'l', 'd', '!'],
            vec![],
            vec!['D', 'e', 's', 'c', 'r', 'i', 'p', 't', 'i', 'o', 'n'],
        ];
        let mut ui_state = UIState::new(5, lines);
        ui_state.set_editor_offset(30, 0);

        for _ in 0..6 {
            ui_state.remove_next_character();
        }

        assert_eq!(String::from_iter(&ui_state.lines[0]), "world!");
        assert_eq!(ui_state.cursor_column, 1);
    }
}
