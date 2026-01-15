# Agentic Framework Design

## Executive Summary

This document synthesizes research from multiple domains to define the architecture for the `agentic` Rust framework. The framework enables building AI agents with sophisticated planning, memory, creativity, safety, and tool use capabilities.

## Core Design Principles

Based on our research synthesis:

1. **Type-Safe by Design** - Leverage Rust's type system to enforce safety invariants at compile time
2. **Async-First** - Build on tokio for concurrent operations
3. **Trait-Based Extensibility** - Define clean trait interfaces for all major components
4. **Defense in Depth** - Multiple layers of validation and safety checks
5. **Memory Efficiency** - Arena allocation, interning, and efficient data structures
6. **Observable** - Built-in tracing, metrics, and audit logging

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                            AGENT RUNTIME                                 │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │   PLANNING   │  │   MEMORY     │  │  CREATIVITY  │  │    TOOLS     │ │
│  │              │  │              │  │              │  │              │ │
│  │ • HTN/GOAP   │  │ • Episodic   │  │ • Divergent  │  │ • Registry   │ │
│  │ • Goals      │  │ • Semantic   │  │ • Convergent │  │ • Execution  │ │
│  │ • Replanning │  │ • Procedural │  │ • Novelty    │  │ • Validation │ │
│  └──────────────┘  └──────────────┘  └──────────────┘  └──────────────┘ │
│                                                                          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │   CONTEXT    │  │   SAFETY     │  │  SECURITY    │  │   PRIVACY    │ │
│  │              │  │              │  │              │  │              │ │
│  │ • Window Mgmt│  │ • Guardrails │  │ • Sandboxing │  │ • PII Filter │ │
│  │ • RAG        │  │ • Approval   │  │ • Injection  │  │ • Consent    │ │
│  │ • State      │  │ • Audit      │  │ • Rate Limit │  │ • Compliance │ │
│  └──────────────┘  └──────────────┘  └──────────────┘  └──────────────┘ │
│                                                                          │
├─────────────────────────────────────────────────────────────────────────┤
│                         LLM PROVIDER LAYER                               │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │  Anthropic │ OpenAI │ Google │ Local Models │ Custom Providers     │ │
│  └────────────────────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────────────┤
│                         TRANSPORT LAYER (MCP)                            │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │  stdio │ HTTP/SSE │ WebSocket │ Custom Transports                  │ │
│  └────────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Core Traits

### 1. Agent Trait

```rust
/// Core agent trait - the central abstraction
#[async_trait]
pub trait Agent: Send + Sync {
    /// Unique identifier for this agent
    fn id(&self) -> &AgentId;

    /// Agent's name and description
    fn metadata(&self) -> &AgentMetadata;

    /// Process a message and produce a response
    async fn process(&self, message: Message, context: &mut Context) -> Result<Response, AgentError>;

    /// Available tools for this agent
    fn tools(&self) -> &[Arc<dyn Tool>];

    /// Agent's capabilities
    fn capabilities(&self) -> Capabilities;
}
```

### 2. Tool Trait

```rust
/// Tool interface following MCP patterns
#[async_trait]
pub trait Tool: Send + Sync {
    /// Unique name for the tool
    fn name(&self) -> &str;

    /// Human-readable description
    fn description(&self) -> &str;

    /// JSON Schema for input parameters
    fn input_schema(&self) -> &JsonSchema;

    /// Optional output schema for validation
    fn output_schema(&self) -> Option<&JsonSchema>;

    /// Execute the tool with given parameters
    async fn execute(&self, params: Value, context: &ToolContext) -> Result<ToolOutput, ToolError>;

    /// Tool annotations (hints for behavior)
    fn annotations(&self) -> &ToolAnnotations {
        &ToolAnnotations::default()
    }

    /// Whether this tool is idempotent (safe to retry)
    fn is_idempotent(&self) -> bool {
        false
    }
}

pub struct ToolAnnotations {
    /// Tool may have destructive effects
    pub destructive: bool,
    /// Tool is read-only
    pub read_only: bool,
    /// Tool requires human approval
    pub requires_approval: bool,
    /// Tool may be slow
    pub slow: bool,
    /// Estimated cost category
    pub cost: CostCategory,
}
```

### 3. Memory Traits

