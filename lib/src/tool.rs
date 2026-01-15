//! Tool trait and tool-related types.
//!
//! Tools are executable functions that agents can invoke to interact with
//! external systems, perform computations, or gather information.

use crate::error::ToolError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::time::Duration;

/// Core tool trait - defines an executable function for agents
#[async_trait]
pub trait Tool: Send + Sync {
    /// Get the tool's unique name
    fn name(&self) -> &str;

    /// Get a human-readable description of what the tool does
    fn description(&self) -> &str;

    /// Get the JSON Schema for input parameters
    fn input_schema(&self) -> &Value;

    /// Get optional JSON Schema for output validation
    fn output_schema(&self) -> Option<&Value> {
        None
    }

    /// Get tool annotations (behavioral hints)
    fn annotations(&self) -> &ToolAnnotations {
        static DEFAULT: ToolAnnotations = ToolAnnotations::default_const();
        &DEFAULT
    }

    /// Execute the tool with the given parameters
    async fn execute(&self, params: Value, context: &ToolContext) -> Result<ToolOutput, ToolError>;

    /// Whether this tool is idempotent (safe to retry)
    fn is_idempotent(&self) -> bool {
        false
    }

    /// Estimated cost category for this tool
    fn cost_category(&self) -> CostCategory {
        CostCategory::Low
    }

    /// Validate input parameters before execution
    fn validate_params(&self, params: &Value) -> Result<(), ToolError> {
        // Default implementation does no validation
        let _ = params;
        Ok(())
    }
}

/// Annotations providing hints about tool behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolAnnotations {
    /// Tool may have destructive/irreversible effects
    pub destructive: bool,
    /// Tool only reads data, no side effects
    pub read_only: bool,
    /// Tool requires human approval before execution
    pub requires_approval: bool,
    /// Tool may be slow (> 10 seconds)
    pub slow: bool,
    /// Tool accesses external network
    pub network_access: bool,
    /// Tool accesses file system
    pub file_system_access: bool,
    /// Tool may incur financial cost
    pub has_cost: bool,
}

impl ToolAnnotations {
    /// Create default annotations (const fn for static use)
    pub const fn default_const() -> Self {
        Self {
            destructive: false,
            read_only: false,
            requires_approval: false,
            slow: false,
            network_access: false,
            file_system_access: false,
            has_cost: false,
        }
    }

    /// Create annotations for a read-only tool
    pub const fn read_only() -> Self {
        Self {
            destructive: false,
            read_only: true,
            requires_approval: false,
            slow: false,
            network_access: false,
            file_system_access: false,
            has_cost: false,
        }
    }

    /// Create annotations for a destructive tool
    pub const fn destructive() -> Self {
        Self {
            destructive: true,
            read_only: false,
            requires_approval: true,
            slow: false,
            network_access: false,
            file_system_access: false,
            has_cost: false,
        }
    }
}

impl Default for ToolAnnotations {
    fn default() -> Self {
        Self::default_const()
    }
}

/// Cost category for tools
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CostCategory {
    /// Free or negligible cost
    Free,
    /// Low cost (< $0.01)
    Low,
    /// Medium cost ($0.01 - $0.10)
    Medium,
    /// High cost (> $0.10)
    High,
}

/// Context provided to tools during execution
#[derive(Debug, Clone, Default)]
pub struct ToolContext {
    /// Current agent ID
    pub agent_id: Option<crate::id::AgentId>,
    /// Session/conversation ID
    pub session_id: Option<crate::id::SessionId>,
    /// Working directory for file operations
    pub working_directory: Option<String>,
    /// Environment variables
    pub environment: HashMap<String, String>,
    /// Timeout for this tool execution
    pub timeout: Option<Duration>,
    /// Whether to run in dry-run mode
    pub dry_run: bool,
}

impl ToolContext {
    /// Create a new tool context
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the working directory
    pub fn with_working_directory(mut self, dir: impl Into<String>) -> Self {
        self.working_directory = Some(dir.into());
        self
    }

    /// Set a timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Enable dry-run mode
    pub fn with_dry_run(mut self) -> Self {
        self.dry_run = true;
        self
    }

    /// Add an environment variable
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.environment.insert(key.into(), value.into());
        self
    }
}

/// Output from tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolOutput {
    /// The main content output
    pub content: String,
    /// Structured output (if applicable)
    pub structured: Option<Value>,
    /// Additional metadata
    pub metadata: ToolOutputMetadata,
}

