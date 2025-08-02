use crate::app_state::editor::UIState;
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
}