```rust
/// Episodic memory for specific experiences
#[async_trait]
pub trait EpisodicMemory: Send + Sync {
    async fn store(&self, experience: Experience) -> Result<MemoryId, MemoryError>;
    async fn retrieve(&self, query: &str, k: usize) -> Result<Vec<Experience>, MemoryError>;
    async fn retrieve_recent(&self, limit: usize) -> Result<Vec<Experience>, MemoryError>;
    async fn forget(&self, id: MemoryId) -> Result<(), MemoryError>;
}

/// Semantic memory for facts and knowledge
#[async_trait]
pub trait SemanticMemory: Send + Sync {
    async fn store_fact(&self, fact: Fact) -> Result<FactId, MemoryError>;
    async fn query_facts(&self, query: &str) -> Result<Vec<Fact>, MemoryError>;
    async fn update_fact(&self, id: FactId, update: FactUpdate) -> Result<(), MemoryError>;
}

/// Procedural memory for skills
#[async_trait]
pub trait ProceduralMemory: Send + Sync {
    async fn store_skill(&self, skill: Skill) -> Result<SkillId, MemoryError>;
    async fn retrieve_skill(&self, name: &str) -> Result<Option<Skill>, MemoryError>;
    async fn list_skills(&self) -> Result<Vec<SkillMetadata>, MemoryError>;
}

/// Unified memory manager
#[async_trait]
pub trait MemoryManager: Send + Sync {
    type Episodic: EpisodicMemory;
    type Semantic: SemanticMemory;
    type Procedural: ProceduralMemory;

    fn episodic(&self) -> &Self::Episodic;
    fn semantic(&self) -> &Self::Semantic;
    fn procedural(&self) -> &Self::Procedural;

    /// Recall relevant memories for a context
    async fn recall(&self, context: &Context) -> Result<RelevantMemories, MemoryError>;

    /// Consolidate memories (extract patterns, prune)
    async fn consolidate(&self) -> Result<ConsolidationReport, MemoryError>;

    /// Create a checkpoint
    async fn checkpoint(&self) -> Result<MemoryCheckpoint, MemoryError>;
}
```

### 4. Planning Traits

```rust
/// Goal representation
pub struct Goal {
    pub id: GoalId,
    pub description: String,
    pub priority: Priority,
    pub parent: Option<GoalId>,
    pub status: GoalStatus,
    pub deadline: Option<DateTime<Utc>>,
}

/// Plan representation
pub struct Plan {
    pub id: PlanId,
    pub goal: GoalId,
    pub steps: Vec<PlanStep>,
    pub dependencies: Vec<(StepIndex, StepIndex)>,
    pub status: PlanStatus,
}

/// Planner trait
#[async_trait]
pub trait Planner: Send + Sync {
    /// Generate a plan for a goal
    async fn plan(&self, goal: &Goal, context: &PlanningContext) -> Result<Plan, PlanError>;

    /// Verify a plan before execution
    async fn verify(&self, plan: &Plan) -> Result<PlanVerification, PlanError>;

    /// Replan when execution fails
    async fn replan(&self, plan: &Plan, failure: &ExecutionFailure) -> Result<Plan, PlanError>;
}

/// Goal manager trait
#[async_trait]
pub trait GoalManager: Send + Sync {
    async fn add_goal(&self, goal: Goal) -> Result<GoalId, GoalError>;
    async fn decompose(&self, goal: GoalId) -> Result<Vec<GoalId>, GoalError>;
    async fn prioritize(&self) -> Result<Vec<GoalId>, GoalError>;
    async fn resolve_conflicts(&self, goals: &[GoalId]) -> Result<ConflictResolution, GoalError>;
}
```

### 5. Safety Traits

```rust
/// Safety validator trait
#[async_trait]
pub trait SafetyValidator: Send + Sync {
    /// Validate an action before execution
    async fn validate(&self, action: &Action, context: &SafetyContext) -> SafetyResult;
}

/// Guardrail trait
pub trait Guardrail: Send + Sync {
    /// Check if an action passes this guardrail
    fn check(&self, action: &Action) -> GuardrailResult;

    /// Guardrail severity level
    fn severity(&self) -> Severity;

    /// Whether this guardrail can be overridden
    fn overridable(&self) -> bool;
}

/// Approval workflow
#[async_trait]
pub trait ApprovalWorkflow: Send + Sync {
    /// Request approval for an action
    async fn request_approval(&self, action: &Action) -> Result<ApprovalRequest, ApprovalError>;

    /// Wait for approval decision
    async fn await_decision(&self, request: &ApprovalRequest) -> Result<ApprovalDecision, ApprovalError>;
}

/// Type-safe action states
pub struct UnvalidatedAction(Action);
pub struct ValidatedAction(Action);
pub struct ApprovedAction(Action);

impl UnvalidatedAction {
    pub async fn validate(self, validator: &dyn SafetyValidator) -> Result<ValidatedAction, SafetyError> {
        match validator.validate(&self.0, &SafetyContext::default()).await {
            SafetyResult::Pass => Ok(ValidatedAction(self.0)),
            SafetyResult::Fail(reason) => Err(SafetyError::ValidationFailed(reason)),
        }
    }
}

impl ValidatedAction {
    pub async fn approve(self, workflow: &dyn ApprovalWorkflow) -> Result<ApprovedAction, ApprovalError> {
        let request = workflow.request_approval(&self.0).await?;
        match workflow.await_decision(&request).await? {
            ApprovalDecision::Approved => Ok(ApprovedAction(self.0)),
            ApprovalDecision::Denied(reason) => Err(ApprovalError::Denied(reason)),
        }
    }

    /// Skip approval if action doesn't require it
    pub fn auto_approve(self) -> ApprovedAction {
        ApprovedAction(self.0)
    }
}
```

