# Context Management in AI Agents: Research Report

## Executive Summary

Context management is one of the most critical challenges in building effective AI agents. As agents tackle increasingly complex tasks requiring multi-step reasoning, tool use, and long-running conversations, the ability to efficiently manage what information is available in the model's context window becomes paramount. This report synthesizes research and best practices across five key areas: context window optimization, summarization, retrieval-augmented generation (RAG), state management, and practical systems.

---

## 1. Context Window Optimization

### 1.1 The Fundamental Challenge

Modern LLMs have finite context windows (ranging from 8K to 200K+ tokens for various models). Agents often need to track:
- Conversation history
- Tool outputs (which can be verbose)
- System prompts and instructions
- Retrieved documents
- Working memory and scratch space

The challenge is fitting all relevant information while staying within limits.

### 1.2 Strategies for Fitting More Into Limited Context

#### **Priority-Based Context Selection**
The most effective systems assign priority levels to different context elements:

1. **Critical (Always Include)**
   - Core system prompt / persona
   - Current task instructions
   - Active tool call results
   - User's most recent message

2. **High Priority (Include When Possible)**
   - Recent conversation history (last N turns)
   - Relevant retrieved documents
   - Key state variables

3. **Medium Priority (Compress or Summarize)**
   - Older conversation history
   - Verbose tool outputs
   - Background information

4. **Low Priority (Drop When Necessary)**
   - Redundant information
   - Stale context
   - Previously resolved subproblems

#### **Dynamic Context Pruning**
Rather than static truncation, sophisticated systems use dynamic pruning:

- **Recency-weighted pruning**: More recent items are preserved over older ones
- **Relevance-weighted pruning**: Use embedding similarity to retain contextually relevant history
- **Task-aware pruning**: Different tasks need different context; coding tasks need code, conversations need dialogue history

**Key Insight**: The best pruning strategies are task-aware and use semantic relevance, not just recency.

#### **Context Compression Techniques**

1. **Token-Level Compression**
   - Remove filler words, redundant whitespace
   - Use abbreviations where unambiguous
   - Strip verbose formatting from tool outputs

2. **Semantic Compression**
   - Replace verbose explanations with key facts
   - Convert narrative descriptions to bullet points
   - Extract structured data from unstructured text

3. **Learned Compression (Research Frontier)**
   - AutoCompressors: Models trained to compress context into fewer tokens
   - Gist tokens: Learning to represent context as special tokens
   - ICAE (In-Context Autoencoder): Compressing long context into memory tokens

### 1.3 Recommended Patterns

```
ContextBudget {
    total_tokens: usize,
    reserved_system: usize,      // For system prompt (fixed)
    reserved_user: usize,        // For user input (variable)
    reserved_response: usize,    // For model response
    available_for_context: usize // Dynamically computed
}
```

**Pattern: Sliding Window with Anchors**
- Keep the first N messages (establishes context)
- Keep the last M messages (recent context)
- Summarize the middle
- Always preserve "anchor" messages marked as important

---

## 2. Summarization Strategies

### 2.1 Progressive Summarization

Progressive summarization creates increasingly condensed versions of context as it ages:

```
Level 0: Verbatim transcript (most recent)
Level 1: Key points extracted (1-5 turns old)
Level 2: Paragraph summary (5-20 turns old)
Level 3: One-line summary (20+ turns old)
Level 4: Tags/keywords only (very old, potentially dropped)
```

**Implementation Approach**:
- Maintain multiple summary levels per conversation segment
- Promote summaries to higher compression levels as context ages
- Allow "expansion" back to lower levels if topic becomes relevant again

### 2.2 Hierarchical Summarization

For very long contexts, hierarchical approaches work well:

```
Document/Conversation
├── Section 1 Summary
│   ├── Subsection 1.1 details
│   └── Subsection 1.2 details
├── Section 2 Summary
│   ├── Subsection 2.1 details
│   └── Subsection 2.2 details
└── Global Summary
```

