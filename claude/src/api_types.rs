//! Internal Anthropic Messages API types.

use serde::{Deserialize, Serialize};

use crate::types::ContentBlock;

#[derive(Debug, Serialize)]
pub(crate) struct ApiRequest {
    pub model: String,
    pub max_tokens: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    pub messages: Vec<ApiMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ApiTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ApiToolChoice>,
    pub stream: bool,
}

#[derive(Debug, Serialize)]
pub(crate) struct ApiMessage {
    pub role: String,
    pub content: Vec<ApiContentBlock>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(crate) enum ApiContentBlock {
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

impl ApiContentBlock {
    pub(crate) fn from_content(block: &ContentBlock) -> Option<Self> {
        Some(match block {
            ContentBlock::Text { text } => Self::Text { text: text.clone() },
            ContentBlock::Image { data, media_type } => Self::Image {
                source: ApiImageSource {
                    r#type: "base64".to_string(),
                    media_type: media_type.clone(),
                    data: data.clone(),
                },
            },
            ContentBlock::ToolUse { id, name, input } => Self::ToolUse {
                id: id.clone(),
                name: name.clone(),
                input: input.clone(),
            },
            ContentBlock::ToolResult {
                tool_use_id,
                content,
                is_error,
            } => Self::ToolResult {
                tool_use_id: tool_use_id.clone(),
                content: content.clone(),
                is_error: *is_error,
            },
            // Neither type is ordinary assistant text. Anthropic extended
            // thinking replay requires signed provider-native blocks, which
            // this client does not request or synthesize.
            ContentBlock::Thinking { .. } | ContentBlock::ReasoningDetails { .. } => return None,
        })
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct ApiImageSource {
    pub r#type: String,
    pub media_type: String,
    pub data: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct ApiTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub(crate) struct ApiToolChoice {
    pub r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ApiResponse {
    pub id: String,
    pub model: String,
    pub content: Vec<ApiContent>,
    pub stop_reason: String,
    pub usage: ApiUsage,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(crate) enum ApiContent {
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
pub(crate) struct ApiUsage {
    pub input_tokens: usize,
    pub output_tokens: usize,
}
