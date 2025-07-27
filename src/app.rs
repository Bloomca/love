use std::io;
use std::process::exit;
use std::env;
use std::path::PathBuf;
use std::fs;

use ratatui::{DefaultTerminal, Frame, layout::{Layout, Constraint}, widgets::{Block}};
use crossterm::event::{self, Event, KeyEventKind, KeyCode};

use crate::app_state::AppState;
use crate::editor::render_editor;
use crate::file_tree::render_file_tree;

pub fn start_tui_editor() -> io::Result<()> {
    let (file_content, directory_path) = get_file_from_args();
    let app_state = AppState::new(file_content, directory_path);

    let mut terminal = ratatui::init();
    let app_result = run(&mut terminal, &app_state);

    ratatui::restore();
    return app_result;
}

fn run(terminal: &mut DefaultTerminal, app_state: &AppState) -> io::Result<()> {
    loop {
        terminal.draw(|frame| render(frame, app_state))?;

        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                match key_event.code {
                    KeyCode::Char('q') => {
                        // TODO: add ctrl/cmd modifier
                        return Ok(());
                    },
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

fn render(frame: &mut Frame, app_state: &AppState) {
    let vertical = Layout::vertical([Constraint::Fill(1), Constraint::Length(3)]);
    let [main_area, status_area] = vertical.areas(frame.area());

    frame.render_widget(Block::bordered(), status_area);

    let horizontal = Layout::horizontal([Constraint::Length(50), Constraint::Fill(1)]);
    let [left_area, right_area] = horizontal.areas(main_area);

    render_file_tree(frame, left_area, app_state);
    render_editor(frame, right_area, app_state);
}

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