use std::time::SystemTime;

const MAX_BUFFER_DEBOUNCE_TIME_MS: u128 = 250;

struct Action {
    // type: 'remove', text, position
    // type: 'add', text, start position, end position
}

pub enum UndoAction {
    AddCharacter(char, (usize, usize)),
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
    fn new() -> Self {
        AddCharacterBuffer {
            start_position: (0, 0),
            end_position: (0, 0),
            chars: vec![],
            last_action_timestamp: SystemTime::now(),
        }
    }

    fn should_commit(&self) -> bool {
        if let Ok(elapsed) = self.last_action_timestamp.elapsed() {
            return elapsed.as_millis() < MAX_BUFFER_DEBOUNCE_TIME_MS;
        }

        true
    }

    fn add_char(&mut self, ch: char) {
        self.chars.push(ch);
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
    /// pointer to the currently active action in actions
    /// by default it points to the last action, but
    pointer: Option<usize>,
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
            pointer: None,
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
            UndoAction::AddCharacter(ch, (line_num, col_num)) => match &mut self.buffer {
                Some(buffer) => match buffer {
                    Buffer::AddCharacter(add_character_buffer) => {
                        if add_character_buffer.should_commit() {
                            self.commit_buffer();
                            self.buffer = Some(Buffer::AddCharacter(AddCharacterBuffer::new()));
                        } else {
                            add_character_buffer.add_char(ch);
                        }
                    }
                    Buffer::RemoveCharacter(_) => {
                        self.commit_buffer();
                        self.buffer = Some(Buffer::AddCharacter(AddCharacterBuffer::new()));
                    }
                },
                None => self.buffer = Some(Buffer::AddCharacter(AddCharacterBuffer::new())),
            },
            UndoAction::Paste => todo!(),
            UndoAction::RemoveCharacter => todo!(),
        }
    }

    fn commit_buffer(&mut self) {
        if let Some(buffer) = &mut self.buffer {
            match buffer {
                Buffer::AddCharacter(add_character_buffer) => {
                    //
                }
                Buffer::RemoveCharacter(_) => {
                    //
                }
            }
        }
    }

    fn undo_action(&mut self) {
        self.commit_buffer();
        self.buffer = None;

        let Some(action) = self.undo_actions.pop() else {
            return;
        };

        // reverse action

        self.redo_actions.push(action);
    }

    fn redo_action(&mut self) {
        let Some(action) = self.redo_actions.pop() else {
            return;
        };

        // apply action

        self.undo_actions.push(action);
    }
}
