//! Headless mode for the D&D game.
//!
//! This module provides a simple text-based interface for running the game
//! without a TUI. It's designed for automated testing and AI agents.

use dnd_core::{
    headless::{HeadlessConfig, HeadlessGame},
    Background, CharacterClass, RaceType, SessionError,
};
use std::io::{self, BufRead, Write};

/// Run the game in headless mode.
///
/// This provides a simple line-oriented protocol:
/// - Lines starting with `>` are player input
/// - Lines starting with `#` are commands (save, load, quit, status)
/// - All other output is narrative or game state
pub async fn run_headless(config: HeadlessConfig) -> Result<(), SessionError> {
    let mut game = HeadlessGame::new(config).await?;

    // Print initial game info
    println!("=== D&D Headless Mode ===");
    println!("Character: {} ({} {})", game.player_name(),
             game.player_class().unwrap_or("Unknown"),
             game.player_background());
    println!("Location: {}", game.current_location());
    println!("HP: {}/{}", game.current_hp(), game.max_hp());
    println!();
    println!("Commands:");
    println!("  #quit        - Exit the game");
    println!("  #save <path> - Save the game");
    println!("  #load <path> - Load a saved game");
    println!("  #status      - Show current game status");
    println!("  #help        - Show this help");
    println!();
    println!("Enter your actions (one per line):");
    println!();

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Error reading input: {e}");
                break;
            }
        };

        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Handle commands
        if line.starts_with('#') {
            let parts: Vec<&str> = line[1..].split_whitespace().collect();
            match parts.first().copied() {
                Some("quit") | Some("exit") => {
                    println!("Goodbye!");
                    break;
                }
                Some("save") => {
                    if let Some(path) = parts.get(1) {
                        match game.save(path).await {
                            Ok(()) => println!("[SAVED] Game saved to {path}"),
                            Err(e) => println!("[ERROR] Save failed: {e}"),
                        }
                    } else {
                        println!("[ERROR] Usage: #save <path>");
                    }
                }
                Some("load") => {
                    if let Some(path) = parts.get(1) {
                        match HeadlessGame::load(path).await {
                            Ok(loaded) => {
                                game = loaded;
                                println!("[LOADED] Game loaded from {path}");
                                println!("[STATUS] {} at {}, HP: {}/{}",
                                         game.player_name(),
                                         game.current_location(),
                                         game.current_hp(),
                                         game.max_hp());
                            }
                            Err(e) => println!("[ERROR] Load failed: {e}"),
                        }
                    } else {
                        println!("[ERROR] Usage: #load <path>");
                    }
                }
                Some("status") => {
                    println!("[STATUS]");
                    println!("  Character: {} ({} {})",
                             game.player_name(),
                             game.player_class().unwrap_or("Unknown"),
                             game.player_background());
                    println!("  Location: {}", game.current_location());
                    println!("  HP: {}/{}", game.current_hp(), game.max_hp());
                    println!("  In Combat: {}", game.in_combat());

                    // Display active conditions if any
                    let conditions = game.conditions();
                    if !conditions.is_empty() {
                        println!("  Conditions: {}", conditions.join(", "));
                    }

                    // Use appropriate label for turn/round counter
                    if game.in_combat() {
                        println!("  Round: {}", game.turn_count());
                    } else {
                        println!("  Interactions: {}", game.turn_count());
                    }
                }
                Some("help") => {
                    println!("[HELP]");
                    println!("  #quit        - Exit the game");
                    println!("  #save <path> - Save the game");
                    println!("  #load <path> - Load a saved game");
                    println!("  #status      - Show current game status");
                    println!("  #help        - Show this help");
                    println!("  (anything else is sent as player action)");
                }
                _ => {
                    println!("[ERROR] Unknown command. Type #help for help.");
                }
            }
            stdout.flush().ok();
            continue;
        }

        // Send player input to the game
        print!("[PROCESSING]");
        stdout.flush().ok();

        match game.send(line).await {
            Ok(response) => {
                // Clear the processing indicator
                print!("\r            \r");
                stdout.flush().ok();

                // Print the DM's response
                println!("[DM]");
                for para in response.narrative.split("\n\n") {
                    println!("{para}");
                }
                println!();

                // Print status if notable
                if response.in_combat {
                    println!("[COMBAT] HP: {}/{}, Your turn: {}",
                             response.current_hp,
                             response.max_hp,
                             response.is_player_turn);
                }
            }
            Err(e) => {
                print!("\r            \r");
                stdout.flush().ok();
                println!("[ERROR] {e}");
            }
        }
    }

    Ok(())
}

