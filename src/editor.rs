use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Paragraph},
};

use crate::app_state::AppState;

pub fn render_editor(frame: &mut Frame, area: Rect, app_state: &mut AppState) {
    app_state
        .ui_state
        .set_editor_offset(area.x as usize, area.y as usize, area.height as usize);

    let selection_exists = app_state.ui_state.has_any_selection();

    let lines_number = app_state.ui_state.lines.len();
    let text: Vec<Line> = app_state
        .ui_state
        .lines
        .iter()
        .skip(app_state.ui_state.editor_scroll_offset)
        .take(app_state.ui_state.editor_lines_num)
        .enumerate()
        .map(|(i, line)| {
            generate_code_line(
                line,
                i + 1 + app_state.ui_state.editor_scroll_offset,
                lines_number,
                selection_exists,
                app_state,
            )
        })
        .collect();

    let block = Block::bordered().title("Editor");
    let text_widget = Paragraph::new(text)
        .block(block)
        .style(Style::new().white().on_black())
        .alignment(Alignment::Left);

    frame.render_widget(text_widget, area);
}

fn generate_code_line(
    line: &[char],
    current_line: usize,
    lines_number: usize,
    selection_exists: bool,
    app_state: &AppState,
) -> Line<'static> {
    let current_line_width = current_line.to_string().len();
    let lines_number_width = lines_number.to_string().len();

    let padding_times = lines_number_width - current_line_width;
    let padding_str = " ".repeat(padding_times);

    let mut result = vec![Span::styled(
        format!("|{padding_str}{current_line} "),
        Style::new().dark_gray(),
    )];
    let mut code_line_spans =
        generate_styled_code_line(line, current_line, selection_exists, app_state);

    result.append(&mut code_line_spans);

    Line::from(result)
}

fn generate_styled_code_line(
    line: &[char],
    current_line_num: usize,
    selection_exists: bool,
    app_state: &AppState,
) -> Vec<Span<'static>> {
    if !selection_exists {
        return generate_line_with_no_selection(line);
    }

    if app_state.ui_state.is_entire_line_selected(current_line_num) {
        return generate_line_with_full_selection(line);
    }

    if let Some((min_col, max_col)) = app_state.ui_state.get_selection_range(current_line_num) {
        return generate_line_with_selection(line, min_col, max_col);
    } else {
        return generate_line_with_no_selection(line);
    }
}

fn generate_line_with_no_selection(line: &[char]) -> Vec<Span<'static>> {
    vec![Span::raw(line.iter().collect::<String>())]
}

fn generate_line_with_full_selection(line: &[char]) -> Vec<Span<'static>> {
    vec![Span::styled(
        line.iter().collect::<String>(),
        Style::new().bg(Color::Blue),
    )]
}

fn generate_line_with_selection(
    line: &[char],
    min_col: usize,
    max_col: usize,
) -> Vec<Span<'static>> {
    let mut result = vec![];

    if line.is_empty() {
        return result;
    }

    if min_col == 0 {
        if max_col > 0 {
            if max_col > line.len() {
                result.push(Span::styled(
                    line[0..line.len()].iter().collect::<String>(),
                    Style::new().bg(Color::Blue),
                ));
            } else {
                result.push(Span::styled(
                    line[0..max_col].iter().collect::<String>(),
                    Style::new().bg(Color::Blue),
                ));
                result.push(Span::raw(line[max_col..].iter().collect::<String>()));
            }
        } else {
            result.push(Span::styled(
                line[min_col..line.len()].iter().collect::<String>(),
                Style::new().bg(Color::Blue),
            ));
        }
    } else {
        if min_col > 1 {
            result.push(Span::raw(line[0..(min_col - 1)].iter().collect::<String>()));
        }

        if max_col > 0 {
            if max_col > line.len() {
                 result.push(Span::styled(
                    line[(min_col - 1)..line.len()].iter().collect::<String>(),
                    Style::new().bg(Color::Blue),
                ));
            } else {
                result.push(Span::styled(
                    line[(min_col - 1)..max_col].iter().collect::<String>(),
                    Style::new().bg(Color::Blue),
                ));
                result.push(Span::raw(line[max_col..].iter().collect::<String>()));        
            }
        } else {
            result.push(Span::styled(
                line[(min_col - 1)..line.len()].iter().collect::<String>(),
                Style::new().bg(Color::Blue),
            ));
        }
    }

    result
}
