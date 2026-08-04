//! Location management tools: create, connect, and update locations in the game world.

use chronicler_llm::Tool;
use serde_json::json;

/// Create a new location in the game world.
pub fn create_location() -> Tool {
    Tool {
        name: "create_location".to_string(),
        description: "Create a new location in the game world. Use this when introducing a new place that players might visit or that's relevant to the story. Create locations for important areas like towns, dungeons, taverns, or any place that might be revisited or referenced. Parent locations allow nesting (e.g., a room inside a tavern inside a town).".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "The location's name (e.g., 'The Rusty Anchor Tavern', 'Darkwood Forest', 'Chamber of Echoes')"
                },
                "location_type": {
                    "type": "string",
                    "enum": ["wilderness", "town", "city", "dungeon", "building", "room", "road", "cave", "other"],
                    "description": "Type of location"
                },
                "description": {
                    "type": "string",
                    "description": "Atmospheric description of the place - what players see, hear, smell, and feel when entering"
                },
                "parent_location": {
                    "type": "string",
                    "description": "Name of containing location if this is nested (e.g., a room inside a tavern, a shop inside a town)"
                },
                "items": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Notable items or loot present at this location that players might interact with"
                },
                "npcs_present": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Names of NPCs currently present at this location"
                }
            },
            "required": ["name", "location_type", "description"]
        }),
    }
}

/// Create a connection between two locations.
pub fn connect_locations() -> Tool {
    Tool {
        name: "connect_locations".to_string(),
        description: "Create a navigable connection between two locations. Use this to establish how players can travel between places - doors between rooms, roads between towns, paths through wilderness, etc. Connections help track the world's geography and enable travel descriptions.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "from_location": {
                    "type": "string",
                    "description": "Name of the first location"
                },
                "to_location": {
                    "type": "string",
                    "description": "Name of the second location"
                },
                "direction": {
                    "type": "string",
                    "description": "Direction or method of travel (e.g., 'north', 'east', 'up the stairs', 'through the iron door', 'across the bridge')"
                },
                "travel_time_minutes": {
                    "type": "integer",
                    "minimum": 0,
                    "description": "How long travel takes in minutes (0 for immediate, like walking through a door)"
                },
                "bidirectional": {
                    "type": "boolean",
                    "default": true,
                    "description": "If true, creates connection in both directions. Set false for one-way paths like cliffs or locked doors."
                }
            },
            "required": ["from_location", "to_location"]
        }),
    }
}

/// Update an existing location's state.
pub fn update_location() -> Tool {
    Tool {
        name: "update_location".to_string(),
        description: "Update an existing location's state after events change it. Use this when combat damages a location, NPCs arrive or leave, items are added or removed, or the description should change to reflect story events (e.g., 'the tavern is now on fire', 'the chest has been looted').".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "location_name": {
                    "type": "string",
                    "description": "Name of the location to update"
                },
                "new_description": {
                    "type": "string",
                    "description": "Updated atmospheric description reflecting changes (e.g., after a battle, after looting, after time passes)"
                },
                "add_items": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Items to add to the location"
                },
                "remove_items": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Items to remove from the location (taken by players, destroyed, etc.)"
                },
                "add_npcs": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "NPCs who have arrived at this location"
                },
                "remove_npcs": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "NPCs who have left this location (departed, killed, etc.)"
                }
            },
            "required": ["location_name"]
        }),
    }
}
