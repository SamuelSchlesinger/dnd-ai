//! QA tests for combat mechanics in the headless D&D game API.
//!
//! These tests require ANTHROPIC_API_KEY to be set.
//! Run with: `cargo test -p dnd-core --test qa_combat -- --ignored --nocapture`

use dnd_core::headless::{HeadlessConfig, HeadlessGame};

/// Load environment variables from .env file
fn setup() {
    let _ = dotenvy::dotenv();
}

/// Check if API key is available
fn has_api_key() -> bool {
    std::env::var("ANTHROPIC_API_KEY").is_ok()
}

// =============================================================================
// TEST 1: Initiating Combat
// =============================================================================

#[tokio::test]
#[ignore]
async fn test_combat_initiation() {
    setup();
    if !has_api_key() {
        eprintln!("Skipping test: ANTHROPIC_API_KEY not set");
        return;
    }

    let config = HeadlessConfig::quick_start("Combat Tester");
    let mut game = HeadlessGame::new(config)
        .await
        .expect("Failed to create game");

    // Verify we start NOT in combat
    assert!(!game.in_combat(), "Should not be in combat initially");
    let initial_hp = game.current_hp();
    let max_hp = game.max_hp();
    println!(
        "Initial state: HP {}/{}, in_combat={}",
        initial_hp,
        max_hp,
        game.in_combat()
    );

    // Try to initiate combat by attacking
    let response = game
        .send("I draw my sword and attack the nearest goblin!")
        .await
        .expect("DM should respond");

    println!("\n=== Combat Initiation Response ===");
    println!("Narrative: {}", response.narrative);
    println!("in_combat: {}", response.in_combat);
    println!("is_player_turn: {}", response.is_player_turn);
    println!("HP: {}/{}", response.current_hp, response.max_hp);

    // Check combat state via game API as well
    let game_in_combat = game.in_combat();
    println!("\nGame state check: in_combat={}", game_in_combat);

    // Combat should have been initiated
    if response.in_combat {
        println!("SUCCESS: Combat was initiated!");
    } else {
        println!("WARNING: Combat may not have been initiated - DM might have described scenario differently");
        // This isn't necessarily a bug - the DM might say there's no goblin around
    }
}

// =============================================================================
// TEST 2: Combat State Tracking
// =============================================================================

#[tokio::test]
#[ignore]
async fn test_combat_state_tracking() {
    setup();
    if !has_api_key() {
        eprintln!("Skipping test: ANTHROPIC_API_KEY not set");
        return;
    }

    let config = HeadlessConfig::quick_start("State Tracker");
    let mut game = HeadlessGame::new(config)
        .await
        .expect("Failed to create game");

    // First, establish there's an enemy
    let response1 = game
        .send("I look around. Is there anyone hostile nearby?")
        .await
        .expect("DM should respond");

    println!("\n=== Scene Setup ===");
    println!("Narrative: {}", response1.narrative);

    // Now start combat explicitly
    let response2 = game
        .send("I attack! Roll for initiative!")
        .await
        .expect("DM should respond");

    println!("\n=== Combat Start ===");
    println!("Narrative: {}", response2.narrative);
    println!("in_combat: {}", response2.in_combat);
    println!("is_player_turn: {}", response2.is_player_turn);

    // Track state consistency
    let in_combat_response = response2.in_combat;
    let in_combat_game = game.in_combat();

    println!("\n=== State Consistency Check ===");
    println!("Response.in_combat: {}", in_combat_response);
    println!("game.in_combat(): {}", in_combat_game);

    // BUG CHECK: Response and game state should match
    if in_combat_response != in_combat_game {
        println!(
            "BUG FOUND: Response in_combat ({}) doesn't match game.in_combat() ({})",
            in_combat_response, in_combat_game
        );
    } else {
        println!("State consistency: OK");
    }

    // If in combat, take a turn and check turn tracking
    if game.in_combat() {
        let response3 = game
            .send("I swing my sword at the enemy")
            .await
            .expect("DM should respond");

        println!("\n=== After Attack ===");
        println!("Narrative: {}", response3.narrative);
        println!("in_combat: {}", response3.in_combat);
        println!("is_player_turn: {}", response3.is_player_turn);
        println!("turn_count: {}", game.turn_count());
    }
}

// =============================================================================
// TEST 3: Damage and Healing
// =============================================================================

