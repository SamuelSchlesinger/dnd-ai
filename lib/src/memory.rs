//! Memory traits for agent learning and persistence.
//!
//! The memory system is organized into three types following cognitive architecture:
//! - Episodic: Specific experiences and events
//! - Semantic: General facts and knowledge
//! - Procedural: Skills and procedures

use crate::error::MemoryError;
use crate::id::{FactId, MemoryId, SkillId};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Episodic memory stores specific experiences
#[async_trait]
pub trait EpisodicMemory: Send + Sync {
    /// Store a new experience
    async fn store(&self, experience: Experience) -> Result<MemoryId, MemoryError>;

    /// Retrieve experiences by semantic similarity
    async fn retrieve(&self, query: &str, limit: usize) -> Result<Vec<Experience>, MemoryError>;

    /// Retrieve recent experiences
    async fn retrieve_recent(&self, limit: usize) -> Result<Vec<Experience>, MemoryError>;

    /// Forget (delete) an experience
    async fn forget(&self, id: MemoryId) -> Result<(), MemoryError>;

    /// Get a specific experience by ID
    async fn get(&self, id: MemoryId) -> Result<Option<Experience>, MemoryError>;

    /// Count total experiences
    async fn count(&self) -> Result<usize, MemoryError>;
}

/// Semantic memory stores facts and knowledge
#[async_trait]
pub trait SemanticMemory: Send + Sync {
    /// Store a new fact
    async fn store_fact(&self, fact: Fact) -> Result<FactId, MemoryError>;

    /// Query facts by semantic similarity
    async fn query(&self, query: &str, limit: usize) -> Result<Vec<Fact>, MemoryError>;

    /// Get a specific fact by ID
    async fn get_fact(&self, id: FactId) -> Result<Option<Fact>, MemoryError>;

    /// Update a fact
    async fn update_fact(&self, id: FactId, update: FactUpdate) -> Result<(), MemoryError>;

    /// Delete a fact
    async fn delete_fact(&self, id: FactId) -> Result<(), MemoryError>;

    /// Get facts by subject
    async fn facts_about(&self, subject: &str) -> Result<Vec<Fact>, MemoryError>;
}

/// Procedural memory stores skills and procedures
#[async_trait]
pub trait ProceduralMemory: Send + Sync {
    /// Store a new skill
    async fn store_skill(&self, skill: Skill) -> Result<SkillId, MemoryError>;

    /// Get a skill by name
    async fn get_skill(&self, name: &str) -> Result<Option<Skill>, MemoryError>;

    /// Get a skill by ID
    async fn get_skill_by_id(&self, id: SkillId) -> Result<Option<Skill>, MemoryError>;

    /// List all skills
    async fn list_skills(&self) -> Result<Vec<SkillMetadata>, MemoryError>;

    /// Search skills by description
    async fn search_skills(&self, query: &str, limit: usize) -> Result<Vec<Skill>, MemoryError>;

    /// Update skill statistics after use
    async fn record_skill_use(
        &self,
        id: SkillId,
        success: bool,
        duration: std::time::Duration,
    ) -> Result<(), MemoryError>;

    /// Delete a skill
    async fn delete_skill(&self, id: SkillId) -> Result<(), MemoryError>;
}

/// Unified memory manager that coordinates all memory types
#[async_trait]
pub trait MemoryManager: Send + Sync {
    /// The episodic memory implementation
    type Episodic: EpisodicMemory;
    /// The semantic memory implementation
    type Semantic: SemanticMemory;
    /// The procedural memory implementation
    type Procedural: ProceduralMemory;

    /// Get the episodic memory
    fn episodic(&self) -> &Self::Episodic;

    /// Get the semantic memory
    fn semantic(&self) -> &Self::Semantic;

    /// Get the procedural memory
    fn procedural(&self) -> &Self::Procedural;

    /// Recall relevant memories for a given context
    async fn recall(&self, query: &str, context: &RecallContext) -> Result<RelevantMemories, MemoryError>;

    /// Consolidate memories (extract patterns, prune old memories)
    async fn consolidate(&self) -> Result<ConsolidationReport, MemoryError>;

    /// Create a checkpoint of all memory
    async fn checkpoint(&self) -> Result<MemoryCheckpoint, MemoryError>;

    /// Restore from a checkpoint
    async fn restore(&self, checkpoint: &MemoryCheckpoint) -> Result<(), MemoryError>;
}

