//! Testing utilities for the D&D game.
//!
//! This module provides tools for integration testing:
//! - `MockDm` for deterministic testing without API calls
//! - `TestHarness` for scripted game scenarios
//! - Assertion helpers for verifying game state

use crate::dm::{DmResponse, StoryMemory};
use crate::rules::{Intent, RulesEngine};
use crate::world::{create_sample_fighter, Character, GameWorld, NarrativeType};

/// A mock DM that returns scripted responses.
///
/// Use this for deterministic integration tests without API calls.
pub struct MockDm {
    /// Scripted responses to return in order.
    responses: Vec<MockResponse>,
    /// Index of next response to return.
    response_index: usize,
    /// Story memory (shared with real DM API).
    story_memory: StoryMemory,
    /// Rules engine for resolving intents.
    rules: RulesEngine,
}

/// A scripted response from the mock DM.
#[derive(Debug, Clone)]
pub struct MockResponse {
    /// Narrative text to return.
    pub narrative: String,
    /// Intents to execute (will be resolved by rules engine).
    pub intents: Vec<Intent>,
}

impl MockResponse {
    /// Create a simple narrative response with no mechanical effects.
    pub fn narrative(text: impl Into<String>) -> Self {
        Self {
            narrative: text.into(),
            intents: Vec::new(),
        }
    }

    /// Create a response with intents that will be resolved.
    pub fn with_intents(text: impl Into<String>, intents: Vec<Intent>) -> Self {
        Self {
            narrative: text.into(),
            intents,
        }
    }
}

impl MockDm {
    /// Create a new mock DM with scripted responses.
    pub fn new(responses: Vec<MockResponse>) -> Self {
        Self {
            responses,
            response_index: 0,
            story_memory: StoryMemory::new(),
            rules: RulesEngine::new(),
        }
    }

    /// Process input and return the next scripted response.
    ///
    /// Intents are resolved through the real rules engine.
    pub fn process_input(&mut self, _input: &str, world: &mut GameWorld) -> DmResponse {
        // Advance story memory turn
        self.story_memory.advance_turn();

        // Get next response or return default
        let response = if self.response_index < self.responses.len() {
            let r = self.responses[self.response_index].clone();
            self.response_index += 1;
            r
        } else {
            MockResponse::narrative("The DM has no more scripted responses.")
        };

        // Resolve intents through rules engine
        let mut all_effects = Vec::new();
        let mut all_resolutions = Vec::new();

        for intent in &response.intents {
            let resolution = self.rules.resolve(world, intent.clone());
            crate::rules::apply_effects(world, &resolution.effects);
            all_effects.extend(resolution.effects.clone());
            all_resolutions.push(resolution);
        }

        // Add narrative to world
        world.add_narrative(response.narrative.clone(), NarrativeType::DmNarration);

        DmResponse {
            narrative: response.narrative,
            intents: response.intents,
            effects: all_effects,
            resolutions: all_resolutions,
        }
    }

    /// Get the story memory.
    pub fn story_memory(&self) -> &StoryMemory {
        &self.story_memory
    }

    /// Get mutable story memory.
    pub fn story_memory_mut(&mut self) -> &mut StoryMemory {
        &mut self.story_memory
    }

    /// Add a response to the queue.
    pub fn queue_response(&mut self, response: MockResponse) {
        self.responses.push(response);
    }

    /// Reset the response index to replay from the beginning.
    pub fn reset(&mut self) {
        self.response_index = 0;
    }
}

/// Test harness for running game scenarios.
pub struct TestHarness {
    /// The mock DM.
    pub dm: MockDm,
    /// The game world.
    pub world: GameWorld,
}

impl TestHarness {
    /// Create a new test harness with a sample character.
    pub fn new() -> Self {
        let character = create_sample_fighter("Test Hero");
        let world = GameWorld::new("Test Campaign", character);
        let dm = MockDm::new(Vec::new());

        Self { dm, world }
    }

    /// Create a test harness with a custom character.
    pub fn with_character(character: Character) -> Self {
        let world = GameWorld::new("Test Campaign", character);
        let dm = MockDm::new(Vec::new());

        Self { dm, world }
    }

    /// Queue a narrative response.
    pub fn expect_narrative(&mut self, text: impl Into<String>) -> &mut Self {
        self.dm.queue_response(MockResponse::narrative(text));
        self
    }

    /// Queue a response with intents.
    pub fn expect_response(&mut self, response: MockResponse) -> &mut Self {
        self.dm.queue_response(response);
        self
    }

    /// Send player input and get response.
    pub fn input(&mut self, text: &str) -> DmResponse {
        // Add player input to world narrative
        self.world
            .add_narrative(text.to_string(), NarrativeType::PlayerAction);

        // Process through mock DM
        self.dm.process_input(text, &mut self.world)
    }

    /// Get current player HP as (current, max).
    pub fn player_hp(&self) -> (i32, i32) {
        let hp = &self.world.player_character.hit_points;
        (hp.current, hp.maximum)
    }

    /// Check if player has a condition.
    pub fn player_has_condition(&self, condition: crate::world::Condition) -> bool {
        self.world
            .player_character
            .conditions
            .iter()
            .any(|c| c.condition == condition)
    }

    /// Check if in combat.
    pub fn in_combat(&self) -> bool {
        self.world.combat.is_some()
    }

    /// Get story memory entity count.
    pub fn entity_count(&self) -> usize {
        self.dm.story_memory().entity_count()
    }

    /// Get story memory fact count.
    pub fn fact_count(&self) -> usize {
        self.dm.story_memory().fact_count()
    }

    /// Check if an entity exists by name.
    pub fn has_entity(&self, name: &str) -> bool {
        self.dm.story_memory().find_entity_by_name(name).is_some()
    }

