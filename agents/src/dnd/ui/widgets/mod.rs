//! TUI widgets for the D&D game

pub mod narrative;
pub mod character_panel;
pub mod combat_tracker;
pub mod dice_roll;
pub mod input;
pub mod status_bar;

pub use narrative::NarrativeWidget;
pub use character_panel::CharacterPanelWidget;
pub use combat_tracker::CombatTrackerWidget;
pub use dice_roll::DiceRollWidget;
pub use input::InputWidget;
pub use status_bar::StatusBarWidget;
