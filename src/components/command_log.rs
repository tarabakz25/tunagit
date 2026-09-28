use ratatui::{Frame, layout::Rect};

use super::theme;
use crate::app::App;

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let message = if app.notice.is_empty() {
        &app.command_log
    } else {
        &app.notice
    };
    frame.render_widget(theme::text_panel("Command log", message, false, 0), area);
}