#[tokio::test]
#[ignore]
async fn test_damage_and_healing() {
    setup();
    if !has_api_key() {
        eprintln!("Skipping test: ANTHROPIC_API_KEY not set");
        return;
    }

    let config = HeadlessConfig::quick_start("Damage Test Fighter");
    let mut game = HeadlessGame::new(config)
        .await
        .expect("Failed to create game");

    let initial_hp = game.current_hp();
    let max_hp = game.max_hp();
    println!("Starting HP: {}/{}", initial_hp, max_hp);

    // Engage in combat and try to get damaged
    let response1 = game
        .send("I charge recklessly at the nearest enemy, leaving myself open to attack!")
        .await
        .expect("DM should respond");

    println!("\n=== After Reckless Charge ===");
    println!("Narrative: {}", response1.narrative);
    println!("HP: {}/{}", game.current_hp(), game.max_hp());

    let hp_after_charge = game.current_hp();

    // Continue combat for a few rounds to potentially take damage
    for i in 1..=3 {
        if !game.in_combat() {
            println!("Combat ended after round {}", i);
            break;
        }

        let response = game
            .send("I attack the enemy!")
            .await
            .expect("DM should respond");

        println!("\n=== Round {} ===", i);
        println!(
            "Narrative: {}...",
            &response.narrative[..response.narrative.len().min(200)]
        );
        println!("HP: {}/{}", game.current_hp(), game.max_hp());
    }

    let final_hp = game.current_hp();
    println!("\n=== HP Summary ===");
    println!("Initial: {}", initial_hp);
    println!("After charge: {}", hp_after_charge);
    println!("Final: {}", final_hp);

    // Check if damage was tracked
    if final_hp < initial_hp {
        println!(
            "SUCCESS: Damage was tracked (lost {} HP)",
            initial_hp - final_hp
        );
    } else {
        println!("NOTE: No damage taken - enemies may have missed or combat went differently");
    }

    // Test healing via Second Wind (Fighter ability)
    if final_hp < max_hp {
        let healing_response = game
            .send("I use my Second Wind to heal myself!")
            .await
            .expect("DM should respond");

        println!("\n=== Second Wind ===");
        println!("Narrative: {}", healing_response.narrative);
        println!("HP after healing: {}/{}", game.current_hp(), game.max_hp());

        if game.current_hp() > final_hp {
            println!(
                "SUCCESS: Healing was tracked (gained {} HP)",
                game.current_hp() - final_hp
            );
        } else {
            println!("WARNING: Second Wind may not have healed or was already used");
        }
    }
}

// =============================================================================
// TEST 4: Combat Resolution
// =============================================================================

#[tokio::test]
#[ignore]
async fn test_combat_resolution() {
    setup();
    if !has_api_key() {
        eprintln!("Skipping test: ANTHROPIC_API_KEY not set");
        return;
    }

    let config = HeadlessConfig::quick_start("Victor");
    let mut game = HeadlessGame::new(config)
        .await
        .expect("Failed to create game");

    // Start combat
    let response1 = game
        .send("I attack the goblin!")
        .await
        .expect("DM should respond");

    println!("\n=== Combat Started ===");
    println!(
        "Narrative: {}...",
        &response1.narrative[..response1.narrative.len().min(200)]
    );
    println!("in_combat: {}", response1.in_combat);

    // Fight until combat ends (max 10 rounds to avoid infinite loop)
    let mut rounds = 0;
    while game.in_combat() && rounds < 10 {
        rounds += 1;
        let _response = game
            .send("I attack with all my might!")
            .await
            .expect("DM should respond");

        println!("\n=== Round {} ===", rounds);
        println!(
            "in_combat: {}, HP: {}/{}",
            game.in_combat(),
            game.current_hp(),
            game.max_hp()
        );
    }

    println!("\n=== Combat Resolution ===");
    if !game.in_combat() {
        println!("SUCCESS: Combat ended after {} rounds", rounds);
    } else {
        println!(
            "WARNING: Combat did not end after {} rounds (may need more)",
            rounds
        );
    }

    // After combat, verify we can do non-combat things
    if !game.in_combat() {
        let explore_response = game
            .send("I search the area for loot")
            .await
            .expect("DM should respond");

        println!("\n=== Post-Combat Exploration ===");
        println!(
            "Narrative: {}...",
            &explore_response.narrative[..explore_response.narrative.len().min(200)]
        );
        println!("in_combat: {}", explore_response.in_combat);

        if !explore_response.in_combat {
            println!("SUCCESS: Can explore after combat ends");
        }
    }
}

// =============================================================================
// TEST 5: HP Tracking During Combat
// =============================================================================

