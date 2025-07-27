use std::io;
use std::process::exit;
use std::env;
use std::path::PathBuf;
use std::fs;

use ratatui::{
    backend::{CrosstermBackend}, layout::{Constraint, Layout}, widgets::Block, Frame, Terminal
};
use crossterm::{event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind}, terminal};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::execute;

use crate::app_state::AppState;
use crate::editor::render_editor;
use crate::file_tree::render_file_tree;

pub fn start_tui_editor() -> io::Result<()> {
    let (file_content, directory_path) = get_file_from_args();
    let app_state = AppState::new(file_content, directory_path);

    let mut terminal = setup_terminal().expect("Failed to set up terminal");
    let app_result = run(&mut terminal, &app_state);

    ratatui::restore();
    return app_result;
}

fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app_state: &AppState) -> io::Result<()> {
    loop {
        terminal.draw(|frame| render(frame, app_state))?;

        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                match key_event.code {
                    // TODO: add ctrl/cmd modifier
                    KeyCode::Char('q') => {
                        restore_terminal(terminal).expect("Could not shut down the app gracefully, terminal might not work properly");
                        return Ok(());
                    },
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>, Box<dyn std::error::Error>> {
    let mut stdout = io::stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

/// critical to call this function on exit, otherwise it will not stop listening for events, so the
/// terminal will be unusable
fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<(), Box<dyn std::error::Error>> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    Ok(())
} 

/// Render general application layout without any specific details.
/// For now the layout is fixed, but the plan is to allow full customization
/// on what is shown and the position.
fn render(frame: &mut Frame, app_state: &AppState) {
    let vertical = Layout::vertical([Constraint::Fill(1), Constraint::Length(3)]);
    let [main_area, status_area] = vertical.areas(frame.area());

    frame.render_widget(Block::bordered(), status_area);

    let horizontal = Layout::horizontal([Constraint::Length(50), Constraint::Fill(1)]);
    let [left_area, right_area] = horizontal.areas(main_area);

    render_file_tree(frame, left_area, app_state);
    render_editor(frame, right_area, app_state);
}

/// Parse passed parameters to the CLI command. It can be either a folder
/// or a file (although folder is not supported right now).
fn get_file_from_args() -> (String, PathBuf) {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("Please specify a filename to open");
        exit(1);
    }

    let passed_path = args.last().expect("Could not read passed path");
    let mut canonical_path = fs::canonicalize(passed_path).expect("Could not read passed path");

    let file_string = fs::read_to_string(&canonical_path).expect("Could not open file");

    canonical_path.pop();

    return (file_string, canonical_path);
}