//! D&D 5e conditions
//!
//! All 14 conditions from the PHB Appendix A, plus exhaustion levels.

use serde::{Deserialize, Serialize};
use std::fmt;

/// D&D 5e conditions (PHB Appendix A)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Condition {
    Blinded,
    Charmed,
    Deafened,
    Frightened,
    Grappled,
    Incapacitated,
    Invisible,
    Paralyzed,
    Petrified,
    Poisoned,
    Prone,
    Restrained,
    Stunned,
    Unconscious,
    /// Exhaustion has 6 levels
    Exhaustion(u8),
}

impl Condition {
    /// Get the condition name for display
    pub fn name(&self) -> &'static str {
        match self {
            Condition::Blinded => "Blinded",
            Condition::Charmed => "Charmed",
            Condition::Deafened => "Deafened",
            Condition::Frightened => "Frightened",
            Condition::Grappled => "Grappled",
            Condition::Incapacitated => "Incapacitated",
            Condition::Invisible => "Invisible",
            Condition::Paralyzed => "Paralyzed",
            Condition::Petrified => "Petrified",
            Condition::Poisoned => "Poisoned",
            Condition::Prone => "Prone",
            Condition::Restrained => "Restrained",
            Condition::Stunned => "Stunned",
            Condition::Unconscious => "Unconscious",
            Condition::Exhaustion(_) => "Exhaustion",
        }
    }

    /// Get the rules description
    pub fn description(&self) -> &'static str {
        match self {
            Condition::Blinded => {
                "A blinded creature can't see and automatically fails any ability check \
                 that requires sight. Attack rolls against the creature have advantage, \
                 and the creature's attack rolls have disadvantage."
            }
            Condition::Charmed => {
                "A charmed creature can't attack the charmer or target the charmer with \
                 harmful abilities or magical effects. The charmer has advantage on any \
                 ability check to interact socially with the creature."
            }
            Condition::Deafened => {
                "A deafened creature can't hear and automatically fails any ability check \
                 that requires hearing."
            }
            Condition::Frightened => {
                "A frightened creature has disadvantage on ability checks and attack rolls \
                 while the source of its fear is within line of sight. The creature can't \
                 willingly move closer to the source of its fear."
            }
            Condition::Grappled => {
                "A grappled creature's speed becomes 0, and it can't benefit from any bonus \
                 to its speed. The condition ends if the grappler is incapacitated or if an \
                 effect removes the grappled creature from the reach of the grappler."
            }
            Condition::Incapacitated => {
                "An incapacitated creature can't take actions or reactions."
            }
            Condition::Invisible => {
                "An invisible creature is impossible to see without the aid of magic or a \
                 special sense. The creature is heavily obscured. Attack rolls against the \
                 creature have disadvantage, and the creature's attack rolls have advantage."
            }
            Condition::Paralyzed => {
                "A paralyzed creature is incapacitated and can't move or speak. The creature \
                 automatically fails Strength and Dexterity saving throws. Attack rolls \
                 against the creature have advantage. Any attack that hits is a critical hit \
                 if the attacker is within 5 feet."
            }
            Condition::Petrified => {
                "A petrified creature is transformed into a solid inanimate substance. It is \
                 incapacitated, can't move or speak, and is unaware of its surroundings. \
                 Attack rolls against it have advantage. It automatically fails Strength and \
                 Dexterity saving throws. It has resistance to all damage and is immune to \
                 poison and disease."
            }
            Condition::Poisoned => {
                "A poisoned creature has disadvantage on attack rolls and ability checks."
            }
            Condition::Prone => {
                "A prone creature's only movement option is to crawl. The creature has \
                 disadvantage on attack rolls. An attack roll against the creature has \
                 advantage if the attacker is within 5 feet, otherwise disadvantage."
            }
            Condition::Restrained => {
                "A restrained creature's speed becomes 0. Attack rolls against the creature \
                 have advantage. The creature's attack rolls have disadvantage. The creature \
                 has disadvantage on Dexterity saving throws."
            }
            Condition::Stunned => {
                "A stunned creature is incapacitated, can't move, and can speak only \
                 falteringly. The creature automatically fails Strength and Dexterity \
                 saving throws. Attack rolls against the creature have advantage."
            }
            Condition::Unconscious => {
                "An unconscious creature is incapacitated, can't move or speak, and is \
                 unaware of its surroundings. The creature drops whatever it's holding \
                 and falls prone. Attack rolls against the creature have advantage. Any \
                 attack that hits is a critical hit if the attacker is within 5 feet."
            }
            Condition::Exhaustion(level) => match level {
                1 => "Disadvantage on ability checks",
                2 => "Speed halved",
                3 => "Disadvantage on attack rolls and saving throws",
                4 => "Hit point maximum halved",
                5 => "Speed reduced to 0",
                6 => "Death",
                _ => "Invalid exhaustion level",
            },
        }
    }

    /// Check if this condition causes incapacitation
    pub fn is_incapacitating(&self) -> bool {
        matches!(
            self,
            Condition::Incapacitated
                | Condition::Paralyzed
                | Condition::Petrified
                | Condition::Stunned
                | Condition::Unconscious
        )
    }

    /// Check if this condition prevents movement
    pub fn prevents_movement(&self) -> bool {
        matches!(
            self,
            Condition::Grappled
                | Condition::Paralyzed
                | Condition::Petrified
                | Condition::Restrained
                | Condition::Stunned
                | Condition::Unconscious
                | Condition::Exhaustion(5..)
        )
    }

    /// Get attack roll advantage/disadvantage granted by this condition
    pub fn attack_advantage(&self) -> Option<bool> {
        match self {
            Condition::Blinded | Condition::Poisoned | Condition::Restrained => Some(false), // disadvantage
            Condition::Invisible => Some(true), // advantage
            Condition::Exhaustion(3..) => Some(false), // disadvantage
            _ => None,
        }
    }

    /// Check if attacks against this creature have advantage
    pub fn grants_advantage_to_attackers(&self) -> bool {
        matches!(
            self,
            Condition::Blinded
                | Condition::Paralyzed
                | Condition::Petrified
                | Condition::Restrained
                | Condition::Stunned
                | Condition::Unconscious
        )
    }
}

