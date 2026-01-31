//! Integration tests that call the real Claude API.
//!
//! These tests require ANTHROPIC_API_KEY to be set (via .env file or environment).
//! Run with: `cargo test -p dnd-core --test api_integration -- --ignored`
//!
//! These are marked #[ignore] by default to avoid:
//! - API costs in CI
//! - Test failures when no API key is available
//! - Slow test runs (API calls take seconds)

use dnd_core::dm::story_memory::ConsequenceSeverity;
use dnd_core::dm::{DmConfig, DungeonMaster, RelevanceChecker, StoryMemory};
use dnd_core::world::{
    create_sample_fighter, Ability, AbilityScores, Character, CharacterClass, GameWorld, Race,
    SlotInfo, SpellSlots, SpellcastingData, NPC,
};

/// Create a sample wizard for spellcasting tests.
fn create_sample_wizard(name: &str) -> Character {
    let mut character = Character::new(name);

    character.ability_scores = AbilityScores::new(8, 14, 14, 16, 12, 10);
    character.race = Race {
        name: "Human".to_string(),
        subrace: None,
        race_type: Some(dnd_core::world::RaceType::Human),
    };
    character.classes = vec![dnd_core::world::ClassLevel {
        class: CharacterClass::Wizard,
        level: 3,
        subclass: Some("School of Evocation".to_string()),
    }];
    character.level = 3;

    // Set up spellcasting
    character.spellcasting = Some(SpellcastingData {
        ability: Ability::Intelligence,
        spells_known: vec![
            "Magic Missile".to_string(),
            "Shield".to_string(),
            "Burning Hands".to_string(),
            "Mage Armor".to_string(),
            "Thunderwave".to_string(),
            "Sleep".to_string(),
        ],
        spells_prepared: vec![
            "Magic Missile".to_string(),
            "Shield".to_string(),
            "Burning Hands".to_string(),
            "Mage Armor".to_string(),
        ],
        cantrips_known: vec![
            "Fire Bolt".to_string(),
            "Light".to_string(),
            "Mage Hand".to_string(),
        ],
        spell_slots: SpellSlots {
            slots: [
                SlotInfo { total: 4, used: 0 }, // 1st level: 4 slots
                SlotInfo { total: 2, used: 0 }, // 2nd level: 2 slots
                SlotInfo { total: 0, used: 0 }, // 3rd level
                SlotInfo { total: 0, used: 0 },
                SlotInfo { total: 0, used: 0 },
                SlotInfo { total: 0, used: 0 },
                SlotInfo { total: 0, used: 0 },
                SlotInfo { total: 0, used: 0 },
                SlotInfo { total: 0, used: 0 },
            ],
        },
    });

    // Set hit points
    character.hit_points.maximum = 20;
    character.hit_points.current = 20;

    // Add proficiencies
    character
        .saving_throw_proficiencies
        .insert(Ability::Intelligence);
    character.saving_throw_proficiencies.insert(Ability::Wisdom);

    character
}

/// Create a sample cleric for healing spell tests.
fn create_sample_cleric(name: &str) -> Character {
    let mut character = Character::new(name);

    character.ability_scores = AbilityScores::new(14, 10, 14, 10, 16, 12);
    character.race = Race {
        name: "Human".to_string(),
        subrace: None,
        race_type: Some(dnd_core::world::RaceType::Human),
    };
    character.classes = vec![dnd_core::world::ClassLevel {
        class: CharacterClass::Cleric,
        level: 3,
        subclass: Some("Life Domain".to_string()),
    }];
    character.level = 3;

    // Set up spellcasting
    character.spellcasting = Some(SpellcastingData {
        ability: Ability::Wisdom,
        spells_known: vec![],
        spells_prepared: vec![
            "Cure Wounds".to_string(),
            "Healing Word".to_string(),
            "Guiding Bolt".to_string(),
            "Shield of Faith".to_string(),
        ],
        cantrips_known: vec![
            "Sacred Flame".to_string(),
            "Light".to_string(),
            "Guidance".to_string(),
        ],
        spell_slots: SpellSlots {
            slots: [
                SlotInfo { total: 4, used: 0 }, // 1st level: 4 slots
                SlotInfo { total: 2, used: 0 }, // 2nd level: 2 slots
                SlotInfo { total: 0, used: 0 },
                SlotInfo { total: 0, used: 0 },
                SlotInfo { total: 0, used: 0 },
                SlotInfo { total: 0, used: 0 },
                SlotInfo { total: 0, used: 0 },
                SlotInfo { total: 0, used: 0 },
                SlotInfo { total: 0, used: 0 },
            ],
        },
    });

    // Set hit points
    character.hit_points.maximum = 24;
    character.hit_points.current = 24;

    character.saving_throw_proficiencies.insert(Ability::Wisdom);
    character
        .saving_throw_proficiencies
        .insert(Ability::Charisma);

    character
}

