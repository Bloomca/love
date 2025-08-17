use crate::app_state::editor::UIState;

use crossterm::event::KeyModifiers;

use crate::app_state::selection::Selection;
use crate::app_state::undo_redo::UndoSelection;

impl UIState {
    /// Start a new selection if necessary, clear existing one if necessary or do nothing if one exists
    pub fn start_selection(&mut self, modifiers: &KeyModifiers) {
        if modifiers.contains(KeyModifiers::SHIFT) {
            if self.selection.is_none() {
                self.selection = Some(Selection::new(self.cursor_line, self.cursor_column));
            }
        } else {
            self.selection = None;
        }
    }

    /// This function only adjust existing selection, but it doesn't clear it based on keyboard modifiers.
    /// However, it will clear the selection if the cursor is at the same spot as it started.
    pub fn adjust_selection(&mut self) {
        if let Some(selection) = &mut self.selection {
            if selection.start.0 == self.cursor_line && selection.start.1 == self.cursor_column {
                self.selection = None;
            } else {
                selection.set_end(self.cursor_line, self.cursor_column);
            }
        }
    }

    /// a single check to see if we have _any_ selection
    pub fn has_any_selection(&self) -> bool {
        self.selection.is_some()
    }

    /// a single check to see if the entire line is selected
    pub fn is_entire_line_selected(&self, line: usize) -> bool {
        match &self.selection {
            Some(selection) => selection.is_line_selected(line),
            None => false,
        }
    }

    /// Get selection range -- intended to be called once per line
    /// and is not meant to return range of the full line.
    ///
    /// To check if the entire line is selected, call `is_entire_line_selected` method.
    /// Line needs to be in cursor mode, starting with `1``
    pub fn get_selection_range(&self, line: usize) -> Option<(usize, usize)> {
        match &self.selection {
            Some(selection) => selection.get_selection_range(line),
            None => None,
        }
    }

    pub fn delete_selection(&mut self) -> Option<UndoSelection> {
        let Some(selection) = &self.selection else {
            return None;
        };

        let result = self.delete_range(selection.start, selection.end);

        let Some(selection) = &self.selection else {
            return None;
        };

        let (start_line, start_column) = selection.get_first_position();

        self.cursor_column = start_column;
        self.cursor_line = start_line;
        self.selection = None;

        result
    }

