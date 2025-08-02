use std::collections::HashMap;

pub enum SelectionType {
    Line,
    To(usize),
    From(usize),
    Range(usize, usize),
}

pub struct Selection {
    pub start: (usize, usize),
    pub end: (usize, usize),
    cache: HashMap<usize, SelectionType>,
}

impl Selection {
    pub fn new(current_cursor_line: usize, current_cursor_col: usize) -> Self {
        Selection {
            start: (current_cursor_line, current_cursor_col),
            end: (current_cursor_line, current_cursor_col),
            cache: HashMap::new(),
        }
    }

    pub fn set_end(&mut self, current_cursor_line: usize, current_cursor_col: usize) {
        // we recalculate cache completely in order to avoid clearing
        // it atomically
        self.cache = HashMap::new();

        self.end = (current_cursor_line, current_cursor_col);

        let (start_line, start_column) = self.start;

        if start_line == current_cursor_line {
            self.cache
                .entry(start_line)
                .insert_entry(SelectionType::Range(
                    start_column.min(current_cursor_col),
                    start_column.max(current_cursor_col),
                ));
        } else {
            let min_line = start_line.min(current_cursor_line);
            let max_line = start_line.max(current_cursor_line);

            let (min_line_column, max_line_column) = if start_line < current_cursor_line {
                (start_column, current_cursor_col)
            } else {
                (current_cursor_col, start_column)
            };

            for line_num in min_line..=max_line {
                let entry = match line_num {
                    _ if line_num == min_line => SelectionType::From(min_line_column),
                    _ if line_num == max_line => SelectionType::To(max_line_column),
                    _ => SelectionType::Line,
                };

                self.cache.entry(line_num).insert_entry(entry);
            }
        }
    }

    pub fn is_line_selected(&self, line: usize) -> bool {
        match self.cache.get(&line) {
            Some(selection_type) => match selection_type {
                SelectionType::Line => true,
                SelectionType::To(_) => false,
                SelectionType::From(_) => false,
                SelectionType::Range(_, _) => false,
            },
            None => false,
        }
    }

    pub fn get_selection_range(&self, line: usize) -> Option<(usize, usize)> {
        match self.cache.get(&line) {
            Some(selection_type) => match selection_type {
                SelectionType::Line => None,
                SelectionType::To(column) => Some((0, *column)),
                SelectionType::From(column) => Some((*column, 0)),
                SelectionType::Range(min_col, max_col) => Some((*min_col, *max_col)),
            },
            None => None,
        }
    }

    pub fn get_first_position(&self) -> (usize, usize) {
        let (start_line, start_column) = self.start;
        let (end_line, end_column) = self.end;
        let direction_forward = (end_line, end_column) > (start_line, start_column);

        if direction_forward {
            self.start
        } else {
            self.end
        }
    }
}
