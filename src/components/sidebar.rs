use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, ListState},
};

use super::theme;
use crate::app::{App, ClickTarget, PrFocus, View};

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let [status, files, branches, commits, stash, prs] = panel_areas(area);

    let repo = if app.git.repo.is_empty() {
        "loading…"
    } else {
        &app.git.repo
    };
    let branch = if app.git.branch.is_empty() {
        "…"
    } else {
        &app.git.branch
    };
    frame.render_widget(
        theme::text_panel(
            "[1] Status",
            &format!("{repo} → {branch}"),
            app.view == View::Overview,
            0,
        ),
        status,
    );
    frame.render_widget(
        theme::text_panel(
            "[2] Files",
            &preview(&app.git.files, "Working tree clean", 3),
            app.view == View::Git,
            0,
        ),
        files,
    );
    let branch_preview = if app.git.branches.is_empty() || app.git.branches == "(no output)" {
        format!("* {branch}")
    } else {
        preview(&app.git.branches, "No branches", 2)
    };
    frame.render_widget(
        theme::text_panel(
            "[3] Local branches",
            &branch_preview,
            app.view == View::Branches,
            0,
        ),
        branches,
    );
    let commit_preview = if app.git.commits.contains("does not have any commits") {
        "No commits yet".to_string()
    } else {
        preview(&app.git.commits, "No commits", 1)
    };
    frame.render_widget(
        theme::text_panel("[4] Commits", &commit_preview, app.view == View::Commits, 0),
        commits,
    );
    frame.render_widget(
        theme::text_panel(
            "[5] Stash",
            &preview(&app.git.stash, "Stash empty", 1),
            app.view == View::Stash,
            0,
        ),
        stash,
    );
    render_prs(frame, prs, app);
}

pub(super) fn panel_areas(area: Rect) -> [Rect; 6] {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Length(5),
            Constraint::Length(4),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(4),
        ])
        .areas(area)
}

pub(super) fn pr_area(area: Rect) -> Rect {
    panel_areas(area)[5]
}

pub(super) fn click_target(area: Rect, column: u16, row: u16, app: &App) -> Option<ClickTarget> {
    let panels = panel_areas(area);
    let views = [
        View::Overview,
        View::Git,
        View::Branches,
        View::Commits,
        View::Stash,
    ];
    for (panel, view) in panels.iter().take(5).zip(views) {
        if contains(panel, column, row) {
            return Some(ClickTarget::SidebarView(view));
        }
    }
    let prs = panels[5];
    if !contains(&prs, column, row) {
        return None;
    }
    if app.pull_requests.is_empty() {
        return Some(ClickTarget::SidebarView(View::PullRequests));
    }
    if row == prs.y
        || row + 1 >= prs.y.saturating_add(prs.height)
        || column == prs.x
        || column + 1 >= prs.x.saturating_add(prs.width)
    {
        return Some(ClickTarget::SidebarView(View::PullRequests));
    }
    let visible = prs.height.saturating_sub(2) as usize;
    if visible == 0 {
        return Some(ClickTarget::SidebarView(View::PullRequests));
    }
    let offset = app.selected_pr.saturating_sub(visible.saturating_sub(1));
    let index = offset.saturating_add(row.saturating_sub(prs.y + 1) as usize);
    if index < app.pull_requests.len() {
        Some(ClickTarget::PrIndex(index))
    } else {
        Some(ClickTarget::SidebarView(View::PullRequests))
    }
}

fn contains(area: &Rect, column: u16, row: u16) -> bool {
    column >= area.x
        && column < area.x.saturating_add(area.width)
        && row >= area.y
        && row < area.y.saturating_add(area.height)
}

fn preview(text: &str, empty: &str, max_lines: usize) -> String {
    if text.is_empty() || text == "(no output)" {
        empty.to_string()
    } else {
        text.lines().take(max_lines).collect::<Vec<_>>().join("\n")
    }
}

fn render_prs(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let mut items = Vec::new();
    if app.pull_requests.is_empty() {
        let message = if app.pr_message.starts_with('0') {
            "No pull requests"
        } else if app.pr_message.starts_with("Press") {
            "Loading…"
        } else {
            "gh unavailable"
        };
        items.push(ListItem::new(message).style(Style::default().fg(theme::MUTED)));
    }
    for pr in &app.pull_requests {
        let color = if pr.draft {
            theme::ACCENT
        } else if pr.state == "MERGED" {
            theme::PURPLE
        } else {
            theme::GREEN
        };
        items.push(ListItem::new(Line::from(vec![
            Span::styled(
                format!("#{:<4}", pr.number),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(pr.title.clone(), Style::default().fg(theme::FG)),
        ])));
    }
    let list = List::new(items)
        .block(theme::panel(
            "[6] Pull requests",
            matches!(app.view, View::PullRequests | View::Checks) && app.pr_focus == PrFocus::List,
        ))
        .highlight_style(
            Style::default()
                .bg(theme::SELECTED)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");
    let mut state = ListState::default();
    state.select((!app.pull_requests.is_empty()).then_some(app.selected_pr));
    frame.render_stateful_widget(list, area, &mut state);
}
