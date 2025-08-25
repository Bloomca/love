use std::io;

mod app;
mod app_state;
mod editor;
mod file_tree;
mod status_bar;
mod syntax_highlighter;

use app::start_tui_editor;

fn main() -> io::Result<()> {
    start_tui_editor()
}
