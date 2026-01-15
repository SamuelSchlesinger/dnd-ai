//! Anthropic Claude API provider.
//!
//! This module implements the LLM provider trait for Anthropic's Claude models.

use super::{
    CompletionRequest, CompletionResponse, ContentDelta, LlmProvider, StopReason, StreamEvent,
    TokenUsage,
};
use crate::error::LlmError;
use crate::id::ToolCallId;
use crate::message::{ContentBlock, Message, Role};
use async_trait::async_trait;
use futures::StreamExt;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use tokio_stream::Stream;

/// Anthropic API base URL
const API_BASE: &str = "https://api.anthropic.com/v1";

/// API version header value
const API_VERSION: &str = "2023-06-01";

/// Anthropic Claude provider
pub struct AnthropicProvider {
    client: reqwest::Client,
    api_key: String,
    default_model: String,
}

impl AnthropicProvider {
    /// Create a new Anthropic provider
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key: api_key.into(),
            default_model: "claude-sonnet-4-20250514".to_string(),
        }
    }

    /// Create from environment variable ANTHROPIC_API_KEY
    pub fn from_env() -> Result<Self, LlmError> {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .map_err(|_| LlmError::Configuration("ANTHROPIC_API_KEY not set".to_string()))?;
        Ok(Self::new(api_key))
    }

    /// Set the default model
    pub fn with_default_model(mut self, model: impl Into<String>) -> Self {
        self.default_model = model.into();
        self
    }

    /// Build headers for API requests
    fn build_headers(&self) -> Result<HeaderMap, LlmError> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            "x-api-key",
            HeaderValue::from_str(&self.api_key)
                .map_err(|e| LlmError::Configuration(format!("Invalid API key: {}", e)))?,
        );
        headers.insert(
            "anthropic-version",
            HeaderValue::from_static(API_VERSION),
        );
        Ok(headers)
    }

    /// Convert our request format to Anthropic's API format
    fn to_api_request(&self, request: &CompletionRequest) -> ApiRequest {
        let messages: Vec<ApiMessage> = request
            .messages
            .iter()
            .map(|m| ApiMessage {
                role: match m.role {
                    Role::User => "user".to_string(),
                    Role::Assistant => "assistant".to_string(),
                    Role::System => "user".to_string(), // System is handled separately
                    Role::Tool => "user".to_string(),   // Tool results come from user side
                },
                content: m.content.iter().map(|c| c.into()).collect(),
            })
            .collect();

        let tools: Option<Vec<ApiTool>> = request.tools.as_ref().map(|tools| {
            tools
                .iter()
                .map(|t| ApiTool {
                    name: t.name.clone(),
                    description: t.description.clone(),
                    input_schema: t.input_schema.clone(),
                })
                .collect()
        });

        ApiRequest {
            model: request.model.clone(),
            max_tokens: request.max_tokens,
            system: request.system.clone(),
            messages,
            temperature: request.temperature,
            top_p: request.top_p,
            stop_sequences: request.stop_sequences.clone(),
            tools,
            tool_choice: request.tool_choice.as_ref().map(|tc| match tc {
                super::ToolChoice::Auto => ApiToolChoice { r#type: "auto".to_string(), name: None },
                super::ToolChoice::Any => ApiToolChoice { r#type: "any".to_string(), name: None },
                super::ToolChoice::Tool { name } => ApiToolChoice { r#type: "tool".to_string(), name: Some(name.clone()) },
                super::ToolChoice::None => ApiToolChoice { r#type: "none".to_string(), name: None },
            }),
            stream: false,
        }
    }

    /// Parse API response to our format
    fn parse_response(&self, api_response: ApiResponse) -> CompletionResponse {
        let content: Vec<ContentBlock> = api_response
            .content
            .into_iter()
            .map(|c| match c {
                ApiContent::Text { text } => ContentBlock::Text { text },
                ApiContent::ToolUse { id, name, input } => ContentBlock::ToolUse {
                    id: ToolCallId::from_string(id),
                    name,
                    input,
                },
                ApiContent::Thinking { thinking } => ContentBlock::Thinking { thinking },
            })
            .collect();

        let stop_reason = match api_response.stop_reason.as_str() {
            "end_turn" => StopReason::EndTurn,
            "max_tokens" => StopReason::MaxTokens,
            "stop_sequence" => StopReason::StopSequence,
            "tool_use" => StopReason::ToolUse,
            _ => StopReason::EndTurn,
        };

        CompletionResponse {
            id: api_response.id.clone(),
            model: api_response.model.clone(),
            message: Message::new(Role::Assistant, content),
            stop_reason,
            usage: TokenUsage {
                input_tokens: api_response.usage.input_tokens,
                output_tokens: api_response.usage.output_tokens,
                cache_read_tokens: api_response.usage.cache_read_input_tokens,
                cache_write_tokens: api_response.usage.cache_creation_input_tokens,
            },
        }
    }
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, LlmError> {
        let headers = self.build_headers()?;
        let api_request = self.to_api_request(&request);

        let response = self
            .client
            .post(format!("{}/messages", API_BASE))
            .headers(headers)
            .json(&api_request)
            .send()
            .await
            .map_err(|e| LlmError::Network(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().await.unwrap_or_default();
            return Err(LlmError::Api {
                status: status.as_u16(),
                message: error_body,
            });
        }

        let api_response: ApiResponse = response
            .json()
            .await
            .map_err(|e| LlmError::Parse(e.to_string()))?;

        Ok(self.parse_response(api_response))
    }

    async fn complete_stream(
        &self,
        request: CompletionRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamEvent, LlmError>> + Send>>, LlmError> {
        let headers = self.build_headers()?;
        let mut api_request = self.to_api_request(&request);
        api_request.stream = true;

        let response = self
            .client
            .post(format!("{}/messages", API_BASE))
            .headers(headers)
            .json(&api_request)
            .send()
            .await
            .map_err(|e| LlmError::Network(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().await.unwrap_or_default();
            return Err(LlmError::Api {
                status: status.as_u16(),
                message: error_body,
            });
        }

        let stream = response.bytes_stream().map(move |result| {
            result
                .map_err(|e| LlmError::Network(e.to_string()))
                .and_then(|bytes| {
                    let text = String::from_utf8_lossy(&bytes);
                    parse_sse_event(&text)
                })
        });

        Ok(Box::pin(stream))
    }

    fn name(&self) -> &str {
        "anthropic"
    }

    fn supported_models(&self) -> &[&str] {
        &[
            "claude-opus-4-20250514",
            "claude-sonnet-4-20250514",
            "claude-3-5-haiku-20241022",
            "claude-3-5-sonnet-20241022",
            "claude-3-opus-20240229",
            "claude-3-haiku-20240307",
        ]
    }

    fn is_ready(&self) -> bool {
        !self.api_key.is_empty()
    }
}

/// Parse SSE event from stream
fn parse_sse_event(text: &str) -> Result<StreamEvent, LlmError> {
    // SSE format: event: <type>\ndata: <json>\n\n
    for line in text.lines() {
        if line.starts_with("data: ") {
            let json_str = &line[6..];
            if json_str == "[DONE]" {
                return Ok(StreamEvent::MessageStop);
            }

            let event: ApiStreamEvent = serde_json::from_str(json_str)
                .map_err(|e| LlmError::Parse(format!("Failed to parse SSE: {}", e)))?;

            return Ok(match event {
                ApiStreamEvent::MessageStart { message } => StreamEvent::MessageStart {
                    message_id: message.id,
                    model: message.model,
                },
                ApiStreamEvent::ContentBlockStart { index, content_block } => {
                    StreamEvent::ContentBlockStart {
                        index,
                        content_type: content_block.r#type,
                    }
                }
                ApiStreamEvent::ContentBlockDelta { index, delta } => {
                    let content_delta = match delta {
                        ApiDelta::TextDelta { text } => ContentDelta::TextDelta { text },
                        ApiDelta::InputJsonDelta { partial_json } => {
                            ContentDelta::InputJsonDelta { partial_json }
                        }
                        ApiDelta::ThinkingDelta { thinking } => {
                            ContentDelta::ThinkingDelta { thinking }
                        }
                    };
                    StreamEvent::ContentBlockDelta {
                        index,
                        delta: content_delta,
                    }
                }
                ApiStreamEvent::ContentBlockStop { index } => {
                    StreamEvent::ContentBlockStop { index }
                }
                ApiStreamEvent::MessageDelta { delta, usage } => StreamEvent::MessageDelta {
                    stop_reason: delta.stop_reason.map(|s| match s.as_str() {
                        "end_turn" => StopReason::EndTurn,
                        "max_tokens" => StopReason::MaxTokens,
                        "stop_sequence" => StopReason::StopSequence,
                        "tool_use" => StopReason::ToolUse,
                        _ => StopReason::EndTurn,
                    }),
                    usage: usage.map(|u| TokenUsage {
                        input_tokens: u.input_tokens.unwrap_or(0),
                        output_tokens: u.output_tokens,
                        cache_read_tokens: None,
                        cache_write_tokens: None,
                    }),
                },
                ApiStreamEvent::MessageStop => StreamEvent::MessageStop,
                ApiStreamEvent::Ping => StreamEvent::Ping,
                ApiStreamEvent::Error { error } => StreamEvent::Error {
                    message: error.message,
                },
            });
        }
    }

    // If no data line found, return a ping (keepalive)
    Ok(StreamEvent::Ping)
}

// API request/response types

#[derive(Debug, Serialize)]
struct ApiRequest {
    model: String,
    max_tokens: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    messages: Vec<ApiMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_sequences: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<ApiTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<ApiToolChoice>,
    stream: bool,
}

#[derive(Debug, Serialize)]
struct ApiMessage {
    role: String,
    content: Vec<ApiContentBlock>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ApiContentBlock {
    Text { text: String },
    Image { source: ApiImageSource },
    ToolUse { id: String, name: String, input: serde_json::Value },
    ToolResult { tool_use_id: String, content: String, is_error: Option<bool> },
}

impl From<&ContentBlock> for ApiContentBlock {
    fn from(block: &ContentBlock) -> Self {
        match block {
            ContentBlock::Text { text } => ApiContentBlock::Text { text: text.clone() },
            ContentBlock::Image { data, media_type } => ApiContentBlock::Image {
                source: ApiImageSource {
                    r#type: "base64".to_string(),
                    media_type: media_type.clone(),
                    data: data.clone(),
                },
            },
            ContentBlock::ToolUse { id, name, input } => ApiContentBlock::ToolUse {
                id: id.to_string(),
                name: name.clone(),
                input: input.clone(),
            },
            ContentBlock::ToolResult { tool_use_id, content, is_error } => {
                ApiContentBlock::ToolResult {
                    tool_use_id: tool_use_id.to_string(),
                    content: content.clone(),
                    is_error: Some(*is_error),
                }
            }
            ContentBlock::Thinking { thinking } => {
                // Thinking blocks are not sent to the API, convert to text
                ApiContentBlock::Text { text: thinking.clone() }
            }
        }
    }
}

#[derive(Debug, Serialize)]
struct ApiImageSource {
    r#type: String,
    media_type: String,
    data: String,
}

#[derive(Debug, Serialize)]
struct ApiTool {
    name: String,
    description: String,
    input_schema: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct ApiToolChoice {
    r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ApiResponse {
    id: String,
    model: String,
    content: Vec<ApiContent>,
    stop_reason: String,
    usage: ApiUsage,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ApiContent {
    Text { text: String },
    ToolUse { id: String, name: String, input: serde_json::Value },
    Thinking { thinking: String },
}

#[derive(Debug, Deserialize)]
struct ApiUsage {
    input_tokens: usize,
    output_tokens: usize,
    cache_read_input_tokens: Option<usize>,
    cache_creation_input_tokens: Option<usize>,
}

// Streaming types

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ApiStreamEvent {
    MessageStart { message: ApiMessageStart },
    ContentBlockStart { index: usize, content_block: ApiContentBlockStart },
    ContentBlockDelta { index: usize, delta: ApiDelta },
    ContentBlockStop { index: usize },
    MessageDelta { delta: ApiMessageDelta, usage: Option<ApiStreamUsage> },
    MessageStop,
    Ping,
    Error { error: ApiError },
}

#[derive(Debug, Deserialize)]
struct ApiMessageStart {
    id: String,
    model: String,
}

#[derive(Debug, Deserialize)]
struct ApiContentBlockStart {
    r#type: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ApiDelta {
    TextDelta { text: String },
    InputJsonDelta { partial_json: String },
    ThinkingDelta { thinking: String },
}

#[derive(Debug, Deserialize)]
struct ApiMessageDelta {
    stop_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ApiStreamUsage {
    input_tokens: Option<usize>,
    output_tokens: usize,
}

#[derive(Debug, Deserialize)]
struct ApiError {
    message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_creation() {
        let provider = AnthropicProvider::new("test-key");
        assert_eq!(provider.name(), "anthropic");
        assert!(provider.is_ready());
    }

    #[test]
    fn test_supported_models() {
        let provider = AnthropicProvider::new("test-key");
        let models = provider.supported_models();
        assert!(models.contains(&"claude-sonnet-4-20250514"));
        assert!(models.contains(&"claude-opus-4-20250514"));
    }
}
