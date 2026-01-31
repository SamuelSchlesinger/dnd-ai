//! Character builder for D&D 5e character creation.
//!
//! Provides a step-by-step builder for creating new characters with
//! proper D&D 5e rules for ability scores, class features, and proficiencies.

use crate::world::{
    Ability, AbilityScores, Background, Character, CharacterClass, ClassLevel, ClassResources,
    HitDice, HitPoints, ProficiencyLevel, Race, RaceType, Skill, SlotInfo, Speed, SpellSlots,
    SpellcastingData,
};
use std::collections::{HashMap, HashSet};

/// Method for determining ability scores.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AbilityMethod {
    /// Standard array: 15, 14, 13, 12, 10, 8
    #[default]
    StandardArray,
    /// Point buy: 27 points, scores 8-15
    PointBuy,
    /// Roll 4d6, drop lowest, 6 times
    Rolled,
}

impl AbilityMethod {
    pub fn name(&self) -> &'static str {
        match self {
            AbilityMethod::StandardArray => "Standard Array",
            AbilityMethod::PointBuy => "Point Buy",
            AbilityMethod::Rolled => "Rolled",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            AbilityMethod::StandardArray => "Assign 15, 14, 13, 12, 10, 8 to your abilities",
            AbilityMethod::PointBuy => "Spend 27 points to customize your scores (8-15 range)",
            AbilityMethod::Rolled => "Roll 4d6, drop the lowest die, six times",
        }
    }

    pub fn all() -> &'static [AbilityMethod] {
        &[
            AbilityMethod::StandardArray,
            AbilityMethod::PointBuy,
            AbilityMethod::Rolled,
        ]
    }
}

/// Standard array values.
pub const STANDARD_ARRAY: [u8; 6] = [15, 14, 13, 12, 10, 8];

/// Point buy costs for each score value.
pub fn point_buy_cost(score: u8) -> Option<u8> {
    match score {
        8 => Some(0),
        9 => Some(1),
        10 => Some(2),
        11 => Some(3),
        12 => Some(4),
        13 => Some(5),
        14 => Some(7),
        15 => Some(9),
        _ => None,
    }
}

/// Total points available for point buy.
pub const POINT_BUY_TOTAL: u8 = 27;

/// Builder for creating D&D 5e characters.
#[derive(Debug, Clone, Default)]
pub struct CharacterBuilder {
    name: Option<String>,
    race: Option<RaceType>,
    class: Option<CharacterClass>,
    background: Option<Background>,
    ability_scores: Option<AbilityScores>,
    ability_method: AbilityMethod,
    selected_skills: Vec<Skill>,
    /// For Half-Elf: two additional +1 ability bonuses
    half_elf_bonus_abilities: Option<[Ability; 2]>,
    /// Optional character backstory
    backstory: Option<String>,
}

/// Error from character building.
#[derive(Debug, Clone)]
pub enum BuilderError {
    MissingName,
    MissingRace,
    MissingClass,
    MissingBackground,
    MissingAbilityScores,
    InvalidSkillCount { expected: usize, got: usize },
    SkillNotAvailable(Skill),
    HalfElfNeedsBonusAbilities,
}

impl std::fmt::Display for BuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuilderError::MissingName => write!(f, "Character name is required"),
            BuilderError::MissingRace => write!(f, "Race selection is required"),
            BuilderError::MissingClass => write!(f, "Class selection is required"),
            BuilderError::MissingBackground => write!(f, "Background selection is required"),
            BuilderError::MissingAbilityScores => write!(f, "Ability scores are required"),
            BuilderError::InvalidSkillCount { expected, got } => {
                write!(f, "Expected {expected} skills, got {got}")
            }
            BuilderError::SkillNotAvailable(skill) => {
                write!(f, "Skill {} is not available for this class", skill.name())
            }
            BuilderError::HalfElfNeedsBonusAbilities => {
                write!(f, "Half-Elf requires two additional ability bonuses")
            }
        }
    }
}