/// An experience stored in episodic memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Experience {
    /// Unique identifier
    pub id: MemoryId,
    /// When this experience occurred
    pub timestamp: DateTime<Utc>,
    /// The context/situation
    pub context: String,
    /// What action was taken
    pub action: String,
    /// What was the outcome
    pub outcome: String,
    /// Was this a positive or negative experience
    pub valence: Valence,
    /// Importance score (0.0 - 1.0)
    pub importance: f32,
    /// Optional embedding for similarity search
    #[serde(skip)]
    pub embedding: Option<Vec<f32>>,
    /// Additional metadata
    pub metadata: ExperienceMetadata,
}

impl Experience {
    /// Create a new experience
    pub fn new(context: impl Into<String>, action: impl Into<String>, outcome: impl Into<String>) -> Self {
        Self {
            id: MemoryId::new(),
            timestamp: Utc::now(),
            context: context.into(),
            action: action.into(),
            outcome: outcome.into(),
            valence: Valence::Neutral,
            importance: 0.5,
            embedding: None,
            metadata: ExperienceMetadata::default(),
        }
    }

    /// Set the valence
    pub fn with_valence(mut self, valence: Valence) -> Self {
        self.valence = valence;
        self
    }

    /// Set the importance
    pub fn with_importance(mut self, importance: f32) -> Self {
        self.importance = importance.clamp(0.0, 1.0);
        self
    }

    /// Create a text representation for embedding
    pub fn to_embedding_text(&self) -> String {
        format!(
            "Context: {}\nAction: {}\nOutcome: {}",
            self.context, self.action, self.outcome
        )
    }
}

/// Emotional valence of an experience
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Valence {
    /// Positive experience
    Positive,
    /// Neutral experience
    Neutral,
    /// Negative experience
    Negative,
}

/// Metadata for an experience
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExperienceMetadata {
    /// Source of the experience
    pub source: Option<String>,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Related memory IDs
    pub related: Vec<MemoryId>,
    /// Custom fields
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

/// A fact stored in semantic memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fact {
    /// Unique identifier
    pub id: FactId,
    /// Subject of the fact
    pub subject: String,
    /// Predicate/relationship
    pub predicate: String,
    /// Object of the fact
    pub object: String,
    /// Confidence in this fact (0.0 - 1.0)
    pub confidence: f32,
    /// When this fact was learned
    pub learned_at: DateTime<Utc>,
    /// When this fact was last verified
    pub verified_at: Option<DateTime<Utc>>,
    /// Source of this fact
    pub source: Option<String>,
    /// Optional embedding
    #[serde(skip)]
    pub embedding: Option<Vec<f32>>,
}

impl Fact {
    /// Create a new fact
    pub fn new(
        subject: impl Into<String>,
        predicate: impl Into<String>,
        object: impl Into<String>,
    ) -> Self {
        Self {
            id: FactId::new(),
            subject: subject.into(),
            predicate: predicate.into(),
            object: object.into(),
            confidence: 1.0,
            learned_at: Utc::now(),
            verified_at: None,
            source: None,
            embedding: None,
        }
    }

    /// Set confidence
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    /// Set source
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Create a text representation
    pub fn to_text(&self) -> String {
        format!("{} {} {}", self.subject, self.predicate, self.object)
    }
}

/// Update to apply to a fact
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FactUpdate {
    /// New subject
    pub subject: Option<String>,
    /// New predicate
    pub predicate: Option<String>,
    /// New object
    pub object: Option<String>,
    /// New confidence
    pub confidence: Option<f32>,
    /// Mark as verified
    pub verified: bool,
}

/// A skill stored in procedural memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    /// Unique identifier
    pub id: SkillId,
    /// Skill name
    pub name: String,
    /// Description of what this skill does
    pub description: String,
    /// Preconditions for using this skill
    pub preconditions: Vec<String>,
    /// Expected effects of using this skill
    pub effects: Vec<String>,
    /// The implementation
    pub implementation: SkillImplementation,
    /// Usage statistics
    pub stats: SkillStats,
    /// When this skill was learned
    pub learned_at: DateTime<Utc>,
}

