//! System prompts for the D&D DM agent

use crate::dnd::game::character::Character;
use crate::dnd::game::state::{GameMode, GameWorld};

/// Build the main DM system prompt
pub fn build_dm_system_prompt(game: &GameWorld) -> String {
    let character = &game.player_character;
    let location = &game.current_location;

    format!(
        r#"You are an expert D&D 5th Edition Dungeon Master running a solo adventure for an experienced player.

## Your Role
1. Narrate the world vividly and immersively
2. Control all NPCs with distinct personalities
3. Adjudicate rules fairly and accurately
4. Create challenging but fair encounters
5. React dynamically to player choices

## Current Game State

### Character
Name: {char_name}
Class: {char_class}
Level: {level}
HP: {hp_current}/{hp_max}
AC: {ac}

### Location
{location_name}
{location_desc}

### Time
{time}

## Rules Enforcement

ALWAYS use the appropriate tools for:
- Dice rolls (attacks, saves, checks, damage)
- HP changes (damage, healing)
- Condition application/removal
- Combat actions

NEVER skip dice rolls. The dice tell the story.

## Response Style
- Be descriptive but concise
- Use present tense for narration
- Give the player clear options when appropriate
- Ask clarifying questions if the player's intent is unclear
- Balance challenge with fun

## Combat Protocol
When combat starts:
1. Roll initiative for all combatants
2. Describe the battlefield
3. Track turn order strictly
4. Announce whose turn it is
5. Apply all rules effects correctly

Remember: You are creating an engaging, interactive story where the player's choices matter!
"#,
        char_name = character.name,
        char_class = character
            .primary_class()
            .map(|c| c.class.to_string())
            .unwrap_or_else(|| "Adventurer".to_string()),
        level = character.level,
        hp_current = character.hit_points.current,
        hp_max = character.hit_points.maximum,
        ac = character.current_ac(),
        location_name = location.name,
        location_desc = location.description,
        time = game.game_time.to_string_detailed(),
    )
}

/// Build a combat-specific system prompt
pub fn build_combat_prompt(game: &GameWorld) -> String {
    let base = build_dm_system_prompt(game);

    let combat_context = if let Some(ref combat) = game.combat {
        let mut ctx = String::from("\n## Active Combat\n\n");
        ctx.push_str(&format!("Round: {}\n", combat.round));
        ctx.push_str("Initiative Order:\n");

        for (i, entry) in combat.initiative_order.iter().enumerate() {
            let marker = if i == combat.turn_index { ">>>" } else { "   " };
            ctx.push_str(&format!(
                "{} {}. {} (Init: {})\n",
                marker,
                i + 1,
                entry.name,
                entry.initiative_total
            ));
        }

        if let Some(current) = combat.current_combatant() {
            ctx.push_str(&format!("\nCurrent Turn: {}\n", current.name));
        }

        ctx
    } else {
        String::new()
    };

    format!("{}\n{}", base, combat_context)
}

/// Build a dialogue-specific system prompt for NPC interaction
pub fn build_dialogue_prompt(game: &GameWorld, npc_name: &str, npc_personality: &str) -> String {
    format!(
        r#"You are roleplaying as {npc_name}, an NPC in a D&D campaign.

## Your Character
{npc_personality}

## Player Character
{char_name} - Level {level} {char_class}

## Guidelines
- Stay in character as {npc_name}
- React naturally to the player's words
- Have your own goals and motivations
- Don't reveal information you wouldn't know
- Use appropriate dialect/speech patterns for your character

Respond only as {npc_name}. Do not break character.
"#,
        npc_name = npc_name,
        npc_personality = npc_personality,
        char_name = game.player_character.name,
        level = game.player_character.level,
        char_class = game
            .player_character
            .primary_class()
            .map(|c| c.class.to_string())
            .unwrap_or_else(|| "Adventurer".to_string()),
    )
}

/// Build prompt for encounter generation
pub fn build_encounter_prompt(party_level: u8, difficulty: &str, environment: &str) -> String {
    format!(
        r#"Generate a balanced D&D 5e encounter.

## Parameters
- Party Level: {party_level}
- Difficulty: {difficulty}
- Environment: {environment}

## Requirements
1. Select appropriate monsters for the environment
2. Calculate total XP budget based on difficulty thresholds
3. Design interesting terrain features
4. Include tactical elements (cover, hazards, objectives)
5. Suggest possible enemy tactics

## XP Thresholds (per character, level {party_level})
- Easy: {easy_xp} XP
- Medium: {medium_xp} XP
- Hard: {hard_xp} XP
- Deadly: {deadly_xp} XP

Provide a complete encounter with monster stats, positioning, and tactics.
"#,
        party_level = party_level,
        difficulty = difficulty,
        environment = environment,
        easy_xp = xp_threshold(party_level, "easy"),
        medium_xp = xp_threshold(party_level, "medium"),
        hard_xp = xp_threshold(party_level, "hard"),
        deadly_xp = xp_threshold(party_level, "deadly"),
    )
}

/// Get XP threshold for difficulty and level
fn xp_threshold(level: u8, difficulty: &str) -> u32 {
    // Simplified XP thresholds from DMG
    let thresholds: [(u32, u32, u32, u32); 20] = [
        (25, 50, 75, 100),       // Level 1
        (50, 100, 150, 200),     // Level 2
        (75, 150, 225, 400),     // Level 3
        (125, 250, 375, 500),    // Level 4
        (250, 500, 750, 1100),   // Level 5
        (300, 600, 900, 1400),   // Level 6
        (350, 750, 1100, 1700),  // Level 7
        (450, 900, 1400, 2100),  // Level 8
        (550, 1100, 1600, 2400), // Level 9
        (600, 1200, 1900, 2800), // Level 10
        (800, 1600, 2400, 3600), // Level 11
        (1000, 2000, 3000, 4500), // Level 12
        (1100, 2200, 3400, 5100), // Level 13
        (1250, 2500, 3800, 5700), // Level 14
        (1400, 2800, 4300, 6400), // Level 15
        (1600, 3200, 4800, 7200), // Level 16
        (2000, 3900, 5900, 8800), // Level 17
        (2100, 4200, 6300, 9500), // Level 18
        (2400, 4900, 7300, 10900), // Level 19
        (2800, 5700, 8500, 12700), // Level 20
    ];

    let level_idx = (level.saturating_sub(1) as usize).min(19);
    let (easy, medium, hard, deadly) = thresholds[level_idx];

    match difficulty.to_lowercase().as_str() {
        "easy" => easy,
        "medium" => medium,
        "hard" => hard,
        "deadly" => deadly,
        _ => medium,
    }
}
