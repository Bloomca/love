use std::io;

use ratatui::{backend::{CrosstermBackend}, Terminal};
use crossterm::{event::{self, Event, KeyCode, KeyEventKind}};

use crate::app_state::AppState;
use super::terminal_setup::restore_terminal;
use super::render_app_layout::render;

pub(super) fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app_state: &mut AppState) -> io::Result<()> {
    loop {
        terminal.draw(|frame| render(frame, app_state))?;

        app_state.ui_state.show_cursor_if_needed();

        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                match key_event.code {
                    // TODO: add ctrl/cmd modifier
                    KeyCode::Char(character) => {
                        if character == 'q' {
                            restore_terminal(terminal).expect("Could not shut down the app gracefully, terminal might not work properly");
                            return Ok(());
                        }

                        app_state.ui_state.insert_character(character);
                    }
                    KeyCode::Left => app_state.ui_state.cursor_move_left(),
                    KeyCode::Right => app_state.ui_state.cursor_move_right(),
                    KeyCode::Down => app_state.ui_state.cursor_move_down(),
                    KeyCode::Up => app_state.ui_state.cursor_move_up(),
                    KeyCode::Backspace => app_state.ui_state.remove_previous_character(),
                    KeyCode::Delete => app_state.ui_state.remove_next_character(),
                    _ => {}
                }
            }
            _ => {}
        }
    }
}