//! AI Dungeon Master implementation
//!
//! Uses the agentic framework to provide:
//! - Main DM agent with tool execution
//! - Specialized subagents (combat, NPC, world, etc.)
//! - System prompts for D&D context

pub mod dm_agent;
pub mod tools;
pub mod prompts;
pub mod subagents;

pub use dm_agent::DungeonMasterAgent;
