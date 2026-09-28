use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use super::theme;
use crate::app::{App, View};

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let line = if let Some(input) = &app.input {
        Line::from(vec![
            Span::styled(
                " : ",
                Style::default()
                    .fg(theme::CYAN)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(input.clone(), Style::default().fg(theme::FG)),
            Span::styled("▏", Style::default().fg(theme::CYAN)),
        ])
    } else {
        let mut spans = vec![Span::raw(" ")];
        let shortcuts = if matches!(app.view, View::PullRequests | View::Checks) {
            [
                ("h/l", " pane  "),
                ("j/k", " move/scroll  "),
                ("PgUp/Dn", " page  "),
                ("Enter", " detail  "),
                ("c", " checks  "),
                ("q", " quit"),
            ]
        } else {
            [
                ("1-5", " Git  "),
                ("6", " PRs  "),
                ("c", " checks  "),
                (":", " command  "),
                ("r", " refresh  "),
                ("q", " quit"),
            ]
        };
        for (key, label) in shortcuts {
            spans.push(Span::styled(
                key,
                Style::default()
                    .fg(theme::CYAN)
                    .add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::styled(label, Style::default().fg(theme::MUTED)));
        }
        Line::from(spans)
    };
    frame.render_widget(
        Paragraph::new(line).style(Style::default().bg(theme::BG)),
        area,
    );
}
