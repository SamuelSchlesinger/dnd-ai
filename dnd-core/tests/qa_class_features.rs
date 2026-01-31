//! QA tests for class-specific features.
//!
//! These tests verify that class abilities work correctly through the headless API.
//! Run with: `cargo test -p dnd-core --test qa_class_features -- --ignored --nocapture`

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
// FIGHTER CLASS FEATURES
// =============================================================================

#[tokio::test]
#[ignore]
async fn test_fighter_second_wind() {
    setup();
    if !has_api_key() {
        eprintln!("Skipping test: ANTHROPIC_API_KEY not set");
        return;
    }

    // Create a fighter using HeadlessConfig
    let config = HeadlessConfig::custom(
        "Roland the Fighter",
        RaceType::Human,
        CharacterClass::Fighter,
        Background::Soldier,
    );

    let mut game = HeadlessGame::new(config)
        .await
        .expect("Failed to create game");

    println!("=== FIGHTER SECOND WIND TEST ===");
    println!("Character: {}", game.player_name());
    println!("Class: {:?}", game.player_class());
    println!("Initial HP: {}/{}", game.current_hp(), game.max_hp());

    // First, we need to take some damage to make Second Wind relevant
    // Ask the DM to set up a scenario where the fighter takes damage
    let response = game
        .send("A goblin ambushes me and hits me for 5 damage before I can react")
        .await
        .expect("DM should respond");

    println!("\nAfter ambush:");
    println!("  Narrative: {}", response.narrative);
    println!("  HP: {}/{}", game.current_hp(), game.max_hp());

    // Now use Second Wind
    let initial_hp = game.current_hp();
    let response = game
        .send("I use my Second Wind ability to heal myself")
        .await
        .expect("DM should respond");

    println!("\nAfter Second Wind:");
    println!("  Narrative: {}", response.narrative);
    println!("  HP: {}/{}", game.current_hp(), game.max_hp());

    let final_hp = game.current_hp();

    // Check if Second Wind was used (character should have gained HP)
    if final_hp > initial_hp {
        println!(
            "\nSUCCESS: Second Wind healed {} HP!",
            final_hp - initial_hp
        );
    } else if initial_hp == game.max_hp() {
        println!("\nNOTE: Character was at full HP, healing had no effect");
    } else {
        println!("\nPOTENTIAL BUG: Second Wind did not heal the character");
        println!("  Expected HP > {}, got {}", initial_hp, final_hp);
    }

    // Verify the response mentions Second Wind
    let mentions_second_wind = response.narrative.to_lowercase().contains("second wind")
        || response
            .narrative
            .to_lowercase()
            .contains("surge of energy")
        || response
            .narrative
            .to_lowercase()
            .contains("catch your breath");

    if !mentions_second_wind {
        println!("\nNOTE: DM response doesn't explicitly mention Second Wind");
    }
}

#[tokio::test]
#[ignore]
async fn test_fighter_action_surge() {
    setup();
    if !has_api_key() {
        eprintln!("Skipping test: ANTHROPIC_API_KEY not set");
        return;
    }

    let config = HeadlessConfig::custom(
        "Ser Marcus",
        RaceType::Human,
        CharacterClass::Fighter,
        Background::Soldier,
    )
    .with_starting_location("The Training Grounds");

    let mut game = HeadlessGame::new(config)
        .await
        .expect("Failed to create game");

    println!("=== FIGHTER ACTION SURGE TEST ===");
    println!("Character: {}", game.player_name());

    // Start combat to make Action Surge relevant
    let response = game
        .send("I see a training dummy and prepare to demonstrate my fighting skills. I attack it!")
        .await
        .expect("DM should respond");

    println!("\nInitial attack:");
    println!("  Narrative: {}", response.narrative);

    // Use Action Surge for an extra action
    let response = game
        .send("I use Action Surge to make additional attacks on the training dummy!")
        .await
        .expect("DM should respond");

    println!("\nAfter Action Surge:");
    println!("  Narrative: {}", response.narrative);

    // Check if Action Surge was acknowledged
    let mentions_action_surge = response.narrative.to_lowercase().contains("action surge")
        || response.narrative.to_lowercase().contains("burst of speed")
        || response.narrative.to_lowercase().contains("extra action")
        || response
            .narrative
            .to_lowercase()
            .contains("additional action");

    if mentions_action_surge {
        println!("\nSUCCESS: Action Surge was acknowledged by the DM!");
    } else {
        println!("\nPOTENTIAL ISSUE: DM response may not properly acknowledge Action Surge");
    }

    // Check the class_resources to see if action_surge_used was tracked
    let session = game.session();
    let action_surge_used = session
        .world()
        .player_character
        .class_resources
        .action_surge_used;
    println!("  Action Surge used (tracked): {}", action_surge_used);

    if action_surge_used {
        println!("\nSUCCESS: Action Surge was properly tracked as used!");
    } else {
        println!("\nPOTENTIAL BUG: Action Surge tracking not updated in class_resources");
    }
}

