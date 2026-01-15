//! D&D 5e character representation
//!
//! Complete character sheets with all stats, features, and equipment.

use super::conditions::ActiveCondition;
use super::dice::{DiceExpression, DieType, RollResult};
use super::skills::{ProficiencyLevel, Skill};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;
use uuid::Uuid;

/// Unique identifier for characters
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CharacterId(pub Uuid);

impl CharacterId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for CharacterId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for CharacterId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The six ability scores
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Ability {
    Strength,
    Dexterity,
    Constitution,
    Intelligence,
    Wisdom,
    Charisma,
}

impl Ability {
    pub fn abbreviation(&self) -> &'static str {
        match self {
            Ability::Strength => "STR",
            Ability::Dexterity => "DEX",
            Ability::Constitution => "CON",
            Ability::Intelligence => "INT",
            Ability::Wisdom => "WIS",
            Ability::Charisma => "CHA",
        }
    }

    pub fn all() -> [Ability; 6] {
        [
            Ability::Strength,
            Ability::Dexterity,
            Ability::Constitution,
            Ability::Intelligence,
            Ability::Wisdom,
            Ability::Charisma,
        ]
    }
}

impl fmt::Display for Ability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.abbreviation())
    }
}

/// Ability scores container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbilityScores {
    pub strength: u8,
    pub dexterity: u8,
    pub constitution: u8,
    pub intelligence: u8,
    pub wisdom: u8,
    pub charisma: u8,
}

impl AbilityScores {
    pub fn new(str: u8, dex: u8, con: u8, int: u8, wis: u8, cha: u8) -> Self {
        Self {
            strength: str,
            dexterity: dex,
            constitution: con,
            intelligence: int,
            wisdom: wis,
            charisma: cha,
        }
    }

    /// Standard array: 15, 14, 13, 12, 10, 8
    pub fn standard_array() -> Self {
        Self::new(15, 14, 13, 12, 10, 8)
    }

    /// Get score for an ability
    pub fn get(&self, ability: Ability) -> u8 {
        match ability {
            Ability::Strength => self.strength,
            Ability::Dexterity => self.dexterity,
            Ability::Constitution => self.constitution,
            Ability::Intelligence => self.intelligence,
            Ability::Wisdom => self.wisdom,
            Ability::Charisma => self.charisma,
        }
    }

    /// Set score for an ability
    pub fn set(&mut self, ability: Ability, value: u8) {
        match ability {
            Ability::Strength => self.strength = value,
            Ability::Dexterity => self.dexterity = value,
            Ability::Constitution => self.constitution = value,
            Ability::Intelligence => self.intelligence = value,
            Ability::Wisdom => self.wisdom = value,
            Ability::Charisma => self.charisma = value,
        }
    }

    /// Calculate modifier for an ability score (score - 10) / 2
    pub fn modifier(&self, ability: Ability) -> i8 {
        let score = self.get(ability) as i8;
        (score - 10) / 2
    }
}

impl Default for AbilityScores {
    fn default() -> Self {
        Self::new(10, 10, 10, 10, 10, 10)
    }
}

/// Hit points tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HitPoints {
    pub current: i32,
    pub maximum: i32,
    pub temporary: i32,
}

impl HitPoints {
    pub fn new(maximum: i32) -> Self {
        Self {
            current: maximum,
            maximum,
            temporary: 0,
        }
    }

    /// Take damage, reducing temp HP first
    pub fn take_damage(&mut self, amount: i32) -> DamageResult {
        let mut remaining = amount;

        // Reduce temporary HP first
        if self.temporary > 0 {
            if self.temporary >= remaining {
                self.temporary -= remaining;
                return DamageResult {
                    damage_taken: amount,
                    temp_hp_absorbed: remaining,
                    dropped_to_zero: false,
                    massive_damage: false,
                };
            } else {
                remaining -= self.temporary;
                let temp_absorbed = self.temporary;
                self.temporary = 0;

                self.current -= remaining;
                let dropped = self.current <= 0;
                let massive = (-self.current) >= self.maximum;

                return DamageResult {
                    damage_taken: amount,
                    temp_hp_absorbed: temp_absorbed,
                    dropped_to_zero: dropped,
                    massive_damage: massive,
                };
            }
        }

        self.current -= remaining;
        let dropped = self.current <= 0;
        let massive = (-self.current) >= self.maximum;

        DamageResult {
            damage_taken: amount,
            temp_hp_absorbed: 0,
            dropped_to_zero: dropped,
            massive_damage: massive,
        }
    }