impl Skill {
    /// Create a new skill
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            id: SkillId::new(),
            name: name.into(),
            description: description.into(),
            preconditions: Vec::new(),
            effects: Vec::new(),
            implementation: SkillImplementation::Prompt(String::new()),
            stats: SkillStats::default(),
            learned_at: Utc::now(),
        }
    }

    /// Set implementation
    pub fn with_implementation(mut self, impl_: SkillImplementation) -> Self {
        self.implementation = impl_;
        self
    }

    /// Add a precondition
    pub fn with_precondition(mut self, precondition: impl Into<String>) -> Self {
        self.preconditions.push(precondition.into());
        self
    }

    /// Add an effect
    pub fn with_effect(mut self, effect: impl Into<String>) -> Self {
        self.effects.push(effect.into());
        self
    }
}

/// How a skill is implemented
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SkillImplementation {
    /// A prompt template
    Prompt(String),
    /// A sequence of tool calls
    ToolSequence(Vec<ToolStep>),
    /// Executable code
    Code { language: String, code: String },
    /// Composed of other skills
    Composite(Vec<SkillId>),
}

/// A step in a tool sequence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolStep {
    /// Tool name
    pub tool: String,
    /// Parameter template (may contain variables)
    pub params: Value,
    /// Variable to store result in
    pub result_var: Option<String>,
}

/// Usage statistics for a skill
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SkillStats {
    /// Number of times used
    pub use_count: u64,
    /// Number of successful uses
    pub success_count: u64,
    /// Average execution time in milliseconds
    pub avg_duration_ms: f64,
    /// Last time this skill was used
    pub last_used: Option<DateTime<Utc>>,
}

impl SkillStats {
    /// Calculate success rate
    pub fn success_rate(&self) -> f64 {
        if self.use_count == 0 {
            0.0
        } else {
            self.success_count as f64 / self.use_count as f64
        }
    }
}

/// Metadata for skill listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMetadata {
    /// Skill ID
    pub id: SkillId,
    /// Skill name
    pub name: String,
    /// Brief description
    pub description: String,
    /// Success rate
    pub success_rate: f64,
    /// Use count
    pub use_count: u64,
}

/// Context for memory recall
#[derive(Debug, Clone, Default)]
pub struct RecallContext {
    /// Maximum episodic memories to retrieve
    pub max_episodic: usize,
    /// Maximum facts to retrieve
    pub max_facts: usize,
    /// Maximum skills to retrieve
    pub max_skills: usize,
    /// Minimum relevance score (0.0 - 1.0)
    pub min_relevance: f32,
    /// Time range filter
    pub time_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
}

impl RecallContext {
    /// Create a default recall context
    pub fn new() -> Self {
        Self {
            max_episodic: 5,
            max_facts: 10,
            max_skills: 3,
            min_relevance: 0.5,
            time_range: None,
        }
    }
}

/// Relevant memories retrieved for a query
#[derive(Debug, Clone, Default)]
pub struct RelevantMemories {
    /// Relevant experiences
    pub experiences: Vec<(Experience, f32)>,
    /// Relevant facts
    pub facts: Vec<(Fact, f32)>,
    /// Relevant skills
    pub skills: Vec<(Skill, f32)>,
}

/// Report from memory consolidation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConsolidationReport {
    /// Number of experiences processed
    pub experiences_processed: usize,
    /// Number of new facts extracted
    pub facts_extracted: usize,
    /// Number of memories pruned
    pub memories_pruned: usize,
    /// Patterns discovered
    pub patterns: Vec<String>,
}

/// A checkpoint of memory state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryCheckpoint {
    /// Checkpoint ID
    pub id: crate::id::CheckpointId,
    /// When the checkpoint was created
    pub created_at: DateTime<Utc>,
    /// Serialized memory data
    pub data: Vec<u8>,
    /// Checksum for integrity verification
    pub checksum: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_experience_creation() {
        let exp = Experience::new("testing", "ran tests", "tests passed")
            .with_valence(Valence::Positive)
            .with_importance(0.8);

        assert_eq!(exp.valence, Valence::Positive);
        assert_eq!(exp.importance, 0.8);
    }

    #[test]
    fn test_fact_creation() {
        let fact = Fact::new("Rust", "is", "a programming language")
            .with_confidence(0.95)
            .with_source("documentation");

        assert_eq!(fact.subject, "Rust");
        assert_eq!(fact.confidence, 0.95);
    }

    #[test]
    fn test_skill_stats() {
        let mut stats = SkillStats::default();
        stats.use_count = 10;
        stats.success_count = 8;

        assert_eq!(stats.success_rate(), 0.8);
    }
}
