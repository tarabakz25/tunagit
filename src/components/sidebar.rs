use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, ListState, Paragraph},
};

use super::theme;
use crate::app::{App, ClickTarget, PrFocus, View};

const TITLES: [&str; 6] = [
    "Status",
    "Files",
    "Local branches",
    "Commits",
    "Stash",
    "Pull requests",
];

pub fn active_section(view: View) -> Option<usize> {
    match view {
        View::Overview => Some(0),
        View::Git => Some(1),
        View::Branches => Some(2),
        View::Commits => Some(3),
        View::Stash => Some(4),
        View::PullRequests | View::Checks => Some(5),
        View::Output => None,
    }
}

fn layout_active(view: View) -> usize {
    active_section(view).unwrap_or(5)
}

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let active = active_section(app.view);
    let expanded = layout_active(app.view);
    let areas = panel_areas(area, expanded);
    for (index, section) in areas.iter().enumerate() {
        if index == expanded {
            render_expanded(frame, *section, app, index, active == Some(index));
        } else {
            render_collapsed(frame, *section, app, index);
        }
    }
}

pub(super) fn panel_areas(area: Rect, expanded: usize) -> [Rect; 6] {
    let mut constraints = [Constraint::Length(1); 6];
    if let Some(slot) = constraints.get_mut(expanded) {
        *slot = Constraint::Min(3);
    }
    Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .areas(area)
}

pub(super) fn pr_area(area: Rect, view: View) -> Rect {
    panel_areas(area, layout_active(view))[5]
}

