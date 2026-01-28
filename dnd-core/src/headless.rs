//! Headless game interface for programmatic use.
//!
//! This module provides a simplified interface for running D&D games without
//! a TUI. It's designed for:
//! - Automated testing with real AI responses
//! - Coding agents playing the game
//! - Script-driven game sessions
//!
//! # Example
//!
//! ```ignore
//! use dnd_core::headless::{HeadlessGame, HeadlessConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = HeadlessConfig::quick_start("Thorin");
//!     let mut game = HeadlessGame::new(config).await?;
//!
//!     // Play the game
//!     let response = game.send("I look around the tavern").await?;
//!     println!("{}", response.narrative);
//!
//!     // Check game state
//!     println!("HP: {}/{}", game.current_hp(), game.max_hp());
//!     println!("In combat: {}", game.in_combat());
//!
//!     // Save progress
//!     game.save("my_game.json").await?;
//!
//!     Ok(())
//! }
//! ```

use crate::character_builder::{roll_ability_scores, AbilityMethod, CharacterBuilder, STANDARD_ARRAY};
use crate::session::{GameSession, SessionConfig, SessionError};
use crate::world::{Ability, AbilityScores, Background, Character, CharacterClass, Condition, RaceType};
use std::path::Path;

/// Configuration for a headless game session.
#[derive(Debug, Clone)]
pub struct HeadlessConfig {
    /// Character name.
    pub name: String,
    /// Character race.
    pub race: RaceType,
    /// Character class.
    pub class: CharacterClass,
    /// Character background.
    pub background: Background,
    /// How ability scores are determined.
    pub ability_method: AbilityMethod,
    /// Campaign name.
    pub campaign_name: String,
    /// Starting location.
    pub starting_location: String,
}

