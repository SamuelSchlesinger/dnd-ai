//! Scheduling tools for time-based events.
//!
//! These tools allow the DM to schedule future events that trigger
//! based on time passage, not just player actions.

use chronicler_llm::Tool;
use serde_json::json;

/// Schedule a future event.
pub fn schedule_event() -> Tool {
    Tool {
        name: "schedule_event".to_string(),
        description: r#"Schedule a future event that will trigger based on time passage.

Use this when:
- An NPC says "meet me at the tavern tomorrow at noon"
- A festival or market has a specific start time
- A patrol will arrive in 30 minutes
- A curse will take effect at midnight
- Any timed deadline or appointment exists

The event will be tracked and you'll be notified when time passes and the event should trigger.

TIME SPECIFICATION:
- Use `minutes` and/or `hours` for relative time from now (e.g., "in 2 hours")
- Use `day`, `month`, `year`, `hour` for absolute time (e.g., "on day 5 at noon")
- Use `daily_hour` and `daily_minute` for recurring daily events (e.g., "every day at 8:00")

VISIBILITY:
- "public": Player knows about this event (appointments, announced festivals)
- "private": Player doesn't know (ambushes, secret meetings)
- "hinted": Player knows something will happen but not details

Examples:
- Guard patrol arrives: schedule_event(description="Town guard patrol arrives at the east gate", minutes=30, location="East Gate")
- Festival tomorrow: schedule_event(description="The Harvest Festival begins", day=5, hour=10, location="Town Square", visibility="public")
- Daily market: schedule_event(description="The morning market opens", daily_hour=8, daily_minute=0, repeating=true)"#.to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "description": {
                    "type": "string",
                    "description": "What happens when this event triggers"
                },
                "minutes": {
                    "type": "integer",
                    "minimum": 1,
                    "description": "Minutes from now until the event triggers (for relative timing)"
                },
                "hours": {
                    "type": "integer",
                    "minimum": 0,
                    "description": "Hours from now until the event triggers (for relative timing)"
                },
                "day": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 30,
                    "description": "Day of the month for absolute timing"
                },
                "month": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 12,
                    "description": "Month for absolute timing"
                },
                "year": {
                    "type": "integer",
                    "description": "Year for absolute timing"
                },
                "hour": {
                    "type": "integer",
                    "minimum": 0,
                    "maximum": 23,
                    "description": "Hour of day (0-23) for the event"
                },
                "daily_hour": {
                    "type": "integer",
                    "minimum": 0,
                    "maximum": 23,
                    "description": "Hour for daily recurring events (0-23)"
                },
                "daily_minute": {
                    "type": "integer",
                    "minimum": 0,
                    "maximum": 59,
                    "description": "Minute for daily recurring events (0-59)"
                },
                "location": {
                    "type": "string",
                    "description": "Where this event occurs (optional)"
                },
                "involved_entities": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "NPCs or other entities involved in this event"
                },
                "visibility": {
                    "type": "string",
                    "enum": ["public", "private", "hinted"],
                    "default": "public",
                    "description": "Whether the player knows about this event"
                },
                "repeating": {
                    "type": "boolean",
                    "default": false,
                    "description": "Whether this event repeats (only for daily events)"
                }
            },
            "required": ["description"]
        }),
    }
}

/// Check the schedule for upcoming events.
pub fn check_schedule() -> Tool {
    Tool {
        name: "check_schedule".to_string(),
        description: "Query upcoming scheduled events. Returns events the player knows about (public or hinted). Use this to remind yourself what's coming up or when the player asks about their schedule.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "location": {
                    "type": "string",
                    "description": "Filter to events at a specific location (optional)"
                },
                "include_private": {
                    "type": "boolean",
                    "default": false,
                    "description": "Include private events (for DM reference only)"
                }
            },
            "required": []
        }),
    }
}

/// Cancel a scheduled event.
pub fn cancel_event() -> Tool {
    Tool {
        name: "cancel_event".to_string(),
        description: "Cancel a scheduled event that is no longer happening. Use when circumstances change and a planned event won't occur (e.g., the NPC who was supposed to meet the player has died).".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "event_description": {
                    "type": "string",
                    "description": "Description of the event to cancel (partial match)"
                },
                "reason": {
                    "type": "string",
                    "description": "Why the event is being cancelled"
                }
            },
            "required": ["event_description", "reason"]
        }),
    }
}
