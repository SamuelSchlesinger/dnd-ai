//! D&D 5e game engine with AI Dungeon Master.
//!
//! This crate provides:
//! - Complete D&D 5e game mechanics
//! - AI-powered Dungeon Master using Anthropic, OpenAI, OpenRouter, or local models
//! - Intent/Effect rules system for deterministic game state
//! - Campaign persistence
//!
//! # Quick Start
//!
//! ```ignore
//! use chronicler_core::{GameSession, SessionConfig};
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
//!
//! # Explicit provider injection
//!
//! [`GameSession::new`] and [`GameSession::load`] resolve provider configuration
//! from the environment. Hosts can instead inject any supported local or cloud
//! client. Provider configuration is not serialized into campaign saves.
//!
//! ```ignore
//! use chronicler_core::{
//!     GameSession, LlmClient, SessionConfig, LOCAL_DEFAULT_MODEL,
//! };
//!
//! let client = LlmClient::local(LOCAL_DEFAULT_MODEL)?;
//! let session = GameSession::new_with_client(
//!     SessionConfig::new("Local Campaign"),
//!     client,
//! ).await?;
//! ```

pub mod character_builder;
pub mod class_data;
pub mod dice;
pub mod dm;
pub mod headless;
pub mod items;
pub mod persist;
pub mod rules;
pub mod session;
pub mod spells;
pub mod testing;
pub mod world;

// Primary public API
pub use character_builder::{AbilityMethod, CharacterBuilder};
pub use chronicler_llm::{
    Client as LlmClient, ClientConfig as LlmConfig, ProviderKind as LlmProvider,
    LOCAL_DEFAULT_MODEL, OLLAMA_BASE_URL, OPENROUTER_DEFAULT_MODEL,
};
pub use headless::{HeadlessConfig, HeadlessGame};
pub use persist::{CharacterMetadata, CharacterSaveInfo, SavedCharacter};
pub use session::{GameSession, Response, SessionConfig, SessionError};
pub use testing::{MockDm, MockResponse, TestHarness};
pub use world::{Background, CharacterClass, RaceType};
