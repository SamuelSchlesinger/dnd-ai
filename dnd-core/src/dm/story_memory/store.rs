//! Story memory store for entity and fact management.

#[cfg(test)]
use super::consequence::ConsequenceStatus;
use super::consequence::{Consequence, ConsequenceId, ConsequenceSeverity};
use super::entity::{Entity, EntityId, EntityType};
use super::fact::{FactCategory, FactSource, StoryFact};
use super::relationship::{Relationship, RelationshipType};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;

/// Maximum facts to include in context.
const MAX_CONTEXT_FACTS: usize = 30;

/// Maximum consequences to include in relevance checking.
const MAX_CONTEXT_CONSEQUENCES: usize = 20;

/// Importance decay rate per turn.
const IMPORTANCE_DECAY_PER_TURN: f32 = 0.02;

/// Consequence decay rate per turn (slower than facts).
const CONSEQUENCE_DECAY_PER_TURN: f32 = 0.01;

/// The main story memory store.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StoryMemory {
    /// All tracked entities.
    entities: HashMap<EntityId, Entity>,
    /// Name index for fast lookup.
    name_index: HashMap<String, EntityId>,
    /// All story facts.
    facts: Vec<StoryFact>,
    /// All relationships.
    relationships: Vec<Relationship>,
    /// All pending consequences.
    #[serde(default)]
    consequences: Vec<Consequence>,
    /// Current turn number.
    current_turn: u32,
}

impl StoryMemory {
    /// Create a new empty story memory.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the current turn.
    pub fn current_turn(&self) -> u32 {
        self.current_turn
    }

    /// Advance to the next turn (and decay importance).
    pub fn advance_turn(&mut self) {
        self.current_turn += 1;

        // Decay entity importance
        for entity in self.entities.values_mut() {
            entity.decay_importance(IMPORTANCE_DECAY_PER_TURN);
        }

        // Decay fact importance (slower for stable facts)
        for fact in &mut self.facts {
            let rate = if fact.category.is_stable() {
                IMPORTANCE_DECAY_PER_TURN * 0.5
            } else {
                IMPORTANCE_DECAY_PER_TURN
            };
            fact.decay_importance(rate);
        }

        // Check consequence expiry and decay importance
        for consequence in &mut self.consequences {
            consequence.check_expiry(self.current_turn);
            if consequence.status.is_active() {
                consequence.decay_importance(CONSEQUENCE_DECAY_PER_TURN);
            }
        }
    }

    // =========================================================================
    // Entity Management
    // =========================================================================

    /// Add a new entity to story memory.
    pub fn add_entity(&mut self, entity: Entity) -> EntityId {
        let id = entity.id;

        // Index by lowercase name
        self.name_index.insert(entity.name.to_lowercase(), id);

        // Also index aliases
        for alias in &entity.aliases {
            self.name_index.insert(alias.to_lowercase(), id);
        }

        self.entities.insert(id, entity);
        id
    }

    /// Create and add a new entity.
    pub fn create_entity(&mut self, entity_type: EntityType, name: impl Into<String>) -> EntityId {
        let entity = Entity::new(entity_type, name, self.current_turn);
        self.add_entity(entity)
    }

    /// Get an entity by ID.
    pub fn get_entity(&self, id: EntityId) -> Option<&Entity> {
        self.entities.get(&id)
    }

    /// Get a mutable entity by ID.
    pub fn get_entity_mut(&mut self, id: EntityId) -> Option<&mut Entity> {
        self.entities.get_mut(&id)
    }

    /// Find an entity by name (case-insensitive exact match).
    pub fn find_entity_by_name(&self, name: &str) -> Option<&Entity> {
        let lower = name.to_lowercase();
        self.name_index
            .get(&lower)
            .and_then(|id| self.entities.get(id))
    }

    /// Find an entity ID by name.
    pub fn find_entity_id(&self, name: &str) -> Option<EntityId> {
        self.name_index.get(&name.to_lowercase()).copied()
    }

    /// Find entities by partial name match.
    pub fn find_entities_partial(&self, query: &str) -> Vec<&Entity> {
        self.entities
            .values()
            .filter(|e| e.matches_partial(query))
            .collect()
    }

