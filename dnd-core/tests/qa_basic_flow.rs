//! QA tests for basic game flow using the headless API.
//!
//! These tests verify the basic game flow works correctly:
//! - Character creation with different races/classes
//! - Basic exploration commands
//! - NPC dialogue
//! - State queries
//!
//! Run with: `ANTHROPIC_API_KEY=$ANTHROPIC_API_KEY cargo test -p dnd-core qa_basic_flow -- --ignored --nocapture`

use dnd_core::headless::{HeadlessConfig, HeadlessGame};
use dnd_core::world::{Background, CharacterClass, RaceType};

/// Load environment variables from .env file
fn setup() {
    let _ = dotenvy::dotenv();
}

/// Check if API key is available
fn has_api_key() -> bool {
    std::env::var("ANTHROPIC_API_KEY").is_ok()
}

// =============================================================================
// CHARACTER CREATION TESTS
// =============================================================================

#[tokio::test]
#[ignore]
async fn test_quick_start_character_creation() {
    setup();
    if !has_api_key() {
        eprintln!("Skipping test: ANTHROPIC_API_KEY not set");
        return;
    }

    println!("\n=== Testing Quick Start Character Creation ===\n");

    let config = HeadlessConfig::quick_start("Thorin");
    let game = HeadlessGame::new(config).await;

    match game {
        Ok(game) => {
            println!("SUCCESS: Game created successfully");
            println!("  Player name: {}", game.player_name());
            println!("  Player class: {:?}", game.player_class());
            println!("  Player background: {}", game.player_background());
            println!("  Current location: {}", game.current_location());
            println!("  HP: {}/{}", game.current_hp(), game.max_hp());

            assert_eq!(game.player_name(), "Thorin");
            assert_eq!(game.player_class(), Some("Fighter"));
            assert_eq!(game.player_background(), "Folk Hero");
            assert!(game.current_hp() > 0);
            assert!(game.max_hp() > 0);
            assert!(!game.in_combat());
        }
        Err(e) => {
            panic!("FAILED: Could not create game: {:?}", e);
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_custom_character_elf_wizard() {
    setup();
    if !has_api_key() {
        eprintln!("Skipping test: ANTHROPIC_API_KEY not set");
        return;
    }

    println!("\n=== Testing Custom Character: Elf Wizard ===\n");

    let config = HeadlessConfig::custom(
        "Elara",
        RaceType::Elf,
        CharacterClass::Wizard,
        Background::Sage,
    )
    .with_campaign_name("The Lost Library")
    .with_starting_location("The Academy");

    let game = HeadlessGame::new(config).await;

    match game {
        Ok(game) => {
            println!("SUCCESS: Elf Wizard created");
            println!("  Player name: {}", game.player_name());
            println!("  Player class: {:?}", game.player_class());
            println!("  Player background: {}", game.player_background());
            println!("  Current location: {}", game.current_location());
            println!("  HP: {}/{}", game.current_hp(), game.max_hp());

            assert_eq!(game.player_name(), "Elara");
            assert_eq!(game.player_class(), Some("Wizard"));
            assert_eq!(game.player_background(), "Sage");
            assert_eq!(game.current_location(), "The Academy");
        }
        Err(e) => {
            panic!("FAILED: Could not create Elf Wizard: {:?}", e);
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_custom_character_dwarf_cleric() {
    setup();
    if !has_api_key() {
        eprintln!("Skipping test: ANTHROPIC_API_KEY not set");
        return;
    }

    println!("\n=== Testing Custom Character: Dwarf Cleric ===\n");

    let config = HeadlessConfig::custom(
        "Gimli",
        RaceType::Dwarf,
        CharacterClass::Cleric,
        Background::Acolyte,
    );

    let game = HeadlessGame::new(config).await;

    match game {
        Ok(game) => {
            println!("SUCCESS: Dwarf Cleric created");
            println!("  Player name: {}", game.player_name());
            println!("  Player class: {:?}", game.player_class());
            println!("  Player background: {}", game.player_background());
            println!("  HP: {}/{}", game.current_hp(), game.max_hp());

            assert_eq!(game.player_name(), "Gimli");
            assert_eq!(game.player_class(), Some("Cleric"));
            assert_eq!(game.player_background(), "Acolyte");
        }
        Err(e) => {
            panic!("FAILED: Could not create Dwarf Cleric: {:?}", e);
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_custom_character_halfling_rogue() {
    setup();
    if !has_api_key() {
        eprintln!("Skipping test: ANTHROPIC_API_KEY not set");
        return;
    }

    println!("\n=== Testing Custom Character: Halfling Rogue ===\n");

    let config = HeadlessConfig::custom(
        "Pippin",
        RaceType::Halfling,
        CharacterClass::Rogue,
        Background::Criminal,
    );

    let game = HeadlessGame::new(config).await;

    match game {
        Ok(game) => {
            println!("SUCCESS: Halfling Rogue created");
            println!("  Player name: {}", game.player_name());
            println!("  Player class: {:?}", game.player_class());
            println!("  Player background: {}", game.player_background());

            assert_eq!(game.player_name(), "Pippin");
            assert_eq!(game.player_class(), Some("Rogue"));
            assert_eq!(game.player_background(), "Criminal");
        }
        Err(e) => {
            panic!("FAILED: Could not create Halfling Rogue: {:?}", e);
        }
    }
}

// =============================================================================
// EXPLORATION COMMAND TESTS
// =============================================================================

#[tokio::test]
#[ignore]
async fn test_look_around_command() {
    setup();
    if !has_api_key() {
        eprintln!("Skipping test: ANTHROPIC_API_KEY not set");
        return;
    }

    println!("\n=== Testing 'look around' Command ===\n");

    let config = HeadlessConfig::quick_start("Explorer");
    let mut game = HeadlessGame::new(config)
        .await
        .expect("Failed to create game");

    let response = game.send("I look around").await;

    match response {
        Ok(response) => {
            println!("SUCCESS: Got response for 'look around'");
            println!("  Narrative length: {} chars", response.narrative.len());
            println!("  In combat: {}", response.in_combat);
            println!(
                "  Narrative preview: {}...",
                &response.narrative[..response.narrative.len().min(200)]
            );

            assert!(
                !response.narrative.is_empty(),
                "Narrative should not be empty"
            );
            assert!(
                !response.in_combat,
                "Should not be in combat from looking around"
            );
        }
        Err(e) => {
            panic!("FAILED: Error on 'look around': {:?}", e);
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_examine_command() {
    setup();
    if !has_api_key() {
        eprintln!("Skipping test: ANTHROPIC_API_KEY not set");
        return;
    }

    println!("\n=== Testing 'examine' Command ===\n");

    let config = HeadlessConfig::quick_start("Inspector");
    let mut game = HeadlessGame::new(config)
        .await
        .expect("Failed to create game");

    // First look around to establish context
    let _ = game.send("I look around the room").await;

    // Then examine something specific
    let response = game.send("I examine the nearest object closely").await;

    match response {
        Ok(response) => {
            println!("SUCCESS: Got response for 'examine'");
            println!("  Narrative length: {} chars", response.narrative.len());
            println!(
                "  Narrative preview: {}...",
                &response.narrative[..response.narrative.len().min(200)]
            );

            assert!(
                !response.narrative.is_empty(),
                "Narrative should not be empty"
            );
        }
        Err(e) => {
            panic!("FAILED: Error on 'examine': {:?}", e);
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_move_command() {
    setup();
    if !has_api_key() {
        eprintln!("Skipping test: ANTHROPIC_API_KEY not set");
        return;
    }

    println!("\n=== Testing Movement Command ===\n");

    let config =
        HeadlessConfig::quick_start("Wanderer").with_starting_location("The Crossroads Inn");
    let mut game = HeadlessGame::new(config)
        .await
        .expect("Failed to create game");

    let initial_location = game.current_location().to_string();
    println!("  Initial location: {}", initial_location);

    // Try to move somewhere
    let response = game.send("I walk outside the inn").await;

    match response {
        Ok(response) => {
            println!("SUCCESS: Got response for movement");
            println!("  Narrative length: {} chars", response.narrative.len());
            println!("  Current location after move: {}", game.current_location());
            println!(
                "  Narrative preview: {}...",
                &response.narrative[..response.narrative.len().min(200)]
            );

            assert!(
                !response.narrative.is_empty(),
                "Narrative should not be empty"
            );
            // Note: Location may or may not change depending on DM interpretation
        }
        Err(e) => {
            panic!("FAILED: Error on movement: {:?}", e);
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_search_command() {
    setup();
    if !has_api_key() {
        eprintln!("Skipping test: ANTHROPIC_API_KEY not set");
        return;
    }

    println!("\n=== Testing 'search' Command ===\n");

    let config = HeadlessConfig::quick_start("Searcher");
    let mut game = HeadlessGame::new(config)
        .await
        .expect("Failed to create game");

    let response = game
        .send("I carefully search the area for hidden objects or secret doors")
        .await;

    match response {
        Ok(response) => {
            println!("SUCCESS: Got response for 'search'");
            println!("  Narrative length: {} chars", response.narrative.len());
            println!(
                "  Narrative preview: {}...",
                &response.narrative[..response.narrative.len().min(200)]
            );

            assert!(
                !response.narrative.is_empty(),
                "Narrative should not be empty"
            );
            // Search often triggers a skill check - the DM should handle this
        }
        Err(e) => {
            panic!("FAILED: Error on 'search': {:?}", e);
        }
    }
}

// =============================================================================
// NPC DIALOGUE TESTS
// =============================================================================

#[tokio::test]
#[ignore]
async fn test_npc_dialogue_greeting() {
    setup();
    if !has_api_key() {
        eprintln!("Skipping test: ANTHROPIC_API_KEY not set");
        return;
    }

    println!("\n=== Testing NPC Dialogue (Greeting) ===\n");

    let config =
        HeadlessConfig::quick_start("Diplomat").with_starting_location("The Crossroads Inn");
    let mut game = HeadlessGame::new(config)
        .await
        .expect("Failed to create game");

    // First establish there's someone to talk to
    let _ = game
        .send("I look around the inn for someone to talk to")
        .await;

    // Then try to initiate dialogue
    let response = game
        .send("I approach the innkeeper and say 'Good day! What news do you have?'")
        .await;

    match response {
        Ok(response) => {
            println!("SUCCESS: Got NPC dialogue response");
            println!("  Narrative length: {} chars", response.narrative.len());
            println!(
                "  Narrative preview: {}...",
                &response.narrative[..response.narrative.len().min(300)]
            );

            assert!(
                !response.narrative.is_empty(),
                "Narrative should not be empty"
            );
        }
        Err(e) => {
            panic!("FAILED: Error on NPC dialogue: {:?}", e);
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_npc_dialogue_question() {
    setup();
    if !has_api_key() {
        eprintln!("Skipping test: ANTHROPIC_API_KEY not set");
        return;
    }

    println!("\n=== Testing NPC Dialogue (Question) ===\n");

    let config =
        HeadlessConfig::quick_start("Questioner").with_starting_location("The Market Square");
    let mut game = HeadlessGame::new(config)
        .await
        .expect("Failed to create game");

    // Try to get information from an NPC
    let response = game
        .send("I find a merchant and ask them about any rumors of adventure or danger in the area")
        .await;

    match response {
        Ok(response) => {
            println!("SUCCESS: Got response to NPC question");
            println!("  Narrative length: {} chars", response.narrative.len());
            println!(
                "  Narrative preview: {}...",
                &response.narrative[..response.narrative.len().min(300)]
            );

            assert!(
                !response.narrative.is_empty(),
                "Narrative should not be empty"
            );
        }
        Err(e) => {
            panic!("FAILED: Error on NPC question: {:?}", e);
        }
    }
}

// =============================================================================
// STATE QUERY TESTS
// =============================================================================

#[tokio::test]
#[ignore]
async fn test_state_queries_after_gameplay() {
    setup();
    if !has_api_key() {
        eprintln!("Skipping test: ANTHROPIC_API_KEY not set");
        return;
    }

    println!("\n=== Testing State Queries After Gameplay ===\n");

    let config = HeadlessConfig::custom(
        "StateTest",
        RaceType::Human,
        CharacterClass::Fighter,
        Background::Soldier,
    );
    let mut game = HeadlessGame::new(config)
        .await
        .expect("Failed to create game");

    // Record initial state
    let initial_hp = game.current_hp();
    let initial_max_hp = game.max_hp();
    let initial_location = game.current_location().to_string();

    println!("Initial state:");
    println!("  Name: {}", game.player_name());
    println!("  Class: {:?}", game.player_class());
    println!("  Background: {}", game.player_background());
    println!("  HP: {}/{}", initial_hp, initial_max_hp);
    println!("  Location: {}", initial_location);
    println!("  In combat: {}", game.in_combat());
    println!("  Conditions: {:?}", game.conditions());
    println!("  Turn count: {}", game.turn_count());

    // Perform some actions
    let _ = game.send("I look around").await;
    let _ = game.send("I check my equipment").await;

    // Check state after actions
    let final_hp = game.current_hp();
    let transcript = game.transcript();

    println!("\nAfter actions:");
    println!("  HP: {}/{}", final_hp, game.max_hp());
    println!("  Transcript entries: {}", transcript.len());
    println!("  Turn count: {}", game.turn_count());

    // Verify transcript recorded interactions
    assert_eq!(transcript.len(), 2, "Should have 2 transcript entries");
    assert_eq!(transcript[0].turn, 1);
    assert_eq!(transcript[1].turn, 2);

    // Check last response
    if let Some(last) = game.last_response() {
        println!(
            "  Last response preview: {}...",
            &last[..last.len().min(100)]
        );
    }
}

#[tokio::test]
#[ignore]
async fn test_hp_tracking() {
    setup();
    if !has_api_key() {
        eprintln!("Skipping test: ANTHROPIC_API_KEY not set");
        return;
    }

    println!("\n=== Testing HP Tracking ===\n");

    let config = HeadlessConfig::quick_start("HPTest");
    let mut game = HeadlessGame::new(config)
        .await
        .expect("Failed to create game");

    let initial_hp = game.current_hp();
    let max_hp = game.max_hp();

    println!("HP Status: {}/{}", initial_hp, max_hp);

    assert!(initial_hp > 0, "HP should be positive");
    assert!(max_hp > 0, "Max HP should be positive");
    assert!(initial_hp <= max_hp, "Current HP should not exceed max HP");

    // HP should be reported in game response too
    let response = game
        .send("I check my health")
        .await
        .expect("Should get response");
    println!("Response HP: {}/{}", response.current_hp, response.max_hp);

    assert_eq!(
        response.max_hp, max_hp,
        "Response max HP should match game state"
    );
}

// =============================================================================
// MULTI-TURN FLOW TEST
// =============================================================================

#[tokio::test]
#[ignore]
async fn test_multi_turn_game_flow() {
    setup();
    if !has_api_key() {
        eprintln!("Skipping test: ANTHROPIC_API_KEY not set");
        return;
    }

    println!("\n=== Testing Multi-Turn Game Flow ===\n");

    let config = HeadlessConfig::custom(
        "Adventurer",
        RaceType::HalfElf,
        CharacterClass::Bard,
        Background::Entertainer,
    )
    .with_starting_location("The Rusty Dragon Inn");

    let mut game = HeadlessGame::new(config)
        .await
        .expect("Failed to create game");

    println!(
        "Character: {} the {} {}",
        game.player_name(),
        game.player_background(),
        game.player_class().unwrap_or("Unknown")
    );
    println!("Starting at: {}", game.current_location());
    println!();

    // Turn 1: Look around
    println!("--- Turn 1: Looking around ---");
    let r1 = game
        .send("I look around the inn, taking in the atmosphere")
        .await;
    match r1 {
        Ok(response) => {
            println!(
                "Response: {}...",
                &response.narrative[..response.narrative.len().min(150)]
            );
            println!();
        }
        Err(e) => {
            panic!("Turn 1 failed: {:?}", e);
        }
    }

    // Turn 2: Social interaction
    println!("--- Turn 2: Social interaction ---");
    let r2 = game
        .send("I introduce myself to anyone nearby with a theatrical bow")
        .await;
    match r2 {
        Ok(response) => {
            println!(
                "Response: {}...",
                &response.narrative[..response.narrative.len().min(150)]
            );
            println!();
        }
        Err(e) => {
            panic!("Turn 2 failed: {:?}", e);
        }
    }

    // Turn 3: Gather information
    println!("--- Turn 3: Gathering information ---");
    let r3 = game
        .send("I ask if anyone knows of opportunities for a traveling performer")
        .await;
    match r3 {
        Ok(response) => {
            println!(
                "Response: {}...",
                &response.narrative[..response.narrative.len().min(150)]
            );
            println!();
        }
        Err(e) => {
            panic!("Turn 3 failed: {:?}", e);
        }
    }

    // Verify game state after multiple turns
    let transcript = game.transcript();
    println!("=== Game Summary ===");
    println!("Turns completed: {}", transcript.len());
    println!("In combat: {}", game.in_combat());
    println!("HP: {}/{}", game.current_hp(), game.max_hp());

    assert_eq!(transcript.len(), 3, "Should have 3 turns in transcript");
    assert!(
        !game.in_combat(),
        "Should not be in combat after social interactions"
    );

    println!("\nSUCCESS: Multi-turn flow completed without errors");
}

// =============================================================================
// ALL CHARACTER CLASSES TEST
// =============================================================================

#[tokio::test]
#[ignore]
async fn test_all_character_classes() {
    setup();
    if !has_api_key() {
        eprintln!("Skipping test: ANTHROPIC_API_KEY not set");
        return;
    }

    println!("\n=== Testing All Character Classes ===\n");

    let classes = [
        (CharacterClass::Barbarian, "Conan"),
        (CharacterClass::Bard, "Dandelion"),
        (CharacterClass::Cleric, "Brother Marcus"),
        (CharacterClass::Druid, "Oakwise"),
        (CharacterClass::Fighter, "Roland"),
        (CharacterClass::Monk, "Kenshin"),
        (CharacterClass::Paladin, "Sir Galahad"),
        (CharacterClass::Ranger, "Strider"),
        (CharacterClass::Rogue, "Shadow"),
        (CharacterClass::Sorcerer, "Merlin"),
        (CharacterClass::Warlock, "Faust"),
        (CharacterClass::Wizard, "Gandalf"),
    ];

    let mut successes = 0;
    let mut failures = Vec::new();

    for (class, name) in classes {
        let config = HeadlessConfig::custom(name, RaceType::Human, class, Background::FolkHero);

        match HeadlessGame::new(config).await {
            Ok(game) => {
                let actual_class = game.player_class().unwrap_or("Unknown");
                if actual_class == class.name() {
                    println!("  [OK] {} - {}", class.name(), name);
                    successes += 1;
                } else {
                    println!(
                        "  [MISMATCH] {} - Expected {}, got {}",
                        name,
                        class.name(),
                        actual_class
                    );
                    failures.push(format!("{}: class mismatch", class.name()));
                }
            }
            Err(e) => {
                println!("  [FAIL] {} - {}: {:?}", class.name(), name, e);
                failures.push(format!("{}: {:?}", class.name(), e));
            }
        }
    }

    println!("\nResults: {}/12 classes created successfully", successes);

    if !failures.is_empty() {
        println!("Failures:");
        for f in &failures {
            println!("  - {}", f);
        }
    }

    assert_eq!(successes, 12, "All 12 classes should be creatable");
}

// =============================================================================
// ALL RACES TEST
// =============================================================================

#[tokio::test]
#[ignore]
async fn test_all_races() {
    setup();
    if !has_api_key() {
        eprintln!("Skipping test: ANTHROPIC_API_KEY not set");
        return;
    }

    println!("\n=== Testing All Races ===\n");

    let races = [
        (RaceType::Human, "John"),
        (RaceType::Elf, "Legolas"),
        (RaceType::Dwarf, "Gimli"),
        (RaceType::Halfling, "Bilbo"),
        (RaceType::HalfOrc, "Thrall"),
        (RaceType::HalfElf, "Tanis"),
        (RaceType::Tiefling, "Zariel"),
        (RaceType::Gnome, "Wilbur"),
        (RaceType::Dragonborn, "Drakon"),
    ];

    let mut successes = 0;
    let mut failures = Vec::new();

    for (race, name) in races {
        let config =
            HeadlessConfig::custom(name, race, CharacterClass::Fighter, Background::Soldier);

        match HeadlessGame::new(config).await {
            Ok(game) => {
                println!("  [OK] {} - {}", race.name(), name);
                println!("       HP: {}/{}", game.current_hp(), game.max_hp());
                successes += 1;
            }
            Err(e) => {
                println!("  [FAIL] {} - {}: {:?}", race.name(), name, e);
                failures.push(format!("{}: {:?}", race.name(), e));
            }
        }
    }

    println!("\nResults: {}/9 races created successfully", successes);

    if !failures.is_empty() {
        println!("Failures:");
        for f in &failures {
            println!("  - {}", f);
        }
    }

    assert_eq!(successes, 9, "All 9 races should be creatable");
}
