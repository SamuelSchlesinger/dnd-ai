//! Provider-neutral request, response, and streaming types.

/// A completion request to send to an LLM provider.
///
/// Use builder methods to configure the request. At minimum, provide messages via [`Request::new`].
///
/// # Example
///
/// ```
/// use chronicler_llm::{Request, Message};
///
/// let request = Request::new(vec![Message::user("Hello!")])
///     .with_system("You are a helpful assistant.")
///     .with_max_tokens(1024);
/// ```
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
    /// Creates a new request with the given messages. Defaults to 4096 max tokens.
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

    /// Sets the model, overriding the client default.
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Sets the maximum number of tokens in the response.
    pub fn with_max_tokens(mut self, max_tokens: usize) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    /// Sets the system prompt that guides the model's behavior.
    pub fn with_system(mut self, system: impl Into<String>) -> Self {
        self.system = Some(system.into());
        self
    }

    /// Sets the sampling temperature (0.0-1.0). Higher values increase randomness.
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Adds tool definitions that the model can call during the conversation.
    pub fn with_tools(mut self, tools: Vec<Tool>) -> Self {
        self.tools = Some(tools);
        self
    }

    /// Configures how the model should choose tools.
    pub fn with_tool_choice(mut self, tool_choice: ToolChoice) -> Self {
        self.tool_choice = Some(tool_choice);
        self
    }
}

/// A message in the conversation.
///
/// Messages alternate between user and assistant roles. Use [`Message::user`]
/// and [`Message::assistant`] for simple text messages.
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
    /// Opaque provider reasoning metadata that must be replayed verbatim.
    ///
    /// OpenRouter uses this during multi-step tool interactions. Applications
    /// should preserve it in assistant history but must not display it as text.
    ReasoningDetails {
        details: Vec<serde_json::Value>,
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

/// A completion response from an LLM provider.
///
/// Contains generated content, usage statistics, and stop reason.
#[derive(Debug, Clone)]
pub struct Response {
    pub id: String,
    pub model: String,
    pub content: Vec<ContentBlock>,
    pub stop_reason: StopReason,
    pub usage: Usage,
}

impl Response {
    /// Gets all text content concatenated into a single string.
    ///
    /// Extracts and joins all text blocks, ignoring tool use and other content types.
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

/// A tool use request from the model.
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
    /// Provider-supplied reasoning text. Preserve it for tool continuation,
    /// but do not display it as narrative output.
    ThinkingDelta {
        index: usize,
        text: String,
    },
    /// Opaque OpenRouter reasoning details, in provider-supplied order.
    ReasoningDetails {
        details: Vec<serde_json::Value>,
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