    /// Delete text between the range; please note that the cursor is not moved
    /// so it is the responsibility of the caller to do so
    pub fn delete_range(
        &mut self,
        start: (usize, usize),
        end: (usize, usize),
    ) -> Option<UndoSelection> {
        let (start_line, start_column) = start;
        let (end_line, end_column) = end;
        let min_line = start_line.min(end_line) - 1;
        let max_line = start_line.max(end_line) - 1;

        if min_line == max_line {
            // if the line is equal, we need to remove selected range
            let min_column = start_column.min(end_column);
            let max_column = start_column.max(end_column);

            self.lines.get_mut(min_line).map(|line| UndoSelection {
                text: line.drain((min_column - 1)..(max_column - 1)).collect(),
                start,
                end,
            })
        } else {
            // if there are multiple lines, we need to get the last line, remove selected elements;
            // get the first line, remove selected elements; append last line to the first
            // if there are lines between first and last, we need to remove them

            let mut result = String::new();

            let mut last_line = self.lines.remove(max_line);

            let (last_line_col, first_line_col) = if end_line > start_line {
                (end_column, start_column)
            } else {
                (start_column, end_column)
            };

            let last_line_data: Vec<char> = last_line.drain(0..(last_line_col - 1)).collect();

            let first_line = self.lines.get_mut(min_line)?;

            result.extend(first_line.drain((first_line_col - 1)..));
            first_line.append(&mut last_line);

            let lines_between = max_line - min_line - 1;
            // it means that we have some lines we need to completely remove
            if lines_between > 0 {
                for i in 0..lines_between {
                    result.push('\n');
                    result.extend(self.lines.remove(max_line - i - 1));
                }
            }

            result.push('\n');
            result.extend(last_line_data);

            Some(UndoSelection {
                text: result,
                start,
                end,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_state::undo_redo::UndoRedo;

    #[test]
    fn handle_selection_correctly() {
        let lines = vec![
            vec!['H', 'e', 'l', 'l', 'o', ' ', 'w', 'o', 'r', 'l', 'd', '!'],
            vec!['A', 'n', 'o', 't', 'h', 'e', 'r', ' ', 'l', 'i', 'n', 'e'],
            vec!['D', 'e', 's', 'c', 'r', 'i', 'p', 't', 'i', 'o', 'n'],
        ];
        let mut ui_state = UIState::new(5, lines);
        ui_state.set_editor_offset(30, 0, 50);

        ui_state.cursor_move_right(&KeyModifiers::SHIFT);
        ui_state.cursor_move_right(&KeyModifiers::SHIFT);

        assert!(ui_state.has_any_selection());
        let selection = ui_state.get_selection_range(1).unwrap();

        assert_eq!(selection, (1, 3));

        ui_state.cursor_move_right(&KeyModifiers::NONE);

        assert!(!ui_state.has_any_selection());

        ui_state.cursor_move_left(&KeyModifiers::SHIFT);
        ui_state.cursor_move_left(&KeyModifiers::SHIFT);

        let selection = ui_state.get_selection_range(1).unwrap();
        assert_eq!(selection, (2, 4));

        assert_eq!(ui_state.cursor_column, 2);
        assert_eq!(ui_state.cursor_line, 1);

        ui_state.cursor_move_right(&KeyModifiers::NONE);

        assert!(!ui_state.has_any_selection());

        ui_state.cursor_move_right(&KeyModifiers::SHIFT);
        ui_state.cursor_move_down(&KeyModifiers::SHIFT);

        let selection1 = ui_state.get_selection_range(1).unwrap();
        let selection2 = ui_state.get_selection_range(2).unwrap();

        assert_eq!(selection1, (3, 0));
        assert_eq!(selection2, (0, 4));
    }

    #[test]
    fn handle_multi_line_selection_correctly() {
        let lines = vec![
            vec!['H', 'e', 'l', 'l', 'o', ' ', 'w', 'o', 'r', 'l', 'd', '!'],
            vec!['A', 'n', 'o', 't', 'h', 'e', 'r', ' ', 'l', 'i', 'n', 'e'],
            vec!['D', 'e', 's', 'c', 'r', 'i', 'p', 't', 'i', 'o', 'n'],
        ];
        let mut ui_state = UIState::new(5, lines);
        ui_state.set_editor_offset(30, 0, 50);

        ui_state.cursor_move_down(&KeyModifiers::SHIFT);

        let selection1 = ui_state.get_selection_range(1).unwrap();
        let selection2 = ui_state.get_selection_range(2).unwrap();

        assert_eq!(selection1, (1, 0));
        assert_eq!(selection2, (0, 1));
    }

    #[test]
    fn handles_selection_delete_correctly() {
        let mut undo_redo = UndoRedo::new();
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

        ui_state.remove_previous_character(&mut undo_redo);

        assert_eq!(ui_state.cursor_column, 4);
        assert_eq!(ui_state.cursor_line, 1);

        assert_eq!(String::from_iter(&ui_state.lines[0]), "Hel world!");

        ui_state.cursor_move_left(&KeyModifiers::SHIFT);
        ui_state.cursor_move_left(&KeyModifiers::SHIFT);

        ui_state.remove_next_character(&mut undo_redo);

        assert_eq!(ui_state.cursor_column, 2);
        assert_eq!(ui_state.cursor_line, 1);

        assert_eq!(String::from_iter(&ui_state.lines[0]), "H world!");

        ui_state.cursor_move_down(&KeyModifiers::NONE);
        ui_state.cursor_move_right(&KeyModifiers::NONE);
        ui_state.cursor_move_right(&KeyModifiers::NONE);
        ui_state.cursor_move_down(&KeyModifiers::SHIFT);

        ui_state.remove_next_character(&mut undo_redo);

        assert_eq!(ui_state.cursor_column, 4);
        assert_eq!(ui_state.cursor_line, 2);

        assert_eq!(String::from_iter(&ui_state.lines[1]), "Anocription");

        ui_state.cursor_move_right(&KeyModifiers::SHIFT);
        ui_state.cursor_move_right(&KeyModifiers::SHIFT);

        ui_state.insert_character('R', &mut undo_redo);
        ui_state.insert_character('O', &mut undo_redo);
        ui_state.insert_character('O', &mut undo_redo);
        ui_state.insert_character('T', &mut undo_redo);

        assert_eq!(ui_state.cursor_column, 8);
        assert_eq!(ui_state.cursor_line, 2);

        assert_eq!(String::from_iter(&ui_state.lines[1]), "AnoROOTiption");

        ui_state.cursor_move_left(&KeyModifiers::SHIFT);
        ui_state.cursor_move_left(&KeyModifiers::SHIFT);

        ui_state.add_new_line(&mut undo_redo);

        assert_eq!(ui_state.cursor_column, 1);
        assert_eq!(ui_state.cursor_line, 3);

        assert_eq!(String::from_iter(&ui_state.lines[1]), "AnoRO");
        assert_eq!(String::from_iter(&ui_state.lines[2]), "iption");
        assert_eq!(String::from_iter(&ui_state.lines[0]), "H world!");

        ui_state.cursor_move_line_end(&KeyModifiers::NONE);
        ui_state.add_new_line(&mut undo_redo);

        ui_state.insert_character('S', &mut undo_redo);
        ui_state.insert_character('o', &mut undo_redo);
        ui_state.insert_character('m', &mut undo_redo);
        ui_state.insert_character('e', &mut undo_redo);
        ui_state.insert_character('t', &mut undo_redo);
        ui_state.insert_character('h', &mut undo_redo);
        ui_state.insert_character('i', &mut undo_redo);
        ui_state.insert_character('n', &mut undo_redo);
        ui_state.insert_character('g', &mut undo_redo);

        assert_eq!(String::from_iter(&ui_state.lines[3]), "Something");

        ui_state.cursor_move_left(&KeyModifiers::NONE);
        ui_state.cursor_move_left(&KeyModifiers::NONE);

        ui_state.cursor_move_up(&KeyModifiers::SHIFT);
        ui_state.cursor_move_up(&KeyModifiers::SHIFT);
        ui_state.cursor_move_up(&KeyModifiers::SHIFT);

        ui_state.remove_previous_character(&mut undo_redo);

        assert_eq!(ui_state.cursor_column, 8);
        assert_eq!(ui_state.cursor_line, 1);

        ui_state.insert_character(' ', &mut undo_redo);

        assert_eq!(ui_state.lines.len(), 1);
        assert_eq!(String::from_iter(&ui_state.lines[0]), "H world ng");
    }

    #[test]
    fn handles_line_start_end_selection() {
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

        ui_state.cursor_move_line_end(&KeyModifiers::SHIFT);

        let selection = ui_state.get_selection_range(1).unwrap();
        assert_eq!(selection, (4, 13));

        ui_state.cursor_move_line_start(&KeyModifiers::SHIFT);

        let selection = ui_state.get_selection_range(1).unwrap();
        assert_eq!(selection, (1, 4));
    }
}
