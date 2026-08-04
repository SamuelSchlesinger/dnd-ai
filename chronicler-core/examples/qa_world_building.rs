//! QA Test: World Building Tools
//!
//! This example tests the DM's ability to use world-building tools like
//! creating NPCs, locations, updating NPC dispositions, and using
//! the proactive world design tools through natural player interactions.
//!
//! Run with: cargo run -p chronicler-core --example qa_world_building

use chronicler_core::headless::{HeadlessConfig, HeadlessGame};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env file
    let _ = dotenvy::dotenv();

    // Check for provider configuration
    if chronicler_core::LlmConfig::from_env().is_err() {
        eprintln!("Error: no LLM provider is configured");
        eprintln!("Configure one in your environment or create a .env file");
        std::process::exit(1);
    }

    println!("=== QA Test: World Building Tools ===\n");

    // Create game
    let config = HeadlessConfig::quick_start("QA Tester");
    let mut game = HeadlessGame::new(config).await?;

    println!("Game created successfully!");
    println!("Character: {}", game.player_name());
    println!("Location: {}", game.current_location());
    println!();

    // Show initial story memory state
    let story_memory = game.session().dm().story_memory();
    println!("Initial story memory state:");
    println!("  Entities: {}", story_memory.entity_count());
    println!("  Facts: {}", story_memory.fact_count());
    println!(
        "  Pending consequences: {}",
        story_memory.pending_consequences().len()
    );
    println!(
        "  Scheduled events: {}",
        story_memory.pending_events().len()
    );
    println!();

    // Initial state
    let initial_npcs = game.session().world().npcs.len();
    let initial_locations = game.session().world().known_locations.len();
    println!("Initial state:");
    println!("  NPCs: {}", initial_npcs);
    println!("  Known locations: {}", initial_locations);
    println!();

    // Test NPC creation
    println!("--- Test 1: NPC Creation ---");
    let response = game
        .send("I approach the bartender and introduce myself. What's their name?")
        .await?;
    println!("Player: I approach the bartender and introduce myself. What's their name?");
    println!("DM Response:\n{}\n", response.narrative);
    println!(
        "  NPCs after interaction: {}",
        game.session().world().npcs.len()
    );
    println!(
        "  Locations after interaction: {}",
        game.session().world().known_locations.len()
    );
    println!();

    // Test location creation
    println!("--- Test 2: Location Creation ---");
    let response = game
        .send("I ask about interesting places nearby that I could explore")
        .await?;
    println!("Player: I ask about interesting places nearby that I could explore");
    println!("DM Response:\n{}\n", response.narrative);
    println!(
        "  NPCs after interaction: {}",
        game.session().world().npcs.len()
    );
    println!(
        "  Locations after interaction: {}",
        game.session().world().known_locations.len()
    );
    println!();

    // Test location connection
    println!("--- Test 3: Location Connection ---");
    let response = game.send("How do I get to those places from here?").await?;
    println!("Player: How do I get to those places from here?");
    println!("DM Response:\n{}\n", response.narrative);
    println!(
        "  NPCs after interaction: {}",
        game.session().world().npcs.len()
    );
    println!(
        "  Locations after interaction: {}",
        game.session().world().known_locations.len()
    );
    println!();

    // Test NPC update (disposition change)
    println!("--- Test 4: NPC Disposition Update ---");
    let response = game
        .send("I buy the bartender a drink and try to befriend them")
        .await?;
    println!("Player: I buy the bartender a drink and try to befriend them");
    println!("DM Response:\n{}\n", response.narrative);
    println!(
        "  NPCs after interaction: {}",
        game.session().world().npcs.len()
    );
    println!(
        "  Locations after interaction: {}",
        game.session().world().known_locations.len()
    );
    println!();

    // Test proactive world design - entering a new location
    println!("--- Test 5: Proactive World Design (New Location) ---");
    let response = game
        .send("I leave the inn and walk to the market square to look around")
        .await?;
    println!("Player: I leave the inn and walk to the market square to look around");
    println!("DM Response:\n{}\n", response.narrative);
    println!(
        "  NPCs after entering market: {}",
        game.session().world().npcs.len()
    );
    println!(
        "  Locations after entering market: {}",
        game.session().world().known_locations.len()
    );

    // Check story memory for scheduled events and consequences
    let story_memory = game.session().dm().story_memory();
    println!(
        "  Scheduled events: {}",
        story_memory.pending_events().len()
    );
    println!(
        "  Pending consequences: {}",
        story_memory.pending_consequences().len()
    );
    println!();

    // Test that relationships exist in facts
    println!("--- Test 6: Relationship and Secret Tracking ---");
    let response = game
        .send("I approach a merchant and ask about local rumors and gossip")
        .await?;
    println!("Player: I approach a merchant and ask about local rumors and gossip");
    println!("DM Response:\n{}\n", response.narrative);

    // Check facts for relationship/backstory information
    let story_memory = game.session().dm().story_memory();
    // Use recent_facts with a large window to get all facts from this session
    let all_recent_facts = story_memory.recent_facts(10000);
    let relationship_facts: Vec<_> = all_recent_facts
        .iter()
        .filter(|f| {
            matches!(
                f.category,
                chronicler_core::dm::story_memory::FactCategory::Relationship
                    | chronicler_core::dm::story_memory::FactCategory::Backstory
            )
        })
        .collect();
    println!(
        "  Facts about relationships/backstories: {}",
        relationship_facts.len()
    );
    for fact in relationship_facts.iter().take(5) {
        println!("    - [{:?}]: {}", fact.category, fact.content);
    }
    if relationship_facts.len() > 5 {
        println!("    ... and {} more", relationship_facts.len() - 5);
    }
    println!();

    // Final summary
    println!("=== Summary ===");
    let final_npcs = game.session().world().npcs.len();
    let final_locations = game.session().world().known_locations.len();
    let final_story_memory = game.session().dm().story_memory();

    println!("Total NPCs created: {} (was {})", final_npcs, initial_npcs);
    println!(
        "Total locations known: {} (was {})",
        final_locations, initial_locations
    );
    println!(
        "Total entities in story memory: {}",
        final_story_memory.entity_count()
    );
    println!("Total facts recorded: {}", final_story_memory.fact_count());
    println!(
        "Total scheduled events: {}",
        final_story_memory.pending_events().len()
    );
    println!(
        "Total pending consequences: {}",
        final_story_memory.pending_consequences().len()
    );
    println!();

    // List all NPCs
    println!("All NPCs:");
    for (id, npc) in &game.session().world().npcs {
        println!(
            "  - {} (ID: {}, Disposition: {:?})",
            npc.name, id, npc.disposition
        );
        if !npc.description.is_empty() {
            println!("    Description: {}", npc.description);
        }
        if let Some(ref occupation) = npc.occupation {
            println!("    Occupation: {}", occupation);
        }
    }
    println!();

    // List all locations
    println!("All known locations:");
    for (id, location) in &game.session().world().known_locations {
        println!(
            "  - {} (ID: {:?}, Type: {:?})",
            location.name, id, location.location_type
        );
        if !location.description.is_empty() {
            let desc_preview: String = location.description.chars().take(100).collect();
            println!(
                "    Description: {}{}",
                desc_preview,
                if location.description.len() > 100 {
                    "..."
                } else {
                    ""
                }
            );
        }
        if !location.connections.is_empty() {
            println!("    Connections:");
            for conn in &location.connections {
                println!(
                    "      -> {} ({} min travel)",
                    conn.destination_name, conn.travel_time_minutes
                );
            }
        }
    }
    println!();

    // List scheduled events
    let events = final_story_memory.pending_events();
    if !events.is_empty() {
        println!("Scheduled events:");
        for event in &events {
            println!(
                "  - {} (visibility: {:?})",
                event.description, event.visibility
            );
            if let Some(ref loc) = event.location {
                println!("    Location: {}", loc);
            }
        }
        println!();
    }

    // List pending consequences
    let consequences = final_story_memory.pending_consequences();
    if !consequences.is_empty() {
        println!("Pending consequences:");
        for consequence in &consequences {
            println!(
                "  - Trigger: {} -> {}",
                consequence.trigger_description, consequence.consequence_description
            );
            println!("    Severity: {:?}", consequence.severity);
        }
        println!();
    }

    // Verification checks
    println!("=== Verification ===");
    let mut all_passed = true;

    // Check that NPCs were created
    if final_npcs > initial_npcs {
        println!("[PASS] NPCs were created during gameplay");
    } else {
        println!("[FAIL] No NPCs were created");
        all_passed = false;
    }

    // Check that locations were created
    if final_locations > initial_locations {
        println!("[PASS] Locations were created/discovered");
    } else {
        println!("[FAIL] No new locations were created");
        all_passed = false;
    }

    // Check that facts were recorded
    if final_story_memory.fact_count() > 0 {
        println!(
            "[PASS] Facts were recorded ({} total)",
            final_story_memory.fact_count()
        );
    } else {
        println!("[FAIL] No facts were recorded");
        all_passed = false;
    }

    // Check for relationship/backstory facts (indicates design_environment or remember_fact usage)
    let all_final_facts = final_story_memory.recent_facts(10000);
    let rich_facts: Vec<_> = all_final_facts
        .iter()
        .filter(|f| {
            matches!(
                f.category,
                chronicler_core::dm::story_memory::FactCategory::Relationship
                    | chronicler_core::dm::story_memory::FactCategory::Backstory
            )
        })
        .collect();
    if !rich_facts.is_empty() {
        println!(
            "[PASS] Rich NPC data recorded ({} relationship/backstory facts)",
            rich_facts.len()
        );
    } else {
        println!("[INFO] No relationship/backstory facts recorded (DM may not have used design_environment)");
    }

    // Check for scheduled events
    if !events.is_empty() {
        println!("[PASS] Scheduled events exist ({} events)", events.len());
    } else {
        println!("[INFO] No scheduled events (DM may not have scheduled any)");
    }

    // Check for consequences
    if !consequences.is_empty() {
        println!(
            "[PASS] Consequences registered ({} pending)",
            consequences.len()
        );
    } else {
        println!("[INFO] No consequences registered");
    }

    println!();
    if all_passed {
        println!("=== QA Test Complete: All core checks PASSED ===");
    } else {
        println!("=== QA Test Complete: Some checks FAILED ===");
    }

    Ok(())
}