    /// Get or create an entity by name and type.
    pub fn get_or_create_entity(
        &mut self,
        entity_type: EntityType,
        name: impl Into<String>,
    ) -> EntityId {
        let name = name.into();
        if let Some(id) = self.find_entity_id(&name) {
            // Touch existing entity to boost importance
            if let Some(entity) = self.entities.get_mut(&id) {
                entity.touch(self.current_turn);
            }
            id
        } else {
            self.create_entity(entity_type, name)
        }
    }

    /// Touch an entity (update last_seen and boost importance).
    pub fn touch_entity(&mut self, id: EntityId) {
        if let Some(entity) = self.entities.get_mut(&id) {
            entity.touch(self.current_turn);
        }
    }

    /// Get all entities of a specific type.
    pub fn entities_of_type(&self, entity_type: EntityType) -> Vec<&Entity> {
        self.entities
            .values()
            .filter(|e| e.entity_type == entity_type)
            .collect()
    }

    /// Get all entities, sorted by importance.
    pub fn all_entities_by_importance(&self) -> Vec<&Entity> {
        let mut entities: Vec<_> = self.entities.values().collect();
        entities.sort_by(|a, b| {
            b.importance
                .partial_cmp(&a.importance)
                .unwrap_or(Ordering::Equal)
        });
        entities
    }

    // =========================================================================
    // Fact Management
    // =========================================================================

    /// Add a story fact.
    pub fn add_fact(&mut self, fact: StoryFact) {
        // Touch the subject entity
        self.touch_entity(fact.subject);

        // Touch mentioned entities
        for &entity_id in &fact.mentioned_entities {
            self.touch_entity(entity_id);
        }

        self.facts.push(fact);
    }

    /// Create and add a fact about an entity.
    pub fn record_fact(
        &mut self,
        subject_id: EntityId,
        content: impl Into<String>,
        category: FactCategory,
        source: FactSource,
    ) {
        let fact = StoryFact::new(subject_id, content, category, source, self.current_turn);
        self.add_fact(fact);
    }

    /// Record a fact with mentioned entities.
    pub fn record_fact_with_mentions(
        &mut self,
        subject_id: EntityId,
        content: impl Into<String>,
        category: FactCategory,
        source: FactSource,
        mentioned: &[EntityId],
    ) {
        let mut fact = StoryFact::new(subject_id, content, category, source, self.current_turn);
        for &id in mentioned {
            fact = fact.with_mentioned(id);
        }
        self.add_fact(fact);
    }

    /// Record a fact with mentioned entities and custom importance.
    pub fn record_fact_full(
        &mut self,
        subject_id: EntityId,
        content: impl Into<String>,
        category: FactCategory,
        source: FactSource,
        mentioned: &[EntityId],
        importance: f32,
    ) {
        let mut fact = StoryFact::new(subject_id, content, category, source, self.current_turn)
            .with_importance(importance);
        for &id in mentioned {
            fact = fact.with_mentioned(id);
        }
        self.add_fact(fact);
    }

    /// Get all facts about an entity.
    pub fn facts_about(&self, entity_id: EntityId) -> Vec<&StoryFact> {
        self.facts
            .iter()
            .filter(|f| f.involves(entity_id) && f.is_current)
            .collect()
    }

    /// Get facts by category.
    pub fn facts_by_category(&self, category: FactCategory) -> Vec<&StoryFact> {
        self.facts
            .iter()
            .filter(|f| f.category == category && f.is_current)
            .collect()
    }

    /// Get recent facts (within N turns).
    pub fn recent_facts(&self, within_turns: u32) -> Vec<&StoryFact> {
        let min_turn = self.current_turn.saturating_sub(within_turns);
        self.facts
            .iter()
            .filter(|f| f.established.turn >= min_turn && f.is_current)
            .collect()
    }

    // =========================================================================
    // Relationship Management
    // =========================================================================

    /// Add a relationship.
    pub fn add_relationship(&mut self, relationship: Relationship) {
        self.relationships.push(relationship);
    }

