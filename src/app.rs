mod terminal_setup;
mod run_event_loop;
mod render_app_layout;
mod parse_args;

use std::io;

use crate::{app_state::AppState};
use terminal_setup::setup_terminal;
use run_event_loop::run;
use parse_args::get_file_from_args;

pub fn start_tui_editor() -> io::Result<()> {
    let (file_content, directory_path) = get_file_from_args();
    let mut app_state = AppState::new(file_content, directory_path);

    // TODO: think where it should be initialized
    app_state.read_directory(app_state.working_directory.clone());

    let mut terminal = setup_terminal().expect("Failed to set up terminal");
    let app_result = run(&mut terminal, &mut app_state);

    ratatui::restore();
    return app_result;
}

