//! Relationships between entities.

use super::entity::{EntityId, StoryMoment};
use serde::{Deserialize, Serialize};

/// Types of relationships between entities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelationshipType {
    // Positive relationships
    /// Family member.
    Family,
    /// Close friend.
    Friend,
    /// Ally or partner.
    Ally,
    /// Mentor or teacher.
    Mentor,
    /// Student or apprentice.
    Student,
    /// Romantic interest.
    Romantic,
    /// Employer.
    Employer,
    /// Employee or servant.
    Employee,

    // Neutral relationships
    /// Acquaintance.
    Acquaintance,
    /// Business contact.
    Business,
    /// Member of same group.
    FellowMember,

    // Negative relationships
    /// Rival or competitor.
    Rival,
    /// Enemy.
    Enemy,
    /// Betrayer.
    Betrayer,
    /// Target of hunt.
    Hunts,

    // Location relationships
    /// Lives at a location.
    LivesAt,
    /// Works at a location.
    WorksAt,
    /// Owns a location or item.
    Owns,
    /// Created an item.
    Created,

    // Organization relationships
    /// Leads an organization.
    Leads,
    /// Member of an organization.
    MemberOf,
}

impl RelationshipType {
    /// Get the display name.
    pub fn name(&self) -> &'static str {
        match self {
            RelationshipType::Family => "family of",
            RelationshipType::Friend => "friend of",
            RelationshipType::Ally => "ally of",
            RelationshipType::Mentor => "mentor to",
            RelationshipType::Student => "student of",
            RelationshipType::Romantic => "romantic with",
            RelationshipType::Employer => "employer of",
            RelationshipType::Employee => "works for",
            RelationshipType::Acquaintance => "acquainted with",
            RelationshipType::Business => "does business with",
            RelationshipType::FellowMember => "fellow member with",
            RelationshipType::Rival => "rival of",
            RelationshipType::Enemy => "enemy of",
            RelationshipType::Betrayer => "betrayed",
            RelationshipType::Hunts => "hunting",
            RelationshipType::LivesAt => "lives at",
            RelationshipType::WorksAt => "works at",
            RelationshipType::Owns => "owns",
            RelationshipType::Created => "created",
            RelationshipType::Leads => "leads",
            RelationshipType::MemberOf => "member of",
        }
    }

    /// Check if this is a positive relationship.
    pub fn is_positive(&self) -> bool {
        matches!(
            self,
            RelationshipType::Family
                | RelationshipType::Friend
                | RelationshipType::Ally
                | RelationshipType::Mentor
                | RelationshipType::Student
                | RelationshipType::Romantic
        )
    }

    /// Check if this is a negative relationship.
    pub fn is_negative(&self) -> bool {
        matches!(
            self,
            RelationshipType::Rival
                | RelationshipType::Enemy
                | RelationshipType::Betrayer
                | RelationshipType::Hunts
        )
    }

    /// Get the inverse relationship type (if applicable).
    pub fn inverse(&self) -> Option<RelationshipType> {
        match self {
            RelationshipType::Mentor => Some(RelationshipType::Student),
            RelationshipType::Student => Some(RelationshipType::Mentor),
            RelationshipType::Employer => Some(RelationshipType::Employee),
            RelationshipType::Employee => Some(RelationshipType::Employer),
            RelationshipType::Leads => Some(RelationshipType::MemberOf),
            // Symmetric relationships
            RelationshipType::Family
            | RelationshipType::Friend
            | RelationshipType::Ally
            | RelationshipType::Romantic
            | RelationshipType::Acquaintance
            | RelationshipType::Business
            | RelationshipType::FellowMember
            | RelationshipType::Rival
            | RelationshipType::Enemy => Some(*self),
            // One-directional relationships
            _ => None,
        }
    }
}

/// A relationship between two entities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    /// The source entity.
    pub from_entity: EntityId,
    /// The target entity.
    pub to_entity: EntityId,
    /// Type of relationship.
    pub relationship_type: RelationshipType,
    /// Optional description or context.
    pub description: String,
    /// Strength of the relationship (-1.0 to 1.0, negative = hostile).
    pub strength: f32,
    /// When this relationship was established.
    pub established: StoryMoment,
    /// Whether this relationship is still active.
    pub is_active: bool,
}

impl Relationship {
    /// Create a new relationship.
    pub fn new(
        from_entity: EntityId,
        to_entity: EntityId,
        relationship_type: RelationshipType,
        current_turn: u32,
    ) -> Self {
        let default_strength = if relationship_type.is_positive() {
            0.5
        } else if relationship_type.is_negative() {
            -0.5
        } else {
            0.0
        };

        Self {
            from_entity,
            to_entity,
            relationship_type,
            description: String::new(),
            strength: default_strength,
            established: StoryMoment::new(current_turn),
            is_active: true,
        }
    }

    /// Set the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Set the strength.
    pub fn with_strength(mut self, strength: f32) -> Self {
        self.strength = strength.clamp(-1.0, 1.0);
        self
    }

    /// Adjust the strength.
    pub fn adjust_strength(&mut self, delta: f32) {
        self.strength = (self.strength + delta).clamp(-1.0, 1.0);
    }

    /// Mark the relationship as ended.
    pub fn end(&mut self) {
        self.is_active = false;
    }

    /// Check if this relationship involves a specific entity.
    pub fn involves(&self, entity_id: EntityId) -> bool {
        self.from_entity == entity_id || self.to_entity == entity_id
    }

    /// Get the other entity in the relationship.
    pub fn other(&self, entity_id: EntityId) -> Option<EntityId> {
        if self.from_entity == entity_id {
            Some(self.to_entity)
        } else if self.to_entity == entity_id {
            Some(self.from_entity)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relationship_creation() {
        let entity1 = EntityId::new();
        let entity2 = EntityId::new();

        let rel = Relationship::new(entity1, entity2, RelationshipType::Friend, 5)
            .with_description("Childhood friends");

        assert!(rel.involves(entity1));
        assert!(rel.involves(entity2));
        assert!(rel.strength > 0.0); // Positive relationship
        assert!(rel.is_active);
    }

    #[test]
    fn test_relationship_inverse() {
        assert_eq!(
            RelationshipType::Mentor.inverse(),
            Some(RelationshipType::Student)
        );
        assert_eq!(
            RelationshipType::Friend.inverse(),
            Some(RelationshipType::Friend)
        );
        assert_eq!(RelationshipType::Owns.inverse(), None);
    }
}