    /// Create and add a relationship between entities.
    pub fn create_relationship(
        &mut self,
        from_id: EntityId,
        to_id: EntityId,
        relationship_type: RelationshipType,
    ) {
        let rel = Relationship::new(from_id, to_id, relationship_type, self.current_turn);
        self.add_relationship(rel);
    }

    /// Get all relationships involving an entity.
    pub fn relationships_of(&self, entity_id: EntityId) -> Vec<&Relationship> {
        self.relationships
            .iter()
            .filter(|r| r.involves(entity_id) && r.is_active)
            .collect()
    }

    /// Find a specific relationship between two entities.
    pub fn find_relationship(&self, from_id: EntityId, to_id: EntityId) -> Option<&Relationship> {
        self.relationships
            .iter()
            .find(|r| r.from_entity == from_id && r.to_entity == to_id && r.is_active)
    }

    /// Get a mutable relationship.
    pub fn find_relationship_mut(
        &mut self,
        from_id: EntityId,
        to_id: EntityId,
    ) -> Option<&mut Relationship> {
        self.relationships
            .iter_mut()
            .find(|r| r.from_entity == from_id && r.to_entity == to_id && r.is_active)
    }

    // =========================================================================
    // Consequence Management
    // =========================================================================

    /// Add a consequence.
    pub fn add_consequence(&mut self, consequence: Consequence) -> ConsequenceId {
        let id = consequence.id;
        self.consequences.push(consequence);
        id
    }

    /// Create and add a new consequence.
    pub fn create_consequence(
        &mut self,
        trigger_description: impl Into<String>,
        consequence_description: impl Into<String>,
        severity: ConsequenceSeverity,
    ) -> ConsequenceId {
        let consequence = Consequence::new(
            trigger_description,
            consequence_description,
            severity,
            self.current_turn,
        );
        self.add_consequence(consequence)
    }

    /// Create a consequence with an expiry.
    pub fn create_consequence_with_expiry(
        &mut self,
        trigger_description: impl Into<String>,
        consequence_description: impl Into<String>,
        severity: ConsequenceSeverity,
        expires_in_turns: u32,
    ) -> ConsequenceId {
        let consequence = Consequence::new(
            trigger_description,
            consequence_description,
            severity,
            self.current_turn,
        )
        .with_expiry(self.current_turn + expires_in_turns);
        self.add_consequence(consequence)
    }

    /// Get a consequence by ID.
    pub fn get_consequence(&self, id: ConsequenceId) -> Option<&Consequence> {
        self.consequences.iter().find(|c| c.id == id)
    }

    /// Get a mutable consequence by ID.
    pub fn get_consequence_mut(&mut self, id: ConsequenceId) -> Option<&mut Consequence> {
        self.consequences.iter_mut().find(|c| c.id == id)
    }

    /// Get all pending (active) consequences.
    pub fn pending_consequences(&self) -> Vec<&Consequence> {
        self.consequences
            .iter()
            .filter(|c| c.status.is_active())
            .collect()
    }

    /// Get pending consequences sorted by importance.
    pub fn pending_consequences_by_importance(&self) -> Vec<&Consequence> {
        let mut consequences: Vec<_> = self.pending_consequences();
        consequences.sort_by(|a, b| {
            b.importance
                .partial_cmp(&a.importance)
                .unwrap_or(Ordering::Equal)
        });
        consequences
    }

    /// Get consequences involving a specific entity.
    pub fn consequences_involving(&self, entity_id: EntityId) -> Vec<&Consequence> {
        self.consequences
            .iter()
            .filter(|c| c.status.is_active() && c.involves(entity_id))
            .collect()
    }

    /// Mark a consequence as triggered.
    pub fn trigger_consequence(&mut self, id: ConsequenceId) -> bool {
        if let Some(consequence) = self.get_consequence_mut(id) {
            consequence.trigger();
            true
        } else {
            false
        }
    }

    /// Mark a consequence as resolved (handled without triggering).
    pub fn resolve_consequence(&mut self, id: ConsequenceId) -> bool {
        if let Some(consequence) = self.get_consequence_mut(id) {
            consequence.resolve();
            true
        } else {
            false
        }
    }

    /// Get the total number of consequences (all statuses).
    pub fn consequence_count(&self) -> usize {
        self.consequences.len()
    }

