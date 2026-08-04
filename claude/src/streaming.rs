//! SSE parsing for the Anthropic Messages API.

use serde::Deserialize;

use crate::error::Error;
use crate::types::{StopReason, StreamEvent};

// Internal streaming API types

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(crate) enum ApiStreamEvent {
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
pub(crate) struct ApiMessageStart {
    pub id: String,
    pub model: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ApiContentBlockStart {
    pub r#type: String,
    /// Tool use ID (present for tool_use blocks)
    #[serde(default)]
    pub id: Option<String>,
    /// Tool name (present for tool_use blocks)
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[allow(clippy::enum_variant_names)]
pub(crate) enum ApiDelta {
    TextDelta { text: String },
    InputJsonDelta { partial_json: String },
    ThinkingDelta { thinking: String },
}

#[derive(Debug, Deserialize)]
pub(crate) struct ApiMessageDelta {
    pub stop_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ApiError {
    pub message: String,
}

/// Parse SSE events from a buffer, consuming complete events and leaving incomplete data.
///
/// SSE events are separated by double newlines. This function finds complete events,
/// parses them, and removes them from the buffer, leaving any incomplete event data
/// for the next chunk.
pub(crate) fn parse_sse_events_buffered(buffer: &mut String) -> Vec<Result<StreamEvent, Error>> {
    let mut events = Vec::new();

    // Process complete SSE events (terminated by \n\n or at end of valid data lines)
    while let Some(newline_pos) = buffer.find('\n') {
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

pub(crate) fn convert_stream_event(event: ApiStreamEvent) -> StreamEvent {
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
            ApiDelta::ThinkingDelta { thinking } => StreamEvent::ThinkingDelta {
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
