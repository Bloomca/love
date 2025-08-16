use crate::app_state::editor::UIState;
use crate::app_state::undo_redo::{RemoveBufferType, UndoAction, UndoRedo};
use crossterm::event::KeyModifiers;

impl UIState {
    pub fn remove_previous_character(&mut self, undo_redo: &mut UndoRedo) {
        self.vertical_offset_target = 0;

        // if we successfully deleted the selection, we don't need to do anything else
        if self.delete_selection() {
            return;
        }

        let index = self.cursor_column - 1;
        let start = (self.cursor_line, self.cursor_column);

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

                undo_redo.add_undo_action(UndoAction::RemoveCharacter(
                    '\n',
                    start,
                    (self.cursor_line, self.cursor_column),
                    RemoveBufferType::Backspace,
                ));
            } else if index <= line.len() {
                let deleted_character = line.get(index - 1).copied();
                line.remove(index - 1);
                self.cursor_move_left(&KeyModifiers::NONE);

                if let Some(ch) = deleted_character {
                    undo_redo.add_undo_action(UndoAction::RemoveCharacter(
                        ch,
                        start,
                        (self.cursor_line, self.cursor_column),
                        RemoveBufferType::Backspace,
                    ));
                }
            }
        }
    }

    // if `delete` is pressed, we delete the next character
    pub fn remove_next_character(&mut self, undo_redo: &mut UndoRedo) {
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

                    undo_redo.add_undo_action(UndoAction::RemoveCharacter(
                        '\n',
                        // the cursor never moves when pressing delete,
                        // so the value can be the same
                        (self.cursor_line, self.cursor_column),
                        (self.cursor_line, self.cursor_column),
                        RemoveBufferType::Delete,
                    ));
                }
            } else if line_len > 0 && index < line_len {
                let deleted_character = line.get(index).copied();
                line.remove(index);

                if let Some(ch) = deleted_character {
                    undo_redo.add_undo_action(UndoAction::RemoveCharacter(
                        ch,
                        // the cursor never moves when pressing delete,
                        // so the value can be the same
                        (self.cursor_line, self.cursor_column),
                        (self.cursor_line, self.cursor_column),
                        RemoveBufferType::Delete,
                    ));
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn deletes_previous_character_correctly() {
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

        // intentionally delete more than 6 characters
        for _ in 0..15 {
            ui_state.remove_previous_character(&mut undo_redo);
        }

        assert_eq!(String::from_iter(&ui_state.lines[0]), "world!");
        assert_eq!(ui_state.cursor_column, 1);
    }

    #[test]
    fn deletes_next_character_correctly() {
        let mut undo_redo = UndoRedo::new();
        let lines = vec![
            vec!['H', 'e', 'l', 'l', 'o', ' ', 'w', 'o', 'r', 'l', 'd', '!'],
            vec![],
            vec!['D', 'e', 's', 'c', 'r', 'i', 'p', 't', 'i', 'o', 'n'],
        ];
        let mut ui_state = UIState::new(5, lines);
        ui_state.set_editor_offset(30, 0, 50);

        for _ in 0..6 {
            ui_state.remove_next_character(&mut undo_redo);
        }

        assert_eq!(String::from_iter(&ui_state.lines[0]), "world!");
        assert_eq!(ui_state.cursor_column, 1);
    }

    #[test]
    fn handles_backspace_lines_deleting_correctly() {
        let mut undo_redo = UndoRedo::new();
        let lines = vec![
            vec!['H', 'e', 'l', 'l', 'o', ' ', 'w', 'o', 'r', 'l', 'd', '!'],
            vec![],
            vec!['D', 'e', 's', 'c', 'r', 'i', 'p', 't', 'i', 'o', 'n'],
        ];
        let mut ui_state = UIState::new(5, lines);
        ui_state.set_editor_offset(30, 0, 50);

        ui_state.cursor_move_down(&KeyModifiers::NONE);

        ui_state.remove_previous_character(&mut undo_redo);

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

        ui_state.remove_previous_character(&mut undo_redo);

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
        let mut undo_redo = UndoRedo::new();
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

        ui_state.remove_next_character(&mut undo_redo);

        assert_eq!(ui_state.lines.len(), 2);

        ui_state.remove_next_character(&mut undo_redo);
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
        ui_state.remove_next_character(&mut undo_redo);

        assert_eq!(ui_state.cursor_column, 24);
        assert_eq!(ui_state.cursor_line, 1);
    }
}