#[tokio::test]
#[ignore]
async fn test_hp_tracking_during_combat() {
    setup();
    if !has_api_key() {
        eprintln!("Skipping test: ANTHROPIC_API_KEY not set");
        return;
    }

    let config = HeadlessConfig::quick_start("HP Tracker");
    let mut game = HeadlessGame::new(config)
        .await
        .expect("Failed to create game");

    let initial_hp = game.current_hp();
    let max_hp = game.max_hp();

    println!("=== Initial HP Check ===");
    println!("current_hp(): {}", initial_hp);
    println!("max_hp(): {}", max_hp);

    // Verify HP is reasonable for a level 1 fighter (should be 10 + CON mod, so 10-14 typically)
    assert!(max_hp >= 10, "Fighter max HP should be at least 10");
    assert!(max_hp <= 20, "Level 1 Fighter max HP should not exceed 20");
    assert_eq!(initial_hp, max_hp, "Should start at full HP");

    // Start combat and track HP through multiple turns
    let mut hp_history: Vec<i32> = vec![initial_hp];

    let response1 = game
        .send("A goblin attacks me! I fight back!")
        .await
        .expect("DM should respond");

    println!("\n=== Combat Turn 1 ===");
    println!("Response HP: {}/{}", response1.current_hp, response1.max_hp);
    println!("game.current_hp(): {}", game.current_hp());

    // BUG CHECK: Response HP and game HP should match
    if response1.current_hp != game.current_hp() {
        println!(
            "BUG FOUND: Response HP ({}) doesn't match game.current_hp() ({})",
            response1.current_hp,
            game.current_hp()
        );
    }

    hp_history.push(game.current_hp());

    // Continue combat
    for turn in 2..=5 {
        if !game.in_combat() {
            println!("Combat ended at turn {}", turn);
            break;
        }

        let response = game
            .send("I continue fighting!")
            .await
            .expect("DM should respond");

        hp_history.push(game.current_hp());

        println!("\n=== Combat Turn {} ===", turn);
        println!("HP: {}/{}", game.current_hp(), game.max_hp());

        // BUG CHECK: HP should never exceed max
        if game.current_hp() > game.max_hp() {
            println!(
                "BUG FOUND: Current HP ({}) exceeds max HP ({})",
                game.current_hp(),
                game.max_hp()
            );
        }

        // BUG CHECK: HP should be consistent between response and game state
        if response.current_hp != game.current_hp() {
            println!(
                "BUG FOUND: Response HP ({}) doesn't match game.current_hp() ({})",
                response.current_hp,
                game.current_hp()
            );
        }
    }

    println!("\n=== HP History ===");
    for (i, hp) in hp_history.iter().enumerate() {
        println!("Turn {}: {} HP", i, hp);
    }

    // Check for HP tracking consistency
    let hp_changes: Vec<i32> = hp_history.windows(2).map(|w| w[1] - w[0]).collect();

    println!("\nHP changes between turns: {:?}", hp_changes);

    // Verify we tracked HP changes correctly
    if hp_changes.iter().any(|&c| c != 0) {
        println!("SUCCESS: HP changes were tracked during combat");
    } else {
        println!("NOTE: No HP changes detected - may indicate damage/healing not being tracked");
    }
}

// =============================================================================
// TEST 6: Comprehensive Combat Flow
// =============================================================================

#[tokio::test]
#[ignore]
async fn test_comprehensive_combat_flow() {
    setup();
    if !has_api_key() {
        eprintln!("Skipping test: ANTHROPIC_API_KEY not set");
        return;
    }

    let config = HeadlessConfig::quick_start("Comprehensive Tester");
    let mut game = HeadlessGame::new(config)
        .await
        .expect("Failed to create game");

    println!("=== COMPREHENSIVE COMBAT FLOW TEST ===\n");

    // Phase 1: Pre-combat state
    println!("--- Phase 1: Pre-Combat ---");
    assert!(!game.in_combat(), "Should not start in combat");
    let pre_combat_hp = game.current_hp();
    println!("Pre-combat HP: {}/{}", pre_combat_hp, game.max_hp());
    println!("Pre-combat in_combat: {}", game.in_combat());

    // Phase 2: Combat initiation
    println!("\n--- Phase 2: Combat Initiation ---");
    let init_response = game
        .send("I see a goblin and charge at it with my sword drawn!")
        .await
        .expect("DM should respond");

    println!(
        "Narrative excerpt: {}...",
        &init_response.narrative[..init_response.narrative.len().min(150)]
    );
    println!("in_combat (response): {}", init_response.in_combat);
    println!("in_combat (game): {}", game.in_combat());
    println!("is_player_turn: {}", init_response.is_player_turn);

    // Phase 3: Combat actions
    if game.in_combat() {
        println!("\n--- Phase 3: Combat Actions ---");

        // Attack action
        let attack_response = game
            .send("I attack the goblin with my sword!")
            .await
            .expect("DM should respond");

        println!("Attack response in_combat: {}", attack_response.in_combat);
        println!("HP after attack: {}/{}", game.current_hp(), game.max_hp());

        // Track conditions
        let conditions = game.conditions();
        if !conditions.is_empty() {
            println!("Active conditions: {:?}", conditions);
        }

        // Phase 4: Attempt to end combat
        println!("\n--- Phase 4: Combat Resolution Attempt ---");

        // Try multiple attacks to defeat enemy
        for i in 1..=5 {
            if !game.in_combat() {
                println!("Combat ended at attempt {}", i);
                break;
            }

            let _ = game.send("I strike again!").await;
            println!(
                "Attempt {}: in_combat={}, HP={}",
                i,
                game.in_combat(),
                game.current_hp()
            );
        }
    } else {
        println!("WARNING: Combat was not initiated - DM may have described scene differently");
    }

    // Phase 5: Post-combat verification
    println!("\n--- Phase 5: Post-Combat State ---");
    let post_combat_hp = game.current_hp();
    println!("Post-combat HP: {}/{}", post_combat_hp, game.max_hp());
    println!("Post-combat in_combat: {}", game.in_combat());

    // Summary
    println!("\n=== TEST SUMMARY ===");
    println!(
        "HP change: {} -> {} (delta: {})",
        pre_combat_hp,
        post_combat_hp,
        post_combat_hp - pre_combat_hp
    );
    println!(
        "Combat tracked: {}",
        game.in_combat() || pre_combat_hp != post_combat_hp
    );
}
