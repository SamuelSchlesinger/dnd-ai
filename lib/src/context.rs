//! Context management for agent conversations.
//!
//! Handles context window optimization, summarization, RAG integration,
//! and state persistence.

use crate::error::ContextError;
use crate::id::{CheckpointId, SessionId, ThreadId};
use crate::message::Message;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Context manager for optimizing context window usage
#[async_trait]
pub trait ContextManager: Send + Sync {
    /// Build context for an interaction
    async fn build_context(&self, request: &ContextRequest) -> Result<BuiltContext, ContextError>;

    /// Compress context when approaching limits
    async fn compress(&self, context: &mut ConversationContext) -> Result<CompressionReport, ContextError>;

    /// Summarize content
    async fn summarize(&self, content: &str) -> Result<String, ContextError>;

    /// Estimate token count for content
    fn estimate_tokens(&self, content: &str) -> usize;
}

/// Request for building context
#[derive(Debug, Clone)]
pub struct ContextRequest {
    /// The current query/message
    pub query: String,
    /// Session ID for conversation continuity
    pub session_id: Option<SessionId>,
    /// Thread ID for branching conversations
    pub thread_id: Option<ThreadId>,
    /// Maximum tokens for the context
    pub max_tokens: usize,
    /// Priority settings for different content types
    pub priorities: ContextPriorities,
}

impl Default for ContextRequest {
    fn default() -> Self {
        Self {
            query: String::new(),
            session_id: None,
            thread_id: None,
            max_tokens: 100_000,
            priorities: ContextPriorities::default(),
        }
    }
}

impl ContextRequest {
    /// Create a new context request
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            ..Default::default()
        }
    }

    /// Set maximum tokens
    pub fn with_max_tokens(mut self, max: usize) -> Self {
        self.max_tokens = max;
        self
    }

    /// Set session ID
    pub fn with_session(mut self, session_id: SessionId) -> Self {
        self.session_id = Some(session_id);
        self
    }
}

/// Priority settings for context building
#[derive(Debug, Clone)]
pub struct ContextPriorities {
    /// Priority for system prompt
    pub system: Priority,
    /// Priority for recent messages
    pub recent_messages: Priority,
    /// Priority for retrieved documents
    pub retrieved: Priority,
    /// Priority for memories
    pub memories: Priority,
    /// Priority for tool outputs
    pub tool_outputs: Priority,
}

impl Default for ContextPriorities {
    fn default() -> Self {
        Self {
            system: Priority::Critical,
            recent_messages: Priority::High,
            retrieved: Priority::Medium,
            memories: Priority::Medium,
            tool_outputs: Priority::High,
        }
    }
}

/// Priority level for context segments
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    /// Lowest priority - can be dropped first
    Low,
    /// Medium priority
    Medium,
    /// High priority
    High,
    /// Critical - never drop
    Critical,
}

/// Built context ready for use
#[derive(Debug, Clone)]
pub struct BuiltContext {
    /// The system prompt
    pub system: Option<String>,
    /// Messages to include
    pub messages: Vec<Message>,
    /// Retrieved context documents
    pub retrieved: Vec<RetrievedDocument>,
    /// Total token count
    pub token_count: usize,
    /// Whether context was truncated
    pub was_truncated: bool,
}

/// A document retrieved for context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievedDocument {
    /// Document ID
    pub id: String,
    /// Document content
    pub content: String,
    /// Source of the document
    pub source: String,
    /// Relevance score (0.0 - 1.0)
    pub relevance: f32,
    /// Additional metadata
    pub metadata: HashMap<String, Value>,
}

/// Report from context compression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionReport {
    /// Tokens before compression
    pub tokens_before: usize,
    /// Tokens after compression
    pub tokens_after: usize,
    /// Number of messages summarized
    pub messages_summarized: usize,
    /// Number of documents dropped
    pub documents_dropped: usize,
}

/// Conversation context that can be persisted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationContext {
    /// Session identifier
    pub session_id: SessionId,
    /// Thread identifier
    pub thread_id: ThreadId,
    /// Conversation messages
    pub messages: Vec<Message>,
    /// Summary of older messages
    pub summary: Option<String>,
    /// Working state
    pub state: ConversationState,
    /// Creation time
    pub created_at: DateTime<Utc>,
    /// Last update time
    pub updated_at: DateTime<Utc>,
    /// Token count
    pub token_count: usize,
}

impl ConversationContext {
    /// Create a new conversation context
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            session_id: SessionId::new(),
            thread_id: ThreadId::new(),
            messages: Vec::new(),
            summary: None,
            state: ConversationState::default(),
            created_at: now,
            updated_at: now,
            token_count: 0,
        }
    }

    /// Add a message to the context
    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
        self.updated_at = Utc::now();
    }

    /// Get recent messages
    pub fn recent_messages(&self, count: usize) -> &[Message] {
        let start = self.messages.len().saturating_sub(count);
        &self.messages[start..]
    }

    /// Set a summary of older messages
    pub fn set_summary(&mut self, summary: String) {
        self.summary = Some(summary);
        self.updated_at = Utc::now();
    }

    /// Get message count
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }
}

