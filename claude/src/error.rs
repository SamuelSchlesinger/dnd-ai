//! Error types for the LLM client.

use thiserror::Error;

/// Errors that can occur when using the LLM client.
#[derive(Debug, Error)]
pub enum Error {
    #[error("No LLM provider is configured")]
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
