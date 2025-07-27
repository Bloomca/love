use ratatui::{
    Frame,
    layout::{Rect, Alignment},
    text::{Line, Span},
    widgets::{Block, Paragraph},
    style::{Style, Stylize}
};

use crate::app_state::{AppState, FileTreeEntry};

pub fn render_file_tree(frame: &mut Frame, area: Rect, app_state: &AppState) {
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