use super::app::AppState;

impl AppState {
    pub fn insert_character(&mut self, character: char) {
        let result = self.lines.get_mut((self.ui_state.cursor_line - 1) as usize);

        match result {
            Some(line) => {
                let index = (self.ui_state.cursor_column - 1) as usize;
                if index <= line.len() {
                    line.insert(index, character);
                    self.ui_state.cursor_move_right();
                }
            }
            None => {
                // ????
            }
        }
    }

    pub fn remove_previous_character(&mut self) {
        let index = (self.ui_state.cursor_column - 1) as usize;

        let result = self.lines.get_mut((self.ui_state.cursor_line - 1) as usize);

        match result {
            Some(line) => {
                if index == 0 {
                    // we need to prepend current line to the previous one
                    return;
                } else if index <= line.len() {
                    line.remove(index - 1);
                    self.ui_state.cursor_move_left();
                }
            }
            None => {
                // ????
            }
        }
    }

    // if `delete` is pressed, we delete the next character
    pub fn remove_next_character(&mut self) {
        let index = (self.ui_state.cursor_column - 1) as usize;

        let result = self.lines.get_mut((self.ui_state.cursor_line - 1) as usize);

        match result {
            Some(line) => {
                let line_len = line.len();
                if index == line_len {
                    // we need to get the next line and append it to the current line
                } else if line_len > 0 && index <= line_len - 1 {
                    line.remove(index);
                }
            }
            None => {
                // ????
            }
        }
    }
}