/// Create a sample barbarian for rage tests.
fn create_sample_barbarian(name: &str) -> Character {
    let mut character = Character::new(name);

    character.ability_scores = AbilityScores::new(16, 14, 16, 8, 12, 10);
    character.race = Race {
        name: "Human".to_string(),
        subrace: None,
        race_type: Some(dnd_core::world::RaceType::Human),
    };
    character.classes = vec![dnd_core::world::ClassLevel {
        class: CharacterClass::Barbarian,
        level: 3,
        subclass: Some("Path of the Berserker".to_string()),
    }];
    character.level = 3;

    // Barbarian rage is tracked via rage_active, not uses
    // The DM/rules manage when rage can be used based on features
    character.class_resources.rage_active = false;

    // Set hit points (barbarians have d12 hit dice + high CON)
    character.hit_points.maximum = 38;
    character.hit_points.current = 38;

    character
        .saving_throw_proficiencies
        .insert(Ability::Strength);
    character
        .saving_throw_proficiencies
        .insert(Ability::Constitution);

    character
}

/// Helper to add an NPC to the world
fn add_hostile_npc(world: &mut GameWorld, name: &str, description: &str) {
    let mut npc = NPC::new(name);
    npc.description = description.to_string();
    npc.disposition = dnd_core::world::Disposition::Hostile;
    world.npcs.insert(npc.id, npc);
}

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
    assert!(
        !response.narrative.is_empty(),
        "DM should provide a narrative"
    );

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
    let riverside_id =
        story_memory.create_entity(dnd_core::dm::EntityType::Location, "Riverside Village");

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
    println!(
        "Triggered consequences: {:?}",
        result.triggered_consequences
    );
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

// =============================================================================
// SPELLCASTING INTEGRATION TESTS
// =============================================================================

#[tokio::test]
#[ignore]
async fn test_dm_handles_cantrip_casting() {
    setup();
    if !has_api_key() {
        eprintln!("Skipping test: ANTHROPIC_API_KEY not set");
        return;
    }

    let character = create_sample_wizard("Merlin");
    let mut world = GameWorld::new("Spell Test", character);
    let mut dm = DungeonMaster::from_env().expect("Failed to create DM");

    dm = dm.with_config(DmConfig {
        model: Some("claude-sonnet-4-20250514".to_string()),
        max_tokens: 1024,
        temperature: Some(0.7),
        custom_system_prompt: None,
    });

    // Add an enemy to target
    add_hostile_npc(&mut world, "Goblin", "A snarling goblin");

    // Cast a cantrip - should NOT consume spell slots
    let initial_slots = world
        .player_character
        .spellcasting
        .as_ref()
        .map(|s| s.spell_slots.slots[0].used)
        .unwrap_or(0);

    let response = dm
        .process_input("I cast Fire Bolt at the goblin", &mut world)
        .await
        .expect("DM should respond");

    println!("Cantrip cast response: {}", response.narrative);
    println!("Intents: {:?}", response.intents);
    println!("Effects: {:?}", response.effects);

    // Cantrips don't use slots
    let final_slots = world
        .player_character
        .spellcasting
        .as_ref()
        .map(|s| s.spell_slots.slots[0].used)
        .unwrap_or(0);

    assert_eq!(
        initial_slots, final_slots,
        "Cantrips should not consume spell slots"
    );

    // Response should mention fire/damage or the spell
    assert!(
        !response.narrative.is_empty(),
        "DM should provide narrative"
    );
}

