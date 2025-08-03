use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::Style,
    widgets::{Block, Paragraph},
};

use crate::app_state::AppState;

pub fn render_status_bar(frame: &mut Frame, area: Rect, app_state: &AppState) {
    let block = Block::default().style(Style::default().bg(app_state.theme.status_bar_color));

    let line_num = app_state.ui_state.cursor_line;
    let column_num = app_state.ui_state.cursor_column;
    let formatted_text = format!("Line {line_num} | Column {column_num}");
    let text = Paragraph::new(formatted_text)
        .block(block)
        .alignment(Alignment::Right);

    frame.render_widget(text, area);
}
