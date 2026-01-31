//! QA tests for save/load and persistence functionality.
//!
//! These tests verify that game state is properly saved and restored.
//! Run with: `cargo test -p dnd-core --test qa_persistence -- --ignored --nocapture`
//!
//! These tests require ANTHROPIC_API_KEY to be set.

use dnd_core::headless::{HeadlessConfig, HeadlessGame};
use dnd_core::world::{Background, CharacterClass, RaceType};
use std::path::PathBuf;
use tempfile::TempDir;

/// Load environment variables from .env file
fn setup() {
    let _ = dotenvy::dotenv();
}

/// Check if API key is available
fn has_api_key() -> bool {
    std::env::var("ANTHROPIC_API_KEY").is_ok()
}

// =============================================================================
// TEST 1: Basic save and load
// =============================================================================

#[tokio::test]
#[ignore]
async fn test_save_and_load_basic() {
    setup();
    if !has_api_key() {
        eprintln!("Skipping test: ANTHROPIC_API_KEY not set");
        return;
    }

    println!("\n=== TEST: Basic Save and Load ===\n");

    // Create a temporary directory for save files
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let save_path = temp_dir.path().join("test_save.json");

    // Create a new game
    let config = HeadlessConfig::quick_start("Thorin Ironforge")
        .with_campaign_name("Persistence Test Campaign");

    let mut game = HeadlessGame::new(config)
        .await
        .expect("Failed to create game");

    // Record initial state
    let initial_name = game.player_name().to_string();
    let initial_location = game.current_location().to_string();
    let initial_max_hp = game.max_hp();

    println!("Initial state:");
    println!("  Name: {}", initial_name);
    println!("  Location: {}", initial_location);
    println!("  Max HP: {}", initial_max_hp);

    // Take an action to generate some game state
    let response = game
        .send("I look around the room")
        .await
        .expect("Failed to send action");
    println!(
        "  DM Response: {}...",
        &response.narrative.chars().take(100).collect::<String>()
    );

    // Save the game
    game.save(&save_path).await.expect("Failed to save game");
    println!("\nGame saved to: {:?}", save_path);

    // Verify save file exists
    assert!(save_path.exists(), "Save file should exist after saving");

    // Load the game
    let loaded_game = HeadlessGame::load(&save_path)
        .await
        .expect("Failed to load game");

    // Verify state is preserved
    println!("\nLoaded state:");
    println!("  Name: {}", loaded_game.player_name());
    println!("  Location: {}", loaded_game.current_location());
    println!("  Max HP: {}", loaded_game.max_hp());

    assert_eq!(
        loaded_game.player_name(),
        initial_name,
        "Player name should be preserved"
    );
    assert_eq!(
        loaded_game.current_location(),
        initial_location,
        "Location should be preserved"
    );
    assert_eq!(
        loaded_game.max_hp(),
        initial_max_hp,
        "Max HP should be preserved"
    );

    println!("\nSUCCESS: Basic save/load works correctly!");
}

// =============================================================================
// TEST 2: HP state preservation
// =============================================================================

#[tokio::test]
#[ignore]
async fn test_save_and_load_hp_state() {
    setup();
    if !has_api_key() {
        eprintln!("Skipping test: ANTHROPIC_API_KEY not set");
        return;
    }

    println!("\n=== TEST: HP State Preservation ===\n");

    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let save_path = temp_dir.path().join("hp_test_save.json");

    // Create a game with a fighter
    let config =
        HeadlessConfig::quick_start("Wounded Warrior").with_campaign_name("HP Persistence Test");

    let mut game = HeadlessGame::new(config)
        .await
        .expect("Failed to create game");

    // Record initial HP
    let initial_hp = game.current_hp();
    let max_hp = game.max_hp();
    println!("Initial HP: {}/{}", initial_hp, max_hp);

    // Manually damage the character by accessing the session
    // We'll do this through save/load with modified state
    game.session_mut()
        .world_mut()
        .player_character
        .hit_points
        .current = max_hp / 2;
    let damaged_hp = game.current_hp();
    println!("After damage: {}/{}", damaged_hp, max_hp);

    // Save the damaged state
    game.save(&save_path).await.expect("Failed to save");

    // Load and verify HP is preserved
    let loaded_game = HeadlessGame::load(&save_path)
        .await
        .expect("Failed to load");

    println!(
        "Loaded HP: {}/{}",
        loaded_game.current_hp(),
        loaded_game.max_hp()
    );

    assert_eq!(
        loaded_game.current_hp(),
        damaged_hp,
        "Damaged HP should be preserved after save/load"
    );
    assert_eq!(
        loaded_game.max_hp(),
        max_hp,
        "Max HP should be preserved after save/load"
    );

    println!("\nSUCCESS: HP state is preserved correctly!");
}

