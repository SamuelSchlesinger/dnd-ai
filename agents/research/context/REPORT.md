# Context Management Research Report

## Executive Summary

This report synthesizes research across four critical domains of context management for LLM-based agentic systems: context window optimization, progressive summarization, retrieval-augmented generation (RAG), and state persistence. The findings inform the architecture of a robust context management system implementable in Rust.

---

## 1. Context Management Architecture

### 1.1 Layered Context Model

A production-ready context management system should implement a multi-layer architecture:

```
+--------------------------------------------------+
|              Active Context Window               |
|  (System Prompt + Current Task + Recent History) |
+--------------------------------------------------+
|              Working Memory Layer                |
|    (Session State + Active Entities + Goals)     |
+--------------------------------------------------+
|             Summarized History Layer             |
|  (Progressive Summaries + Key Decision Points)   |
+--------------------------------------------------+
|              Long-term Memory (RAG)              |
|   (Vector Store + Semantic Search + Archives)    |
+--------------------------------------------------+
|              Persistent State Store              |
|    (Checkpoints + Serialized State + Logs)       |
+--------------------------------------------------+
```

### 1.2 Token Budget Allocation Strategy

For a 128K context window, recommended allocation:

| Component | Token Budget | Purpose |
|-----------|-------------|---------|
| System Prompt | 2K-4K (2-3%) | Core instructions, persona |
| Tool Definitions | 4K-8K (3-6%) | Available capabilities |
| Current Task | 8K-16K (6-12%) | Active problem context |
| Recent Messages | 16K-32K (12-25%) | Immediate conversation |
| Working Memory | 8K-16K (6-12%) | Active entities, goals |
| Retrieved Context | 16K-32K (12-25%) | RAG results |
| Summarized History | 8K-16K (6-12%) | Compressed past |
| Response Buffer | 16K-32K (12-25%) | Generation space |

### 1.3 Priority Selection Algorithm

```rust
#[derive(Debug, Clone)]
pub struct ContextItem {
    pub id: Uuid,
    pub content: String,
    pub token_count: usize,
    pub priority_score: f32,
    pub item_type: ContextItemType,
    pub timestamp: DateTime<Utc>,
    pub access_count: u32,
    pub relevance_embedding: Vec<f32>,
}

impl ContextItem {
    /// Calculate priority score using weighted factors
    pub fn calculate_priority(&mut self, current_query: &[f32], now: DateTime<Utc>) {
        let recency_score = self.recency_decay(now);
        let relevance_score = self.semantic_similarity(current_query);
        let importance_score = self.intrinsic_importance();
        let access_score = (self.access_count as f32).ln_1p() / 10.0;

        // Weighted combination
        self.priority_score =
            0.35 * relevance_score +
            0.25 * recency_score +
            0.25 * importance_score +
            0.15 * access_score;
    }

    fn recency_decay(&self, now: DateTime<Utc>) -> f32 {
        let hours_elapsed = (now - self.timestamp).num_hours() as f32;
        (-hours_elapsed / 24.0).exp() // Exponential decay, half-life ~17 hours
    }

    fn semantic_similarity(&self, query: &[f32]) -> f32 {
        // Cosine similarity with current query embedding
        cosine_similarity(&self.relevance_embedding, query)
    }

    fn intrinsic_importance(&self) -> f32 {
        match self.item_type {
            ContextItemType::SystemPrompt => 1.0,
            ContextItemType::UserGoal => 0.95,
            ContextItemType::ActiveTask => 0.9,
            ContextItemType::ToolResult => 0.7,
            ContextItemType::Observation => 0.6,
            ContextItemType::ConversationHistory => 0.5,
            ContextItemType::Summary => 0.4,
        }
    }
}
```

---

## 2. Context Window Optimization

### 2.1 Compression Techniques

#### 2.1.1 Prompt Compression (LLMLingua-style)

Selective token removal based on perplexity:

```rust
pub struct PromptCompressor {
    /// Perplexity threshold for token removal
    perplexity_threshold: f32,
    /// Target compression ratio
    target_ratio: f32,
    /// Tokens to never remove
    protected_patterns: Vec<Regex>,
}

impl PromptCompressor {
    pub fn compress(&self, tokens: &[Token], perplexities: &[f32]) -> Vec<Token> {
        let mut result = Vec::new();
        let target_len = (tokens.len() as f32 * self.target_ratio) as usize;

        // Sort by perplexity (higher = more important)
        let mut scored: Vec<_> = tokens.iter()
            .zip(perplexities.iter())
            .enumerate()
            .collect();

        scored.sort_by(|a, b| b.1.1.partial_cmp(a.1.1).unwrap());

        // Keep highest perplexity tokens up to target length
        let mut kept_indices: Vec<_> = scored.iter()
            .take(target_len)
            .map(|(idx, _)| *idx)
            .collect();

        kept_indices.sort(); // Restore original order

        for idx in kept_indices {
            result.push(tokens[idx].clone());
        }

        result
    }
}
```

