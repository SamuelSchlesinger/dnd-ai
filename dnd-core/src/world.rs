//! D&D 5e game world types.
//!
//! Contains all types for representing game state: characters, locations,
//! NPCs, quests, combat, conditions, and the complete game world.

use crate::dice::DieType;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;
use uuid::Uuid;

// ============================================================================
// ID Types
// ============================================================================

/// Unique identifier for characters.
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

/// Unique identifier for locations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LocationId(pub Uuid);

impl LocationId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for LocationId {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Ability Scores
// ============================================================================

/// The six ability scores.
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

    pub fn name(&self) -> &'static str {
        match self {
            Ability::Strength => "Strength",
            Ability::Dexterity => "Dexterity",
            Ability::Constitution => "Constitution",
            Ability::Intelligence => "Intelligence",
            Ability::Wisdom => "Wisdom",
            Ability::Charisma => "Charisma",
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

/// Ability scores container.
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

    pub fn standard_array() -> Self {
        Self::new(15, 14, 13, 12, 10, 8)
    }

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

    pub fn modifier(&self, ability: Ability) -> i8 {
        let score = self.get(ability) as i8;
        // Use floor division to correctly handle negative numbers
        // D&D 5e: score 8-9 = -1, 10-11 = 0, 12-13 = +1, etc.
        (score - 10).div_euclid(2)
    }
}

impl Default for AbilityScores {
    fn default() -> Self {
        Self::new(10, 10, 10, 10, 10, 10)
    }
}

// ============================================================================
// Skills
// ============================================================================

/// D&D 5e skills.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Skill {
    Athletics,
    Acrobatics,
    SleightOfHand,
    Stealth,
    Arcana,
    History,
    Investigation,
    Nature,
    Religion,
    AnimalHandling,
    Insight,
    Medicine,
    Perception,
    Survival,
    Deception,
    Intimidation,
    Performance,
    Persuasion,
}

impl Skill {
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
}

impl fmt::Display for Skill {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Proficiency level for skills/tools.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ProficiencyLevel {
    #[default]
    None,
    Half,
    Proficient,
    Expertise,
}

impl ProficiencyLevel {
    pub fn bonus(&self, proficiency_bonus: i8) -> i8 {
        match self {
            ProficiencyLevel::None => 0,
            ProficiencyLevel::Half => proficiency_bonus / 2,
            ProficiencyLevel::Proficient => proficiency_bonus,
            ProficiencyLevel::Expertise => proficiency_bonus * 2,
        }
    }
}

// ============================================================================
// Conditions
// ============================================================================

/// D&D 5e conditions.
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
    Exhaustion(u8),
}

impl Condition {
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
}

impl fmt::Display for Condition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Condition::Exhaustion(level) => write!(f, "Exhaustion ({level})"),
            _ => write!(f, "{}", self.name()),
        }
    }
}

/// A condition applied to a creature with tracking info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveCondition {
    pub condition: Condition,
    pub source: String,
    pub duration_rounds: Option<u32>,
}

impl ActiveCondition {
    pub fn new(condition: Condition, source: impl Into<String>) -> Self {
        Self {
            condition,
            source: source.into(),
            duration_rounds: None,
        }
    }

    pub fn with_duration(mut self, rounds: u32) -> Self {
        self.duration_rounds = Some(rounds);
        self
    }
}

// ============================================================================
// Hit Points and Health
// ============================================================================

/// Hit points tracking.
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

    pub fn take_damage(&mut self, amount: i32) -> DamageResult {
        let mut remaining = amount;

        if self.temporary > 0 {
            if self.temporary >= remaining {
                self.temporary -= remaining;
                return DamageResult {
                    damage_taken: amount,
                    dropped_to_zero: false,
                };
            } else {
                remaining -= self.temporary;
                self.temporary = 0;
            }
        }

        self.current -= remaining;
        DamageResult {
            damage_taken: amount,
            dropped_to_zero: self.current <= 0,
        }
    }

    pub fn heal(&mut self, amount: i32) -> i32 {
        let old = self.current;
        self.current = (self.current + amount).min(self.maximum);
        self.current - old
    }

    pub fn add_temp_hp(&mut self, amount: i32) {
        self.temporary = self.temporary.max(amount);
    }

    pub fn is_unconscious(&self) -> bool {
        self.current <= 0
    }

    pub fn ratio(&self) -> f32 {
        (self.current as f32 / self.maximum as f32).max(0.0)
    }
}

/// Result of taking damage.
#[derive(Debug, Clone)]
pub struct DamageResult {
    pub damage_taken: i32,
    pub dropped_to_zero: bool,
}

/// Hit dice tracking.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HitDice {
    pub total: HashMap<DieType, u8>,
    pub remaining: HashMap<DieType, u8>,
}

impl HitDice {
    pub fn new() -> Self {
        Self::default()
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
}

/// Death saving throws.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeathSaves {
    pub successes: u8,
    pub failures: u8,
}

impl DeathSaves {
    pub fn add_success(&mut self) -> bool {
        self.successes += 1;
        self.successes >= 3
    }

    pub fn add_failure(&mut self) -> bool {
        self.failures += 1;
        self.failures >= 3
    }

    pub fn reset(&mut self) {
        self.successes = 0;
        self.failures = 0;
    }
}

// ============================================================================
// Armor and Defense
// ============================================================================

/// Armor class calculation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArmorClass {
    pub base: u8,
    pub armor_type: Option<ArmorType>,
    pub shield_bonus: u8,
}

impl ArmorClass {
    pub fn unarmored() -> Self {
        Self {
            base: 10,
            armor_type: None,
            shield_bonus: 0,
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

        (base + dex_bonus + shield).max(0) as u8
    }
}

impl Default for ArmorClass {
    fn default() -> Self {
        Self::unarmored()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ArmorType {
    Light,
    Medium,
    Heavy,
}

/// Movement speed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Speed {
    pub walk: u32,
    pub swim: Option<u32>,
    pub fly: Option<u32>,
    pub climb: Option<u32>,
}

impl Speed {
    pub fn new(walk: u32) -> Self {
        Self {
            walk,
            swim: None,
            fly: None,
            climb: None,
        }
    }
}

impl Default for Speed {
    fn default() -> Self {
        Self::new(30)
    }
}

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
    pub subclass: Option<String>,
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

// ============================================================================
// Spellcasting
// ============================================================================

/// Spellcasting data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpellcastingData {
    pub ability: Ability,
    pub spells_known: Vec<String>,
    pub spells_prepared: Vec<String>,
    pub cantrips_known: Vec<String>,
    pub spell_slots: SpellSlots,
}

impl SpellcastingData {
    pub fn spell_save_dc(&self, ability_scores: &AbilityScores, proficiency: i8) -> u8 {
        let ability_mod = ability_scores.modifier(self.ability);
        (8 + proficiency + ability_mod).max(0) as u8
    }

