//! Minimal Anthropic Claude API client.
//!
//! This crate provides a focused client for Claude's Messages API with:
//! - Non-streaming and streaming completions
//! - Tool use support
//! - Proper SSE parsing for streaming responses

use futures::StreamExt;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use thiserror::Error;
use tokio_stream::Stream;

const API_BASE: &str = "https://api.anthropic.com/v1";
const API_VERSION: &str = "2023-06-01";
const DEFAULT_MODEL: &str = "claude-sonnet-4-20250514";

/// Errors that can occur when using the Claude client.
#[derive(Debug, Error)]
pub enum Error {
    #[error("API key not configured")]
    NoApiKey,

    #[error("Network error: {0}")]
    Network(String),

    #[error("API error (status {status}): {message}")]
    Api { status: u16, message: String },

    #[error("Failed to parse response: {0}")]
    Parse(String),

    #[error("Invalid configuration: {0}")]
    Config(String),
}

/// Claude API client.
#[derive(Clone)]
pub struct Claude {
    client: reqwest::Client,
    api_key: String,
    model: String,
}

impl Claude {
    /// Create a new Claude client with the given API key.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .connect_timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to build HTTP client"),
            api_key: api_key.into(),
            model: DEFAULT_MODEL.to_string(),
        }
    }

    /// Create a Claude client from the ANTHROPIC_API_KEY environment variable.
    pub fn from_env() -> Result<Self, Error> {
        let api_key = std::env::var("ANTHROPIC_API_KEY").map_err(|_| Error::NoApiKey)?;
        Ok(Self::new(api_key))
    }

    /// Set the default model for this client.
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Send a completion request and return the full response.
    pub async fn complete(&self, request: Request) -> Result<Response, Error> {
        let api_request = self.build_api_request(&request, false);
        let headers = self.build_headers()?;

        let response = self
            .client
            .post(format!("{API_BASE}/messages"))
            .headers(headers)
            .json(&api_request)
            .send()
            .await
            .map_err(|e| Error::Network(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                status,
                message: body,
            });
        }

        let api_response: ApiResponse = response
            .json()
            .await
            .map_err(|e| Error::Parse(e.to_string()))?;

        Ok(self.parse_response(api_response))
    }

    /// Send a completion request and stream the response.
    pub async fn stream(
        &self,
        request: Request,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamEvent, Error>> + Send>>, Error> {
        let api_request = self.build_api_request(&request, true);
        let headers = self.build_headers()?;

        let response = self
            .client
            .post(format!("{API_BASE}/messages"))
            .headers(headers)
            .json(&api_request)
            .send()
            .await
            .map_err(|e| Error::Network(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                status,
                message: body,
            });
        }

        // Use scan to maintain a buffer for incomplete SSE events across chunks
        let stream = response
            .bytes_stream()
            .scan(String::new(), |buffer, result| {
                let events = match result {
                    Ok(bytes) => {
                        buffer.push_str(&String::from_utf8_lossy(&bytes));
                        parse_sse_events_buffered(buffer)
                    }
                    Err(e) => vec![Err(Error::Network(e.to_string()))],
                };
                // Return Some to continue, the events vector
                futures::future::ready(Some(events))
            })
            .flat_map(futures::stream::iter);

        Ok(Box::pin(stream))
    }

    /// Run a tool use loop until completion.
    ///
    /// Given a request with tools and an executor function, this method will:
    /// 1. Send the request to Claude
    /// 2. If Claude calls tools, execute them using the provided function
    /// 3. Send tool results back and repeat until Claude stops using tools
    pub async fn complete_with_tools<F, Fut>(
        &self,
        mut request: Request,
        mut executor: F,
    ) -> Result<Response, Error>
    where
        F: FnMut(ToolUse) -> Fut,
        Fut: std::future::Future<Output = ToolResult>,
    {
        loop {
            let response = self.complete(request.clone()).await?;

            if response.stop_reason != StopReason::ToolUse {
                return Ok(response);
            }

            // Collect tool uses
            let tool_uses: Vec<ToolUse> = response
                .content
                .iter()
                .filter_map(|block| {
                    if let ContentBlock::ToolUse { id, name, input } = block {
                        Some(ToolUse {
                            id: id.clone(),
                            name: name.clone(),
                            input: input.clone(),
                        })
                    } else {
                        None
                    }
                })
                .collect();

            if tool_uses.is_empty() {
                return Ok(response);
            }

            // Add assistant response to messages
            request.messages.push(Message {
                role: Role::Assistant,
                content: response.content.clone(),
            });

            // Execute tools and collect results
            let mut tool_results = Vec::new();
            for tool_use in tool_uses {
                let result = executor(tool_use.clone()).await;
                tool_results.push(ContentBlock::ToolResult {
                    tool_use_id: tool_use.id,
                    content: result.content,
                    is_error: result.is_error,
                });
            }

            // Add tool results as user message
            request.messages.push(Message {
                role: Role::User,
                content: tool_results,
            });
        }
    }

    fn build_headers(&self) -> Result<HeaderMap, Error> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            "x-api-key",
            HeaderValue::from_str(&self.api_key)
                .map_err(|e| Error::Config(format!("Invalid API key: {e}")))?,
        );
        headers.insert("anthropic-version", HeaderValue::from_static(API_VERSION));
        Ok(headers)
    }

    fn build_api_request(&self, request: &Request, stream: bool) -> ApiRequest {
        let messages: Vec<ApiMessage> = request
            .messages
            .iter()
            .map(|m| ApiMessage {
                role: match m.role {
                    Role::User => "user".to_string(),
                    Role::Assistant => "assistant".to_string(),
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
            model: request.model.clone().unwrap_or_else(|| self.model.clone()),
            max_tokens: request.max_tokens,
            system: request.system.clone(),
            messages,
            temperature: request.temperature,
            tools,
            tool_choice: request.tool_choice.as_ref().map(|tc| match tc {
                ToolChoice::Auto => ApiToolChoice {
                    r#type: "auto".to_string(),
                    name: None,
                },
                ToolChoice::Any => ApiToolChoice {
                    r#type: "any".to_string(),
                    name: None,
                },
                ToolChoice::Tool { name } => ApiToolChoice {
                    r#type: "tool".to_string(),
                    name: Some(name.clone()),
                },
            }),
            stream,
        }
    }

    fn parse_response(&self, api_response: ApiResponse) -> Response {
        let content: Vec<ContentBlock> = api_response
            .content
            .into_iter()
            .map(|c| match c {
                ApiContent::Text { text } => ContentBlock::Text { text },
                ApiContent::ToolUse { id, name, input } => {
                    ContentBlock::ToolUse { id, name, input }
                }
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

        Response {
            id: api_response.id,
            model: api_response.model,
            content,
            stop_reason,
            usage: Usage {
                input_tokens: api_response.usage.input_tokens,
                output_tokens: api_response.usage.output_tokens,
            },
        }
    }
}

// ============================================================================
// Public types
// ============================================================================

/// A completion request to send to Claude.
#[derive(Debug, Clone)]
pub struct Request {
    pub model: Option<String>,
    pub max_tokens: usize,
    pub system: Option<String>,
    pub messages: Vec<Message>,
    pub temperature: Option<f32>,
    pub tools: Option<Vec<Tool>>,
    pub tool_choice: Option<ToolChoice>,
}

impl Request {
    /// Create a new request with the given messages.
    pub fn new(messages: Vec<Message>) -> Self {
        Self {
            model: None,
            max_tokens: 4096,
            system: None,
            messages,
            temperature: None,
            tools: None,
            tool_choice: None,
        }
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    pub fn with_max_tokens(mut self, max_tokens: usize) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    pub fn with_system(mut self, system: impl Into<String>) -> Self {
        self.system = Some(system.into());
        self
    }

    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    pub fn with_tools(mut self, tools: Vec<Tool>) -> Self {
        self.tools = Some(tools);
        self
    }

    pub fn with_tool_choice(mut self, tool_choice: ToolChoice) -> Self {
        self.tool_choice = Some(tool_choice);
        self
    }
}

/// A message in the conversation.
#[derive(Debug, Clone)]
pub struct Message {
    pub role: Role,
    pub content: Vec<ContentBlock>,
}

impl Message {
    /// Create a user message with text content.
    pub fn user(text: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: vec![ContentBlock::Text { text: text.into() }],
        }
    }

    /// Create an assistant message with text content.
    pub fn assistant(text: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: vec![ContentBlock::Text { text: text.into() }],
        }
    }
}

/// The role of a message sender.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    User,
    Assistant,
}

/// A block of content in a message.
#[derive(Debug, Clone)]
pub enum ContentBlock {
    Text {
        text: String,
    },
    Image {
        media_type: String,
        data: String,
    },
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    ToolResult {
        tool_use_id: String,
        content: String,
        is_error: bool,
    },
    Thinking {
        thinking: String,
    },
}

impl ContentBlock {
    /// Extract text from a Text content block.
    pub fn as_text(&self) -> Option<&str> {
        if let ContentBlock::Text { text } = self {
            Some(text)
        } else {
            None
        }
    }
}

/// A tool definition.
#[derive(Debug, Clone)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

/// Tool choice configuration.
#[derive(Debug, Clone)]
pub enum ToolChoice {
    Auto,
    Any,
    Tool { name: String },
}

/// A completion response from Claude.
#[derive(Debug, Clone)]
pub struct Response {
    pub id: String,
    pub model: String,
    pub content: Vec<ContentBlock>,
    pub stop_reason: StopReason,
    pub usage: Usage,
}

impl Response {
    /// Get all text content concatenated.
    pub fn text(&self) -> String {
        self.content
            .iter()
            .filter_map(|block| block.as_text())
            .collect::<Vec<_>>()
            .join("")
    }
}

/// Why the model stopped generating.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StopReason {
    EndTurn,
    MaxTokens,
    StopSequence,
    ToolUse,
}