// =============================================================================
// BARBARIAN CLASS FEATURES
// =============================================================================

#[tokio::test]
#[ignore]
async fn test_barbarian_rage() {
    setup();
    if !has_api_key() {
        eprintln!("Skipping test: ANTHROPIC_API_KEY not set");
        return;
    }

    let config = HeadlessConfig::custom(
        "Grog the Mighty",
        RaceType::HalfOrc,
        CharacterClass::Barbarian,
        Background::Outlander,
    )
    .with_starting_location("A dark forest clearing");

    let mut game = HeadlessGame::new(config)
        .await
        .expect("Failed to create game");

    println!("=== BARBARIAN RAGE TEST ===");
    println!("Character: {}", game.player_name());
    println!("Class: {:?}", game.player_class());

    // Check initial rage status
    let session = game.session();
    let initial_rage_active = session.world().player_character.class_resources.rage_active;
    println!("Initial rage active: {}", initial_rage_active);

    // Encounter an enemy and enter rage
    let response = game
        .send("A wolf pack surrounds me! I let out a primal roar and enter my barbarian RAGE!")
        .await
        .expect("DM should respond");

    println!("\nAfter entering rage:");
    println!("  Narrative: {}", response.narrative);

    // Check if rage was activated
    let session = game.session();
    let final_rage_active = session.world().player_character.class_resources.rage_active;
    println!("  Rage active (tracked): {}", final_rage_active);

    if final_rage_active && !initial_rage_active {
        println!("\nSUCCESS: Barbarian rage was properly activated!");
    } else if final_rage_active {
        println!("\nNOTE: Rage was already active");
    } else {
        println!("\nPOTENTIAL BUG: Rage tracking not updated - rage_active should be true");
    }

    // Check if DM mentions rage benefits
    let mentions_rage_benefits = response.narrative.to_lowercase().contains("rage")
        || response.narrative.to_lowercase().contains("fury")
        || response.narrative.to_lowercase().contains("strength")
        || response.narrative.to_lowercase().contains("damage bonus");

    if mentions_rage_benefits {
        println!("SUCCESS: DM acknowledged the rage!");
    } else {
        println!("NOTE: DM response may not explicitly mention rage");
    }

    // Try to attack while raging to verify rage damage bonus is described
    let response = game
        .send("I attack the nearest wolf with my greataxe while raging!")
        .await
        .expect("DM should respond");

    println!("\nAttack while raging:");
    println!("  Narrative: {}", response.narrative);

    // Rage should still be active
    let session = game.session();
    let still_raging = session.world().player_character.class_resources.rage_active;
    println!("  Still raging: {}", still_raging);
}

// =============================================================================
// WIZARD/SORCERER SPELLCASTING
// =============================================================================

