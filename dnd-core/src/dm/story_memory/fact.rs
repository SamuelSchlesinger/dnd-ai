//! Story facts for narrative consistency.

use super::entity::{EntityId, StoryMoment};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a story fact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FactId(Uuid);

impl FactId {
    /// Create a new unique fact ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for FactId {
    fn default() -> Self {
        Self::new()
    }
}

/// Categories of story facts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FactCategory {
    /// Physical appearance, clothing, distinguishing features.
    Appearance,
    /// Personality traits, behavior patterns, quirks.
    Personality,
    /// Something that happened in the story.
    Event,
    /// Connection between entities.
    Relationship,
    /// History or backstory.
    Backstory,
    /// Goals, desires, or motivations.
    Motivation,
    /// Abilities, skills, or powers.
    Capability,
    /// Where something is or was.
    Location,
    /// Items owned or carried.
    Possession,
    /// Current state or condition.
    Status,
    /// Secrets or hidden information.
    Secret,
}

impl FactCategory {
    /// Get the display name for this category.
    pub fn name(&self) -> &'static str {
        match self {
            FactCategory::Appearance => "Appearance",
            FactCategory::Personality => "Personality",
            FactCategory::Event => "Event",
            FactCategory::Relationship => "Relationship",
            FactCategory::Backstory => "Backstory",
            FactCategory::Motivation => "Motivation",
            FactCategory::Capability => "Capability",
            FactCategory::Location => "Location",
            FactCategory::Possession => "Possession",
            FactCategory::Status => "Status",
            FactCategory::Secret => "Secret",
        }
    }

    /// Check if this category is typically stable (doesn't change often).
    pub fn is_stable(&self) -> bool {
        matches!(
            self,
            FactCategory::Appearance
                | FactCategory::Personality
                | FactCategory::Backstory
                | FactCategory::Capability
        )
    }
}

/// Source of where a fact came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FactSource {
    /// Established by DM narration.
    DmNarration,
    /// Stated by player action or dialogue.
    PlayerAction,
    /// Said by an NPC.
    NpcDialogue,
    /// Discovered through game mechanics.
    Mechanics,
    /// Part of world building / lore.
    WorldBuilding,
}

/// A story fact - a piece of information about an entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryFact {
    /// Unique identifier.
    pub id: FactId,
    /// Primary entity this fact is about.
    pub subject: EntityId,
    /// Other entities mentioned in this fact.
    pub mentioned_entities: Vec<EntityId>,
    /// The fact content in natural language.
    pub content: String,
    /// Category of the fact.
    pub category: FactCategory,
    /// When this fact was established.
    pub established: StoryMoment,
    /// Whether this fact is still current (can be superseded).
    pub is_current: bool,
    /// Importance score (0.0 to 1.0).
    pub importance: f32,
    /// Where this fact came from.
    pub source: FactSource,
}

impl StoryFact {
    /// Create a new story fact.
    pub fn new(
        subject: EntityId,
        content: impl Into<String>,
        category: FactCategory,
        source: FactSource,
        current_turn: u32,
    ) -> Self {
        Self {
            id: FactId::new(),
            subject,
            mentioned_entities: Vec::new(),
            content: content.into(),
            category,
            established: StoryMoment::new(current_turn),
            is_current: true,
            importance: 1.0,
            source,
        }
    }

    /// Add a mentioned entity reference.
    pub fn with_mentioned(mut self, entity_id: EntityId) -> Self {
        if !self.mentioned_entities.contains(&entity_id) && entity_id != self.subject {
            self.mentioned_entities.push(entity_id);
        }
        self
    }

    /// Set the importance level.
    pub fn with_importance(mut self, importance: f32) -> Self {
        self.importance = importance.clamp(0.0, 1.0);
        self
    }

    /// Mark this fact as no longer current (superseded by newer info).
    pub fn supersede(&mut self) {
        self.is_current = false;
    }

    /// Decay importance over time.
    pub fn decay_importance(&mut self, decay_rate: f32) {
        self.importance = (self.importance - decay_rate).max(0.1);
    }

    /// Check if this fact is about the given entity (subject or mentioned).
    pub fn involves(&self, entity_id: EntityId) -> bool {
        self.subject == entity_id || self.mentioned_entities.contains(&entity_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fact_creation() {
        let entity_id = EntityId::new();
        let fact = StoryFact::new(
            entity_id,
            "Gandalf wears a grey cloak and carries a wooden staff",
            FactCategory::Appearance,
            FactSource::DmNarration,
            10,
        );

        assert_eq!(fact.subject, entity_id);
        assert!(fact.content.contains("grey cloak"));
        assert!(fact.is_current);
        assert!(fact.importance > 0.9);
    }

    #[test]
    fn test_fact_involvement() {
        let entity1 = EntityId::new();
        let entity2 = EntityId::new();
        let entity3 = EntityId::new();

        let fact = StoryFact::new(
            entity1,
            "Test fact",
            FactCategory::Event,
            FactSource::DmNarration,
            0,
        )
        .with_mentioned(entity2);

        assert!(fact.involves(entity1));
        assert!(fact.involves(entity2));
        assert!(!fact.involves(entity3));
    }
}