    /// Get the last narrative entry.
    pub fn last_narrative(&self) -> Option<&str> {
        self.world
            .narrative_history
            .last()
            .map(|e| e.content.as_str())
    }
}

impl Default for TestHarness {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Assertion Helpers
// ============================================================================

/// Assert that story memory contains an entity with the given name.
#[track_caller]
pub fn assert_has_entity(harness: &TestHarness, name: &str) {
    assert!(
        harness.has_entity(name),
        "Expected entity '{name}' to exist in story memory"
    );
}

/// Assert that story memory does NOT contain an entity with the given name.
#[track_caller]
pub fn assert_no_entity(harness: &TestHarness, name: &str) {
    assert!(
        !harness.has_entity(name),
        "Expected entity '{name}' to NOT exist in story memory"
    );
}

/// Assert player HP is at expected values.
#[track_caller]
pub fn assert_hp(harness: &TestHarness, current: i32, max: i32) {
    let (actual_current, actual_max) = harness.player_hp();
    assert_eq!(
        (actual_current, actual_max),
        (current, max),
        "Expected HP {current}/{max}, got {actual_current}/{actual_max}"
    );
}

/// Assert player is in combat.
#[track_caller]
pub fn assert_in_combat(harness: &TestHarness) {
    assert!(harness.in_combat(), "Expected to be in combat");
}

/// Assert player is NOT in combat.
#[track_caller]
pub fn assert_not_in_combat(harness: &TestHarness) {
    assert!(!harness.in_combat(), "Expected to NOT be in combat");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::Intent;
    use crate::dice::Advantage;
    use crate::rules::DamageType;
    use crate::world::Skill;

    #[test]
    fn test_mock_dm_basic() {
        let mut harness = TestHarness::new();
        harness.expect_narrative("You stand in a dusty tavern.");

        let response = harness.input("I look around");

        assert_eq!(response.narrative, "You stand in a dusty tavern.");
        assert!(response.intents.is_empty());
    }

    #[test]
    fn test_mock_dm_with_damage() {
        let mut harness = TestHarness::new();
        let initial_hp = harness.player_hp();

        harness.expect_response(MockResponse::with_intents(
            "The goblin stabs you!",
            vec![Intent::Damage {
                target_id: harness.world.player_character.id,
                amount: 5,
                damage_type: DamageType::Piercing,
                source: "Goblin dagger".to_string(),
            }],
        ));

        let response = harness.input("I approach the goblin");

        assert!(response.narrative.contains("goblin stabs"));
        assert_eq!(harness.player_hp().0, initial_hp.0 - 5);
    }

    #[test]
    fn test_mock_dm_skill_check() {
        let mut harness = TestHarness::new();

        harness.expect_response(MockResponse::with_intents(
            "You attempt to sneak past the guard...",
            vec![Intent::SkillCheck {
                character_id: harness.world.player_character.id,
                skill: Skill::Stealth,
                dc: 12,
                advantage: Advantage::Normal,
                description: "Sneaking past the guard".to_string(),
            }],
        ));

        let response = harness.input("I try to sneak past");

        // Should have resolved the skill check
        assert!(!response.effects.is_empty());
    }

    #[test]
    fn test_harness_combat_flow() {
        use crate::rules::CombatantInit;
        use crate::world::CharacterId;

        let mut harness = TestHarness::new();

        // Start combat
        harness.expect_response(MockResponse::with_intents(
            "A goblin leaps from the shadows!",
            vec![Intent::StartCombat {
                combatants: vec![
                    CombatantInit {
                        id: harness.world.player_character.id,
                        name: "Hero".to_string(),
                        is_player: true,
                        is_ally: true,
                        current_hp: 10,
                        max_hp: 10,
                    },
                    CombatantInit {
                        id: CharacterId::new(),
                        name: "Goblin".to_string(),
                        is_player: false,
                        is_ally: false,
                        current_hp: 7,
                        max_hp: 7,
                    },
                ],
            }],
        ));

        assert_not_in_combat(&harness);
        harness.input("I enter the cave");
        assert_in_combat(&harness);

        // End combat
        harness.expect_response(MockResponse::with_intents(
            "The goblin falls!",
            vec![Intent::EndCombat],
        ));

        harness.input("I strike the goblin");
        assert_not_in_combat(&harness);
    }

    #[test]
    fn test_story_memory_integration() {
        use crate::dm::story_memory::{EntityType, FactCategory, FactSource};

        let mut harness = TestHarness::new();

        // Manually add entity and fact (simulating what DM would do)
        let mira_id = harness
            .dm
            .story_memory_mut()
            .create_entity(EntityType::Npc, "Mira");

        harness.dm.story_memory_mut().record_fact(
            mira_id,
            "Mira is a nervous half-elf herbalist",
            FactCategory::Personality,
            FactSource::DmNarration,
        );

        assert_has_entity(&harness, "Mira");
        assert_eq!(harness.entity_count(), 1);
        assert_eq!(harness.fact_count(), 1);

        // Test context retrieval
        let context = harness
            .dm
            .story_memory()
            .build_context_for_input("I want to talk to Mira");
        assert!(context.contains("Mira"));
        assert!(context.contains("nervous half-elf"));
    }

    #[test]
    fn test_multiple_responses() {
        let mut harness = TestHarness::new();

        harness
            .expect_narrative("Response 1")
            .expect_narrative("Response 2")
            .expect_narrative("Response 3");

        assert_eq!(harness.input("first").narrative, "Response 1");
        assert_eq!(harness.input("second").narrative, "Response 2");
        assert_eq!(harness.input("third").narrative, "Response 3");

        // After scripted responses exhausted, get default
        assert!(harness
            .input("fourth")
            .narrative
            .contains("no more scripted"));
    }
}