    /// Get the number of pending consequences.
    pub fn pending_consequence_count(&self) -> usize {
        self.consequences
            .iter()
            .filter(|c| c.status.is_active())
            .count()
    }

    /// Build context string for pending consequences.
    /// This is used by the relevance checker.
    pub fn build_consequences_for_relevance(&self) -> String {
        let pending = self.pending_consequences_by_importance();
        if pending.is_empty() {
            return String::new();
        }

        let mut context = String::new();
        for (i, consequence) in pending.iter().take(MAX_CONTEXT_CONSEQUENCES).enumerate() {
            context.push_str(&format!(
                "{}. [{}] TRIGGER: {} -> EFFECT: {}\n",
                i + 1,
                consequence.id,
                consequence.trigger_description,
                consequence.consequence_description
            ));
        }
        context
    }

    // =========================================================================
    // Context Building
    // =========================================================================

    /// Extract entity names mentioned in text.
    ///
    /// Names are matched at word boundaries only, so "Thor" will match in
    /// "I ask Thor about the hammer" but not in "I ask Thorin about the ring".
    pub fn extract_mentioned_entities(&self, text: &str) -> Vec<EntityId> {
        let text_lower = text.to_lowercase();
        let mut found = Vec::new();

        for (name, &id) in &self.name_index {
            if contains_word(&text_lower, name) && !found.contains(&id) {
                found.push(id);
            }
        }

        found
    }
}

/// Check if `text` contains `word` at word boundaries.
///
/// A word boundary is the start/end of string or a non-alphanumeric character.
/// This handles multi-word names correctly (e.g., "Old Tom" matches as a phrase).
fn contains_word(text: &str, word: &str) -> bool {
    if word.is_empty() {
        return false;
    }

    let text_bytes = text.as_bytes();
    let word_bytes = word.as_bytes();
    let text_len = text_bytes.len();
    let word_len = word_bytes.len();

    if word_len > text_len {
        return false;
    }

    // Scan through text looking for the word
    let mut i = 0;
    while i + word_len <= text_len {
        // Check if word matches at position i
        if &text_bytes[i..i + word_len] == word_bytes {
            // Check left boundary: start of string or non-alphanumeric
            let left_ok = i == 0 || !is_alphanumeric(text_bytes[i - 1]);

            // Check right boundary: end of string or non-alphanumeric
            let right_ok = i + word_len == text_len || !is_alphanumeric(text_bytes[i + word_len]);

            if left_ok && right_ok {
                return true;
            }
        }
        i += 1;
    }

    false
}

/// Check if a byte is alphanumeric (a-z, A-Z, 0-9).
fn is_alphanumeric(b: u8) -> bool {
    b.is_ascii_alphanumeric()
}

impl StoryMemory {
    /// Build context string for entities mentioned in input.
    pub fn build_context_for_input(&self, input: &str) -> String {
        let mentioned = self.extract_mentioned_entities(input);
        self.build_relevant_context(&mentioned)
    }

