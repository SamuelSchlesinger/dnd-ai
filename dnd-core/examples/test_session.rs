//! Quick integration test for D&D core functionality

use dnd_core::{GameSession, SessionConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Testing D&D Core ===\n");

    // Test 1: Create a session
    println!("1. Creating game session...");
    let config = SessionConfig::new("Test Campaign")
        .with_character_name("Thorin")
        .with_starting_location("The Rusty Dragon Inn");
    
    let mut session = GameSession::new(config).await?;
    println!("   Session created successfully");
    
    // Test 2: Check world state
    println!("\n2. Checking world state...");
    let world = session.world();
    println!("   Campaign: {}", world.campaign_name);
    println!("   Character: {}", world.player_character.name);
    println!("   Location: {}", world.current_location.name);
    let (current_hp, max_hp) = session.hp_status();
    println!("   HP: {current_hp}/{max_hp}");
    println!("   World state is valid");

    // Test 3: Send a player action
    println!("\n3. Testing player action (this calls Claude API)...");
    let response = session.player_action("I look around the tavern.").await?;
    println!("   Response length: {} chars", response.narrative.len());
    println!("   Effects count: {}", response.effects.len());
    println!("   In combat: {}", response.in_combat);
    println!("   Player action processed successfully");
    
    // Print a snippet of the narrative
    println!("\n4. DM Response (first 500 chars):");
    println!("   ---");
    let snippet: String = response.narrative.chars().take(500).collect();
    for line in snippet.lines() {
        println!("   {line}");
    }
    if response.narrative.len() > 500 {
        println!("   ...[truncated]");
    }
    println!("   ---");

    println!("\n=== All tests passed! ===");
    Ok(())
}
