mod app;
mod commands;
mod components;

use crossterm::{
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::{
    io::{self, Stdout},
    sync::mpsc,
    time::Duration,
};

use app::App;
use commands::{JobKind, JobResult};

type Tui = Terminal<CrosstermBackend<Stdout>>;

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;
    let (tx, rx) = mpsc::channel::<JobResult>();
    let mut app = App::new(tx, rx);
    app.run(JobKind::Overview);

    let result = event_loop(&mut terminal, &mut app);
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
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
            Event::Resize(_, _) => redraw = true,
            _ => {}
        }
    }
}
