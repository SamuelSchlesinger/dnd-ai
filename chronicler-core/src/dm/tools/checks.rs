//! Dice rolling and check tools.

use chronicler_llm::Tool;
use serde_json::json;

/// Roll dice using standard D&D notation.
pub fn roll_dice() -> Tool {
    Tool {
        name: "roll_dice".to_string(),
        description: "Roll dice using standard D&D notation (e.g., '2d6+3', '1d20', '4d6kh3')."
            .to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "notation": {
                    "type": "string",
                    "description": "Dice notation (e.g., '2d6+3', '1d20+5', '4d6kh3')"
                },
                "purpose": {
                    "type": "string",
                    "description": "What the roll is for (e.g., 'damage', 'initiative')"
                }
            },
            "required": ["notation", "purpose"]
        }),
    }
}

/// Have a character make a skill check against a DC.
pub fn skill_check() -> Tool {
    Tool {
        name: "skill_check".to_string(),
        description: "Have a character make a skill check against a DC.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "skill": {
                    "type": "string",
                    "enum": ["athletics", "acrobatics", "sleight_of_hand", "stealth",
                            "arcana", "history", "investigation", "nature", "religion",
                            "animal_handling", "insight", "medicine", "perception", "survival",
                            "deception", "intimidation", "performance", "persuasion"],
                    "description": "The skill to check"
                },
                "dc": {
                    "type": "integer",
                    "description": "Difficulty Class for the check"
                },
                "description": {
                    "type": "string",
                    "description": "What the character is attempting"
                },
                "advantage": {
                    "type": "string",
                    "enum": ["normal", "advantage", "disadvantage"],
                    "description": "Advantage state for the roll"
                }
            },
            "required": ["skill", "dc", "description"]
        }),
    }
}

/// Have a character make a raw ability check (not tied to a skill).
pub fn ability_check() -> Tool {
    Tool {
        name: "ability_check".to_string(),
        description: "Have a character make a raw ability check (not tied to a skill).".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "ability": {
                    "type": "string",
                    "enum": ["strength", "dexterity", "constitution", "intelligence", "wisdom", "charisma"],
                    "description": "The ability to check"
                },
                "dc": {
                    "type": "integer",
                    "description": "Difficulty Class for the check"
                },
                "description": {
                    "type": "string",
                    "description": "What the character is attempting"
                },
                "advantage": {
                    "type": "string",
                    "enum": ["normal", "advantage", "disadvantage"],
                    "description": "Advantage state for the roll"
                }
            },
            "required": ["ability", "dc", "description"]
        }),
    }
}

/// Have a character make a saving throw.
pub fn saving_throw() -> Tool {
    Tool {
        name: "saving_throw".to_string(),
        description: "Have a character make a saving throw.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "ability": {
                    "type": "string",
                    "enum": ["strength", "dexterity", "constitution", "intelligence", "wisdom", "charisma"],
                    "description": "The ability for the save"
                },
                "dc": {
                    "type": "integer",
                    "description": "Difficulty Class for the save"
                },
                "source": {
                    "type": "string",
                    "description": "What is causing the saving throw"
                },
                "advantage": {
                    "type": "string",
                    "enum": ["normal", "advantage", "disadvantage"],
                    "description": "Advantage state for the roll"
                }
            },
            "required": ["ability", "dc", "source"]
        }),
    }
}
