//! TUI widgets for the D&D game

pub mod character_panel;
pub mod combat_tracker;
pub mod dice_roll;
pub mod input;
pub mod narrative;
pub mod status_bar;

pub use character_panel::CharacterPanelWidget;
pub use combat_tracker::{CombatTrackerWidget, EnemyHpWidget, EnemyStatus, HpEstimate};
pub use dice_roll::{DiceAnimationState, DiceRollWidget};
pub use input::InputWidget;
pub use narrative::NarrativeWidget;
pub use status_bar::{HotkeyBarWidget, StatusBarWidget};
