//! World state tools: rest, location, story memory, spells, and progression.

use chronicler_llm::Tool;
use serde_json::json;

/// Take a short rest.
pub fn short_rest() -> Tool {
    Tool {
        name: "short_rest".to_string(),
        description: "Take a short rest (1 hour). Recover some abilities.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {},
            "required": []
        }),
    }
}

/// Take a long rest.
pub fn long_rest() -> Tool {
    Tool {
        name: "long_rest".to_string(),
        description: "Take a long rest (8 hours). Fully recover HP and abilities.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {},
            "required": []
        }),
    }
}

/// Change the current location.
pub fn change_location() -> Tool {
    Tool {
        name: "change_location".to_string(),
        description: "Change the current location when the player travels somewhere new. Use this whenever the player moves to a different area, enters a building, or travels to a new place.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "new_location": {
                    "type": "string",
                    "description": "Name of the new location (e.g., 'The Dark Forest', 'Town Square', 'Goblin Cave')"
                },
                "location_type": {
                    "type": "string",
                    "enum": ["city", "town", "village", "dungeon", "wilderness", "building", "room", "other"],
                    "description": "Type of location"
                },
                "description": {
                    "type": "string",
                    "description": "Brief description of the location for future reference"
                }
            },
            "required": ["new_location"]
        }),
    }
}

/// Record an important story fact.
pub fn remember_fact() -> Tool {
    Tool {
        name: "remember_fact".to_string(),
        description: "Record an important story fact for future reference. Use this when introducing NPCs, establishing locations, recording player decisions, or revealing plot points. Facts are indexed and used to maintain narrative consistency.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "subject_name": {
                    "type": "string",
                    "description": "Name of the entity this fact is about (NPC name, location name, item name, etc.)"
                },
                "subject_type": {
                    "type": "string",
                    "enum": ["npc", "location", "item", "quest", "organization", "event", "creature"],
                    "description": "Type of entity"
                },
                "fact": {
                    "type": "string",
                    "description": "The fact to record in natural language"
                },
                "category": {
                    "type": "string",
                    "enum": ["appearance", "personality", "event", "relationship", "backstory", "motivation", "capability", "location", "possession", "status", "secret"],
                    "description": "Category of the fact"
                },
                "related_entities": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Names of other entities mentioned in this fact (optional)"
                },
                "importance": {
                    "type": "number",
                    "minimum": 0.1,
                    "maximum": 1.0,
                    "description": "How important this fact is (0.1-1.0, default 0.7)"
                }
            },
            "required": ["subject_name", "subject_type", "fact", "category"]
        }),
    }
}

/// Register a future consequence.
pub fn register_consequence() -> Tool {
    Tool {
        name: "register_consequence".to_string(),
        description: "Register a future consequence based on player actions. Use this when something the player does should have future ramifications - like making an enemy, breaking a law, or triggering a curse. The consequence will be surfaced when relevant conditions arise.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "trigger_description": {
                    "type": "string",
                    "description": "Natural language description of when this consequence should trigger (e.g., 'Player enters Riverside village', 'Player encounters Baron Aldric', 'Player tries to sleep')"
                },
                "consequence_description": {
                    "type": "string",
                    "description": "Natural language description of what happens when triggered (e.g., 'Town guards attempt to arrest the player for crimes against the baron', 'The curse drains 1d6 HP')"
                },
                "severity": {
                    "type": "string",
                    "enum": ["minor", "moderate", "major", "critical"],
                    "description": "How severe this consequence is. Minor=flavor/inconvenience, Moderate=meaningful impact, Major=significant story impact, Critical=life-threatening"
                },
                "related_entities": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Names of entities involved (NPCs, locations, organizations)"
                },
                "importance": {
                    "type": "number",
                    "minimum": 0.1,
                    "maximum": 1.0,
                    "description": "How important this consequence is for relevance ranking (0.1-1.0, default based on severity)"
                },
                "expires_in_turns": {
                    "type": "integer",
                    "minimum": 1,
                    "description": "Number of turns until this consequence expires (omit for permanent consequences)"
                }
            },
            "required": ["trigger_description", "consequence_description", "severity"]
        }),
    }
}