**Tree-Based Summarization**:
1. Split content into chunks
2. Summarize each chunk
3. Recursively summarize summaries
4. Maintain tree structure for selective expansion

### 2.3 Preserving Critical Details While Compressing

Not all information compresses equally. Preserve:

- **Named entities**: People, places, organizations
- **Numerical data**: Dates, quantities, measurements
- **Decisions made**: Commitments, choices, conclusions
- **Tool invocations and results**: What was done and what happened
- **User corrections**: Explicit feedback that changed agent behavior

**Lossy vs Lossless Compression**:
- Some information can be regenerated (general knowledge) - lossy OK
- Some information is unique to this context (user data, decisions) - preserve losslessly

### 2.4 When to Summarize vs Retain Verbatim

**Retain Verbatim**:
- Current working context (code being edited, active documents)
- User's exact words for preference-sensitive tasks
- Tool outputs that may need re-examination
- Error messages and debugging information
- Anything the user might reference by quote

**Summarize**:
- Successfully completed subtasks
- Exploratory paths that were abandoned
- Verbose explanations after key facts extracted
- Repetitive patterns (loop N times → summarize as "did X N times")

---

## 3. Retrieval-Augmented Generation (RAG)

### 3.1 When to Use RAG vs In-Context

**Use In-Context When**:
- Information fits comfortably in context window
- High precision required (no retrieval errors)
- Information changes frequently within conversation
- Low latency critical

**Use RAG When**:
- Knowledge base exceeds context window
- Information is relatively static
- Queries are semantically clear
- Tolerance for occasional retrieval misses

**Hybrid Approaches (Recommended)**:
- Keep recent/critical info in-context
- Use RAG for background knowledge
- Cache frequent retrievals in context
- Use retrieval to decide what to load into context

### 3.2 Chunking Strategies

Chunking strategy dramatically affects RAG quality:

**Fixed-Size Chunking**:
```rust
struct FixedChunk {
    size: usize,        // e.g., 512 tokens
    overlap: usize,     // e.g., 50 tokens
}
```
- Simple, predictable
- May split semantic units

**Semantic Chunking**:
- Split on paragraph/section boundaries
- Keep code functions together
- Preserve logical units

**Recursive Chunking**:
- Start with large chunks
- Split if too large
- Maintain hierarchy

**Agentic Chunking** (Research Frontier):
- Use LLM to decide chunk boundaries
- Create chunks based on "propositions" or atomic facts
- More expensive but higher quality

### 3.3 Retrieval Algorithms for Agent Context

**Dense Retrieval (Embedding-Based)**:
- Embed query and documents
- Find nearest neighbors
- Fast, handles semantic similarity
- Can miss keyword matches

**Sparse Retrieval (BM25/TF-IDF)**:
- Keyword matching
- Handles exact terms well
- Misses paraphrases

**Hybrid Retrieval (Recommended)**:
```rust
struct HybridRetriever {
    dense_weight: f32,   // e.g., 0.7
    sparse_weight: f32,  // e.g., 0.3
    reranker: Option<Reranker>,
}
```

**Multi-Stage Retrieval**:
1. Fast first-stage retrieval (get top 100)
2. Reranking with more expensive model (get top 10)
3. Final selection based on context budget

### 3.4 Agent-Specific RAG Considerations

Agents have unique retrieval needs:

1. **Tool Documentation Retrieval**: Retrieve relevant tool docs based on current task
2. **Example Retrieval**: Find similar past problems/solutions
3. **Conversation History Retrieval**: Search past turns for relevant context
4. **Error Pattern Retrieval**: Find similar errors and solutions

**Self-RAG Pattern**:
- Agent decides when to retrieve
- Agent formulates retrieval query
- Agent evaluates retrieval results
- Agent decides what to include in context

---

## 4. State Management

### 4.1 Conversation State Across Turns

**Explicit State Tracking**:
```rust
struct ConversationState {
    turn_number: u32,
    current_task: Option<Task>,
    completed_tasks: Vec<TaskSummary>,
    pending_questions: Vec<String>,
    user_preferences: HashMap<String, Value>,
    working_memory: HashMap<String, Value>,
}
```

