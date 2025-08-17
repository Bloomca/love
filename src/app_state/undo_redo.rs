use std::time::SystemTime;

use crate::app_state::editor::UIState;

const MAX_BUFFER_DEBOUNCE_TIME_MS: u128 = 250;

pub struct UndoSelection {
    pub text: String,
    pub start: (usize, usize),
    pub end: (usize, usize),
}

struct AddAction {
    start: (usize, usize),
    end: (usize, usize),
    chars: Vec<char>,
    selection: Option<UndoSelection>,
}

struct RemoveBackAction {
    data: String,
    start: (usize, usize),
    end: (usize, usize),
}

struct RemoveForwardAction {
    data: String,
    start: (usize, usize),
    end: (usize, usize),
}

struct PasteAction {
    data: String,
    start: (usize, usize),
    end: (usize, usize),
    selection: Option<UndoSelection>,
}

enum Action {
    Add(AddAction),
    RemoveBack(RemoveBackAction),
    RemoveForward(RemoveForwardAction),
    Paste(PasteAction),
}

pub enum UndoAction {
    /// character, starting position, ending position, removed selection
    AddCharacter(char, (usize, usize), (usize, usize), Option<UndoSelection>),
    Paste(
        String,
        (usize, usize),
        (usize, usize),
        Option<UndoSelection>,
    ),
    RemoveCharacter(char, (usize, usize), (usize, usize), RemoveBufferType),
}

struct AddCharacterBuffer {
    start_position: (usize, usize),
    end_position: (usize, usize),
    chars: Vec<char>,
    last_action_timestamp: SystemTime,
    selection: Option<UndoSelection>,
}

impl AddCharacterBuffer {
    fn new(line_num: usize, line_column: usize, selection: Option<UndoSelection>) -> Self {
        AddCharacterBuffer {
            start_position: (line_num, line_column),
            end_position: (line_num, line_column),
            chars: vec![],
            last_action_timestamp: SystemTime::now(),
            selection,
        }
    }

    fn should_commit(&self) -> bool {
        if let Ok(elapsed) = self.last_action_timestamp.elapsed() {
            return elapsed.as_millis() > MAX_BUFFER_DEBOUNCE_TIME_MS;
        }

        true
    }

    fn add_char(&mut self, ch: char, end_line: usize, end_col: usize) {
        self.chars.push(ch);
        self.end_position = (end_line, end_col);
        self.last_action_timestamp = SystemTime::now();
    }
}

#[derive(PartialEq, Eq)]
pub enum RemoveBufferType {
    Backspace,
    Delete,
}

struct RemoveCharacterBuffer {
    start_position: (usize, usize),
    end_position: (usize, usize),
    chars: Vec<char>,
    last_action_timestamp: SystemTime,
    remove_type: RemoveBufferType,
}

impl RemoveCharacterBuffer {
    fn new(line_num: usize, line_column: usize, remove_type: RemoveBufferType) -> Self {
        RemoveCharacterBuffer {
            start_position: (line_num, line_column),
            end_position: (line_num, line_column),
            chars: vec![],
            last_action_timestamp: SystemTime::now(),
            remove_type,
        }
    }

    fn should_commit(&self) -> bool {
        if let Ok(elapsed) = self.last_action_timestamp.elapsed() {
            return elapsed.as_millis() > MAX_BUFFER_DEBOUNCE_TIME_MS;
        }

        true
    }

    fn remove_char(&mut self, ch: char, end_line: usize, end_col: usize) {
        self.chars.push(ch);
        self.end_position = (end_line, end_col);
        self.last_action_timestamp = SystemTime::now();
    }
}

enum Buffer {
    AddCharacter(AddCharacterBuffer),
    RemoveCharacter(RemoveCharacterBuffer),
}

pub struct UndoRedo {
    undo_actions: Vec<Action>,
    redo_actions: Vec<Action>,
    /// keep actions here so that we don't undo
    /// characters typed 1 by 1, only commit to the queue
    /// after certain amount of time passed
    buffer: Option<Buffer>,
}

impl UndoRedo {
    pub fn new() -> Self {
        UndoRedo {
            undo_actions: Vec::new(),
            redo_actions: Vec::new(),
            buffer: None,
        }
    }

