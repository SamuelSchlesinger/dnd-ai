//! D&D 5e character classes and class-related features.
//!
//! This module contains character class definitions, class features,
//! spellcasting progression, and class-specific resource tracking.

use crate::dice::DieType;
use serde::{Deserialize, Serialize};
use std::fmt;

use super::Ability;

// ============================================================================
// Classes and Features
// ============================================================================

/// D&D character classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CharacterClass {
    Barbarian,
    Bard,
    Cleric,
    Druid,
    Fighter,
    Monk,
    Paladin,
    Ranger,
    Rogue,
    Sorcerer,
    Warlock,
    Wizard,
}

impl CharacterClass {
    pub fn hit_die(&self) -> DieType {
        match self {
            CharacterClass::Barbarian => DieType::D12,
            CharacterClass::Fighter | CharacterClass::Paladin | CharacterClass::Ranger => {
                DieType::D10
            }
            CharacterClass::Bard
            | CharacterClass::Cleric
            | CharacterClass::Druid
            | CharacterClass::Monk
            | CharacterClass::Rogue
            | CharacterClass::Warlock => DieType::D8,
            CharacterClass::Sorcerer | CharacterClass::Wizard => DieType::D6,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            CharacterClass::Barbarian => "Barbarian",
            CharacterClass::Bard => "Bard",
            CharacterClass::Cleric => "Cleric",
            CharacterClass::Druid => "Druid",
            CharacterClass::Fighter => "Fighter",
            CharacterClass::Monk => "Monk",
            CharacterClass::Paladin => "Paladin",
            CharacterClass::Ranger => "Ranger",
            CharacterClass::Rogue => "Rogue",
            CharacterClass::Sorcerer => "Sorcerer",
            CharacterClass::Warlock => "Warlock",
            CharacterClass::Wizard => "Wizard",
        }
    }

    /// Returns true if this class has spellcasting at level 1.
    pub fn is_spellcaster(&self) -> bool {
        matches!(
            self,
            CharacterClass::Bard
                | CharacterClass::Cleric
                | CharacterClass::Druid
                | CharacterClass::Sorcerer
                | CharacterClass::Warlock
                | CharacterClass::Wizard
        )
        // Note: Paladin and Ranger get spellcasting at level 2, not level 1
    }

    /// Returns the number of attacks per Attack action at the given level.
    /// Most martial classes get Extra Attack at level 5.
    /// Fighters get additional attacks at levels 11 and 20.
    pub fn attacks_per_action(&self, level: u8) -> u8 {
        match self {
            CharacterClass::Fighter => match level {
                1..=4 => 1,
                5..=10 => 2,
                11..=19 => 3,
                20.. => 4,
                _ => 1,
            },
            CharacterClass::Barbarian
            | CharacterClass::Monk
            | CharacterClass::Paladin
            | CharacterClass::Ranger
                if level >= 5 =>
            {
                2
            }
            // Casters and Rogues don't get Extra Attack (Rogues get Sneak Attack instead)
            _ => 1,
        }
    }

    /// Returns the spellcasting ability for this class, if any.
    pub fn spellcasting_ability(&self) -> Option<Ability> {
        match self {
            CharacterClass::Bard | CharacterClass::Sorcerer | CharacterClass::Warlock => {
                Some(Ability::Charisma)
            }
            CharacterClass::Cleric | CharacterClass::Druid | CharacterClass::Ranger => {
                Some(Ability::Wisdom)
            }
            CharacterClass::Wizard => Some(Ability::Intelligence),
            CharacterClass::Paladin => Some(Ability::Charisma),
            _ => None,
        }
    }

    /// Returns the number of cantrips known at level 1.
    pub fn cantrips_known_at_level_1(&self) -> usize {
        match self {
            CharacterClass::Bard => 2,
            CharacterClass::Cleric => 3,
            CharacterClass::Druid => 2,
            CharacterClass::Sorcerer => 4,
            CharacterClass::Warlock => 2,
            CharacterClass::Wizard => 3,
            _ => 0,
        }
    }

    /// Returns the number of spells known at level 1 (for classes that learn specific spells).
    pub fn spells_known_at_level_1(&self) -> usize {
        match self {
            CharacterClass::Bard => 4,
            CharacterClass::Sorcerer => 2,
            CharacterClass::Warlock => 2,
            CharacterClass::Wizard => 6, // Spellbook spells
            _ => 0,                      // Clerics and Druids prepare from entire list
        }
    }