    pub fn spell_attack_bonus(&self, ability_scores: &AbilityScores, proficiency: i8) -> i8 {
        ability_scores.modifier(self.ability) + proficiency
    }
}

/// Spell slot tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpellSlots {
    pub slots: [SlotInfo; 9],
}

impl SpellSlots {
    pub fn new() -> Self {
        Self {
            slots: std::array::from_fn(|_| SlotInfo { total: 0, used: 0 }),
        }
    }

    pub fn use_slot(&mut self, level: u8) -> bool {
        if (1..=9).contains(&level) {
            let slot = &mut self.slots[level as usize - 1];
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

// ============================================================================
// Equipment
// ============================================================================

/// Inventory item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub name: String,
    pub quantity: u32,
    pub weight: f32,
    pub value_gp: f32,
    pub description: Option<String>,
    pub item_type: ItemType,
    pub magical: bool,
}

impl Item {
    /// Returns true if this item type can stack in inventory.
    /// Weapons, armor, and shields don't stack (each is a distinct item).
    /// Consumables and gear can stack.
    pub fn is_stackable(&self) -> bool {
        match self.item_type {
            ItemType::Weapon | ItemType::Armor | ItemType::Shield => false,
            ItemType::Wand | ItemType::Ring | ItemType::Wondrous => false, // Unique items
            ItemType::Potion
            | ItemType::Scroll
            | ItemType::Adventuring
            | ItemType::Tool
            | ItemType::Other => true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemType {
    Weapon,
    Armor,
    Shield,
    Potion,
    Scroll,
    Wand,
    Ring,
    Wondrous,
    Adventuring,
    Tool,
    Other,
}

/// Character inventory.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Inventory {
    pub items: Vec<Item>,
    pub gold: f32,
}

// ============================================================================
// Equipment System
// ============================================================================

/// Equipment slots for what's actively equipped.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Equipment {
    pub armor: Option<ArmorItem>,
    pub shield: Option<Item>,
    pub main_hand: Option<WeaponItem>,
    pub off_hand: Option<Item>,
}

impl Equipment {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Armor with D&D 5e properties.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArmorItem {
    pub base: Item,
    pub armor_type: ArmorType,
    pub base_ac: u8,
    pub strength_requirement: Option<u8>,
    pub stealth_disadvantage: bool,
}

impl ArmorItem {
    pub fn new(name: impl Into<String>, armor_type: ArmorType, base_ac: u8) -> Self {
        Self {
            base: Item {
                name: name.into(),
                quantity: 1,
                weight: 0.0,
                value_gp: 0.0,
                description: None,
                item_type: ItemType::Armor,
                magical: false,
            },
            armor_type,
            base_ac,
            strength_requirement: None,
            stealth_disadvantage: false,
        }
    }

    pub fn with_weight(mut self, weight: f32) -> Self {
        self.base.weight = weight;
        self
    }

    pub fn with_value(mut self, value_gp: f32) -> Self {
        self.base.value_gp = value_gp;
        self
    }

    pub fn with_strength_requirement(mut self, str_req: u8) -> Self {
        self.strength_requirement = Some(str_req);
        self
    }

    pub fn with_stealth_disadvantage(mut self) -> Self {
        self.stealth_disadvantage = true;
        self
    }

    pub fn magical(mut self) -> Self {
        self.base.magical = true;
        self
    }
}

/// Weapons with D&D 5e properties.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponItem {
    pub base: Item,
    pub damage_dice: String,
    pub damage_type: WeaponDamageType,
    pub properties: Vec<WeaponProperty>,
    pub range: Option<(u32, u32)>,
}

impl WeaponItem {
    pub fn new(
        name: impl Into<String>,
        damage_dice: impl Into<String>,
        damage_type: WeaponDamageType,
    ) -> Self {
        Self {
            base: Item {
                name: name.into(),
                quantity: 1,
                weight: 0.0,
                value_gp: 0.0,
                description: None,
                item_type: ItemType::Weapon,
                magical: false,
            },
            damage_dice: damage_dice.into(),
            damage_type,
            properties: Vec::new(),
            range: None,
        }
    }

    pub fn with_weight(mut self, weight: f32) -> Self {
        self.base.weight = weight;
        self
    }

    pub fn with_value(mut self, value_gp: f32) -> Self {
        self.base.value_gp = value_gp;
        self
    }

    pub fn with_properties(mut self, properties: Vec<WeaponProperty>) -> Self {
        self.properties = properties;
        self
    }

    pub fn with_range(mut self, normal: u32, long: u32) -> Self {
        self.range = Some((normal, long));
        self
    }

    pub fn magical(mut self) -> Self {
        self.base.magical = true;
        self
    }

    pub fn is_finesse(&self) -> bool {
        self.properties.contains(&WeaponProperty::Finesse)
    }

    pub fn is_ranged(&self) -> bool {
        self.range.is_some() || self.properties.contains(&WeaponProperty::Thrown)
    }

    pub fn is_two_handed(&self) -> bool {
        self.properties.contains(&WeaponProperty::TwoHanded)
    }

    pub fn versatile_damage(&self) -> Option<&str> {
        for prop in &self.properties {
            if let WeaponProperty::Versatile(dice) = prop {
                return Some(dice);
            }
        }
        None
    }
}

/// Weapon damage type (separate from spell/effect damage types).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeaponDamageType {
    Slashing,
    Piercing,
    Bludgeoning,
}

impl WeaponDamageType {
    pub fn name(&self) -> &'static str {
        match self {
            WeaponDamageType::Slashing => "slashing",
            WeaponDamageType::Piercing => "piercing",
            WeaponDamageType::Bludgeoning => "bludgeoning",
        }
    }
}

/// Weapon properties per D&D 5e.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeaponProperty {
    Finesse,
    Light,
    Heavy,
    TwoHanded,
    Versatile(String),
    Thrown,
    Ammunition,
    Loading,
    Reach,
}