    pub fn add_undo_action(&mut self, action: UndoAction) {
        // if we perform any action, all redo actions are immediately invalidated
        self.redo_actions.clear();
        match action {
            UndoAction::AddCharacter(
                ch,
                (start_line, start_col),
                (end_line, end_col),
                removed_selection,
            ) => match &mut self.buffer {
                Some(buffer) => match buffer {
                    Buffer::AddCharacter(add_character_buffer) => {
                        // if we somehow receive new selection while the buffer is live,
                        // we simply commit the buffer as it would be too hard to restore
                        if removed_selection.is_some() || add_character_buffer.should_commit() {
                            self.commit_buffer();
                            let mut add_buffer: AddCharacterBuffer =
                                AddCharacterBuffer::new(start_line, start_col, removed_selection);
                            add_buffer.add_char(ch, end_line, end_col);
                            self.buffer = Some(Buffer::AddCharacter(add_buffer));
                        } else {
                            add_character_buffer.add_char(ch, end_line, end_col);
                        }
                    }
                    Buffer::RemoveCharacter(_) => {
                        self.commit_buffer();
                        let mut add_buffer =
                            AddCharacterBuffer::new(start_line, start_col, removed_selection);
                        add_buffer.add_char(ch, end_line, end_col);
                        self.buffer = Some(Buffer::AddCharacter(add_buffer));
                    }
                },
                None => {
                    let mut add_buffer =
                        AddCharacterBuffer::new(start_line, start_col, removed_selection);
                    add_buffer.add_char(ch, end_line, end_col);
                    self.buffer = Some(Buffer::AddCharacter(add_buffer))
                }
            },
            UndoAction::Paste(data, start, end, selection) => {
                if let Some(buffer) = &self.buffer {
                    match buffer {
                        Buffer::AddCharacter(_) => {
                            self.commit_buffer();
                        }
                        Buffer::RemoveCharacter(_) => {
                            self.commit_buffer();
                        }
                    }
                }

                self.buffer = None;
                self.undo_actions.push(Action::Paste(PasteAction {
                    data,
                    start,
                    end,
                    selection,
                }))
            }
            UndoAction::RemoveCharacter(
                ch,
                (start_line, start_col),
                (end_line, end_col),
                remove_type,
            ) => match &mut self.buffer {
                Some(buffer) => match buffer {
                    Buffer::AddCharacter(_) => {
                        self.commit_buffer();
                        let mut remove_buffer =
                            RemoveCharacterBuffer::new(start_line, start_col, remove_type);
                        remove_buffer.remove_char(ch, end_line, end_col);
                        self.buffer = Some(Buffer::RemoveCharacter(remove_buffer));
                    }
                    Buffer::RemoveCharacter(remove_ch_buffer) => {
                        if remove_ch_buffer.remove_type != remove_type
                            || remove_ch_buffer.should_commit()
                        {
                            self.commit_buffer();
                            let mut remove_buffer: RemoveCharacterBuffer =
                                RemoveCharacterBuffer::new(start_line, start_col, remove_type);
                            remove_buffer.remove_char(ch, end_line, end_col);
                            self.buffer = Some(Buffer::RemoveCharacter(remove_buffer));
                        } else {
                            remove_ch_buffer.remove_char(ch, end_line, end_col);
                        }
                    }
                },
                None => {
                    let mut remove_ch_buffer =
                        RemoveCharacterBuffer::new(start_line, start_col, remove_type);
                    remove_ch_buffer.remove_char(ch, end_line, end_col);
                    self.buffer = Some(Buffer::RemoveCharacter(remove_ch_buffer))
                }
            },
        }
    }

    fn commit_buffer(&mut self) {
        if let Some(buffer) = self.buffer.take() {
            match buffer {
                Buffer::AddCharacter(add_character_buffer) => {
                    let action = AddAction {
                        start: add_character_buffer.start_position,
                        end: add_character_buffer.end_position,
                        chars: add_character_buffer.chars,
                        selection: add_character_buffer.selection,
                    };

                    self.undo_actions.push(Action::Add(action));
                }
                Buffer::RemoveCharacter(remove_character_buffer) => {
                    match remove_character_buffer.remove_type {
                        RemoveBufferType::Backspace => {
                            let data: String = remove_character_buffer.chars.iter().rev().collect();
                            let action = RemoveBackAction {
                                data,
                                start: remove_character_buffer.start_position,
                                end: remove_character_buffer.end_position,
                            };

                            self.undo_actions.push(Action::RemoveBack(action));
                        }
                        RemoveBufferType::Delete => {
                            let data: String = remove_character_buffer.chars.iter().collect();
                            let action = RemoveForwardAction {
                                data,
                                start: remove_character_buffer.start_position,
                                end: remove_character_buffer.end_position,
                            };

                            self.undo_actions.push(Action::RemoveForward(action));
                        }
                    }
                }
            }
        }
    }