impl std::error::Error for BuilderError {}

impl CharacterBuilder {
    /// Create a new character builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the character's name.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the character's race.
    pub fn race(mut self, race: RaceType) -> Self {
        self.race = Some(race);
        self
    }

    /// Set the character's class.
    pub fn class(mut self, class: CharacterClass) -> Self {
        self.class = Some(class);
        self
    }

    /// Set the character's background.
    pub fn background(mut self, background: Background) -> Self {
        self.background = Some(background);
        self
    }

    /// Set the ability score method.
    pub fn ability_method(mut self, method: AbilityMethod) -> Self {
        self.ability_method = method;
        self
    }

    /// Set the ability scores directly.
    pub fn ability_scores(mut self, scores: AbilityScores) -> Self {
        self.ability_scores = Some(scores);
        self
    }

    /// Set ability scores from standard array assignment.
    ///
    /// `assignment` maps each standard array value (15, 14, 13, 12, 10, 8) to an ability.
    pub fn standard_array(mut self, assignment: [(u8, Ability); 6]) -> Self {
        let mut scores = AbilityScores::default();
        for (value, ability) in assignment {
            scores.set(ability, value);
        }
        self.ability_scores = Some(scores);
        self.ability_method = AbilityMethod::StandardArray;
        self
    }

    /// Set ability scores from point buy values.
    pub fn point_buy(mut self, scores: AbilityScores) -> Self {
        self.ability_scores = Some(scores);
        self.ability_method = AbilityMethod::PointBuy;
        self
    }

    /// Set ability scores from rolled values.
    pub fn rolled(mut self, scores: AbilityScores) -> Self {
        self.ability_scores = Some(scores);
        self.ability_method = AbilityMethod::Rolled;
        self
    }

    /// Set the skills selected from class options.
    pub fn skills(mut self, skills: Vec<Skill>) -> Self {
        self.selected_skills = skills;
        self
    }

    /// For Half-Elf: set the two abilities that get +1 bonus.
    pub fn half_elf_bonuses(mut self, abilities: [Ability; 2]) -> Self {
        self.half_elf_bonus_abilities = Some(abilities);
        self
    }

    /// Set the character's backstory.
    pub fn backstory(mut self, backstory: impl Into<String>) -> Self {
        self.backstory = Some(backstory.into());
        self
    }