/// Consumable item effects.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsumableEffect {
    /// Healing potion - roll dice and add bonus
    Healing { dice: String, bonus: i32 },
    /// Restore a spell slot of the given level
    RestoreSpellSlot { level: u8 },
    /// Remove a condition
    RemoveCondition { condition: Condition },
    /// Grant a condition for a duration
    GrantCondition {
        condition: Condition,
        duration_rounds: u32,
    },
    /// Cast a spell from a scroll
    CastSpell { spell_name: String, level: u8 },
    /// Grant temporary hit points
    TemporaryHitPoints { amount: i32 },
    /// Grant advantage on a type of roll for duration
    GrantAdvantage {
        roll_type: String,
        duration_rounds: u32,
    },
}

/// A consumable item with its effect.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumableItem {
    pub base: Item,
    pub effect: ConsumableEffect,
}

impl ConsumableItem {
    pub fn healing_potion(
        name: impl Into<String>,
        dice: impl Into<String>,
        bonus: i32,
        value_gp: f32,
    ) -> Self {
        Self {
            base: Item {
                name: name.into(),
                quantity: 1,
                weight: 0.5,
                value_gp,
                description: Some(
                    "A magical potion that restores health when consumed.".to_string(),
                ),
                item_type: ItemType::Potion,
                magical: true,
            },
            effect: ConsumableEffect::Healing {
                dice: dice.into(),
                bonus,
            },
        }
    }

    pub fn spell_scroll(spell_name: impl Into<String>, level: u8, value_gp: f32) -> Self {
        let spell_name_str: String = spell_name.into();
        let name = format!("Scroll of {spell_name_str}");
        Self {
            base: Item {
                name,
                quantity: 1,
                weight: 0.0,
                value_gp,
                description: Some("A magical scroll containing a spell.".to_string()),
                item_type: ItemType::Scroll,
                magical: true,
            },
            effect: ConsumableEffect::CastSpell {
                spell_name: spell_name_str,
                level,
            },
        }
    }
}

impl Inventory {
    pub fn total_weight(&self) -> f32 {
        self.items
            .iter()
            .map(|i| i.weight * i.quantity as f32)
            .sum()
    }

    /// Add an item to the inventory.
    /// Stackable items (potions, scrolls, adventuring gear, etc.) stack with existing items.
    /// Non-stackable items (weapons, armor, shields) are added as separate entries.
    pub fn add_item(&mut self, item: Item) {
        // Only stack stackable item types
        if item.is_stackable() {
            if let Some(existing) = self.items.iter_mut().find(|i| i.name == item.name) {
                existing.quantity += item.quantity;
                return;
            }
        }
        self.items.push(item);
    }

    /// Remove an item from the inventory. Returns true if successful.
    /// Name matching is case-insensitive.
    pub fn remove_item(&mut self, name: &str, quantity: u32) -> bool {
        let name_lower = name.to_lowercase();
        if let Some(idx) = self
            .items
            .iter()
            .position(|i| i.name.to_lowercase() == name_lower)
        {
            if self.items[idx].quantity >= quantity {
                self.items[idx].quantity -= quantity;
                if self.items[idx].quantity == 0 {
                    self.items.remove(idx);
                }
                return true;
            }
        }
        false
    }

    /// Find an item by name.
    pub fn find_item(&self, name: &str) -> Option<&Item> {
        self.items
            .iter()
            .find(|i| i.name.to_lowercase() == name.to_lowercase())
    }

    /// Find an item by name (mutable).
    pub fn find_item_mut(&mut self, name: &str) -> Option<&mut Item> {
        self.items
            .iter_mut()
            .find(|i| i.name.to_lowercase() == name.to_lowercase())
    }

    /// Check if the inventory contains an item.
    pub fn has_item(&self, name: &str) -> bool {
        self.find_item(name).is_some()
    }

    /// Adjust gold amount. Returns new total or error if insufficient funds.
    pub fn adjust_gold(&mut self, amount: f32) -> Result<f32, &'static str> {
        let new_total = self.gold + amount;
        if new_total < 0.0 {
            Err("Insufficient gold")
        } else {
            self.gold = new_total;
            Ok(self.gold)
        }
    }
}

// ============================================================================
// Races
// ============================================================================

/// D&D 5e playable races.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RaceType {
    Human,
    Elf,
    Dwarf,
    Halfling,
    HalfOrc,
    HalfElf,
    Tiefling,
    Gnome,
    Dragonborn,
}

