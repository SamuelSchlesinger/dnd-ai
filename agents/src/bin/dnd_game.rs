//! D&D Dungeon Master Game - Entry Point
//!
//! A single-player D&D 5e experience with an AI Dungeon Master.

use std::io::{self, stdout};
use std::time::Duration;

use crossterm::{
    event::{self},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;

use agents::dnd::app::AppState;
use agents::dnd::events::{handle_event, EventResult};
use agents::dnd::ui::render::render;

fn main() -> io::Result<()> {
    // Load environment variables from .env file
    if dotenvy::from_path("../.env").is_err() {
        let _ = dotenvy::dotenv();
    }

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create application state
    let mut app = AppState::new();

    // Main loop
    let result = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(e) = result {
        eprintln!("Error: {}", e);
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut AppState) -> io::Result<()> {
    let tick_rate = Duration::from_millis(100);

    loop {
        // Draw
        terminal.draw(|frame| render(frame, app))?;

        // Handle events with timeout for animations
        if event::poll(tick_rate)? {
            let event = event::read()?;

            match handle_event(app, event) {
                EventResult::Quit => break,
                EventResult::Continue | EventResult::NeedsRedraw => {}
            }
        }

        // Tick for animations
        app.tick();

        // Check for quit flag
        if app.should_quit {
            break;
        }
    }

    Ok(())
}