    /// Heal HP (cannot exceed maximum)
    pub fn heal(&mut self, amount: i32) -> i32 {
        let old = self.current;
        self.current = (self.current + amount).min(self.maximum);
        self.current - old
    }

    /// Add temporary HP (doesn't stack, takes higher)
    pub fn add_temp_hp(&mut self, amount: i32) {
        self.temporary = self.temporary.max(amount);
    }

    /// Get effective HP (current + temp)
    pub fn effective(&self) -> i32 {
        self.current + self.temporary
    }

    /// Check if at 0 HP
    pub fn is_unconscious(&self) -> bool {
        self.current <= 0
    }

    /// Get HP as a ratio (0.0 to 1.0)
    pub fn ratio(&self) -> f32 {
        (self.current as f32 / self.maximum as f32).max(0.0)
    }
}

/// Result of taking damage
#[derive(Debug, Clone)]
pub struct DamageResult {
    pub damage_taken: i32,
    pub temp_hp_absorbed: i32,
    pub dropped_to_zero: bool,
    pub massive_damage: bool,
}

/// Hit dice tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HitDice {
    pub total: HashMap<DieType, u8>,
    pub remaining: HashMap<DieType, u8>,
}

impl HitDice {
    pub fn new() -> Self {
        Self {
            total: HashMap::new(),
            remaining: HashMap::new(),
        }
    }

    pub fn add(&mut self, die_type: DieType, count: u8) {
        *self.total.entry(die_type).or_insert(0) += count;
        *self.remaining.entry(die_type).or_insert(0) += count;
    }

    pub fn spend(&mut self, die_type: DieType) -> bool {
        if let Some(remaining) = self.remaining.get_mut(&die_type) {
            if *remaining > 0 {
                *remaining -= 1;
                return true;
            }
        }
        false
    }

    pub fn recover_half(&mut self) {
        for (die_type, total) in &self.total {
            let to_recover = (*total as f32 / 2.0).ceil() as u8;
            if let Some(remaining) = self.remaining.get_mut(die_type) {
                *remaining = (*remaining + to_recover).min(*total);
            }
        }
    }

    pub fn recover_all(&mut self) {
        self.remaining = self.total.clone();
    }
}

impl Default for HitDice {
    fn default() -> Self {
        Self::new()
    }
}

/// Death saving throws
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeathSaves {
    pub successes: u8,
    pub failures: u8,
}

impl DeathSaves {
    pub fn add_success(&mut self) -> DeathSaveOutcome {
        self.successes += 1;
        if self.successes >= 3 {
            DeathSaveOutcome::Stabilized
        } else {
            DeathSaveOutcome::Ongoing
        }
    }

    pub fn add_failure(&mut self) -> DeathSaveOutcome {
        self.failures += 1;
        if self.failures >= 3 {
            DeathSaveOutcome::Dead
        } else {
            DeathSaveOutcome::Ongoing
        }
    }

    pub fn natural_20(&mut self) -> DeathSaveOutcome {
        self.reset();
        DeathSaveOutcome::Conscious
    }

    pub fn natural_1(&mut self) -> DeathSaveOutcome {
        self.failures += 2;
        if self.failures >= 3 {
            DeathSaveOutcome::Dead
        } else {
            DeathSaveOutcome::Ongoing
        }
    }

    pub fn reset(&mut self) {
        self.successes = 0;
        self.failures = 0;
    }

    pub fn is_stable(&self) -> bool {
        self.successes >= 3
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeathSaveOutcome {
    Ongoing,
    Stabilized,
    Conscious,
    Dead,
}

/// Armor class calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArmorClass {
    pub base: u8,
    pub armor_type: Option<ArmorType>,
    pub shield_bonus: u8,
    pub modifiers: Vec<ACModifier>,
}

impl ArmorClass {
    pub fn unarmored(dex_mod: i8) -> Self {
        Self {
            base: 10,
            armor_type: None,
            shield_bonus: 0,
            modifiers: vec![ACModifier {
                source: "Dexterity".to_string(),
                value: dex_mod,
            }],
        }
    }

