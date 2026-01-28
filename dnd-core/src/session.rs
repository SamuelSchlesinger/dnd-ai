//! GameSession - the primary public API for D&D gameplay.
//!
//! This module provides a clean, high-level interface for all D&D
//! game interactions. It wraps the DungeonMaster, GameWorld, and
//! persistence logic into a single, easy-to-use API.

use crate::dm::{DmConfig, DmError, DmResponse, DungeonMaster};
use crate::rules::Effect;
use crate::world::{create_sample_fighter, GameWorld};
use std::path::Path;
use thiserror::Error;
use tokio::fs;

/// Errors from GameSession operations.
#[derive(Debug, Error)]
pub enum SessionError {
    #[error("DM error: {0}")]
    Dm(#[from] DmError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("No API key configured - set ANTHROPIC_API_KEY environment variable")]
    NoApiKey,
}

/// Configuration for creating a new game session.
#[derive(Debug, Clone)]
pub struct SessionConfig {
    /// Campaign name.
    pub campaign_name: String,

    /// Player character name.
    pub character_name: String,

    /// Starting location name.
    pub starting_location: String,

    /// Custom DM system prompt.
    pub custom_dm_prompt: Option<String>,

    /// Model to use for the DM.
    pub model: Option<String>,

    /// Maximum tokens for DM responses.
    pub max_tokens: usize,

    /// Temperature for DM generation.
    pub temperature: Option<f32>,
}

impl SessionConfig {
    /// Create a new session config with campaign name.
    pub fn new(campaign_name: impl Into<String>) -> Self {
        Self {
            campaign_name: campaign_name.into(),
            character_name: "Adventurer".to_string(),
            starting_location: "The Rusty Dragon Inn".to_string(),
            custom_dm_prompt: None,
            model: None,
            max_tokens: 4096,
            temperature: Some(0.8),
        }
    }

    /// Set the player character name.
    pub fn with_character_name(mut self, name: impl Into<String>) -> Self {
        self.character_name = name.into();
        self
    }

    /// Set the starting location.
    pub fn with_starting_location(mut self, location: impl Into<String>) -> Self {
        self.starting_location = location.into();
        self
    }

    /// Set a custom DM system prompt.
    pub fn with_dm_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.custom_dm_prompt = Some(prompt.into());
        self
    }

    /// Set the model to use.
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Set max tokens for responses.
    pub fn with_max_tokens(mut self, tokens: usize) -> Self {
        self.max_tokens = tokens;
        self
    }

    /// Set temperature for generation.
    pub fn with_temperature(mut self, temp: f32) -> Self {
        self.temperature = Some(temp);
        self
    }
}

/// Response from a player action.
#[derive(Debug, Clone)]
pub struct Response {
    /// The narrative text from the DM.
    pub narrative: String,

    /// Effects that were applied to the game world.
    pub effects: Vec<Effect>,

    /// Whether combat is currently active.
    pub in_combat: bool,

    /// The current player's turn in combat (if applicable).
    pub is_player_turn: bool,
}

impl From<DmResponse> for Response {
    fn from(dm: DmResponse) -> Self {
        Self {
            narrative: dm.narrative,
            effects: dm.effects,
            in_combat: false, // Will be set by GameSession
            is_player_turn: false,
        }
    }
}

/// A D&D game session.
///
/// This is the main entry point for all D&D gameplay. It manages:
/// - The game world (characters, locations, combat state)
/// - The AI Dungeon Master
/// - Session persistence
pub struct GameSession {
    dm: DungeonMaster,
    world: GameWorld,
}

impl GameSession {
    /// Create a new game session with the given configuration.
    ///
    /// Requires `ANTHROPIC_API_KEY` environment variable to be set.
    pub async fn new(config: SessionConfig) -> Result<Self, SessionError> {
        let dm_config = DmConfig {
            model: config.model,
            max_tokens: config.max_tokens,
            temperature: config.temperature,
            custom_system_prompt: config.custom_dm_prompt,
        };

        let dm = DungeonMaster::from_env()
            .map_err(|_| SessionError::NoApiKey)?
            .with_config(dm_config);

        // Create a sample character
        let character = create_sample_fighter(&config.character_name);

        let mut world = GameWorld::new(config.campaign_name, character);

        // Update starting location name if custom
        world.current_location.name = config.starting_location;

        Ok(Self { dm, world })
    }

    /// Create a new game session with a custom character.
    ///
    /// Requires `ANTHROPIC_API_KEY` environment variable to be set.
    pub async fn new_with_character(
        config: SessionConfig,
        character: crate::world::Character,
    ) -> Result<Self, SessionError> {
        let dm_config = DmConfig {
            model: config.model,
            max_tokens: config.max_tokens,
            temperature: config.temperature,
            custom_system_prompt: config.custom_dm_prompt,
        };

        let dm = DungeonMaster::from_env()
            .map_err(|_| SessionError::NoApiKey)?
            .with_config(dm_config);

        let mut world = GameWorld::new(config.campaign_name, character);

        // Update starting location name if custom
        world.current_location.name = config.starting_location;

        Ok(Self { dm, world })
    }

