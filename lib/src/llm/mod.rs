//! LLM provider implementations.
//!
//! This module contains implementations for various LLM providers
//! that can power agent responses.

pub mod anthropic;

use crate::error::LlmError;
use crate::message::Message;
use crate::tool::ToolDefinition;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use tokio_stream::Stream;

/// Core trait for LLM providers
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Send a completion request
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, LlmError>;

    /// Send a streaming completion request
    async fn complete_stream(
        &self,
        request: CompletionRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamEvent, LlmError>> + Send>>, LlmError>;

    /// Get the provider name
    fn name(&self) -> &str;

    /// Get supported models
    fn supported_models(&self) -> &[&str];

    /// Check if provider is configured and ready
    fn is_ready(&self) -> bool;
}

/// Request for LLM completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    /// Model to use
    pub model: String,
    /// System prompt
    pub system: Option<String>,
    /// Messages in the conversation
    pub messages: Vec<Message>,
    /// Maximum tokens to generate
    pub max_tokens: usize,
    /// Temperature (0.0 - 1.0)
    pub temperature: Option<f32>,
    /// Top-p sampling
    pub top_p: Option<f32>,
    /// Stop sequences
    pub stop_sequences: Option<Vec<String>>,
    /// Available tools
    pub tools: Option<Vec<ToolDefinition>>,
    /// Tool choice setting
    pub tool_choice: Option<ToolChoice>,
}

impl Default for CompletionRequest {
    fn default() -> Self {
        Self {
            model: "claude-sonnet-4-20250514".to_string(),
            system: None,
            messages: Vec::new(),
            max_tokens: 4096,
            temperature: None,
            top_p: None,
            stop_sequences: None,
            tools: None,
            tool_choice: None,
        }
    }
}

impl CompletionRequest {
    /// Create a new completion request
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            ..Default::default()
        }
    }

    /// Set the system prompt
    pub fn with_system(mut self, system: impl Into<String>) -> Self {
        self.system = Some(system.into());
        self
    }

    /// Add a message
    pub fn with_message(mut self, message: Message) -> Self {
        self.messages.push(message);
        self
    }

    /// Set messages
    pub fn with_messages(mut self, messages: Vec<Message>) -> Self {
        self.messages = messages;
        self
    }

    /// Set max tokens
    pub fn with_max_tokens(mut self, max: usize) -> Self {
        self.max_tokens = max;
        self
    }

    /// Set temperature
    pub fn with_temperature(mut self, temp: f32) -> Self {
        self.temperature = Some(temp.clamp(0.0, 1.0));
        self
    }

    /// Set tools
    pub fn with_tools(mut self, tools: Vec<ToolDefinition>) -> Self {
        self.tools = Some(tools);
        self
    }

    /// Set tool choice
    pub fn with_tool_choice(mut self, choice: ToolChoice) -> Self {
        self.tool_choice = Some(choice);
        self
    }
}

/// Tool choice configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ToolChoice {
    /// Model decides whether to use tools
    Auto,
    /// Model must use a tool
    Any,
    /// Model must use a specific tool
    Tool { name: String },
    /// Model should not use tools
    None,
}

/// Response from LLM completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    /// Unique response ID
    pub id: String,
    /// Model used
    pub model: String,
    /// Response message
    pub message: Message,
    /// Stop reason
    pub stop_reason: StopReason,
    /// Token usage
    pub usage: TokenUsage,
}

/// Reason the model stopped generating
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    /// Reached end of response
    EndTurn,
    /// Hit max tokens
    MaxTokens,
    /// Hit a stop sequence
    StopSequence,
    /// Model wants to use a tool
    ToolUse,
}

/// Token usage statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenUsage {
    /// Input tokens
    pub input_tokens: usize,
    /// Output tokens
    pub output_tokens: usize,
    /// Cache read tokens (if applicable)
    pub cache_read_tokens: Option<usize>,
    /// Cache write tokens (if applicable)
    pub cache_write_tokens: Option<usize>,
}

impl TokenUsage {
    /// Get total tokens used
    pub fn total(&self) -> usize {
        self.input_tokens + self.output_tokens
    }
}

/// Event from streaming completion
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamEvent {
    /// Message started
    MessageStart {
        message_id: String,
        model: String,
    },
    /// Content block started
    ContentBlockStart {
        index: usize,
        content_type: String,
    },
    /// Text delta
    ContentBlockDelta {
        index: usize,
        delta: ContentDelta,
    },
    /// Content block finished
    ContentBlockStop {
        index: usize,
    },
    /// Message finished
    MessageDelta {
        stop_reason: Option<StopReason>,
        usage: Option<TokenUsage>,
    },
    /// Message complete
    MessageStop,
    /// Ping/keepalive
    Ping,
    /// Error occurred
    Error {
        message: String,
    },
}

/// Delta content in streaming
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentDelta {
    /// Text delta
    TextDelta { text: String },
    /// Tool input delta (JSON string)
    InputJsonDelta { partial_json: String },
    /// Thinking delta
    ThinkingDelta { thinking: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completion_request() {
        let req = CompletionRequest::new("claude-sonnet-4-20250514")
            .with_system("You are a helpful assistant")
            .with_message(Message::user("Hello"))
            .with_max_tokens(1000)
            .with_temperature(0.7);

        assert_eq!(req.model, "claude-sonnet-4-20250514");
        assert_eq!(req.system, Some("You are a helpful assistant".to_string()));
        assert_eq!(req.max_tokens, 1000);
        assert_eq!(req.temperature, Some(0.7));
    }

    #[test]
    fn test_token_usage() {
        let usage = TokenUsage {
            input_tokens: 100,
            output_tokens: 50,
            cache_read_tokens: Some(20),
            cache_write_tokens: None,
        };
        assert_eq!(usage.total(), 150);
    }
}