#[tokio::test]
#[ignore]
async fn test_dm_handles_leveled_spell_casting() {
    setup();
    if !has_api_key() {
        eprintln!("Skipping test: ANTHROPIC_API_KEY not set");
        return;
    }

    let character = create_sample_wizard("Gandalf");
    let mut world = GameWorld::new("Spell Slot Test", character);
    let mut dm = DungeonMaster::from_env().expect("Failed to create DM");

    dm = dm.with_config(DmConfig {
        model: Some("claude-sonnet-4-20250514".to_string()),
        max_tokens: 1024,
        temperature: Some(0.7),
        custom_system_prompt: None,
    });

    // Add enemies
    add_hostile_npc(&mut world, "Goblin Scout", "A goblin with a bow");

    // Verify initial spell slots
    let initial_1st_level_used = world
        .player_character
        .spellcasting
        .as_ref()
        .map(|s| s.spell_slots.slots[0].used)
        .unwrap_or(0);

    println!("Initial 1st level slots used: {}", initial_1st_level_used);

    // Cast Magic Missile - a 1st level spell
    let response = dm
        .process_input("I cast Magic Missile at the goblin", &mut world)
        .await
        .expect("DM should respond");

    println!("Leveled spell response: {}", response.narrative);
    println!("Intents: {:?}", response.intents);
    println!("Effects: {:?}", response.effects);

    // Check if a spell slot was consumed
    let final_1st_level_used = world
        .player_character
        .spellcasting
        .as_ref()
        .map(|s| s.spell_slots.slots[0].used)
        .unwrap_or(0);

    println!("Final 1st level slots used: {}", final_1st_level_used);

    // The DM should have used the cast_spell tool which consumes a slot
    if final_1st_level_used > initial_1st_level_used {
        println!("SUCCESS: Spell slot was consumed!");
    } else {
        println!("NOTE: Spell slot was not consumed - DM may have described casting differently");
    }

    assert!(
        !response.narrative.is_empty(),
        "DM should provide narrative"
    );
}

#[tokio::test]
#[ignore]
async fn test_dm_handles_healing_spell() {
    setup();
    if !has_api_key() {
        eprintln!("Skipping test: ANTHROPIC_API_KEY not set");
        return;
    }

    let mut character = create_sample_cleric("Elara");
    // Damage the cleric so healing is relevant
    character.hit_points.current = 10;

    let mut world = GameWorld::new("Healing Test", character);
    let mut dm = DungeonMaster::from_env().expect("Failed to create DM");

    dm = dm.with_config(DmConfig {
        model: Some("claude-sonnet-4-20250514".to_string()),
        max_tokens: 1024,
        temperature: Some(0.7),
        custom_system_prompt: None,
    });

    let initial_hp = world.player_character.hit_points.current;
    println!(
        "Initial HP: {}/{}",
        initial_hp, world.player_character.hit_points.maximum
    );

    // Cast Cure Wounds on self
    let response = dm
        .process_input("I cast Cure Wounds on myself", &mut world)
        .await
        .expect("DM should respond");

    println!("Healing spell response: {}", response.narrative);
    println!("Intents: {:?}", response.intents);
    println!("Effects: {:?}", response.effects);

    let final_hp = world.player_character.hit_points.current;
    println!(
        "Final HP: {}/{}",
        final_hp, world.player_character.hit_points.maximum
    );

    // HP should have increased (or stayed same if at max)
    if final_hp > initial_hp {
        println!(
            "SUCCESS: Character was healed from {} to {} HP!",
            initial_hp, final_hp
        );
    } else {
        println!("NOTE: HP did not increase - DM may have handled healing differently");
    }

    assert!(
        !response.narrative.is_empty(),
        "DM should provide narrative"
    );
}

// =============================================================================
// CLASS RESOURCE INTEGRATION TESTS
// =============================================================================