    /// Build context string for specific entities.
    pub fn build_relevant_context(&self, entity_ids: &[EntityId]) -> String {
        if entity_ids.is_empty() {
            return String::new();
        }

        let mut context = String::new();
        context.push_str("## Relevant Story Context\n\n");

        // Gather facts about mentioned entities
        let mut relevant_facts: Vec<(&StoryFact, f32)> = Vec::new();

        for &entity_id in entity_ids {
            for fact in &self.facts {
                if fact.involves(entity_id) && fact.is_current {
                    // Calculate relevance score
                    let recency_bonus = if fact.established.turn + 10 >= self.current_turn {
                        0.3
                    } else {
                        0.0
                    };
                    let score = fact.importance + recency_bonus;

                    // Check if already added
                    if !relevant_facts.iter().any(|(f, _)| f.id == fact.id) {
                        relevant_facts.push((fact, score));
                    }
                }
            }
        }

        // Sort by relevance score
        relevant_facts.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));

        // Take top facts
        let top_facts: Vec<_> = relevant_facts
            .into_iter()
            .take(MAX_CONTEXT_FACTS)
            .map(|(f, _)| f)
            .collect();

        if top_facts.is_empty() {
            return String::new();
        }

        // Group facts by entity
        let mut by_entity: HashMap<EntityId, Vec<&StoryFact>> = HashMap::new();
        for fact in &top_facts {
            by_entity.entry(fact.subject).or_default().push(fact);
        }

        // Write context grouped by entity
        for (entity_id, entity_facts) in by_entity {
            if let Some(entity) = self.entities.get(&entity_id) {
                context.push_str(&format!(
                    "### {} ({})\n",
                    entity.name,
                    entity.entity_type.name()
                ));

                for fact in entity_facts {
                    context.push_str(&format!("- {}\n", fact.content));
                }

                // Add relationships
                let rels = self.relationships_of(entity_id);
                if !rels.is_empty() {
                    for rel in rels.iter().take(3) {
                        if let Some(other) = self
                            .entities
                            .get(&rel.other(entity_id).unwrap_or(entity_id))
                        {
                            if !rel.description.is_empty() {
                                context.push_str(&format!(
                                    "- {} {} ({})\n",
                                    rel.relationship_type.name(),
                                    other.name,
                                    rel.description
                                ));
                            } else {
                                context.push_str(&format!(
                                    "- {} {}\n",
                                    rel.relationship_type.name(),
                                    other.name
                                ));
                            }
                        }
                    }
                }

                context.push('\n');
            }
        }

        context
    }

    /// Build a summary of important story elements.
    pub fn build_summary(&self) -> String {
        let mut summary = String::new();

        // Top NPCs
        let npcs: Vec<_> = self
            .entities_of_type(EntityType::Npc)
            .into_iter()
            .take(5)
            .collect();

        if !npcs.is_empty() {
            summary.push_str("### Key NPCs\n");
            for npc in npcs {
                summary.push_str(&format!("- **{}**", npc.name));
                if let Some(desc) = &npc.description {
                    summary.push_str(&format!(": {desc}"));
                }
                summary.push('\n');
            }
            summary.push('\n');
        }

        // Key locations
        let locations: Vec<_> = self
            .entities_of_type(EntityType::Location)
            .into_iter()
            .take(3)
            .collect();

        if !locations.is_empty() {
            summary.push_str("### Notable Locations\n");
            for loc in locations {
                summary.push_str(&format!("- **{}**", loc.name));
                if let Some(desc) = &loc.description {
                    summary.push_str(&format!(": {desc}"));
                }
                summary.push('\n');
            }
            summary.push('\n');
        }

        // Active quests
        let quests: Vec<_> = self
            .entities_of_type(EntityType::Quest)
            .into_iter()
            .filter(|e| e.importance > 0.3)
            .collect();

        if !quests.is_empty() {
            summary.push_str("### Active Quests\n");
            for quest in quests {
                summary.push_str(&format!("- **{}**", quest.name));
                if let Some(desc) = &quest.description {
                    summary.push_str(&format!(": {desc}"));
                }
                summary.push('\n');
            }
            summary.push('\n');
        }

        // Recent events
        let recent_events = self.recent_facts(5);
        let event_facts: Vec<_> = recent_events
            .iter()
            .filter(|f| f.category == FactCategory::Event)
            .take(5)
            .collect();

        if !event_facts.is_empty() {
            summary.push_str("### Recent Events\n");
            for fact in event_facts {
                summary.push_str(&format!("- {}\n", fact.content));
            }
            summary.push('\n');
        }

        summary
    }

    // =========================================================================
    // Statistics and Debug
    // =========================================================================

    /// Get the total number of entities.
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    /// Get the total number of facts.
    pub fn fact_count(&self) -> usize {
        self.facts.len()
    }

    /// Get the total number of relationships.
    pub fn relationship_count(&self) -> usize {
        self.relationships.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_creation() {
        let store = StoryMemory::new();
        assert_eq!(store.entity_count(), 0);
        assert_eq!(store.fact_count(), 0);
    }

    #[test]
    fn test_entity_management() {
        let mut store = StoryMemory::new();

        let gandalf_id = store.create_entity(EntityType::Npc, "Gandalf");
        assert!(store.get_entity(gandalf_id).is_some());
        assert!(store.find_entity_by_name("gandalf").is_some());
        assert!(store.find_entity_by_name("GANDALF").is_some());
    }

    #[test]
    fn test_get_or_create() {
        let mut store = StoryMemory::new();

        let id1 = store.get_or_create_entity(EntityType::Npc, "Frodo");
        let id2 = store.get_or_create_entity(EntityType::Npc, "Frodo");

        assert_eq!(id1, id2);
        assert_eq!(store.entity_count(), 1);
    }

    #[test]
    fn test_fact_recording() {
        let mut store = StoryMemory::new();

        let gandalf_id = store.create_entity(EntityType::Npc, "Gandalf");
        store.record_fact(
            gandalf_id,
            "Gandalf wears a grey cloak",
            FactCategory::Appearance,
            FactSource::DmNarration,
        );

        let facts = store.facts_about(gandalf_id);
        assert_eq!(facts.len(), 1);
        assert!(facts[0].content.contains("grey cloak"));
    }

    #[test]
    fn test_entity_mention_extraction() {
        let mut store = StoryMemory::new();

        store.create_entity(EntityType::Npc, "Gandalf");
        store.create_entity(EntityType::Location, "Moria");

        let text = "I want to ask Gandalf about the path through Moria";
        let mentioned = store.extract_mentioned_entities(text);

        assert_eq!(mentioned.len(), 2);
    }

    #[test]
    fn test_entity_mention_word_boundaries() {
        let mut store = StoryMemory::new();

        let thor_id = store.create_entity(EntityType::Npc, "Thor");
        let ian_id = store.create_entity(EntityType::Npc, "Ian");
        let old_tom_id = store.create_entity(EntityType::Npc, "Old Tom");

        // "Thor" should match - name at word boundaries
        let mentioned = store.extract_mentioned_entities("I ask Thor about the hammer");
        assert!(
            mentioned.contains(&thor_id),
            "Thor should match in 'I ask Thor about the hammer'"
        );

        // "Thor" should NOT match in "Thorin" - partial word match
        let mentioned = store.extract_mentioned_entities("I ask Thorin about the ring");
        assert!(
            !mentioned.contains(&thor_id),
            "Thor should NOT match in 'I ask Thorin about the ring'"
        );

        // "Ian" should NOT match in "Christian"
        let mentioned = store.extract_mentioned_entities("Christian is here");
        assert!(
            !mentioned.contains(&ian_id),
            "Ian should NOT match in 'Christian is here'"
        );

        // "Ian" should match when it's a whole word
        let mentioned = store.extract_mentioned_entities("Ian is here");
        assert!(
            mentioned.contains(&ian_id),
            "Ian should match in 'Ian is here'"
        );

        // Multi-word names should match as whole phrases
        let mentioned = store.extract_mentioned_entities("I visit Old Tom at the tavern");
        assert!(
            mentioned.contains(&old_tom_id),
            "Old Tom should match in 'I visit Old Tom at the tavern'"
        );

        // Name at start of string
        let mentioned = store.extract_mentioned_entities("Thor is mighty");
        assert!(
            mentioned.contains(&thor_id),
            "Thor should match at start of string"
        );

        // Name at end of string
        let mentioned = store.extract_mentioned_entities("I speak to Thor");
        assert!(
            mentioned.contains(&thor_id),
            "Thor should match at end of string"
        );

        // Name with punctuation boundary
        let mentioned = store.extract_mentioned_entities("Thor, the god of thunder");
        assert!(
            mentioned.contains(&thor_id),
            "Thor should match before comma"
        );

        // Case insensitivity still works
        let mentioned = store.extract_mentioned_entities("I ask THOR about lightning");
        assert!(
            mentioned.contains(&thor_id),
            "Thor should match case-insensitively"
        );
    }

    #[test]
    fn test_contains_word_helper() {
        // Test the helper function directly
        assert!(contains_word("hello world", "hello"));
        assert!(contains_word("hello world", "world"));
        assert!(!contains_word("helloworld", "hello"));
        assert!(!contains_word("worldly", "world"));
        assert!(contains_word("hello, world!", "world"));
        assert!(contains_word("world", "world")); // exact match
        assert!(!contains_word("wor", "world")); // word longer than text
        assert!(!contains_word("hello", "")); // empty word
    }

    #[test]
    fn test_relationship_creation() {
        let mut store = StoryMemory::new();

        let gandalf_id = store.create_entity(EntityType::Npc, "Gandalf");
        let frodo_id = store.create_entity(EntityType::Npc, "Frodo");

        store.create_relationship(gandalf_id, frodo_id, RelationshipType::Mentor);

        let gandalf_rels = store.relationships_of(gandalf_id);
        assert_eq!(gandalf_rels.len(), 1);

        let frodo_rels = store.relationships_of(frodo_id);
        assert_eq!(frodo_rels.len(), 1);
    }

    #[test]
    fn test_context_building() {
        let mut store = StoryMemory::new();

        let gandalf_id = store.create_entity(EntityType::Npc, "Gandalf");
        store.record_fact(
            gandalf_id,
            "Gandalf is a powerful wizard",
            FactCategory::Capability,
            FactSource::DmNarration,
        );

        let context = store.build_context_for_input("I speak to Gandalf");
        assert!(context.contains("Gandalf"));
        assert!(context.contains("powerful wizard"));
    }

    #[test]
    fn test_consequence_creation() {
        let mut store = StoryMemory::new();

        let id = store.create_consequence(
            "Player enters Riverside",
            "Guards attempt arrest",
            ConsequenceSeverity::Major,
        );

        assert_eq!(store.consequence_count(), 1);
        assert_eq!(store.pending_consequence_count(), 1);

        let consequence = store.get_consequence(id).unwrap();
        assert!(consequence.status.is_active());
        assert_eq!(consequence.severity, ConsequenceSeverity::Major);
    }

    #[test]
    fn test_consequence_trigger() {
        let mut store = StoryMemory::new();

        let id = store.create_consequence(
            "Player enters tavern",
            "Bounty hunter attacks",
            ConsequenceSeverity::Critical,
        );

        assert!(store.trigger_consequence(id));
        assert_eq!(store.pending_consequence_count(), 0);

        let consequence = store.get_consequence(id).unwrap();
        assert_eq!(consequence.status, ConsequenceStatus::Triggered);
    }

    #[test]
    fn test_consequence_expiry() {
        let mut store = StoryMemory::new();

        let id = store.create_consequence_with_expiry(
            "Wolves are hunting in the forest",
            "Wolves attack",
            ConsequenceSeverity::Moderate,
            5, // Expires in 5 turns
        );

        // Advance time but not enough to expire
        for _ in 0..4 {
            store.advance_turn();
        }
        assert_eq!(store.pending_consequence_count(), 1);

        // One more turn - should expire
        store.advance_turn();
        assert_eq!(store.pending_consequence_count(), 0);

        let consequence = store.get_consequence(id).unwrap();
        assert_eq!(consequence.status, ConsequenceStatus::Expired);
    }

    #[test]
    fn test_consequences_by_importance() {
        let mut store = StoryMemory::new();

        store.create_consequence("Minor trigger", "Minor effect", ConsequenceSeverity::Minor);
        store.create_consequence(
            "Critical trigger",
            "Critical effect",
            ConsequenceSeverity::Critical,
        );
        store.create_consequence(
            "Moderate trigger",
            "Moderate effect",
            ConsequenceSeverity::Moderate,
        );

        let sorted = store.pending_consequences_by_importance();
        assert_eq!(sorted.len(), 3);
        // Critical should be first (highest importance)
        assert_eq!(sorted[0].severity, ConsequenceSeverity::Critical);
        // Minor should be last (lowest importance)
        assert_eq!(sorted[2].severity, ConsequenceSeverity::Minor);
    }

    #[test]
    fn test_consequence_involving_entity() {
        let mut store = StoryMemory::new();

        let npc_id = store.create_entity(EntityType::Npc, "Baron Aldric");

        let consequence = Consequence::new(
            "Player enters Riverside",
            "Baron's guards arrest player",
            ConsequenceSeverity::Major,
            store.current_turn(),
        )
        .with_subject(npc_id);

        store.add_consequence(consequence);

        let involving = store.consequences_involving(npc_id);
        assert_eq!(involving.len(), 1);
    }
}