impl RaceType {
    pub fn name(&self) -> &'static str {
        match self {
            RaceType::Human => "Human",
            RaceType::Elf => "Elf",
            RaceType::Dwarf => "Dwarf",
            RaceType::Halfling => "Halfling",
            RaceType::HalfOrc => "Half-Orc",
            RaceType::HalfElf => "Half-Elf",
            RaceType::Tiefling => "Tiefling",
            RaceType::Gnome => "Gnome",
            RaceType::Dragonborn => "Dragonborn",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            RaceType::Human => {
                "Versatile and ambitious, humans are the most adaptable of all races."
            }
            RaceType::Elf => "Graceful and long-lived, elves are masters of magic and artistry.",
            RaceType::Dwarf => {
                "Stout and hardy, dwarves are renowned craftsmen and fierce warriors."
            }
            RaceType::Halfling => {
                "Small but brave, halflings are known for their luck and stealth."
            }
            RaceType::HalfOrc => {
                "Strong and enduring, half-orcs combine human versatility with orcish might."
            }
            RaceType::HalfElf => "Charismatic and adaptable, half-elves bridge two worlds.",
            RaceType::Tiefling => {
                "Touched by infernal heritage, tieflings possess innate magical abilities."
            }
            RaceType::Gnome => {
                "Curious and inventive, gnomes are natural tinkers and illusionists."
            }
            RaceType::Dragonborn => "Proud and powerful, dragonborn carry the blood of dragons.",
        }
    }

    /// Apply racial ability score bonuses to base scores.
    pub fn apply_ability_bonuses(&self, scores: &mut AbilityScores) {
        match self {
            RaceType::Human => {
                scores.strength += 1;
                scores.dexterity += 1;
                scores.constitution += 1;
                scores.intelligence += 1;
                scores.wisdom += 1;
                scores.charisma += 1;
            }
            RaceType::Elf => {
                scores.dexterity += 2;
            }
            RaceType::Dwarf => {
                scores.constitution += 2;
            }
            RaceType::Halfling => {
                scores.dexterity += 2;
            }
            RaceType::HalfOrc => {
                scores.strength += 2;
                scores.constitution += 1;
            }
            RaceType::HalfElf => {
                scores.charisma += 2;
                // Note: Half-elves also get +1 to two other abilities of choice
                // This is handled in character builder
            }
            RaceType::Tiefling => {
                scores.charisma += 2;
                scores.intelligence += 1;
            }
            RaceType::Gnome => {
                scores.intelligence += 2;
            }
            RaceType::Dragonborn => {
                scores.strength += 2;
                scores.charisma += 1;
            }
        }
    }

    /// Get ability bonus description for display.
    pub fn ability_bonuses(&self) -> &'static str {
        match self {
            RaceType::Human => "+1 to all abilities",
            RaceType::Elf => "+2 Dexterity",
            RaceType::Dwarf => "+2 Constitution",
            RaceType::Halfling => "+2 Dexterity",
            RaceType::HalfOrc => "+2 Strength, +1 Constitution",
            RaceType::HalfElf => "+2 Charisma, +1 to two others",
            RaceType::Tiefling => "+2 Charisma, +1 Intelligence",
            RaceType::Gnome => "+2 Intelligence",
            RaceType::Dragonborn => "+2 Strength, +1 Charisma",
        }
    }

    pub fn base_speed(&self) -> u32 {
        match self {
            RaceType::Dwarf | RaceType::Halfling | RaceType::Gnome => 25,
            _ => 30,
        }
    }

    pub fn all() -> &'static [RaceType] {
        &[
            RaceType::Human,
            RaceType::Elf,
            RaceType::Dwarf,
            RaceType::Halfling,
            RaceType::HalfOrc,
            RaceType::HalfElf,
            RaceType::Tiefling,
            RaceType::Gnome,
            RaceType::Dragonborn,
        ]
    }
}

impl fmt::Display for RaceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

// ============================================================================
// Backgrounds
// ============================================================================

/// D&D 5e character backgrounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Background {
    Acolyte,
    Charlatan,
    Criminal,
    Entertainer,
    FolkHero,
    GuildArtisan,
    Hermit,
    Noble,
    Outlander,
    Sage,
    Sailor,
    Soldier,
    Urchin,
}

impl Background {
    pub fn name(&self) -> &'static str {
        match self {
            Background::Acolyte => "Acolyte",
            Background::Charlatan => "Charlatan",
            Background::Criminal => "Criminal",
            Background::Entertainer => "Entertainer",
            Background::FolkHero => "Folk Hero",
            Background::GuildArtisan => "Guild Artisan",
            Background::Hermit => "Hermit",
            Background::Noble => "Noble",
            Background::Outlander => "Outlander",
            Background::Sage => "Sage",
            Background::Sailor => "Sailor",
            Background::Soldier => "Soldier",
            Background::Urchin => "Urchin",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Background::Acolyte => "You have spent your life in service to a temple.",
            Background::Charlatan => "You have always had a way with people.",
            Background::Criminal => "You have a history of breaking the law.",
            Background::Entertainer => "You thrive in front of an audience.",
            Background::FolkHero => {
                "You come from a humble background but are destined for greatness."
            }
            Background::GuildArtisan => "You are a member of an artisan's guild.",
            Background::Hermit => "You lived in seclusion for a formative part of your life.",
            Background::Noble => "You understand wealth, power, and privilege.",
            Background::Outlander => "You grew up in the wilds, far from civilization.",
            Background::Sage => "You spent years learning the lore of the multiverse.",
            Background::Sailor => "You sailed on a seagoing vessel for years.",
            Background::Soldier => "You trained as a soldier and served in a military.",
            Background::Urchin => "You grew up on the streets, alone and poor.",
        }
    }

    pub fn skill_proficiencies(&self) -> [Skill; 2] {
        match self {
            Background::Acolyte => [Skill::Insight, Skill::Religion],
            Background::Charlatan => [Skill::Deception, Skill::SleightOfHand],
            Background::Criminal => [Skill::Deception, Skill::Stealth],
            Background::Entertainer => [Skill::Acrobatics, Skill::Performance],
            Background::FolkHero => [Skill::AnimalHandling, Skill::Survival],
            Background::GuildArtisan => [Skill::Insight, Skill::Persuasion],
            Background::Hermit => [Skill::Medicine, Skill::Religion],
            Background::Noble => [Skill::History, Skill::Persuasion],
            Background::Outlander => [Skill::Athletics, Skill::Survival],
            Background::Sage => [Skill::Arcana, Skill::History],
            Background::Sailor => [Skill::Athletics, Skill::Perception],
            Background::Soldier => [Skill::Athletics, Skill::Intimidation],
            Background::Urchin => [Skill::SleightOfHand, Skill::Stealth],
        }
    }

    pub fn all() -> &'static [Background] {
        &[
            Background::Acolyte,
            Background::Charlatan,
            Background::Criminal,
            Background::Entertainer,
            Background::FolkHero,
            Background::GuildArtisan,
            Background::Hermit,
            Background::Noble,
            Background::Outlander,
            Background::Sage,
            Background::Sailor,
            Background::Soldier,
            Background::Urchin,
        ]
    }
}

impl fmt::Display for Background {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

// ============================================================================
// Character
// ============================================================================

/// D&D race (legacy struct for compatibility).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Race {
    pub name: String,
    pub subrace: Option<String>,
    #[serde(default)]
    pub race_type: Option<RaceType>,
}

