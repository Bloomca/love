use std::time::SystemTime;

use crate::app_state::editor::UIState;

const MAX_BUFFER_DEBOUNCE_TIME_MS: u128 = 250;

struct AddAction {
    start: (usize, usize),
    end: (usize, usize),
    chars: Vec<char>,
}

struct RemoveAction {
    // pass
}

struct PasteAction {
    // pass
}

enum Action {
    Add(AddAction),
    Remove(RemoveAction),
    Paste(PasteAction),
}

pub enum UndoAction {
    /// character, starting position, ending position
    AddCharacter(char, (usize, usize), (usize, usize)),
    Paste,
    RemoveCharacter,
}

struct AddCharacterBuffer {
    start_position: (usize, usize),
    end_position: (usize, usize),
    chars: Vec<char>,
    last_action_timestamp: SystemTime,
}

impl AddCharacterBuffer {
    fn new(line_num: usize, line_column: usize) -> Self {
        AddCharacterBuffer {
            start_position: (line_num, line_column),
            end_position: (line_num, line_column),
            chars: vec![],
            last_action_timestamp: SystemTime::now(),
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

struct RemoveCharacterBuffer {
    start_position: (usize, usize),
    chars: Vec<char>,
    last_action_timestamp: SystemTime,
}

enum Buffer {
    AddCharacter(AddCharacterBuffer),
    RemoveCharacter(RemoveCharacterBuffer),
}

pub struct UndoRedo {
    max_actions_memory: usize,
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
            max_actions_memory: 20,
            undo_actions: Vec::new(),
            redo_actions: Vec::new(),
            buffer: None,
        }
    }

    pub fn add_undo_action(&mut self, action: UndoAction) {
        // if we perform any action, all redo actions are immediately invalidated
        self.redo_actions.clear();
        match action {
            UndoAction::AddCharacter(ch, (start_line, start_col), (end_line, end_col)) => {
                match &mut self.buffer {
                    Some(buffer) => match buffer {
                        Buffer::AddCharacter(add_character_buffer) => {
                            if add_character_buffer.should_commit() {
                                self.commit_buffer();
                                let mut add_buffer: AddCharacterBuffer =
                                    AddCharacterBuffer::new(start_line, start_col);
                                add_buffer.add_char(ch, end_line, end_col);
                                self.buffer = Some(Buffer::AddCharacter(add_buffer));
                            } else {
                                add_character_buffer.add_char(ch, end_line, end_col);
                            }
                        }
                        Buffer::RemoveCharacter(_) => {
                            self.commit_buffer();
                            let mut add_buffer = AddCharacterBuffer::new(start_line, start_col);
                            add_buffer.add_char(ch, end_line, end_col);
                            self.buffer = Some(Buffer::AddCharacter(add_buffer));
                        }
                    },
                    None => {
                        let mut add_buffer = AddCharacterBuffer::new(start_line, start_col);
                        add_buffer.add_char(ch, end_line, end_col);
                        self.buffer = Some(Buffer::AddCharacter(add_buffer))
                    }
                }
            }
            UndoAction::Paste => todo!(),
            UndoAction::RemoveCharacter => todo!(),
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
                    };

                    self.undo_actions.push(Action::Add(action));
                }
                Buffer::RemoveCharacter(_) => {
                    //
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

                if start_line == end_line {
                    if end_column <= start_column {
                        // SHOULD NEVER HAPPEN
                        return;
                    }

                    if let Some(line) = editor_state.lines.get_mut(start_line - 1) {
                        line.drain((start_column - 1)..(end_column - 1));
                    }
                } else {
                    //
                }
            }
            Action::Remove(remove_action) => todo!(),
            Action::Paste(paste_action) => todo!(),
        }

        self.redo_actions.push(action);
    }

    pub fn redo_action(&mut self, editor_state: &mut UIState) {
        let Some(action) = self.redo_actions.pop() else {
            return;
        };

        match &action {
            Action::Add(add_action) => {
                let (start_line, start_column) = add_action.start;
                let (end_line, end_column) = add_action.end;

                if start_line == end_line {
                    if let Some(line) = editor_state.lines.get_mut(start_line - 1) {
                        add_action.chars.iter().enumerate().for_each(|(i, char)| {
                            line.insert(start_column - 1 + i, *char);
                        });
                    }
                }

                // TODO: handle multiline

                editor_state.cursor_line = end_line;
                editor_state.cursor_column = end_column;
            }
            Action::Remove(remove_action) => todo!(),
            Action::Paste(paste_action) => todo!(),
        }

        self.undo_actions.push(action);
    }
}