    pub fn undo_action(&mut self, editor_state: &mut UIState) {
        self.commit_buffer();
        self.buffer = None;

        let Some(action) = self.undo_actions.pop() else {
            return;
        };

        match &action {
            Action::Add(add_action) => {
                // we don't need to use actual characters, as we delete them
                // using only start and end positions
                let (start_line, start_column) = add_action.start;
                let (end_line, end_column) = add_action.end;
                editor_state.cursor_line = start_line;
                editor_state.cursor_column = start_column;

                if end_line < start_line {
                    // SHOULD NEVER HAPPEN
                    return;
                }

                if start_line == end_line && end_column <= start_column {
                    // SHOULD NEVER HAPPEN
                    return;
                }

                editor_state.delete_range(add_action.start, add_action.end);

                self.insert_selection_back(&add_action.selection, editor_state);
            }
            Action::Paste(paste_action) => {
                editor_state.delete_range(paste_action.start, paste_action.end);
                editor_state.cursor_line = paste_action.start.0;
                editor_state.cursor_column = paste_action.start.1;

                self.insert_selection_back(&paste_action.selection, editor_state);
            }
            Action::RemoveBack(remove_back_action) => {
                // we should prepend all deleted whitespaces
                editor_state.insert_text(remove_back_action.data.clone(), false);
            }
            Action::RemoveForward(remove_forward_action) => {
                editor_state.insert_text(remove_forward_action.data.clone(), false);
                // we need to move the cursor to the beginning to replicate the state
                // before the user pressed delete
                editor_state.cursor_line = remove_forward_action.start.0;
                editor_state.cursor_column = remove_forward_action.start.1;
            }
        }

        self.redo_actions.push(action);
    }

    pub fn redo_action(&mut self, editor_state: &mut UIState) {
        let Some(action) = self.redo_actions.pop() else {
            return;
        };

        match &action {
            Action::Add(add_action) => {
                self.remove_selection(&add_action.selection, editor_state);

                // we need to put the cursor where new data started so that
                // it is inserted into the correct place; `insert_text` fn
                // will move the cursor automatically
                editor_state.cursor_line = add_action.start.0;
                editor_state.cursor_column = add_action.start.1;

                let data: String = add_action.chars.iter().collect();

                // we handle all the whitespaces when adding newlines
                editor_state.insert_text(data, false);

                // self.insert_selection_back(&add_action.selection, editor_state);
            }
            Action::Paste(paste_action) => {
                self.remove_selection(&paste_action.selection, editor_state);

                // put the cursor at the start to insert correctly
                editor_state.cursor_line = paste_action.start.0;
                editor_state.cursor_column = paste_action.start.1;

                // we insert whitespaces, because that's what happens when we paste as well
                // we need to clone the string, because we move the action to the undo vector
                editor_state.insert_text(paste_action.data.clone(), true);
            }
            Action::RemoveBack(remove_back_action) => {
                editor_state.delete_range(remove_back_action.end, remove_back_action.start);
                editor_state.cursor_line = remove_back_action.end.0;
                editor_state.cursor_column = remove_back_action.end.1;
            }
            Action::RemoveForward(remove_forward_action) => {
                editor_state.delete_range(remove_forward_action.start, remove_forward_action.end);
            }
        }

        self.undo_actions.push(action);
    }

    fn insert_selection_back(
        &self,
        selection_option: &Option<UndoSelection>,
        editor_state: &mut UIState,
    ) {
        let Some(selection) = &selection_option else {
            return;
        };
        editor_state.insert_text(selection.text.clone(), false);
        editor_state.set_selection(selection.start, selection.end);

        let (end_line, end_column) = selection.end;
        editor_state.cursor_line = end_line;
        editor_state.cursor_column = end_column;
    }

    fn remove_selection(
        &self,
        selection_option: &Option<UndoSelection>,
        editor_state: &mut UIState,
    ) {
        let Some(selection) = &selection_option else {
            return;
        };

        editor_state.set_selection(selection.start, selection.end);
        let (end_line, end_column) = selection.end;
        editor_state.cursor_line = end_line;
        editor_state.cursor_column = end_column;

        // we don't save the returned `Option<UndoSelection>` as it should
        // be exactly the same as the current one
        editor_state.delete_selection();
    }
}