    /// Returns the total number of cantrips known at a given level.
    pub fn cantrips_known_at_level(&self, level: u8) -> usize {
        match self {
            CharacterClass::Bard => match level {
                1..=3 => 2,
                4..=9 => 3,
                10..=13 => 4,
                14..=20 => 5,
                _ => 2,
            },
            CharacterClass::Cleric => match level {
                1..=3 => 3,
                4..=9 => 4,
                10..=20 => 5,
                _ => 3,
            },
            CharacterClass::Druid => match level {
                1..=3 => 2,
                4..=9 => 3,
                10..=20 => 4,
                _ => 2,
            },
            CharacterClass::Sorcerer => match level {
                1..=3 => 4,
                4..=9 => 5,
                10..=20 => 6,
                _ => 4,
            },
            CharacterClass::Warlock => match level {
                1..=3 => 2,
                4..=9 => 3,
                10..=20 => 4,
                _ => 2,
            },
            CharacterClass::Wizard => match level {
                1..=3 => 3,
                4..=9 => 4,
                10..=20 => 5,
                _ => 3,
            },
            _ => 0,
        }
    }

    /// Returns the total number of spells known at a given level.
    /// For "known" casters (Bard, Sorcerer, Warlock, Ranger).
    /// Returns None for prepared casters (Cleric, Druid, Paladin) and Wizard (spellbook).
    pub fn spells_known_at_level(&self, level: u8) -> Option<usize> {
        match self {
            // Bard: spells known progression
            CharacterClass::Bard => Some(match level {
                1 => 4,
                2 => 5,
                3 => 6,
                4 => 7,
                5 => 8,
                6 => 9,
                7 => 10,
                8 => 11,
                9 => 12,
                10 => 14,
                11 => 15,
                12 => 15,
                13 => 16,
                14 => 18,
                15 => 19,
                16 => 19,
                17 => 20,
                18 => 22,
                19 => 22,
                20 => 22,
                _ => 4,
            }),
            // Sorcerer: spells known progression
            CharacterClass::Sorcerer => Some(match level {
                1 => 2,
                2 => 3,
                3 => 4,
                4 => 5,
                5 => 6,
                6 => 7,
                7 => 8,
                8 => 9,
                9 => 10,
                10 => 11,
                11 => 12,
                12 => 12,
                13 => 13,
                14 => 13,
                15 => 14,
                16 => 14,
                17 => 15,
                18 => 15,
                19 => 15,
                20 => 15,
                _ => 2,
            }),
            // Warlock: spells known progression
            CharacterClass::Warlock => Some(match level {
                1 => 2,
                2 => 3,
                3 => 4,
                4 => 5,
                5 => 6,
                6 => 7,
                7 => 8,
                8 => 9,
                9 => 10,
                10 => 10,
                11 => 11,
                12 => 11,
                13 => 12,
                14 => 12,
                15 => 13,
                16 => 13,
                17 => 14,
                18 => 14,
                19 => 15,
                20 => 15,
                _ => 2,
            }),
            // Ranger: spells known progression (starts at level 2)
            CharacterClass::Ranger => Some(match level {
                1 => 0,
                2 => 2,
                3 => 3,
                4 => 3,
                5 => 4,
                6 => 4,
                7 => 5,
                8 => 5,
                9 => 6,
                10 => 6,
                11 => 7,
                12 => 7,
                13 => 8,
                14 => 8,
                15 => 9,
                16 => 9,
                17 => 10,
                18 => 10,
                19 => 11,
                20 => 11,
                _ => 0,
            }),
            // Wizard uses spellbook, not spells known
            // Cleric, Druid, Paladin prepare spells
            _ => None,
        }
    }

    /// Returns how many spells a Wizard adds to their spellbook at the given level.
    /// Wizards add 2 spells per level (6 at level 1, then 2 per level after).
    pub fn wizard_spellbook_spells_at_level(&self, level: u8) -> usize {
        match self {
            CharacterClass::Wizard => {
                if level == 1 {
                    6
                } else {
                    2
                }
            }
            _ => 0,
        }
    }

    /// Returns the maximum number of spells a prepared caster can prepare.
    /// Formula: spellcasting ability modifier + class level (minimum 1).
    /// For half-casters (Paladin, Ranger), it's ability mod + half class level.
    pub fn max_prepared_spells(&self, level: u8, ability_modifier: i8) -> Option<usize> {
        let base = match self {
            CharacterClass::Cleric | CharacterClass::Druid => {
                (ability_modifier as i32 + level as i32).max(1) as usize
            }
            CharacterClass::Paladin => {
                if level >= 2 {
                    (ability_modifier as i32 + (level as i32 / 2)).max(1) as usize
                } else {
                    0
                }
            }
            // Ranger uses spells known, not prepared
            _ => return None,
        };
        Some(base)
    }

