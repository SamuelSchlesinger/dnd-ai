//! State assertion and inference tools.
//!
//! These tools provide a simpler, more declarative way to track state changes,
//! automatically capturing the "why" along with the "what". The `assert_state` tool
//! is designed to be easier for AI models to use correctly compared to fine-grained
//! update tools.

use chronicler_llm::Tool;
use serde_json::json;

/// Assert a state change declaratively.
///
/// This tool provides a simpler alternative to `update_npc`, `move_npc`, etc.
/// by allowing declarative state assertions that capture both the change and its reason.
pub fn assert_state() -> Tool {
    Tool {
        name: "assert_state".to_string(),
        description: r#"Declaratively assert a state change for any entity in the game world.

This is a SIMPLER alternative to update_npc, move_npc, and similar tools. Use it when:
- An NPC's disposition changes after player interaction
- An entity's location changes
- An NPC's status changes (alive/dead/injured/missing)
- A relationship between entities changes

The tool automatically records the reason for the change, creating a richer narrative history.

⚠️ IMPORTANT: Call this tool whenever player actions or story events cause state changes!
If the player helps an NPC and they smile warmly → assert_state(entity_name="Mira", state_type="disposition", new_value="friendly", reason="Player saved her shop from bandits")
If an NPC moves somewhere → assert_state(entity_name="Guard Captain", state_type="location", new_value="Northern Gate", reason="Responding to alarm")
If the player befriends someone → assert_state(entity_name="Mira", state_type="relationship", new_value="ally", target_entity="Player", reason="Trust built through shared adventure")"#.to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "entity_name": {
                    "type": "string",
                    "description": "Name of the entity whose state is changing"
                },
                "state_type": {
                    "type": "string",
                    "enum": ["disposition", "location", "status", "knowledge", "relationship"],
                    "description": "Type of state change: disposition (attitude toward player), location (where they are), status (alive/dead/injured/etc), knowledge (what they know), relationship (connection to another entity)"
                },
                "new_value": {
                    "type": "string",
                    "description": "The new state value. For disposition: hostile/unfriendly/neutral/friendly/helpful. For status: alive/dead/injured/missing/etc. For location/knowledge/relationship: descriptive text."
                },
                "reason": {
                    "type": "string",
                    "description": "Why this state changed - captures the narrative cause (e.g., 'Player saved their child', 'Witnessed the murder', 'Bribed with gold')"
                },
                "target_entity": {
                    "type": "string",
                    "description": "For relationship changes, the other entity in the relationship (e.g., if setting Mira's relationship to the Player, target_entity='Player')"
                }
            },
            "required": ["entity_name", "state_type", "new_value", "reason"]
        }),
    }
}

/// Check the current state of an entity.
pub fn query_state() -> Tool {
    Tool {
        name: "query_state".to_string(),
        description: "Query the current state of an entity. Use this to check an NPC's disposition, location, status, or relationships before making decisions.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "entity_name": {
                    "type": "string",
                    "description": "Name of the entity to query"
                },
                "state_type": {
                    "type": "string",
                    "enum": ["disposition", "location", "status", "knowledge", "relationship", "all"],
                    "description": "Type of state to query, or 'all' for complete state"
                }
            },
            "required": ["entity_name"]
        }),
    }
}
