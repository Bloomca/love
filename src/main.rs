use std::io;
use std::fs;
use std::env;
use std::process::exit;

use crossterm::event::{self, Event, KeyEventKind, KeyCode};
use ratatui::{
    DefaultTerminal, Frame, layout::{Layout, Constraint},
    widgets::Block
};

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let app_result = run(&mut terminal);

    ratatui::restore();
    return app_result;
}

fn run(terminal: &mut DefaultTerminal) -> io::Result<()> {
    loop {
        terminal.draw(render)?;

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

fn render(frame: &mut Frame) {
    let vertical = Layout::vertical([Constraint::Fill(1), Constraint::Length(3)]);
    let [main_area, status_area] = vertical.areas(frame.area());

    frame.render_widget(Block::bordered(), status_area);

    let horizontal = Layout::horizontal([Constraint::Length(50), Constraint::Fill(1)]);
    let [left_area, right_area] = horizontal.areas(main_area);

    frame.render_widget(Block::bordered().title("File explorer"), left_area);
    frame.render_widget(Block::bordered().title("Editor"), right_area);
}
