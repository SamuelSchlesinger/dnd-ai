//! D&D 5e class data for character creation.
//!
//! Contains saving throw proficiencies, skill options, and level 1 features
//! for all 12 PHB classes.

use crate::world::{Ability, CharacterClass, Feature, FeatureUses, RechargeType, Skill};

/// Class-specific data for character creation.
pub struct ClassData {
    /// Saving throw proficiencies granted by the class.
    pub saving_throws: [Ability; 2],
    /// Number of skills to choose.
    pub skill_count: usize,
    /// Skills available to choose from.
    pub skill_options: &'static [Skill],
    /// Starting HP at level 1 (hit die max, before CON modifier).
    pub base_hp: i32,
    /// Features gained at level 1.
    pub level_1_features: Vec<Feature>,
}

impl CharacterClass {
    /// Get class data for character creation.
    pub fn data(&self) -> ClassData {
        match self {
            CharacterClass::Barbarian => ClassData {
                saving_throws: [Ability::Strength, Ability::Constitution],
                skill_count: 2,
                skill_options: &[
                    Skill::AnimalHandling,
                    Skill::Athletics,
                    Skill::Intimidation,
                    Skill::Nature,
                    Skill::Perception,
                    Skill::Survival,
                ],
                base_hp: 12,
                level_1_features: vec![
                    Feature {
                        name: "Rage".to_string(),
                        description: "Enter a rage as a bonus action. Gain advantage on STR checks/saves, bonus rage damage, and resistance to physical damage.".to_string(),
                        source: "Barbarian".to_string(),
                        uses: Some(FeatureUses {
                            current: 2,
                            maximum: 2,
                            recharge: RechargeType::LongRest,
                        }),
                    },
                    Feature {
                        name: "Unarmored Defense".to_string(),
                        description: "While not wearing armor, your AC equals 10 + DEX modifier + CON modifier.".to_string(),
                        source: "Barbarian".to_string(),
                        uses: None,
                    },
                ],
            },
            CharacterClass::Bard => ClassData {
                saving_throws: [Ability::Dexterity, Ability::Charisma],
                skill_count: 3,
                skill_options: &[
                    Skill::Acrobatics, Skill::AnimalHandling, Skill::Arcana, Skill::Athletics,
                    Skill::Deception, Skill::History, Skill::Insight, Skill::Intimidation,
                    Skill::Investigation, Skill::Medicine, Skill::Nature, Skill::Perception,
                    Skill::Performance, Skill::Persuasion, Skill::Religion, Skill::SleightOfHand,
                    Skill::Stealth, Skill::Survival,
                ],
                base_hp: 8,
                level_1_features: vec![
                    Feature {
                        name: "Bardic Inspiration".to_string(),
                        description: "As a bonus action, give one creature a d6 inspiration die to add to one ability check, attack, or save.".to_string(),
                        source: "Bard".to_string(),
                        uses: Some(FeatureUses {
                            current: 3, // Assumes CHA mod of +3
                            maximum: 3,
                            recharge: RechargeType::LongRest,
                        }),
                    },
                    Feature {
                        name: "Spellcasting".to_string(),
                        description: "You can cast bard spells using Charisma as your spellcasting ability.".to_string(),
                        source: "Bard".to_string(),
                        uses: None,
                    },
                ],
            },
            CharacterClass::Cleric => ClassData {
                saving_throws: [Ability::Wisdom, Ability::Charisma],
                skill_count: 2,
                skill_options: &[
                    Skill::History,
                    Skill::Insight,
                    Skill::Medicine,
                    Skill::Persuasion,
                    Skill::Religion,
                ],
                base_hp: 8,
                level_1_features: vec![
                    Feature {
                        name: "Spellcasting".to_string(),
                        description: "You can cast cleric spells using Wisdom as your spellcasting ability.".to_string(),
                        source: "Cleric".to_string(),
                        uses: None,
                    },
                    Feature {
                        name: "Divine Domain".to_string(),
                        description: "Choose a divine domain that grants you additional spells and features.".to_string(),
                        source: "Cleric".to_string(),
                        uses: None,
                    },
                ],
            },
            CharacterClass::Druid => ClassData {
                saving_throws: [Ability::Intelligence, Ability::Wisdom],
                skill_count: 2,
                skill_options: &[
                    Skill::Arcana,
                    Skill::AnimalHandling,
                    Skill::Insight,
                    Skill::Medicine,
                    Skill::Nature,
                    Skill::Perception,
                    Skill::Religion,
                    Skill::Survival,
                ],
                base_hp: 8,
                level_1_features: vec![
                    Feature {
                        name: "Druidic".to_string(),
                        description: "You know Druidic, the secret language of druids.".to_string(),
                        source: "Druid".to_string(),
                        uses: None,
                    },
                    Feature {
                        name: "Spellcasting".to_string(),
                        description: "You can cast druid spells using Wisdom as your spellcasting ability.".to_string(),
                        source: "Druid".to_string(),
                        uses: None,
                    },
                ],
            },
            CharacterClass::Fighter => ClassData {
                saving_throws: [Ability::Strength, Ability::Constitution],
                skill_count: 2,
                skill_options: &[
                    Skill::Acrobatics,
                    Skill::AnimalHandling,
                    Skill::Athletics,
                    Skill::History,
                    Skill::Insight,
                    Skill::Intimidation,
                    Skill::Perception,
                    Skill::Survival,
                ],
                base_hp: 10,
                level_1_features: vec![
                    Feature {
                        name: "Fighting Style".to_string(),
                        description: "You adopt a particular style of fighting as your specialty.".to_string(),
                        source: "Fighter".to_string(),
                        uses: None,
                    },
                    Feature {
                        name: "Second Wind".to_string(),
                        description: "As a bonus action, regain 1d10 + fighter level HP.".to_string(),
                        source: "Fighter".to_string(),
                        uses: Some(FeatureUses {
                            current: 1,
                            maximum: 1,
                            recharge: RechargeType::ShortRest,
                        }),
                    },
                ],
            },
            CharacterClass::Monk => ClassData {
                saving_throws: [Ability::Strength, Ability::Dexterity],
                skill_count: 2,
                skill_options: &[
                    Skill::Acrobatics,
                    Skill::Athletics,
                    Skill::History,
                    Skill::Insight,
                    Skill::Religion,
                    Skill::Stealth,
                ],
                base_hp: 8,
                level_1_features: vec![
                    Feature {
                        name: "Unarmored Defense".to_string(),
                        description: "While not wearing armor, your AC equals 10 + DEX modifier + WIS modifier.".to_string(),
                        source: "Monk".to_string(),
                        uses: None,
                    },
                    Feature {
                        name: "Martial Arts".to_string(),
                        description: "You can use DEX for unarmed strikes and monk weapons. Your unarmed strike damage is 1d4.".to_string(),
                        source: "Monk".to_string(),
                        uses: None,
                    },
                ],
            },
            CharacterClass::Paladin => ClassData {
                saving_throws: [Ability::Wisdom, Ability::Charisma],
                skill_count: 2,
                skill_options: &[
                    Skill::Athletics,
                    Skill::Insight,
                    Skill::Intimidation,
                    Skill::Medicine,
                    Skill::Persuasion,
                    Skill::Religion,
                ],
                base_hp: 10,
                level_1_features: vec![
                    Feature {
                        name: "Divine Sense".to_string(),
                        description: "Detect celestials, fiends, and undead within 60 feet.".to_string(),
                        source: "Paladin".to_string(),
                        uses: Some(FeatureUses {
                            current: 4, // 1 + CHA mod, assuming +3
                            maximum: 4,
                            recharge: RechargeType::LongRest,
                        }),
                    },
                    Feature {
                        name: "Lay on Hands".to_string(),
                        description: "You have a pool of 5 HP to restore with a touch.".to_string(),
                        source: "Paladin".to_string(),
                        uses: Some(FeatureUses {
                            current: 5,
                            maximum: 5,
                            recharge: RechargeType::LongRest,
                        }),
                    },
                ],
            },
            CharacterClass::Ranger => ClassData {
                saving_throws: [Ability::Strength, Ability::Dexterity],
                skill_count: 3,
                skill_options: &[
                    Skill::AnimalHandling,
                    Skill::Athletics,
                    Skill::Insight,
                    Skill::Investigation,
                    Skill::Nature,
                    Skill::Perception,
                    Skill::Stealth,
                    Skill::Survival,
                ],
                base_hp: 10,
                level_1_features: vec![
                    Feature {
                        name: "Favored Enemy".to_string(),
                        description: "Choose a type of favored enemy. You have advantage on tracking them and recalling information about them.".to_string(),
                        source: "Ranger".to_string(),
                        uses: None,
                    },
                    Feature {
                        name: "Natural Explorer".to_string(),
                        description: "Choose a favored terrain. You gain benefits when traveling and foraging in that terrain.".to_string(),
                        source: "Ranger".to_string(),
                        uses: None,
                    },
                ],
            },
            CharacterClass::Rogue => ClassData {
                saving_throws: [Ability::Dexterity, Ability::Intelligence],
                skill_count: 4,
                skill_options: &[
                    Skill::Acrobatics,
                    Skill::Athletics,
                    Skill::Deception,
                    Skill::Insight,
                    Skill::Intimidation,
                    Skill::Investigation,
                    Skill::Perception,
                    Skill::Performance,
                    Skill::Persuasion,
                    Skill::SleightOfHand,
                    Skill::Stealth,
                ],
                base_hp: 8,
                level_1_features: vec![
                    Feature {
                        name: "Expertise".to_string(),
                        description: "Choose two skills to gain expertise in (double proficiency bonus).".to_string(),
                        source: "Rogue".to_string(),
                        uses: None,
                    },
                    Feature {
                        name: "Sneak Attack".to_string(),
                        description: "Once per turn, deal extra 1d6 damage when you have advantage or an ally is adjacent to the target.".to_string(),
                        source: "Rogue".to_string(),
                        uses: None,
                    },
                    Feature {
                        name: "Thieves' Cant".to_string(),
                        description: "You know Thieves' Cant, a secret mix of dialect, jargon, and code.".to_string(),
                        source: "Rogue".to_string(),
                        uses: None,
                    },
                ],
            },
            CharacterClass::Sorcerer => ClassData {
                saving_throws: [Ability::Constitution, Ability::Charisma],
                skill_count: 2,
                skill_options: &[
                    Skill::Arcana,
                    Skill::Deception,
                    Skill::Insight,
                    Skill::Intimidation,
                    Skill::Persuasion,
                    Skill::Religion,
                ],
                base_hp: 6,
                level_1_features: vec![
                    Feature {
                        name: "Spellcasting".to_string(),
                        description: "You can cast sorcerer spells using Charisma as your spellcasting ability.".to_string(),
                        source: "Sorcerer".to_string(),
                        uses: None,
                    },
                    Feature {
                        name: "Sorcerous Origin".to_string(),
                        description: "Choose a sorcerous origin that describes the source of your innate magical power.".to_string(),
                        source: "Sorcerer".to_string(),
                        uses: None,
                    },
                ],
            },
            CharacterClass::Warlock => ClassData {
                saving_throws: [Ability::Wisdom, Ability::Charisma],
                skill_count: 2,
                skill_options: &[
                    Skill::Arcana,
                    Skill::Deception,
                    Skill::History,
                    Skill::Intimidation,
                    Skill::Investigation,
                    Skill::Nature,
                    Skill::Religion,
                ],
                base_hp: 8,
                level_1_features: vec![
                    Feature {
                        name: "Otherworldly Patron".to_string(),
                        description: "You have struck a bargain with an otherworldly being.".to_string(),
                        source: "Warlock".to_string(),
                        uses: None,
                    },
                    Feature {
                        name: "Pact Magic".to_string(),
                        description: "You can cast warlock spells using Charisma. Your spell slots recover on a short rest.".to_string(),
                        source: "Warlock".to_string(),
                        uses: None,
                    },
                ],
            },
            CharacterClass::Wizard => ClassData {
                saving_throws: [Ability::Intelligence, Ability::Wisdom],
                skill_count: 2,
                skill_options: &[
                    Skill::Arcana,
                    Skill::History,
                    Skill::Insight,
                    Skill::Investigation,
                    Skill::Medicine,
                    Skill::Religion,
                ],
                base_hp: 6,
                level_1_features: vec![
                    Feature {
                        name: "Spellcasting".to_string(),
                        description: "You can cast wizard spells using Intelligence as your spellcasting ability.".to_string(),
                        source: "Wizard".to_string(),
                        uses: None,
                    },
                    Feature {
                        name: "Arcane Recovery".to_string(),
                        description: "Once per day during a short rest, recover spell slots with a combined level equal to half your wizard level (rounded up).".to_string(),
                        source: "Wizard".to_string(),
                        uses: Some(FeatureUses {
                            current: 1,
                            maximum: 1,
                            recharge: RechargeType::LongRest,
                        }),
                    },
                ],
            },
        }
    }