#### 2.1.2 Semantic Compression

Merge semantically similar content:

```rust
pub struct SemanticCompressor {
    similarity_threshold: f32,
    embedding_model: Box<dyn EmbeddingModel>,
}

impl SemanticCompressor {
    pub async fn compress_similar(&self, items: Vec<ContextItem>) -> Vec<ContextItem> {
        let embeddings: Vec<_> = items.iter()
            .map(|i| &i.relevance_embedding)
            .collect();

        let clusters = self.cluster_by_similarity(&embeddings);

        clusters.into_iter()
            .map(|cluster| self.merge_cluster(cluster, &items))
            .collect()
    }

    fn merge_cluster(&self, indices: Vec<usize>, items: &[ContextItem]) -> ContextItem {
        // Combine content, keep highest priority metadata
        let merged_content = indices.iter()
            .map(|&i| items[i].content.as_str())
            .collect::<Vec<_>>()
            .join("\n---\n");

        let max_priority = indices.iter()
            .map(|&i| items[i].priority_score)
            .fold(f32::NEG_INFINITY, f32::max);

        ContextItem {
            content: merged_content,
            priority_score: max_priority,
            // ... other fields
        }
    }
}
```

### 2.2 Pruning Strategies

#### 2.2.1 Sliding Window with Importance Retention

```rust
pub struct SlidingWindowPruner {
    window_size: usize,
    retention_buffer: usize,  // Extra space for high-priority items
}

impl SlidingWindowPruner {
    pub fn prune(&self, items: &mut Vec<ContextItem>, current_tokens: usize, max_tokens: usize) {
        if current_tokens <= max_tokens {
            return;
        }

        let tokens_to_remove = current_tokens - max_tokens + self.retention_buffer;
        let mut removed = 0;

        // Sort by priority (ascending, so lowest first)
        let mut indices_by_priority: Vec<_> = (0..items.len())
            .filter(|&i| items[i].item_type != ContextItemType::SystemPrompt)
            .collect();

        indices_by_priority.sort_by(|&a, &b|
            items[a].priority_score.partial_cmp(&items[b].priority_score).unwrap()
        );

        let mut to_remove = Vec::new();
        for idx in indices_by_priority {
            if removed >= tokens_to_remove {
                break;
            }
            to_remove.push(idx);
            removed += items[idx].token_count;
        }

        // Remove in reverse order to preserve indices
        to_remove.sort_by(|a, b| b.cmp(a));
        for idx in to_remove {
            items.remove(idx);
        }
    }
}
```

#### 2.2.2 Conversation Turn Consolidation

```rust
pub struct TurnConsolidator {
    max_turns_full: usize,      // Keep this many recent turns in full
    max_turns_summary: usize,   // Summarize this many older turns
    summarizer: Box<dyn Summarizer>,
}

impl TurnConsolidator {
    pub async fn consolidate(&self, turns: Vec<ConversationTurn>) -> ConsolidatedContext {
        let total_turns = turns.len();

        if total_turns <= self.max_turns_full {
            return ConsolidatedContext {
                full_turns: turns,
                summarized_turns: None,
            };
        }

        let summary_end = total_turns.saturating_sub(self.max_turns_full);
        let summary_start = summary_end.saturating_sub(self.max_turns_summary);

        let to_summarize = &turns[summary_start..summary_end];
        let to_keep = turns[total_turns - self.max_turns_full..].to_vec();

        let summary = self.summarizer.summarize_turns(to_summarize).await;

        ConsolidatedContext {
            full_turns: to_keep,
            summarized_turns: Some(summary),
        }
    }
}
```

---

## 3. Progressive Summarization

### 3.1 Hierarchical Summary Architecture

```
Level 0: Raw conversation turns
    |
    v (every 5-10 turns)
Level 1: Turn-level summaries (1-2 sentences per turn)
    |
    v (every 3-5 L1 summaries)
Level 2: Segment summaries (key decisions, outcomes)
    |
    v (end of session)
Level 3: Session summaries (goals achieved, state changes)
    |
    v (multiple sessions)
Level 4: Archival summaries (long-term memory)
```