    /// Create a session with a pre-configured world.
    ///
    /// This allows for custom character creation and world setup.
    pub fn with_world(dm: DungeonMaster, world: GameWorld) -> Self {
        Self { dm, world }
    }

    /// Load a saved session from a file.
    pub async fn load(path: impl AsRef<Path>) -> Result<Self, SessionError> {
        let content = fs::read_to_string(path).await?;
        let saved: SavedSession = serde_json::from_str(&content)?;

        let dm = DungeonMaster::from_env().map_err(|_| SessionError::NoApiKey)?;

        // Restore memory from saved session
        let mut session = Self {
            dm,
            world: saved.world,
        };

        // Restore memory context
        if let Some(summary) = saved.conversation_summary {
            session.dm.memory_mut().set_summary(summary);
        }

        for fact in saved.campaign_facts {
            session
                .dm
                .memory_mut()
                .add_fact(fact.category, &fact.content);
        }

        // Restore story memory if present
        if let Some(story_memory) = saved.story_memory {
            *session.dm.story_memory_mut() = story_memory;
        }

        Ok(session)
    }

    /// Save the current session to a file.
    pub async fn save(&self, path: impl AsRef<Path>) -> Result<(), SessionError> {
        let saved = SavedSession {
            world: self.world.clone(),
            campaign_facts: self
                .dm
                .memory()
                .campaign_facts.to_vec(),
            conversation_summary: Some(self.dm.memory().generate_summary()),
            story_memory: Some(self.dm.story_memory().clone()),
        };

        let content = serde_json::to_string_pretty(&saved)?;
        fs::write(path, content).await?;
        Ok(())
    }

    /// Process a player action and get the DM's response.
    ///
    /// This is the main gameplay loop entry point.
    pub async fn player_action(&mut self, input: &str) -> Result<Response, SessionError> {
        let dm_response = self.dm.process_input(input, &mut self.world).await?;

        let in_combat = self.world.combat.is_some();
        let is_player_turn = self
            .world
            .combat
            .as_ref()
            .and_then(|c| c.current_combatant())
            .map(|c| c.is_player)
            .unwrap_or(false);

        Ok(Response {
            narrative: dm_response.narrative,
            effects: dm_response.effects,
            in_combat,
            is_player_turn,
        })
    }

    /// Get a reference to the game world.
    pub fn world(&self) -> &GameWorld {
        &self.world
    }

    /// Get a mutable reference to the game world.
    ///
    /// Use with caution - direct modifications bypass the rules engine.
    pub fn world_mut(&mut self) -> &mut GameWorld {
        &mut self.world
    }

    /// Get a reference to the DM.
    pub fn dm(&self) -> &DungeonMaster {
        &self.dm
    }

    /// Get a mutable reference to the DM.
    pub fn dm_mut(&mut self) -> &mut DungeonMaster {
        &mut self.dm
    }

    /// Get the player character's name.
    pub fn player_name(&self) -> &str {
        &self.world.player_character.name
    }

    /// Get the current location name.
    pub fn current_location(&self) -> &str {
        &self.world.current_location.name
    }

    /// Get the player character's class name.
    pub fn player_class(&self) -> Option<&str> {
        self.world
            .player_character
            .classes
            .first()
            .map(|c| c.class.name())
    }

    /// Get the player character's background name.
    pub fn player_background(&self) -> &str {
        self.world.player_character.background.name()
    }

    /// Check if the session is in combat.
    pub fn in_combat(&self) -> bool {
        self.world.combat.is_some()
    }

    /// Get the current HP status.
    ///
    /// Returns (current, max) where current is clamped to 0 minimum
    /// (actual internal value may be negative for massive damage tracking).
    pub fn hp_status(&self) -> (i32, i32) {
        let hp = &self.world.player_character.hit_points;
        (hp.current.max(0), hp.maximum)
    }
}

/// Serializable session state for persistence.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct SavedSession {
    world: GameWorld,
    campaign_facts: Vec<crate::dm::memory::CampaignFact>,
    conversation_summary: Option<String>,
    #[serde(default)]
    story_memory: Option<crate::dm::StoryMemory>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_config() {
        let config = SessionConfig::new("Test Campaign")
            .with_character_name("Thorin")
            .with_starting_location("Mountain Hall")
            .with_max_tokens(2048);

        assert_eq!(config.campaign_name, "Test Campaign");
        assert_eq!(config.character_name, "Thorin");
        assert_eq!(config.starting_location, "Mountain Hall");
        assert_eq!(config.max_tokens, 2048);
    }

    #[test]
    fn test_response_from_dm() {
        let dm_response = DmResponse {
            narrative: "You see a dragon!".to_string(),
            intents: vec![],
            effects: vec![],
            resolutions: vec![],
        };

        let response: Response = dm_response.into();
        assert_eq!(response.narrative, "You see a dragon!");
        assert!(!response.in_combat);
    }
}
