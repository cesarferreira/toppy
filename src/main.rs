mod app;
mod input;
mod metrics;
mod process;
mod theme;
mod ui;

use std::io::{self, stdout, Stdout};
use std::panic;
use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

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
    let mut terminal = setup_terminal()?;
    let result = run(&mut terminal, cli.refresh_rate);
    restore_terminal()?;
    result
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;

    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let _ = restore_terminal_inner();
        original_hook(info);
    }));

    Ok(terminal)
}

fn restore_terminal() -> Result<()> {
    restore_terminal_inner()?;
    Ok(())
}

fn restore_terminal_inner() -> io::Result<()> {
    disable_raw_mode()?;
    execute!(
        io::stdout(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    Ok(())
}

fn run(terminal: &mut Terminal<CrosstermBackend<Stdout>>, refresh_ms: u64) -> Result<()> {
    let refresh_ms = refresh_ms.clamp(input::MIN_REFRESH_MS, input::MAX_REFRESH_MS);
    let mut app = App::new(refresh_ms);
    terminal.draw(|frame| ui::render(frame, &mut app))?;

    loop {
        if crossterm::event::poll(Duration::from_millis(app.refresh_ms))? {
            // Drain any queued events before redrawing — coalesces rapid keystrokes
            // into a single redraw.
            let evt = crossterm::event::read()?;
            input::handle_event(&mut app, evt);
            while crossterm::event::poll(Duration::ZERO)? {
                let evt = crossterm::event::read()?;
                input::handle_event(&mut app, evt);
            }
        } else {
            app.tick();
        }

        if app.should_quit {
            break;
        }

        terminal.draw(|frame| ui::render(frame, &mut app))?;
    }

    Ok(())
}
