use super::ui::UIState;

impl UIState {
    pub fn insert_character(&mut self, character: char) {
        self.vertical_offset_target = 0;

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
        self.vertical_offset_target = 0;

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
        self.vertical_offset_target = 0;

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

    pub fn add_new_line(&mut self) {
        self.vertical_offset_target = 0;

        let current_line_len = self.get_line_len(self.cursor_line - 1);

        if self.cursor_column > current_line_len {
            self.lines.insert(self.cursor_line as usize, vec![]);
        } else {
            match self.lines.get_mut((self.cursor_line - 1) as usize) {
                Some(line) => {
                    let new_line = line.split_off((self.cursor_column - 1) as usize);
                    self.lines.insert(self.cursor_line as usize, new_line);
                    self.cursor_line += 1;
                    self.cursor_column = 1;
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

    #[test]
    fn adds_new_line_correctly() {
        let lines = vec![
            vec!['H', 'e', 'l', 'l', 'o', ' ', 'w', 'o', 'r', 'l', 'd', '!'],
            vec![],
            vec!['D', 'e', 's', 'c', 'r', 'i', 'p', 't', 'i', 'o', 'n'],
        ];
        let mut ui_state = UIState::new(5, lines);
        ui_state.set_editor_offset(30, 0);

        ui_state.cursor_move_right();
        ui_state.cursor_move_right();
        ui_state.cursor_move_right();

        ui_state.add_new_line();

        assert_eq!(ui_state.cursor_column, 1);
        assert_eq!(ui_state.cursor_line, 2);

        assert_eq!(String::from_iter(&ui_state.lines[0]), "Hel");
        assert_eq!(String::from_iter(&ui_state.lines[1]), "lo world!");

        ui_state.cursor_move_down();
        ui_state.add_new_line();

        assert_eq!(ui_state.cursor_column, 1);
        assert_eq!(ui_state.cursor_line, 3);

        assert_eq!(String::from_iter(&ui_state.lines[1]), "lo world!");
        assert_eq!(String::from_iter(&ui_state.lines[4]), "Description");
    }
}
