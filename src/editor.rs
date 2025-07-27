use ratatui::{
    Frame,
    layout::{Rect, Alignment},
    style::{Style, Stylize},
    widgets::{Block, Paragraph, Wrap},
    text::{Line, Span}
};

use crate::app_state::{AppState};

pub fn render_editor(frame: &mut Frame, area: Rect, app_state: &mut AppState) {
    app_state.ui_state.set_editor_offset(area.x, area.y);

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

pub fn generate_code_line(line: &str, current_line: usize, lines_number: usize) -> Line<'_> {
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