**State Serialization to Context**:
```
<state>
Current task: Implementing user authentication
Completed: [Database schema, User model]
Pending questions: [OAuth provider preference?]
Working memory: {current_file: "auth.rs", line: 42}
</state>
```

### 4.2 Long-Running Agent State

For agents that run extended tasks:

**Working Memory Pattern**:
- Short-term: Current step, immediate context
- Medium-term: Current task progress, relevant findings
- Long-term: User preferences, learned patterns

**Scratchpad Pattern**:
```
<scratchpad>
Observations:
- User prefers async Rust patterns
- Previous error was type mismatch
- Need to check auth middleware

Next steps:
1. Fix type annotation on line 42
2. Add middleware to router
3. Test with mock user

Open questions:
- Should sessions be stored in Redis or memory?
</scratchpad>
```

### 4.3 Checkpointing and Recovery

For reliability in long-running agents:

**Checkpoint Structure**:
```rust
struct AgentCheckpoint {
    id: Uuid,
    timestamp: DateTime<Utc>,
    conversation_history: Vec<Message>,
    current_state: AgentState,
    pending_actions: Vec<Action>,
    tool_state: HashMap<String, ToolState>,
    version: u32,
}
```

**Recovery Strategy**:
1. Load most recent valid checkpoint
2. Reconstruct minimal context needed
3. Summarize any lost history
4. Resume with explicit acknowledgment

### 4.4 Multi-Session Continuity

For agents that span multiple sessions:

**Session Resume Pattern**:
```rust
struct SessionResume {
    previous_session_summary: String,
    key_decisions: Vec<Decision>,
    unfinished_tasks: Vec<Task>,
    user_context: UserContext,
}
```

**Implementation**:
- At session end: Generate structured summary
- At session start: Load summary into context
- Explicitly acknowledge: "Last time we were working on X..."

---

## 5. Practical Systems Analysis

### 5.1 Claude Code's Context Management

Based on observed behavior and documentation, Claude Code employs:

**Conversation Compaction**:
- Monitors context usage as conversation grows
- Automatically summarizes older parts of conversation
- Preserves recent turns verbatim
- Creates "conversation summary" when approaching limits

**Tool Output Management**:
- Large tool outputs (file contents, search results) are primary context consumers
- Implements truncation for very large outputs
- Uses intelligent summarization for repetitive patterns

**File Context Strategy**:
- Reads files on-demand rather than preloading
- Caches recently accessed file contents
- Uses file headers/summaries when full content not needed

**Prompt Caching**:
- System prompt cached across turns
- Reduces token costs for repeated context

### 5.2 LangChain Memory Modules

LangChain provides several memory abstractions:

**ConversationBufferMemory**:
- Stores complete conversation history
- Simple but doesn't scale

**ConversationBufferWindowMemory**:
- Keeps last K turns
- Loses long-term context

**ConversationSummaryMemory**:
- Maintains running summary
- Updates summary each turn
- Loses specific details

**ConversationSummaryBufferMemory**:
- Hybrid: summary + recent buffer
- Best of both approaches

**VectorStoreRetrieverMemory**:
- Stores all messages in vector store
- Retrieves relevant context per query
- Scales well but adds latency

**Entity Memory**:
- Tracks entities mentioned in conversation
- Maintains entity-specific summaries
- Good for relationship tracking

### 5.3 MemGPT's Approach

MemGPT (Memory-GPT) introduces a virtual memory hierarchy inspired by operating systems:

**Core Concepts**:

1. **Main Context (In-Context)**
   - Limited, precious resource
   - Contains system prompt, recent messages, active data

2. **Archival Storage (External Database)**
   - Unlimited capacity
   - Vector database for semantic search
   - Stores conversation history, documents, facts

3. **Recall Storage (Conversation Database)**
   - Complete conversation history
   - Searchable by recency and relevance

