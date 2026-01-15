//! Agent trait and core agent types.
//!
//! The Agent trait is the central abstraction in the framework. Agents process
//! messages, use tools, and produce responses.

use crate::error::AgentError;
use crate::id::AgentId;
use crate::message::Message;
use crate::tool::Tool;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Core agent trait - the central abstraction for all agents
#[async_trait]
pub trait Agent: Send + Sync {
    /// Get the agent's unique identifier
    fn id(&self) -> &AgentId;

    /// Get the agent's metadata (name, description, etc.)
    fn metadata(&self) -> &AgentMetadata;

    /// Get the agent's capabilities
    fn capabilities(&self) -> &Capabilities;

    /// Get the tools available to this agent
    fn tools(&self) -> &[Arc<dyn Tool>];

    /// Process a message and produce a response
    ///
    /// This is the main entry point for agent interaction.
    async fn process(
        &self,
        message: Message,
        context: &mut Context,
    ) -> Result<Response, AgentError>;

    /// Check if the agent is ready to process requests
    fn is_ready(&self) -> bool {
        true
    }

    /// Initialize the agent (called before first use)
    async fn initialize(&mut self) -> Result<(), AgentError> {
        Ok(())
    }

    /// Shutdown the agent (called when done)
    async fn shutdown(&mut self) -> Result<(), AgentError> {
        Ok(())
    }
}

/// Metadata describing an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetadata {
    /// Human-readable name
    pub name: String,
    /// Description of what this agent does
    pub description: String,
    /// Version string
    pub version: String,
    /// Author/creator
    pub author: Option<String>,
    /// Additional tags for categorization
    pub tags: Vec<String>,
}

impl AgentMetadata {
    /// Create new agent metadata
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            version: "0.1.0".to_string(),
            author: None,
            tags: Vec::new(),
        }
    }

    /// Set the version
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    /// Set the author
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    /// Add a tag
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }
}

/// Capabilities that an agent may have
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Capabilities {
    /// Can use tools
    pub tool_use: bool,
    /// Can maintain memory across interactions
    pub memory: bool,
    /// Can plan multi-step tasks
    pub planning: bool,
    /// Can stream responses
    pub streaming: bool,
    /// Can process images
    pub vision: bool,
    /// Can delegate to other agents
    pub multi_agent: bool,
    /// Maximum context window size (tokens)
    pub max_context_tokens: Option<usize>,
    /// Supported content types
    pub supported_content: Vec<String>,
}

impl Capabilities {
    /// Create capabilities with all features enabled
    pub fn full() -> Self {
        Self {
            tool_use: true,
            memory: true,
            planning: true,
            streaming: true,
            vision: true,
            multi_agent: true,
            max_context_tokens: Some(200_000),
            supported_content: vec![
                "text".to_string(),
                "image".to_string(),
                "tool_use".to_string(),
                "tool_result".to_string(),
            ],
        }
    }

    /// Create minimal capabilities (text only)
    pub fn minimal() -> Self {
        Self {
            tool_use: false,
            memory: false,
            planning: false,
            streaming: false,
            vision: false,
            multi_agent: false,
            max_context_tokens: Some(8_000),
            supported_content: vec!["text".to_string()],
        }
    }
}

/// Context for agent processing
#[derive(Debug, Clone, Default)]
pub struct Context {
    /// Conversation history
    pub messages: Vec<Message>,
    /// Current working state
    pub state: ContextState,
    /// Token usage tracking
    pub token_count: usize,
    /// Maximum tokens allowed
    pub max_tokens: usize,
}

impl Context {
    /// Create a new empty context
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            state: ContextState::default(),
            token_count: 0,
            max_tokens: 200_000,
        }
    }

    /// Create a context with a maximum token limit
    pub fn with_max_tokens(mut self, max: usize) -> Self {
        self.max_tokens = max;
        self
    }

    /// Add a message to the context
    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
    }

    /// Get the most recent messages
    pub fn recent_messages(&self, count: usize) -> &[Message] {
        let start = self.messages.len().saturating_sub(count);
        &self.messages[start..]
    }

    /// Check if context is approaching token limit
    pub fn is_near_limit(&self, threshold: f64) -> bool {
        let ratio = self.token_count as f64 / self.max_tokens as f64;
        ratio >= threshold
    }
}

/// State tracked within a context
#[derive(Debug, Clone, Default)]
pub struct ContextState {
    /// Current goal being worked on
    pub current_goal: Option<String>,
    /// Current plan step
    pub current_step: Option<usize>,
    /// Variables/facts accumulated
    pub variables: serde_json::Map<String, serde_json::Value>,
    /// Error count in current session
    pub error_count: usize,
}

/// Response from agent processing
#[derive(Debug, Clone)]
pub struct Response {
    /// The response message
    pub message: Message,
    /// Actions that were taken
    pub actions_taken: Vec<ActionSummary>,
    /// Whether the agent wants to continue
    pub wants_to_continue: bool,
    /// Suggested next step (if any)
    pub next_step: Option<String>,
}

impl Response {
    /// Create a simple text response
    pub fn text(content: impl Into<String>) -> Self {
        Self {
            message: Message::assistant(content),
            actions_taken: Vec::new(),
            wants_to_continue: false,
            next_step: None,
        }
    }

    /// Create a response from a message
    pub fn from_message(message: Message) -> Self {
        Self {
            message,
            actions_taken: Vec::new(),
            wants_to_continue: false,
            next_step: None,
        }
    }

    /// Mark that the agent wants to continue
    pub fn with_continuation(mut self, next_step: impl Into<String>) -> Self {
        self.wants_to_continue = true;
        self.next_step = Some(next_step.into());
        self
    }

    /// Add an action summary
    pub fn with_action(mut self, action: ActionSummary) -> Self {
        self.actions_taken.push(action);
        self
    }
}

/// Summary of an action taken during processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionSummary {
    /// Type of action
    pub action_type: String,
    /// Brief description
    pub description: String,
    /// Whether it succeeded
    pub success: bool,
    /// Duration
    pub duration_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_builder() {
        let meta = AgentMetadata::new("test", "A test agent")
            .with_version("1.0.0")
            .with_author("Test Author")
            .with_tag("test");

        assert_eq!(meta.name, "test");
        assert_eq!(meta.version, "1.0.0");
        assert_eq!(meta.author, Some("Test Author".to_string()));
        assert_eq!(meta.tags, vec!["test"]);
    }

    #[test]
    fn test_capabilities() {
        let full = Capabilities::full();
        assert!(full.tool_use);
        assert!(full.memory);

        let minimal = Capabilities::minimal();
        assert!(!minimal.tool_use);
        assert!(!minimal.memory);
    }

    #[test]
    fn test_context() {
        let mut ctx = Context::new().with_max_tokens(100);
        ctx.add_message(Message::user("Hello"));

        assert_eq!(ctx.messages.len(), 1);
        assert_eq!(ctx.max_tokens, 100);
    }
}