/// Cast a spell.
pub fn cast_spell() -> Tool {
    Tool {
        name: "cast_spell".to_string(),
        description: "Cast a spell. Handles spell slot consumption, attack rolls, saving throws, and damage/healing. For cantrips (level 0), no spell slot is consumed. For leveled spells, a spell slot of the appropriate level or higher must be available.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "spell_name": {
                    "type": "string",
                    "description": "Name of the spell to cast (e.g., 'Fireball', 'Cure Wounds', 'Fire Bolt')"
                },
                "slot_level": {
                    "type": "integer",
                    "minimum": 0,
                    "maximum": 9,
                    "description": "Spell slot level to use. Use 0 for cantrips. Can upcast by using a higher slot than the spell's base level."
                },
                "targets": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Names of targets for the spell (for targeted spells)"
                }
            },
            "required": ["spell_name"]
        }),
    }
}

/// Award experience points.
pub fn award_experience() -> Tool {
    Tool {
        name: "award_experience".to_string(),
        description: "Award experience points (XP) to the player character. Use this after combat victories, completing quests, clever problem-solving, or significant story achievements. Standard XP awards: trivial encounter (25-50), easy encounter (50-100), medium encounter (100-200), hard encounter (200-400), deadly encounter (400+), quest completion (varies by significance).".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "amount": {
                    "type": "integer",
                    "description": "Amount of XP to award"
                },
                "reason": {
                    "type": "string",
                    "description": "Why the XP is being awarded (e.g., 'defeated goblin ambush', 'completed quest')"
                }
            },
            "required": ["amount", "reason"]
        }),
    }
}

/// Temporarily modify an ability score.
pub fn modify_ability_score() -> Tool {
    Tool {
        name: "modify_ability_score".to_string(),
        description: "Temporarily modify an ability score. Use this when a spell, poison, curse, magic item, or other effect grants a bonus or penalty to an ability score. Examples: Ray of Enfeeblement reducing Strength, a Belt of Giant Strength boosting Strength, poison reducing Constitution, an Intellect Devourer draining Intelligence.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "ability": {
                    "type": "string",
                    "enum": ["strength", "dexterity", "constitution", "intelligence", "wisdom", "charisma"],
                    "description": "The ability score to modify"
                },
                "modifier": {
                    "type": "integer",
                    "description": "Amount to add or subtract from the ability score (can be negative for penalties)"
                },
                "source": {
                    "type": "string",
                    "description": "What is causing the modification (e.g., 'poison', 'Ray of Enfeeblement spell', 'Belt of Giant Strength', 'curse')"
                },
                "duration": {
                    "type": "string",
                    "description": "How long the modification lasts (e.g., 'until long rest', '1 hour', '1 minute', 'permanent', 'until dispelled')"
                }
            },
            "required": ["ability", "modifier", "source"]
        }),
    }
}

/// Advance game time without resting.
pub fn advance_time() -> Tool {
    Tool {
        name: "advance_time".to_string(),
        description: "Advance game time without taking a rest. Use this when time passes during travel, waiting, searching, or other activities that don't qualify as a short or long rest. This tracks duration-based effects and can trigger time-sensitive events. At least one of 'minutes' or 'hours' must be provided.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "minutes": {
                    "type": "integer",
                    "description": "Number of minutes to advance"
                },
                "hours": {
                    "type": "integer",
                    "description": "Number of hours to advance"
                },
                "description": {
                    "type": "string",
                    "description": "What happens during this time (e.g., 'traveling through the forest', 'waiting for nightfall', 'searching the library')"
                }
            },
            "required": []
        }),
    }
}

/// Restore a specific spell slot.
pub fn restore_spell_slot() -> Tool {
    Tool {
        name: "restore_spell_slot".to_string(),
        description: "Restore a specific spell slot without taking a rest. Use this for class features like Arcane Recovery (Wizard), magic items that restore spell slots, or other special abilities. Do NOT use this for normal rest recovery - use short_rest or long_rest instead.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "slot_level": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 9,
                    "description": "The level of the spell slot to restore (1-9)"
                },
                "source": {
                    "type": "string",
                    "description": "Why the spell slot is being restored (e.g., 'Arcane Recovery', 'Pearl of Power', 'Font of Magic')"
                }
            },
            "required": ["slot_level", "source"]
        }),
    }
}