impl ToolOutput {
    /// Create a text output
    pub fn text(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            structured: None,
            metadata: ToolOutputMetadata::default(),
        }
    }

    /// Create a structured output
    pub fn structured(content: impl Into<String>, data: Value) -> Self {
        Self {
            content: content.into(),
            structured: Some(data),
            metadata: ToolOutputMetadata::default(),
        }
    }

    /// Create an error output
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            content: message.into(),
            structured: None,
            metadata: ToolOutputMetadata {
                is_error: true,
                ..Default::default()
            },
        }
    }

    /// Add metadata
    pub fn with_metadata(mut self, metadata: ToolOutputMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// Check if this is an error output
    pub fn is_error(&self) -> bool {
        self.metadata.is_error
    }
}

impl fmt::Display for ToolOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.content)
    }
}

/// Metadata about tool output
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ToolOutputMetadata {
    /// Whether this represents an error
    pub is_error: bool,
    /// Whether output was truncated
    pub truncated: bool,
    /// Original size before truncation
    pub original_size: Option<usize>,
    /// Execution duration
    pub duration_ms: Option<u64>,
    /// Additional properties
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// Tool registry for managing available tools
#[derive(Default)]
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl fmt::Debug for ToolRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ToolRegistry")
            .field("tool_count", &self.tools.len())
            .field("tools", &self.tools.keys().collect::<Vec<_>>())
            .finish()
    }
}

impl ToolRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a tool
    pub fn register<T: Tool + 'static>(&mut self, tool: T) -> &mut Self {
        self.tools.insert(tool.name().to_string(), Arc::new(tool));
        self
    }

    /// Register a tool with Arc
    pub fn register_arc(&mut self, tool: Arc<dyn Tool>) -> &mut Self {
        self.tools.insert(tool.name().to_string(), tool);
        self
    }

    /// Get a tool by name
    pub fn get(&self, name: &str) -> Option<&Arc<dyn Tool>> {
        self.tools.get(name)
    }

    /// Check if a tool exists
    pub fn contains(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    /// Get all tool names
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.tools.keys().map(|s| s.as_str())
    }

    /// Get all tools
    pub fn all(&self) -> impl Iterator<Item = &Arc<dyn Tool>> {
        self.tools.values()
    }

    /// Number of registered tools
    pub fn len(&self) -> usize {
        self.tools.len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }

    /// Generate tool definitions for LLM
    pub fn tool_definitions(&self) -> Vec<ToolDefinition> {
        self.tools
            .values()
            .map(|t| ToolDefinition {
                name: t.name().to_string(),
                description: t.description().to_string(),
                input_schema: t.input_schema().clone(),
                annotations: t.annotations().clone(),
            })
            .collect()
    }
}

/// Tool definition for LLM consumption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: String,
    /// Input parameter schema
    pub input_schema: Value,
    /// Tool annotations
    #[serde(skip_serializing_if = "is_default_annotations")]
    pub annotations: ToolAnnotations,
}

fn is_default_annotations(annotations: &ToolAnnotations) -> bool {
    !annotations.destructive
        && !annotations.read_only
        && !annotations.requires_approval
        && !annotations.slow
        && !annotations.network_access
        && !annotations.file_system_access
        && !annotations.has_cost
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestTool;

    #[async_trait]
    impl Tool for TestTool {
        fn name(&self) -> &str {
            "test_tool"
        }

        fn description(&self) -> &str {
            "A test tool"
        }

        fn input_schema(&self) -> &Value {
            static SCHEMA: once_cell::sync::Lazy<Value> = once_cell::sync::Lazy::new(|| {
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "input": { "type": "string" }
                    }
                })
            });
            &SCHEMA
        }

        async fn execute(&self, params: Value, _context: &ToolContext) -> Result<ToolOutput, ToolError> {
            Ok(ToolOutput::text(format!("Executed with: {}", params)))
        }
    }

    #[test]
    fn test_tool_registry() {
        let mut registry = ToolRegistry::new();
        registry.register(TestTool);

        assert!(registry.contains("test_tool"));
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_tool_output() {
        let output = ToolOutput::text("Hello");
        assert!(!output.is_error());

        let error = ToolOutput::error("Something went wrong");
        assert!(error.is_error());
    }

    #[test]
    fn test_tool_context() {
        let ctx = ToolContext::new()
            .with_working_directory("/tmp")
            .with_timeout(Duration::from_secs(30))
            .with_env("KEY", "VALUE");

        assert_eq!(ctx.working_directory, Some("/tmp".to_string()));
        assert_eq!(ctx.timeout, Some(Duration::from_secs(30)));
        assert_eq!(ctx.environment.get("KEY"), Some(&"VALUE".to_string()));
    }
}
