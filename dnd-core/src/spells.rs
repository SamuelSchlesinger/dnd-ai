//! Spell database and spellcasting mechanics.
//!
//! Contains SRD 5.2 spell definitions and lookup functions.

use crate::rules::DamageType;
use crate::world::Ability;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::LazyLock;

/// Schools of magic in D&D.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpellSchool {
    Abjuration,
    Conjuration,
    Divination,
    Enchantment,
    Evocation,
    Illusion,
    Necromancy,
    Transmutation,
}

impl SpellSchool {
    pub fn name(&self) -> &'static str {
        match self {
            SpellSchool::Abjuration => "Abjuration",
            SpellSchool::Conjuration => "Conjuration",
            SpellSchool::Divination => "Divination",
            SpellSchool::Enchantment => "Enchantment",
            SpellSchool::Evocation => "Evocation",
            SpellSchool::Illusion => "Illusion",
            SpellSchool::Necromancy => "Necromancy",
            SpellSchool::Transmutation => "Transmutation",
        }
    }
}

/// How long it takes to cast a spell.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CastingTime {
    Action,
    BonusAction,
    Reaction(String), // Trigger condition
    Minutes(u32),
    Hours(u32),
}

impl CastingTime {
    pub fn description(&self) -> String {
        match self {
            CastingTime::Action => "1 action".to_string(),
            CastingTime::BonusAction => "1 bonus action".to_string(),
            CastingTime::Reaction(trigger) => format!("1 reaction, {}", trigger),
            CastingTime::Minutes(m) => format!("{} minute{}", m, if *m == 1 { "" } else { "s" }),
            CastingTime::Hours(h) => format!("{} hour{}", h, if *h == 1 { "" } else { "s" }),
        }
    }
}

/// Range of a spell.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpellRange {
    Self_,
    Touch,
    Feet(u32),
    Miles(u32),
    Sight,
    Unlimited,
    SelfRadius(u32),    // Self with radius (e.g., Thunderwave)
    SelfCone(u32),      // Self with cone (e.g., Burning Hands)
    SelfLine(u32, u32), // Self with line (length, width)
}

impl SpellRange {
    pub fn description(&self) -> String {
        match self {
            SpellRange::Self_ => "Self".to_string(),
            SpellRange::Touch => "Touch".to_string(),
            SpellRange::Feet(f) => format!("{} feet", f),
            SpellRange::Miles(m) => format!("{} mile{}", m, if *m == 1 { "" } else { "s" }),
            SpellRange::Sight => "Sight".to_string(),
            SpellRange::Unlimited => "Unlimited".to_string(),
            SpellRange::SelfRadius(r) => format!("Self ({}-foot radius)", r),
            SpellRange::SelfCone(c) => format!("Self ({}-foot cone)", c),
            SpellRange::SelfLine(l, w) => format!("Self ({}-foot line, {} feet wide)", l, w),
        }
    }
}

/// Spell components required.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Components {
    pub verbal: bool,
    pub somatic: bool,
    pub material: Option<String>,
}

impl Components {
    pub fn v() -> Self {
        Self {
            verbal: true,
            somatic: false,
            material: None,
        }
    }

    pub fn vs() -> Self {
        Self {
            verbal: true,
            somatic: true,
            material: None,
        }
    }

    pub fn vsm(material: &str) -> Self {
        Self {
            verbal: true,
            somatic: true,
            material: Some(material.to_string()),
        }
    }

    pub fn s() -> Self {
        Self {
            verbal: false,
            somatic: true,
            material: None,
        }
    }

    pub fn sm(material: &str) -> Self {
        Self {
            verbal: false,
            somatic: true,
            material: Some(material.to_string()),
        }
    }

    pub fn description(&self) -> String {
        let mut parts = Vec::new();
        if self.verbal {
            parts.push("V");
        }
        if self.somatic {
            parts.push("S");
        }
        if self.material.is_some() {
            parts.push("M");
        }
        let base = parts.join(", ");
        if let Some(ref mat) = self.material {
            format!("{} ({})", base, mat)
        } else {
            base
        }
    }
}

/// Duration of a spell.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpellDuration {
    Instantaneous,
    Rounds(u32),
    Minutes(u32),
    Hours(u32),
    Days(u32),
    UntilDispelled,
    Special,
}

impl SpellDuration {
    pub fn description(&self) -> String {
        match self {
            SpellDuration::Instantaneous => "Instantaneous".to_string(),
            SpellDuration::Rounds(r) => format!("{} round{}", r, if *r == 1 { "" } else { "s" }),
            SpellDuration::Minutes(m) => format!("{} minute{}", m, if *m == 1 { "" } else { "s" }),
            SpellDuration::Hours(h) => format!("{} hour{}", h, if *h == 1 { "" } else { "s" }),
            SpellDuration::Days(d) => format!("{} day{}", d, if *d == 1 { "" } else { "s" }),
            SpellDuration::UntilDispelled => "Until dispelled".to_string(),
            SpellDuration::Special => "Special".to_string(),
        }
    }
}

/// Type of spell attack (if any).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpellAttackType {
    Melee,
    Ranged,
}

/// Area of effect shape.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AreaOfEffect {
    None,
    Sphere(u32),        // Radius in feet
    Cube(u32),          // Side length in feet
    Cone(u32),          // Length in feet
    Line(u32, u32),     // Length, width in feet
    Cylinder(u32, u32), // Radius, height in feet
}

