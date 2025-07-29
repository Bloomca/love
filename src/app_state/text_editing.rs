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

        if self.cursor_column == 1 && self.cursor_line == 1 {
            // nothing to remove, we are already at the beginning
            return;
        }

        // if we are in the middle of the first line, there is no previous line
        // it is safe to put 0 there, as the length is relevant only to the case
        // where index is 0, and we already covered the case with 0:0 position.
        let previous_line_len = if self.cursor_line > 1 {
            self.get_line_len(self.cursor_line - 2)
        } else {
            0
        };

        if let Some(line) = self.lines.get_mut((self.cursor_line - 1) as usize) {
            if index == 0 {
                // we need to prepend current line to the previous one
                // 1. call `let line = self.lines.remove(self.cursor_line - 1)`
                // 2. call self.lines[self.cursor_line - 2].append(line)
                // 3. call `self.cursor_line -= 1`
                // 4. call `self.cursor_column = previous_line_len + 1`
                // similar idea with delete, but at the end of the line

                if line.is_empty() {
                    self.lines.remove((self.cursor_line - 1) as usize);
                } else {
                    let mut current_line = self.lines.remove((self.cursor_line - 1) as usize);
                    match self.lines.get_mut((self.cursor_line - 2) as usize) {
                        Some(previous_line) => {
                            previous_line.append(&mut current_line);
                        }
                        None => {
                            return;
                        }
                    }
                }

                self.cursor_line -= 1;
                self.cursor_column = previous_line_len + 1;
            } else if index <= line.len() {
                line.remove(index - 1);
                self.cursor_move_left();
            }
        }
    }

    // if `delete` is pressed, we delete the next character
    pub fn remove_next_character(&mut self) {
        self.vertical_offset_target = 0;

        let index = (self.cursor_column - 1) as usize;

        if let Some(line) = self.lines.get_mut((self.cursor_line - 1) as usize) {
            let line_len = line.len();
            if index == line_len {
                // we need to get the next line and append it to the current line

                let is_last_line = self.lines.len() == self.cursor_line as usize;
                if is_last_line {
                    // do nothing, we are at the end of the file
                } else {
                    let mut next_line = self.lines.remove(self.cursor_line as usize);

                    if let Some(current_line) = self.lines.get_mut((self.cursor_line - 1) as usize)
                    {
                        current_line.append(&mut next_line);
                    }
                }
            } else if line_len > 0 && index < line_len {
                line.remove(index);
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

    #[test]
    fn handles_backspace_lines_deleting_correctly() {
        let lines = vec![
            vec!['H', 'e', 'l', 'l', 'o', ' ', 'w', 'o', 'r', 'l', 'd', '!'],
            vec![],
            vec!['D', 'e', 's', 'c', 'r', 'i', 'p', 't', 'i', 'o', 'n'],
        ];
        let mut ui_state = UIState::new(5, lines);
        ui_state.set_editor_offset(30, 0);

        ui_state.cursor_move_down();

        ui_state.remove_previous_character();

        assert_eq!(ui_state.lines.len(), 2);
        assert_eq!(ui_state.cursor_column, 13);
        assert_eq!(ui_state.cursor_line, 1);

        // bring it to the first character
        ui_state.cursor_move_up();
        // forget the vertical offset
        ui_state.cursor_move_left();
        ui_state.cursor_move_down();

        // make sure we are at the beginning of the second line
        assert_eq!(ui_state.cursor_column, 1);
        assert_eq!(ui_state.cursor_line, 2);

        ui_state.remove_previous_character();

        assert_eq!(ui_state.cursor_column, 13);
        assert_eq!(ui_state.cursor_line, 1);

        assert_eq!(ui_state.lines.len(), 1);
        assert_eq!(
            String::from_iter(&ui_state.lines[0]),
            "Hello world!Description"
        );
    }

    #[test]
    fn handles_deletekey_lines_deleting_correctly() {
        let lines = vec![
            vec!['H', 'e', 'l', 'l', 'o', ' ', 'w', 'o', 'r', 'l', 'd', '!'],
            vec![],
            vec!['D', 'e', 's', 'c', 'r', 'i', 'p', 't', 'i', 'o', 'n'],
        ];
        let mut ui_state = UIState::new(5, lines);
        ui_state.set_editor_offset(30, 0);

        for _ in 0..12 {
            ui_state.cursor_move_right();
        }

        // make sure we are at the beginning of the first line
        assert_eq!(ui_state.cursor_column, 13);
        assert_eq!(ui_state.cursor_line, 1);

        ui_state.remove_next_character();

        assert_eq!(ui_state.lines.len(), 2);

        ui_state.remove_next_character();
        assert_eq!(ui_state.lines.len(), 1);

        assert_eq!(
            String::from_iter(&ui_state.lines[0]),
            "Hello world!Description"
        );

        // navigate to the end of the last line
        ui_state.cursor_move_down();

        assert_eq!(ui_state.cursor_column, 24);
        assert_eq!(ui_state.cursor_line, 1);

        // nothing should happen, we are at the end of the file
        ui_state.remove_next_character();

        assert_eq!(ui_state.cursor_column, 24);
        assert_eq!(ui_state.cursor_line, 1);
    }
}