    /// Build the character, returning an error if any required field is missing.
    pub fn build(self) -> Result<Character, BuilderError> {
        let name = self.name.ok_or(BuilderError::MissingName)?;
        let race = self.race.ok_or(BuilderError::MissingRace)?;
        let class = self.class.ok_or(BuilderError::MissingClass)?;
        let background = self.background.ok_or(BuilderError::MissingBackground)?;
        let mut ability_scores = self
            .ability_scores
            .ok_or(BuilderError::MissingAbilityScores)?;

        // Apply racial ability bonuses
        race.apply_ability_bonuses(&mut ability_scores);

        // Handle Half-Elf's extra +1 to two abilities
        if race == RaceType::HalfElf {
            let bonuses = self
                .half_elf_bonus_abilities
                .ok_or(BuilderError::HalfElfNeedsBonusAbilities)?;
            for ability in bonuses {
                let current = ability_scores.get(ability);
                ability_scores.set(ability, current + 1);
            }
        }

        // Get class data
        let class_data = class.data();

        // Validate skill count
        let expected_skills = class_data.skill_count;
        if self.selected_skills.len() != expected_skills {
            return Err(BuilderError::InvalidSkillCount {
                expected: expected_skills,
                got: self.selected_skills.len(),
            });
        }

        // Validate skill availability
        for skill in &self.selected_skills {
            if !class_data.skill_options.contains(skill) {
                return Err(BuilderError::SkillNotAvailable(*skill));
            }
        }

        // Calculate HP
        let con_mod = ability_scores.modifier(Ability::Constitution);
        let hp = (class_data.base_hp + con_mod as i32).max(1);

        // Build the character
        let mut character = Character::new(&name);
        character.ability_scores = ability_scores;
        character.level = 1;
        character.hit_points = HitPoints::new(hp);

        // Set hit dice
        character.hit_dice = HitDice::new();
        character.hit_dice.add(class.hit_die(), 1);

        // Set class
        character.classes = vec![ClassLevel {
            class,
            level: 1,
            subclass: None,
        }];

        // Set saving throw proficiencies
        character.saving_throw_proficiencies = HashSet::new();
        for ability in class_data.saving_throws {
            character.saving_throw_proficiencies.insert(ability);
        }

        // Set skill proficiencies (class skills)
        character.skill_proficiencies = HashMap::new();
        for skill in &self.selected_skills {
            character
                .skill_proficiencies
                .insert(*skill, ProficiencyLevel::Proficient);
        }

        // Add background skill proficiencies
        for skill in background.skill_proficiencies() {
            character
                .skill_proficiencies
                .entry(skill)
                .or_insert(ProficiencyLevel::Proficient);
        }

        // Set features
        character.features = class_data.level_1_features;

        // Set race
        character.race = Race {
            name: race.name().to_string(),
            subrace: None,
            race_type: Some(race),
        };
        character.race_type = race;

        // Set background
        character.background = background;
        character.background_name = background.name().to_string();

        // Set speed
        character.speed = Speed::new(race.base_speed());

        // Set backstory
        character.backstory = self.backstory;

        // Initialize spellcasting for spellcasting classes
        if class.is_spellcaster() {
            if let Some(ability) = class.spellcasting_ability() {
                let mut spell_slots = SpellSlots::new();

                // Set up level 1 spell slots based on class
                match class {
                    CharacterClass::Warlock => {
                        // Pact Magic: 1 first-level slot at level 1
                        spell_slots.slots[0] = SlotInfo { total: 1, used: 0 };
                    }
                    CharacterClass::Bard
                    | CharacterClass::Cleric
                    | CharacterClass::Druid
                    | CharacterClass::Sorcerer
                    | CharacterClass::Wizard => {
                        // Standard spellcasting: 2 first-level slots at level 1
                        spell_slots.slots[0] = SlotInfo { total: 2, used: 0 };
                    }
                    _ => {}
                }

                // Default cantrips based on class (using spells from the database)
                let cantrips_known = match class {
                    CharacterClass::Wizard => {
                        vec![
                            "Fire Bolt".to_string(),
                            "Light".to_string(),
                            "Mage Hand".to_string(),
                        ]
                    }
                    CharacterClass::Sorcerer => {
                        vec![
                            "Fire Bolt".to_string(),
                            "Ray of Frost".to_string(),
                            "Light".to_string(),
                            "Prestidigitation".to_string(),
                        ]
                    }
                    CharacterClass::Warlock => {
                        vec!["Eldritch Blast".to_string(), "Chill Touch".to_string()]
                    }
                    CharacterClass::Cleric => {
                        vec![
                            "Sacred Flame".to_string(),
                            "Light".to_string(),
                            "Spare the Dying".to_string(),
                        ]
                    }
                    CharacterClass::Bard => {
                        vec!["Light".to_string(), "Mage Hand".to_string()]
                    }
                    CharacterClass::Druid => {
                        vec!["Produce Flame".to_string(), "Druidcraft".to_string()]
                    }
                    _ => vec![],
                };

                // Default spells known based on class
                let spells_known = match class {
                    CharacterClass::Wizard => {
                        vec![
                            "Magic Missile".to_string(),
                            "Shield".to_string(),
                            "Burning Hands".to_string(),
                            "Detect Magic".to_string(),
                            "Sleep".to_string(),
                            "Mage Armor".to_string(),
                        ]
                    }
                    CharacterClass::Sorcerer => {
                        vec!["Magic Missile".to_string(), "Shield".to_string()]
                    }
                    CharacterClass::Warlock => {
                        // Warlocks know 2 spells at level 1
                        vec!["Hex".to_string(), "Hellish Rebuke".to_string()]
                    }
                    CharacterClass::Bard => {
                        vec![
                            "Cure Wounds".to_string(),
                            "Healing Word".to_string(),
                            "Charm Person".to_string(),
                            "Dissonant Whispers".to_string(),
                        ]
                    }
                    // Clerics and Druids prepare spells, so no spells_known
                    _ => vec![],
                };

                // For prepared casters, spells_prepared starts empty
                // (player chooses after long rest)
                let spells_prepared = vec![];

                character.spellcasting = Some(SpellcastingData {
                    ability,
                    spells_known,
                    spells_prepared,
                    cantrips_known,
                    spell_slots,
                });
            }
        }

        // Initialize class-specific resources
        let mut class_resources = ClassResources::default();
        class_resources.initialize_for_class(class, 1);

        // Set Bard's Bardic Inspiration uses based on Charisma modifier
        if class == CharacterClass::Bard {
            let cha_mod = character.ability_scores.modifier(Ability::Charisma);
            let uses = (cha_mod.max(1)) as u8; // Minimum 1 use
            class_resources.bardic_inspiration_uses = uses;
            class_resources.max_bardic_inspiration = uses;
        }

        character.class_resources = class_resources;

        Ok(character)
    }
}

