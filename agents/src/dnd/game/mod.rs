//! D&D 5e game mechanics and state management
//!
//! This module contains all the core D&D rules implementations:
//! - Dice rolling with standard notation
//! - Character sheets with full 5e stats
//! - Combat tracking with initiative
//! - World state management

pub mod dice;
pub mod character;
pub mod combat;
pub mod state;
pub mod conditions;
pub mod skills;

pub use dice::{DiceRoll, DiceExpression, RollResult, Advantage};
pub use character::{Character, AbilityScores, Ability, HitPoints};
pub use combat::{CombatState, InitiativeEntry, CombatantStatus};
pub use state::{GameWorld, GameMode, Location};
pub use conditions::Condition;
pub use skills::Skill;
