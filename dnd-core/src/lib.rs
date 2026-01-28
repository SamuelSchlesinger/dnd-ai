//! D&D 5e game engine with AI Dungeon Master.
//!
//! This crate provides:
//! - Complete D&D 5e game mechanics
//! - AI-powered Dungeon Master using Claude
//! - Intent/Effect rules system for deterministic game state
//! - Campaign persistence
//!
//! # Quick Start
//!
//! ```ignore
//! use dnd_core::{GameSession, SessionConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = SessionConfig::new("My Campaign")
//!         .with_character_name("Thorin");
//!
//!     let mut session = GameSession::new(config).await?;
//!
//!     let response = session.player_action("I look around the tavern").await?;
//!     println!("{}", response.narrative);
//!
//!     session.save("my_campaign.json").await?;
//!     Ok(())
//! }
//! ```

pub mod character_builder;
pub mod class_data;
pub mod dice;
pub mod dm;
pub mod headless;
pub mod persist;
pub mod rules;
pub mod session;
pub mod testing;
pub mod world;

// Re-export for convenience
pub use dnd_macros::Tool;

// Primary public API
pub use character_builder::{AbilityMethod, CharacterBuilder};
pub use headless::{HeadlessConfig, HeadlessGame};
pub use session::{GameSession, Response, SessionConfig, SessionError};
pub use testing::{MockDm, MockResponse, TestHarness};
pub use world::{Background, CharacterClass, RaceType};

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    /// Roll dice using standard D&D notation
    #[derive(Tool, Deserialize)]
    #[tool(name = "roll_dice")]
    struct RollDice {
        /// Dice notation like "2d6+3" or "1d20"
        notation: String,
        /// Optional purpose for the roll
        purpose: Option<String>,
    }

    #[test]
    fn test_tool_derive() {
        assert_eq!(RollDice::tool_name(), "roll_dice");
        assert_eq!(
            RollDice::tool_description(),
            "Roll dice using standard D&D notation"
        );
    }

    #[test]
    fn test_tool_schema() {
        let schema = RollDice::input_schema();
        assert_eq!(schema["type"], "object");
        assert_eq!(schema["properties"]["notation"]["type"], "string");
        assert_eq!(schema["properties"]["purpose"]["type"], "string");

        // notation should be required, purpose should not be (it's Option)
        let required = schema["required"].as_array().unwrap();
        assert!(required.iter().any(|v| v == "notation"));
        assert!(!required.iter().any(|v| v == "purpose"));
    }

    #[test]
    fn test_tool_as_tool() {
        let tool = RollDice::as_tool();
        assert_eq!(tool.name, "roll_dice");
        assert!(!tool.description.is_empty());
    }
}