### 3.2 Summary Preservation Strategy

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreservationTags {
    pub entities: Vec<Entity>,           // Named entities to preserve
    pub decisions: Vec<Decision>,        // Key decisions made
    pub code_artifacts: Vec<CodeRef>,    // Code created/modified
    pub numerical_values: Vec<NumericFact>, // Numbers, dates, quantities
    pub user_preferences: Vec<Preference>,  // Learned preferences
}

pub struct PreservingSummarizer {
    llm: Box<dyn LLMClient>,
    entity_extractor: EntityExtractor,
}

impl PreservingSummarizer {
    pub async fn summarize_with_preservation(
        &self,
        content: &str,
    ) -> (String, PreservationTags) {
        // First pass: extract entities and facts
        let tags = self.entity_extractor.extract(content).await;

        // Second pass: summarize with explicit preservation instructions
        let prompt = format!(
            "Summarize the following content, ensuring these items are preserved:\n\
            Entities: {:?}\n\
            Decisions: {:?}\n\
            Code: {:?}\n\
            Numbers: {:?}\n\n\
            Content:\n{}",
            tags.entities, tags.decisions, tags.code_artifacts,
            tags.numerical_values, content
        );

        let summary = self.llm.complete(&prompt).await;
        (summary, tags)
    }
}
```

### 3.3 Incremental Summarization Triggers

```rust
pub struct SummarizationTrigger {
    /// Trigger summarization when token count exceeds this
    token_threshold: usize,
    /// Trigger when turn count exceeds this
    turn_threshold: usize,
    /// Trigger after this duration of inactivity (for session summaries)
    inactivity_threshold: Duration,
    /// Trigger on explicit topic/task change
    topic_change_threshold: f32,
}

impl SummarizationTrigger {
    pub fn should_summarize(&self, context: &ContextState) -> SummarizationDecision {
        // Check token threshold
        if context.current_tokens > self.token_threshold {
            return SummarizationDecision::Immediate {
                reason: "Token threshold exceeded",
                urgency: 1.0,
            };
        }

        // Check turn threshold
        if context.turn_count > self.turn_threshold {
            return SummarizationDecision::Recommended {
                reason: "Turn threshold exceeded",
                urgency: 0.7,
            };
        }

        // Check topic shift
        if let Some(shift) = context.detect_topic_shift() {
            if shift > self.topic_change_threshold {
                return SummarizationDecision::Recommended {
                    reason: "Topic change detected",
                    urgency: 0.8,
                };
            }
        }

        // Check inactivity
        if context.time_since_last_activity() > self.inactivity_threshold {
            return SummarizationDecision::Background {
                reason: "Session idle - creating checkpoint",
            };
        }

        SummarizationDecision::NotNeeded
    }
}
```

---

## 4. RAG vs In-Context Learning

### 4.1 Decision Framework

| Factor | Favor RAG | Favor In-Context |
|--------|-----------|------------------|
| Data Volume | >100K tokens total | <50K tokens |
| Query Pattern | Sparse retrieval | Dense, sequential |
| Update Frequency | Frequent updates | Static content |
| Latency Tolerance | Can tolerate retrieval | Needs instant response |
| Accuracy Needs | Can verify sources | Needs exact reproduction |
| Content Type | Factual, lookup | Procedural, reasoning |

### 4.2 Hybrid RAG Architecture

```rust
pub struct HybridRAG {
    /// Dense retrieval using embeddings
    dense_retriever: DenseRetriever,
    /// Sparse retrieval using BM25/TF-IDF
    sparse_retriever: SparseRetriever,
    /// Fusion weights
    dense_weight: f32,
    sparse_weight: f32,
    /// Reranker for final scoring
    reranker: Option<Box<dyn Reranker>>,
}

impl HybridRAG {
    pub async fn retrieve(&self, query: &str, top_k: usize) -> Vec<RetrievedChunk> {
        // Parallel retrieval
        let (dense_results, sparse_results) = tokio::join!(
            self.dense_retriever.retrieve(query, top_k * 2),
            self.sparse_retriever.retrieve(query, top_k * 2)
        );

        // Reciprocal Rank Fusion
        let fused = self.reciprocal_rank_fusion(
            &dense_results,
            &sparse_results,
            60 // RRF constant k
        );

        // Optional reranking
        let final_results = match &self.reranker {
            Some(reranker) => reranker.rerank(query, fused, top_k).await,
            None => fused.into_iter().take(top_k).collect(),
        };

        final_results
    }

