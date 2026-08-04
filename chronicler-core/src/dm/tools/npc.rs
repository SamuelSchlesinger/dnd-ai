//! NPC management tools: creation, updates, movement, and removal.

use chronicler_llm::Tool;
use serde_json::json;

/// Create a new NPC in the game world.
pub fn create_npc() -> Tool {
    Tool {
        name: "create_npc".to_string(),
        description: "Create a new NPC in the game world. Use this when introducing a new character that the player can interact with - shopkeepers, quest givers, guards, villagers, enemies, or any other non-player character. The NPC will be tracked for consistent portrayal across interactions.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "The NPC's name (e.g., 'Mira the Innkeeper', 'Guard Captain Aldric', 'Old Tom')"
                },
                "description": {
                    "type": "string",
                    "description": "Physical appearance and notable features (e.g., 'A stout dwarf with a braided red beard and a missing finger on his left hand')"
                },
                "personality": {
                    "type": "string",
                    "description": "Personality traits, mannerisms, and speech patterns (e.g., 'Gruff but fair, speaks in short sentences, always cleaning a mug')"
                },
                "occupation": {
                    "type": "string",
                    "description": "Their job or role in the world (e.g., 'innkeeper', 'town guard', 'blacksmith', 'traveling merchant', 'bandit leader')"
                },
                "disposition": {
                    "type": "string",
                    "enum": ["hostile", "unfriendly", "neutral", "friendly", "helpful"],
                    "default": "neutral",
                    "description": "Initial attitude toward the player: hostile (attacks on sight), unfriendly (distrustful), neutral (indifferent), friendly (welcoming), helpful (goes out of their way to assist)"
                },
                "location": {
                    "type": "string",
                    "description": "Name of the location where they can be found (e.g., 'The Rusty Anchor Inn', 'Town Square', 'Northern Gate')"
                },
                "known_information": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Secrets, rumors, or useful information this NPC knows that players might learn through interaction"
                }
            },
            "required": ["name", "description", "personality"]
        }),
    }
}

/// Update an existing NPC's state.
pub fn update_npc() -> Tool {
    Tool {
        name: "update_npc".to_string(),
        description: "Update an existing NPC's state or characteristics. Use this when an NPC's disposition changes due to player actions, when they learn new information, or when their circumstances change (e.g., after being helped, threatened, or witnessing events).".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "npc_name": {
                    "type": "string",
                    "description": "Name of the NPC to update (must match an existing NPC)"
                },
                "disposition": {
                    "type": "string",
                    "enum": ["hostile", "unfriendly", "neutral", "friendly", "helpful"],
                    "description": "New disposition toward the player"
                },
                "add_information": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "New information the NPC has learned or can now share"
                },
                "new_description": {
                    "type": "string",
                    "description": "Updated physical description (e.g., after injury, acquiring new gear, or aging)"
                },
                "new_personality": {
                    "type": "string",
                    "description": "Updated personality traits (e.g., after traumatic events, character growth, or corruption)"
                }
            },
            "required": ["npc_name"]
        }),
    }
}

/// Move an NPC to a different location.
pub fn move_npc() -> Tool {
    Tool {
        name: "move_npc".to_string(),
        description: "Move an NPC to a different location. Use this when NPCs travel, relocate, or are no longer at their usual spot. This keeps NPC locations consistent when players revisit areas or ask about someone's whereabouts.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "npc_name": {
                    "type": "string",
                    "description": "Name of the NPC to move (must match an existing NPC)"
                },
                "destination": {
                    "type": "string",
                    "description": "Name of the new location (e.g., 'The King's Castle', 'Wilderness Road', 'Fled the City')"
                },
                "reason": {
                    "type": "string",
                    "description": "Why they moved - useful for narrative consistency (e.g., 'Summoned by the king', 'Fled after the attack', 'Following the trade caravan')"
                }
            },
            "required": ["npc_name", "destination"]
        }),
    }
}

/// Remove an NPC from the game.
pub fn remove_npc() -> Tool {
    Tool {
        name: "remove_npc".to_string(),
        description: "Remove an NPC from the game permanently or temporarily. Use this when an NPC dies, permanently leaves the area, or otherwise exits the story. For temporary absences (traveling, hiding), consider using move_npc instead unless they're truly gone.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "npc_name": {
                    "type": "string",
                    "description": "Name of the NPC to remove (must match an existing NPC)"
                },
                "reason": {
                    "type": "string",
                    "description": "Why they're being removed from the game (e.g., 'Killed in the tavern brawl', 'Sailed away to distant lands', 'Turned to stone by the medusa')"
                },
                "permanent": {
                    "type": "boolean",
                    "default": true,
                    "description": "If true, the NPC is permanently gone. If false, they may return later (e.g., presumed dead, missing, or on a long journey)"
                }
            },
            "required": ["npc_name", "reason"]
        }),
    }
}
