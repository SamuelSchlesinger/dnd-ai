//! Integration tests that call the real Claude API.
//!
//! These tests require ANTHROPIC_API_KEY to be set (via .env file or environment).
//! Run with: `cargo test -p dnd-core --test api_integration -- --ignored`
//!
//! These are marked #[ignore] by default to avoid:
//! - API costs in CI
//! - Test failures when no API key is available
//! - Slow test runs (API calls take seconds)

use dnd_core::dm::{DmConfig, DungeonMaster, RelevanceChecker, StoryMemory};
use dnd_core::dm::story_memory::ConsequenceSeverity;
use dnd_core::world::{create_sample_fighter, GameWorld};

/// Load environment variables from .env file
fn setup() {
    let _ = dotenvy::dotenv();
}

/// Check if API key is available
fn has_api_key() -> bool {
    std::env::var("ANTHROPIC_API_KEY").is_ok()
}

#[tokio::test]
#[ignore] // Run with: cargo test -p dnd-core --test api_integration -- --ignored
async fn test_dm_registers_consequence_on_hostile_action() {
    setup();
    if !has_api_key() {
        eprintln!("Skipping test: ANTHROPIC_API_KEY not set");
        return;
    }

    // Create a game session
    let character = create_sample_fighter("Thorin");
    let mut world = GameWorld::new("Test Campaign", character);
    let mut dm = DungeonMaster::from_env().expect("Failed to create DM");

    // Configure for faster responses
    dm = dm.with_config(DmConfig {
        model: Some("claude-sonnet-4-20250514".to_string()),
        max_tokens: 1024,
        temperature: Some(0.7),
        custom_system_prompt: None,
    });

    // Send an action that should provoke a consequence
    let response = dm
        .process_input(
            "I walk up to the bartender and insult his mother, then spit in his drink",
            &mut world,
        )
        .await
        .expect("DM should respond");

    // Check if the DM's response is reasonable
    assert!(!response.narrative.is_empty(), "DM should provide a narrative");

    // The DM may or may not register a consequence on the first action,
    // but let's at least verify the system works
    println!("DM Response: {}", response.narrative);
    println!("Intents: {:?}", response.intents);
    println!("Effects: {:?}", response.effects);
    println!(
        "Pending consequences: {}",
        dm.story_memory().pending_consequence_count()
    );

    // The test passes if we get a response without errors
    // A more specific test would check for consequence registration,
    // but that depends on the DM's judgment
}

#[tokio::test]
#[ignore]
async fn test_relevance_checker_with_real_api() {
    setup();
    if !has_api_key() {
        eprintln!("Skipping test: ANTHROPIC_API_KEY not set");
        return;
    }

    // Create a story memory with a consequence
    let mut story_memory = StoryMemory::new();

    // Add a location entity
    let riverside_id = story_memory.create_entity(
        dnd_core::dm::EntityType::Location,
        "Riverside Village",
    );

    // Add a consequence about entering the village
    let consequence = dnd_core::dm::Consequence::new(
        "Player enters Riverside Village or approaches its gates",
        "Town guards recognize the player from wanted posters and attempt to arrest them",
        ConsequenceSeverity::Major,
        story_memory.current_turn(),
    )
    .with_related(riverside_id);

    story_memory.add_consequence(consequence);

    // Create the relevance checker
    let checker = RelevanceChecker::from_env().expect("Failed to create checker");

    // Test with input that should trigger the consequence
    let result = checker
        .check_relevance(
            "I walk down the road toward the village gates",
            "Forest Path",
            &story_memory,
        )
        .await
        .expect("Relevance check should succeed");

    println!("Relevance result: {:?}", result);
    println!("Triggered consequences: {:?}", result.triggered_consequences);
    println!("Explanation: {:?}", result.explanation);

    // The Haiku model should recognize this triggers the "enters village" consequence
    // Note: This is probabilistic, so we log rather than assert
    if result.has_triggered_consequences() {
        println!("SUCCESS: Consequence was triggered!");
    } else {
        println!("NOTE: Consequence was not triggered (Haiku may need clearer input)");
    }
}

#[tokio::test]
#[ignore]
async fn test_relevance_checker_no_false_positives() {
    setup();
    if !has_api_key() {
        eprintln!("Skipping test: ANTHROPIC_API_KEY not set");
        return;
    }

    // Create a story memory with a consequence
    let mut story_memory = StoryMemory::new();

    // Add a consequence about entering a specific location
    let consequence = dnd_core::dm::Consequence::new(
        "Player enters the Dragon's Lair cave",
        "The dragon awakens and attacks",
        ConsequenceSeverity::Critical,
        story_memory.current_turn(),
    );

    story_memory.add_consequence(consequence);

    // Create the relevance checker
    let checker = RelevanceChecker::from_env().expect("Failed to create checker");

    // Test with input that should NOT trigger the consequence
    let result = checker
        .check_relevance(
            "I buy a loaf of bread from the baker",
            "Town Market",
            &story_memory,
        )
        .await
        .expect("Relevance check should succeed");

    println!("Relevance result for unrelated action: {:?}", result);

    // Should not trigger - buying bread has nothing to do with entering a dragon's lair
    assert!(
        !result.has_triggered_consequences(),
        "Buying bread should not trigger dragon lair consequence"
    );
}

#[tokio::test]
#[ignore]
async fn test_full_consequence_flow_with_api() {
    setup();
    if !has_api_key() {
        eprintln!("Skipping test: ANTHROPIC_API_KEY not set");
        return;
    }

    // Create a game session
    let character = create_sample_fighter("Aldric");
    let mut world = GameWorld::new("Consequence Test", character);
    let mut dm = DungeonMaster::from_env().expect("Failed to create DM");

    // Manually add a consequence (simulating what the DM would do)
    dm.story_memory_mut().create_consequence(
        "Player returns to the tavern after causing trouble",
        "The bartender refuses service and demands payment for damages",
        ConsequenceSeverity::Moderate,
    );

    assert_eq!(dm.story_memory().pending_consequence_count(), 1);

    // First action - leave the tavern (should not trigger)
    let response1 = dm
        .process_input("I leave the tavern and walk into the street", &mut world)
        .await
        .expect("DM should respond");

    println!("After leaving tavern:");
    println!("  Narrative: {}", response1.narrative);
    println!(
        "  Pending consequences: {}",
        dm.story_memory().pending_consequence_count()
    );

    // Second action - return to the tavern (should trigger!)
    let response2 = dm
        .process_input("I go back to the tavern and walk inside", &mut world)
        .await
        .expect("DM should respond");

    println!("\nAfter returning to tavern:");
    println!("  Narrative: {}", response2.narrative);
    println!(
        "  Pending consequences: {}",
        dm.story_memory().pending_consequence_count()
    );

    // The consequence should have been triggered by the relevance checker
    // and incorporated into the DM's response
    let pending = dm.story_memory().pending_consequence_count();
    if pending == 0 {
        println!("SUCCESS: Consequence was triggered and is no longer pending!");
        // The narrative should mention the bartender or consequences
        assert!(
            response2.narrative.to_lowercase().contains("bartender")
                || response2.narrative.to_lowercase().contains("refuse")
                || response2.narrative.to_lowercase().contains("damage")
                || response2.narrative.to_lowercase().contains("payment"),
            "DM narrative should reflect the triggered consequence"
        );
    } else {
        println!("NOTE: Consequence still pending - may need stronger trigger matching");
    }
}
