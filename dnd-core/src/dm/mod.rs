//! AI Dungeon Master module.
//!
//! Contains the DM agent, tools, and memory management for
//! running AI-powered D&D sessions.

mod agent;
pub mod memory;
pub mod story_memory;
mod tools;

pub use agent::{DmConfig, DmError, DmResponse, DungeonMaster};
pub use memory::{CampaignFact, DmMemory, FactCategory};
pub use story_memory::{
    Entity, EntityId, EntityType, FactCategory as StoryFactCategory, FactSource, Relationship,
    RelationshipType, StoryFact, StoryMemory, StoryMoment,
};
pub use tools::DmTools;