impl HeadlessConfig {
    /// Create a quick-start configuration with sensible defaults.
    ///
    /// Uses Human Fighter with Folk Hero background and standard array.
    pub fn quick_start(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            race: RaceType::Human,
            class: CharacterClass::Fighter,
            background: Background::FolkHero,
            ability_method: AbilityMethod::StandardArray,
            campaign_name: "Headless Adventure".to_string(),
            starting_location: "The Crossroads Inn".to_string(),
        }
    }

    /// Create a configuration with full customization.
    pub fn custom(
        name: impl Into<String>,
        race: RaceType,
        class: CharacterClass,
        background: Background,
    ) -> Self {
        Self {
            name: name.into(),
            race,
            class,
            background,
            ability_method: AbilityMethod::StandardArray,
            campaign_name: "Headless Adventure".to_string(),
            starting_location: "The Crossroads Inn".to_string(),
        }
    }

    /// Set the ability score method.
    pub fn with_ability_method(mut self, method: AbilityMethod) -> Self {
        self.ability_method = method;
        self
    }

    /// Set the campaign name.
    pub fn with_campaign_name(mut self, name: impl Into<String>) -> Self {
        self.campaign_name = name.into();
        self
    }

    /// Set the starting location.
    pub fn with_starting_location(mut self, location: impl Into<String>) -> Self {
        self.starting_location = location.into();
        self
    }

    /// Build the character from this configuration.
    fn build_character(&self) -> Result<Character, SessionError> {
        let class_data = self.class.data();

        // Auto-select skills based on class
        let skills: Vec<_> = class_data
            .skill_options
            .iter()
            .take(class_data.skill_count)
            .copied()
            .collect();

        // Generate ability scores based on method
        let ability_scores = self.generate_ability_scores();

        CharacterBuilder::new()
            .name(&self.name)
            .race(self.race)
            .class(self.class)
            .background(self.background)
            .ability_scores(ability_scores)
            .skills(skills)
            .build()
            .map_err(|e| SessionError::Dm(crate::dm::DmError::ToolError(e.to_string())))
    }

    /// Generate ability scores based on the configured method.
    fn generate_ability_scores(&self) -> AbilityScores {
        match self.ability_method {
            AbilityMethod::StandardArray => {
                // Assign standard array based on class primary abilities
                let assignment = self.standard_array_assignment();
                let mut scores = AbilityScores::default();
                for (value, ability) in assignment {
                    scores.set(ability, value);
                }
                scores
            }
            AbilityMethod::PointBuy => {
                // Default point buy: balanced 14, 14, 14, 10, 10, 10 (uses 21 points)
                let mut scores = AbilityScores::default();
                let abilities = self.class_ability_priority();
                scores.set(abilities[0], 14);
                scores.set(abilities[1], 14);
                scores.set(abilities[2], 14);
                scores.set(abilities[3], 10);
                scores.set(abilities[4], 10);
                scores.set(abilities[5], 10);
                scores
            }
            AbilityMethod::Rolled => {
                // Roll and assign by class priority
                let rolled = roll_ability_scores();
                let abilities = self.class_ability_priority();
                let mut scores = AbilityScores::default();
                for (i, ability) in abilities.iter().enumerate() {
                    scores.set(*ability, rolled[i]);
                }
                scores
            }
        }
    }

    /// Get standard array assignment based on class.
    fn standard_array_assignment(&self) -> [(u8, Ability); 6] {
        let abilities = self.class_ability_priority();
        [
            (STANDARD_ARRAY[0], abilities[0]), // 15 to primary
            (STANDARD_ARRAY[1], abilities[1]), // 14 to secondary
            (STANDARD_ARRAY[2], abilities[2]), // 13 to tertiary
            (STANDARD_ARRAY[3], abilities[3]), // 12
            (STANDARD_ARRAY[4], abilities[4]), // 10
            (STANDARD_ARRAY[5], abilities[5]), // 8
        ]
    }

    /// Get ability priority order for the class.
    fn class_ability_priority(&self) -> [Ability; 6] {
        use Ability::*;
        match self.class {
            CharacterClass::Barbarian => [Strength, Constitution, Dexterity, Wisdom, Charisma, Intelligence],
            CharacterClass::Bard => [Charisma, Dexterity, Constitution, Wisdom, Intelligence, Strength],
            CharacterClass::Cleric => [Wisdom, Constitution, Strength, Charisma, Dexterity, Intelligence],
            CharacterClass::Druid => [Wisdom, Constitution, Dexterity, Intelligence, Charisma, Strength],
            CharacterClass::Fighter => [Strength, Constitution, Dexterity, Wisdom, Charisma, Intelligence],
            CharacterClass::Monk => [Dexterity, Wisdom, Constitution, Strength, Charisma, Intelligence],
            CharacterClass::Paladin => [Strength, Charisma, Constitution, Wisdom, Dexterity, Intelligence],
            CharacterClass::Ranger => [Dexterity, Wisdom, Constitution, Strength, Intelligence, Charisma],
            CharacterClass::Rogue => [Dexterity, Constitution, Charisma, Intelligence, Wisdom, Strength],
            CharacterClass::Sorcerer => [Charisma, Constitution, Dexterity, Wisdom, Intelligence, Strength],
            CharacterClass::Warlock => [Charisma, Constitution, Dexterity, Wisdom, Intelligence, Strength],
            CharacterClass::Wizard => [Intelligence, Constitution, Dexterity, Wisdom, Charisma, Strength],
        }
    }
}

/// A simplified response from the game.
#[derive(Debug, Clone)]
pub struct GameResponse {
    /// The narrative text from the DM.
    pub narrative: String,
    /// Whether the player is currently in combat.
    pub in_combat: bool,
    /// Whether it's the player's turn (in combat).
    pub is_player_turn: bool,
    /// Current HP.
    pub current_hp: i32,
    /// Maximum HP.
    pub max_hp: i32,
}

/// A headless D&D game that can be controlled programmatically.
///
/// This wraps `GameSession` with a simpler interface for automated use.
pub struct HeadlessGame {
    session: GameSession,
    /// Transcript of all exchanges.
    transcript: Vec<TranscriptEntry>,
}

/// An entry in the game transcript.
#[derive(Debug, Clone)]
pub struct TranscriptEntry {
    /// Player input.
    pub player_input: String,
    /// DM response.
    pub dm_response: String,
    /// Turn number.
    pub turn: usize,
}

impl HeadlessGame {
    /// Create a new headless game with the given configuration.
    ///
    /// Requires `ANTHROPIC_API_KEY` environment variable to be set.
    pub async fn new(config: HeadlessConfig) -> Result<Self, SessionError> {
        let character = config.build_character()?;

        let session_config = SessionConfig::new(&config.campaign_name)
            .with_starting_location(&config.starting_location);

        let session = GameSession::new_with_character(session_config, character).await?;

        Ok(Self {
            session,
            transcript: Vec::new(),
        })
    }