/// Complete D&D 5e character.
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
    pub class_resources: ClassResources,

    // Spellcasting
    pub spellcasting: Option<SpellcastingData>,

    // Skills & proficiencies
    pub skill_proficiencies: HashMap<Skill, ProficiencyLevel>,
    pub saving_throw_proficiencies: HashSet<Ability>,
    pub languages: Vec<String>,

    // Equipment
    pub inventory: Inventory,
    pub equipment: Equipment,

    // Background and race
    pub race: Race,
    pub race_type: RaceType,
    pub background: Background,
    pub background_name: String, // For display/legacy

    // Player backstory
    pub backstory: Option<String>,
}

impl Character {
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
            class_resources: ClassResources::new(),
            spellcasting: None,
            skill_proficiencies: HashMap::new(),
            saving_throw_proficiencies: HashSet::new(),
            languages: vec!["Common".to_string()],
            inventory: Inventory {
                items: Vec::new(),
                gold: 15.0, // Starting gold
            },
            equipment: Equipment::default(),
            race: Race {
                name: "Human".to_string(),
                subrace: None,
                race_type: Some(RaceType::Human),
            },
            race_type: RaceType::Human,
            background: Background::Soldier,
            background_name: "Soldier".to_string(),
            backstory: None,
        }
    }

    pub fn proficiency_bonus(&self) -> i8 {
        match self.level {
            0 => 2, // Invalid level, but default to minimum
            1..=4 => 2,
            5..=8 => 3,
            9..=12 => 4,
            13..=16 => 5,
            // Level 17+ caps at proficiency bonus 6 (D&D 5e max level is 20)
            _ => 6,
        }
    }

    pub fn initiative_modifier(&self) -> i8 {
        self.ability_scores.modifier(Ability::Dexterity)
    }

    pub fn skill_modifier(&self, skill: Skill) -> i8 {
        let ability_mod = self.ability_scores.modifier(skill.ability());
        let proficiency = self
            .skill_proficiencies
            .get(&skill)
            .copied()
            .unwrap_or(ProficiencyLevel::None);
        ability_mod + proficiency.bonus(self.proficiency_bonus())
    }

    pub fn saving_throw_modifier(&self, ability: Ability) -> i8 {
        let ability_mod = self.ability_scores.modifier(ability);
        if self.saving_throw_proficiencies.contains(&ability) {
            ability_mod + self.proficiency_bonus()
        } else {
            ability_mod
        }
    }

    /// Calculate current AC from equipped armor and shield.
    ///
    /// If equipment is set, AC is calculated from equipped armor.
    /// Otherwise, falls back to the armor_class field for backwards compatibility.
    pub fn current_ac(&self) -> u8 {
        let dex_mod = self.ability_scores.modifier(Ability::Dexterity);

        // Calculate base AC from equipped armor or unarmored
        let base_ac = if let Some(ref armor) = self.equipment.armor {
            match armor.armor_type {
                ArmorType::Light => armor.base_ac as i8 + dex_mod,
                ArmorType::Medium => armor.base_ac as i8 + dex_mod.min(2),
                ArmorType::Heavy => armor.base_ac as i8,
            }
        } else if self.equipment.main_hand.is_some()
            || self.equipment.shield.is_some()
            || self.equipment.off_hand.is_some()
        {
            // Equipment is being used but no armor - unarmored defense
            10 + dex_mod
        } else {
            // No equipment set - use legacy armor_class field
            return self
                .armor_class
                .calculate(self.ability_scores.modifier(Ability::Dexterity));
        };

        // Add shield bonus if equipped
        let shield_bonus: i8 = if self.equipment.shield.is_some() {
            2
        } else {
            0
        };

        (base_ac + shield_bonus).max(1) as u8
    }

    pub fn is_conscious(&self) -> bool {
        self.hit_points.current > 0
    }

    /// Check if the character has a specific condition.
    pub fn has_condition(&self, condition: Condition) -> bool {
        self.conditions
            .iter()
            .any(|c| std::mem::discriminant(&c.condition) == std::mem::discriminant(&condition))
    }

    /// Add a condition if not already present. Returns true if the condition was added.
    pub fn add_condition(&mut self, condition: Condition, source: impl Into<String>) -> bool {
        self.add_condition_with_duration(condition, source, None)
    }

    /// Add a condition with optional duration. Returns true if the condition was added.
    pub fn add_condition_with_duration(
        &mut self,
        condition: Condition,
        source: impl Into<String>,
        duration_rounds: Option<u32>,
    ) -> bool {
        if self.has_condition(condition) {
            false
        } else {
            let mut active = ActiveCondition::new(condition, source);
            if let Some(duration) = duration_rounds {
                active = active.with_duration(duration);
            }
            self.conditions.push(active);
            true
        }
    }

    pub fn passive_perception(&self) -> i8 {
        10 + self.skill_modifier(Skill::Perception)
    }
}

// ============================================================================
// Locations
// ============================================================================

/// A location in the game world.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub id: LocationId,
    pub name: String,
    pub location_type: LocationType,
    pub description: String,
    pub connections: Vec<LocationConnection>,
    pub npcs_present: Vec<CharacterId>,
    pub items: Vec<String>,
}

