mod app;
mod commands;
mod components;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind, MouseEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend, layout::Rect};
use std::{
    io::{self, Stdout},
    sync::mpsc,
    time::Duration,
};

use app::App;
use commands::{JobKind, JobResult};

type Tui = Terminal<CrosstermBackend<Stdout>>;

fn main() -> io::Result<()> {
    match cli_action(&std::env::args().skip(1).collect::<Vec<_>>()) {
        CliAction::PrintVersion => {
            println!("tunagit {}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }
        CliAction::PrintHelp => {
            println!(
                "tunagit {}\n\nA terminal interface for local Git and GitHub CLI.\n\nUsage: tunagit [--version | --help]\n\nOptions:\n  -V, --version  Print the version\n  -h, --help     Print this help",
                env!("CARGO_PKG_VERSION")
            );
            return Ok(());
        }
        CliAction::Run => {}
    }
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;
    let (tx, rx) = mpsc::channel::<JobResult>();
    let mut app = App::new(tx, rx);
    app.run(JobKind::Overview);

    let result = event_loop(&mut terminal, &mut app);
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        DisableMouseCapture,
        LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;
    result
}

#[derive(Debug, PartialEq, Eq)]
enum CliAction {
    Run,
    PrintVersion,
    PrintHelp,
}

fn cli_action(args: &[String]) -> CliAction {
    if args.iter().any(|arg| arg == "--version" || arg == "-V") {
        CliAction::PrintVersion
    } else if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        CliAction::PrintHelp
    } else {
        CliAction::Run
    }
}

fn event_loop(terminal: &mut Tui, app: &mut App) -> io::Result<()> {
    let mut redraw = true;
    loop {
        redraw |= app.receive();
        if redraw {
            terminal.draw(|frame| components::render(frame, app))?;
            redraw = false;
        }
        if !event::poll(Duration::from_millis(100))? {
            continue;
        }
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                if app.handle_key(key.code) {
                    return Ok(());
                }
                redraw = true;
            }
            Event::Mouse(mouse) => {
                let size = terminal.size()?;
                let area = Rect::new(0, 0, size.width, size.height);
                match mouse.kind {
                    MouseEventKind::ScrollDown | MouseEventKind::ScrollUp => {
                        if let Some(target) =
                            components::scroll_target(area, mouse.column, mouse.row, app.view)
                        {
                            redraw |= app.handle_mouse_scroll(mouse.kind, target);
                        }
                    }
                    MouseEventKind::Down(_) => {
                        if let Some(target) =
                            components::click_target(area, mouse.column, mouse.row, app)
                        {
                            redraw |= app.handle_mouse_click(mouse.kind, target);
                        }
                    }
                    _ => {}
                }
            }
            Event::Resize(_, _) => redraw = true,
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(flags: &[&str]) -> Vec<String> {
        flags.iter().map(|flag| flag.to_string()).collect()
    }

    #[test]
    fn parses_cli_actions() {
        assert_eq!(cli_action(&args(&[])), CliAction::Run);
        assert_eq!(cli_action(&args(&["--version"])), CliAction::PrintVersion);
        assert_eq!(cli_action(&args(&["-V"])), CliAction::PrintVersion);
        assert_eq!(cli_action(&args(&["--help"])), CliAction::PrintHelp);
        assert_eq!(cli_action(&args(&["-h"])), CliAction::PrintHelp);
    }
}
