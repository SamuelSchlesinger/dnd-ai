//! D&D 5e skills
//!
//! All 18 skills with their associated abilities.

use super::Ability;
use serde::{Deserialize, Serialize};
use std::fmt;

/// D&D 5e skills
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Skill {
    // Strength
    Athletics,
    // Dexterity
    Acrobatics,
    SleightOfHand,
    Stealth,
    // Intelligence
    Arcana,
    History,
    Investigation,
    Nature,
    Religion,
    // Wisdom
    AnimalHandling,
    Insight,
    Medicine,
    Perception,
    Survival,
    // Charisma
    Deception,
    Intimidation,
    Performance,
    Persuasion,
}

impl Skill {
    /// Get the ability score associated with this skill
    pub fn ability(&self) -> Ability {
        match self {
            Skill::Athletics => Ability::Strength,
            Skill::Acrobatics | Skill::SleightOfHand | Skill::Stealth => Ability::Dexterity,
            Skill::Arcana
            | Skill::History
            | Skill::Investigation
            | Skill::Nature
            | Skill::Religion => Ability::Intelligence,
            Skill::AnimalHandling
            | Skill::Insight
            | Skill::Medicine
            | Skill::Perception
            | Skill::Survival => Ability::Wisdom,
            Skill::Deception | Skill::Intimidation | Skill::Performance | Skill::Persuasion => {
                Ability::Charisma
            }
        }
    }

    /// Get the skill name for display
    pub fn name(&self) -> &'static str {
        match self {
            Skill::Athletics => "Athletics",
            Skill::Acrobatics => "Acrobatics",
            Skill::SleightOfHand => "Sleight of Hand",
            Skill::Stealth => "Stealth",
            Skill::Arcana => "Arcana",
            Skill::History => "History",
            Skill::Investigation => "Investigation",
            Skill::Nature => "Nature",
            Skill::Religion => "Religion",
            Skill::AnimalHandling => "Animal Handling",
            Skill::Insight => "Insight",
            Skill::Medicine => "Medicine",
            Skill::Perception => "Perception",
            Skill::Survival => "Survival",
            Skill::Deception => "Deception",
            Skill::Intimidation => "Intimidation",
            Skill::Performance => "Performance",
            Skill::Persuasion => "Persuasion",
        }
    }

    /// Get all skills for a given ability
    pub fn for_ability(ability: Ability) -> Vec<Skill> {
        Self::all()
            .into_iter()
            .filter(|s| s.ability() == ability)
            .collect()
    }

    /// Get all skills
    pub fn all() -> Vec<Skill> {
        vec![
            Skill::Athletics,
            Skill::Acrobatics,
            Skill::SleightOfHand,
            Skill::Stealth,
            Skill::Arcana,
            Skill::History,
            Skill::Investigation,
            Skill::Nature,
            Skill::Religion,
            Skill::AnimalHandling,
            Skill::Insight,
            Skill::Medicine,
            Skill::Perception,
            Skill::Survival,
            Skill::Deception,
            Skill::Intimidation,
            Skill::Performance,
            Skill::Persuasion,
        ]
    }
}

impl fmt::Display for Skill {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Proficiency level for skills/tools
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ProficiencyLevel {
    #[default]
    None,
    /// Half proficiency (Bard's Jack of All Trades)
    Half,
    /// Normal proficiency
    Proficient,
    /// Expertise (double proficiency)
    Expertise,
}

impl ProficiencyLevel {
    /// Get the proficiency multiplier
    pub fn multiplier(&self) -> f32 {
        match self {
            ProficiencyLevel::None => 0.0,
            ProficiencyLevel::Half => 0.5,
            ProficiencyLevel::Proficient => 1.0,
            ProficiencyLevel::Expertise => 2.0,
        }
    }

    /// Calculate bonus given proficiency bonus
    pub fn bonus(&self, proficiency_bonus: i8) -> i8 {
        match self {
            ProficiencyLevel::None => 0,
            ProficiencyLevel::Half => proficiency_bonus / 2,
            ProficiencyLevel::Proficient => proficiency_bonus,
            ProficiencyLevel::Expertise => proficiency_bonus * 2,
        }
    }
}
