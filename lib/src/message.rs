//! Message types for agent communication.
//!
//! Messages are the primary means of communication between users, agents, and tools.

use crate::id::{MessageId, ToolCallId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

/// Role of the message sender
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    /// Message from the user
    User,
    /// Message from the assistant/agent
    Assistant,
    /// System instructions
    System,
    /// Tool result
    Tool,
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Role::User => write!(f, "user"),
            Role::Assistant => write!(f, "assistant"),
            Role::System => write!(f, "system"),
            Role::Tool => write!(f, "tool"),
        }
    }
}

/// Content block within a message
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    /// Plain text content
    Text {
        /// The text content
        text: String,
    },

    /// Image content (base64 encoded)
    Image {
        /// Base64 encoded image data
        data: String,
        /// MIME type (e.g., "image/png")
        media_type: String,
    },

    /// Tool use request from assistant
    ToolUse {
        /// Unique ID for this tool call
        id: ToolCallId,
        /// Name of the tool to invoke
        name: String,
        /// Input parameters as JSON
        input: Value,
    },

    /// Tool result (response to tool use)
    ToolResult {
        /// ID of the tool use this is responding to
        tool_use_id: ToolCallId,
        /// Result content
        content: String,
        /// Whether the tool execution failed
        #[serde(default)]
        is_error: bool,
    },

    /// Thinking/reasoning block (for extended thinking)
    Thinking {
        /// The thinking content
        thinking: String,
    },
}

impl ContentBlock {
    /// Create a text content block
    pub fn text(text: impl Into<String>) -> Self {
        ContentBlock::Text { text: text.into() }
    }

    /// Create an image content block
    pub fn image(data: impl Into<String>, media_type: impl Into<String>) -> Self {
        ContentBlock::Image {
            data: data.into(),
            media_type: media_type.into(),
        }
    }

    /// Create a tool use content block
    pub fn tool_use(id: ToolCallId, name: impl Into<String>, input: Value) -> Self {
        ContentBlock::ToolUse {
            id,
            name: name.into(),
            input,
        }
    }

    /// Create a tool result content block
    pub fn tool_result(tool_use_id: ToolCallId, content: impl Into<String>, is_error: bool) -> Self {
        ContentBlock::ToolResult {
            tool_use_id,
            content: content.into(),
            is_error,
        }
    }

    /// Check if this is a text block
    pub fn is_text(&self) -> bool {
        matches!(self, ContentBlock::Text { .. })
    }

    /// Check if this is a tool use block
    pub fn is_tool_use(&self) -> bool {
        matches!(self, ContentBlock::ToolUse { .. })
    }

    /// Check if this is a tool result block
    pub fn is_tool_result(&self) -> bool {
        matches!(self, ContentBlock::ToolResult { .. })
    }

    /// Get the text content if this is a text block
    pub fn as_text(&self) -> Option<&str> {
        match self {
            ContentBlock::Text { text } => Some(text),
            _ => None,
        }
    }

    /// Get tool use details if this is a tool use block
    pub fn as_tool_use(&self) -> Option<(&ToolCallId, &str, &Value)> {
        match self {
            ContentBlock::ToolUse { id, name, input } => Some((id, name, input)),
            _ => None,
        }
    }
}

/// Metadata associated with a message
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MessageMetadata {
    /// Model that generated this message (for assistant messages)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    /// Token usage information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<TokenUsage>,

    /// Stop reason (for assistant messages)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<StopReason>,

    /// Additional custom metadata
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

/// Token usage information
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TokenUsage {
    /// Input/prompt tokens
    pub input_tokens: usize,
    /// Output/completion tokens
    pub output_tokens: usize,
    /// Cache read tokens (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_read_tokens: Option<usize>,
    /// Cache creation tokens (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_creation_tokens: Option<usize>,
}

impl TokenUsage {
    /// Total tokens used
    pub fn total(&self) -> usize {
        self.input_tokens + self.output_tokens
    }
}

/// Reason the model stopped generating
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    /// Natural end of turn
    EndTurn,
    /// Hit max tokens limit
    MaxTokens,
    /// Model wants to use a tool
    ToolUse,
    /// Hit a stop sequence
    StopSequence,
}

/// A message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Unique message ID
    pub id: MessageId,
    /// Role of the sender
    pub role: Role,
    /// Content blocks
    pub content: Vec<ContentBlock>,
    /// When the message was created
    pub timestamp: DateTime<Utc>,
    /// Additional metadata
    #[serde(default)]
    pub metadata: MessageMetadata,
}

impl Message {
    /// Create a new message
    pub fn new(role: Role, content: Vec<ContentBlock>) -> Self {
        Self {
            id: MessageId::new(),
            role,
            content,
            timestamp: Utc::now(),
            metadata: MessageMetadata::default(),
        }
    }

    /// Create a user message with text content
    pub fn user(text: impl Into<String>) -> Self {
        Self::new(Role::User, vec![ContentBlock::text(text)])
    }

    /// Create an assistant message with text content
    pub fn assistant(text: impl Into<String>) -> Self {
        Self::new(Role::Assistant, vec![ContentBlock::text(text)])
    }

    /// Create a system message
    pub fn system(text: impl Into<String>) -> Self {
        Self::new(Role::System, vec![ContentBlock::text(text)])
    }

    /// Create a tool result message
    pub fn tool_result(tool_use_id: ToolCallId, content: impl Into<String>, is_error: bool) -> Self {
        Self::new(
            Role::Tool,
            vec![ContentBlock::tool_result(tool_use_id, content, is_error)],
        )
    }

    /// Get all text content concatenated
    pub fn text_content(&self) -> String {
        self.content
            .iter()
            .filter_map(|block| block.as_text())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Get all tool use blocks
    pub fn tool_uses(&self) -> Vec<(&ToolCallId, &str, &Value)> {
        self.content
            .iter()
            .filter_map(|block| block.as_tool_use())
            .collect()
    }

    /// Check if this message contains any tool use requests
    pub fn has_tool_use(&self) -> bool {
        self.content.iter().any(|block| block.is_tool_use())
    }

    /// Add a content block to this message
    pub fn with_content(mut self, block: ContentBlock) -> Self {
        self.content.push(block);
        self
    }

    /// Set metadata on this message
    pub fn with_metadata(mut self, metadata: MessageMetadata) -> Self {
        self.metadata = metadata;
        self
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.role, self.text_content())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let msg = Message::user("Hello, world!");
        assert_eq!(msg.role, Role::User);
        assert_eq!(msg.text_content(), "Hello, world!");
    }

    #[test]
    fn test_message_tool_use() {
        let tool_call_id = ToolCallId::new();
        let msg = Message::new(
            Role::Assistant,
            vec![ContentBlock::tool_use(
                tool_call_id,
                "read_file",
                serde_json::json!({"path": "/tmp/test.txt"}),
            )],
        );

        assert!(msg.has_tool_use());
        let tool_uses = msg.tool_uses();
        assert_eq!(tool_uses.len(), 1);
        assert_eq!(tool_uses[0].1, "read_file");
    }

    #[test]
    fn test_message_serde() {
        let msg = Message::user("Test message");
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.role, msg.role);
        assert_eq!(parsed.text_content(), msg.text_content());
    }
}