/// Token usage information.
#[derive(Debug, Clone)]
pub struct Usage {
    pub input_tokens: usize,
    pub output_tokens: usize,
}

/// A tool use request from Claude.
#[derive(Debug, Clone)]
pub struct ToolUse {
    pub id: String,
    pub name: String,
    pub input: serde_json::Value,
}

/// Result of executing a tool.
#[derive(Debug, Clone)]
pub struct ToolResult {
    pub content: String,
    pub is_error: bool,
}

impl ToolResult {
    pub fn success(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            is_error: false,
        }
    }

    pub fn error(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            is_error: true,
        }
    }
}

// ============================================================================
// Streaming types
// ============================================================================

/// Events from a streaming response.
#[derive(Debug, Clone)]
pub enum StreamEvent {
    MessageStart {
        id: String,
        model: String,
    },
    ContentBlockStart {
        index: usize,
        content_type: String,
        /// Tool use ID (only present for tool_use blocks)
        tool_use_id: Option<String>,
        /// Tool name (only present for tool_use blocks)
        tool_name: Option<String>,
    },
    TextDelta {
        index: usize,
        text: String,
    },
    InputJsonDelta {
        index: usize,
        partial_json: String,
    },
    ContentBlockStop {
        index: usize,
    },
    MessageDelta {
        stop_reason: Option<StopReason>,
    },
    MessageStop,
    Ping,
    Error {
        message: String,
    },
}

