use crate::app_state::editor::UIState;

impl UIState {
    pub fn handle_paste(&mut self, data: String) {
        self.vertical_offset_target = 0;

        self.delete_selection();

        let total_pasted_lines = data.lines().count();
        let lines = data.lines().enumerate().collect::<Vec<_>>();

        for (i, line) in lines {
            if i == 0 {
                // 1. get the current line
                // 2. append the new line to it
                // 3. if there is only one line, move cursor to it
            } else {
                // 1. get the previous line and calculate whitespaces
                // 2. create a new line with those whitespaces and add new data
                // 3. append that new line in the new index (self.cursor_line + i)
                // 4. if that is the last line (i + 1 == total_pasted_lines), put cursor at the end
            }
        }
    }

    pub fn handle_copy(&self) {
        // 1. read selection, if none do nothing
        // 2. copy selection (join all strings with '\n' character)
        // 3. Execute a crossterm command: https://docs.rs/crossterm/0.29.0/crossterm/clipboard/struct.CopyToClipboard.html
    }
}