**Memory Operations** (Exposed as Tools):
- `core_memory_append`: Add to main context
- `core_memory_replace`: Update main context
- `archival_memory_insert`: Store for later retrieval
- `archival_memory_search`: Retrieve from archives
- `conversation_search`: Search past conversations

**Self-Directed Memory Management**:
- Agent explicitly manages its own memory
- Decides what to keep in context vs archive
- Can retrieve archived information when needed

**Key Innovation**: Rather than automatic memory management, MemGPT gives the agent explicit control over its memory operations, making memory a first-class tool.

---

## 6. Recommended Patterns for Agentic Framework

### 6.1 Context Manager Architecture

```rust
/// Central context management for agents
pub struct ContextManager {
    /// Maximum tokens available
    max_tokens: usize,

    /// Token counter (model-specific)
    tokenizer: Box<dyn Tokenizer>,

    /// Priority-ordered context segments
    segments: Vec<ContextSegment>,

    /// Summarization strategy
    summarizer: Box<dyn Summarizer>,

    /// Optional retrieval system
    retriever: Option<Box<dyn Retriever>>,
}

pub struct ContextSegment {
    /// Unique identifier
    id: SegmentId,

    /// Content of this segment
    content: String,

    /// Priority level (higher = more important)
    priority: u8,

    /// Can this segment be compressed?
    compressible: bool,

    /// Compressed version if available
    compressed: Option<String>,

    /// Token count
    tokens: usize,

    /// When this segment was created
    created_at: Instant,

    /// When this segment was last accessed
    last_accessed: Instant,

    /// Segment type for specialized handling
    segment_type: SegmentType,
}

pub enum SegmentType {
    SystemPrompt,
    ConversationTurn { role: Role },
    ToolOutput { tool_name: String },
    RetrievedDocument { source: String },
    WorkingMemory,
    Summary { summarizes: Vec<SegmentId> },
}
```

### 6.2 Hierarchical Memory System

```rust
/// Three-tier memory system inspired by MemGPT
pub struct HierarchicalMemory {
    /// In-context memory (limited, fast)
    working_memory: WorkingMemory,

    /// Session memory (larger, still fast)
    session_memory: SessionMemory,

    /// Persistent memory (unlimited, retrieval-based)
    long_term_memory: LongTermMemory,
}

pub struct WorkingMemory {
    /// Current context segments
    segments: Vec<ContextSegment>,

    /// Maximum tokens
    capacity: usize,

    /// Current usage
    used: usize,
}

pub struct SessionMemory {
    /// Full conversation history
    conversation: Vec<Message>,

    /// Current state
    state: AgentState,

    /// Temporary scratchpad
    scratchpad: HashMap<String, Value>,
}

pub struct LongTermMemory {
    /// Vector store for semantic search
    vector_store: Box<dyn VectorStore>,

    /// Structured storage for entities
    entity_store: EntityStore,

    /// Checkpoints for recovery
    checkpoints: Vec<Checkpoint>,
}
```

### 6.3 Summarization Pipeline

```rust
pub trait Summarizer: Send + Sync {
    /// Summarize content to target token count
    fn summarize(
        &self,
        content: &str,
        target_tokens: usize,
        context: &SummarizationContext,
    ) -> impl Future<Output = Result<String, SummarizationError>>;

    /// Check if content should be summarized
    fn should_summarize(&self, segment: &ContextSegment) -> bool;
}

pub struct SummarizationContext {
    /// What type of content is being summarized
    content_type: ContentType,

    /// What must be preserved
    preserve: Vec<PreservationRule>,

    /// Current task context for relevance
    current_task: Option<String>,
}

pub enum PreservationRule {
    /// Keep named entities
    NamedEntities,
    /// Keep numerical values
    Numbers,
    /// Keep specific keywords
    Keywords(Vec<String>),
    /// Keep decisions/conclusions
    Decisions,
    /// Custom extraction
    Custom(Box<dyn Fn(&str) -> Vec<String>>),
}
```

### 6.4 Retrieval Integration