impl Location {
    pub fn new(name: impl Into<String>, location_type: LocationType) -> Self {
        Self {
            id: LocationId::new(),
            name: name.into(),
            location_type,
            description: String::new(),
            connections: Vec::new(),
            npcs_present: Vec::new(),
            items: Vec::new(),
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LocationType {
    Wilderness,
    Town,
    City,
    Dungeon,
    Building,
    Room,
    Road,
    Cave,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationConnection {
    pub destination_id: LocationId,
    pub destination_name: String,
    pub direction: Option<String>,
    pub travel_time_minutes: u32,
}

// ============================================================================
// NPCs
// ============================================================================

/// An NPC in the game.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NPC {
    pub id: CharacterId,
    pub name: String,
    pub description: String,
    pub personality: String,
    pub occupation: Option<String>,
    pub location_id: Option<LocationId>,
    pub disposition: Disposition,
    pub known_information: Vec<String>,
}

impl NPC {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: CharacterId::new(),
            name: name.into(),
            description: String::new(),
            personality: String::new(),
            occupation: None,
            location_id: None,
            disposition: Disposition::Neutral,
            known_information: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Disposition {
    Hostile,
    Unfriendly,
    Neutral,
    Friendly,
    Helpful,
}

// ============================================================================
// Quests
// ============================================================================

/// A quest or objective.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quest {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub status: QuestStatus,
    pub objectives: Vec<QuestObjective>,
    pub rewards: Vec<String>,
    pub giver: Option<String>,
}

impl Quest {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            description: description.into(),
            status: QuestStatus::Active,
            objectives: Vec::new(),
            rewards: Vec::new(),
            giver: None,
        }
    }

    pub fn is_complete(&self) -> bool {
        !self.objectives.is_empty() && self.objectives.iter().all(|o| o.completed)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuestStatus {
    Active,
    Completed,
    Failed,
    Abandoned,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestObjective {
    pub description: String,
    pub completed: bool,
    pub optional: bool,
}

// ============================================================================
// Combat
// ============================================================================

/// Combat participant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Combatant {
    pub id: CharacterId,
    pub name: String,
    pub initiative: i32,
    pub is_player: bool,
    pub is_ally: bool,
    pub current_hp: i32,
    pub max_hp: i32,
    pub armor_class: u8,
}

/// Combat state tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatState {
    pub active: bool,
    pub round: u32,
    pub turn_index: usize,
    pub combatants: Vec<Combatant>,
}

impl CombatState {
    pub fn new() -> Self {
        Self {
            active: true,
            round: 1,
            turn_index: 0,
            combatants: Vec::new(),
        }
    }

    pub fn add_combatant(&mut self, combatant: Combatant) {
        self.combatants.push(combatant);
        self.combatants
            .sort_by(|a, b| b.initiative.cmp(&a.initiative));
    }

    pub fn current_combatant(&self) -> Option<&Combatant> {
        self.combatants.get(self.turn_index)
    }

    pub fn next_turn(&mut self) {
        self.turn_index += 1;
        if self.turn_index >= self.combatants.len() {
            self.turn_index = 0;
            self.round += 1;
        }
    }

    pub fn end_combat(&mut self) {
        self.active = false;
    }

    /// Update a combatant's HP
    pub fn update_combatant_hp(&mut self, id: CharacterId, new_hp: i32) {
        if let Some(combatant) = self.combatants.iter_mut().find(|c| c.id == id) {
            combatant.current_hp = new_hp;
        }
    }

    /// Get non-player combatants (enemies and allies)
    pub fn get_enemies(&self) -> Vec<&Combatant> {
        self.combatants.iter().filter(|c| !c.is_player).collect()
    }
}

impl Default for CombatState {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Time
// ============================================================================

/// In-game time tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameTime {
    pub year: i32,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
}

impl GameTime {
    pub fn new(year: i32, month: u8, day: u8, hour: u8, minute: u8) -> Self {
        Self {
            year,
            month,
            day,
            hour,
            minute,
        }
    }

    pub fn advance_minutes(&mut self, minutes: u32) {
        let total_minutes = self.minute as u32 + minutes;
        self.minute = (total_minutes % 60) as u8;
        let hours_to_add = total_minutes / 60;
        self.advance_hours(hours_to_add);
    }

    pub fn advance_hours(&mut self, hours: u32) {
        let total_hours = self.hour as u32 + hours;
        self.hour = (total_hours % 24) as u8;
        let days_to_add = total_hours / 24;
        self.advance_days(days_to_add);
    }

    pub fn advance_days(&mut self, days: u32) {
        let total_days = self.day as u32 + days;
        self.day = ((total_days - 1) % 30 + 1) as u8;
        let months_to_add = (total_days - 1) / 30;
        self.advance_months(months_to_add);
    }

    pub fn advance_months(&mut self, months: u32) {
        let total_months = self.month as u32 + months;
        self.month = ((total_months - 1) % 12 + 1) as u8;
        let years_to_add = (total_months - 1) / 12;
        self.year += years_to_add as i32;
    }

    pub fn is_daytime(&self) -> bool {
        self.hour >= 6 && self.hour < 18
    }

    pub fn time_of_day(&self) -> &'static str {
        match self.hour {
            5..=7 => "dawn",
            8..=11 => "morning",
            12..=13 => "midday",
            14..=17 => "afternoon",
            18..=20 => "evening",
            _ => "night",
        }
    }
}

impl Default for GameTime {
    fn default() -> Self {
        Self::new(1492, 3, 1, 10, 0) // Day 1 of the month
    }
}

// ============================================================================
// Game World
// ============================================================================

/// Current game mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum GameMode {
    #[default]
    Exploration,
    Combat,
    Dialogue,
    Rest,
}

/// Entry in the narrative history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeEntry {
    pub content: String,
    pub entry_type: NarrativeType,
    pub game_time: GameTime,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum NarrativeType {
    DmNarration,
    PlayerAction,
    NpcDialogue,
    Combat,
    System,
}

/// The complete game world state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameWorld {
    pub session_id: Uuid,
    pub campaign_name: String,

    // Player character
    pub player_character: Character,

    // NPCs
    pub npcs: HashMap<CharacterId, NPC>,

    // Current state
    pub mode: GameMode,
    pub combat: Option<CombatState>,
    pub current_location: Location,
    pub game_time: GameTime,

    // Locations
    pub known_locations: HashMap<LocationId, Location>,

    // Campaign progress
    pub quests: Vec<Quest>,
    pub narrative_history: Vec<NarrativeEntry>,
}

impl GameWorld {
    pub fn new(campaign_name: impl Into<String>, player_character: Character) -> Self {
        let starting_location = Location::new("Starting Location", LocationType::Town)
            .with_description("A quiet place where your adventure begins.");

        let mut known_locations = HashMap::new();
        known_locations.insert(starting_location.id, starting_location.clone());

        Self {
            session_id: Uuid::new_v4(),
            campaign_name: campaign_name.into(),
            player_character,
            npcs: HashMap::new(),
            mode: GameMode::Exploration,
            combat: None,
            current_location: starting_location,
            game_time: GameTime::default(),
            known_locations,
            quests: Vec::new(),
            narrative_history: Vec::new(),
        }
    }

    pub fn start_combat(&mut self) -> &mut CombatState {
        self.mode = GameMode::Combat;
        self.combat = Some(CombatState::new());
        self.combat.as_mut().unwrap()
    }

    pub fn end_combat(&mut self) {
        self.combat = None;
        self.mode = GameMode::Exploration;
    }