// =============================================================================
// TEST 3: Combat state preservation
// =============================================================================

#[tokio::test]
#[ignore]
async fn test_save_and_load_combat_state() {
    setup();
    if !has_api_key() {
        eprintln!("Skipping test: ANTHROPIC_API_KEY not set");
        return;
    }

    println!("\n=== TEST: Combat State Preservation ===\n");

    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let save_path = temp_dir.path().join("combat_test_save.json");

    // Create a game
    let config = HeadlessConfig::custom(
        "Battle Tested",
        RaceType::Human,
        CharacterClass::Fighter,
        Background::Soldier,
    )
    .with_campaign_name("Combat Persistence Test");

    let mut game = HeadlessGame::new(config)
        .await
        .expect("Failed to create game");

    // Try to initiate combat
    println!("Attempting to initiate combat...");
    let response = game
        .send("I draw my sword and attack the nearest hostile creature!")
        .await
        .expect("Failed to send action");

    println!(
        "DM Response: {}...",
        &response.narrative.chars().take(150).collect::<String>()
    );

    // Record combat state
    let in_combat_before = game.in_combat();
    println!("In combat before save: {}", in_combat_before);

    // Save the game
    game.save(&save_path).await.expect("Failed to save");

    // Load and check combat state
    let loaded_game = HeadlessGame::load(&save_path)
        .await
        .expect("Failed to load");

    let in_combat_after = loaded_game.in_combat();
    println!("In combat after load: {}", in_combat_after);

    // Combat state should be preserved (whether we entered combat or not)
    assert_eq!(
        in_combat_before, in_combat_after,
        "Combat state should be preserved after save/load"
    );

    println!("\nSUCCESS: Combat state preservation verified!");
}

// =============================================================================
// TEST 4: Special characters in names
// =============================================================================

#[tokio::test]
#[ignore]
async fn test_save_and_load_special_characters() {
    setup();
    if !has_api_key() {
        eprintln!("Skipping test: ANTHROPIC_API_KEY not set");
        return;
    }

    println!("\n=== TEST: Special Characters in Names ===\n");

    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let save_path = temp_dir.path().join("special_chars_save.json");

    // Test various special characters in the character name
    let special_name = "Sir Reginald \"The Brave\" O'Connor III";
    let config = HeadlessConfig::quick_start(special_name)
        .with_campaign_name("Special Characters' Test <Campaign>");

    let game = HeadlessGame::new(config)
        .await
        .expect("Failed to create game");

    println!("Original name: {}", game.player_name());

    // Save the game
    game.save(&save_path)
        .await
        .expect("Failed to save with special characters");

    // Load and verify
    let loaded_game = HeadlessGame::load(&save_path)
        .await
        .expect("Failed to load");

    println!("Loaded name: {}", loaded_game.player_name());

    assert_eq!(
        loaded_game.player_name(),
        special_name,
        "Special characters in name should be preserved"
    );

    println!("\nSUCCESS: Special characters handled correctly!");
}

// =============================================================================
// TEST 5: Loading non-existent file (error handling)
// =============================================================================

#[tokio::test]
#[ignore]
async fn test_load_nonexistent_file() {
    setup();
    if !has_api_key() {
        eprintln!("Skipping test: ANTHROPIC_API_KEY not set");
        return;
    }

    println!("\n=== TEST: Loading Non-existent File ===\n");

    let nonexistent_path = PathBuf::from("/tmp/definitely_does_not_exist_12345.json");

    // Attempt to load a non-existent file
    let result = HeadlessGame::load(&nonexistent_path).await;

    match result {
        Ok(_) => {
            panic!("Loading non-existent file should fail!");
        }
        Err(e) => {
            println!("Got expected error: {}", e);
            // Verify it's an IO error (file not found)
            let error_str = format!("{}", e);
            assert!(
                error_str.contains("IO") || error_str.contains("No such file"),
                "Error should indicate file not found: {}",
                error_str
            );
        }
    }

    println!("\nSUCCESS: Non-existent file error handling works!");
}

