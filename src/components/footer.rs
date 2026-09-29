use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use super::theme;
use crate::app::{App, View};

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let version = format!("v{}", env!("CARGO_PKG_VERSION"));
    let version_width = version.len() as u16 + 1;
    if area.width > version_width + 24 {
        let [left, right] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(0), Constraint::Length(version_width)])
            .areas(area);
        render_hints(frame, left, app);
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                format!("{version} "),
                Style::default().fg(theme::MUTED),
            )))
            .alignment(Alignment::Right)
            .style(Style::default().bg(theme::BG)),
            right,
        );
    } else {
        render_hints(frame, area, app);
    }
}

fn render_hints(frame: &mut Frame<'_>, area: Rect, app: &App) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};
    use std::sync::mpsc;

    fn render_footer(width: u16) -> String {
        let backend = TestBackend::new(width, 1);
        let mut terminal = Terminal::new(backend).expect("terminal");
        let (tx, rx) = mpsc::channel();
        let app = App::new(tx, rx);
        terminal
            .draw(|frame| render(frame, frame.area(), &app))
            .expect("draw");
        let buffer = terminal.backend().buffer().clone();
        (0..width)
            .map(|x| buffer[(x, 0)].symbol().to_string())
            .collect()
    }

    #[test]
    fn shows_version_at_bottom_right() {
        let line = render_footer(80);
        let version = format!("v{} ", env!("CARGO_PKG_VERSION"));
        assert!(
            line.ends_with(&version),
            "expected footer to end with {version:?}, got {line:?}"
        );
    }

    #[test]
    fn hides_version_on_narrow_screens() {
        let line = render_footer(20);
        assert!(
            !line.contains(env!("CARGO_PKG_VERSION")),
            "expected no version on narrow footer, got {line:?}"
        );
    }
}