/// Roll 4d6, drop lowest, for ability score generation.
pub fn roll_4d6_drop_lowest() -> u8 {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let mut rolls: Vec<u8> = (0..4).map(|_| rng.gen_range(1..=6)).collect();
    rolls.sort();
    rolls[1..].iter().sum()
}

/// Roll a full set of ability scores (6 values).
pub fn roll_ability_scores() -> [u8; 6] {
    let mut scores: Vec<u8> = (0..6).map(|_| roll_4d6_drop_lowest()).collect();
    scores.sort_by(|a, b| b.cmp(a)); // Sort descending
    [
        scores[0], scores[1], scores[2], scores[3], scores[4], scores[5],
    ]
}

/// Validate point buy scores.
pub fn validate_point_buy(scores: &AbilityScores) -> Result<(), String> {
    let mut total_cost = 0u8;

    for ability in Ability::all() {
        let score = scores.get(ability);
        if !(8..=15).contains(&score) {
            return Err(format!(
                "{} score {} is out of range (8-15)",
                ability.abbreviation(),
                score
            ));
        }
        total_cost += point_buy_cost(score).unwrap();
    }

    if total_cost > POINT_BUY_TOTAL {
        return Err(format!(
            "Total point cost {total_cost} exceeds maximum {POINT_BUY_TOTAL}"
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_fighter() {
        let character = CharacterBuilder::new()
            .name("Thorin")
            .race(RaceType::Dwarf)
            .class(CharacterClass::Fighter)
            .background(Background::Soldier)
            .standard_array([
                (15, Ability::Strength),
                (14, Ability::Constitution),
                (13, Ability::Dexterity),
                (12, Ability::Wisdom),
                (10, Ability::Intelligence),
                (8, Ability::Charisma),
            ])
            .skills(vec![Skill::Athletics, Skill::Perception])
            .build()
            .expect("Should build successfully");

        assert_eq!(character.name, "Thorin");
        assert_eq!(character.race_type, RaceType::Dwarf);
        assert_eq!(character.level, 1);
        // STR 15, CON 14 + 2 (dwarf) = 16
        assert_eq!(character.ability_scores.strength, 15);
        assert_eq!(character.ability_scores.constitution, 16); // 14 + 2 dwarf
                                                               // HP = 10 (fighter) + 3 (CON mod) = 13
        assert_eq!(character.hit_points.maximum, 13);
    }

    #[test]
    fn test_build_wizard() {
        let character = CharacterBuilder::new()
            .name("Gandalf")
            .race(RaceType::Human)
            .class(CharacterClass::Wizard)
            .background(Background::Sage)
            .standard_array([
                (15, Ability::Intelligence),
                (14, Ability::Constitution),
                (13, Ability::Dexterity),
                (12, Ability::Wisdom),
                (10, Ability::Strength),
                (8, Ability::Charisma),
            ])
            .skills(vec![Skill::Arcana, Skill::Investigation])
            .build()
            .expect("Should build successfully");

        assert_eq!(character.name, "Gandalf");
        // Human gets +1 to all
        assert_eq!(character.ability_scores.intelligence, 16); // 15 + 1
        assert_eq!(character.ability_scores.constitution, 15); // 14 + 1
                                                               // HP = 6 (wizard) + 2 (CON mod) = 8
        assert_eq!(character.hit_points.maximum, 8);
    }

    #[test]
    fn test_missing_name_error() {
        let result = CharacterBuilder::new()
            .race(RaceType::Human)
            .class(CharacterClass::Fighter)
            .background(Background::Soldier)
            .standard_array([
                (15, Ability::Strength),
                (14, Ability::Constitution),
                (13, Ability::Dexterity),
                (12, Ability::Wisdom),
                (10, Ability::Intelligence),
                (8, Ability::Charisma),
            ])
            .skills(vec![Skill::Athletics, Skill::Perception])
            .build();

        assert!(matches!(result, Err(BuilderError::MissingName)));
    }

    #[test]
    fn test_wrong_skill_count_error() {
        let result = CharacterBuilder::new()
            .name("Test")
            .race(RaceType::Human)
            .class(CharacterClass::Fighter)
            .background(Background::Soldier)
            .standard_array([
                (15, Ability::Strength),
                (14, Ability::Constitution),
                (13, Ability::Dexterity),
                (12, Ability::Wisdom),
                (10, Ability::Intelligence),
                (8, Ability::Charisma),
            ])
            .skills(vec![Skill::Athletics]) // Only 1, need 2
            .build();

        assert!(matches!(
            result,
            Err(BuilderError::InvalidSkillCount {
                expected: 2,
                got: 1
            })
        ));
    }

    #[test]
    fn test_point_buy_validation() {
        // Valid point buy
        let valid = AbilityScores::new(15, 14, 13, 12, 10, 8);
        assert!(validate_point_buy(&valid).is_ok());

        // Too expensive
        let expensive = AbilityScores::new(15, 15, 15, 15, 8, 8);
        assert!(validate_point_buy(&expensive).is_err());

        // Out of range
        let out_of_range = AbilityScores::new(16, 14, 13, 12, 10, 8);
        assert!(validate_point_buy(&out_of_range).is_err());
    }

    #[test]
    fn test_roll_4d6_drop_lowest() {
        for _ in 0..100 {
            let score = roll_4d6_drop_lowest();
            assert!((3..=18).contains(&score));
        }
    }

    #[test]
    fn test_build_with_backstory() {
        let backstory = "A former soldier who left the army after witnessing too much bloodshed.";

        let character = CharacterBuilder::new()
            .name("Marcus")
            .race(RaceType::Human)
            .class(CharacterClass::Fighter)
            .background(Background::Soldier)
            .standard_array([
                (15, Ability::Strength),
                (14, Ability::Constitution),
                (13, Ability::Dexterity),
                (12, Ability::Wisdom),
                (10, Ability::Intelligence),
                (8, Ability::Charisma),
            ])
            .skills(vec![Skill::Athletics, Skill::Perception])
            .backstory(backstory)
            .build()
            .expect("Should build successfully");

        assert_eq!(character.name, "Marcus");
        assert!(character.backstory.is_some());
        assert_eq!(character.backstory.as_ref().unwrap(), backstory);
    }

    #[test]
    fn test_build_without_backstory() {
        let character = CharacterBuilder::new()
            .name("Jane")
            .race(RaceType::Elf)
            .class(CharacterClass::Wizard)
            .background(Background::Sage)
            .standard_array([
                (15, Ability::Intelligence),
                (14, Ability::Dexterity),
                (13, Ability::Constitution),
                (12, Ability::Wisdom),
                (10, Ability::Strength),
                (8, Ability::Charisma),
            ])
            .skills(vec![Skill::Arcana, Skill::Investigation])
            .build()
            .expect("Should build successfully");

        assert_eq!(character.name, "Jane");
        assert!(character.backstory.is_none());
    }

    #[test]
    fn test_wizard_gets_spellcasting() {
        let character = CharacterBuilder::new()
            .name("Merlin")
            .race(RaceType::Human)
            .class(CharacterClass::Wizard)
            .background(Background::Sage)
            .standard_array([
                (15, Ability::Intelligence),
                (14, Ability::Constitution),
                (13, Ability::Dexterity),
                (12, Ability::Wisdom),
                (10, Ability::Strength),
                (8, Ability::Charisma),
            ])
            .skills(vec![Skill::Arcana, Skill::Investigation])
            .build()
            .expect("Should build successfully");

        // Wizard should have spellcasting initialized
        assert!(character.spellcasting.is_some());
        let spellcasting = character.spellcasting.as_ref().unwrap();

        // Wizard uses Intelligence
        assert_eq!(spellcasting.ability, Ability::Intelligence);

        // Wizard gets 3 cantrips at level 1
        assert_eq!(spellcasting.cantrips_known.len(), 3);
        assert!(spellcasting
            .cantrips_known
            .contains(&"Fire Bolt".to_string()));

        // Wizard gets 6 spells at level 1
        assert_eq!(spellcasting.spells_known.len(), 6);
        assert!(spellcasting
            .spells_known
            .contains(&"Magic Missile".to_string()));

        // Wizard gets 2 first-level slots at level 1
        assert_eq!(spellcasting.spell_slots.slots[0].total, 2);
        assert_eq!(spellcasting.spell_slots.slots[0].used, 0);
    }

    #[test]
    fn test_warlock_gets_pact_magic() {
        let character = CharacterBuilder::new()
            .name("Shadowbane")
            .race(RaceType::Tiefling)
            .class(CharacterClass::Warlock)
            .background(Background::Criminal)
            .standard_array([
                (15, Ability::Charisma),
                (14, Ability::Constitution),
                (13, Ability::Dexterity),
                (12, Ability::Intelligence),
                (10, Ability::Wisdom),
                (8, Ability::Strength),
            ])
            .skills(vec![Skill::Arcana, Skill::Intimidation])
            .build()
            .expect("Should build successfully");

        // Warlock should have spellcasting
        assert!(character.spellcasting.is_some());
        let spellcasting = character.spellcasting.as_ref().unwrap();

        // Warlock uses Charisma
        assert_eq!(spellcasting.ability, Ability::Charisma);

        // Warlock gets 2 cantrips at level 1
        assert_eq!(spellcasting.cantrips_known.len(), 2);
        assert!(spellcasting
            .cantrips_known
            .contains(&"Eldritch Blast".to_string()));

        // Warlock knows 2 spells at level 1
        assert_eq!(spellcasting.spells_known.len(), 2);

        // Warlock gets only 1 first-level slot (Pact Magic)
        assert_eq!(spellcasting.spell_slots.slots[0].total, 1);
    }

    #[test]
    fn test_fighter_has_no_spellcasting() {
        let character = CharacterBuilder::new()
            .name("Conan")
            .race(RaceType::Human)
            .class(CharacterClass::Fighter)
            .background(Background::Soldier)
            .standard_array([
                (15, Ability::Strength),
                (14, Ability::Constitution),
                (13, Ability::Dexterity),
                (12, Ability::Wisdom),
                (10, Ability::Intelligence),
                (8, Ability::Charisma),
            ])
            .skills(vec![Skill::Athletics, Skill::Perception])
            .build()
            .expect("Should build successfully");

        // Fighter is not a spellcaster at level 1
        assert!(character.spellcasting.is_none());
    }
}