    /// Returns the highest spell level this class can cast at a given character level.
    pub fn max_spell_level(&self, level: u8) -> u8 {
        let slots = self.spell_slots_at_level(level);
        for (spell_level, &count) in slots.iter().enumerate().rev() {
            if count > 0 {
                return (spell_level + 1) as u8;
            }
        }
        0
    }

    /// Returns spell slots for a given character level.
    /// Returns an array of 9 elements representing slots for spell levels 1-9.
    pub fn spell_slots_at_level(&self, level: u8) -> [u8; 9] {
        // Full casters: Bard, Cleric, Druid, Sorcerer, Wizard
        // Half casters: Paladin, Ranger (start at level 2)
        // Pact Magic: Warlock (different progression)

        match self {
            // Full casters (standard progression)
            CharacterClass::Bard
            | CharacterClass::Cleric
            | CharacterClass::Druid
            | CharacterClass::Sorcerer
            | CharacterClass::Wizard => full_caster_slots(level),

            // Half casters (Paladin, Ranger)
            CharacterClass::Paladin | CharacterClass::Ranger => half_caster_slots(level),

            // Warlock uses Pact Magic
            CharacterClass::Warlock => warlock_slots(level),

            // Non-casters
            _ => [0; 9],
        }
    }
}

/// Standard full caster spell slot progression (D&D 5e SRD).
fn full_caster_slots(level: u8) -> [u8; 9] {
    match level {
        1 => [2, 0, 0, 0, 0, 0, 0, 0, 0],
        2 => [3, 0, 0, 0, 0, 0, 0, 0, 0],
        3 => [4, 2, 0, 0, 0, 0, 0, 0, 0],
        4 => [4, 3, 0, 0, 0, 0, 0, 0, 0],
        5 => [4, 3, 2, 0, 0, 0, 0, 0, 0],
        6 => [4, 3, 3, 0, 0, 0, 0, 0, 0],
        7 => [4, 3, 3, 1, 0, 0, 0, 0, 0],
        8 => [4, 3, 3, 2, 0, 0, 0, 0, 0],
        9 => [4, 3, 3, 3, 1, 0, 0, 0, 0],
        10 => [4, 3, 3, 3, 2, 0, 0, 0, 0],
        11 => [4, 3, 3, 3, 2, 1, 0, 0, 0],
        12 => [4, 3, 3, 3, 2, 1, 0, 0, 0],
        13 => [4, 3, 3, 3, 2, 1, 1, 0, 0],
        14 => [4, 3, 3, 3, 2, 1, 1, 0, 0],
        15 => [4, 3, 3, 3, 2, 1, 1, 1, 0],
        16 => [4, 3, 3, 3, 2, 1, 1, 1, 0],
        17 => [4, 3, 3, 3, 2, 1, 1, 1, 1],
        18 => [4, 3, 3, 3, 3, 1, 1, 1, 1],
        19 => [4, 3, 3, 3, 3, 2, 1, 1, 1],
        20 => [4, 3, 3, 3, 3, 2, 2, 1, 1],
        _ => [0; 9],
    }
}

/// Half caster spell slot progression (Paladin, Ranger).
fn half_caster_slots(level: u8) -> [u8; 9] {
    match level {
        1 => [0, 0, 0, 0, 0, 0, 0, 0, 0], // No slots at level 1
        2 => [2, 0, 0, 0, 0, 0, 0, 0, 0],
        3 => [3, 0, 0, 0, 0, 0, 0, 0, 0],
        4 => [3, 0, 0, 0, 0, 0, 0, 0, 0],
        5 => [4, 2, 0, 0, 0, 0, 0, 0, 0],
        6 => [4, 2, 0, 0, 0, 0, 0, 0, 0],
        7 => [4, 3, 0, 0, 0, 0, 0, 0, 0],
        8 => [4, 3, 0, 0, 0, 0, 0, 0, 0],
        9 => [4, 3, 2, 0, 0, 0, 0, 0, 0],
        10 => [4, 3, 2, 0, 0, 0, 0, 0, 0],
        11 => [4, 3, 3, 0, 0, 0, 0, 0, 0],
        12 => [4, 3, 3, 0, 0, 0, 0, 0, 0],
        13 => [4, 3, 3, 1, 0, 0, 0, 0, 0],
        14 => [4, 3, 3, 1, 0, 0, 0, 0, 0],
        15 => [4, 3, 3, 2, 0, 0, 0, 0, 0],
        16 => [4, 3, 3, 2, 0, 0, 0, 0, 0],
        17 => [4, 3, 3, 3, 1, 0, 0, 0, 0],
        18 => [4, 3, 3, 3, 1, 0, 0, 0, 0],
        19 => [4, 3, 3, 3, 2, 0, 0, 0, 0],
        20 => [4, 3, 3, 3, 2, 0, 0, 0, 0],
        _ => [0; 9],
    }
}

