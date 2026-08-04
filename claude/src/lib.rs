//! Provider-neutral LLM client for Chronicler.
//!
//! Supports Anthropic's Messages API and the OpenAI-compatible Chat
//! Completions protocol used by OpenAI, OpenRouter, Ollama, and other local
//! inference servers. Both transports support streaming and function tools.
//!
//! # Selecting a backend
//!
//! [`Client::from_env`] resolves Chronicler's provider environment variables.
//! Applications can instead construct an explicit client, which keeps provider
//! selection outside game logic:
//!
//! ```
//! use chronicler_llm::{Client, ClientConfig, LOCAL_DEFAULT_MODEL};
//!
//! # fn example() -> Result<(), chronicler_llm::Error> {
//! let local = Client::local(LOCAL_DEFAULT_MODEL)?;
//! let openrouter = Client::from_config(
//!     ClientConfig::openrouter("key")
//!         .with_model("moonshotai/kimi-k3")
//!         .with_fast_model("moonshotai/kimi-k3"),
//! )?;
//! # let _ = (local, openrouter);
//! # Ok(())
//! # }
//! ```
//!
//! OpenAI-compatible models must support JSON-schema function/tool calling and
//! tool-result continuation to run Chronicler; plain text compatibility alone
//! is insufficient.

mod api_types;
mod client;
mod config;
mod error;
mod openai;
mod streaming;
mod types;

pub use client::Client;
pub use config::{
    ClientConfig, ProviderKind, ANTHROPIC_BASE_URL, ANTHROPIC_DEFAULT_FAST_MODEL,
    ANTHROPIC_DEFAULT_MODEL, LOCAL_DEFAULT_MODEL, OLLAMA_BASE_URL, OPENAI_BASE_URL,
    OPENAI_DEFAULT_FAST_MODEL, OPENAI_DEFAULT_MODEL, OPENROUTER_BASE_URL, OPENROUTER_DEFAULT_MODEL,
};
pub use error::Error;
pub use types::{
    ContentBlock, Message, Request, Response, Role, StopReason, StreamEvent, Tool, ToolChoice,
    ToolResult, ToolUse, Usage,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_anthropic_client() {
        let client = Client::anthropic("test-key").expect("client");
        assert_eq!(client.provider(), ProviderKind::Anthropic);
        assert_eq!(client.model(), ANTHROPIC_DEFAULT_MODEL);
    }

    #[test]
    fn overrides_models_independently() {
        let client = Client::openrouter("test-key")
            .expect("client")
            .with_model("another/main")
            .with_fast_model("another/fast");
        assert_eq!(client.model(), "another/main");
        assert_eq!(client.fast_model(), "another/fast");
    }

    #[test]
    fn request_builder() {
        let request = Request::new(vec![Message::user("Hello")])
            .with_system("You are a helpful assistant")
            .with_max_tokens(1000)
            .with_temperature(0.7);

        assert_eq!(request.max_tokens, 1000);
        assert!(request.system.is_some());
        assert_eq!(request.temperature, Some(0.7));
    }

    #[test]
    fn message_creation() {
        let user = Message::user("Hello");
        assert!(matches!(user.role, Role::User));
        assert_eq!(user.content.len(), 1);

        let assistant = Message::assistant("Hi there");
        assert!(matches!(assistant.role, Role::Assistant));
    }

    #[test]
    fn tool_result() {
        let success = ToolResult::success("worked");
        assert!(!success.is_error);
        assert_eq!(success.content, "worked");

        let error = ToolResult::error("failed");
        assert!(error.is_error);
        assert_eq!(error.content, "failed");
    }
}
