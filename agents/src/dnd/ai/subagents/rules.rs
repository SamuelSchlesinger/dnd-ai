//! Rules lookup and adjudication subagent

use std::sync::Arc;

use async_trait::async_trait;

use agentic::agent::{Agent, AgentMetadata, Capabilities, Context, Response};
use agentic::error::AgentError;
use agentic::id::AgentId;
use agentic::message::Message;
use agentic::tool::Tool;

/// Rules specialist agent
///
/// Handles:
/// - Rules lookups (spells, conditions, abilities)
/// - Edge case adjudication
/// - House rule application
/// - Rules clarifications
pub struct RulesAgent {
    id: AgentId,
    metadata: AgentMetadata,
    capabilities: Capabilities,
}

impl RulesAgent {
    pub fn new() -> Self {
        Self {
            id: AgentId::new(),
            metadata: AgentMetadata::new("Rules Agent", "Specialist for D&D 5e rules lookup and adjudication"),
            capabilities: Capabilities {
                tool_use: false,
                memory: true, // RAG-backed rules database
                planning: false,
                streaming: false,
                vision: false,
                multi_agent: false,
                max_context_tokens: Some(40_000), // Large context for rules text
                supported_content: vec!["text".to_string()],
            },
        }
    }
}

impl Default for RulesAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Agent for RulesAgent {
    fn id(&self) -> &AgentId {
        &self.id
    }

    fn metadata(&self) -> &AgentMetadata {
        &self.metadata
    }

    fn capabilities(&self) -> &Capabilities {
        &self.capabilities
    }

    fn tools(&self) -> &[Arc<dyn Tool>] {
        &[]
    }

    async fn process(&self, message: Message, context: &mut Context) -> Result<Response, AgentError> {
        let input = message.text_content();

        // This would:
        // 1. Search rules database
        // 2. Find relevant rules text
        // 3. Provide authoritative answer

        Ok(Response::text(format!("[Rules] Looking up: {}", input)))
    }

    async fn initialize(&mut self) -> Result<(), AgentError> {
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), AgentError> {
        Ok(())
    }
}
