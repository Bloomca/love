use std::io;

use crossterm::event::{self, Event, KeyEventKind, KeyCode};
use ratatui::{DefaultTerminal, Frame};

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
    frame.render_widget("Hello, world!", frame.area());
}
