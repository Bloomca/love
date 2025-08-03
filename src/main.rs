use std::io;

mod app;
mod app_state;
mod editor;
mod file_tree;
mod status_bar;

use app::start_tui_editor;

fn main() -> io::Result<()> {
    start_tui_editor()
}