/// How spell damage scales with level.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DamageScaling {
    /// Cantrip scaling: increases at levels 5, 11, 17
    CantripScaling,
    /// Scales with upcast level (e.g., +1d6 per level above base)
    PerSlotLevel { extra_dice: String },
    /// No scaling
    None,
}

/// Complete spell definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpellData {
    pub name: String,
    pub level: u8, // 0 for cantrips
    pub school: SpellSchool,
    pub casting_time: CastingTime,
    pub range: SpellRange,
    pub components: Components,
    pub duration: SpellDuration,
    pub concentration: bool,
    pub ritual: bool,
    pub description: String,

    // Combat mechanics
    pub damage_dice: Option<String>,
    pub damage_type: Option<DamageType>,
    pub damage_scaling: DamageScaling,
    pub healing_dice: Option<String>,
    pub save_type: Option<Ability>,
    pub save_effect: Option<String>, // What happens on save (e.g., "half damage")
    pub attack_type: Option<SpellAttackType>,
    pub area_of_effect: AreaOfEffect,

    // Class lists (simplified - which classes can learn this spell)
    pub classes: Vec<SpellClass>,
}

/// Classes that can learn spells.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpellClass {
    Bard,
    Cleric,
    Druid,
    Paladin,
    Ranger,
    Sorcerer,
    Warlock,
    Wizard,
}

impl SpellData {
    /// Check if this is a cantrip.
    pub fn is_cantrip(&self) -> bool {
        self.level == 0
    }

    /// Get the number of damage dice based on caster level (for cantrips).
    pub fn cantrip_dice_count(&self, caster_level: u8) -> u8 {
        match caster_level {
            1..=4 => 1,
            5..=10 => 2,
            11..=16 => 3,
            _ => 4,
        }
    }

    /// Calculate damage dice for a given caster level and slot level.
    pub fn effective_damage_dice(&self, caster_level: u8, slot_level: u8) -> Option<String> {
        let base_dice = self.damage_dice.as_ref()?;

        match &self.damage_scaling {
            DamageScaling::CantripScaling => {
                // Parse base dice (e.g., "1d10") and multiply
                let count = self.cantrip_dice_count(caster_level);
                if let Some(d_pos) = base_dice.find('d') {
                    let die_type = &base_dice[d_pos..];
                    Some(format!("{}{}", count, die_type))
                } else {
                    Some(base_dice.clone())
                }
            }
            DamageScaling::PerSlotLevel { extra_dice } => {
                if slot_level > self.level {
                    let extra_levels = slot_level - self.level;
                    // Parse extra dice (e.g., "1d6") and multiply
                    if let Some(d_pos) = extra_dice.find('d') {
                        let num: u8 = extra_dice[..d_pos].parse().unwrap_or(1);
                        let die_type = &extra_dice[d_pos..];
                        let total_extra = num * extra_levels;
                        Some(format!("{} + {}{}", base_dice, total_extra, die_type))
                    } else {
                        Some(base_dice.clone())
                    }
                } else {
                    Some(base_dice.clone())
                }
            }
            DamageScaling::None => Some(base_dice.clone()),
        }
    }
}

// ============================================================================
// Spell Database
// ============================================================================

/// Global spell database.
static SPELL_DATABASE: LazyLock<HashMap<String, SpellData>> = LazyLock::new(build_spell_database);

/// Look up a spell by name (case-insensitive).
pub fn get_spell(name: &str) -> Option<&'static SpellData> {
    SPELL_DATABASE.get(&name.to_lowercase())
}

/// Get all spells in the database.
pub fn all_spells() -> impl Iterator<Item = &'static SpellData> {
    SPELL_DATABASE.values()
}

/// Get all spells of a specific level.
pub fn spells_by_level(level: u8) -> impl Iterator<Item = &'static SpellData> {
    SPELL_DATABASE.values().filter(move |s| s.level == level)
}

/// Get all spells available to a class.
pub fn spells_for_class(class: SpellClass) -> impl Iterator<Item = &'static SpellData> {
    SPELL_DATABASE
        .values()
        .filter(move |s| s.classes.contains(&class))
}