/// Warlock Pact Magic slot progression.
fn warlock_slots(level: u8) -> [u8; 9] {
    // Warlocks have fewer slots but they're all at their highest available level
    // and recharge on short rest. For simplicity, we track them in the slot array
    // at the appropriate level.
    match level {
        1 => [1, 0, 0, 0, 0, 0, 0, 0, 0], // 1 slot, 1st level
        2 => [2, 0, 0, 0, 0, 0, 0, 0, 0], // 2 slots, 1st level
        3 => [0, 2, 0, 0, 0, 0, 0, 0, 0], // 2 slots, 2nd level
        4 => [0, 2, 0, 0, 0, 0, 0, 0, 0],
        5 => [0, 0, 2, 0, 0, 0, 0, 0, 0], // 2 slots, 3rd level
        6 => [0, 0, 2, 0, 0, 0, 0, 0, 0],
        7 => [0, 0, 0, 2, 0, 0, 0, 0, 0], // 2 slots, 4th level
        8 => [0, 0, 0, 2, 0, 0, 0, 0, 0],
        9 => [0, 0, 0, 0, 2, 0, 0, 0, 0], // 2 slots, 5th level
        10 => [0, 0, 0, 0, 2, 0, 0, 0, 0],
        11 => [0, 0, 0, 0, 3, 0, 0, 0, 0], // 3 slots, 5th level
        12 => [0, 0, 0, 0, 3, 0, 0, 0, 0],
        13 => [0, 0, 0, 0, 3, 0, 0, 0, 0],
        14 => [0, 0, 0, 0, 3, 0, 0, 0, 0],
        15 => [0, 0, 0, 0, 3, 0, 0, 0, 0],
        16 => [0, 0, 0, 0, 3, 0, 0, 0, 0],
        17 => [0, 0, 0, 0, 4, 0, 0, 0, 0], // 4 slots, 5th level
        18 => [0, 0, 0, 0, 4, 0, 0, 0, 0],
        19 => [0, 0, 0, 0, 4, 0, 0, 0, 0],
        20 => [0, 0, 0, 0, 4, 0, 0, 0, 0],
        _ => [0; 9],
    }
}

impl fmt::Display for CharacterClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Class information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassLevel {
    pub class: CharacterClass,
    pub level: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subclass: Option<super::subclasses::Subclass>,
}

/// Class feature/ability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feature {
    pub name: String,
    pub description: String,
    pub source: String,
    pub uses: Option<FeatureUses>,
}

/// Limited use tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureUses {
    pub current: u8,
    pub maximum: u8,
    pub recharge: RechargeType,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RechargeType {
    ShortRest,
    LongRest,
    Dawn,
}

// ============================================================================
// Class Resources
// ============================================================================

/// Tracks class-specific resources that need to be managed separately
/// from general features due to their special mechanics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClassResources {
    // Barbarian
    /// Whether the character is currently raging
    pub rage_active: bool,
    /// Rounds remaining in current rage (rage ends after 1 minute = 10 rounds)
    pub rage_rounds_remaining: Option<u8>,
    /// Current rage damage bonus (+2/+3/+4 based on level)
    pub rage_damage_bonus: i8,

    // Monk
    /// Current ki points (called "Monk's Focus" in SRD 5.2)
    pub ki_points: u8,
    /// Maximum ki points (equals Monk level)
    pub max_ki_points: u8,

    // Druid
    /// Current Wild Shape form (None if not transformed)
    pub wild_shape_form: Option<String>,
    /// Remaining HP in Wild Shape form
    pub wild_shape_hp: Option<i32>,

    // Bard
    /// Current Bardic Inspiration uses remaining
    pub bardic_inspiration_uses: u8,
    /// Maximum Bardic Inspiration uses (equals Charisma modifier, minimum 1)
    pub max_bardic_inspiration: u8,

    // Cleric/Paladin
    /// Whether Channel Divinity has been used this short rest
    pub channel_divinity_used: bool,

    // Paladin
    /// Current Lay on Hands pool (max = 5 × Paladin level)
    pub lay_on_hands_pool: u32,
    /// Maximum Lay on Hands pool
    pub lay_on_hands_max: u32,

    // Sorcerer
    /// Current sorcery points
    pub sorcery_points: u8,
    /// Maximum sorcery points (equals Sorcerer level)
    pub max_sorcery_points: u8,

    // Fighter
    /// Whether Action Surge has been used this short rest
    pub action_surge_used: bool,
    /// Whether Second Wind has been used this short rest
    pub second_wind_used: bool,

    // Wizard
    /// Spell slot levels recovered via Arcane Recovery today
    pub arcane_recovery_used: u8,
}