    pub fn calculate(&self, dex_mod: i8) -> u8 {
        let base = self.base as i8;
        let shield = self.shield_bonus as i8;

        let dex_bonus = match self.armor_type {
            None => dex_mod,
            Some(ArmorType::Light) => dex_mod,
            Some(ArmorType::Medium) => dex_mod.min(2),
            Some(ArmorType::Heavy) => 0,
        };

        let modifier_total: i8 = self
            .modifiers
            .iter()
            .filter(|m| m.source != "Dexterity")
            .map(|m| m.value)
            .sum();

        (base + dex_bonus + shield + modifier_total).max(0) as u8
    }
}

impl Default for ArmorClass {
    fn default() -> Self {
        Self::unarmored(0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ACModifier {
    pub source: String,
    pub value: i8,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ArmorType {
    Light,
    Medium,
    Heavy,
}

/// Movement speed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Speed {
    pub walk: u32,
    pub swim: Option<u32>,
    pub fly: Option<u32>,
    pub climb: Option<u32>,
    pub burrow: Option<u32>,
}

impl Speed {
    pub fn new(walk: u32) -> Self {
        Self {
            walk,
            swim: None,
            fly: None,
            climb: None,
            burrow: None,
        }
    }
}

impl Default for Speed {
    fn default() -> Self {
        Self::new(30)
    }
}

/// Class information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassLevel {
    pub class: CharacterClass,
    pub level: u8,
    pub subclass: Option<String>,
}

/// D&D character classes
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

    pub fn is_spellcaster(&self) -> bool {
        !matches!(
            self,
            CharacterClass::Barbarian | CharacterClass::Fighter | CharacterClass::Rogue
        )
    }
}

impl fmt::Display for CharacterClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Class feature/ability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feature {
    pub name: String,
    pub description: String,
    pub source: String,
    pub uses: Option<FeatureUses>,
}

/// Limited use tracking
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
    Custom,
}

/// Spellcasting data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpellcastingData {
    pub ability: Ability,
    pub spells_known: Vec<String>,
    pub spells_prepared: Vec<String>,
    pub cantrips_known: Vec<String>,
    pub spell_slots: SpellSlots,
    pub concentration: Option<ConcentratedSpell>,
    pub ritual_casting: bool,
}

impl SpellcastingData {
    pub fn spell_save_dc(&self, ability_scores: &AbilityScores, proficiency: i8) -> u8 {
        let ability_mod = ability_scores.modifier(self.ability);
        (8 + proficiency + ability_mod).max(0) as u8
    }

    pub fn spell_attack_bonus(&self, ability_scores: &AbilityScores, proficiency: i8) -> i8 {
        let ability_mod = ability_scores.modifier(self.ability);
        proficiency + ability_mod
    }
}

/// Spell slot tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpellSlots {
    /// Array index 0 = 1st level, index 8 = 9th level
    pub slots: [SlotInfo; 9],
}

impl SpellSlots {
    pub fn new() -> Self {
        Self {
            slots: std::array::from_fn(|_| SlotInfo { total: 0, used: 0 }),
        }
    }

    pub fn get(&self, level: u8) -> Option<&SlotInfo> {
        if level >= 1 && level <= 9 {
            Some(&self.slots[level as usize - 1])
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, level: u8) -> Option<&mut SlotInfo> {
        if level >= 1 && level <= 9 {
            Some(&mut self.slots[level as usize - 1])
        } else {
            None
        }
    }

    pub fn use_slot(&mut self, level: u8) -> bool {
        if let Some(slot) = self.get_mut(level) {
            if slot.available() > 0 {
                slot.used += 1;
                return true;
            }
        }
        false
    }

    pub fn recover_all(&mut self) {
        for slot in &mut self.slots {
            slot.used = 0;
        }
    }
}

impl Default for SpellSlots {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SlotInfo {
    pub total: u8,
    pub used: u8,
}

impl SlotInfo {
    pub fn available(&self) -> u8 {
        self.total.saturating_sub(self.used)
    }
}

/// Active concentration spell
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcentratedSpell {
    pub spell_name: String,
    pub targets: Vec<CharacterId>,
}

/// Inventory item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub name: String,
    pub quantity: u32,
    pub weight: f32,
    pub value_gp: f32,
    pub description: Option<String>,
    pub item_type: ItemType,
    pub magical: bool,
    pub requires_attunement: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ItemType {
    Weapon,
    Armor,
    Potion,
    Scroll,
    Wand,
    Ring,
    Wondrous,
    Adventuring,
    Tool,
    Other,
}

/// Character inventory
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Inventory {
    pub items: Vec<Item>,
    pub gold: f32,
    pub silver: f32,
    pub copper: f32,
    pub platinum: f32,
    pub electrum: f32,
}

impl Inventory {
    pub fn total_weight(&self) -> f32 {
        self.items.iter().map(|i| i.weight * i.quantity as f32).sum()
    }

