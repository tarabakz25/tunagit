mod command_log;
mod detail;
mod footer;
mod sidebar;
mod theme;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    widgets::Block,
};

use crate::app::{App, ClickTarget, ScrollTarget, View};

pub fn render(frame: &mut Frame<'_>, app: &App) {
    frame.render_widget(
        Block::default().style(Style::default().bg(theme::BG)),
        frame.area(),
    );
    let (left, main, log, bottom) = regions(frame.area());

    sidebar::render(frame, left, app);
    detail::render(frame, main, app);
    command_log::render(frame, log, app);
    footer::render(frame, bottom, app);
}

fn regions(area: Rect) -> (Rect, Rect, Rect, Rect) {
    let [body, bottom] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(1)])
        .areas(area);
    let [left, right] = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(34), Constraint::Percentage(66)])
        .areas(body);
    let [main, log] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(3)])
        .areas(right);

    (left, main, log, bottom)
}

pub fn scroll_target(area: Rect, column: u16, row: u16, view: View) -> Option<ScrollTarget> {
    let (left, main, _, _) = regions(area);
    if matches!(view, View::PullRequests | View::Checks)
        && contains(sidebar::pr_area(left, view), column, row)
    {
        Some(ScrollTarget::PrList)
    } else if contains(main, column, row) || contains(left, column, row) {
        Some(ScrollTarget::Detail)
    } else {
        None
    }
}

pub fn click_target(area: Rect, column: u16, row: u16, app: &App) -> Option<ClickTarget> {
    let (left, main, _, _) = regions(area);
    if contains(left, column, row) {
        return sidebar::click_target(left, column, row, app);
    }
    if contains(main, column, row) {
        return Some(ClickTarget::Detail);
    }
    None
}

fn contains(area: Rect, column: u16, row: u16) -> bool {
    column >= area.x
        && column < area.x.saturating_add(area.width)
        && row >= area.y
        && row < area.y.saturating_add(area.height)
}
