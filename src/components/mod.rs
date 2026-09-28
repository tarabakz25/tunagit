mod command_log;
mod detail;
mod footer;
mod sidebar;
mod theme;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::Style,
    widgets::Block,
};

use crate::app::App;

pub fn render(frame: &mut Frame<'_>, app: &App) {
    frame.render_widget(
        Block::default().style(Style::default().bg(theme::BG)),
        frame.area(),
    );
    let [body, bottom] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(1)])
        .areas(frame.area());
    let [left, right] = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(34), Constraint::Percentage(66)])
        .areas(body);
    let [main, log] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(3)])
        .areas(right);

    sidebar::render(frame, left, app);
    detail::render(frame, main, app);
    command_log::render(frame, log, app);
    footer::render(frame, bottom, app);
}
