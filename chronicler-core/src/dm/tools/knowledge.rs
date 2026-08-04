//! Knowledge tracking tools for information asymmetry.
//!
//! These tools track what entities know, when they learned it,
//! and from whom. This enables secrets, rumors, and information
//! asymmetry between characters.

use chronicler_llm::Tool;
use serde_json::json;

/// Share knowledge between entities.
pub fn share_knowledge() -> Tool {
    Tool {
        name: "share_knowledge".to_string(),
        description: r#"Record that an entity knows a piece of information.

Use this when:
- An NPC learns something from the player
- The player learns something from an NPC
- An NPC tells another NPC something
- An entity witnesses an event
- You want to track secrets, rumors, or information asymmetry

The verification_status indicates whether the information is:
- "true" - Verified fact
- "false" - Deliberate lie or misinformation
- "partial" - Contains both true and false elements
- "unknown" - Rumor or unverified information
- "outdated" - Was true but is no longer current

Examples:
- Player tells Mira about the haunted mine: share_knowledge(knowing_entity="Mira", content="The old mine is haunted by undead", source="player", verification="true")
- Bartender spreads a rumor: share_knowledge(knowing_entity="Various Patrons", content="The baron is bankrupt", source="Bartender Joe", verification="unknown")
- NPC lies to player: share_knowledge(knowing_entity="Player", content="The treasure is in the east tower", source_entity="Treacherous Guard", verification="false")"#.to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "knowing_entity": {
                    "type": "string",
                    "description": "Name of the entity that now knows this information"
                },
                "content": {
                    "type": "string",
                    "description": "The information being shared"
                },
                "source": {
                    "type": "string",
                    "description": "Where/who the information came from: 'player', 'observation', an entity name, or a written source"
                },
                "verification": {
                    "type": "string",
                    "enum": ["true", "false", "partial", "unknown", "outdated"],
                    "default": "unknown",
                    "description": "The verification status of this information"
                },
                "context": {
                    "type": "string",
                    "description": "Optional context about how this information was shared (e.g., 'whispered in secret', 'overheard at the tavern')"
                }
            },
            "required": ["knowing_entity", "content", "source"]
        }),
    }
}

/// Query what an entity knows.
pub fn query_knowledge() -> Tool {
    Tool {
        name: "query_knowledge".to_string(),
        description: "Query what an entity knows. Returns all current knowledge the entity has, or filters by topic if specified.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "entity_name": {
                    "type": "string",
                    "description": "Name of the entity to query"
                },
                "topic": {
                    "type": "string",
                    "description": "Optional topic to filter by (e.g., 'treasure', 'baron', 'haunted mine')"
                }
            },
            "required": ["entity_name"]
        }),
    }
}