/// Parse character configuration from command line arguments.
pub fn parse_config_from_args(args: &[String]) -> HeadlessConfig {
    let mut config = HeadlessConfig::quick_start("Adventurer");

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--name" => {
                if let Some(name) = args.get(i + 1) {
                    config.name = name.clone();
                    i += 1;
                }
            }
            "--race" => {
                if let Some(race) = args.get(i + 1) {
                    config.race = parse_race(race).unwrap_or(RaceType::Human);
                    i += 1;
                }
            }
            "--class" => {
                if let Some(class) = args.get(i + 1) {
                    config.class = parse_class(class).unwrap_or(CharacterClass::Fighter);
                    i += 1;
                }
            }
            "--background" => {
                if let Some(bg) = args.get(i + 1) {
                    config.background = parse_background(bg).unwrap_or(Background::FolkHero);
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    config
}

fn parse_race(s: &str) -> Option<RaceType> {
    match s.to_lowercase().as_str() {
        "human" => Some(RaceType::Human),
        "elf" => Some(RaceType::Elf),
        "dwarf" => Some(RaceType::Dwarf),
        "halfling" => Some(RaceType::Halfling),
        "halforc" | "half-orc" => Some(RaceType::HalfOrc),
        "halfelf" | "half-elf" => Some(RaceType::HalfElf),
        "tiefling" => Some(RaceType::Tiefling),
        "gnome" => Some(RaceType::Gnome),
        "dragonborn" => Some(RaceType::Dragonborn),
        _ => None,
    }
}

fn parse_class(s: &str) -> Option<CharacterClass> {
    match s.to_lowercase().as_str() {
        "barbarian" => Some(CharacterClass::Barbarian),
        "bard" => Some(CharacterClass::Bard),
        "cleric" => Some(CharacterClass::Cleric),
        "druid" => Some(CharacterClass::Druid),
        "fighter" => Some(CharacterClass::Fighter),
        "monk" => Some(CharacterClass::Monk),
        "paladin" => Some(CharacterClass::Paladin),
        "ranger" => Some(CharacterClass::Ranger),
        "rogue" => Some(CharacterClass::Rogue),
        "sorcerer" => Some(CharacterClass::Sorcerer),
        "warlock" => Some(CharacterClass::Warlock),
        "wizard" => Some(CharacterClass::Wizard),
        _ => None,
    }
}

fn parse_background(s: &str) -> Option<Background> {
    match s.to_lowercase().as_str() {
        "acolyte" => Some(Background::Acolyte),
        "charlatan" => Some(Background::Charlatan),
        "criminal" => Some(Background::Criminal),
        "entertainer" => Some(Background::Entertainer),
        "folkhero" | "folk-hero" | "folk_hero" => Some(Background::FolkHero),
        "guildartisan" | "guild-artisan" | "guild_artisan" => Some(Background::GuildArtisan),
        "hermit" => Some(Background::Hermit),
        "noble" => Some(Background::Noble),
        "outlander" => Some(Background::Outlander),
        "sage" => Some(Background::Sage),
        "sailor" => Some(Background::Sailor),
        "soldier" => Some(Background::Soldier),
        "urchin" => Some(Background::Urchin),
        _ => None,
    }
}