### 6. Context Management Traits

```rust
/// Context window manager
#[async_trait]
pub trait ContextManager: Send + Sync {
    /// Build context for an interaction
    async fn build_context(&self, request: &ContextRequest) -> Result<Context, ContextError>;

    /// Compress context when approaching limits
    async fn compress(&self, context: &mut Context) -> Result<CompressionReport, ContextError>;

    /// Summarize a portion of context
    async fn summarize(&self, content: &str) -> Result<String, ContextError>;
}

/// RAG retriever trait
#[async_trait]
pub trait Retriever: Send + Sync {
    /// Retrieve relevant documents
    async fn retrieve(&self, query: &str, options: &RetrievalOptions) -> Result<Vec<Document>, RetrievalError>;
}

/// State persistence trait
#[async_trait]
pub trait StatePersistence: Send + Sync {
    /// Save state checkpoint
    async fn checkpoint(&self, state: &AgentState) -> Result<CheckpointId, PersistenceError>;

    /// Restore from checkpoint
    async fn restore(&self, id: CheckpointId) -> Result<AgentState, PersistenceError>;

    /// List available checkpoints
    async fn list_checkpoints(&self) -> Result<Vec<CheckpointMetadata>, PersistenceError>;
}
```

---

## Module Structure

```
lib/src/
├── lib.rs              # Crate root, re-exports
├── agent/
│   ├── mod.rs          # Agent trait and core types
│   ├── builder.rs      # AgentBuilder for configuration
│   ├── runtime.rs      # Agent execution runtime
│   └── error.rs        # Agent-related errors
├── tool/
│   ├── mod.rs          # Tool trait and registry
│   ├── executor.rs     # Tool execution with safety checks
│   ├── builtin/        # Built-in tools (file, shell, etc.)
│   └── mcp.rs          # MCP protocol integration
├── memory/
│   ├── mod.rs          # Memory traits
│   ├── episodic.rs     # Episodic memory implementation
│   ├── semantic.rs     # Semantic memory with knowledge graph
│   ├── procedural.rs   # Skill library
│   ├── working.rs      # Working memory manager
│   └── vector.rs       # Vector store integration
├── planning/
│   ├── mod.rs          # Planning traits
│   ├── htn.rs          # Hierarchical Task Network planner
│   ├── goap.rs         # Goal-Oriented Action Planning
│   ├── goals.rs        # Goal management
│   └── state.rs        # World state representation
├── context/
│   ├── mod.rs          # Context management traits
│   ├── window.rs       # Context window optimization
│   ├── summarizer.rs   # Progressive summarization
│   ├── rag.rs          # RAG integration
│   └── state.rs        # Conversation state
├── creativity/
│   ├── mod.rs          # Creativity module
│   ├── divergent.rs    # Divergent thinking / brainstorming
│   ├── convergent.rs   # Convergent evaluation
│   ├── novelty.rs      # Novelty search
│   └── blend.rs        # Conceptual blending
├── safety/
│   ├── mod.rs          # Safety traits and types
│   ├── guardrails.rs   # Guardrail implementations
│   ├── approval.rs     # Approval workflows
│   ├── constitution.rs # Constitutional constraints
│   └── audit.rs        # Audit logging
├── security/
│   ├── mod.rs          # Security module
│   ├── sandbox.rs      # Sandboxed execution
│   ├── injection.rs    # Injection prevention
│   └── rate_limit.rs   # Rate limiting
├── privacy/
│   ├── mod.rs          # Privacy module
│   ├── pii.rs          # PII detection and handling
│   ├── consent.rs      # Consent management
│   └── compliance.rs   # Regulatory compliance
├── llm/
│   ├── mod.rs          # LLM provider traits
│   ├── anthropic.rs    # Anthropic Claude integration
│   ├── openai.rs       # OpenAI integration
│   └── streaming.rs    # Streaming support
├── transport/
│   ├── mod.rs          # Transport traits
│   ├── stdio.rs        # stdio transport
│   └── http.rs         # HTTP/SSE transport
└── util/
    ├── mod.rs          # Utilities
    ├── id.rs           # ID types
    ├── token.rs        # Token counting
    └── retry.rs        # Retry logic
```