pub(super) fn click_target(area: Rect, column: u16, row: u16, app: &App) -> Option<ClickTarget> {
    let panels = panel_areas(area, layout_active(app.view));
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

fn repo_name(app: &App) -> &str {
    if app.git.repo.is_empty() {
        "loading…"
    } else {
        &app.git.repo
    }
}

fn branch_name(app: &App) -> &str {
    if app.git.branch.is_empty() {
        "…"
    } else {
        &app.git.branch
    }
}

fn render_expanded(frame: &mut Frame<'_>, area: Rect, app: &App, index: usize, active: bool) {
    let rows = area.height.saturating_sub(2) as usize;
    match index {
        0 => frame.render_widget(
            theme::text_panel(
                "[1] Status",
                &format!("{} → {}", repo_name(app), branch_name(app)),
                active,
                0,
            ),
            area,
        ),
        1 => frame.render_widget(
            theme::text_panel(
                "[2] Files",
                &preview(&app.git.files, "Working tree clean", rows),
                active,
                0,
            ),
            area,
        ),
        2 => {
            let branch_preview = if app.git.branches.is_empty() || app.git.branches == "(no output)"
            {
                format!("* {}", branch_name(app))
            } else {
                preview(&app.git.branches, "No branches", rows)
            };
            frame.render_widget(
                theme::text_panel("[3] Local branches", &branch_preview, active, 0),
                area,
            );
        }
        3 => {
            let commit_preview = if app.git.commits.contains("does not have any commits") {
                "No commits yet".to_string()
            } else {
                preview(&app.git.commits, "No commits", rows)
            };
            frame.render_widget(
                theme::text_panel("[4] Commits", &commit_preview, active, 0),
                area,
            );
        }
        4 => frame.render_widget(
            theme::text_panel(
                "[5] Stash",
                &preview(&app.git.stash, "Stash empty", rows),
                active,
                0,
            ),
            area,
        ),
        _ => render_prs(frame, area, app, active),
    }
}

fn render_collapsed(frame: &mut Frame<'_>, area: Rect, app: &App, index: usize) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let width = area.width as usize;
    let summary = collapsed_summary(app, index);
    let head = format!("─[{}] {} ", index + 1, TITLES[index]);
    let tail = if summary.is_empty() {
        String::new()
    } else {
        format!("{summary} ─")
    };
    let head_len = head.chars().count();
    let line = if head_len >= width {
        Line::from(Span::styled(
            head.chars().take(width).collect::<String>(),
            Style::default()
                .fg(theme::BORDER)
                .add_modifier(Modifier::BOLD),
        ))
    } else {
        let tail = if head_len + tail.chars().count() > width {
            tail.chars().take(width - head_len).collect::<String>()
        } else {
            tail
        };
        let fill = width - head_len - tail.chars().count();
        Line::from(vec![
            Span::styled(
                head,
                Style::default()
                    .fg(theme::BORDER)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("─".repeat(fill), Style::default().fg(theme::MUTED)),
            Span::styled(tail, Style::default().fg(theme::MUTED)),
        ])
    };
    frame.render_widget(
        Paragraph::new(line).style(Style::default().bg(theme::PANEL)),
        area,
    );
}

fn collapsed_summary(app: &App, index: usize) -> String {
    match index {
        0 => format!(
            "{} → {}",
            one_line(repo_name(app)),
            one_line(branch_name(app))
        ),
        1 => count_lines(&app.git.files).to_string(),
        2 => {
            if app.git.branches.is_empty() || app.git.branches == "(no output)" {
                "1".to_string()
            } else {
                count_lines(&app.git.branches).to_string()
            }
        }
        3 => {
            if app.git.commits.contains("does not have any commits") {
                "0".to_string()
            } else {
                count_lines(&app.git.commits).to_string()
            }
        }
        4 => count_lines(&app.git.stash).to_string(),
        _ => {
            if app.pull_requests.is_empty() {
                "0".to_string()
            } else {
                let current = app
                    .selected_pr
                    .min(app.pull_requests.len().saturating_sub(1))
                    + 1;
                format!("{current} of {}", app.pull_requests.len())
            }
        }
    }
}

fn one_line(text: &str) -> &str {
    text.lines().next().unwrap_or(text)
}

fn count_lines(text: &str) -> usize {
    if text.is_empty() || text == "(no output)" {
        0
    } else {
        text.lines().count()
    }
}

fn preview(text: &str, empty: &str, max_lines: usize) -> String {
    if text.is_empty() || text == "(no output)" {
        empty.to_string()
    } else {
        text.lines().take(max_lines).collect::<Vec<_>>().join("\n")
    }
}

fn render_prs(frame: &mut Frame<'_>, area: Rect, app: &App, active: bool) {
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
            active && app.pr_focus == PrFocus::List,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::JobResult;

    fn test_app() -> App {
        let (tx, rx) = std::sync::mpsc::channel::<JobResult>();
        App::new(tx, rx)
    }

    #[test]
    fn active_section_maps_views() {
        assert_eq!(active_section(View::Overview), Some(0));
        assert_eq!(active_section(View::Git), Some(1));
        assert_eq!(active_section(View::Branches), Some(2));
        assert_eq!(active_section(View::Commits), Some(3));
        assert_eq!(active_section(View::Stash), Some(4));
        assert_eq!(active_section(View::PullRequests), Some(5));
        assert_eq!(active_section(View::Checks), Some(5));
        assert_eq!(active_section(View::Output), None);
    }

    #[test]
    fn expanded_panel_takes_remaining_height() {
        let area = Rect::new(0, 0, 40, 20);
        for expanded in 0..6 {
            let areas = panel_areas(area, expanded);
            for (index, section) in areas.iter().enumerate() {
                if index == expanded {
                    assert_eq!(section.height, 15, "expanded {expanded}");
                } else {
                    assert_eq!(section.height, 1, "section {index} collapsed");
                }
            }
            let mut y = 0;
            for section in &areas {
                assert_eq!(section.y, y);
                assert_eq!(section.x, 0);
                assert_eq!(section.width, 40);
                y += section.height;
            }
        }
    }

    #[test]
    fn collapsed_summaries_count_items() {
        let mut app = test_app();
        app.git.repo = "tunagit".into();
        app.git.branch = "master".into();
        app.git.files = " M src/main.rs\n?? new.txt".into();
        app.git.branches = "* master\n  feature".into();
        app.git.commits = "abc first\ndef second".into();
        app.git.stash = String::new();
        assert_eq!(collapsed_summary(&app, 0), "tunagit → master");
        assert_eq!(collapsed_summary(&app, 1), "2");
        assert_eq!(collapsed_summary(&app, 2), "2");
        assert_eq!(collapsed_summary(&app, 3), "2");
        assert_eq!(collapsed_summary(&app, 4), "0");
        assert_eq!(collapsed_summary(&app, 5), "0");
    }

    #[test]
    fn sidebar_renders_accordion() {
        use ratatui::{Terminal, backend::TestBackend};

        let mut app = test_app();
        app.git.repo = "tunagit".into();
        app.git.branch = "master".into();
        app.git.files = " M src/main.rs".into();
        let backend = TestBackend::new(40, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| render(frame, frame.area(), &app))
            .unwrap();
        let buffer = terminal.backend().buffer().clone();
        let rows: Vec<String> = (0..20)
            .map(|y| {
                (0..40)
                    .map(|x| buffer.cell((x, y)).unwrap().symbol().to_string())
                    .collect()
            })
            .collect();
        assert!(rows[0].contains("[1] Status"), "row 0: {}", rows[0]);
        assert!(rows[1].contains("tunagit → master"), "row 1: {}", rows[1]);
        for (offset, row) in rows[15..].iter().enumerate() {
            assert!(
                row.contains(&format!("[{}]", offset + 2)),
                "collapsed row: {row}"
            );
        }
    }
}