    fn reciprocal_rank_fusion(
        &self,
        dense: &[RetrievedChunk],
        sparse: &[RetrievedChunk],
        k: usize,
    ) -> Vec<RetrievedChunk> {
        let mut scores: HashMap<ChunkId, f32> = HashMap::new();

        for (rank, chunk) in dense.iter().enumerate() {
            let rrf_score = self.dense_weight / (k + rank + 1) as f32;
            *scores.entry(chunk.id).or_default() += rrf_score;
        }

        for (rank, chunk) in sparse.iter().enumerate() {
            let rrf_score = self.sparse_weight / (k + rank + 1) as f32;
            *scores.entry(chunk.id).or_default() += rrf_score;
        }

        // Sort by fused score
        let mut results: Vec<_> = scores.into_iter().collect();
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Return chunks with fused scores
        results.into_iter()
            .filter_map(|(id, score)| {
                dense.iter().chain(sparse.iter())
                    .find(|c| c.id == id)
                    .map(|c| RetrievedChunk { score, ..c.clone() })
            })
            .collect()
    }
}
```

### 4.3 Optimal Chunking Strategy

```rust
pub struct AdaptiveChunker {
    /// Target chunk size in tokens
    target_size: usize,
    /// Overlap between chunks
    overlap: usize,
    /// Semantic boundary detector
    boundary_detector: BoundaryDetector,
}

impl AdaptiveChunker {
    pub fn chunk(&self, document: &str) -> Vec<Chunk> {
        let mut chunks = Vec::new();
        let tokens = tokenize(document);

        // Detect semantic boundaries (paragraphs, sections, code blocks)
        let boundaries = self.boundary_detector.detect(&tokens);

        let mut current_start = 0;

        while current_start < tokens.len() {
            // Find nearest boundary after target size
            let target_end = current_start + self.target_size;

            let chunk_end = boundaries.iter()
                .filter(|&&b| b > current_start && b <= target_end + self.target_size / 2)
                .min_by_key(|&&b| (b as i64 - target_end as i64).abs())
                .copied()
                .unwrap_or(target_end.min(tokens.len()));

            let chunk_tokens = &tokens[current_start..chunk_end];

            chunks.push(Chunk {
                content: detokenize(chunk_tokens),
                start_offset: current_start,
                end_offset: chunk_end,
                token_count: chunk_tokens.len(),
            });

            // Move start, accounting for overlap
            current_start = chunk_end.saturating_sub(self.overlap);
        }

        chunks
    }
}
```

**Recommended Chunking Parameters:**
- Target chunk size: 256-512 tokens for dense retrieval
- Overlap: 10-20% of chunk size
- Use semantic boundaries when possible (markdown headers, code blocks)
- Include parent context (document title, section header) in each chunk

---

## 5. Rust Serialization Strategies

### 5.1 State Serialization with Serde

```rust
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentState {
    /// Unique session identifier
    pub session_id: Uuid,
    /// State version for migration support
    #[serde(default = "default_version")]
    pub version: u32,
    /// Timestamp of state creation
    pub created_at: DateTime<Utc>,
    /// Last modification timestamp
    pub updated_at: DateTime<Utc>,
    /// Conversation context
    pub context: ConversationContext,
    /// Active goals and tasks
    pub goals: Vec<Goal>,
    /// Working memory
    pub working_memory: WorkingMemory,
    /// User preferences learned during session
    pub preferences: UserPreferences,
    /// Checkpoint metadata
    pub checkpoint: Option<CheckpointMetadata>,
}

