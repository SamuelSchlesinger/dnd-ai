//! # Agentic
//!
//! A Rust framework for building AI agents with sophisticated planning, memory,
//! creativity, and safety capabilities.
//!
//! ## Core Concepts
//!
//! - **Agent**: The central abstraction that processes messages and produces responses
//! - **Tool**: Executable functions that agents can invoke
//! - **Memory**: Episodic, semantic, and procedural memory systems
//! - **Planning**: Goal management and task decomposition
//! - **Safety**: Guardrails, approval workflows, and audit logging
//!
//! ## Example
//!
//! ```rust,ignore
//! use agentic::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let agent = AgentBuilder::new()
//!         .name("assistant")
//!         .with_tool(FileReadTool::new())
//!         .with_guardrail(NoDestructiveActions)
//!         .build()?;
//!
//!     let response = agent.process(Message::user("Hello!"), &mut Context::new()).await?;
//!     println!("{}", response);
//!     Ok(())
//! }
//! ```

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

pub mod id;
pub mod message;
pub mod action;
pub mod error;
pub mod agent;
pub mod tool;
pub mod memory;
pub mod safety;
pub mod context;
pub mod llm;

/// Prelude for convenient imports
pub mod prelude {
    pub use crate::id::*;
    pub use crate::message::*;
    pub use crate::action::*;
    pub use crate::error::*;
    pub use crate::agent::{Agent, AgentMetadata, Capabilities, Context, Response};
    pub use crate::tool::{Tool, ToolOutput, ToolAnnotations, ToolContext, ToolRegistry};
    pub use crate::memory::{EpisodicMemory, SemanticMemory, ProceduralMemory, MemoryManager};
    pub use crate::safety::{SafetyValidator, Guardrail, SafetyResult};
    pub use crate::llm::{LlmProvider, CompletionRequest, CompletionResponse};
}
