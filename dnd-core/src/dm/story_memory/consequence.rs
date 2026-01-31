//! Consequence system for deferred narrative effects.
//!
//! Consequences represent pending narrative effects that should trigger
//! when certain conditions are met. Unlike facts which record what happened,
//! consequences track what should happen in the future.
//!
//! Examples:
//! - "If the player enters Riverside, guards will attempt arrest"
//! - "The merchant remembers being cheated and will refuse service"
//! - "The curse activates when the player sleeps"

use super::entity::{EntityId, StoryMoment};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a consequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConsequenceId(Uuid);

impl ConsequenceId {
    /// Create a new unique consequence ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ConsequenceId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ConsequenceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The current status of a consequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsequenceStatus {
    /// Waiting to be triggered.
    Pending,
    /// Has been triggered and the effect has occurred.
    Triggered,
    /// Was resolved without triggering (player avoided it, or it was addressed).
    Resolved,
    /// Expired without being triggered (time limit passed).
    Expired,
}

impl ConsequenceStatus {
    /// Check if this consequence is still active (could still trigger).
    pub fn is_active(&self) -> bool {
        matches!(self, ConsequenceStatus::Pending)
    }
}

/// How severe the consequence is (affects how prominently it's surfaced).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsequenceSeverity {
    /// Minor inconvenience or flavor (merchant is rude).
    Minor,
    /// Moderate impact (denied service, unfriendly NPCs).
    Moderate,
    /// Major story impact (bounty hunters, locked out of quests).
    Major,
    /// Critical/life-threatening (assassination attempts, town guards attack).
    Critical,
}

impl ConsequenceSeverity {
    /// Get the display name.
    pub fn name(&self) -> &'static str {
        match self {
            ConsequenceSeverity::Minor => "Minor",
            ConsequenceSeverity::Moderate => "Moderate",
            ConsequenceSeverity::Major => "Major",
            ConsequenceSeverity::Critical => "Critical",
        }
    }

    /// Get a base importance score for this severity.
    pub fn base_importance(&self) -> f32 {
        match self {
            ConsequenceSeverity::Minor => 0.3,
            ConsequenceSeverity::Moderate => 0.5,
            ConsequenceSeverity::Major => 0.8,
            ConsequenceSeverity::Critical => 1.0,
        }
    }
}

/// A pending consequence that may trigger in the future.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Consequence {
    /// Unique identifier.
    pub id: ConsequenceId,

    /// Natural language description of when this triggers.
    /// Used for semantic matching by the relevance checker.
    /// Example: "The player enters Riverside village or its surrounding area"
    pub trigger_description: String,

    /// Natural language description of what happens when triggered.
    /// Example: "Town guards recognize the player and attempt to arrest them for crimes against the baron"
    pub consequence_description: String,

    /// How severe this consequence is.
    pub severity: ConsequenceSeverity,

    /// The primary entity this consequence is about (if any).
    /// For example, an NPC who was wronged, or a location with a trap.
    pub subject_entity: Option<EntityId>,

    /// Other entities involved in this consequence.
    pub related_entities: Vec<EntityId>,

    /// When this consequence was created.
    pub created: StoryMoment,

    /// When this consequence expires (if ever).
    /// After this turn, the consequence becomes Expired.
    pub expires_turn: Option<u32>,

    /// Current status.
    pub status: ConsequenceStatus,

    /// Importance score for relevance ranking (0.0 to 1.0).
    pub importance: f32,

    /// Optional reference to the fact/event that caused this consequence.
    pub source_description: Option<String>,
}

impl Consequence {
    /// Create a new pending consequence.
    pub fn new(
        trigger_description: impl Into<String>,
        consequence_description: impl Into<String>,
        severity: ConsequenceSeverity,
        current_turn: u32,
    ) -> Self {
        Self {
            id: ConsequenceId::new(),
            trigger_description: trigger_description.into(),
            consequence_description: consequence_description.into(),
            severity,
            subject_entity: None,
            related_entities: Vec::new(),
            created: StoryMoment::new(current_turn),
            expires_turn: None,
            status: ConsequenceStatus::Pending,
            importance: severity.base_importance(),
            source_description: None,
        }
    }