#[tokio::test]
#[ignore]
async fn test_wizard_spellcasting() {
    setup();
    if !has_api_key() {
        eprintln!("Skipping test: ANTHROPIC_API_KEY not set");
        return;
    }

    let config = HeadlessConfig::custom(
        "Merlin the Wise",
        RaceType::Elf,
        CharacterClass::Wizard,
        Background::Sage,
    )
    .with_starting_location("An ancient library");

    let mut game = HeadlessGame::new(config)
        .await
        .expect("Failed to create game");

    println!("=== WIZARD SPELLCASTING TEST ===");
    println!("Character: {}", game.player_name());
    println!("Class: {:?}", game.player_class());

    // Check initial spell slots
    let session = game.session();
    let initial_slots = session
        .world()
        .player_character
        .spellcasting
        .as_ref()
        .map(|s| (s.spell_slots.slots[0].total, s.spell_slots.slots[0].used));

    println!("Initial 1st level slots: {:?}", initial_slots);

    // Cast a cantrip (should not use spell slots)
    let response = game
        .send("I cast the Light cantrip on my staff to illuminate the dark corner")
        .await
        .expect("DM should respond");

    println!("\nAfter casting Light cantrip:");
    println!("  Narrative: {}", response.narrative);

    let session = game.session();
    let slots_after_cantrip = session
        .world()
        .player_character
        .spellcasting
        .as_ref()
        .map(|s| (s.spell_slots.slots[0].total, s.spell_slots.slots[0].used));

    println!("  1st level slots: {:?}", slots_after_cantrip);

    if slots_after_cantrip == initial_slots {
        println!("SUCCESS: Cantrip did not consume spell slots!");
    } else {
        println!("POTENTIAL BUG: Cantrip consumed a spell slot (should be free)");
    }

    // Cast a leveled spell (should use a spell slot)
    let response = game
        .send("I cast Magic Missile at a nearby training target")
        .await
        .expect("DM should respond");

    println!("\nAfter casting Magic Missile:");
    println!("  Narrative: {}", response.narrative);

    let session = game.session();
    let slots_after_spell = session
        .world()
        .player_character
        .spellcasting
        .as_ref()
        .map(|s| (s.spell_slots.slots[0].total, s.spell_slots.slots[0].used));

    println!("  1st level slots: {:?}", slots_after_spell);

    // Check if a spell slot was consumed
    if let (Some((_, used_before)), Some((_, used_after))) =
        (slots_after_cantrip, slots_after_spell)
    {
        if used_after > used_before {
            println!("SUCCESS: Leveled spell consumed a spell slot!");
        } else {
            println!("POTENTIAL BUG: Leveled spell did not consume a spell slot");
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_sorcerer_metamagic() {
    setup();
    if !has_api_key() {
        eprintln!("Skipping test: ANTHROPIC_API_KEY not set");
        return;
    }

    let config = HeadlessConfig::custom(
        "Seraphina",
        RaceType::Tiefling,
        CharacterClass::Sorcerer,
        Background::Noble,
    )
    .with_starting_location("A noble's estate");

    let mut game = HeadlessGame::new(config)
        .await
        .expect("Failed to create game");

    println!("=== SORCERER METAMAGIC TEST ===");
    println!("Character: {}", game.player_name());

    // Check initial sorcery points
    let session = game.session();
    let initial_sorcery_points = session
        .world()
        .player_character
        .class_resources
        .sorcery_points;
    println!("Initial sorcery points: {}", initial_sorcery_points);

    // Try to use Quickened Spell metamagic
    let response = game
        .send("I use Quickened Spell metamagic to cast Fire Bolt as a bonus action!")
        .await
        .expect("DM should respond");

    println!("\nAfter using Quickened Spell:");
    println!("  Narrative: {}", response.narrative);

    let session = game.session();
    let final_sorcery_points = session
        .world()
        .player_character
        .class_resources
        .sorcery_points;
    println!("  Sorcery points: {}", final_sorcery_points);

    // Quickened Spell costs 2 sorcery points
    if initial_sorcery_points > 0 && final_sorcery_points < initial_sorcery_points {
        let points_spent = initial_sorcery_points - final_sorcery_points;
        println!("SUCCESS: {} sorcery point(s) were spent!", points_spent);
    } else if initial_sorcery_points == 0 {
        println!("NOTE: Sorcerer had 0 sorcery points - may need to check level/initialization");
    } else {
        println!("POTENTIAL BUG: Sorcery points not consumed for metamagic");
    }
}

// =============================================================================
// CLERIC CLASS FEATURES
// =============================================================================

#[tokio::test]
#[ignore]
async fn test_cleric_channel_divinity() {
    setup();
    if !has_api_key() {
        eprintln!("Skipping test: ANTHROPIC_API_KEY not set");
        return;
    }

    let config = HeadlessConfig::custom(
        "Brother Marcus",
        RaceType::Human,
        CharacterClass::Cleric,
        Background::Acolyte,
    )
    .with_starting_location("A haunted cemetery");

    let mut game = HeadlessGame::new(config)
        .await
        .expect("Failed to create game");

    println!("=== CLERIC CHANNEL DIVINITY TEST ===");
    println!("Character: {}", game.player_name());

    // Check initial channel divinity state
    let session = game.session();
    let initial_cd_used = session
        .world()
        .player_character
        .class_resources
        .channel_divinity_used;
    println!("Initial channel_divinity_used: {}", initial_cd_used);

    // Use Turn Undead
    let response = game
        .send("I raise my holy symbol and use Channel Divinity: Turn Undead against the approaching skeletons!")
        .await
        .expect("DM should respond");

    println!("\nAfter using Turn Undead:");
    println!("  Narrative: {}", response.narrative);

    // Check if DM describes the Turn Undead effect
    let mentions_turn_undead = response.narrative.to_lowercase().contains("turn")
        || response.narrative.to_lowercase().contains("flee")
        || response.narrative.to_lowercase().contains("undead")
        || response.narrative.to_lowercase().contains("holy");

    if mentions_turn_undead {
        println!("SUCCESS: DM described the Turn Undead effect!");
    } else {
        println!("NOTE: DM may not have explicitly described Turn Undead mechanics");
    }

    // Check if the channel_divinity_used field was updated
    let session = game.session();
    let final_cd_used = session
        .world()
        .player_character
        .class_resources
        .channel_divinity_used;
    println!(
        "  channel_divinity_used after Turn Undead: {}",
        final_cd_used
    );

    // Note: The DM tool should set channel_divinity_used = true when used
    // Currently the field exists but the actual tracking happens via the Feature system
}

#[tokio::test]
#[ignore]
async fn test_cleric_healing_spell() {
    setup();
    if !has_api_key() {
        eprintln!("Skipping test: ANTHROPIC_API_KEY not set");
        return;
    }

    let config = HeadlessConfig::custom(
        "Sister Elara",
        RaceType::Human,
        CharacterClass::Cleric,
        Background::Acolyte,
    );

    let mut game = HeadlessGame::new(config)
        .await
        .expect("Failed to create game");

    println!("=== CLERIC HEALING SPELL TEST ===");
    println!("Character: {}", game.player_name());
    println!("Initial HP: {}/{}", game.current_hp(), game.max_hp());

    // Take some damage first
    let _response = game
        .send("While exploring the ruins, I accidentally trigger a trap that deals 8 damage to me")
        .await
        .expect("DM should respond");

    println!("\nAfter trap damage:");
    println!("  HP: {}/{}", game.current_hp(), game.max_hp());

    let hp_after_damage = game.current_hp();

    // Cast Cure Wounds on self
    let response = game
        .send("I cast Cure Wounds on myself using a 1st level spell slot")
        .await
        .expect("DM should respond");

    println!("\nAfter Cure Wounds:");
    println!("  Narrative: {}", response.narrative);
    println!("  HP: {}/{}", game.current_hp(), game.max_hp());

    let hp_after_heal = game.current_hp();

    if hp_after_heal > hp_after_damage {
        let amount_healed = hp_after_heal - hp_after_damage;
        println!("SUCCESS: Cure Wounds healed {} HP!", amount_healed);

        // Cure Wounds heals 1d8 + spellcasting modifier (minimum 1)
        // For a typical cleric, this should be 1-13 range
        if amount_healed >= 1 && amount_healed <= 15 {
            println!(
                "  Healing amount {} is in expected range for Cure Wounds",
                amount_healed
            );
        } else {
            println!(
                "  WARNING: Healing amount {} seems unusual for 1st level Cure Wounds",
                amount_healed
            );
        }
    } else if hp_after_heal == game.max_hp() {
        println!("SUCCESS: Character healed to full HP!");
    } else {
        println!("POTENTIAL BUG: Cure Wounds did not heal the character");
    }

    // Check spell slot consumption
    let session = game.session();
    if let Some(ref spellcasting) = session.world().player_character.spellcasting {
        let slots_used = spellcasting.spell_slots.slots[0].used;
        println!("  1st level slots used: {}", slots_used);
        if slots_used > 0 {
            println!("SUCCESS: Spell slot was consumed!");
        }
    }
}

// =============================================================================
// ROGUE CLASS FEATURES
// =============================================================================

#[tokio::test]
#[ignore]
async fn test_rogue_sneak_attack() {
    setup();
    if !has_api_key() {
        eprintln!("Skipping test: ANTHROPIC_API_KEY not set");
        return;
    }

    let config = HeadlessConfig::custom(
        "Shadow",
        RaceType::Halfling,
        CharacterClass::Rogue,
        Background::Criminal,
    )
    .with_starting_location("A dark alley");

    let mut game = HeadlessGame::new(config)
        .await
        .expect("Failed to create game");

    println!("=== ROGUE SNEAK ATTACK TEST ===");
    println!("Character: {}", game.player_name());
    println!("Class: {:?}", game.player_class());

    // Set up a sneak attack scenario
    let response = game
        .send("I hide in the shadows and wait for an unsuspecting guard to pass by")
        .await
        .expect("DM should respond");

    println!("\nSetting up ambush:");
    println!("  Narrative: {}", response.narrative);

    // Attempt sneak attack
    let response = game
        .send("I strike the guard from hiding with my dagger, using my Sneak Attack!")
        .await
        .expect("DM should respond");

    println!("\nSneak Attack attempt:");
    println!("  Narrative: {}", response.narrative);

    // Check if sneak attack was acknowledged
    let mentions_sneak_attack = response.narrative.to_lowercase().contains("sneak attack")
        || response.narrative.to_lowercase().contains("extra damage")
        || response.narrative.to_lowercase().contains("precise strike")
        || response.narrative.to_lowercase().contains("d6")
        || response.narrative.to_lowercase().contains("vital spot");

    if mentions_sneak_attack {
        println!("\nSUCCESS: DM acknowledged Sneak Attack!");
    } else {
        println!("\nNOTE: DM may not have explicitly mentioned Sneak Attack mechanics");
        println!("  This could be a narrative style choice or a bug");
    }

    // Check for damage description - sneak attack adds dice
    let mentions_damage = response.narrative.to_lowercase().contains("damage")
        || response.narrative.to_lowercase().contains("wound")
        || response.narrative.to_lowercase().contains("hit")
        || response.narrative.to_lowercase().contains("strike");

    if mentions_damage {
        println!("SUCCESS: Attack/damage was described!");
    }
}

#[tokio::test]
#[ignore]
async fn test_rogue_cunning_action() {
    setup();
    if !has_api_key() {
        eprintln!("Skipping test: ANTHROPIC_API_KEY not set");
        return;
    }

    let config = HeadlessConfig::custom(
        "Nimble",
        RaceType::Elf,
        CharacterClass::Rogue,
        Background::Urchin,
    )
    .with_starting_location("A crowded marketplace");

    let mut game = HeadlessGame::new(config)
        .await
        .expect("Failed to create game");

    println!("=== ROGUE CUNNING ACTION TEST ===");
    println!("Character: {}", game.player_name());

    // Use Cunning Action to Dash
    let response = game
        .send("Guards are chasing me! I use my Cunning Action to Dash as a bonus action and run through the crowd!")
        .await
        .expect("DM should respond");

    println!("\nUsing Cunning Action (Dash):");
    println!("  Narrative: {}", response.narrative);

    let mentions_cunning_action = response.narrative.to_lowercase().contains("cunning")
        || response.narrative.to_lowercase().contains("dash")
        || response.narrative.to_lowercase().contains("quick")
        || response.narrative.to_lowercase().contains("nimble")
        || response.narrative.to_lowercase().contains("agile");

    if mentions_cunning_action {
        println!("\nSUCCESS: Cunning Action was acknowledged!");
    } else {
        println!("\nNOTE: DM may not have explicitly mentioned Cunning Action");
    }

    // Use Cunning Action to Hide
    let response = game
        .send("I duck into an alley and use Cunning Action to Hide as a bonus action!")
        .await
        .expect("DM should respond");

    println!("\nUsing Cunning Action (Hide):");
    println!("  Narrative: {}", response.narrative);

    let mentions_hide = response.narrative.to_lowercase().contains("hide")
        || response.narrative.to_lowercase().contains("shadow")
        || response.narrative.to_lowercase().contains("concealed")
        || response.narrative.to_lowercase().contains("stealth");

    if mentions_hide {
        println!("SUCCESS: Hide action was acknowledged!");
    }
}

// =============================================================================
// CROSS-CLASS TESTS
// =============================================================================

#[tokio::test]
#[ignore]
async fn test_multiclass_like_scenarios() {
    setup();
    if !has_api_key() {
        eprintln!("Skipping test: ANTHROPIC_API_KEY not set");
        return;
    }

    // Test that the DM correctly handles a character trying to use abilities
    // they don't have
    let config = HeadlessConfig::custom(
        "Confused Warrior",
        RaceType::Human,
        CharacterClass::Fighter,
        Background::Soldier,
    );

    let mut game = HeadlessGame::new(config)
        .await
        .expect("Failed to create game");

    println!("=== CLASS RESTRICTION TEST ===");
    println!("Character: {} (Fighter)", game.player_name());

    // Try to use a Barbarian ability as a Fighter
    let response = game
        .send("I enter a barbarian rage!")
        .await
        .expect("DM should respond");

    println!("\nFighter trying to use Rage:");
    println!("  Narrative: {}", response.narrative);

    // The DM should either refuse this or explain the character can't rage
    let session = game.session();
    let rage_active = session.world().player_character.class_resources.rage_active;

    if !rage_active {
        println!("SUCCESS: Fighter cannot enter rage (correctly blocked)");
    } else {
        println!("POTENTIAL BUG: Fighter was able to enter rage - class restriction not enforced");
    }

    // Try to cast a spell as a Fighter (no spellcasting)
    let response = game
        .send("I cast Fireball at my enemies!")
        .await
        .expect("DM should respond");

    println!("\nFighter trying to cast Fireball:");
    println!("  Narrative: {}", response.narrative);

    // Check if the DM explains the fighter can't cast spells
    let correctly_handled = response.narrative.to_lowercase().contains("can't cast")
        || response.narrative.to_lowercase().contains("cannot cast")
        || response
            .narrative
            .to_lowercase()
            .contains("not a spellcaster")
        || response.narrative.to_lowercase().contains("no magic")
        || response.narrative.to_lowercase().contains("don't have")
        || !response
            .narrative
            .to_lowercase()
            .contains("fireball explodes");

    if correctly_handled {
        println!("SUCCESS: DM correctly handled non-spellcaster trying to cast");
    } else {
        println!("POTENTIAL BUG: Fighter may have been allowed to cast Fireball");
    }
}

#[tokio::test]
#[ignore]
async fn test_resource_recovery_on_rest() {
    setup();
    if !has_api_key() {
        eprintln!("Skipping test: ANTHROPIC_API_KEY not set");
        return;
    }

    let config = HeadlessConfig::custom(
        "Tired Fighter",
        RaceType::Human,
        CharacterClass::Fighter,
        Background::Soldier,
    );

    let mut game = HeadlessGame::new(config)
        .await
        .expect("Failed to create game");

    println!("=== RESOURCE RECOVERY TEST ===");
    println!("Character: {}", game.player_name());

    // Use Second Wind
    let _response = game
        .send("I take some damage from a fall, then use Second Wind to recover")
        .await
        .expect("DM should respond");

    println!("\nUsed Second Wind:");
    let session = game.session();
    let sw_used_before_rest = session
        .world()
        .player_character
        .class_resources
        .second_wind_used;
    println!("  Second Wind used: {}", sw_used_before_rest);

    // Take a short rest (should recover Second Wind)
    let response = game
        .send("I find a safe spot and take a short rest for an hour")
        .await
        .expect("DM should respond");

    println!("\nAfter short rest:");
    println!("  Narrative: {}", response.narrative);

    let session = game.session();
    let sw_used_after_rest = session
        .world()
        .player_character
        .class_resources
        .second_wind_used;
    println!("  Second Wind used: {}", sw_used_after_rest);

    if sw_used_before_rest && !sw_used_after_rest {
        println!("SUCCESS: Second Wind was recovered on short rest!");
    } else if !sw_used_before_rest {
        println!("NOTE: Second Wind wasn't used before rest");
    } else {
        println!("POTENTIAL BUG: Second Wind was not recovered on short rest");
    }
}