    pub fn short_rest(&mut self) {
        self.game_time.advance_hours(1);

        // Warlocks recover all spell slots on short rest (Pact Magic)
        let is_warlock = self
            .player_character
            .classes
            .iter()
            .any(|c| c.class == CharacterClass::Warlock);
        if is_warlock {
            if let Some(ref mut spellcasting) = self.player_character.spellcasting {
                spellcasting.spell_slots.recover_all();
            }
        }

        // Reset feature uses that recharge on short rest
        for feature in &mut self.player_character.features {
            if let Some(ref mut uses) = feature.uses {
                if matches!(uses.recharge, RechargeType::ShortRest) {
                    uses.current = uses.maximum;
                }
            }
        }

        // Reset class-specific resources
        for class_level in &self.player_character.classes {
            self.player_character
                .class_resources
                .short_rest_recovery(class_level.class, class_level.level);
        }
    }

    pub fn long_rest(&mut self) {
        self.game_time.advance_hours(8);

        // Full HP recovery
        let max_hp = self.player_character.hit_points.maximum;
        self.player_character.hit_points.current = max_hp;

        // Remove Unconscious condition if present (they're now healed)
        self.player_character
            .conditions
            .retain(|c| c.condition != Condition::Unconscious);

        // Reduce exhaustion by 1 level (if any)
        for condition in &mut self.player_character.conditions {
            if let Condition::Exhaustion(level) = &mut condition.condition {
                if *level > 0 {
                    *level -= 1;
                }
            }
        }
        // Remove exhaustion if reduced to 0
        self.player_character
            .conditions
            .retain(|c| !matches!(c.condition, Condition::Exhaustion(0)));

        // Recover half hit dice
        self.player_character.hit_dice.recover_half();

        // Recover spell slots
        if let Some(ref mut spellcasting) = self.player_character.spellcasting {
            spellcasting.spell_slots.recover_all();
        }

        // Reset feature uses (both short rest and long rest features)
        for feature in &mut self.player_character.features {
            if let Some(ref mut uses) = feature.uses {
                if matches!(
                    uses.recharge,
                    RechargeType::LongRest | RechargeType::ShortRest
                ) {
                    uses.current = uses.maximum;
                }
            }
        }

        // Reset class-specific resources
        let classes: Vec<_> = self
            .player_character
            .classes
            .iter()
            .map(|c| (c.class, c.level))
            .collect();
        for (class, level) in classes {
            self.player_character
                .class_resources
                .long_rest_recovery(class, level);
        }
    }

    pub fn add_narrative(&mut self, content: String, entry_type: NarrativeType) {
        self.narrative_history.push(NarrativeEntry {
            content,
            entry_type,
            game_time: self.game_time.clone(),
        });
    }

    pub fn recent_narrative(&self, count: usize) -> Vec<&NarrativeEntry> {
        self.narrative_history.iter().rev().take(count).collect()
    }
}