    /// Set the subject entity.
    pub fn with_subject(mut self, entity_id: EntityId) -> Self {
        self.subject_entity = Some(entity_id);
        self
    }

    /// Add a related entity.
    pub fn with_related(mut self, entity_id: EntityId) -> Self {
        if !self.related_entities.contains(&entity_id) {
            self.related_entities.push(entity_id);
        }
        self
    }

    /// Set when this consequence expires.
    pub fn with_expiry(mut self, expires_turn: u32) -> Self {
        self.expires_turn = Some(expires_turn);
        self
    }

    /// Set the importance score.
    pub fn with_importance(mut self, importance: f32) -> Self {
        self.importance = importance.clamp(0.0, 1.0);
        self
    }

    /// Set the source description.
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source_description = Some(source.into());
        self
    }

    /// Mark this consequence as triggered.
    pub fn trigger(&mut self) {
        self.status = ConsequenceStatus::Triggered;
    }

    /// Mark this consequence as resolved without triggering.
    pub fn resolve(&mut self) {
        self.status = ConsequenceStatus::Resolved;
    }

    /// Check if this consequence has expired based on current turn.
    pub fn check_expiry(&mut self, current_turn: u32) -> bool {
        if let Some(expires) = self.expires_turn {
            if current_turn >= expires && self.status == ConsequenceStatus::Pending {
                self.status = ConsequenceStatus::Expired;
                return true;
            }
        }
        false
    }

    /// Check if this consequence involves a specific entity.
    pub fn involves(&self, entity_id: EntityId) -> bool {
        self.subject_entity == Some(entity_id) || self.related_entities.contains(&entity_id)
    }

    /// Decay importance over time.
    pub fn decay_importance(&mut self, decay_rate: f32) {
        // Consequences decay slower than facts - they're meant to persist
        let min_importance = self.severity.base_importance() * 0.5;
        self.importance = (self.importance - decay_rate).max(min_importance);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consequence_creation() {
        let consequence = Consequence::new(
            "Player enters Riverside",
            "Guards attempt arrest",
            ConsequenceSeverity::Major,
            10,
        );

        assert_eq!(consequence.status, ConsequenceStatus::Pending);
        assert!(consequence.importance >= 0.7);
        assert!(consequence.status.is_active());
    }

    #[test]
    fn test_consequence_trigger() {
        let mut consequence = Consequence::new(
            "Player enters tavern",
            "Bounty hunter attacks",
            ConsequenceSeverity::Critical,
            5,
        );

        consequence.trigger();
        assert_eq!(consequence.status, ConsequenceStatus::Triggered);
        assert!(!consequence.status.is_active());
    }

    #[test]
    fn test_consequence_expiry() {
        let mut consequence = Consequence::new(
            "Player is in the forest at night",
            "Wolves attack",
            ConsequenceSeverity::Moderate,
            10,
        )
        .with_expiry(15);

        // Not expired yet
        assert!(!consequence.check_expiry(12));
        assert_eq!(consequence.status, ConsequenceStatus::Pending);

        // Now expired
        assert!(consequence.check_expiry(15));
        assert_eq!(consequence.status, ConsequenceStatus::Expired);
    }

    #[test]
    fn test_consequence_entities() {
        let entity1 = EntityId::new();
        let entity2 = EntityId::new();
        let entity3 = EntityId::new();

        let consequence = Consequence::new("Trigger", "Effect", ConsequenceSeverity::Minor, 0)
            .with_subject(entity1)
            .with_related(entity2);

        assert!(consequence.involves(entity1));
        assert!(consequence.involves(entity2));
        assert!(!consequence.involves(entity3));
    }

    #[test]
    fn test_severity_importance() {
        let minor = ConsequenceSeverity::Minor;
        let critical = ConsequenceSeverity::Critical;

        assert!(critical.base_importance() > minor.base_importance());
    }
}