```rust
pub trait Retriever: Send + Sync {
    /// Retrieve relevant context
    fn retrieve(
        &self,
        query: &str,
        options: RetrievalOptions,
    ) -> impl Future<Output = Result<Vec<RetrievedItem>, RetrievalError>>;

    /// Index new content
    fn index(
        &self,
        content: &str,
        metadata: IndexMetadata,
    ) -> impl Future<Output = Result<(), RetrievalError>>;
}

pub struct RetrievalOptions {
    /// Maximum items to retrieve
    max_items: usize,

    /// Maximum tokens total
    max_tokens: Option<usize>,

    /// Minimum relevance score
    min_score: f32,

    /// Filter by metadata
    filters: Vec<MetadataFilter>,

    /// Retrieval strategy
    strategy: RetrievalStrategy,
}

pub enum RetrievalStrategy {
    /// Pure vector similarity
    Dense,
    /// Keyword matching
    Sparse,
    /// Combined with weights
    Hybrid { dense_weight: f32 },
    /// Multi-stage with reranking
    MultiStage { first_stage_k: usize },
}
```

### 6.5 State Persistence

```rust
/// Serializable agent state for checkpointing
#[derive(Serialize, Deserialize)]
pub struct AgentCheckpoint {
    /// Unique checkpoint ID
    pub id: Uuid,

    /// When checkpoint was created
    pub timestamp: DateTime<Utc>,

    /// Schema version for forward compatibility
    pub version: u32,

    /// Conversation state
    pub conversation: ConversationCheckpoint,

    /// Agent-specific state
    pub agent_state: serde_json::Value,

    /// Active tool states
    pub tool_states: HashMap<String, serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
pub struct ConversationCheckpoint {
    /// Summary of older conversation
    pub summary: Option<String>,

    /// Recent messages (verbatim)
    pub recent_messages: Vec<Message>,

    /// Key decisions made
    pub decisions: Vec<Decision>,

    /// Unresolved questions
    pub pending_questions: Vec<String>,
}
```

---

## 7. Rust Design Considerations

### 7.1 Data Structures

**Token-Aware Strings**:
```rust
/// String that caches its token count
pub struct TokenString {
    content: String,
    token_count: usize,
    tokenizer_id: &'static str,
}

impl TokenString {
    pub fn new(content: String, tokenizer: &dyn Tokenizer) -> Self {
        let token_count = tokenizer.count(&content);
        Self {
            content,
            token_count,
            tokenizer_id: tokenizer.id(),
        }
    }

    pub fn tokens(&self) -> usize {
        self.token_count
    }
}
```

**Efficient Segment Storage**:
```rust
/// Arena-allocated context segments for efficient memory use
pub struct ContextArena {
    /// Segment storage
    segments: Vec<ContextSegment>,

    /// Free list for reuse
    free_list: Vec<usize>,

    /// Index by ID
    id_index: HashMap<SegmentId, usize>,
}
```

### 7.2 Serialization Strategy

Use `serde` with careful attention to:

1. **Version compatibility**: Include schema version in serialized state
2. **Compact formats**: Use `bincode` or `messagepack` for checkpoints
3. **Human-readable**: Use JSON for debugging/inspection
4. **Streaming**: Support incremental serialization for large states

```rust
/// Checkpoint storage with format flexibility
pub trait CheckpointStore {
    fn save(&self, checkpoint: &AgentCheckpoint, format: Format)
        -> Result<(), StoreError>;
    fn load(&self, id: Uuid) -> Result<AgentCheckpoint, StoreError>;
    fn list(&self, filter: CheckpointFilter) -> Result<Vec<CheckpointMeta>, StoreError>;
}

pub enum Format {
    Json,      // Human readable, debugging
    Bincode,   // Fast, compact
    MessagePack, // Compact, cross-language
}
```

### 7.3 Async Considerations

Context management operations should be async-friendly:

```rust
impl ContextManager {
    /// Build context for next turn (may involve retrieval, summarization)
    pub async fn build_context(&mut self, budget: TokenBudget) -> Result<Context, Error> {
        // Parallel retrieval and summarization
        let (retrieved, summaries) = tokio::join!(
            self.retrieve_relevant_context(&budget),
            self.summarize_if_needed(&budget),
        );

        // Assemble final context
        self.assemble(retrieved?, summaries?, budget)
    }
}
```