#[tokio::test]
#[ignore]
async fn test_dm_handles_barbarian_rage() {
    setup();
    if !has_api_key() {
        eprintln!("Skipping test: ANTHROPIC_API_KEY not set");
        return;
    }

    let character = create_sample_barbarian("Grog");
    let mut world = GameWorld::new("Rage Test", character);
    let mut dm = DungeonMaster::from_env().expect("Failed to create DM");

    dm = dm.with_config(DmConfig {
        model: Some("claude-sonnet-4-20250514".to_string()),
        max_tokens: 1024,
        temperature: Some(0.7),
        custom_system_prompt: None,
    });

    // Add an enemy to trigger combat context
    add_hostile_npc(&mut world, "Orc Warrior", "A fierce orc with a greataxe");

    let initial_rage_active = world.player_character.class_resources.rage_active;
    println!("Initial rage active: {}", initial_rage_active);

    // Enter rage
    let response = dm
        .process_input("I enter a rage and attack the orc!", &mut world)
        .await
        .expect("DM should respond");

    println!("Rage response: {}", response.narrative);
    println!("Intents: {:?}", response.intents);
    println!("Effects: {:?}", response.effects);

    let final_rage_active = world.player_character.class_resources.rage_active;

    println!("Final rage active: {}", final_rage_active);

    if final_rage_active {
        println!("SUCCESS: Barbarian rage was activated!");
    } else {
        println!("NOTE: Rage tracking may not have been updated - DM handled it narratively");
    }

    // Response should acknowledge the rage
    assert!(
        !response.narrative.is_empty(),
        "DM should provide narrative"
    );
}

#[tokio::test]
#[ignore]
async fn test_dm_handles_fighter_action_surge() {
    setup();
    if !has_api_key() {
        eprintln!("Skipping test: ANTHROPIC_API_KEY not set");
        return;
    }

    let mut character = create_sample_fighter("Ser Roland");
    // Fighters get Action Surge at level 2 - tracked as a boolean (used/not used)
    character.class_resources.action_surge_used = false;

    let mut world = GameWorld::new("Action Surge Test", character);
    let mut dm = DungeonMaster::from_env().expect("Failed to create DM");

    dm = dm.with_config(DmConfig {
        model: Some("claude-sonnet-4-20250514".to_string()),
        max_tokens: 1024,
        temperature: Some(0.7),
        custom_system_prompt: None,
    });

    // Add enemies
    add_hostile_npc(&mut world, "Bandit", "A dangerous bandit");

    let initial_action_surge_used = world.player_character.class_resources.action_surge_used;
    println!("Initial Action Surge used: {}", initial_action_surge_used);

    // Use Action Surge
    let response = dm
        .process_input(
            "I use Action Surge to attack the bandit twice in one turn!",
            &mut world,
        )
        .await
        .expect("DM should respond");

    println!("Action Surge response: {}", response.narrative);
    println!("Intents: {:?}", response.intents);
    println!("Effects: {:?}", response.effects);

    let final_action_surge_used = world.player_character.class_resources.action_surge_used;
    println!("Final Action Surge used: {}", final_action_surge_used);

    if final_action_surge_used && !initial_action_surge_used {
        println!("SUCCESS: Action Surge was consumed!");
    } else {
        println!("NOTE: Action Surge not tracked - DM may have handled it narratively");
    }

    assert!(
        !response.narrative.is_empty(),
        "DM should provide narrative"
    );
}

// =============================================================================
// COMBAT INTEGRATION TESTS
// =============================================================================

#[tokio::test]
#[ignore]
async fn test_dm_handles_attack_roll() {
    setup();
    if !has_api_key() {
        eprintln!("Skipping test: ANTHROPIC_API_KEY not set");
        return;
    }

    let character = create_sample_fighter("Warrior");
    let mut world = GameWorld::new("Combat Test", character);
    let mut dm = DungeonMaster::from_env().expect("Failed to create DM");

    dm = dm.with_config(DmConfig {
        model: Some("claude-sonnet-4-20250514".to_string()),
        max_tokens: 1024,
        temperature: Some(0.7),
        custom_system_prompt: None,
    });

    // Add an enemy
    add_hostile_npc(&mut world, "Skeleton", "An animated skeleton warrior");

    let response = dm
        .process_input("I attack the skeleton with my longsword", &mut world)
        .await
        .expect("DM should respond");

    println!("Attack response: {}", response.narrative);
    println!("Intents: {:?}", response.intents);
    println!("Effects: {:?}", response.effects);

    // Check if an attack intent was generated
    let has_attack_intent = response
        .intents
        .iter()
        .any(|i| matches!(i, dnd_core::rules::Intent::Attack { .. }));

    if has_attack_intent {
        println!("SUCCESS: Attack intent was generated!");
    } else {
        println!("Intents found: {:?}", response.intents);
    }

    // Check if damage was dealt (HP changed with negative amount)
    let has_damage_effect = response
        .effects
        .iter()
        .any(|e| matches!(e, dnd_core::rules::Effect::HpChanged { amount, .. } if *amount < 0));

    if has_damage_effect {
        println!("SUCCESS: Damage effect was generated!");
    }

    assert!(
        !response.narrative.is_empty(),
        "DM should provide narrative"
    );
}