/// Create a sample fighter character for testing.
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
        base: 16,
        armor_type: Some(ArmorType::Heavy),
        shield_bonus: 2,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ability_modifier() {
        let scores = AbilityScores::new(16, 14, 12, 10, 8, 6);
        assert_eq!(scores.modifier(Ability::Strength), 3);
        assert_eq!(scores.modifier(Ability::Dexterity), 2);
        assert_eq!(scores.modifier(Ability::Constitution), 1);
        assert_eq!(scores.modifier(Ability::Intelligence), 0);
        assert_eq!(scores.modifier(Ability::Wisdom), -1);
        assert_eq!(scores.modifier(Ability::Charisma), -2);

        // Test odd scores below 10 (edge case for floor division)
        let odd_scores = AbilityScores::new(9, 7, 5, 11, 13, 15);
        assert_eq!(odd_scores.modifier(Ability::Strength), -1); // 9 -> -1
        assert_eq!(odd_scores.modifier(Ability::Dexterity), -2); // 7 -> -2
        assert_eq!(odd_scores.modifier(Ability::Constitution), -3); // 5 -> -3
        assert_eq!(odd_scores.modifier(Ability::Intelligence), 0); // 11 -> 0
        assert_eq!(odd_scores.modifier(Ability::Wisdom), 1); // 13 -> +1
        assert_eq!(odd_scores.modifier(Ability::Charisma), 2); // 15 -> +2
    }

    #[test]
    fn test_hit_points() {
        let mut hp = HitPoints::new(20);
        assert_eq!(hp.current, 20);

        hp.take_damage(5);
        assert_eq!(hp.current, 15);

        hp.heal(10);
        assert_eq!(hp.current, 20); // Capped at max

        hp.add_temp_hp(5);
        hp.take_damage(7);
        assert_eq!(hp.temporary, 0);
        assert_eq!(hp.current, 18);
    }

    #[test]
    fn test_character_proficiency() {
        let mut char = Character::new("Test");
        assert_eq!(char.proficiency_bonus(), 2);

        char.level = 5;
        assert_eq!(char.proficiency_bonus(), 3);

        char.level = 17;
        assert_eq!(char.proficiency_bonus(), 6);
    }

    #[test]
    fn test_sample_fighter() {
        let fighter = create_sample_fighter("Roland");
        assert_eq!(fighter.name, "Roland");
        assert_eq!(fighter.level, 3);
        assert_eq!(fighter.current_ac(), 18); // 16 base + 2 shield, no dex for heavy
    }

    #[test]
    fn test_game_world() {
        let character = create_sample_fighter("Test");
        let world = GameWorld::new("Test Campaign", character);
        assert_eq!(world.campaign_name, "Test Campaign");
        assert!(matches!(world.mode, GameMode::Exploration));
    }

    #[test]
    fn test_inventory_add_item() {
        let mut inventory = Inventory::default();
        assert!(inventory.items.is_empty());

        // Weapons don't stack - each is a separate item
        let sword = Item {
            name: "Longsword".to_string(),
            quantity: 1,
            weight: 3.0,
            value_gp: 15.0,
            description: None,
            item_type: ItemType::Weapon,
            magical: false,
        };
        inventory.add_item(sword);

        assert_eq!(inventory.items.len(), 1);
        assert_eq!(inventory.find_item("Longsword").unwrap().quantity, 1);

        // Adding another weapon creates a second entry (weapons don't stack)
        let sword2 = Item {
            name: "Longsword".to_string(),
            quantity: 1,
            weight: 3.0,
            value_gp: 15.0,
            description: None,
            item_type: ItemType::Weapon,
            magical: false,
        };
        inventory.add_item(sword2);

        assert_eq!(inventory.items.len(), 2); // Two separate swords

        // Stackable items (like potions) DO stack
        let potion1 = Item {
            name: "Healing Potion".to_string(),
            quantity: 1,
            weight: 0.5,
            value_gp: 50.0,
            description: None,
            item_type: ItemType::Potion,
            magical: true,
        };
        inventory.add_item(potion1);
        assert_eq!(inventory.items.len(), 3);

        let potion2 = Item {
            name: "Healing Potion".to_string(),
            quantity: 2,
            weight: 0.5,
            value_gp: 50.0,
            description: None,
            item_type: ItemType::Potion,
            magical: true,
        };
        inventory.add_item(potion2);
        assert_eq!(inventory.items.len(), 3); // Still 3 - potions stacked
        assert_eq!(inventory.find_item("Healing Potion").unwrap().quantity, 3);
    }

    #[test]
    fn test_inventory_remove_item() {
        let mut inventory = Inventory::default();
        let potion = Item {
            name: "Healing Potion".to_string(),
            quantity: 3,
            weight: 0.5,
            value_gp: 50.0,
            description: None,
            item_type: ItemType::Potion,
            magical: true,
        };
        inventory.add_item(potion);

        assert!(inventory.remove_item("Healing Potion", 1));
        assert_eq!(inventory.find_item("Healing Potion").unwrap().quantity, 2);

        assert!(inventory.remove_item("Healing Potion", 2));
        assert!(inventory.find_item("Healing Potion").is_none());

        // Can't remove what you don't have
        assert!(!inventory.remove_item("Healing Potion", 1));
    }

    #[test]
    fn test_inventory_gold() {
        let mut inventory = Inventory {
            gold: 100.0,
            ..Default::default()
        };

        assert!(inventory.adjust_gold(50.0).is_ok());
        assert_eq!(inventory.gold, 150.0);

        assert!(inventory.adjust_gold(-100.0).is_ok());
        assert_eq!(inventory.gold, 50.0);

        // Can't go negative
        assert!(inventory.adjust_gold(-100.0).is_err());
        assert_eq!(inventory.gold, 50.0);
    }

    #[test]
    fn test_equipment_ac_calculation() {
        let mut character = Character::new("Test");
        character.ability_scores.dexterity = 16; // +3 DEX mod

        // Unarmored: 10 + DEX
        character.equipment.shield = Some(Item {
            name: "Shield".to_string(),
            quantity: 1,
            weight: 6.0,
            value_gp: 10.0,
            description: None,
            item_type: ItemType::Shield,
            magical: false,
        });
        // With shield but no armor: 10 + 3 + 2 = 15
        assert_eq!(character.current_ac(), 15);

        // Light armor: base + full DEX
        character.equipment.armor = Some(ArmorItem::new("Studded Leather", ArmorType::Light, 12));
        // 12 + 3 + 2 = 17
        assert_eq!(character.current_ac(), 17);

        // Medium armor: base + DEX (max 2)
        character.equipment.armor = Some(ArmorItem::new("Breastplate", ArmorType::Medium, 14));
        // 14 + 2 (capped) + 2 = 18
        assert_eq!(character.current_ac(), 18);

        // Heavy armor: base only
        character.equipment.armor = Some(ArmorItem::new("Plate Armor", ArmorType::Heavy, 18));
        // 18 + 0 + 2 = 20
        assert_eq!(character.current_ac(), 20);

        // Remove shield
        character.equipment.shield = None;
        assert_eq!(character.current_ac(), 18);
    }

    #[test]
    fn test_weapon_item() {
        let sword = WeaponItem::new("Longsword", "1d8", WeaponDamageType::Slashing)
            .with_properties(vec![WeaponProperty::Versatile("1d10".to_string())]);

        assert_eq!(sword.damage_dice, "1d8");
        assert!(!sword.is_finesse());
        assert!(!sword.is_two_handed());
        assert_eq!(sword.versatile_damage(), Some("1d10"));

        let rapier = WeaponItem::new("Rapier", "1d8", WeaponDamageType::Piercing)
            .with_properties(vec![WeaponProperty::Finesse]);
        assert!(rapier.is_finesse());

        // Two-handed weapons
        let greatsword = WeaponItem::new("Greatsword", "2d6", WeaponDamageType::Slashing)
            .with_properties(vec![WeaponProperty::Heavy, WeaponProperty::TwoHanded]);
        assert!(greatsword.is_two_handed());
    }

    #[test]
    fn test_item_stackability() {
        // Weapons don't stack
        let sword = Item {
            name: "Longsword".to_string(),
            quantity: 1,
            weight: 3.0,
            value_gp: 15.0,
            description: None,
            item_type: ItemType::Weapon,
            magical: false,
        };
        assert!(!sword.is_stackable());

        // Armor doesn't stack
        let armor = Item {
            name: "Chain Mail".to_string(),
            quantity: 1,
            weight: 55.0,
            value_gp: 75.0,
            description: None,
            item_type: ItemType::Armor,
            magical: false,
        };
        assert!(!armor.is_stackable());

        // Potions stack
        let potion = Item {
            name: "Healing Potion".to_string(),
            quantity: 1,
            weight: 0.5,
            value_gp: 50.0,
            description: None,
            item_type: ItemType::Potion,
            magical: true,
        };
        assert!(potion.is_stackable());

        // Adventuring gear stacks
        let rope = Item {
            name: "Rope".to_string(),
            quantity: 1,
            weight: 10.0,
            value_gp: 1.0,
            description: None,
            item_type: ItemType::Adventuring,
            magical: false,
        };
        assert!(rope.is_stackable());
    }

    #[test]
    fn test_character_backstory() {
        // New character should have no backstory
        let character = Character::new("Test");
        assert!(character.backstory.is_none());

        // Can set backstory
        let mut character = Character::new("Test");
        character.backstory = Some("A wandering adventurer seeking glory.".to_string());
        assert!(character.backstory.is_some());
        assert_eq!(
            character.backstory.as_ref().unwrap(),
            "A wandering adventurer seeking glory."
        );
    }
}
