//! Quest management tools for the AI Dungeon Master.
//!
//! These tools allow the DM to create quests, manage objectives,
//! and track quest completion status.

use chronicler_llm::Tool;
use serde_json::json;

/// Create a new quest.
pub fn create_quest() -> Tool {
    Tool {
        name: "create_quest".to_string(),
        description: "Create a new quest for the player. Use this when an NPC assigns a quest, the player discovers a goal, or a story objective is established. The quest will appear in the player's quest log.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Name of the quest (e.g., 'The Missing Merchant', 'Clear the Goblin Cave')"
                },
                "description": {
                    "type": "string",
                    "description": "Description of the quest explaining the goal and context"
                },
                "giver": {
                    "type": "string",
                    "description": "Name of the NPC who gave this quest (optional)"
                },
                "objectives": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "description": {
                                "type": "string",
                                "description": "Description of this objective"
                            },
                            "optional": {
                                "type": "boolean",
                                "description": "Whether this is an optional objective (default: false)"
                            }
                        },
                        "required": ["description"]
                    },
                    "description": "List of objectives for the quest"
                },
                "rewards": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "List of rewards promised for completing the quest (e.g., '100 gold', 'Magic sword', 'Town reputation')"
                }
            },
            "required": ["name", "description"]
        }),
    }
}

/// Add an objective to an existing quest.
pub fn add_quest_objective() -> Tool {
    Tool {
        name: "add_quest_objective".to_string(),
        description: "Add a new objective to an existing active quest. Use this when the player discovers a new step needed to complete a quest, or when quest requirements become clearer.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "quest_name": {
                    "type": "string",
                    "description": "Name of the quest to add the objective to"
                },
                "objective": {
                    "type": "string",
                    "description": "Description of the new objective"
                },
                "optional": {
                    "type": "boolean",
                    "description": "Whether this is an optional objective (default: false)"
                }
            },
            "required": ["quest_name", "objective"]
        }),
    }
}

/// Complete a quest objective.
pub fn complete_objective() -> Tool {
    Tool {
        name: "complete_objective".to_string(),
        description: "Mark a quest objective as completed. Use this when the player accomplishes a specific step of a quest. If all required objectives are complete, the quest status will automatically update.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "quest_name": {
                    "type": "string",
                    "description": "Name of the quest containing the objective"
                },
                "objective_description": {
                    "type": "string",
                    "description": "Description of the objective to mark as complete (partial match allowed)"
                }
            },
            "required": ["quest_name", "objective_description"]
        }),
    }
}

/// Complete a quest.
pub fn complete_quest() -> Tool {
    Tool {
        name: "complete_quest".to_string(),
        description: "Mark a quest as completed. Use this when the player finishes all required objectives and turns in the quest or achieves the final goal. This should typically be followed by awarding any promised rewards.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "quest_name": {
                    "type": "string",
                    "description": "Name of the quest to complete"
                },
                "completion_note": {
                    "type": "string",
                    "description": "Optional note about how the quest was completed"
                }
            },
            "required": ["quest_name"]
        }),
    }
}

/// Fail a quest.
pub fn fail_quest() -> Tool {
    Tool {
        name: "fail_quest".to_string(),
        description: "Mark a quest as failed. Use this when the player can no longer complete a quest due to their actions or inaction (e.g., the NPC they were supposed to save died, the time limit expired).".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "quest_name": {
                    "type": "string",
                    "description": "Name of the quest that failed"
                },
                "failure_reason": {
                    "type": "string",
                    "description": "Explanation of why the quest failed"
                }
            },
            "required": ["quest_name", "failure_reason"]
        }),
    }
}

/// Update quest description or rewards.
pub fn update_quest() -> Tool {
    Tool {
        name: "update_quest".to_string(),
        description: "Update a quest's description or rewards. Use this when new information is revealed about a quest, or when the promised rewards change.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "quest_name": {
                    "type": "string",
                    "description": "Name of the quest to update"
                },
                "new_description": {
                    "type": "string",
                    "description": "New description for the quest (if changing)"
                },
                "add_rewards": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Additional rewards to add"
                }
            },
            "required": ["quest_name"]
        }),
    }
}
