use crate::app_state::editor::UIState;
use crate::app_state::undo_redo::{UndoAction, UndoRedo};
use crossterm::event::KeyModifiers;

impl UIState {
    pub fn insert_character(&mut self, character: char, undo_redo: &mut UndoRedo) {
        self.vertical_offset_target = 0;

        // after deleting selection, we need to insert the character normally
        self.delete_selection();

        let result = self.lines.get_mut(self.cursor_line - 1);

        match result {
            Some(line) => {
                let index = self.cursor_column - 1;
                if index <= line.len() {
                    line.insert(index, character);
                    let start = (self.cursor_line, self.cursor_column);
                    // manually set to have no modifiers, so the newly inserted character is not selected
                    self.cursor_move_right(&KeyModifiers::NONE);
                    let end = (self.cursor_line, self.cursor_column);
                    undo_redo.add_undo_action(UndoAction::AddCharacter(character, start, end));
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
        let mut undo_redo = UndoRedo::new();
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

        ui_state.insert_character('m', &mut undo_redo);
        ui_state.insert_character('y', &mut undo_redo);
        ui_state.insert_character(' ', &mut undo_redo);

        assert_eq!(String::from_iter(&ui_state.lines[0]), "Hello my world!")
    }
}