// ============================================================================
// Internal API types
// ============================================================================

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
    Text {
        text: String,
    },
    Image {
        source: ApiImageSource,
    },
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    ToolResult {
        tool_use_id: String,
        content: String,
        is_error: bool,
    },
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
                id: id.clone(),
                name: name.clone(),
                input: input.clone(),
            },
            ContentBlock::ToolResult {
                tool_use_id,
                content,
                is_error,
            } => ApiContentBlock::ToolResult {
                tool_use_id: tool_use_id.clone(),
                content: content.clone(),
                is_error: *is_error,
            },
            ContentBlock::Thinking { thinking } => ApiContentBlock::Text {
                text: thinking.clone(),
            },
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
    Text {
        text: String,
    },
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    Thinking {
        thinking: String,
    },
}

#[derive(Debug, Deserialize)]
struct ApiUsage {
    input_tokens: usize,
    output_tokens: usize,
}

// Streaming types
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ApiStreamEvent {
    MessageStart {
        message: ApiMessageStart,
    },
    ContentBlockStart {
        index: usize,
        content_block: ApiContentBlockStart,
    },
    ContentBlockDelta {
        index: usize,
        delta: ApiDelta,
    },
    ContentBlockStop {
        index: usize,
    },
    MessageDelta {
        delta: ApiMessageDelta,
    },
    MessageStop,
    Ping,
    Error {
        error: ApiError,
    },
}

#[derive(Debug, Deserialize)]
struct ApiMessageStart {
    id: String,
    model: String,
}