impl Default for ConversationContext {
    fn default() -> Self {
        Self::new()
    }
}

/// State tracked within a conversation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConversationState {
    /// Current topic/goal
    pub current_topic: Option<String>,
    /// Accumulated variables
    pub variables: HashMap<String, Value>,
    /// User preferences learned
    pub preferences: HashMap<String, Value>,
    /// Current mode (e.g., "planning", "executing")
    pub mode: Option<String>,
}

/// Retriever trait for RAG
#[async_trait]
pub trait Retriever: Send + Sync {
    /// Retrieve relevant documents for a query
    async fn retrieve(
        &self,
        query: &str,
        options: &RetrievalOptions,
    ) -> Result<Vec<RetrievedDocument>, ContextError>;
}

/// Options for retrieval
#[derive(Debug, Clone)]
pub struct RetrievalOptions {
    /// Maximum documents to retrieve
    pub max_documents: usize,
    /// Minimum relevance score
    pub min_relevance: f32,
    /// Filter by source
    pub source_filter: Option<Vec<String>>,
    /// Filter by metadata
    pub metadata_filter: Option<HashMap<String, Value>>,
    /// Use hybrid search (dense + sparse)
    pub hybrid: bool,
}

impl Default for RetrievalOptions {
    fn default() -> Self {
        Self {
            max_documents: 5,
            min_relevance: 0.5,
            source_filter: None,
            metadata_filter: None,
            hybrid: true,
        }
    }
}

impl RetrievalOptions {
    /// Create new retrieval options
    pub fn new() -> Self {
        Self::default()
    }

    /// Set maximum documents
    pub fn with_max_documents(mut self, max: usize) -> Self {
        self.max_documents = max;
        self
    }

    /// Set minimum relevance
    pub fn with_min_relevance(mut self, min: f32) -> Self {
        self.min_relevance = min;
        self
    }
}

/// State persistence trait
#[async_trait]
pub trait StatePersistence: Send + Sync {
    /// Save a checkpoint
    async fn checkpoint(&self, context: &ConversationContext) -> Result<CheckpointId, ContextError>;

    /// Restore from a checkpoint
    async fn restore(&self, id: CheckpointId) -> Result<ConversationContext, ContextError>;

    /// List available checkpoints for a session
    async fn list_checkpoints(
        &self,
        session_id: SessionId,
    ) -> Result<Vec<CheckpointMetadata>, ContextError>;

    /// Delete a checkpoint
    async fn delete_checkpoint(&self, id: CheckpointId) -> Result<(), ContextError>;
}

/// Metadata about a checkpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointMetadata {
    /// Checkpoint ID
    pub id: CheckpointId,
    /// Session ID
    pub session_id: SessionId,
    /// When created
    pub created_at: DateTime<Utc>,
    /// Message count at checkpoint
    pub message_count: usize,
    /// Description/label
    pub label: Option<String>,
}

/// Summarizer for progressive summarization
#[async_trait]
pub trait Summarizer: Send + Sync {
    /// Summarize a sequence of messages
    async fn summarize_messages(&self, messages: &[Message]) -> Result<String, ContextError>;

    /// Summarize text content
    async fn summarize_text(&self, text: &str, max_length: usize) -> Result<String, ContextError>;

    /// Create a hierarchical summary
    async fn hierarchical_summarize(
        &self,
        sections: &[(&str, &str)],
    ) -> Result<String, ContextError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversation_context() {
        let mut ctx = ConversationContext::new();
        ctx.add_message(Message::user("Hello"));
        ctx.add_message(Message::assistant("Hi there!"));

        assert_eq!(ctx.message_count(), 2);
        assert_eq!(ctx.recent_messages(1).len(), 1);
    }

    #[test]
    fn test_context_request() {
        let req = ContextRequest::new("What is Rust?")
            .with_max_tokens(50_000)
            .with_session(SessionId::new());

        assert_eq!(req.query, "What is Rust?");
        assert_eq!(req.max_tokens, 50_000);
        assert!(req.session_id.is_some());
    }

    #[test]
    fn test_priority_ordering() {
        assert!(Priority::Critical > Priority::High);
        assert!(Priority::High > Priority::Medium);
        assert!(Priority::Medium > Priority::Low);
    }

    #[test]
    fn test_retrieval_options() {
        let opts = RetrievalOptions::new()
            .with_max_documents(10)
            .with_min_relevance(0.7);

        assert_eq!(opts.max_documents, 10);
        assert_eq!(opts.min_relevance, 0.7);
    }
}
