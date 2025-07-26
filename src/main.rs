use std::io;
use std::fs;
use std::env;
use std::process::exit;

use crossterm::event::{self, Event, KeyEventKind, KeyCode};
use ratatui::{
    DefaultTerminal,
    Frame,
    layout::{Layout, Constraint, Rect, Alignment},
    style::{Style, Stylize},
    widgets::{Block, Paragraph, Wrap},
    text::{Line, Span}
};

struct AppState {
    file_content: String,
    cursor_x: i32,
    cursor_y: i32,
    editor_offset_x: i32,
    editor_offset_y: i32
}

impl AppState {
    fn new(text: String) -> AppState {
        AppState { file_content: text, cursor_x: 0, cursor_y: 0, editor_offset_x: 0, editor_offset_y: 0 }
    }
}

fn main() -> io::Result<()> {
    let file_content = get_file_from_args();
    let app_state = AppState::new(file_content);

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

    frame.render_widget(Block::bordered().title("File explorer"), left_area);
    render_editor(frame, right_area, app_state);
}

fn render_editor(frame: &mut Frame, area: Rect, app_state: &AppState) {
    let lines_number = app_state.file_content.lines().count();
    let text: Vec<Line> = app_state.file_content
        .lines()
        .enumerate()
        .map(|(i, line)| generate_code_line(line, i, lines_number))
        .collect();

    let block = Block::bordered().title("Editor");
    let text_widget = Paragraph::new(text)
        .block(block)
        .style(Style::new().white().on_black())
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });

    frame.render_widget(text_widget, area);
}

fn generate_code_line(line: &str, current_line: usize, lines_number: usize) -> Line<'_> {
    let current_line_width = current_line.to_string().len();
    let lines_number_width = lines_number.to_string().len();

    let padding_times = lines_number_width - current_line_width;
    let padding_str = " ".repeat(padding_times);

    Line::from(vec![
        Span::styled(
            format!("|{}{} ", padding_str, current_line),
            Style::new().dark_gray()
        ),
        Span::raw(line)
    ])
}

fn get_file_from_args() -> String {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("Please specify a filename to open");
        exit(1);
    }

    let file_string = fs::read_to_string(args.last().unwrap()).expect("Could not open file");

    return file_string;
}
