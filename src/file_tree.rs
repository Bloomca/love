use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Padding, Paragraph},
};

use crate::app_state::{AppState, FileTreeEntry};

pub fn render_file_tree(frame: &mut Frame, area: Rect, app_state: &AppState) {
    let file_tree = app_state
        .file_tree
        .get(&app_state.working_directory)
        .expect("Does not have top folder info");

    let text: Vec<Line> = file_tree
        .iter()
        .map(|entry| match entry {
            FileTreeEntry::File(file) => {
                let filename = file.path.file_name().unwrap().to_string_lossy().to_string();
                Line::from(vec![Span::raw(filename)])
            }
            FileTreeEntry::Dir(dir) => {
                let dirname = dir.path.file_name().unwrap().to_string_lossy().to_string();
                Line::from(vec![Span::raw(dirname)])
            }
        })
        .collect();

    let block = Block::bordered()
        .border_style(Style::default().fg(Color::Rgb(80, 80, 80)))
        .style(Style::default().bg(app_state.theme.bg_color))
        .padding(Padding::left(1));
    let file_explorer = Paragraph::new(text)
        .block(block)
        .style(Style::new().white())
        .alignment(Alignment::Left);

    frame.render_widget(file_explorer, area);
}