    /// Load a saved game from a file.
    pub async fn load(path: impl AsRef<Path>) -> Result<Self, SessionError> {
        let session = GameSession::load(path).await?;
        Ok(Self {
            session,
            transcript: Vec::new(),
        })
    }

    /// Send player input to the game and get a response.
    pub async fn send(&mut self, input: &str) -> Result<GameResponse, SessionError> {
        let response = self.session.player_action(input).await?;
        let (current_hp, max_hp) = self.session.hp_status();

        // Record in transcript
        self.transcript.push(TranscriptEntry {
            player_input: input.to_string(),
            dm_response: response.narrative.clone(),
            turn: self.transcript.len() + 1,
        });

        Ok(GameResponse {
            narrative: response.narrative,
            in_combat: response.in_combat,
            is_player_turn: response.is_player_turn,
            current_hp,
            max_hp,
        })
    }

    /// Save the current game to a file.
    pub async fn save(&self, path: impl AsRef<Path>) -> Result<(), SessionError> {
        self.session.save(path).await
    }

    // ========================================================================
    // Game State Queries
    // ========================================================================

    /// Get the player character's name.
    pub fn player_name(&self) -> &str {
        self.session.player_name()
    }

    /// Get the player character's class.
    pub fn player_class(&self) -> Option<&str> {
        self.session.player_class()
    }

    /// Get the player character's background.
    pub fn player_background(&self) -> &str {
        self.session.player_background()
    }

    /// Get current HP.
    pub fn current_hp(&self) -> i32 {
        self.session.hp_status().0
    }

    /// Get maximum HP.
    pub fn max_hp(&self) -> i32 {
        self.session.hp_status().1
    }

    /// Check if in combat.
    pub fn in_combat(&self) -> bool {
        self.session.in_combat()
    }

    /// Get the current location name.
    pub fn current_location(&self) -> &str {
        self.session.current_location()
    }

    /// Get the transcript of all exchanges.
    pub fn transcript(&self) -> &[TranscriptEntry] {
        &self.transcript
    }

    /// Get the current turn/round count.
    /// Returns combat round when in combat, otherwise transcript length.
    pub fn turn_count(&self) -> usize {
        if let Some(ref combat) = self.session.world().combat {
            combat.round as usize
        } else {
            self.transcript.len()
        }
    }

    /// Get active conditions on the player character.
    pub fn conditions(&self) -> Vec<String> {
        self.session
            .world()
            .player_character
            .conditions
            .iter()
            .map(|c| c.condition.to_string())
            .collect()
    }

    /// Get the last DM response, if any.
    pub fn last_response(&self) -> Option<&str> {
        self.transcript.last().map(|e| e.dm_response.as_str())
    }

    /// Check if the player has a specific condition.
    pub fn has_condition(&self, condition: Condition) -> bool {
        self.session
            .world()
            .player_character
            .conditions
            .iter()
            .any(|c| c.condition == condition)
    }

    /// Get the underlying session for advanced use.
    pub fn session(&self) -> &GameSession {
        &self.session
    }

    /// Get mutable access to the underlying session.
    pub fn session_mut(&mut self) -> &mut GameSession {
        &mut self.session
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quick_start_config() {
        let config = HeadlessConfig::quick_start("Thorin");
        assert_eq!(config.name, "Thorin");
        assert!(matches!(config.race, RaceType::Human));
        assert!(matches!(config.class, CharacterClass::Fighter));
        assert!(matches!(config.background, Background::FolkHero));
    }

    #[test]
    fn test_custom_config() {
        let config = HeadlessConfig::custom("Elara", RaceType::Elf, CharacterClass::Wizard, Background::Sage)
            .with_campaign_name("The Lost Library")
            .with_starting_location("The Academy");

        assert_eq!(config.name, "Elara");
        assert!(matches!(config.race, RaceType::Elf));
        assert!(matches!(config.class, CharacterClass::Wizard));
        assert!(matches!(config.background, Background::Sage));
        assert_eq!(config.campaign_name, "The Lost Library");
        assert_eq!(config.starting_location, "The Academy");
    }

    #[test]
    fn test_build_character() {
        let config = HeadlessConfig::quick_start("Test Hero");
        let character = config.build_character().unwrap();
        assert_eq!(character.name, "Test Hero");
    }
}
