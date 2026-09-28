use ratatui::{Frame, layout::Rect};

use super::theme;
use crate::app::{App, PrFocus, View};

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let content = match app.view {
        View::Overview => format!(
            "tunagit  ·  Git + GitHub\n\nRepository  {}\nBranch      {}\n\nWorking tree\n{}\n\nGitHub pull requests\n{}",
            value(&app.git.repo),
            value(&app.git.branch),
            value(&app.git.status),
            value(&app.overview_github),
        ),
        View::Git => format!(
            "Working tree\n\n{}\n\nRemotes\n{}",
            value(&app.git.status),
            value(&app.git.remotes)
        ),
        View::Branches => format!(
            "Local branches\n\n{}\n\nRemotes\n{}",
            value(&app.git.branches),
            value(&app.git.remotes)
        ),
        View::Commits => value(&app.git.commits).to_string(),
        View::Stash => value(&app.git.stash).to_string(),
        View::PullRequests => selected_pr_detail(app),
        View::Checks => selected_checks(app),
        View::Output => app.output.clone(),
    };
    let title = match app.view {
        View::PullRequests | View::Checks => app
            .selected_pr()
            .map(|pr| format!("[0] {} · #{}", app.view.title(), pr.number))
            .unwrap_or_else(|| format!("[0] {}", app.view.title())),
        _ => format!("[0] {}", app.view.title()),
    };
    let active =
        matches!(app.view, View::PullRequests | View::Checks) && app.pr_focus == PrFocus::Detail;
    frame.render_widget(
        theme::text_panel(&title, &content, active, app.scroll),
        area,
    );
}

fn value(text: &str) -> &str {
    if text.is_empty() || text == "(no output)" {
        "(empty)"
    } else {
        text
    }
}

fn selected_pr_detail(app: &App) -> String {
    match app.selected_pr() {
        Some(pr) if app.detail_number == Some(pr.number) => app.detail.clone(),
        Some(pr) => format!(
            "{}  #{}\n\n{} → {}\nState   {}{}\n\n{}\n\nEnter: full details\nc: checks",
            pr.title,
            pr.number,
            pr.head,
            pr.base,
            pr.state,
            if pr.draft { " · draft" } else { "" },
            pr.url,
        ),
        None => app.pr_message.clone(),
    }
}

fn selected_checks(app: &App) -> String {
    match app.selected_pr() {
        Some(pr) if app.checks_number == Some(pr.number) => app.checks.clone(),
        Some(_) => "Press c to load checks for the selected pull request".into(),
        None => "Load pull requests with 6 or p first".into(),
    }
}
