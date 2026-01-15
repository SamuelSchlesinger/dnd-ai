//! Terminal User Interface for D&D game
//!
//! Built with ratatui, providing:
//! - Narrative display with streaming support
//! - Character sheet panels
//! - Combat tracker
//! - Animated dice rolls
//! - Modal overlays

pub mod theme;
pub mod layout;
pub mod widgets;
pub mod render;

pub use theme::GameTheme;
pub use render::render;