fn build_spell_database() -> HashMap<String, SpellData> {
    let mut db = HashMap::new();

    // ========================================================================
    // CANTRIPS (Level 0)
    // ========================================================================

    db.insert("fire bolt".to_string(), SpellData {
        name: "Fire Bolt".to_string(),
        level: 0,
        school: SpellSchool::Evocation,
        casting_time: CastingTime::Action,
        range: SpellRange::Feet(120),
        components: Components::vs(),
        duration: SpellDuration::Instantaneous,
        concentration: false,
        ritual: false,
        description: "You hurl a mote of fire at a creature or object within range. Make a ranged spell attack. On a hit, the target takes 1d10 fire damage. A flammable object hit by this spell ignites if it isn't being worn or carried.".to_string(),
        damage_dice: Some("1d10".to_string()),
        damage_type: Some(DamageType::Fire),
        damage_scaling: DamageScaling::CantripScaling,
        healing_dice: None,
        save_type: None,
        save_effect: None,
        attack_type: Some(SpellAttackType::Ranged),
        area_of_effect: AreaOfEffect::None,
        classes: vec![SpellClass::Sorcerer, SpellClass::Wizard],
    });

    db.insert("eldritch blast".to_string(), SpellData {
        name: "Eldritch Blast".to_string(),
        level: 0,
        school: SpellSchool::Evocation,
        casting_time: CastingTime::Action,
        range: SpellRange::Feet(120),
        components: Components::vs(),
        duration: SpellDuration::Instantaneous,
        concentration: false,
        ritual: false,
        description: "A beam of crackling energy streaks toward a creature within range. Make a ranged spell attack. On a hit, the target takes 1d10 force damage. The spell creates more beams at higher levels: two beams at 5th level, three at 11th, and four at 17th.".to_string(),
        damage_dice: Some("1d10".to_string()),
        damage_type: Some(DamageType::Force),
        damage_scaling: DamageScaling::CantripScaling,
        healing_dice: None,
        save_type: None,
        save_effect: None,
        attack_type: Some(SpellAttackType::Ranged),
        area_of_effect: AreaOfEffect::None,
        classes: vec![SpellClass::Warlock],
    });

    db.insert("sacred flame".to_string(), SpellData {
        name: "Sacred Flame".to_string(),
        level: 0,
        school: SpellSchool::Evocation,
        casting_time: CastingTime::Action,
        range: SpellRange::Feet(60),
        components: Components::vs(),
        duration: SpellDuration::Instantaneous,
        concentration: false,
        ritual: false,
        description: "Flame-like radiance descends on a creature that you can see within range. The target must succeed on a Dexterity saving throw or take 1d8 radiant damage. The target gains no benefit from cover for this saving throw.".to_string(),
        damage_dice: Some("1d8".to_string()),
        damage_type: Some(DamageType::Radiant),
        damage_scaling: DamageScaling::CantripScaling,
        healing_dice: None,
        save_type: Some(Ability::Dexterity),
        save_effect: Some("no damage".to_string()),
        attack_type: None,
        area_of_effect: AreaOfEffect::None,
        classes: vec![SpellClass::Cleric],
    });

    db.insert("ray of frost".to_string(), SpellData {
        name: "Ray of Frost".to_string(),
        level: 0,
        school: SpellSchool::Evocation,
        casting_time: CastingTime::Action,
        range: SpellRange::Feet(60),
        components: Components::vs(),
        duration: SpellDuration::Instantaneous,
        concentration: false,
        ritual: false,
        description: "A frigid beam of blue-white light streaks toward a creature within range. Make a ranged spell attack. On a hit, it takes 1d8 cold damage, and its speed is reduced by 10 feet until the start of your next turn.".to_string(),
        damage_dice: Some("1d8".to_string()),
        damage_type: Some(DamageType::Cold),
        damage_scaling: DamageScaling::CantripScaling,
        healing_dice: None,
        save_type: None,
        save_effect: None,
        attack_type: Some(SpellAttackType::Ranged),
        area_of_effect: AreaOfEffect::None,
        classes: vec![SpellClass::Sorcerer, SpellClass::Wizard],
    });

    db.insert("chill touch".to_string(), SpellData {
        name: "Chill Touch".to_string(),
        level: 0,
        school: SpellSchool::Necromancy,
        casting_time: CastingTime::Action,
        range: SpellRange::Feet(120),
        components: Components::vs(),
        duration: SpellDuration::Rounds(1),
        concentration: false,
        ritual: false,
        description: "You create a ghostly, skeletal hand in the space of a creature within range. Make a ranged spell attack. On a hit, the target takes 1d8 necrotic damage, and it can't regain hit points until the start of your next turn. If you hit an undead target, it also has disadvantage on attack rolls against you until the end of your next turn.".to_string(),
        damage_dice: Some("1d8".to_string()),
        damage_type: Some(DamageType::Necrotic),
        damage_scaling: DamageScaling::CantripScaling,
        healing_dice: None,
        save_type: None,
        save_effect: None,
        attack_type: Some(SpellAttackType::Ranged),
        area_of_effect: AreaOfEffect::None,
        classes: vec![SpellClass::Sorcerer, SpellClass::Warlock, SpellClass::Wizard],
    });

    db.insert("light".to_string(), SpellData {
        name: "Light".to_string(),
        level: 0,
        school: SpellSchool::Evocation,
        casting_time: CastingTime::Action,
        range: SpellRange::Touch,
        components: Components::vsm("a firefly or phosphorescent moss"),
        duration: SpellDuration::Hours(1),
        concentration: false,
        ritual: false,
        description: "You touch one object that is no larger than 10 feet in any dimension. Until the spell ends, the object sheds bright light in a 20-foot radius and dim light for an additional 20 feet.".to_string(),
        damage_dice: None,
        damage_type: None,
        damage_scaling: DamageScaling::None,
        healing_dice: None,
        save_type: None,
        save_effect: None,
        attack_type: None,
        area_of_effect: AreaOfEffect::None,
        classes: vec![SpellClass::Bard, SpellClass::Cleric, SpellClass::Sorcerer, SpellClass::Wizard],
    });

    db.insert("mage hand".to_string(), SpellData {
        name: "Mage Hand".to_string(),
        level: 0,
        school: SpellSchool::Conjuration,
        casting_time: CastingTime::Action,
        range: SpellRange::Feet(30),
        components: Components::vs(),
        duration: SpellDuration::Minutes(1),
        concentration: false,
        ritual: false,
        description: "A spectral, floating hand appears at a point you choose within range. The hand can manipulate objects, open doors, stow or retrieve items, or pour out contents. It can't attack, activate magic items, or carry more than 10 pounds.".to_string(),
        damage_dice: None,
        damage_type: None,
        damage_scaling: DamageScaling::None,
        healing_dice: None,
        save_type: None,
        save_effect: None,
        attack_type: None,
        area_of_effect: AreaOfEffect::None,
        classes: vec![SpellClass::Bard, SpellClass::Sorcerer, SpellClass::Warlock, SpellClass::Wizard],
    });

    db.insert("prestidigitation".to_string(), SpellData {
        name: "Prestidigitation".to_string(),
        level: 0,
        school: SpellSchool::Transmutation,
        casting_time: CastingTime::Action,
        range: SpellRange::Feet(10),
        components: Components::vs(),
        duration: SpellDuration::Hours(1),
        concentration: false,
        ritual: false,
        description: "This spell is a minor magical trick that novice spellcasters use for practice. You create one of several minor effects: sensory effect, light/snuff candle, clean/soil object, chill/warm/flavor material, make symbol or mark, create trinket or illusory image.".to_string(),
        damage_dice: None,
        damage_type: None,
        damage_scaling: DamageScaling::None,
        healing_dice: None,
        save_type: None,
        save_effect: None,
        attack_type: None,
        area_of_effect: AreaOfEffect::None,
        classes: vec![SpellClass::Bard, SpellClass::Sorcerer, SpellClass::Warlock, SpellClass::Wizard],
    });

    // ========================================================================
    // 1ST LEVEL SPELLS
    // ========================================================================

    db.insert("magic missile".to_string(), SpellData {
        name: "Magic Missile".to_string(),
        level: 1,
        school: SpellSchool::Evocation,
        casting_time: CastingTime::Action,
        range: SpellRange::Feet(120),
        components: Components::vs(),
        duration: SpellDuration::Instantaneous,
        concentration: false,
        ritual: false,
        description: "You create three glowing darts of magical force. Each dart hits a creature of your choice that you can see within range. A dart deals 1d4+1 force damage. The darts all strike simultaneously. When cast with a higher level slot, create one additional dart per slot level above 1st.".to_string(),
        damage_dice: Some("3d4+3".to_string()), // 3 darts at 1d4+1 each
        damage_type: Some(DamageType::Force),
        damage_scaling: DamageScaling::PerSlotLevel { extra_dice: "1d4+1".to_string() },
        healing_dice: None,
        save_type: None,
        save_effect: None,
        attack_type: None, // Auto-hit
        area_of_effect: AreaOfEffect::None,
        classes: vec![SpellClass::Sorcerer, SpellClass::Wizard],
    });

    db.insert("cure wounds".to_string(), SpellData {
        name: "Cure Wounds".to_string(),
        level: 1,
        school: SpellSchool::Evocation,
        casting_time: CastingTime::Action,
        range: SpellRange::Touch,
        components: Components::vs(),
        duration: SpellDuration::Instantaneous,
        concentration: false,
        ritual: false,
        description: "A creature you touch regains hit points equal to 1d8 + your spellcasting ability modifier. This spell has no effect on undead or constructs. When cast with a higher level slot, healing increases by 1d8 per slot level above 1st.".to_string(),
        damage_dice: None,
        damage_type: None,
        damage_scaling: DamageScaling::PerSlotLevel { extra_dice: "1d8".to_string() },
        healing_dice: Some("1d8".to_string()),
        save_type: None,
        save_effect: None,
        attack_type: None,
        area_of_effect: AreaOfEffect::None,
        classes: vec![SpellClass::Bard, SpellClass::Cleric, SpellClass::Druid, SpellClass::Paladin, SpellClass::Ranger],
    });

    db.insert("shield".to_string(), SpellData {
        name: "Shield".to_string(),
        level: 1,
        school: SpellSchool::Abjuration,
        casting_time: CastingTime::Reaction("which you take when you are hit by an attack or targeted by magic missile".to_string()),
        range: SpellRange::Self_,
        components: Components::vs(),
        duration: SpellDuration::Rounds(1),
        concentration: false,
        ritual: false,
        description: "An invisible barrier of magical force appears and protects you. Until the start of your next turn, you have a +5 bonus to AC, including against the triggering attack, and you take no damage from magic missile.".to_string(),
        damage_dice: None,
        damage_type: None,
        damage_scaling: DamageScaling::None,
        healing_dice: None,
        save_type: None,
        save_effect: None,
        attack_type: None,
        area_of_effect: AreaOfEffect::None,
        classes: vec![SpellClass::Sorcerer, SpellClass::Wizard],
    });

    db.insert("burning hands".to_string(), SpellData {
        name: "Burning Hands".to_string(),
        level: 1,
        school: SpellSchool::Evocation,
        casting_time: CastingTime::Action,
        range: SpellRange::SelfCone(15),
        components: Components::vs(),
        duration: SpellDuration::Instantaneous,
        concentration: false,
        ritual: false,
        description: "As you hold your hands with thumbs touching and fingers spread, a thin sheet of flames shoots forth. Each creature in a 15-foot cone must make a Dexterity saving throw. A creature takes 3d6 fire damage on a failed save, or half as much on a successful one.".to_string(),
        damage_dice: Some("3d6".to_string()),
        damage_type: Some(DamageType::Fire),
        damage_scaling: DamageScaling::PerSlotLevel { extra_dice: "1d6".to_string() },
        healing_dice: None,
        save_type: Some(Ability::Dexterity),
        save_effect: Some("half damage".to_string()),
        attack_type: None,
        area_of_effect: AreaOfEffect::Cone(15),
        classes: vec![SpellClass::Sorcerer, SpellClass::Wizard],
    });

    db.insert("thunderwave".to_string(), SpellData {
        name: "Thunderwave".to_string(),
        level: 1,
        school: SpellSchool::Evocation,
        casting_time: CastingTime::Action,
        range: SpellRange::SelfRadius(15),
        components: Components::vs(),
        duration: SpellDuration::Instantaneous,
        concentration: false,
        ritual: false,
        description: "A wave of thunderous force sweeps out from you. Each creature in a 15-foot cube originating from you must make a Constitution saving throw. On a failed save, a creature takes 2d8 thunder damage and is pushed 10 feet away. On a success, it takes half damage and isn't pushed.".to_string(),
        damage_dice: Some("2d8".to_string()),
        damage_type: Some(DamageType::Thunder),
        damage_scaling: DamageScaling::PerSlotLevel { extra_dice: "1d8".to_string() },
        healing_dice: None,
        save_type: Some(Ability::Constitution),
        save_effect: Some("half damage, not pushed".to_string()),
        attack_type: None,
        area_of_effect: AreaOfEffect::Cube(15),
        classes: vec![SpellClass::Bard, SpellClass::Druid, SpellClass::Sorcerer, SpellClass::Wizard],
    });

    db.insert("healing word".to_string(), SpellData {
        name: "Healing Word".to_string(),
        level: 1,
        school: SpellSchool::Evocation,
        casting_time: CastingTime::BonusAction,
        range: SpellRange::Feet(60),
        components: Components::v(),
        duration: SpellDuration::Instantaneous,
        concentration: false,
        ritual: false,
        description: "A creature of your choice that you can see within range regains hit points equal to 1d4 + your spellcasting ability modifier. This spell has no effect on undead or constructs.".to_string(),
        damage_dice: None,
        damage_type: None,
        damage_scaling: DamageScaling::PerSlotLevel { extra_dice: "1d4".to_string() },
        healing_dice: Some("1d4".to_string()),
        save_type: None,
        save_effect: None,
        attack_type: None,
        area_of_effect: AreaOfEffect::None,
        classes: vec![SpellClass::Bard, SpellClass::Cleric, SpellClass::Druid],
    });

    db.insert("guiding bolt".to_string(), SpellData {
        name: "Guiding Bolt".to_string(),
        level: 1,
        school: SpellSchool::Evocation,
        casting_time: CastingTime::Action,
        range: SpellRange::Feet(120),
        components: Components::vs(),
        duration: SpellDuration::Rounds(1),
        concentration: false,
        ritual: false,
        description: "A flash of light streaks toward a creature of your choice within range. Make a ranged spell attack. On a hit, the target takes 4d6 radiant damage, and the next attack roll made against this target before the end of your next turn has advantage.".to_string(),
        damage_dice: Some("4d6".to_string()),
        damage_type: Some(DamageType::Radiant),
        damage_scaling: DamageScaling::PerSlotLevel { extra_dice: "1d6".to_string() },
        healing_dice: None,
        save_type: None,
        save_effect: None,
        attack_type: Some(SpellAttackType::Ranged),
        area_of_effect: AreaOfEffect::None,
        classes: vec![SpellClass::Cleric],
    });

    db.insert("mage armor".to_string(), SpellData {
        name: "Mage Armor".to_string(),
        level: 1,
        school: SpellSchool::Abjuration,
        casting_time: CastingTime::Action,
        range: SpellRange::Touch,
        components: Components::vsm("a piece of cured leather"),
        duration: SpellDuration::Hours(8),
        concentration: false,
        ritual: false,
        description: "You touch a willing creature who isn't wearing armor, and a protective magical force surrounds it until the spell ends. The target's base AC becomes 13 + its Dexterity modifier. The spell ends if the target dons armor or if you dismiss the spell as an action.".to_string(),
        damage_dice: None,
        damage_type: None,
        damage_scaling: DamageScaling::None,
        healing_dice: None,
        save_type: None,
        save_effect: None,
        attack_type: None,
        area_of_effect: AreaOfEffect::None,
        classes: vec![SpellClass::Sorcerer, SpellClass::Wizard],
    });

    db.insert("sleep".to_string(), SpellData {
        name: "Sleep".to_string(),
        level: 1,
        school: SpellSchool::Enchantment,
        casting_time: CastingTime::Action,
        range: SpellRange::Feet(90),
        components: Components::vsm("a pinch of fine sand, rose petals, or a cricket"),
        duration: SpellDuration::Minutes(1),
        concentration: false,
        ritual: false,
        description: "This spell sends creatures into a magical slumber. Roll 5d8; the total is how many hit points of creatures this spell can affect. Starting with the creature with the lowest current HP, each creature falls unconscious until the spell ends, it takes damage, or someone uses an action to wake it.".to_string(),
        damage_dice: None,
        damage_type: None,
        damage_scaling: DamageScaling::PerSlotLevel { extra_dice: "2d8".to_string() },
        healing_dice: None,
        save_type: None,
        save_effect: None,
        attack_type: None,
        area_of_effect: AreaOfEffect::Sphere(20),
        classes: vec![SpellClass::Bard, SpellClass::Sorcerer, SpellClass::Wizard],
    });

    db.insert("hex".to_string(), SpellData {
        name: "Hex".to_string(),
        level: 1,
        school: SpellSchool::Enchantment,
        casting_time: CastingTime::BonusAction,
        range: SpellRange::Feet(90),
        components: Components::vsm("the petrified eye of a newt"),
        duration: SpellDuration::Hours(1),
        concentration: true,
        ritual: false,
        description: "You place a curse on a creature that you can see within range. Until the spell ends, you deal an extra 1d6 necrotic damage to the target whenever you hit it with an attack. Also, choose one ability when you cast the spell. The target has disadvantage on ability checks made with the chosen ability. If the target drops to 0 HP before this spell ends, you can use a bonus action on a subsequent turn to curse a new creature.".to_string(),
        damage_dice: Some("1d6".to_string()),
        damage_type: Some(DamageType::Necrotic),
        damage_scaling: DamageScaling::None,
        healing_dice: None,
        save_type: None,
        save_effect: None,
        attack_type: None,
        area_of_effect: AreaOfEffect::None,
        classes: vec![SpellClass::Warlock],
    });

    db.insert("hellish rebuke".to_string(), SpellData {
        name: "Hellish Rebuke".to_string(),
        level: 1,
        school: SpellSchool::Evocation,
        casting_time: CastingTime::Reaction("which you take in response to being damaged by a creature within 60 feet of you that you can see".to_string()),
        range: SpellRange::Feet(60),
        components: Components::vs(),
        duration: SpellDuration::Instantaneous,
        concentration: false,
        ritual: false,
        description: "You point your finger, and the creature that damaged you is momentarily surrounded by hellish flames. The creature must make a Dexterity saving throw. It takes 2d10 fire damage on a failed save, or half as much damage on a successful one. When cast with a higher level slot, the damage increases by 1d10 per slot level above 1st.".to_string(),
        damage_dice: Some("2d10".to_string()),
        damage_type: Some(DamageType::Fire),
        damage_scaling: DamageScaling::PerSlotLevel { extra_dice: "1d10".to_string() },
        healing_dice: None,
        save_type: Some(Ability::Dexterity),
        save_effect: Some("half damage".to_string()),
        attack_type: None,
        area_of_effect: AreaOfEffect::None,
        classes: vec![SpellClass::Warlock],
    });

    db.insert("armor of agathys".to_string(), SpellData {
        name: "Armor of Agathys".to_string(),
        level: 1,
        school: SpellSchool::Abjuration,
        casting_time: CastingTime::Action,
        range: SpellRange::Self_,
        components: Components::vsm("a cup of water"),
        duration: SpellDuration::Hours(1),
        concentration: false,
        ritual: false,
        description: "A protective magical force surrounds you, manifesting as a spectral frost that covers you and your gear. You gain 5 temporary hit points for the duration. If a creature hits you with a melee attack while you have these hit points, the creature takes 5 cold damage. When cast with a higher level slot, both the temporary hit points and cold damage increase by 5 per slot level above 1st.".to_string(),
        damage_dice: Some("5".to_string()), // Flat damage, not dice
        damage_type: Some(DamageType::Cold),
        damage_scaling: DamageScaling::PerSlotLevel { extra_dice: "5".to_string() },
        healing_dice: None,
        save_type: None,
        save_effect: None,
        attack_type: None,
        area_of_effect: AreaOfEffect::None,
        classes: vec![SpellClass::Warlock],
    });

    db.insert("charm person".to_string(), SpellData {
        name: "Charm Person".to_string(),
        level: 1,
        school: SpellSchool::Enchantment,
        casting_time: CastingTime::Action,
        range: SpellRange::Feet(30),
        components: Components::vs(),
        duration: SpellDuration::Hours(1),
        concentration: false,
        ritual: false,
        description: "You attempt to charm a humanoid you can see within range. It must make a Wisdom saving throw, with advantage if you or your companions are fighting it. If it fails, it is charmed by you until the spell ends or until you or your companions do anything harmful to it. The charmed creature regards you as a friendly acquaintance.".to_string(),
        damage_dice: None,
        damage_type: None,
        damage_scaling: DamageScaling::None,
        healing_dice: None,
        save_type: Some(Ability::Wisdom),
        save_effect: Some("not charmed".to_string()),
        attack_type: None,
        area_of_effect: AreaOfEffect::None,
        classes: vec![SpellClass::Bard, SpellClass::Druid, SpellClass::Sorcerer, SpellClass::Warlock, SpellClass::Wizard],
    });

    // ========================================================================
    // 2ND LEVEL SPELLS
    // ========================================================================

    db.insert("scorching ray".to_string(), SpellData {
        name: "Scorching Ray".to_string(),
        level: 2,
        school: SpellSchool::Evocation,
        casting_time: CastingTime::Action,
        range: SpellRange::Feet(120),
        components: Components::vs(),
        duration: SpellDuration::Instantaneous,
        concentration: false,
        ritual: false,
        description: "You create three rays of fire and hurl them at targets within range. You can hurl them at one target or several. Make a ranged spell attack for each ray. On a hit, the target takes 2d6 fire damage. When cast with a higher level slot, you create one additional ray per slot level above 2nd.".to_string(),
        damage_dice: Some("2d6".to_string()), // Per ray
        damage_type: Some(DamageType::Fire),
        damage_scaling: DamageScaling::PerSlotLevel { extra_dice: "2d6".to_string() }, // Additional ray
        healing_dice: None,
        save_type: None,
        save_effect: None,
        attack_type: Some(SpellAttackType::Ranged),
        area_of_effect: AreaOfEffect::None,
        classes: vec![SpellClass::Sorcerer, SpellClass::Wizard],
    });

    db.insert("hold person".to_string(), SpellData {
        name: "Hold Person".to_string(),
        level: 2,
        school: SpellSchool::Enchantment,
        casting_time: CastingTime::Action,
        range: SpellRange::Feet(60),
        components: Components::vsm("a small, straight piece of iron"),
        duration: SpellDuration::Minutes(1),
        concentration: true,
        ritual: false,
        description: "Choose a humanoid that you can see within range. The target must succeed on a Wisdom saving throw or be paralyzed for the duration. At the end of each of its turns, the target can make another Wisdom saving throw. On a success, the spell ends on the target.".to_string(),
        damage_dice: None,
        damage_type: None,
        damage_scaling: DamageScaling::None,
        healing_dice: None,
        save_type: Some(Ability::Wisdom),
        save_effect: Some("not paralyzed".to_string()),
        attack_type: None,
        area_of_effect: AreaOfEffect::None,
        classes: vec![SpellClass::Bard, SpellClass::Cleric, SpellClass::Druid, SpellClass::Sorcerer, SpellClass::Warlock, SpellClass::Wizard],
    });

    db.insert("misty step".to_string(), SpellData {
        name: "Misty Step".to_string(),
        level: 2,
        school: SpellSchool::Conjuration,
        casting_time: CastingTime::BonusAction,
        range: SpellRange::Self_,
        components: Components::v(),
        duration: SpellDuration::Instantaneous,
        concentration: false,
        ritual: false,
        description: "Briefly surrounded by silvery mist, you teleport up to 30 feet to an unoccupied space that you can see.".to_string(),
        damage_dice: None,
        damage_type: None,
        damage_scaling: DamageScaling::None,
        healing_dice: None,
        save_type: None,
        save_effect: None,
        attack_type: None,
        area_of_effect: AreaOfEffect::None,
        classes: vec![SpellClass::Sorcerer, SpellClass::Warlock, SpellClass::Wizard],
    });

    db.insert("spiritual weapon".to_string(), SpellData {
        name: "Spiritual Weapon".to_string(),
        level: 2,
        school: SpellSchool::Evocation,
        casting_time: CastingTime::BonusAction,
        range: SpellRange::Feet(60),
        components: Components::vs(),
        duration: SpellDuration::Minutes(1),
        concentration: false,
        ritual: false,
        description: "You create a floating, spectral weapon within range that lasts for the duration. When you cast the spell, you can make a melee spell attack against a creature within 5 feet of the weapon. On a hit, the target takes 1d8 + your spellcasting ability modifier force damage. As a bonus action on your turn, you can move the weapon up to 20 feet and repeat the attack.".to_string(),
        damage_dice: Some("1d8".to_string()),
        damage_type: Some(DamageType::Force),
        damage_scaling: DamageScaling::PerSlotLevel { extra_dice: "1d8".to_string() }, // Per 2 slot levels
        healing_dice: None,
        save_type: None,
        save_effect: None,
        attack_type: Some(SpellAttackType::Melee),
        area_of_effect: AreaOfEffect::None,
        classes: vec![SpellClass::Cleric],
    });

    // ========================================================================
    // 3RD LEVEL SPELLS
    // ========================================================================

    db.insert("fireball".to_string(), SpellData {
        name: "Fireball".to_string(),
        level: 3,
        school: SpellSchool::Evocation,
        casting_time: CastingTime::Action,
        range: SpellRange::Feet(150),
        components: Components::vsm("a tiny ball of bat guano and sulfur"),
        duration: SpellDuration::Instantaneous,
        concentration: false,
        ritual: false,
        description: "A bright streak flashes from your pointing finger to a point you choose within range and then blossoms with a low roar into an explosion of flame. Each creature in a 20-foot-radius sphere centered on that point must make a Dexterity saving throw. A target takes 8d6 fire damage on a failed save, or half as much damage on a successful one.".to_string(),
        damage_dice: Some("8d6".to_string()),
        damage_type: Some(DamageType::Fire),
        damage_scaling: DamageScaling::PerSlotLevel { extra_dice: "1d6".to_string() },
        healing_dice: None,
        save_type: Some(Ability::Dexterity),
        save_effect: Some("half damage".to_string()),
        attack_type: None,
        area_of_effect: AreaOfEffect::Sphere(20),
        classes: vec![SpellClass::Sorcerer, SpellClass::Wizard],
    });

    db.insert("lightning bolt".to_string(), SpellData {
        name: "Lightning Bolt".to_string(),
        level: 3,
        school: SpellSchool::Evocation,
        casting_time: CastingTime::Action,
        range: SpellRange::SelfLine(100, 5),
        components: Components::vsm("a bit of fur and a rod of amber, crystal, or glass"),
        duration: SpellDuration::Instantaneous,
        concentration: false,
        ritual: false,
        description: "A stroke of lightning forming a line 100 feet long and 5 feet wide blasts out from you in a direction you choose. Each creature in the line must make a Dexterity saving throw. A creature takes 8d6 lightning damage on a failed save, or half as much damage on a successful one.".to_string(),
        damage_dice: Some("8d6".to_string()),
        damage_type: Some(DamageType::Lightning),
        damage_scaling: DamageScaling::PerSlotLevel { extra_dice: "1d6".to_string() },
        healing_dice: None,
        save_type: Some(Ability::Dexterity),
        save_effect: Some("half damage".to_string()),
        attack_type: None,
        area_of_effect: AreaOfEffect::Line(100, 5),
        classes: vec![SpellClass::Sorcerer, SpellClass::Wizard],
    });

    db.insert("counterspell".to_string(), SpellData {
        name: "Counterspell".to_string(),
        level: 3,
        school: SpellSchool::Abjuration,
        casting_time: CastingTime::Reaction("which you take when you see a creature within 60 feet casting a spell".to_string()),
        range: SpellRange::Feet(60),
        components: Components::s(),
        duration: SpellDuration::Instantaneous,
        concentration: false,
        ritual: false,
        description: "You attempt to interrupt a creature in the process of casting a spell. If the creature is casting a spell of 3rd level or lower, its spell fails. If it is casting a spell of 4th level or higher, make an ability check using your spellcasting ability. The DC equals 10 + the spell's level. On a success, the creature's spell fails.".to_string(),
        damage_dice: None,
        damage_type: None,
        damage_scaling: DamageScaling::None,
        healing_dice: None,
        save_type: None,
        save_effect: None,
        attack_type: None,
        area_of_effect: AreaOfEffect::None,
        classes: vec![SpellClass::Sorcerer, SpellClass::Warlock, SpellClass::Wizard],
    });

    db.insert("dispel magic".to_string(), SpellData {
        name: "Dispel Magic".to_string(),
        level: 3,
        school: SpellSchool::Abjuration,
        casting_time: CastingTime::Action,
        range: SpellRange::Feet(120),
        components: Components::vs(),
        duration: SpellDuration::Instantaneous,
        concentration: false,
        ritual: false,
        description: "Choose one creature, object, or magical effect within range. Any spell of 3rd level or lower on the target ends. For each spell of 4th level or higher on the target, make an ability check using your spellcasting ability. The DC equals 10 + the spell's level. On a successful check, the spell ends.".to_string(),
        damage_dice: None,
        damage_type: None,
        damage_scaling: DamageScaling::None,
        healing_dice: None,
        save_type: None,
        save_effect: None,
        attack_type: None,
        area_of_effect: AreaOfEffect::None,
        classes: vec![SpellClass::Bard, SpellClass::Cleric, SpellClass::Druid, SpellClass::Paladin, SpellClass::Sorcerer, SpellClass::Warlock, SpellClass::Wizard],
    });

    db.insert("spirit guardians".to_string(), SpellData {
        name: "Spirit Guardians".to_string(),
        level: 3,
        school: SpellSchool::Conjuration,
        casting_time: CastingTime::Action,
        range: SpellRange::SelfRadius(15),
        components: Components::vsm("a holy symbol"),
        duration: SpellDuration::Minutes(10),
        concentration: true,
        ritual: false,
        description: "You call forth spirits to protect you. They flit around you to a distance of 15 feet for the duration. When a creature enters the area for the first time on a turn or starts its turn there, it must make a Wisdom saving throw. On a failed save, the creature takes 3d8 radiant damage (if you are good or neutral) or 3d8 necrotic damage (if you are evil). On a successful save, the creature takes half as much damage.".to_string(),
        damage_dice: Some("3d8".to_string()),
        damage_type: Some(DamageType::Radiant),
        damage_scaling: DamageScaling::PerSlotLevel { extra_dice: "1d8".to_string() },
        healing_dice: None,
        save_type: Some(Ability::Wisdom),
        save_effect: Some("half damage".to_string()),
        attack_type: None,
        area_of_effect: AreaOfEffect::Sphere(15),
        classes: vec![SpellClass::Cleric],
    });

    db
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_spell() {
        let fireball = get_spell("fireball").expect("Fireball should exist");
        assert_eq!(fireball.name, "Fireball");
        assert_eq!(fireball.level, 3);
        assert_eq!(fireball.school, SpellSchool::Evocation);
        assert!(fireball.damage_dice.is_some());
    }

    #[test]
    fn test_case_insensitive_lookup() {
        assert!(get_spell("FIREBALL").is_some());
        assert!(get_spell("Fireball").is_some());
        assert!(get_spell("fireball").is_some());
    }

    #[test]
    fn test_cantrip_scaling() {
        let fire_bolt = get_spell("fire bolt").expect("Fire Bolt should exist");
        assert!(fire_bolt.is_cantrip());

        // Level 1: 1d10
        assert_eq!(fire_bolt.cantrip_dice_count(1), 1);
        // Level 5: 2d10
        assert_eq!(fire_bolt.cantrip_dice_count(5), 2);
        // Level 11: 3d10
        assert_eq!(fire_bolt.cantrip_dice_count(11), 3);
        // Level 17: 4d10
        assert_eq!(fire_bolt.cantrip_dice_count(17), 4);
    }

    #[test]
    fn test_spell_classes() {
        let cure_wounds = get_spell("cure wounds").expect("Cure Wounds should exist");
        assert!(cure_wounds.classes.contains(&SpellClass::Cleric));
        assert!(cure_wounds.classes.contains(&SpellClass::Druid));
        assert!(!cure_wounds.classes.contains(&SpellClass::Wizard));
    }

    #[test]
    fn test_concentration_spell() {
        let hold_person = get_spell("hold person").expect("Hold Person should exist");
        assert!(hold_person.concentration);

        let fireball = get_spell("fireball").expect("Fireball should exist");
        assert!(!fireball.concentration);
    }

    #[test]
    fn test_spells_by_level() {
        let cantrips: Vec<_> = spells_by_level(0).collect();
        assert!(!cantrips.is_empty());
        for spell in cantrips {
            assert_eq!(spell.level, 0);
        }
    }

    #[test]
    fn test_spells_for_class() {
        let wizard_spells: Vec<_> = spells_for_class(SpellClass::Wizard).collect();
        assert!(!wizard_spells.is_empty());

        // Fire Bolt should be available to Wizards
        assert!(wizard_spells.iter().any(|s| s.name == "Fire Bolt"));

        // Eldritch Blast should NOT be available to Wizards
        assert!(!wizard_spells.iter().any(|s| s.name == "Eldritch Blast"));
    }
}