fn default_version() -> u32 { 1 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationContext {
    /// Full conversation history (for serialization)
    pub messages: Vec<Message>,
    /// Summarized history segments
    pub summaries: Vec<SummarySegment>,
    /// Current token count
    pub token_count: usize,
    /// Extracted entities
    pub entities: HashMap<String, Entity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingMemory {
    /// Currently active task
    pub active_task: Option<Task>,
    /// Pending tool calls
    pub pending_tools: Vec<PendingToolCall>,
    /// Scratchpad for intermediate results
    pub scratchpad: HashMap<String, serde_json::Value>,
    /// Recent observations
    pub observations: VecDeque<Observation>,
}
```

### 5.2 Efficient Binary Serialization

```rust
use bincode::{config, Decode, Encode};

/// Compact binary representation for checkpoints
#[derive(Debug, Clone, Encode, Decode)]
pub struct CompactState {
    pub session_id: [u8; 16],
    pub version: u32,
    pub timestamp: i64,
    pub context_hash: [u8; 32],
    pub messages: Vec<CompactMessage>,
    pub summary_refs: Vec<SummaryRef>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct CompactMessage {
    pub role: u8,
    pub content: String,
    pub timestamp: i64,
    pub token_count: u16,
}

impl AgentState {
    /// Serialize to efficient binary format
    pub fn to_binary(&self) -> Result<Vec<u8>, bincode::error::EncodeError> {
        let config = config::standard()
            .with_little_endian()
            .with_variable_int_encoding();

        let compact = self.to_compact();
        bincode::encode_to_vec(&compact, config)
    }

    /// Deserialize from binary format
    pub fn from_binary(data: &[u8]) -> Result<Self, bincode::error::DecodeError> {
        let config = config::standard()
            .with_little_endian()
            .with_variable_int_encoding();

        let compact: CompactState = bincode::decode_from_slice(data, config)?.0;
        Self::from_compact(compact)
    }
}
```

### 5.3 Incremental Serialization for Large States

```rust
use std::io::{Read, Write};

pub struct IncrementalSerializer {
    /// Base directory for state storage
    base_path: PathBuf,
    /// Maximum size for inline storage
    inline_threshold: usize,
}

impl IncrementalSerializer {
    /// Save state with large blobs stored separately
    pub async fn save(&self, state: &AgentState) -> Result<(), StateError> {
        let session_dir = self.base_path.join(state.session_id.to_string());
        tokio::fs::create_dir_all(&session_dir).await?;

        // Separate large content
        let mut manifest = StateManifest {
            version: state.version,
            session_id: state.session_id,
            created_at: state.created_at,
            blob_refs: Vec::new(),
        };

        // Store messages as separate blob if large
        let messages_size: usize = state.context.messages.iter()
            .map(|m| m.content.len())
            .sum();

        if messages_size > self.inline_threshold {
            let blob_path = session_dir.join("messages.bin");
            let blob_data = bincode::encode_to_vec(
                &state.context.messages,
                config::standard()
            )?;
            tokio::fs::write(&blob_path, &blob_data).await?;
            manifest.blob_refs.push(BlobRef {
                field: "context.messages".to_string(),
                path: blob_path,
                size: blob_data.len(),
                hash: blake3::hash(&blob_data).into(),
            });
        }

        // Store manifest with inline content
        let manifest_path = session_dir.join("manifest.json");
        let manifest_json = serde_json::to_vec_pretty(&manifest)?;
        tokio::fs::write(&manifest_path, &manifest_json).await?;

        Ok(())
    }

    /// Load state, reassembling from blobs
    pub async fn load(&self, session_id: Uuid) -> Result<AgentState, StateError> {
        let session_dir = self.base_path.join(session_id.to_string());
        let manifest_path = session_dir.join("manifest.json");

        let manifest_data = tokio::fs::read(&manifest_path).await?;
        let manifest: StateManifest = serde_json::from_slice(&manifest_data)?;

        let mut state = AgentState::default();
        state.session_id = manifest.session_id;
        state.version = manifest.version;

        // Load blobs
        for blob_ref in &manifest.blob_refs {
            let blob_data = tokio::fs::read(&blob_ref.path).await?;

            // Verify integrity
            let computed_hash = blake3::hash(&blob_data);
            if computed_hash.as_bytes() != &blob_ref.hash {
                return Err(StateError::IntegrityError(blob_ref.field.clone()));
            }

            // Deserialize and assign
            match blob_ref.field.as_str() {
                "context.messages" => {
                    state.context.messages = bincode::decode_from_slice(
                        &blob_data,
                        config::standard()
                    )?.0;
                }
                _ => {}
            }
        }

        Ok(state)
    }
}
```

---

## 6. State Persistence Patterns

### 6.1 Checkpointing Strategy

```rust
pub struct CheckpointManager {
    /// Storage backend
    storage: Box<dyn StateStorage>,
    /// Checkpoint interval
    interval: CheckpointInterval,
    /// Maximum checkpoints to retain
    max_checkpoints: usize,
    /// Compression settings
    compression: CompressionSettings,
}

#[derive(Debug, Clone)]
pub enum CheckpointInterval {
    /// Checkpoint every N messages
    EveryNMessages(usize),
    /// Checkpoint every N tokens added
    EveryNTokens(usize),
    /// Checkpoint on specific events
    OnEvents(Vec<CheckpointEvent>),
    /// Time-based checkpointing
    TimeBased(Duration),
    /// Adaptive based on change rate
    Adaptive { min_interval: Duration, max_changes: usize },
}

#[derive(Debug, Clone)]
pub enum CheckpointEvent {
    TaskComplete,
    ToolExecuted,
    UserResponse,
    ErrorRecovery,
    SessionPause,
}

impl CheckpointManager {
    pub async fn maybe_checkpoint(
        &self,
        state: &AgentState,
        event: Option<CheckpointEvent>,
    ) -> Result<Option<CheckpointId>, CheckpointError> {
        let should_checkpoint = match &self.interval {
            CheckpointInterval::EveryNMessages(n) => {
                state.context.messages.len() % n == 0
            }
            CheckpointInterval::EveryNTokens(n) => {
                state.context.token_count % n < 100 // Within threshold
            }
            CheckpointInterval::OnEvents(events) => {
                event.as_ref().map(|e| events.contains(e)).unwrap_or(false)
            }
            CheckpointInterval::TimeBased(duration) => {
                state.checkpoint.as_ref()
                    .map(|c| Utc::now() - c.timestamp > *duration)
                    .unwrap_or(true)
            }
            CheckpointInterval::Adaptive { min_interval, max_changes } => {
                self.should_adaptive_checkpoint(state, *min_interval, *max_changes)
            }
        };

        if should_checkpoint {
            let checkpoint_id = self.create_checkpoint(state).await?;
            self.prune_old_checkpoints().await?;
            Ok(Some(checkpoint_id))
        } else {
            Ok(None)
        }
    }

    async fn create_checkpoint(&self, state: &AgentState) -> Result<CheckpointId, CheckpointError> {
        let checkpoint_id = CheckpointId::new();
        let checkpoint_data = CheckpointData {
            id: checkpoint_id,
            timestamp: Utc::now(),
            state_version: state.version,
            token_count: state.context.token_count,
            message_count: state.context.messages.len(),
        };

        // Serialize state
        let serialized = if self.compression.enabled {
            let raw = state.to_binary()?;
            compress(&raw, self.compression.level)?
        } else {
            state.to_binary()?
        };

        self.storage.save_checkpoint(
            state.session_id,
            checkpoint_id,
            &serialized,
            &checkpoint_data,
        ).await?;

        Ok(checkpoint_id)
    }
}
```

### 6.2 Multi-Session Continuity

```rust
pub struct SessionManager {
    /// Active sessions by ID
    sessions: HashMap<Uuid, SessionHandle>,
    /// Session index for quick lookup
    session_index: SessionIndex,
    /// Storage for persistence
    storage: Arc<dyn StateStorage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionIndex {
    /// Sessions indexed by user
    by_user: HashMap<UserId, Vec<SessionSummary>>,
    /// Sessions indexed by project/workspace
    by_project: HashMap<ProjectId, Vec<SessionSummary>>,
    /// Full-text search index of session content
    search_index: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub session_id: Uuid,
    pub user_id: UserId,
    pub project_id: Option<ProjectId>,
    pub created_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
    pub title: String,
    pub summary: String,
    pub tags: Vec<String>,
    pub token_count: usize,
    pub message_count: usize,
}

impl SessionManager {
    /// Resume a previous session
    pub async fn resume_session(&mut self, session_id: Uuid) -> Result<&mut SessionHandle, SessionError> {
        if !self.sessions.contains_key(&session_id) {
            // Load from storage
            let state = self.storage.load_session(session_id).await?;
            let handle = SessionHandle::from_state(state);
            self.sessions.insert(session_id, handle);
        }

        // Update last active
        self.update_session_index(session_id, |summary| {
            summary.last_active = Utc::now();
        }).await?;

        Ok(self.sessions.get_mut(&session_id).unwrap())
    }

    /// Find related sessions for context
    pub async fn find_related_sessions(
        &self,
        query: &str,
        user_id: &UserId,
        limit: usize,
    ) -> Result<Vec<SessionSummary>, SessionError> {
        // Get user's sessions
        let user_sessions = self.session_index.by_user
            .get(user_id)
            .cloned()
            .unwrap_or_default();

        // Score by relevance
        let mut scored: Vec<_> = user_sessions.into_iter()
            .map(|s| {
                let title_score = fuzzy_match(&s.title, query);
                let summary_score = fuzzy_match(&s.summary, query);
                let tag_score = s.tags.iter()
                    .map(|t| fuzzy_match(t, query))
                    .max()
                    .unwrap_or(0.0);
                let recency_score = recency_weight(s.last_active);

                let total_score = 0.3 * title_score
                    + 0.3 * summary_score
                    + 0.2 * tag_score
                    + 0.2 * recency_score;

                (s, total_score)
            })
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        Ok(scored.into_iter()
            .take(limit)
            .map(|(s, _)| s)
            .collect())
    }

    /// Import context from another session
    pub async fn import_context(
        &mut self,
        target_session: Uuid,
        source_session: Uuid,
        import_config: ImportConfig,
    ) -> Result<(), SessionError> {
        let source_state = self.storage.load_session(source_session).await?;
        let target = self.sessions.get_mut(&target_session)
            .ok_or(SessionError::NotFound(target_session))?;

        match import_config.mode {
            ImportMode::SummaryOnly => {
                // Just import the session summary
                let summary = summarize_session(&source_state).await;
                target.state.context.summaries.push(summary);
            }
            ImportMode::RecentMessages(n) => {
                // Import last N messages
                let recent: Vec<_> = source_state.context.messages
                    .iter()
                    .rev()
                    .take(n)
                    .rev()
                    .cloned()
                    .collect();
                target.state.context.summaries.push(SummarySegment {
                    content: format!("From previous session {}:", source_session),
                    source_messages: recent,
                    ..Default::default()
                });
            }
            ImportMode::EntitiesOnly => {
                // Import extracted entities
                target.state.context.entities.extend(
                    source_state.context.entities.clone()
                );
            }
        }

        Ok(())
    }
}
```

### 6.3 Memory Hierarchy Implementation

```rust
pub struct MemoryHierarchy {
    /// Immediate context (in-prompt)
    pub short_term: ShortTermMemory,
    /// Session-level working memory
    pub working: WorkingMemory,
    /// Cross-session persistent memory
    pub long_term: LongTermMemory,
    /// Episodic memory for specific events
    pub episodic: EpisodicMemory,
}

#[derive(Debug)]
pub struct ShortTermMemory {
    /// Recent messages in context window
    messages: VecDeque<Message>,
    /// Maximum tokens for short-term
    max_tokens: usize,
    /// Current token count
    current_tokens: usize,
}

impl ShortTermMemory {
    pub fn add(&mut self, message: Message) {
        let tokens = count_tokens(&message.content);

        // Evict oldest if necessary
        while self.current_tokens + tokens > self.max_tokens && !self.messages.is_empty() {
            if let Some(evicted) = self.messages.pop_front() {
                self.current_tokens -= count_tokens(&evicted.content);
            }
        }

        self.messages.push_back(message);
        self.current_tokens += tokens;
    }
}

#[derive(Debug)]
pub struct LongTermMemory {
    /// Vector store for semantic retrieval
    vector_store: Box<dyn VectorStore>,
    /// Key-value store for structured data
    kv_store: Box<dyn KVStore>,
    /// Graph store for entity relationships
    graph_store: Option<Box<dyn GraphStore>>,
}

impl LongTermMemory {
    pub async fn remember(&self, memory: MemoryItem) -> Result<(), MemoryError> {
        // Generate embedding
        let embedding = self.vector_store.embed(&memory.content).await?;

        // Store in vector store
        self.vector_store.insert(MemoryRecord {
            id: memory.id,
            embedding,
            content: memory.content.clone(),
            metadata: memory.metadata.clone(),
            timestamp: Utc::now(),
        }).await?;

        // Store structured data
        if let Some(structured) = &memory.structured_data {
            self.kv_store.set(
                &format!("memory:{}", memory.id),
                structured,
            ).await?;
        }

        // Update entity graph if present
        if let Some(graph) = &self.graph_store {
            for entity in &memory.entities {
                graph.upsert_node(entity.clone()).await?;
                for relation in &entity.relations {
                    graph.upsert_edge(relation.clone()).await?;
                }
            }
        }

        Ok(())
    }

    pub async fn recall(
        &self,
        query: &str,
        filters: &RecallFilters,
        top_k: usize,
    ) -> Result<Vec<MemoryItem>, MemoryError> {
        // Semantic search
        let query_embedding = self.vector_store.embed(query).await?;
        let candidates = self.vector_store.search(
            &query_embedding,
            top_k * 2,
            filters,
        ).await?;

        // Enrich with structured data
        let mut results = Vec::with_capacity(candidates.len());
        for record in candidates {
            let structured = self.kv_store
                .get(&format!("memory:{}", record.id))
                .await
                .ok();

            results.push(MemoryItem {
                id: record.id,
                content: record.content,
                metadata: record.metadata,
                structured_data: structured,
                relevance_score: record.score,
                ..Default::default()
            });
        }

        // Sort by combined relevance and recency
        results.sort_by(|a, b| {
            let score_a = a.relevance_score * 0.7 + a.recency_score() * 0.3;
            let score_b = b.relevance_score * 0.7 + b.recency_score() * 0.3;
            score_b.partial_cmp(&score_a).unwrap()
        });

        Ok(results.into_iter().take(top_k).collect())
    }
}

#[derive(Debug)]
pub struct EpisodicMemory {
    /// Episodes organized by timestamp
    episodes: BTreeMap<DateTime<Utc>, Episode>,
    /// Index by episode type
    type_index: HashMap<EpisodeType, Vec<DateTime<Utc>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Episode {
    pub timestamp: DateTime<Utc>,
    pub episode_type: EpisodeType,
    pub summary: String,
    pub context: String,
    pub outcome: Option<String>,
    pub importance: f32,
    pub related_entities: Vec<EntityRef>,
}

impl EpisodicMemory {
    pub fn record_episode(&mut self, episode: Episode) {
        let timestamp = episode.timestamp;
        let episode_type = episode.episode_type.clone();

        self.episodes.insert(timestamp, episode);
        self.type_index
            .entry(episode_type)
            .or_default()
            .push(timestamp);
    }

    pub fn recall_similar_episodes(
        &self,
        current_context: &str,
        episode_type: Option<EpisodeType>,
        limit: usize,
    ) -> Vec<&Episode> {
        let candidates: Box<dyn Iterator<Item = &Episode>> = match episode_type {
            Some(et) => {
                let timestamps = self.type_index.get(&et).cloned().unwrap_or_default();
                Box::new(timestamps.into_iter().filter_map(|t| self.episodes.get(&t)))
            }
            None => Box::new(self.episodes.values()),
        };

        let mut scored: Vec<_> = candidates
            .map(|e| {
                let context_sim = semantic_similarity(&e.context, current_context);
                let importance = e.importance;
                let recency = recency_weight(e.timestamp);
                let score = 0.5 * context_sim + 0.3 * importance + 0.2 * recency;
                (e, score)
            })
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        scored.into_iter()
            .take(limit)
            .map(|(e, _)| e)
            .collect()
    }
}
```

---

## 7. Implementation Recommendations

### 7.1 Rust Crate Dependencies

```toml
[dependencies]
# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
bincode = "2.0"

# Async runtime
tokio = { version = "1.0", features = ["full"] }

# Date/time
chrono = { version = "0.4", features = ["serde"] }

# UUID generation
uuid = { version = "1.0", features = ["v4", "serde"] }

# Hashing
blake3 = "1.0"

# Compression
lz4_flex = "0.11"
zstd = "0.13"

# Embeddings (local)
fastembed = "3.0"
# Or for remote
reqwest = { version = "0.11", features = ["json"] }

# Vector storage
qdrant-client = "1.0"
# Or for embedded
hnsw_rs = "0.3"

# Key-value storage
sled = "0.34"
# Or for SQL
sqlx = { version = "0.7", features = ["sqlite", "runtime-tokio"] }

# Tokenization
tiktoken-rs = "0.5"

# Text processing
regex = "1.0"
unicode-segmentation = "1.0"
```

### 7.2 Architecture Decision Matrix

| Decision Point | Recommendation | Rationale |
|---------------|----------------|-----------|
| Primary serialization | bincode + zstd | Fast, compact, Rust-native |
| State storage | SQLite + sled | Reliable, embedded, no external deps |
| Vector store | Qdrant or embedded HNSW | Balance of performance and simplicity |
| Embedding model | fastembed (local) | Low latency, no API costs |
| Chunking strategy | Adaptive semantic | Better retrieval quality |
| Summarization trigger | Adaptive + event-based | Balance of freshness and cost |
| Checkpoint interval | Every 10 messages or 5 min | Reasonable recovery points |

### 7.3 Performance Considerations

1. **Token counting**: Cache token counts, use approximate counting for large texts
2. **Embedding generation**: Batch embeddings when possible, cache frequently accessed
3. **Serialization**: Use streaming for large states, compress incrementally
4. **Retrieval**: Pre-filter by metadata before semantic search
5. **Summarization**: Run in background, don't block user interactions

---

## 8. Conclusion

Effective context management for agentic systems requires a multi-layered approach combining:

1. **Active optimization** of the context window through priority-based selection and compression
2. **Progressive summarization** to preserve important information while managing token budgets
3. **Hybrid RAG** for efficient retrieval of large knowledge bases
4. **Robust state persistence** enabling session continuity and crash recovery

The Rust implementations provided offer type-safe, efficient patterns that can be adapted to specific use cases. Key success factors include:

- Choosing the right balance between in-context and retrieved information
- Implementing adaptive triggers for summarization and checkpointing
- Using efficient serialization with integrity verification
- Building a memory hierarchy that mirrors human cognitive patterns

Future research directions include:
- Self-optimizing context selection using reinforcement learning
- Learned compression models specific to conversation types
- Cross-agent memory sharing for multi-agent systems
- Differential privacy for persistent memory stores