impl ClassResources {
    /// Create default class resources
    pub fn new() -> Self {
        Self::default()
    }

    /// Initialize resources for a specific class at a given level
    pub fn initialize_for_class(&mut self, class: CharacterClass, level: u8) {
        match class {
            CharacterClass::Barbarian => {
                // Rage uses are tracked via Feature, but we track active state
                self.rage_active = false;
                self.rage_rounds_remaining = None;
            }
            CharacterClass::Bard => {
                // Bardic Inspiration uses = CHA modifier (set by character builder)
                // Default to 1 (minimum), actual value set based on ability scores
                self.bardic_inspiration_uses = 1;
                self.max_bardic_inspiration = 1;
            }
            CharacterClass::Monk => {
                // Ki points equal Monk level (starting at level 2)
                if level >= 2 {
                    self.ki_points = level;
                    self.max_ki_points = level;
                }
            }
            CharacterClass::Cleric => {
                // Channel Divinity starts fresh
                self.channel_divinity_used = false;
            }
            CharacterClass::Paladin => {
                // Lay on Hands pool = 5 × Paladin level
                self.lay_on_hands_pool = 5 * level as u32;
                self.lay_on_hands_max = 5 * level as u32;
                // Channel Divinity starts fresh
                self.channel_divinity_used = false;
            }
            CharacterClass::Sorcerer => {
                // Sorcery points equal Sorcerer level (starting at level 2)
                if level >= 2 {
                    self.sorcery_points = level;
                    self.max_sorcery_points = level;
                }
            }
            CharacterClass::Fighter => {
                self.action_surge_used = false;
                self.second_wind_used = false;
            }
            CharacterClass::Wizard => {
                self.arcane_recovery_used = 0;
            }
            _ => {}
        }
    }

    /// Reset resources on a short rest
    pub fn short_rest_recovery(&mut self, class: CharacterClass, level: u8) {
        match class {
            CharacterClass::Bard => {
                // Font of Inspiration (level 5+) allows recovery on short rest
                if level >= 5 {
                    self.bardic_inspiration_uses = self.max_bardic_inspiration;
                }
            }
            CharacterClass::Fighter => {
                self.action_surge_used = false;
                self.second_wind_used = false;
            }
            CharacterClass::Cleric | CharacterClass::Paladin => {
                // Channel Divinity recovers on short rest
                self.channel_divinity_used = false;
            }
            CharacterClass::Monk => {
                // Ki points don't recover on short rest in base rules
                // (Uncanny Metabolism at level 2 lets them recover some)
            }
            _ => {}
        }
        let _ = level; // Used for Bard Font of Inspiration check
    }

    /// Reset resources on a long rest
    pub fn long_rest_recovery(&mut self, class: CharacterClass, level: u8) {
        // Long rest recovers everything a short rest does
        self.short_rest_recovery(class, level);

        match class {
            CharacterClass::Barbarian => {
                self.rage_active = false;
                self.rage_rounds_remaining = None;
            }
            CharacterClass::Bard => {
                // Full recovery on long rest
                self.bardic_inspiration_uses = self.max_bardic_inspiration;
            }
            CharacterClass::Monk => {
                self.ki_points = self.max_ki_points;
            }
            CharacterClass::Paladin => {
                self.lay_on_hands_pool = self.lay_on_hands_max;
            }
            CharacterClass::Sorcerer => {
                self.sorcery_points = self.max_sorcery_points;
            }
            CharacterClass::Wizard => {
                self.arcane_recovery_used = 0;
            }
            _ => {}
        }
    }
}
