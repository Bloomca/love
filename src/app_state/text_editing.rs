use super::app::Config;
use super::ui::UIState;
use crossterm::event::KeyModifiers;

impl UIState {
    pub fn insert_character(&mut self, character: char) {
        self.vertical_offset_target = 0;

        // after deleting selection, we need to insert the character normally
        self.delete_selection();

        let result = self.lines.get_mut(self.cursor_line - 1);

        match result {
            Some(line) => {
                let index = self.cursor_column - 1;
                if index <= line.len() {
                    line.insert(index, character);
                    // manually set to have no modifiers, so the newly inserted character is not selected
                    self.cursor_move_right(&KeyModifiers::NONE);
                }
            }
            None => {
                // ????
            }
        }
    }

    pub fn remove_previous_character(&mut self) {
        self.vertical_offset_target = 0;

        // if we successfully deleted the selection, we don't need to do anything else
        if self.delete_selection() {
            return;
        }

        let index = self.cursor_column - 1;

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

        if let Some(line) = self.lines.get_mut(self.cursor_line - 1) {
            if index == 0 {
                // we need to prepend current line to the previous one
                // 1. call `let line = self.lines.remove(self.cursor_line - 1)`
                // 2. call self.lines[self.cursor_line - 2].append(line)
                // 3. call `self.cursor_line -= 1`
                // 4. call `self.cursor_column = previous_line_len + 1`
                // similar idea with delete, but at the end of the line

                if line.is_empty() {
                    self.lines.remove(self.cursor_line - 1);
                } else {
                    let mut current_line = self.lines.remove(self.cursor_line - 1);
                    match self.lines.get_mut(self.cursor_line - 2) {
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

                self.handle_cursor_scrolling();
            } else if index <= line.len() {
                line.remove(index - 1);
                self.cursor_move_left(&KeyModifiers::NONE);
            }
        }
    }

    // if `delete` is pressed, we delete the next character
    pub fn remove_next_character(&mut self) {
        self.vertical_offset_target = 0;

        // if we successfully deleted the selection, we don't need to do anything else
        if self.delete_selection() {
            return;
        }

        let index = self.cursor_column - 1;

        if let Some(line) = self.lines.get_mut(self.cursor_line - 1) {
            let line_len = line.len();
            if index == line_len {
                // we need to get the next line and append it to the current line

                let is_last_line = self.lines.len() == self.cursor_line;
                if is_last_line {
                    // do nothing, we are at the end of the file
                } else {
                    let mut next_line = self.lines.remove(self.cursor_line);

                    if let Some(current_line) = self.lines.get_mut(self.cursor_line - 1) {
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

        // after deleting the selection, we need to insert the newline as usual
        self.delete_selection();

        let current_line_len = self.get_line_len(self.cursor_line - 1);

        let whitespaces = match self.lines.get(self.cursor_line - 1) {
            Some(line) => self.calculate_whitespace_num(line),
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

    pub fn handle_tab_key(&mut self, config: &Config) {
        if config.tabs_to_spaces {
            for _ in 0..config.whitespaces_amount {
                self.insert_character(' ');
            }
        } else {
            self.insert_character('\t');
        }
    }

    fn calculate_whitespace_num(&self, line: &[char]) -> usize {
        line.iter().take_while(|c| c.is_whitespace()).count()
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
        ui_state.set_editor_offset(30, 0, 50);

        for _ in 0..6 {
            ui_state.cursor_move_right(&KeyModifiers::NONE);
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
        ui_state.set_editor_offset(30, 0, 50);

        for _ in 0..6 {
            ui_state.cursor_move_right(&KeyModifiers::NONE);
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
        ui_state.set_editor_offset(30, 0, 50);

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
        ui_state.set_editor_offset(30, 0, 50);

        ui_state.cursor_move_right(&KeyModifiers::NONE);
        ui_state.cursor_move_right(&KeyModifiers::NONE);
        ui_state.cursor_move_right(&KeyModifiers::NONE);

        ui_state.add_new_line();

        assert_eq!(ui_state.cursor_column, 1);
        assert_eq!(ui_state.cursor_line, 2);

        assert_eq!(String::from_iter(&ui_state.lines[0]), "Hel");
        assert_eq!(String::from_iter(&ui_state.lines[1]), "lo world!");

        ui_state.cursor_move_line_end();
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
        ui_state.set_editor_offset(30, 0, 50);

        ui_state.cursor_move_down(&KeyModifiers::NONE);

        ui_state.remove_previous_character();

        assert_eq!(ui_state.lines.len(), 2);
        assert_eq!(ui_state.cursor_column, 13);
        assert_eq!(ui_state.cursor_line, 1);

        // bring it to the first character
        ui_state.cursor_move_up(&KeyModifiers::NONE);
        // forget the vertical offset
        ui_state.cursor_move_left(&KeyModifiers::NONE);
        ui_state.cursor_move_down(&KeyModifiers::NONE);

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
        ui_state.set_editor_offset(30, 0, 50);

        for _ in 0..12 {
            ui_state.cursor_move_right(&KeyModifiers::NONE);
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
        ui_state.cursor_move_down(&KeyModifiers::NONE);

        assert_eq!(ui_state.cursor_column, 24);
        assert_eq!(ui_state.cursor_line, 1);

        // nothing should happen, we are at the end of the file
        ui_state.remove_next_character();

        assert_eq!(ui_state.cursor_column, 24);
        assert_eq!(ui_state.cursor_line, 1);
    }

    #[test]
    fn handles_selection_delete_correctly() {
        let lines = vec![
            vec!['H', 'e', 'l', 'l', 'o', ' ', 'w', 'o', 'r', 'l', 'd', '!'],
            vec!['A', 'n', 'o', 't', 'h', 'e', 'r', ' ', 'l', 'i', 'n', 'e'],
            vec!['D', 'e', 's', 'c', 'r', 'i', 'p', 't', 'i', 'o', 'n'],
        ];
        let mut ui_state = UIState::new(5, lines);
        ui_state.set_editor_offset(30, 0, 50);

        ui_state.cursor_move_right(&KeyModifiers::NONE);
        ui_state.cursor_move_right(&KeyModifiers::NONE);
        ui_state.cursor_move_right(&KeyModifiers::NONE);

        ui_state.cursor_move_right(&KeyModifiers::SHIFT);
        ui_state.cursor_move_right(&KeyModifiers::SHIFT);

        ui_state.remove_previous_character();

        assert_eq!(ui_state.cursor_column, 4);
        assert_eq!(ui_state.cursor_line, 1);

        assert_eq!(String::from_iter(&ui_state.lines[0]), "Hel world!");

        ui_state.cursor_move_left(&KeyModifiers::SHIFT);
        ui_state.cursor_move_left(&KeyModifiers::SHIFT);

        ui_state.remove_next_character();

        assert_eq!(ui_state.cursor_column, 2);
        assert_eq!(ui_state.cursor_line, 1);

        assert_eq!(String::from_iter(&ui_state.lines[0]), "H world!");

        ui_state.cursor_move_down(&KeyModifiers::NONE);
        ui_state.cursor_move_right(&KeyModifiers::NONE);
        ui_state.cursor_move_right(&KeyModifiers::NONE);
        ui_state.cursor_move_down(&KeyModifiers::SHIFT);

        ui_state.remove_next_character();

        assert_eq!(ui_state.cursor_column, 4);
        assert_eq!(ui_state.cursor_line, 2);

        assert_eq!(String::from_iter(&ui_state.lines[1]), "Anocription");

        ui_state.cursor_move_right(&KeyModifiers::SHIFT);
        ui_state.cursor_move_right(&KeyModifiers::SHIFT);

        ui_state.insert_character('R');
        ui_state.insert_character('O');
        ui_state.insert_character('O');
        ui_state.insert_character('T');

        assert_eq!(ui_state.cursor_column, 8);
        assert_eq!(ui_state.cursor_line, 2);

        assert_eq!(String::from_iter(&ui_state.lines[1]), "AnoROOTiption");

        ui_state.cursor_move_left(&KeyModifiers::SHIFT);
        ui_state.cursor_move_left(&KeyModifiers::SHIFT);

        ui_state.add_new_line();

        assert_eq!(ui_state.cursor_column, 1);
        assert_eq!(ui_state.cursor_line, 3);

        assert_eq!(String::from_iter(&ui_state.lines[1]), "AnoRO");
        assert_eq!(String::from_iter(&ui_state.lines[2]), "iption");
        assert_eq!(String::from_iter(&ui_state.lines[0]), "H world!");

        ui_state.cursor_move_line_end();
        ui_state.add_new_line();

        ui_state.insert_character('S');
        ui_state.insert_character('o');
        ui_state.insert_character('m');
        ui_state.insert_character('e');
        ui_state.insert_character('t');
        ui_state.insert_character('h');
        ui_state.insert_character('i');
        ui_state.insert_character('n');
        ui_state.insert_character('g');

        assert_eq!(String::from_iter(&ui_state.lines[3]), "Something");

        ui_state.cursor_move_left(&KeyModifiers::NONE);
        ui_state.cursor_move_left(&KeyModifiers::NONE);

        ui_state.cursor_move_up(&KeyModifiers::SHIFT);
        ui_state.cursor_move_up(&KeyModifiers::SHIFT);
        ui_state.cursor_move_up(&KeyModifiers::SHIFT);

        ui_state.remove_previous_character();

        assert_eq!(ui_state.cursor_column, 8);
        assert_eq!(ui_state.cursor_line, 1);

        ui_state.insert_character(' ');

        assert_eq!(ui_state.lines.len(), 1);
        assert_eq!(String::from_iter(&ui_state.lines[0]), "H world ng");
    }

    #[test]
    fn handles_newline_with_whitespaces() {
        let lines = vec![
            vec!['H', 'e', 'l', 'l', 'o', ' ', 'w', 'o', 'r', 'l', 'd', '!'],
            vec!['A', 'n', 'o', 't', 'h', 'e', 'r', ' ', 'l', 'i', 'n', 'e'],
            vec!['D', 'e', 's', 'c', 'r', 'i', 'p', 't', 'i', 'o', 'n'],
        ];
        let mut ui_state = UIState::new(5, lines);
        let config = Config::new();
        ui_state.set_editor_offset(30, 0, 50);

        ui_state.handle_tab_key(&config);

        assert_eq!(ui_state.cursor_column, 5);
        assert_eq!(ui_state.cursor_line, 1);

        assert_eq!(String::from_iter(&ui_state.lines[0]), "    Hello world!");

        ui_state.cursor_move_line_end();
        ui_state.add_new_line();

        assert_eq!(ui_state.cursor_column, 5);
        assert_eq!(ui_state.cursor_line, 2);

        assert_eq!(String::from_iter(&ui_state.lines[1]), "    ");
    }
}