#[tokio::test]
#[ignore]
async fn test_dm_handles_skill_check() {
    setup();
    if !has_api_key() {
        eprintln!("Skipping test: ANTHROPIC_API_KEY not set");
        return;
    }

    let character = create_sample_fighter("Scout");
    let mut world = GameWorld::new("Skill Check Test", character);
    let mut dm = DungeonMaster::from_env().expect("Failed to create DM");

    dm = dm.with_config(DmConfig {
        model: Some("claude-sonnet-4-20250514".to_string()),
        max_tokens: 1024,
        temperature: Some(0.7),
        custom_system_prompt: None,
    });

    // Request something that should trigger a skill check
    let response = dm
        .process_input(
            "I carefully search the room for hidden doors or traps",
            &mut world,
        )
        .await
        .expect("DM should respond");

    println!("Skill check response: {}", response.narrative);
    println!("Intents: {:?}", response.intents);
    println!("Effects: {:?}", response.effects);

    // Check if a skill check intent was generated
    let has_skill_check = response.intents.iter().any(|i| {
        matches!(
            i,
            dnd_core::rules::Intent::SkillCheck { .. }
                | dnd_core::rules::Intent::AbilityCheck { .. }
        )
    });

    if has_skill_check {
        println!("SUCCESS: Skill check intent was generated!");
    }

    assert!(
        !response.narrative.is_empty(),
        "DM should provide narrative"
    );
}

// =============================================================================
// REST AND RECOVERY INTEGRATION TESTS
// =============================================================================

#[tokio::test]
#[ignore]
async fn test_dm_handles_short_rest() {
    setup();
    if !has_api_key() {
        eprintln!("Skipping test: ANTHROPIC_API_KEY not set");
        return;
    }

    let mut character = create_sample_fighter("Weary Warrior");
    // Damage the fighter and use some resources
    character.hit_points.current = 15;
    character.class_resources.second_wind_used = true;
    // Hit dice uses a HashMap for remaining dice per die type
    // The sample fighter has d10 hit dice

    let mut world = GameWorld::new("Rest Test", character);
    let mut dm = DungeonMaster::from_env().expect("Failed to create DM");

    dm = dm.with_config(DmConfig {
        model: Some("claude-sonnet-4-20250514".to_string()),
        max_tokens: 1024,
        temperature: Some(0.7),
        custom_system_prompt: None,
    });

    println!(
        "Before rest - HP: {}, Second Wind used: {}",
        world.player_character.hit_points.current,
        world.player_character.class_resources.second_wind_used
    );

    let response = dm
        .process_input("I take a short rest to recover", &mut world)
        .await
        .expect("DM should respond");

    println!("Short rest response: {}", response.narrative);
    println!("Intents: {:?}", response.intents);
    println!("Effects: {:?}", response.effects);

    println!(
        "After rest - HP: {}, Second Wind used: {}",
        world.player_character.hit_points.current,
        world.player_character.class_resources.second_wind_used
    );

    // Check if short rest intent was generated
    let has_rest_intent = response
        .intents
        .iter()
        .any(|i| matches!(i, dnd_core::rules::Intent::ShortRest));

    if has_rest_intent {
        println!("SUCCESS: Short rest intent was generated!");
    }

    // Second Wind should be recovered on short rest
    if !world.player_character.class_resources.second_wind_used {
        println!("SUCCESS: Second Wind was recovered!");
    }

    assert!(
        !response.narrative.is_empty(),
        "DM should provide narrative"
    );
}