### 7.4 Memory Efficiency

```rust
/// Copy-on-write segments for efficient cloning
pub struct CowSegment {
    content: Arc<str>,
    compressed: Option<Arc<str>>,
}

/// Lazy summarization - only compute when needed
pub struct LazySummary {
    original: Arc<str>,
    summary: OnceCell<Arc<str>>,
    summarizer: Arc<dyn Summarizer>,
}
```

---

## 8. Open Research Questions

### 8.1 Fundamental Questions

1. **Optimal Compression-Fidelity Tradeoff**
   - How much can we compress without losing task-relevant information?
   - Can we measure "information loss" for specific tasks?

2. **Self-Aware Context Management**
   - Should agents explicitly reason about their context?
   - How to train agents to manage their own memory effectively?

3. **Cross-Model Context Transfer**
   - Can context prepared for one model work for another?
   - How to abstract context representation from specific tokenizers?

### 8.2 Practical Questions

1. **Automatic Priority Assignment**
   - How to automatically determine what's important?
   - Learning priority from user behavior?

2. **Retrieval Quality for Agents**
   - Standard RAG benchmarks don't capture agent needs
   - How to evaluate retrieval for multi-step tasks?

3. **Graceful Degradation**
   - How should agents behave when context is insufficient?
   - When to ask for clarification vs. proceed with incomplete info?

### 8.3 Research Frontier

1. **Learned Compression**
   - Can we train "context compressor" models?
   - What's the theoretical limit of compression?

2. **Structured Memory**
   - Beyond text: knowledge graphs, databases
   - How to integrate structured retrieval with LLMs?

3. **Multi-Agent Context Sharing**
   - How do agents share relevant context efficiently?
   - Privacy-preserving context sharing

---

## 9. Summary and Recommendations

### Key Takeaways

1. **Context is the bottleneck**: For capable agents, context management is often the limiting factor, not model capability.

2. **Hybrid approaches win**: The best systems combine multiple strategies - summarization AND retrieval AND priority-based selection.

3. **Explicit over implicit**: Making context management explicit (like MemGPT's approach) gives more control and transparency.

4. **Task-awareness matters**: Generic context management is worse than task-specific strategies.

5. **Compression is lossy**: All compression loses information; the key is losing the right information.

### Recommended Architecture for Agentic Framework

1. **Implement hierarchical memory**: Working memory + session memory + long-term memory

2. **Build modular summarization**: Pluggable summarizers for different content types

3. **Include retrieval as first-class**: Optional but integrated RAG support

4. **Support checkpointing natively**: Make agent state serializable from day one

5. **Expose context budget**: Let agents reason about their context constraints

6. **Provide observability**: Make context decisions visible for debugging and tuning

### Priority Implementation Order

1. **Core ContextManager** with priority-based selection
2. **Token counting** and budget management
3. **Checkpointing** for reliability
4. **Summarization pipeline** with LLM-based summarizers
5. **Retrieval integration** for scaling beyond context limits
6. **Advanced patterns** (MemGPT-style self-management)

---

## References and Further Reading

### Papers
- "MemGPT: Towards LLMs as Operating Systems" (Packer et al., 2023)
- "Lost in the Middle: How Language Models Use Long Contexts" (Liu et al., 2023)
- "LongLoRA: Efficient Fine-tuning of Long-Context Large Language Models" (Chen et al., 2023)
- "Walking Down the Memory Maze: Beyond Context Limit through Interactive Reading" (2024)

### Documentation
- Anthropic: Prompt Caching, Extended Context
- LangChain: Memory modules documentation
- LlamaIndex: Retrieval and indexing strategies

### Open Source Implementations
- MemGPT/Letta: https://github.com/letta-ai/letta
- LangChain: https://github.com/langchain-ai/langchain
- LlamaIndex: https://github.com/run-llama/llama_index