    pub fn total_gold_value(&self) -> f32 {
        self.gold
            + self.platinum * 10.0
            + self.electrum * 0.5
            + self.silver * 0.1
            + self.copper * 0.01
    }
}

/// Equipped items
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EquippedItems {
    pub main_hand: Option<String>,
    pub off_hand: Option<String>,
    pub armor: Option<String>,
    pub helmet: Option<String>,
    pub cloak: Option<String>,
    pub gloves: Option<String>,
    pub boots: Option<String>,
    pub belt: Option<String>,
    pub amulet: Option<String>,
    pub ring_left: Option<String>,
    pub ring_right: Option<String>,
}

/// Attuned magic item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttunedItem {
    pub item_name: String,
    pub attunement_slot: u8,
}

/// D&D race
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Race {
    pub name: String,
    pub subrace: Option<String>,
    pub traits: Vec<String>,
}

/// Alignment
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Alignment {
    LawfulGood,
    NeutralGood,
    ChaoticGood,
    LawfulNeutral,
    TrueNeutral,
    ChaoticNeutral,
    LawfulEvil,
    NeutralEvil,
    ChaoticEvil,
}

impl fmt::Display for Alignment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Alignment::LawfulGood => "Lawful Good",
            Alignment::NeutralGood => "Neutral Good",
            Alignment::ChaoticGood => "Chaotic Good",
            Alignment::LawfulNeutral => "Lawful Neutral",
            Alignment::TrueNeutral => "True Neutral",
            Alignment::ChaoticNeutral => "Chaotic Neutral",
            Alignment::LawfulEvil => "Lawful Evil",
            Alignment::NeutralEvil => "Neutral Evil",
            Alignment::ChaoticEvil => "Chaotic Evil",
        };
        write!(f, "{}", s)
    }
}

/// Personality traits
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PersonalityTraits {
    pub personality: Vec<String>,
    pub ideals: Vec<String>,
    pub bonds: Vec<String>,
    pub flaws: Vec<String>,
}

/// Complete D&D 5e character
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    pub id: CharacterId,
    pub name: String,
    pub player_name: Option<String>,

    // Core stats
    pub ability_scores: AbilityScores,
    pub level: u8,
    pub experience: u32,

    // Health
    pub hit_points: HitPoints,
    pub hit_dice: HitDice,
    pub death_saves: DeathSaves,

    // Combat
    pub armor_class: ArmorClass,
    pub speed: Speed,
    pub conditions: Vec<ActiveCondition>,

    // Class features
    pub classes: Vec<ClassLevel>,
    pub features: Vec<Feature>,

    // Spellcasting
    pub spellcasting: Option<SpellcastingData>,

    // Skills & proficiencies
    pub skill_proficiencies: HashMap<Skill, ProficiencyLevel>,
    pub saving_throw_proficiencies: HashSet<Ability>,
    pub tool_proficiencies: Vec<String>,
    pub languages: Vec<String>,

    // Equipment
    pub inventory: Inventory,
    pub equipped: EquippedItems,
    pub attunements: Vec<AttunedItem>,

    // Background
    pub race: Race,
    pub background: String,
    pub alignment: Alignment,
    pub traits: PersonalityTraits,
}