#[tokio::test]
#[ignore]
async fn test_dm_handles_long_rest_spell_recovery() {
    setup();
    if !has_api_key() {
        eprintln!("Skipping test: ANTHROPIC_API_KEY not set");
        return;
    }

    let mut character = create_sample_wizard("Exhausted Mage");
    // Use some spell slots
    if let Some(ref mut spellcasting) = character.spellcasting {
        spellcasting.spell_slots.slots[0].used = 3; // Used 3 of 4 first level slots
        spellcasting.spell_slots.slots[1].used = 1; // Used 1 of 2 second level slots
    }
    character.hit_points.current = 10;

    let mut world = GameWorld::new("Long Rest Test", character);
    let mut dm = DungeonMaster::from_env().expect("Failed to create DM");

    dm = dm.with_config(DmConfig {
        model: Some("claude-sonnet-4-20250514".to_string()),
        max_tokens: 1024,
        temperature: Some(0.7),
        custom_system_prompt: None,
    });

    let initial_slots_used = world
        .player_character
        .spellcasting
        .as_ref()
        .map(|s| s.spell_slots.slots[0].used)
        .unwrap_or(0);

    println!(
        "Before long rest - HP: {}, 1st level slots used: {}",
        world.player_character.hit_points.current, initial_slots_used
    );

    let response = dm
        .process_input("I find a safe place and take a long rest", &mut world)
        .await
        .expect("DM should respond");

    println!("Long rest response: {}", response.narrative);
    println!("Intents: {:?}", response.intents);
    println!("Effects: {:?}", response.effects);

    let final_slots_used = world
        .player_character
        .spellcasting
        .as_ref()
        .map(|s| s.spell_slots.slots[0].used)
        .unwrap_or(0);

    println!(
        "After long rest - HP: {}, 1st level slots used: {}",
        world.player_character.hit_points.current, final_slots_used
    );

    // Check if long rest intent was generated
    let has_rest_intent = response
        .intents
        .iter()
        .any(|i| matches!(i, dnd_core::rules::Intent::LongRest));

    if has_rest_intent {
        println!("SUCCESS: Long rest intent was generated!");
    }

    // Spell slots should be recovered
    if final_slots_used < initial_slots_used {
        println!("SUCCESS: Spell slots were recovered!");
    }

    // HP should be restored
    if world.player_character.hit_points.current == world.player_character.hit_points.maximum {
        println!("SUCCESS: HP was fully restored!");
    }

    assert!(
        !response.narrative.is_empty(),
        "DM should provide narrative"
    );
}

// =============================================================================
// MULTI-TURN INTEGRATION TESTS
// =============================================================================

#[tokio::test]
#[ignore]
async fn test_dm_remembers_context_across_turns() {
    setup();
    if !has_api_key() {
        eprintln!("Skipping test: ANTHROPIC_API_KEY not set");
        return;
    }

    let character = create_sample_wizard("Sage");
    let mut world = GameWorld::new("Context Test", character);
    let mut dm = DungeonMaster::from_env().expect("Failed to create DM");

    dm = dm.with_config(DmConfig {
        model: Some("claude-sonnet-4-20250514".to_string()),
        max_tokens: 1024,
        temperature: Some(0.7),
        custom_system_prompt: None,
    });

    // First turn - establish context
    let response1 = dm
        .process_input(
            "I introduce myself to the innkeeper as Sage the Wise",
            &mut world,
        )
        .await
        .expect("DM should respond");

    println!("Turn 1: {}", response1.narrative);

    // Second turn - reference previous context
    let response2 = dm
        .process_input("I ask the innkeeper if he's heard any rumors", &mut world)
        .await
        .expect("DM should respond");

    println!("Turn 2: {}", response2.narrative);

    // Third turn - use a spell
    let response3 = dm
        .process_input(
            "I thank the innkeeper and cast Light on my staff",
            &mut world,
        )
        .await
        .expect("DM should respond");

    println!("Turn 3: {}", response3.narrative);

    // All responses should be coherent
    assert!(!response1.narrative.is_empty());
    assert!(!response2.narrative.is_empty());
    assert!(!response3.narrative.is_empty());

    println!("SUCCESS: Multi-turn conversation completed!");
}