    /// Get a short description of the class.
    pub fn description(&self) -> &'static str {
        match self {
            CharacterClass::Barbarian => "A fierce warrior who can enter a battle rage",
            CharacterClass::Bard => "An inspiring magician whose music weaves magic",
            CharacterClass::Cleric => "A priestly champion who wields divine magic",
            CharacterClass::Druid => "A priest of the Old Faith, wielding nature's power",
            CharacterClass::Fighter => "A master of martial combat, skilled with weapons and armor",
            CharacterClass::Monk => "A master of martial arts, harnessing body and soul",
            CharacterClass::Paladin => "A holy warrior bound to a sacred oath",
            CharacterClass::Ranger => "A warrior who combats threats on the edges of civilization",
            CharacterClass::Rogue => "A scoundrel who uses stealth and trickery",
            CharacterClass::Sorcerer => "A spellcaster who draws on inherent magic",
            CharacterClass::Warlock => "A wielder of magic derived from a bargain with an extraplanar entity",
            CharacterClass::Wizard => "A scholarly magic-user who masters arcane secrets",
        }
    }

    /// Get all character classes.
    pub fn all() -> &'static [CharacterClass] {
        &[
            CharacterClass::Barbarian,
            CharacterClass::Bard,
            CharacterClass::Cleric,
            CharacterClass::Druid,
            CharacterClass::Fighter,
            CharacterClass::Monk,
            CharacterClass::Paladin,
            CharacterClass::Ranger,
            CharacterClass::Rogue,
            CharacterClass::Sorcerer,
            CharacterClass::Warlock,
            CharacterClass::Wizard,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_classes_have_data() {
        for class in CharacterClass::all() {
            let data = class.data();
            assert_eq!(data.saving_throws.len(), 2);
            assert!(data.skill_count > 0);
            assert!(!data.skill_options.is_empty());
            assert!(data.base_hp > 0);
        }
    }

    #[test]
    fn test_fighter_data() {
        let data = CharacterClass::Fighter.data();
        assert_eq!(data.saving_throws, [Ability::Strength, Ability::Constitution]);
        assert_eq!(data.skill_count, 2);
        assert_eq!(data.base_hp, 10);
        assert!(data.level_1_features.iter().any(|f| f.name == "Second Wind"));
    }

    #[test]
    fn test_wizard_data() {
        let data = CharacterClass::Wizard.data();
        assert_eq!(data.saving_throws, [Ability::Intelligence, Ability::Wisdom]);
        assert_eq!(data.base_hp, 6);
    }

    #[test]
    fn test_rogue_gets_4_skills() {
        let data = CharacterClass::Rogue.data();
        assert_eq!(data.skill_count, 4);
    }
}
