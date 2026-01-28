//! Entity types for story memory.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for an entity in story memory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId(Uuid);

impl EntityId {
    /// Create a new unique entity ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for EntityId {
    fn default() -> Self {
        Self::new()
    }
}

/// Types of entities that can be tracked in story memory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntityType {
    /// A non-player character.
    Npc,
    /// A geographic location or place.
    Location,
    /// An item, artifact, or object.
    Item,
    /// A quest, mission, or objective.
    Quest,
    /// An organization, faction, or group.
    Organization,
    /// A significant event that occurred.
    Event,
    /// A creature type or specific monster.
    Creature,
}

impl EntityType {
    /// Get the display name for this entity type.
    pub fn name(&self) -> &'static str {
        match self {
            EntityType::Npc => "NPC",
            EntityType::Location => "Location",
            EntityType::Item => "Item",
            EntityType::Quest => "Quest",
            EntityType::Organization => "Organization",
            EntityType::Event => "Event",
            EntityType::Creature => "Creature",
        }
    }
}

/// A moment in the story timeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[derive(Default)]
pub struct StoryMoment {
    /// Turn number when this moment occurred.
    pub turn: u32,
}

impl StoryMoment {
    /// Create a new story moment at the given turn.
    pub fn new(turn: u32) -> Self {
        Self { turn }
    }

    /// Check if this moment is within N turns of another.
    pub fn is_recent(&self, other: &StoryMoment, within_turns: u32) -> bool {
        self.turn.abs_diff(other.turn) <= within_turns
    }
}


/// An entity tracked in story memory (NPC, location, item, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    /// Unique identifier.
    pub id: EntityId,
    /// What kind of entity this is.
    pub entity_type: EntityType,
    /// Primary name of the entity.
    pub name: String,
    /// Alternative names or nicknames.
    pub aliases: Vec<String>,
    /// Brief description (optional).
    pub description: Option<String>,
    /// When this entity first appeared.
    pub first_seen: StoryMoment,
    /// When this entity was last mentioned.
    pub last_seen: StoryMoment,
    /// Importance score (0.0 to 1.0), decays over time.
    pub importance: f32,
}

impl Entity {
    /// Create a new entity.
    pub fn new(
        entity_type: EntityType,
        name: impl Into<String>,
        current_turn: u32,
    ) -> Self {
        let moment = StoryMoment::new(current_turn);
        Self {
            id: EntityId::new(),
            entity_type,
            name: name.into(),
            aliases: Vec::new(),
            description: None,
            first_seen: moment,
            last_seen: moment,
            importance: 1.0,
        }
    }

    /// Add an alias for this entity.
    pub fn with_alias(mut self, alias: impl Into<String>) -> Self {
        self.aliases.push(alias.into());
        self
    }

    /// Set the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Check if a name matches this entity (case-insensitive).
    pub fn matches_name(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();
        if self.name.to_lowercase() == query_lower {
            return true;
        }
        self.aliases.iter().any(|a| a.to_lowercase() == query_lower)
    }

    /// Check if a name partially matches this entity (for fuzzy lookup).
    pub fn matches_partial(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();
        if self.name.to_lowercase().contains(&query_lower) {
            return true;
        }
        self.aliases.iter().any(|a| a.to_lowercase().contains(&query_lower))
    }

    /// Update the last seen moment.
    pub fn touch(&mut self, current_turn: u32) {
        self.last_seen = StoryMoment::new(current_turn);
        // Boost importance when mentioned
        self.importance = (self.importance + 0.2).min(1.0);
    }

    /// Decay importance over time.
    pub fn decay_importance(&mut self, decay_rate: f32) {
        self.importance = (self.importance - decay_rate).max(0.1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_creation() {
        let entity = Entity::new(EntityType::Npc, "Gandalf", 5);
        assert_eq!(entity.name, "Gandalf");
        assert_eq!(entity.entity_type, EntityType::Npc);
        assert_eq!(entity.first_seen.turn, 5);
        assert!(entity.importance > 0.9);
    }

    #[test]
    fn test_name_matching() {
        let entity = Entity::new(EntityType::Npc, "Gandalf the Grey", 0)
            .with_alias("Mithrandir")
            .with_alias("The Grey Pilgrim");

        assert!(entity.matches_name("gandalf the grey"));
        assert!(entity.matches_name("Mithrandir"));
        assert!(!entity.matches_name("Saruman"));

        assert!(entity.matches_partial("gandalf"));
        assert!(entity.matches_partial("grey"));
    }

    #[test]
    fn test_importance_decay() {
        let mut entity = Entity::new(EntityType::Npc, "Test", 0);
        assert!(entity.importance > 0.9);

        entity.decay_importance(0.3);
        assert!(entity.importance < 0.8);

        // Should not go below minimum
        for _ in 0..10 {
            entity.decay_importance(0.3);
        }
        assert!(entity.importance >= 0.1);
    }
}