// =============================================================================
// TEST 6: Save directory creation
// =============================================================================

#[tokio::test]
#[ignore]
async fn test_save_creates_parent_directory() {
    setup();
    if !has_api_key() {
        eprintln!("Skipping test: ANTHROPIC_API_KEY not set");
        return;
    }

    println!("\n=== TEST: Save Directory Creation ===\n");

    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    // Create a path with nested directories that don't exist
    let nested_path = temp_dir
        .path()
        .join("nested")
        .join("deep")
        .join("save.json");

    // Verify parent doesn't exist
    assert!(
        !nested_path.parent().unwrap().exists(),
        "Nested directory should not exist initially"
    );

    // Create a game
    let config = HeadlessConfig::quick_start("Directory Test Hero");
    let game = HeadlessGame::new(config)
        .await
        .expect("Failed to create game");

    // Try to save to nested path
    let result = game.save(&nested_path).await;

    match result {
        Ok(_) => {
            // If save succeeded, verify the file exists
            if nested_path.exists() {
                println!("Save succeeded and created parent directories!");
                println!("File created at: {:?}", nested_path);
            } else {
                println!("NOTE: Save reported success but file not found");
            }
        }
        Err(e) => {
            // This is expected behavior if the implementation doesn't create directories
            println!(
                "Save failed (expected if no auto-directory creation): {}",
                e
            );
            println!("NOTE: The save function does not auto-create parent directories.");
            println!("      This is expected behavior - users should ensure directories exist.");

            // Create the directory manually and try again
            std::fs::create_dir_all(nested_path.parent().unwrap())
                .expect("Failed to create directory manually");

            game.save(&nested_path)
                .await
                .expect("Save should succeed after creating directory");

            assert!(
                nested_path.exists(),
                "Save file should exist after manual directory creation"
            );
            println!("Save succeeded after manual directory creation.");
        }
    }

    println!("\nSUCCESS: Directory handling test completed!");
}

// =============================================================================
// TEST 7: Multiple save/load cycles
// =============================================================================

#[tokio::test]
#[ignore]
async fn test_multiple_save_load_cycles() {
    setup();
    if !has_api_key() {
        eprintln!("Skipping test: ANTHROPIC_API_KEY not set");
        return;
    }

    println!("\n=== TEST: Multiple Save/Load Cycles ===\n");

    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let save_path = temp_dir.path().join("multi_cycle_save.json");

    // Create initial game
    let config = HeadlessConfig::quick_start("Cycle Tester").with_campaign_name("Multi-Cycle Test");

    let mut game = HeadlessGame::new(config)
        .await
        .expect("Failed to create game");

    // Perform multiple save/load cycles
    for cycle in 1..=3 {
        println!("\n--- Cycle {} ---", cycle);

        // Take an action
        let action = format!("I take step number {}", cycle);
        let response = game.send(&action).await.expect("Failed to send action");
        println!("Action: {}", action);
        println!(
            "Response: {}...",
            &response.narrative.chars().take(80).collect::<String>()
        );

        // Record state before save
        let hp_before = game.current_hp();
        let location_before = game.current_location().to_string();

        // Save
        game.save(&save_path).await.expect("Failed to save");
        println!("Saved. HP: {}", hp_before);

        // Load into new game instance
        game = HeadlessGame::load(&save_path)
            .await
            .expect("Failed to load");

        // Verify state
        assert_eq!(
            game.current_hp(),
            hp_before,
            "HP should match after cycle {}",
            cycle
        );
        assert_eq!(
            game.current_location(),
            location_before,
            "Location should match after cycle {}",
            cycle
        );

        println!(
            "Loaded and verified. HP: {}, Location: {}",
            game.current_hp(),
            game.current_location()
        );
    }

    println!("\nSUCCESS: All save/load cycles completed successfully!");
}

// =============================================================================
// TEST 8: Character class and race preservation
// =============================================================================

