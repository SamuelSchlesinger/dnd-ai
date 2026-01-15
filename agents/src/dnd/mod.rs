//! D&D Dungeon Master Agent with TUI
//!
//! A comprehensive single-player D&D 5e experience powered by AI.
//!
//! # Architecture
//!
//! The DM agent uses a recursive subagent architecture for high throughput:
//! - MasterDM orchestrates specialized subagents
//! - Each subagent has focused context for its domain
//! - Tool-based delegation keeps context efficient
//!
//! # Modules
//!
//! - `game` - Core D&D game mechanics and state
//! - `ui` - Terminal user interface with ratatui
//! - `ai` - DM agent and subagent implementations

pub mod game;
pub mod ui;
pub mod ai;
pub mod app;
pub mod events;

pub use app::AppState;