#[derive(Debug, Deserialize)]
struct ApiContentBlockStart {
    r#type: String,
    /// Tool use ID (present for tool_use blocks)
    #[serde(default)]
    id: Option<String>,
    /// Tool name (present for tool_use blocks)
    #[serde(default)]
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[allow(clippy::enum_variant_names)]
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
struct ApiError {
    message: String,
}

/// Parse SSE events from a buffer, consuming complete events and leaving incomplete data.
///
/// SSE events are separated by double newlines. This function finds complete events,
/// parses them, and removes them from the buffer, leaving any incomplete event data
/// for the next chunk.
fn parse_sse_events_buffered(buffer: &mut String) -> Vec<Result<StreamEvent, Error>> {
    let mut events = Vec::new();

    // Process complete SSE events (terminated by \n\n or at end of valid data lines)
    loop {
        // Find the next complete line (ending with \n)
        let Some(newline_pos) = buffer.find('\n') else {
            // No complete line yet, wait for more data
            break;
        };

        let line = &buffer[..newline_pos];

        // Check if this is a data line
        if let Some(json_str) = line.strip_prefix("data: ") {
            if json_str == "[DONE]" {
                events.push(Ok(StreamEvent::MessageStop));
            } else if !json_str.is_empty() {
                match serde_json::from_str::<ApiStreamEvent>(json_str) {
                    Ok(event) => events.push(Ok(convert_stream_event(event))),
                    Err(e) => {
                        // Check if it looks like incomplete JSON (ends abruptly)
                        // If so, don't consume the line - wait for more data
                        if e.is_eof() {
                            break;
                        }
                        events.push(Err(Error::Parse(format!("SSE parse error: {e}"))));
                    }
                }
            }
        }
        // Skip event: lines, empty lines, and other SSE metadata

        // Consume the processed line (including the newline)
        buffer.drain(..=newline_pos);
    }

    // Return events (may be empty if waiting for more data)
    events
}

fn convert_stream_event(event: ApiStreamEvent) -> StreamEvent {
    match event {
        ApiStreamEvent::MessageStart { message } => StreamEvent::MessageStart {
            id: message.id,
            model: message.model,
        },
        ApiStreamEvent::ContentBlockStart {
            index,
            content_block,
        } => StreamEvent::ContentBlockStart {
            index,
            content_type: content_block.r#type,
            tool_use_id: content_block.id,
            tool_name: content_block.name,
        },
        ApiStreamEvent::ContentBlockDelta { index, delta } => match delta {
            ApiDelta::TextDelta { text } => StreamEvent::TextDelta { index, text },
            ApiDelta::InputJsonDelta { partial_json } => StreamEvent::InputJsonDelta {
                index,
                partial_json,
            },
            ApiDelta::ThinkingDelta { thinking } => StreamEvent::TextDelta {
                index,
                text: thinking,
            },
        },
        ApiStreamEvent::ContentBlockStop { index } => StreamEvent::ContentBlockStop { index },
        ApiStreamEvent::MessageDelta { delta } => StreamEvent::MessageDelta {
            stop_reason: delta.stop_reason.map(|s| match s.as_str() {
                "end_turn" => StopReason::EndTurn,
                "max_tokens" => StopReason::MaxTokens,
                "stop_sequence" => StopReason::StopSequence,
                "tool_use" => StopReason::ToolUse,
                _ => StopReason::EndTurn,
            }),
        },
        ApiStreamEvent::MessageStop => StreamEvent::MessageStop,
        ApiStreamEvent::Ping => StreamEvent::Ping,
        ApiStreamEvent::Error { error } => StreamEvent::Error {
            message: error.message,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = Claude::new("test-key");
        assert_eq!(client.model, DEFAULT_MODEL);
    }

    #[test]
    fn test_client_with_model() {
        let client = Claude::new("test-key").with_model("claude-3-opus");
        assert_eq!(client.model, "claude-3-opus");
    }

    #[test]
    fn test_request_builder() {
        let request = Request::new(vec![Message::user("Hello")])
            .with_system("You are a helpful assistant")
            .with_max_tokens(1000)
            .with_temperature(0.7);

        assert_eq!(request.max_tokens, 1000);
        assert!(request.system.is_some());
        assert_eq!(request.temperature, Some(0.7));
    }

    #[test]
    fn test_message_creation() {
        let user_msg = Message::user("Hello");
        assert!(matches!(user_msg.role, Role::User));
        assert_eq!(user_msg.content.len(), 1);

        let assistant_msg = Message::assistant("Hi there");
        assert!(matches!(assistant_msg.role, Role::Assistant));
    }

    #[test]
    fn test_tool_result() {
        let success = ToolResult::success("worked");
        assert!(!success.is_error);
        assert_eq!(success.content, "worked");

        let error = ToolResult::error("failed");
        assert!(error.is_error);
        assert_eq!(error.content, "failed");
    }
}