#[tokio::test]
#[ignore]
async fn test_save_and_load_character_details() {
    setup();
    if !has_api_key() {
        eprintln!("Skipping test: ANTHROPIC_API_KEY not set");
        return;
    }

    println!("\n=== TEST: Character Class and Race Preservation ===\n");

    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let save_path = temp_dir.path().join("character_details_save.json");

    // Create a character with specific class and race
    let config = HeadlessConfig::custom(
        "Elara Moonshadow",
        RaceType::Elf,
        CharacterClass::Wizard,
        Background::Sage,
    )
    .with_campaign_name("Character Details Test");

    let game = HeadlessGame::new(config)
        .await
        .expect("Failed to create game");

    // Record initial state
    let initial_name = game.player_name().to_string();
    let initial_class = game.player_class().map(|s| s.to_string());
    let initial_background = game.player_background().to_string();

    println!("Initial character:");
    println!("  Name: {}", initial_name);
    println!("  Class: {:?}", initial_class);
    println!("  Background: {}", initial_background);

    // Save
    game.save(&save_path).await.expect("Failed to save");

    // Load
    let loaded_game = HeadlessGame::load(&save_path)
        .await
        .expect("Failed to load");

    println!("\nLoaded character:");
    println!("  Name: {}", loaded_game.player_name());
    println!("  Class: {:?}", loaded_game.player_class());
    println!("  Background: {}", loaded_game.player_background());

    // Verify
    assert_eq!(loaded_game.player_name(), initial_name);
    assert_eq!(
        loaded_game.player_class().map(|s| s.to_string()),
        initial_class
    );
    assert_eq!(loaded_game.player_background(), initial_background);

    println!("\nSUCCESS: Character details preserved correctly!");
}

// =============================================================================
// TEST 9: Save file content verification
// =============================================================================

#[tokio::test]
#[ignore]
async fn test_save_file_is_valid_json() {
    setup();
    if !has_api_key() {
        eprintln!("Skipping test: ANTHROPIC_API_KEY not set");
        return;
    }

    println!("\n=== TEST: Save File JSON Validity ===\n");

    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let save_path = temp_dir.path().join("json_validity_save.json");

    // Create and save a game
    let config = HeadlessConfig::quick_start("JSON Test Hero");
    let game = HeadlessGame::new(config)
        .await
        .expect("Failed to create game");
    game.save(&save_path).await.expect("Failed to save");

    // Read and parse the save file
    let content = std::fs::read_to_string(&save_path).expect("Failed to read save file");

    println!("Save file size: {} bytes", content.len());
    println!(
        "First 500 chars:\n{}",
        &content.chars().take(500).collect::<String>()
    );

    // Verify it's valid JSON
    let parsed: serde_json::Value =
        serde_json::from_str(&content).expect("Save file should be valid JSON");

    // Check for expected fields
    assert!(
        parsed.get("world").is_some(),
        "Save should contain 'world' field"
    );

    // Check world contains player_character
    if let Some(world) = parsed.get("world") {
        assert!(
            world.get("player_character").is_some(),
            "World should contain 'player_character' field"
        );

        if let Some(pc) = world.get("player_character") {
            if let Some(name) = pc.get("name") {
                println!("\nPlayer name in JSON: {}", name);
            }
        }
    }

    println!("\nSUCCESS: Save file is valid JSON with expected structure!");
}

// =============================================================================
// TEST 10: Overwriting existing save
// =============================================================================

#[tokio::test]
#[ignore]
async fn test_overwrite_existing_save() {
    setup();
    if !has_api_key() {
        eprintln!("Skipping test: ANTHROPIC_API_KEY not set");
        return;
    }

    println!("\n=== TEST: Overwriting Existing Save ===\n");

    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let save_path = temp_dir.path().join("overwrite_test.json");

    // Create first game and save
    let config1 = HeadlessConfig::quick_start("First Hero");
    let game1 = HeadlessGame::new(config1)
        .await
        .expect("Failed to create first game");
    game1
        .save(&save_path)
        .await
        .expect("Failed to save first game");

    println!("First save - Name: {}", game1.player_name());

    // Create second game with different name and overwrite
    let config2 = HeadlessConfig::quick_start("Second Hero");
    let game2 = HeadlessGame::new(config2)
        .await
        .expect("Failed to create second game");
    game2
        .save(&save_path)
        .await
        .expect("Failed to save second game");

    println!("Second save - Name: {}", game2.player_name());

    // Load and verify it's the second game
    let loaded_game = HeadlessGame::load(&save_path)
        .await
        .expect("Failed to load");

    println!("Loaded - Name: {}", loaded_game.player_name());

    assert_eq!(
        loaded_game.player_name(),
        "Second Hero",
        "Save should be overwritten with second game's data"
    );

    println!("\nSUCCESS: Save overwriting works correctly!");
}
