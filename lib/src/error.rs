//! Error types for the agentic framework.
//!
//! Uses thiserror for ergonomic error definition.

use crate::id::{ActionId, AgentId, GoalId, MemoryId, PlanId, ToolCallId};

/// Main error type for the agentic framework
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Agent-related error
    #[error("Agent error: {0}")]
    Agent(#[from] AgentError),

    /// Tool-related error
    #[error("Tool error: {0}")]
    Tool(#[from] ToolError),

    /// Memory-related error
    #[error("Memory error: {0}")]
    Memory(#[from] MemoryError),

    /// Planning-related error
    #[error("Planning error: {0}")]
    Planning(#[from] PlanningError),

    /// Safety-related error
    #[error("Safety error: {0}")]
    Safety(#[from] SafetyError),

    /// Context-related error
    #[error("Context error: {0}")]
    Context(#[from] ContextError),

    /// LLM provider error
    #[error("LLM error: {0}")]
    Llm(#[from] LlmError),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Agent-specific errors
#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    /// Agent not found
    #[error("Agent not found: {0}")]
    NotFound(AgentId),

    /// Agent initialization failed
    #[error("Agent initialization failed: {reason}")]
    InitializationFailed { reason: String },

    /// Agent is not ready to process requests
    #[error("Agent not ready: {reason}")]
    NotReady { reason: String },

    /// Processing failed
    #[error("Processing failed: {reason}")]
    ProcessingFailed { reason: String },

    /// Maximum iterations exceeded
    #[error("Maximum iterations ({max}) exceeded")]
    MaxIterationsExceeded { max: usize },

    /// Agent capability not available
    #[error("Capability not available: {capability}")]
    CapabilityNotAvailable { capability: String },
}

/// Tool-specific errors
#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    /// Tool not found
    #[error("Tool not found: {name}")]
    NotFound { name: String },

    /// Invalid tool parameters
    #[error("Invalid parameters for tool {tool}: {reason}")]
    InvalidParameters { tool: String, reason: String },

    /// Tool execution failed
    #[error("Tool execution failed: {reason}")]
    ExecutionFailed {
        tool_call_id: ToolCallId,
        reason: String,
    },

    /// Tool timed out
    #[error("Tool execution timed out after {duration:?}")]
    Timeout {
        tool_call_id: ToolCallId,
        duration: std::time::Duration,
    },

    /// Tool permission denied
    #[error("Permission denied for tool {tool}: {reason}")]
    PermissionDenied { tool: String, reason: String },
}

/// Memory-specific errors
#[derive(Debug, thiserror::Error)]
pub enum MemoryError {
    /// Memory not found
    #[error("Memory not found: {0}")]
    NotFound(MemoryId),

    /// Storage error
    #[error("Storage error: {reason}")]
    StorageError { reason: String },

    /// Retrieval error
    #[error("Retrieval error: {reason}")]
    RetrievalError { reason: String },

    /// Capacity exceeded
    #[error("Memory capacity exceeded: {current}/{max}")]
    CapacityExceeded { current: usize, max: usize },

    /// Embedding error
    #[error("Embedding error: {reason}")]
    EmbeddingError { reason: String },

    /// Checkpoint error
    #[error("Checkpoint error: {reason}")]
    CheckpointError { reason: String },
}

/// Planning-specific errors
#[derive(Debug, thiserror::Error)]
pub enum PlanningError {
    /// Goal not found
    #[error("Goal not found: {0}")]
    GoalNotFound(GoalId),

    /// Plan not found
    #[error("Plan not found: {0}")]
    PlanNotFound(PlanId),

    /// Decomposition failed
    #[error("Goal decomposition failed: {reason}")]
    DecompositionFailed { goal: GoalId, reason: String },

    /// Plan verification failed
    #[error("Plan verification failed: {issues:?}")]
    VerificationFailed { plan: PlanId, issues: Vec<String> },

    /// Plan execution failed
    #[error("Plan execution failed at step {step}: {reason}")]
    ExecutionFailed {
        plan: PlanId,
        step: usize,
        reason: String,
    },

    /// Replanning limit exceeded
    #[error("Replanning limit ({max}) exceeded for plan {plan}")]
    ReplanLimitExceeded { plan: PlanId, max: usize },

    /// Goal conflict
    #[error("Goal conflict between {goals:?}: {conflict_type}")]
    GoalConflict {
        goals: Vec<GoalId>,
        conflict_type: String,
    },
}

/// Safety-specific errors
#[derive(Debug, thiserror::Error)]
pub enum SafetyError {
    /// Validation failed
    #[error("Safety validation failed: {reason}")]
    ValidationFailed { action: ActionId, reason: String },

    /// Guardrail triggered
    #[error("Guardrail '{guardrail}' triggered: {reason}")]
    GuardrailTriggered { guardrail: String, reason: String },

    /// Approval required
    #[error("Action requires approval: {reason}")]
    ApprovalRequired { action: ActionId, reason: String },

    /// Approval denied
    #[error("Approval denied: {reason}")]
    ApprovalDenied { action: ActionId, reason: String },

    /// Approval timeout
    #[error("Approval request timed out after {duration:?}")]
    ApprovalTimeout {
        action: ActionId,
        duration: std::time::Duration,
    },

    /// Constitutional violation
    #[error("Constitutional violation: {principle}")]
    ConstitutionalViolation { principle: String },
}

/// Context-specific errors
#[derive(Debug, thiserror::Error)]
pub enum ContextError {
    /// Context too large
    #[error("Context too large: {current} tokens exceeds {max} limit")]
    TooLarge { current: usize, max: usize },

    /// Compression failed
    #[error("Context compression failed: {reason}")]
    CompressionFailed { reason: String },

    /// Summarization failed
    #[error("Summarization failed: {reason}")]
    SummarizationFailed { reason: String },

    /// Retrieval failed
    #[error("Context retrieval failed: {reason}")]
    RetrievalFailed { reason: String },

    /// State error
    #[error("State error: {reason}")]
    StateError { reason: String },
}

/// LLM provider errors
#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    /// API error from provider
    #[error("API error ({status}): {message}")]
    Api { status: u16, message: String },

    /// Network/connection error
    #[error("Network error: {0}")]
    Network(String),

    /// Response parsing error
    #[error("Parse error: {0}")]
    Parse(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Rate limited
    #[error("Rate limited, retry after {retry_after:?}")]
    RateLimited {
        retry_after: Option<std::time::Duration>,
    },

    /// Authentication failed
    #[error("Authentication failed: {reason}")]
    AuthenticationFailed { reason: String },

    /// Model not available
    #[error("Model not available: {model}")]
    ModelNotAvailable { model: String },

    /// Request timeout
    #[error("Request timed out after {duration:?}")]
    Timeout { duration: std::time::Duration },
}

/// Result type alias for convenience
pub type Result<T> = std::result::Result<T, Error>;

/// Result type for agent operations
pub type AgentResult<T> = std::result::Result<T, AgentError>;

/// Result type for tool operations
pub type ToolResult<T> = std::result::Result<T, ToolError>;

/// Result type for memory operations
pub type MemoryResult<T> = std::result::Result<T, MemoryError>;

/// Result type for planning operations
pub type PlanningResult<T> = std::result::Result<T, PlanningError>;

/// Result type for safety operations
pub type SafetyResult<T> = std::result::Result<T, SafetyError>;

/// Result type for context operations
pub type ContextResult<T> = std::result::Result<T, ContextError>;

/// Result type for LLM operations
pub type LlmResult<T> = std::result::Result<T, LlmError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Error::Config("invalid setting".to_string());
        assert_eq!(err.to_string(), "Configuration error: invalid setting");
    }

    #[test]
    fn test_error_conversion() {
        let tool_err = ToolError::NotFound {
            name: "unknown".to_string(),
        };
        let err: Error = tool_err.into();
        assert!(matches!(err, Error::Tool(_)));
    }
}
