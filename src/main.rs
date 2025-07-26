use std::io;
use std::fs;
use std::env;
use std::path::PathBuf;
use std::process::exit;
use std::collections::HashMap;

use crossterm::event::{self, Event, KeyEventKind, KeyCode};
use ratatui::{
    DefaultTerminal,
    Frame,
    layout::{Layout, Constraint, Rect, Alignment},
    style::{Style, Stylize},
    widgets::{Block, Paragraph, Wrap},
    text::{Line, Span}
};

struct FileEntry {
    path: PathBuf
}

struct DirEntry {
    path: PathBuf,
    file_tree: HashMap<PathBuf, Vec<FileTreeEntry>>,
    expanded: bool
}

enum FileTreeEntry {
    File(FileEntry),
    Dir(DirEntry)
}

impl FileTreeEntry {
    fn new(path: PathBuf) -> FileTreeEntry {
        if path.is_dir() {
            FileTreeEntry::Dir(DirEntry { path, expanded: false, file_tree: HashMap::new() })
        } else {
            FileTreeEntry::File(FileEntry { path })
        }
    }
}

struct AppState {
    /// Directory of the entire project, can only be a single one
    /// It is either opened directly (like `love .`) or it is calculated
    /// based on the provided path
    working_directory: PathBuf,
    file_tree: HashMap<PathBuf, Vec<FileTreeEntry>>,
    file_content: String,
    cursor_x: i32,
    cursor_y: i32,
    editor_offset_x: i32,
    editor_offset_y: i32
}

impl AppState {
    fn new(file_content: String, working_directory: PathBuf) -> AppState {
        let mut app_state = AppState {
            working_directory,
            file_tree: HashMap::new(),
            file_content,
            cursor_x: 0,
            cursor_y: 0,
            editor_offset_x: 0,
            editor_offset_y: 0
        };

        app_state.read_directory(app_state.working_directory.clone());

        app_state
    }

    fn read_directory(&mut self, path: PathBuf) {
        // TODO: save the error as a directory state
        let dir_entries = fs::read_dir(&path).expect("Could not read directory");

        let values: Vec<FileTreeEntry> = dir_entries
            .filter_map(|result| result.ok())
            .map(|entry| FileTreeEntry::new(entry.path()))
            .collect();

        self.file_tree.insert(path, values);
            
    }
}

fn main() -> io::Result<()> {
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

fn render_file_tree(frame: &mut Frame, area: Rect, app_state: &AppState) {
    let file_tree = app_state.file_tree
        .get(&app_state.working_directory)
        .expect("Does not have top folder info");

    let text: Vec<Line> = file_tree.iter().map(|entry| {
        match entry {
            FileTreeEntry::File(file) => {
                let filename = file.path.file_name().unwrap().to_string_lossy().to_string();
                Line::from(vec![
                    Span::raw(filename)
                ])
            }
            FileTreeEntry::Dir(dir) => {
                let dirname = dir.path.file_name().unwrap().to_string_lossy().to_string();
                Line::from(vec![
                    Span::raw(dirname)
                ])
            }
        }
    }).collect();

    let block = Block::bordered().title("File explorer");
    let file_explorer = Paragraph::new(text)
        .block(block)
        .style(Style::new().white().on_black())
        .alignment(Alignment::Left);

    frame.render_widget(file_explorer, area);
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
