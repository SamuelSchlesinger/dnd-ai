//! Specialized subagents for the DM
//!
//! Each subagent handles a specific domain with focused context:
//! - CombatAgent: Combat rules, initiative, action resolution
//! - NPCAgent: NPC personalities and dialogue
//! - RulesAgent: Rules lookup and adjudication
//! - WorldAgent: Location descriptions and world state

pub mod combat;
pub mod npc;
pub mod rules;

use std::sync::Arc;

use async_trait::async_trait;
use once_cell::sync::Lazy;
use serde_json::{json, Value};

use agentic::agent::Agent;
use agentic::tool::{Tool, ToolAnnotations, ToolContext, ToolOutput};
use agentic::error::ToolError;

// Static schema for subagent tools
static SUBAGENT_TOOL_SCHEMA: Lazy<Value> = Lazy::new(|| {
    json!({
        "type": "object",
        "properties": {
            "query": {
                "type": "string",
                "description": "The query or task for the specialized agent"
            },
            "context": {
                "type": "object",
                "description": "Additional context for the agent"
            }
        },
        "required": ["query"]
    })
});

static DEFAULT_ANNOTATIONS: ToolAnnotations = ToolAnnotations {
    destructive: false,
    read_only: true,
    requires_approval: false,
    slow: false,
    network_access: false,
    file_system_access: false,
    has_cost: true, // Subagents use LLM calls
};

/// Wrapper that exposes an agent as a tool
pub struct SubagentTool {
    name: String,
    description: String,
    agent: Arc<dyn Agent>,
}

impl SubagentTool {
    pub fn new(name: impl Into<String>, description: impl Into<String>, agent: Arc<dyn Agent>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            agent,
        }
    }
}

#[async_trait]
impl Tool for SubagentTool {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn input_schema(&self) -> &Value {
        &SUBAGENT_TOOL_SCHEMA
    }

    fn annotations(&self) -> &ToolAnnotations {
        &DEFAULT_ANNOTATIONS
    }

    async fn execute(&self, params: Value, _ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let query = params["query"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidParameters {
                tool: self.name.clone(),
                reason: "Missing 'query' parameter".to_string(),
            })?;

        // In a full implementation, this would:
        // 1. Build a focused context for the subagent
        // 2. Send the query to the subagent
        // 3. Return the result

        // Placeholder response
        Ok(ToolOutput::text(format!(
            "[{} agent] Processing: {}",
            self.name, query
        )))
    }
}

/// Request router to determine which subagent to use
pub struct RequestRouter;

impl RequestRouter {
    /// Classify a player request and determine which subagent(s) to use
    pub fn classify(input: &str) -> Vec<SubagentType> {
        let lower = input.to_lowercase();

        let mut agents = Vec::new();

        // Combat keywords
        if lower.contains("attack")
            || lower.contains("hit")
            || lower.contains("damage")
            || lower.contains("initiative")
            || lower.contains("combat")
        {
            agents.push(SubagentType::Combat);
        }

        // NPC/dialogue keywords
        if lower.contains("talk")
            || lower.contains("speak")
            || lower.contains("ask")
            || lower.contains("say")
            || lower.contains("persuade")
            || lower.contains("intimidate")
        {
            agents.push(SubagentType::NPC);
        }

        // Rules keywords
        if lower.contains("how does")
            || lower.contains("rule")
            || lower.contains("can i")
            || lower.contains("what happens")
        {
            agents.push(SubagentType::Rules);
        }

        // World/exploration keywords
        if lower.contains("look")
            || lower.contains("examine")
            || lower.contains("search")
            || lower.contains("where")
            || lower.contains("travel")
        {
            agents.push(SubagentType::World);
        }

        // Default to world agent for general exploration
        if agents.is_empty() {
            agents.push(SubagentType::World);
        }

        agents
    }
}

/// Types of specialized subagents
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubagentType {
    Combat,
    NPC,
    Rules,
    World,
    Story,
    Encounter,
    Loot,
}
