mod parse_args;
mod render_app_layout;
mod run_event_loop;
mod terminal_setup;

use std::io;

use crate::app_state::AppState;
use parse_args::get_file_from_args;
use run_event_loop::run;
use terminal_setup::setup_terminal;

pub fn start_tui_editor() -> io::Result<()> {
    let (file_content, directory_path) = get_file_from_args();
    let mut app_state = AppState::new(file_content, directory_path);

    // TODO: think where it should be initialized
    app_state.read_directory(app_state.working_directory.clone());

    let mut terminal = setup_terminal().expect("Failed to set up terminal");
    let app_result = run(&mut terminal, &mut app_state);

    ratatui::restore();
    app_result
}
