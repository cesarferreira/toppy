mod app;
mod config;
mod input;
mod metrics;
mod process;
mod theme;
mod ui;

use std::io::{self, Stdout, stdout};
use std::panic;
use std::time::{Duration, Instant};

use anyhow::Result;
use clap::Parser;
use crossterm::{
    cursor::Show,
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use app::App;

#[derive(Parser, Debug)]
#[command(name = "toppy", about = "Fast, colorful terminal system monitor")]
struct Cli {
    /// Refresh interval in milliseconds
    #[arg(long, default_value_t = crate::input::DEFAULT_REFRESH_MS)]
    refresh_rate: u64,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let (mut terminal, mut guard) = setup_terminal()?;
    let result = run(&mut terminal, cli.refresh_rate);
    let cleanup = guard.restore();
    result.and(cleanup.map_err(Into::into))
}

struct TerminalGuard {
    active: bool,
}

impl TerminalGuard {
    fn restore(&mut self) -> io::Result<()> {
        if !self.active {
            return Ok(());
        }
        let result = restore_terminal_inner();
        if result.is_ok() {
            self.active = false;
        }
        result
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}

fn setup_terminal() -> Result<(Terminal<CrosstermBackend<Stdout>>, TerminalGuard)> {
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let _ = restore_terminal_inner();
        original_hook(info);
    }));

    // Construct the guard before the first terminal mutation so every failure
    // path, including a partial setup, attempts restoration.
    let guard = TerminalGuard { active: true };
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let terminal = Terminal::new(CrosstermBackend::new(stdout))?;
    Ok((terminal, guard))
}

fn restore_terminal_inner() -> io::Result<()> {
    // Attempt every cleanup even if another one fails (e.g. stdout disappeared).
    let raw = disable_raw_mode();
    let screen = execute!(io::stdout(), LeaveAlternateScreen);
    let mouse = execute!(io::stdout(), DisableMouseCapture);
    let cursor = execute!(io::stdout(), Show);
    raw.and(screen).and(mouse).and(cursor)
}

fn run(terminal: &mut Terminal<CrosstermBackend<Stdout>>, refresh_ms: u64) -> Result<()> {
    let refresh_ms = refresh_ms.clamp(input::MIN_REFRESH_MS, input::MAX_REFRESH_MS);
    let mut app = App::new(refresh_ms);
    terminal.draw(|frame| ui::render(frame, &mut app))?;
    let mut last_refresh = Instant::now();

    loop {
        let deadline = last_refresh + Duration::from_millis(app.refresh_ms);
        if crossterm::event::poll(deadline.saturating_duration_since(Instant::now()))? {
            // Drain any queued events before redrawing — coalesces rapid keystrokes
            // into a single redraw.
            let evt = crossterm::event::read()?;
            input::handle_event(&mut app, evt);
            // Bound each input batch so a continuous stream cannot starve ticks.
            for _ in 0..63 {
                if app.should_quit
                    || Instant::now() >= deadline
                    || !crossterm::event::poll(Duration::ZERO)?
                {
                    break;
                }
                let evt = crossterm::event::read()?;
                input::handle_event(&mut app, evt);
            }
        }

        if app.should_quit {
            break;
        }

        if last_refresh.elapsed() >= Duration::from_millis(app.refresh_ms) {
            app.tick();
            // Skip missed intervals rather than collecting a burst of old ticks.
            last_refresh = Instant::now();
        }

        terminal.draw(|frame| ui::render(frame, &mut app))?;
    }

    Ok(())
}