impl fmt::Display for Condition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Condition::Exhaustion(level) => write!(f, "Exhaustion ({})", level),
            _ => write!(f, "{}", self.name()),
        }
    }
}

/// A condition applied to a creature with tracking info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveCondition {
    pub condition: Condition,
    pub source: String,
    pub duration: ConditionDuration,
    /// For conditions that allow saves to end
    pub save_info: Option<ConditionSave>,
}

impl ActiveCondition {
    pub fn new(condition: Condition, source: impl Into<String>) -> Self {
        Self {
            condition,
            source: source.into(),
            duration: ConditionDuration::Indefinite,
            save_info: None,
        }
    }

    pub fn with_duration(mut self, duration: ConditionDuration) -> Self {
        self.duration = duration;
        self
    }

    pub fn with_save(mut self, save_info: ConditionSave) -> Self {
        self.save_info = Some(save_info);
        self
    }
}

/// Duration types for conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionDuration {
    /// Lasts until removed by specific means
    Indefinite,
    /// Ends at the start/end of a creature's turn
    UntilTurn { creature_id: String, start_or_end: TurnTiming },
    /// Ends after a number of rounds
    Rounds(u32),
    /// Ends after a number of minutes
    Minutes(u32),
    /// Ends after a number of hours
    Hours(u32),
    /// Concentration-based (ends when concentration breaks)
    Concentration { caster_id: String },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TurnTiming {
    Start,
    End,
}

/// Information for conditions that can be saved against
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionSave {
    pub ability: super::Ability,
    pub dc: u8,
    pub timing: SaveTiming,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SaveTiming {
    /// Save at end of each turn
    EndOfTurn,
    /// Save when taking damage
    WhenDamaged,
    /// Save at start of turn
    StartOfTurn,
}
