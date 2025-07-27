use std::io;

mod editor;
mod app_state;
mod file_tree;
mod app;

use app::start_tui_editor;

fn main() -> io::Result<()> {
    start_tui_editor()
}
