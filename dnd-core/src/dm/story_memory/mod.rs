//! Story Memory System for narrative consistency.
//!
//! This module implements a "gardening style" memory system that extracts and indexes
//! facts from the narrative, enabling consistent storytelling across context window
//! resets and sessions.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                      StoryMemory                                │
//! │                                                                 │
//! │  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐  │
//! │  │ EntityIndex  │  │ FactStore    │  │ RelationshipGraph    │  │
//! │  │ (name→id)    │  │ (id→facts)   │  │ (entity→entity)      │  │
//! │  └──────────────┘  └──────────────┘  └──────────────────────┘  │
//! │                                                                 │
//! │  ┌──────────────────────────────────────────────────────────┐  │
//! │  │ ConsequenceStore (trigger → effect, with expiry)         │  │
//! │  └──────────────────────────────────────────────────────────┘  │
//! └─────────────────────────────────────────────────────────────────┘
//! ```

mod consequence;
mod entity;
mod fact;
mod relationship;
mod store;

pub use consequence::{Consequence, ConsequenceId, ConsequenceSeverity, ConsequenceStatus};
pub use entity::{Entity, EntityId, EntityType, StoryMoment};
pub use fact::{FactCategory, FactId, FactSource, StoryFact};
pub use relationship::{Relationship, RelationshipType};
pub use store::StoryMemory;
