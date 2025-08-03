use ratatui::{
    Frame,
    layout::{Constraint, Layout},
};

use crate::app_state::AppState;
use crate::editor::render_editor;
use crate::file_tree::render_file_tree;
use crate::status_bar::render_status_bar;

/// Render general application layout without any specific details.
/// For now the layout is fixed, but the plan is to allow full customization
/// on what is shown and the position.
pub(super) fn render(frame: &mut Frame, app_state: &mut AppState) {
    let vertical = Layout::vertical([Constraint::Fill(1), Constraint::Length(3)]);
    let [main_area, status_area] = vertical.areas(frame.area());

    render_status_bar(frame, status_area, app_state);

    let horizontal = Layout::horizontal([Constraint::Length(50), Constraint::Fill(1)]);
    let [left_area, right_area] = horizontal.areas(main_area);

    render_file_tree(frame, left_area, app_state);
    render_editor(frame, right_area, app_state);
}