impl Character {
    /// Create a new character with basic defaults
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: CharacterId::new(),
            name: name.into(),
            player_name: None,
            ability_scores: AbilityScores::default(),
            level: 1,
            experience: 0,
            hit_points: HitPoints::new(10),
            hit_dice: HitDice::new(),
            death_saves: DeathSaves::default(),
            armor_class: ArmorClass::default(),
            speed: Speed::default(),
            conditions: Vec::new(),
            classes: Vec::new(),
            features: Vec::new(),
            spellcasting: None,
            skill_proficiencies: HashMap::new(),
            saving_throw_proficiencies: HashSet::new(),
            tool_proficiencies: Vec::new(),
            languages: vec!["Common".to_string()],
            inventory: Inventory::default(),
            equipped: EquippedItems::default(),
            attunements: Vec::new(),
            race: Race {
                name: "Human".to_string(),
                subrace: None,
                traits: Vec::new(),
            },
            background: "Adventurer".to_string(),
            alignment: Alignment::TrueNeutral,
            traits: PersonalityTraits::default(),
        }
    }

    /// Get proficiency bonus based on total level
    pub fn proficiency_bonus(&self) -> i8 {
        match self.level {
            1..=4 => 2,
            5..=8 => 3,
            9..=12 => 4,
            13..=16 => 5,
            17..=20 => 6,
            _ => 2,
        }
    }

    /// Calculate initiative modifier
    pub fn initiative_modifier(&self) -> i8 {
        self.ability_scores.modifier(Ability::Dexterity)
        // TODO: Add feature bonuses (Alert, etc.)
    }

    /// Get skill modifier
    pub fn skill_modifier(&self, skill: Skill) -> i8 {
        let ability_mod = self.ability_scores.modifier(skill.ability());
        let proficiency = self
            .skill_proficiencies
            .get(&skill)
            .copied()
            .unwrap_or(ProficiencyLevel::None);
        ability_mod + proficiency.bonus(self.proficiency_bonus())
    }

    /// Get saving throw modifier
    pub fn saving_throw_modifier(&self, ability: Ability) -> i8 {
        let ability_mod = self.ability_scores.modifier(ability);
        if self.saving_throw_proficiencies.contains(&ability) {
            ability_mod + self.proficiency_bonus()
        } else {
            ability_mod
        }
    }

    /// Calculate current AC
    pub fn current_ac(&self) -> u8 {
        self.armor_class
            .calculate(self.ability_scores.modifier(Ability::Dexterity))
    }

    /// Get primary class (highest level)
    pub fn primary_class(&self) -> Option<&ClassLevel> {
        self.classes.iter().max_by_key(|c| c.level)
    }

    /// Check if character is conscious
    pub fn is_conscious(&self) -> bool {
        self.hit_points.current > 0
    }

    /// Get passive perception
    pub fn passive_perception(&self) -> i8 {
        10 + self.skill_modifier(Skill::Perception)
    }
}

/// Create a simple fighter for testing
pub fn create_sample_fighter(name: &str) -> Character {
    let mut character = Character::new(name);

    character.ability_scores = AbilityScores::new(16, 14, 14, 10, 12, 8);
    character.level = 3;
    character.hit_points = HitPoints::new(28);
    character.hit_dice.add(DieType::D10, 3);

    character.classes.push(ClassLevel {
        class: CharacterClass::Fighter,
        level: 3,
        subclass: Some("Champion".to_string()),
    });

    character
        .saving_throw_proficiencies
        .insert(Ability::Strength);
    character
        .saving_throw_proficiencies
        .insert(Ability::Constitution);

    character
        .skill_proficiencies
        .insert(Skill::Athletics, ProficiencyLevel::Proficient);
    character
        .skill_proficiencies
        .insert(Skill::Perception, ProficiencyLevel::Proficient);
    character
        .skill_proficiencies
        .insert(Skill::Intimidation, ProficiencyLevel::Proficient);

    character.armor_class = ArmorClass {
        base: 16, // Chain mail
        armor_type: Some(ArmorType::Heavy),
        shield_bonus: 2,
        modifiers: Vec::new(),
    };

    character.features.push(Feature {
        name: "Second Wind".to_string(),
        description: "Regain 1d10 + fighter level HP as bonus action".to_string(),
        source: "Fighter".to_string(),
        uses: Some(FeatureUses {
            current: 1,
            maximum: 1,
            recharge: RechargeType::ShortRest,
        }),
    });

    character.features.push(Feature {
        name: "Action Surge".to_string(),
        description: "Take one additional action on your turn".to_string(),
        source: "Fighter".to_string(),
        uses: Some(FeatureUses {
            current: 1,
            maximum: 1,
            recharge: RechargeType::ShortRest,
        }),
    });

    character
}
