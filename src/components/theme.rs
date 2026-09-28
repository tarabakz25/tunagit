use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

pub const BG: Color = Color::Rgb(12, 17, 24);
pub const PANEL: Color = Color::Rgb(13, 18, 25);
pub const SELECTED: Color = Color::Rgb(38, 58, 45);
pub const FG: Color = Color::Rgb(208, 216, 226);
pub const MUTED: Color = Color::Rgb(148, 158, 173);
pub const BORDER: Color = Color::Rgb(153, 162, 177);
pub const GREEN: Color = Color::Rgb(150, 205, 123);
pub const ACCENT: Color = GREEN;
pub const CYAN: Color = Color::Rgb(104, 195, 245);
pub const YELLOW: Color = Color::Rgb(234, 195, 111);
pub const RED: Color = Color::Rgb(245, 124, 130);
pub const PURPLE: Color = Color::Rgb(188, 155, 238);

pub fn panel(title: &str, active: bool) -> Block<'static> {
    let color = if active { ACCENT } else { BORDER };
    Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            format!(" {title} "),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ))
        .border_style(Style::default().fg(color))
        .style(Style::default().bg(PANEL).fg(FG))
}

pub fn text_panel<'a>(title: &str, content: &str, active: bool, scroll: u16) -> Paragraph<'a> {
    let lines = content.lines().map(colored_line).collect::<Vec<_>>();
    Paragraph::new(lines)
        .block(panel(title, active))
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0))
}

fn colored_line(line: &str) -> Line<'static> {
    let line = line.replace('\t', "  ");
    let lower = line.to_ascii_lowercase();
    let color = if line.starts_with("$ ") || line.starts_with("##") {
        CYAN
    } else if line.starts_with("??") || lower.contains("pending") || lower.contains("draft") {
        YELLOW
    } else if lower.contains("failed")
        || lower.contains("error")
        || lower.contains("fail")
        || lower.starts_with("exit 1")
    {
        RED
    } else if lower.contains("pass")
        || lower.contains("success")
        || lower.contains("updated")
        || lower.contains("completed")
        || lower.starts_with("exit 0")
    {
        GREEN
    } else if line.starts_with('*') || line.starts_with("Review") || line.starts_with("Merge") {
        PURPLE
    } else {
        FG
    };
    Line::from(Span::styled(line.to_string(), Style::default().fg(color)))
}
