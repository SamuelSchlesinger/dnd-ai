//! D&D Dungeon Master TUI application.
//!
//! A vim-style terminal interface for playing D&D with an AI Dungeon Master.
//!
//! # Headless Mode
//!
//! Run with `--headless` for a text-based interface suitable for automated testing:
//!
//! ```bash
//! cargo run -p dnd -- --headless --name "Thorin" --class fighter --race dwarf
//! ```

mod app;
mod character_creation;
mod effects;
mod events;
mod headless;
mod ui;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use dnd_core::{GameSession, SessionConfig};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{self, stdout};
use std::time::Duration;

use app::App;
use character_creation::CharacterCreation;
use events::{handle_event, EventResult};
use ui::render::render;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env file if present
    dotenvy::dotenv().ok();

    // Check for API key
    if std::env::var("ANTHROPIC_API_KEY").is_err() {
        eprintln!("Error: ANTHROPIC_API_KEY environment variable not set.");
        eprintln!("Please set it in .env file or with: export ANTHROPIC_API_KEY=your_key_here");
        std::process::exit(1);
    }

    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();

    // Check for --headless mode
    if args.iter().any(|a| a == "--headless") {
        let config = headless::parse_config_from_args(&args);
        return headless::run_headless(config).await.map_err(|e| e.into());
    }

    // Check for --help
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_help();
        return Ok(());
    }

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run character creation
    let character = match run_character_creation(&mut terminal) {
        Ok(Some(c)) => c,
        Ok(None) => {
            // User cancelled
            disable_raw_mode()?;
            execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
            return Ok(());
        }
        Err(e) => {
            disable_raw_mode()?;
            execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
            eprintln!("Character creation failed: {e}");
            std::process::exit(1);
        }
    };

    // Create session with the custom character
    let config = SessionConfig::new("The Dragon's Lair")
        .with_starting_location("The Rusty Dragon Inn");

    let session = match GameSession::new_with_character(config, character).await {
        Ok(s) => s,
        Err(e) => {
            disable_raw_mode()?;
            execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
            eprintln!("Failed to create game session: {e}");
            std::process::exit(1);
        }
    };

    // Run app
    let result = run_app(&mut terminal, App::new(session)).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;

    if let Err(e) = result {
        eprintln!("Error: {e}");
    }

    Ok(())
}

/// Run the character creation wizard.
fn run_character_creation<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
) -> io::Result<Option<dnd_core::world::Character>> {
    let mut creation = CharacterCreation::new();

    loop {
        terminal.draw(|f| {
            let area = f.area();
            creation.render(f, area);
        })?;

        if event::poll(Duration::from_millis(100))? {
            let ev = event::read()?;
            creation.handle_event(ev);
        }

        if creation.cancelled {
            return Ok(None);
        }

        if creation.finished {
            match creation.build_character() {
                Ok(character) => return Ok(Some(character)),
                Err(e) => {
                    // Show error and let them try again
                    creation.finished = false;
                    eprintln!("Error building character: {e}");
                }
            }
        }
    }
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
) -> io::Result<()> {
    // Track pending input for async processing
    let mut pending_input: Option<String> = None;

    loop {
        // Render
        terminal.draw(|f| render(f, &app))?;

        // Process any pending save operation
        if let Some(path) = app.pending_save.take() {
            match app.session.save(&path).await {
                Ok(()) => {
                    app.set_status(format!("Saved to {}", path.display()));
                    app.add_narrative(
                        format!("Game saved to {}.", path.display()),
                        dnd_core::world::NarrativeType::System,
                    );
                    // If this was a :wq command, now it's safe to quit
                    if app.quit_after_save {
                        app.should_quit = true;
                    }
                }
                Err(e) => {
                    app.set_status(format!("Save failed: {e}"));
                    // Don't quit on save failure, let user see the error
                    app.quit_after_save = false;
                }
            }
        }

        // Process any pending load operation
        if let Some(path) = app.pending_load.take() {
            match GameSession::load(&path).await {
                Ok(session) => {
                    app.session = session;
                    app.narrative_history.clear();
                    app.add_narrative(
                        format!("Game loaded from {}.", path.display()),
                        dnd_core::world::NarrativeType::System,
                    );
                    app.set_status(format!("Loaded from {}", path.display()));
                }
                Err(e) => {
                    app.set_status(format!("Load failed: {e}"));
                }
            }
        }

        // Process any pending input asynchronously
        if let Some(input) = pending_input.take() {
            // Add player input to narrative IMMEDIATELY (before processing)
            // This ensures the user sees their input before the "Processing..." status
            app.add_narrative(input.clone(), dnd_core::world::NarrativeType::PlayerAction);
            app.set_status("Processing...");
            terminal.draw(|f| render(f, &app))?;

            if let Err(e) = app.process_player_input_without_echo(&input).await {
                app.set_status(format!("Error: {e}"));
            }
            app.enter_normal_mode();
        }

        // Poll for events with timeout for animations
        if event::poll(Duration::from_millis(100))? {
            let ev = event::read()?;

            // Capture the input buffer before handling the event
            let input_before = if app.input_mode == app::InputMode::Insert {
                Some(app.input_buffer().to_string())
            } else {
                None
            };

            let result = handle_event(&mut app, ev);

            match result {
                EventResult::Quit => {
                    return Ok(());
                }
                EventResult::ProcessInput(needs_async) => {
                    if needs_async {
                        // Get the input that was submitted (it's been cleared by submit_input)
                        if let Some(input) = input_before {
                            if !input.is_empty() {
                                pending_input = Some(input);
                            }
                        }
                    }
                }
                EventResult::NeedsRedraw | EventResult::Continue => {
                    // Just continue the loop
                }
            }
        } else {
            // Tick animations
            app.tick();
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn print_help() {
    println!("D&D Dungeon Master - AI-powered D&D 5e game");
    println!();
    println!("USAGE:");
    println!("  dnd [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("  -h, --help       Show this help message");
    println!("  --headless       Run in headless mode (text-only, no TUI)");
    println!();
    println!("HEADLESS OPTIONS (only with --headless):");
    println!("  --name <NAME>       Character name (default: Adventurer)");
    println!("  --race <RACE>       Character race (default: human)");
    println!("  --class <CLASS>     Character class (default: fighter)");
    println!("  --background <BG>   Character background (default: folk-hero)");
    println!();
    println!("RACES:");
    println!("  human, elf, dwarf, halfling, half-orc, half-elf, tiefling, gnome, dragonborn");
    println!();
    println!("CLASSES:");
    println!("  barbarian, bard, cleric, druid, fighter, monk, paladin, ranger,");
    println!("  rogue, sorcerer, warlock, wizard");
    println!();
    println!("BACKGROUNDS:");
    println!("  acolyte, charlatan, criminal, entertainer, folk-hero, guild-artisan,");
    println!("  hermit, noble, outlander, sage, sailor, soldier, urchin");
    println!();
    println!("EXAMPLES:");
    println!("  dnd                                    # Interactive TUI mode");
    println!("  dnd --headless                         # Headless with defaults");
    println!("  dnd --headless --name Thorin --class fighter --race dwarf");
}