---

## Key Data Types

### Core IDs (Newtypes for Type Safety)

```rust
use uuid::Uuid;

macro_rules! define_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub struct $name(Uuid);

        impl $name {
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }

            pub fn from_uuid(uuid: Uuid) -> Self {
                Self(uuid)
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }
    };
}

define_id!(AgentId);
define_id!(ToolId);
define_id!(MemoryId);
define_id!(GoalId);
define_id!(PlanId);
define_id!(ActionId);
define_id!(CheckpointId);
```

### Message Types

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: MessageId,
    pub role: Role,
    pub content: Vec<ContentBlock>,
    pub timestamp: DateTime<Utc>,
    pub metadata: MessageMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Role {
    User,
    Assistant,
    System,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentBlock {
    Text { text: String },
    Image { data: String, media_type: String },
    ToolUse { id: ToolCallId, name: String, input: Value },
    ToolResult { tool_use_id: ToolCallId, content: String, is_error: bool },
}
```

### Action Pipeline Types

```rust
#[derive(Debug, Clone)]
pub struct Action {
    pub id: ActionId,
    pub action_type: ActionType,
    pub parameters: Value,
    pub source: ActionSource,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub enum ActionType {
    ToolCall { tool_name: String },
    Message { recipient: Option<AgentId> },
    StateChange { change_type: String },
    External { target: String },
}

#[derive(Debug, Clone)]
pub enum SafetyResult {
    Pass,
    Fail(SafetyViolation),
    RequiresApproval(ApprovalReason),
}

#[derive(Debug, Clone)]
pub struct SafetyViolation {
    pub guardrail: String,
    pub severity: Severity,
    pub description: String,
    pub suggested_alternative: Option<String>,
}
```

---

## Implementation Priorities

### Phase 1: Foundation (Core Traits & Types)
1. Define all core traits
2. Implement basic types (IDs, messages, actions)
3. Set up error handling patterns
4. Create basic agent runtime

### Phase 2: LLM Integration
1. Anthropic Claude provider
2. Tool execution loop
3. Streaming support
4. Basic context management

### Phase 3: Safety Layer
1. Guardrail system
2. Approval workflows
3. Audit logging
4. Input validation

### Phase 4: Memory System
1. Vector store integration (Qdrant)
2. Episodic memory
3. Working memory manager
4. Checkpointing

### Phase 5: Planning
1. Goal management
2. HTN planner
3. Re-planning on failure
4. State tracking

### Phase 6: Advanced Features
1. Multi-agent coordination
2. Creativity modules
3. Knowledge graphs
4. MCP protocol full support

---

## Configuration

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub id: AgentId,
    pub name: String,
    pub description: String,

    // LLM settings
    pub llm: LlmConfig,

    // Memory settings
    pub memory: MemoryConfig,

    // Planning settings
    pub planning: PlanningConfig,

    // Safety settings
    pub safety: SafetyConfig,

    // Context settings
    pub context: ContextConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub provider: LlmProvider,
    pub model: String,
    pub max_tokens: usize,
    pub temperature: f32,
    pub retry: RetryConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    pub episodic_capacity: usize,
    pub semantic_enabled: bool,
    pub procedural_enabled: bool,
    pub vector_store: VectorStoreConfig,
    pub persistence: PersistenceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyConfig {
    pub guardrails: Vec<GuardrailConfig>,
    pub require_approval_for: Vec<ActionCategory>,
    pub audit_level: AuditLevel,
    pub max_actions_per_turn: usize,
}
```

---

## Dependencies (Cargo.toml)

```toml
[dependencies]
# Async runtime
tokio = { version = "1", features = ["full"] }
async-trait = "0.1"
futures = "0.3"

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# HTTP client
reqwest = { version = "0.12", features = ["json", "stream"] }

# Types
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }

# Error handling
thiserror = "1"
anyhow = "1"

# Observability
tracing = "0.1"
tracing-subscriber = "0.3"

# Vector store
qdrant-client = "1"

# Utilities
smallvec = "1"
dashmap = "5"
parking_lot = "0.12"

[dev-dependencies]
tokio-test = "0.4"
proptest = "1"
criterion = "0.5"
```

---

## Next Steps

1. **Create core trait definitions** in `lib/src/`
2. **Implement basic agent runtime** with tool execution loop
3. **Add Anthropic provider** for LLM integration
4. **Build safety layer** with guardrails and validation
5. **Create example agents** in `agents/` directory

---

*This design synthesizes research from: Industry Survey (Anthropic, Google, OpenAI, Others), Planning, Continual Learning, Creativity, Context Management, Safety, Security, Privacy, and Miscellaneous domains.*
