//! D&D 5e Rules Engine with Intent/Effect system.
//!
//! This module implements the core game mechanic pipeline:
//! 1. AI suggests an Intent (what the player/NPC wants to do)
//! 2. RulesEngine resolves the Intent using D&D 5e rules
//! 3. Effects are produced that describe state changes
//! 4. Effects are applied to the GameWorld
//!
//! This separation ensures deterministic, testable game mechanics
//! independent of AI decision-making.

use crate::dice::{self, Advantage, DiceExpression, RollResult};
use crate::world::{
    Ability, ActiveCondition, CharacterId, Combatant, Condition, GameWorld, Skill,
};
use serde::{Deserialize, Serialize};

/// An intent represents what a character wants to do.
/// The AI generates intents, the RulesEngine resolves them.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Intent {
    /// Attack a target with a weapon
    Attack {
        attacker_id: CharacterId,
        target_id: CharacterId,
        weapon_name: String,
        advantage: Advantage,
    },

    /// Cast a spell
    CastSpell {
        caster_id: CharacterId,
        spell_name: String,
        targets: Vec<CharacterId>,
        spell_level: u8,
    },

    /// Make a skill check
    SkillCheck {
        character_id: CharacterId,
        skill: Skill,
        dc: i32,
        advantage: Advantage,
        description: String,
    },

    /// Make an ability check (raw ability, not skill)
    AbilityCheck {
        character_id: CharacterId,
        ability: Ability,
        dc: i32,
        advantage: Advantage,
        description: String,
    },

    /// Make a saving throw
    SavingThrow {
        character_id: CharacterId,
        ability: Ability,
        dc: i32,
        advantage: Advantage,
        source: String,
    },

    /// Deal damage to a target
    Damage {
        target_id: CharacterId,
        amount: i32,
        damage_type: DamageType,
        source: String,
    },

    /// Heal a target
    Heal {
        target_id: CharacterId,
        amount: i32,
        source: String,
    },

    /// Apply a condition to a target
    ApplyCondition {
        target_id: CharacterId,
        condition: Condition,
        source: String,
        duration_rounds: Option<u32>,
    },

    /// Remove a condition from a target
    RemoveCondition {
        target_id: CharacterId,
        condition: Condition,
    },

    /// Move to a different location or position
    Move {
        character_id: CharacterId,
        destination: String,
        distance_feet: u32,
    },

    /// Take a short rest
    ShortRest,

    /// Take a long rest
    LongRest,

    /// Start combat
    StartCombat {
        combatants: Vec<CombatantInit>,
    },

    /// End combat
    EndCombat,

    /// Advance to next turn in combat
    NextTurn,

    /// Roll initiative for a character
    RollInitiative {
        character_id: CharacterId,
        name: String,
        modifier: i8,
        is_player: bool,
    },

    /// Generic dice roll (not tied to a specific mechanic)
    RollDice {
        notation: String,
        purpose: String,
    },

    /// Advance game time
    AdvanceTime {
        minutes: u32,
    },

    /// Add experience points
    GainExperience {
        amount: u32,
    },

    /// Use a class feature
    UseFeature {
        character_id: CharacterId,
        feature_name: String,
    },

    /// Remember a story fact (for narrative consistency)
    RememberFact {
        subject_name: String,
        subject_type: String,
        fact: String,
        category: String,
        related_entities: Vec<String>,
        importance: f32,
    },
}

/// Initial combatant data for starting combat.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatantInit {
    pub id: CharacterId,
    pub name: String,
    pub is_player: bool,
    pub is_ally: bool,
    pub current_hp: i32,
    pub max_hp: i32,
}

/// Common D&D damage types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DamageType {
    Slashing,
    Piercing,
    Bludgeoning,
    Fire,
    Cold,
    Lightning,
    Thunder,
    Acid,
    Poison,
    Necrotic,
    Radiant,
    Force,
    Psychic,
}

impl DamageType {
    pub fn name(&self) -> &'static str {
        match self {
            DamageType::Slashing => "slashing",
            DamageType::Piercing => "piercing",
            DamageType::Bludgeoning => "bludgeoning",
            DamageType::Fire => "fire",
            DamageType::Cold => "cold",
            DamageType::Lightning => "lightning",
            DamageType::Thunder => "thunder",
            DamageType::Acid => "acid",
            DamageType::Poison => "poison",
            DamageType::Necrotic => "necrotic",
            DamageType::Radiant => "radiant",
            DamageType::Force => "force",
            DamageType::Psychic => "psychic",
        }
    }
}

/// The result of resolving an intent.
#[derive(Debug, Clone)]
pub struct Resolution {
    pub effects: Vec<Effect>,
    pub narrative: String,
}

impl Resolution {
    pub fn new(narrative: impl Into<String>) -> Self {
        Self {
            effects: Vec::new(),
            narrative: narrative.into(),
        }
    }

    pub fn with_effect(mut self, effect: Effect) -> Self {
        self.effects.push(effect);
        self
    }

    pub fn with_effects(mut self, effects: impl IntoIterator<Item = Effect>) -> Self {
        self.effects.extend(effects);
        self
    }
}

/// Effects are the result of resolving an intent.
/// They describe concrete state changes to apply to the GameWorld.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Effect {
    /// A dice roll occurred
    DiceRolled {
        roll: RollResult,
        purpose: String,
    },

    /// HP changed (damage or healing)
    HpChanged {
        target_id: CharacterId,
        amount: i32,
        new_current: i32,
        new_max: i32,
        dropped_to_zero: bool,
    },

    /// A condition was applied
    ConditionApplied {
        target_id: CharacterId,
        condition: Condition,
        source: String,
    },

    /// A condition was removed
    ConditionRemoved {
        target_id: CharacterId,
        condition: Condition,
    },

    /// Combat started
    CombatStarted,

    /// Combat ended
    CombatEnded,

    /// Turn advanced in combat
    TurnAdvanced {
        round: u32,
        current_combatant: String,
    },

    /// Initiative rolled
    InitiativeRolled {
        character_id: CharacterId,
        name: String,
        roll: i32,
        total: i32,
    },

    /// Combatant added to initiative order
    CombatantAdded {
        id: CharacterId,
        name: String,
        initiative: i32,
        is_ally: bool,
        current_hp: i32,
        max_hp: i32,
    },

    /// Time advanced
    TimeAdvanced {
        minutes: u32,
    },

    /// Experience gained
    ExperienceGained {
        amount: u32,
        new_total: u32,
    },

    /// Level up occurred
    LevelUp {
        new_level: u8,
    },

    /// Feature use consumed
    FeatureUsed {
        feature_name: String,
        uses_remaining: u8,
    },

    /// Spell slot consumed
    SpellSlotUsed {
        level: u8,
        remaining: u8,
    },

    /// Rest completed
    RestCompleted {
        rest_type: RestType,
    },

    /// A check succeeded
    CheckSucceeded {
        check_type: String,
        roll: i32,
        dc: i32,
    },

    /// A check failed
    CheckFailed {
        check_type: String,
        roll: i32,
        dc: i32,
    },

    /// Attack hit
    AttackHit {
        attacker_name: String,
        target_name: String,
        attack_roll: i32,
        target_ac: u8,
        is_critical: bool,
    },

    /// Attack missed
    AttackMissed {
        attacker_name: String,
        target_name: String,
        attack_roll: i32,
        target_ac: u8,
    },

    /// A story fact was recorded for memory
    FactRemembered {
        subject_name: String,
        subject_type: String,
        fact: String,
        category: String,
        related_entities: Vec<String>,
        importance: f32,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RestType {
    Short,
    Long,
}

/// The rules engine resolves intents into effects using D&D 5e rules.
pub struct RulesEngine;

impl RulesEngine {
    pub fn new() -> Self {
        Self
    }

    /// Resolve an intent and produce effects.
    pub fn resolve(&self, world: &GameWorld, intent: Intent) -> Resolution {
        match intent {
            Intent::Attack { attacker_id, target_id, weapon_name, advantage } => {
                self.resolve_attack(world, attacker_id, target_id, &weapon_name, advantage)
            }
            Intent::SkillCheck { character_id, skill, dc, advantage, description } => {
                self.resolve_skill_check(world, character_id, skill, dc, advantage, &description)
            }
            Intent::AbilityCheck { character_id, ability, dc, advantage, description } => {
                self.resolve_ability_check(world, character_id, ability, dc, advantage, &description)
            }
            Intent::SavingThrow { character_id, ability, dc, advantage, source } => {
                self.resolve_saving_throw(world, character_id, ability, dc, advantage, &source)
            }
            Intent::Damage { target_id, amount, damage_type, source } => {
                self.resolve_damage(world, target_id, amount, damage_type, &source)
            }
            Intent::Heal { target_id, amount, source } => {
                self.resolve_heal(world, target_id, amount, &source)
            }
            Intent::ApplyCondition { target_id, condition, source, duration_rounds } => {
                self.resolve_apply_condition(world, target_id, condition, &source, duration_rounds)
            }
            Intent::RemoveCondition { target_id, condition } => {
                self.resolve_remove_condition(world, target_id, condition)
            }
            Intent::ShortRest => self.resolve_short_rest(world),
            Intent::LongRest => self.resolve_long_rest(world),
            Intent::StartCombat { combatants } => self.resolve_start_combat(world, combatants),
            Intent::EndCombat => self.resolve_end_combat(world),
            Intent::NextTurn => self.resolve_next_turn(world),
            Intent::RollInitiative { character_id, name, modifier, is_player } => {
                self.resolve_roll_initiative(character_id, &name, modifier, is_player)
            }
            Intent::RollDice { notation, purpose } => {
                self.resolve_roll_dice(&notation, &purpose)
            }
            Intent::AdvanceTime { minutes } => self.resolve_advance_time(minutes),
            Intent::GainExperience { amount } => self.resolve_gain_experience(world, amount),
            Intent::UseFeature { character_id, feature_name } => {
                self.resolve_use_feature(world, character_id, &feature_name)
            }
            Intent::RememberFact {
                subject_name,
                subject_type,
                fact,
                category,
                related_entities,
                importance,
            } => self.resolve_remember_fact(
                &subject_name,
                &subject_type,
                &fact,
                &category,
                &related_entities,
                importance,
            ),
            #[allow(unreachable_patterns)]
            _ => Resolution::new("Intent not yet implemented"),
        }
    }

    fn resolve_attack(
        &self,
        world: &GameWorld,
        _attacker_id: CharacterId,
        _target_id: CharacterId,
        weapon_name: &str,
        advantage: Advantage,
    ) -> Resolution {
        let attacker = &world.player_character;
        let target_ac = 15; // TODO: Get from target

        // Roll attack
        let attack_mod = attacker.ability_scores.modifier(Ability::Strength) + attacker.proficiency_bonus();
        let attack_expr = DiceExpression::parse(&format!("1d20+{attack_mod}")).unwrap();
        let attack_roll = attack_expr.roll_with_advantage(advantage);

        let mut resolution = Resolution::new(format!(
            "{} attacks with {} (roll: {} vs AC {})",
            attacker.name, weapon_name, attack_roll.total, target_ac
        ));

        resolution = resolution.with_effect(Effect::DiceRolled {
            roll: attack_roll.clone(),
            purpose: format!("Attack with {weapon_name}"),
        });

        if attack_roll.total >= target_ac as i32 || attack_roll.is_critical() {
            resolution = resolution.with_effect(Effect::AttackHit {
                attacker_name: attacker.name.clone(),
                target_name: "target".to_string(),
                attack_roll: attack_roll.total,
                target_ac,
                is_critical: attack_roll.is_critical(),
            });

            // Roll damage
            let damage_dice = if attack_roll.is_critical() { "2d8+3" } else { "1d8+3" };
            let damage_roll = dice::roll(damage_dice).unwrap();
            resolution = resolution.with_effect(Effect::DiceRolled {
                roll: damage_roll.clone(),
                purpose: "Damage".to_string(),
            });
        } else {
            resolution = resolution.with_effect(Effect::AttackMissed {
                attacker_name: attacker.name.clone(),
                target_name: "target".to_string(),
                attack_roll: attack_roll.total,
                target_ac,
            });
        }

        resolution
    }

    fn resolve_skill_check(
        &self,
        world: &GameWorld,
        _character_id: CharacterId,
        skill: Skill,
        dc: i32,
        advantage: Advantage,
        description: &str,
    ) -> Resolution {
        let character = &world.player_character;
        let modifier = character.skill_modifier(skill);

        let expr = DiceExpression::parse(&format!("1d20+{modifier}")).unwrap();
        let roll = expr.roll_with_advantage(advantage);

        let success = roll.total >= dc;
        let result_str = if success { "succeeds" } else { "fails" };

        let mut resolution = Resolution::new(format!(
            "{} {} ({} check: {} vs DC {})",
            character.name, result_str, skill.name(), roll.total, dc
        ));

        resolution = resolution.with_effect(Effect::DiceRolled {
            roll: roll.clone(),
            purpose: format!("{} check - {}", skill.name(), description),
        });

        if success {
            resolution = resolution.with_effect(Effect::CheckSucceeded {
                check_type: skill.name().to_string(),
                roll: roll.total,
                dc,
            });
        } else {
            resolution = resolution.with_effect(Effect::CheckFailed {
                check_type: skill.name().to_string(),
                roll: roll.total,
                dc,
            });
        }

        resolution
    }

    fn resolve_ability_check(
        &self,
        world: &GameWorld,
        _character_id: CharacterId,
        ability: Ability,
        dc: i32,
        advantage: Advantage,
        description: &str,
    ) -> Resolution {
        let character = &world.player_character;
        let modifier = character.ability_scores.modifier(ability);

        let expr = DiceExpression::parse(&format!("1d20+{modifier}")).unwrap();
        let roll = expr.roll_with_advantage(advantage);

        let success = roll.total >= dc;
        let result_str = if success { "succeeds" } else { "fails" };

        let mut resolution = Resolution::new(format!(
            "{} {} ({} check: {} vs DC {})",
            character.name, result_str, ability.abbreviation(), roll.total, dc
        ));

        resolution = resolution.with_effect(Effect::DiceRolled {
            roll: roll.clone(),
            purpose: format!("{} check - {}", ability.abbreviation(), description),
        });

        if success {
            resolution.with_effect(Effect::CheckSucceeded {
                check_type: ability.abbreviation().to_string(),
                roll: roll.total,
                dc,
            })
        } else {
            resolution.with_effect(Effect::CheckFailed {
                check_type: ability.abbreviation().to_string(),
                roll: roll.total,
                dc,
            })
        }
    }

    fn resolve_saving_throw(
        &self,
        world: &GameWorld,
        _character_id: CharacterId,
        ability: Ability,
        dc: i32,
        advantage: Advantage,
        source: &str,
    ) -> Resolution {
        let character = &world.player_character;
        let modifier = character.saving_throw_modifier(ability);

        let expr = DiceExpression::parse(&format!("1d20+{modifier}")).unwrap();
        let roll = expr.roll_with_advantage(advantage);

        let success = roll.total >= dc;
        let result_str = if success { "succeeds" } else { "fails" };

        let mut resolution = Resolution::new(format!(
            "{} {} on {} saving throw ({} vs DC {})",
            character.name, result_str, ability.abbreviation(), roll.total, dc
        ));

        resolution = resolution.with_effect(Effect::DiceRolled {
            roll: roll.clone(),
            purpose: format!("{} save vs {}", ability.abbreviation(), source),
        });

        if success {
            resolution.with_effect(Effect::CheckSucceeded {
                check_type: format!("{} save", ability.abbreviation()),
                roll: roll.total,
                dc,
            })
        } else {
            resolution.with_effect(Effect::CheckFailed {
                check_type: format!("{} save", ability.abbreviation()),
                roll: roll.total,
                dc,
            })
        }
    }

    fn resolve_damage(
        &self,
        world: &GameWorld,
        target_id: CharacterId,
        amount: i32,
        damage_type: DamageType,
        source: &str,
    ) -> Resolution {
        let target = &world.player_character;
        let mut hp = target.hit_points.clone();
        let result = hp.take_damage(amount);

        // Build narrative with HP status so DM knows the character's state
        let hp_status = if result.dropped_to_zero {
            format!(" (HP: 0/{} - UNCONSCIOUS! Character falls and begins making death saving throws)", hp.maximum)
        } else if hp.current <= hp.maximum / 4 {
            format!(" (HP: {}/{} - critically wounded)", hp.current, hp.maximum)
        } else if hp.current <= hp.maximum / 2 {
            format!(" (HP: {}/{} - bloodied)", hp.current, hp.maximum)
        } else {
            format!(" (HP: {}/{})", hp.current, hp.maximum)
        };

        let resolution = Resolution::new(format!(
            "{} takes {} {} damage from {}{}",
            target.name, amount, damage_type.name(), source, hp_status
        ));

        resolution.with_effect(Effect::HpChanged {
            target_id,
            amount: -amount,
            new_current: hp.current,
            new_max: hp.maximum,
            dropped_to_zero: result.dropped_to_zero,
        })
    }

    fn resolve_heal(
        &self,
        world: &GameWorld,
        target_id: CharacterId,
        amount: i32,
        source: &str,
    ) -> Resolution {
        let target = &world.player_character;
        let mut hp = target.hit_points.clone();
        let was_unconscious = hp.current <= 0;
        let healed = hp.heal(amount);

        // Build narrative with HP status
        let hp_status = if was_unconscious && hp.current > 0 {
            format!(" (HP: {}/{} - regains consciousness!)", hp.current, hp.maximum)
        } else if hp.current == hp.maximum {
            format!(" (HP: {}/{} - fully healed)", hp.current, hp.maximum)
        } else {
            format!(" (HP: {}/{})", hp.current, hp.maximum)
        };

        let resolution = Resolution::new(format!(
            "{} heals {} hit points from {}{}",
            target.name, healed, source, hp_status
        ));

        resolution.with_effect(Effect::HpChanged {
            target_id,
            amount: healed,
            new_current: hp.current,
            new_max: hp.maximum,
            dropped_to_zero: false,
        })
    }

    fn resolve_apply_condition(
        &self,
        world: &GameWorld,
        target_id: CharacterId,
        condition: Condition,
        source: &str,
        _duration_rounds: Option<u32>,
    ) -> Resolution {
        let target = &world.player_character;

        let resolution = Resolution::new(format!(
            "{} is now {} ({})",
            target.name, condition.name(), source
        ));

        resolution.with_effect(Effect::ConditionApplied {
            target_id,
            condition,
            source: source.to_string(),
        })
    }

    fn resolve_remove_condition(
        &self,
        world: &GameWorld,
        target_id: CharacterId,
        condition: Condition,
    ) -> Resolution {
        let target = &world.player_character;

        let resolution = Resolution::new(format!(
            "{} is no longer {}",
            target.name, condition.name()
        ));

        resolution.with_effect(Effect::ConditionRemoved { target_id, condition })
    }

    fn resolve_short_rest(&self, _world: &GameWorld) -> Resolution {
        Resolution::new("The party takes a short rest, spending 1 hour resting.")
            .with_effect(Effect::TimeAdvanced { minutes: 60 })
            .with_effect(Effect::RestCompleted { rest_type: RestType::Short })
    }

    fn resolve_long_rest(&self, _world: &GameWorld) -> Resolution {
        Resolution::new("The party takes a long rest, spending 8 hours resting.")
            .with_effect(Effect::TimeAdvanced { minutes: 480 })
            .with_effect(Effect::RestCompleted { rest_type: RestType::Long })
    }

    fn resolve_start_combat(&self, world: &GameWorld, combatants: Vec<CombatantInit>) -> Resolution {
        let mut resolution = Resolution::new("Combat begins! Roll for initiative.")
            .with_effect(Effect::CombatStarted);

        // Roll initiative for each combatant
        for init in combatants {
            let modifier = if init.is_player {
                world.player_character.initiative_modifier()
            } else {
                0 // NPCs would have their own modifier
            };

            let roll = dice::roll("1d20").unwrap();
            let total = roll.total + modifier as i32;

            resolution = resolution.with_effect(Effect::InitiativeRolled {
                character_id: init.id,
                name: init.name.clone(),
                roll: roll.total,
                total,
            });

            resolution = resolution.with_effect(Effect::CombatantAdded {
                id: init.id,
                name: init.name,
                initiative: total,
                is_ally: init.is_ally,
                current_hp: init.current_hp,
                max_hp: init.max_hp,
            });
        }

        resolution
    }

    fn resolve_end_combat(&self, _world: &GameWorld) -> Resolution {
        Resolution::new("Combat ends.")
            .with_effect(Effect::CombatEnded)
    }

    fn resolve_next_turn(&self, world: &GameWorld) -> Resolution {
        if let Some(ref combat) = world.combat {
            let mut combat_clone = combat.clone();
            combat_clone.next_turn();

            let current = combat_clone.current_combatant()
                .map(|c| c.name.clone())
                .unwrap_or_else(|| "Unknown".to_string());

            Resolution::new(format!("Next turn: {} (Round {})", current, combat_clone.round))
                .with_effect(Effect::TurnAdvanced {
                    round: combat_clone.round,
                    current_combatant: current,
                })
        } else {
            Resolution::new("No combat in progress")
        }
    }

    fn resolve_roll_initiative(
        &self,
        character_id: CharacterId,
        name: &str,
        modifier: i8,
        _is_player: bool,
    ) -> Resolution {
        let roll = dice::roll("1d20").unwrap();
        let total = roll.total + modifier as i32;

        Resolution::new(format!("{} rolls initiative: {} + {} = {}", name, roll.total, modifier, total))
            .with_effect(Effect::DiceRolled {
                roll: roll.clone(),
                purpose: "Initiative".to_string(),
            })
            .with_effect(Effect::InitiativeRolled {
                character_id,
                name: name.to_string(),
                roll: roll.total,
                total,
            })
    }

    fn resolve_roll_dice(&self, notation: &str, purpose: &str) -> Resolution {
        match dice::roll(notation) {
            Ok(roll) => {
                Resolution::new(format!("Rolling {notation} for {purpose}: {roll}"))
                    .with_effect(Effect::DiceRolled {
                        roll,
                        purpose: purpose.to_string(),
                    })
            }
            Err(e) => Resolution::new(format!("Failed to roll {notation}: {e}")),
        }
    }

    fn resolve_advance_time(&self, minutes: u32) -> Resolution {
        let hours = minutes / 60;
        let mins = minutes % 60;

        let time_str = if hours > 0 && mins > 0 {
            format!("{hours} hours and {mins} minutes")
        } else if hours > 0 {
            format!("{hours} hours")
        } else {
            format!("{mins} minutes")
        };

        Resolution::new(format!("{time_str} pass."))
            .with_effect(Effect::TimeAdvanced { minutes })
    }

    fn resolve_gain_experience(&self, world: &GameWorld, amount: u32) -> Resolution {
        let new_total = world.player_character.experience + amount;
        let current_level = world.player_character.level;

        // XP thresholds for levels 1-20
        let xp_thresholds = [
            0, 300, 900, 2700, 6500, 14000, 23000, 34000, 48000, 64000,
            85000, 100000, 120000, 140000, 165000, 195000, 225000, 265000, 305000, 355000,
        ];

        let new_level = xp_thresholds.iter()
            .rposition(|&threshold| new_total >= threshold)
            .map(|idx| (idx + 1) as u8)
            .unwrap_or(1);

        let mut resolution = Resolution::new(format!(
            "Gained {amount} experience points (Total: {new_total})"
        ));

        resolution = resolution.with_effect(Effect::ExperienceGained {
            amount,
            new_total,
        });

        if new_level > current_level {
            resolution = resolution.with_effect(Effect::LevelUp { new_level });
        }

        resolution
    }

    fn resolve_use_feature(
        &self,
        world: &GameWorld,
        _character_id: CharacterId,
        feature_name: &str,
    ) -> Resolution {
        let character = &world.player_character;

        if let Some(feature) = character.features.iter().find(|f| f.name == feature_name) {
            if let Some(ref uses) = feature.uses {
                if uses.current > 0 {
                    Resolution::new(format!("{} uses {} ({} uses remaining)",
                        character.name, feature_name, uses.current - 1))
                        .with_effect(Effect::FeatureUsed {
                            feature_name: feature_name.to_string(),
                            uses_remaining: uses.current - 1,
                        })
                } else {
                    Resolution::new(format!("{} has no uses of {} remaining", character.name, feature_name))
                }
            } else {
                Resolution::new(format!("{} uses {}", character.name, feature_name))
            }
        } else {
            Resolution::new(format!("{} does not have the feature {}", character.name, feature_name))
        }
    }

    fn resolve_remember_fact(
        &self,
        subject_name: &str,
        subject_type: &str,
        fact: &str,
        category: &str,
        related_entities: &[String],
        importance: f32,
    ) -> Resolution {
        // The actual storage is handled by the DM agent, not the rules engine.
        // We return a confirmation message and an effect that signals what to store.
        let related_str = if related_entities.is_empty() {
            String::new()
        } else {
            format!(" (related: {})", related_entities.join(", "))
        };

        Resolution::new(format!(
            "Noted: {subject_name} ({subject_type}) - {fact}{related_str}"
        ))
        .with_effect(Effect::FactRemembered {
            subject_name: subject_name.to_string(),
            subject_type: subject_type.to_string(),
            fact: fact.to_string(),
            category: category.to_string(),
            related_entities: related_entities.to_vec(),
            importance,
        })
    }
}

impl Default for RulesEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Apply effects to the game world.
pub fn apply_effects(world: &mut GameWorld, effects: &[Effect]) {
    for effect in effects {
        apply_effect(world, effect);
    }
}

/// Apply a single effect to the game world.
pub fn apply_effect(world: &mut GameWorld, effect: &Effect) {
    match effect {
        Effect::HpChanged { amount, dropped_to_zero, .. } => {
            if *amount < 0 {
                world.player_character.hit_points.take_damage(-*amount);
            } else {
                world.player_character.hit_points.heal(*amount);
            }
            if *dropped_to_zero {
                world.player_character.conditions.push(
                    ActiveCondition::new(Condition::Unconscious, "Dropped to 0 HP")
                );
            }
            // Sync HP to combat state if in combat
            if let Some(ref mut combat) = world.combat {
                let player_id = world.player_character.id;
                combat.update_combatant_hp(player_id, world.player_character.hit_points.current);
            }
        }
        Effect::ConditionApplied { condition, source, .. } => {
            world.player_character.conditions.push(
                ActiveCondition::new(*condition, source.clone())
            );
        }
        Effect::ConditionRemoved { condition, .. } => {
            world.player_character.conditions.retain(|c| c.condition != *condition);
        }
        Effect::CombatStarted => {
            world.start_combat();
        }
        Effect::CombatEnded => {
            world.end_combat();
        }
        Effect::CombatantAdded { id, name, initiative, is_ally, current_hp, max_hp } => {
            if let Some(ref mut combat) = world.combat {
                combat.add_combatant(Combatant {
                    id: *id,
                    name: name.clone(),
                    initiative: *initiative,
                    is_player: *id == world.player_character.id,
                    is_ally: *is_ally,
                    current_hp: *current_hp,
                    max_hp: *max_hp,
                });
            }
        }
        Effect::TurnAdvanced { .. } => {
            if let Some(ref mut combat) = world.combat {
                combat.next_turn();
            }
        }
        Effect::TimeAdvanced { minutes } => {
            world.game_time.advance_minutes(*minutes);
        }
        Effect::RestCompleted { rest_type } => {
            match rest_type {
                RestType::Short => world.short_rest(),
                RestType::Long => world.long_rest(),
            }
        }
        Effect::ExperienceGained { amount, .. } => {
            world.player_character.experience += amount;
        }
        Effect::LevelUp { new_level } => {
            world.player_character.level = *new_level;
        }
        Effect::FeatureUsed { feature_name, uses_remaining } => {
            if let Some(feature) = world.player_character.features.iter_mut()
                .find(|f| f.name == *feature_name)
            {
                if let Some(ref mut uses) = feature.uses {
                    uses.current = *uses_remaining;
                }
            }
        }
        Effect::SpellSlotUsed { level, .. } => {
            if let Some(ref mut spellcasting) = world.player_character.spellcasting {
                spellcasting.spell_slots.use_slot(*level);
            }
        }
        // Effects that don't modify state (informational)
        Effect::DiceRolled { .. } => {}
        Effect::CheckSucceeded { .. } => {}
        Effect::CheckFailed { .. } => {}
        Effect::AttackHit { .. } => {}
        Effect::AttackMissed { .. } => {}
        Effect::InitiativeRolled { .. } => {}
        // FactRemembered is handled by the DM agent's memory system, not world state
        Effect::FactRemembered { .. } => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::create_sample_fighter;

    #[test]
    fn test_skill_check() {
        let character = create_sample_fighter("Roland");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let intent = Intent::SkillCheck {
            character_id: world.player_character.id,
            skill: Skill::Athletics,
            dc: 15,
            advantage: Advantage::Normal,
            description: "Climbing a cliff".to_string(),
        };

        let resolution = engine.resolve(&world, intent);
        assert!(!resolution.effects.is_empty());
        assert!(resolution.effects.iter().any(|e| matches!(e, Effect::DiceRolled { .. })));
    }

    #[test]
    fn test_damage() {
        let character = create_sample_fighter("Roland");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let intent = Intent::Damage {
            target_id: world.player_character.id,
            amount: 10,
            damage_type: DamageType::Slashing,
            source: "Goblin".to_string(),
        };

        let resolution = engine.resolve(&world, intent);
        assert!(resolution.effects.iter().any(|e| matches!(e, Effect::HpChanged { amount, .. } if *amount == -10)));
    }

    #[test]
    fn test_heal() {
        let mut character = create_sample_fighter("Roland");
        character.hit_points.current = 10;
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let intent = Intent::Heal {
            target_id: world.player_character.id,
            amount: 5,
            source: "Healing Potion".to_string(),
        };

        let resolution = engine.resolve(&world, intent);
        assert!(resolution.effects.iter().any(|e| matches!(e, Effect::HpChanged { amount, .. } if *amount == 5)));
    }

    #[test]
    fn test_apply_damage_effect() {
        let character = create_sample_fighter("Roland");
        let mut world = GameWorld::new("Test", character);
        let initial_hp = world.player_character.hit_points.current;

        let effect = Effect::HpChanged {
            target_id: world.player_character.id,
            amount: -10,
            new_current: initial_hp - 10,
            new_max: world.player_character.hit_points.maximum,
            dropped_to_zero: false,
        };

        apply_effect(&mut world, &effect);
        assert_eq!(world.player_character.hit_points.current, initial_hp - 10);
    }

    #[test]
    fn test_start_combat() {
        let character = create_sample_fighter("Roland");
        let world = GameWorld::new("Test", character.clone());
        let engine = RulesEngine::new();

        let intent = Intent::StartCombat {
            combatants: vec![
                CombatantInit {
                    id: character.id,
                    name: "Roland".to_string(),
                    is_player: true,
                    is_ally: true,
                    current_hp: character.hit_points.current,
                    max_hp: character.hit_points.maximum,
                },
            ],
        };

        let resolution = engine.resolve(&world, intent);
        assert!(resolution.effects.iter().any(|e| matches!(e, Effect::CombatStarted)));
        assert!(resolution.effects.iter().any(|e| matches!(e, Effect::InitiativeRolled { .. })));
    }

    #[test]
    fn test_roll_dice() {
        let character = create_sample_fighter("Roland");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let intent = Intent::RollDice {
            notation: "2d6+3".to_string(),
            purpose: "Damage".to_string(),
        };

        let resolution = engine.resolve(&world, intent);
        assert!(resolution.effects.iter().any(|e| matches!(e, Effect::DiceRolled { .. })));
    }

    #[test]
    fn test_damage_narrative_includes_hp_status() {
        let character = create_sample_fighter("Roland");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let intent = Intent::Damage {
            target_id: world.player_character.id,
            amount: 5,
            damage_type: DamageType::Slashing,
            source: "Goblin".to_string(),
        };

        let resolution = engine.resolve(&world, intent);
        // Narrative should include HP status
        assert!(resolution.narrative.contains("HP:"), "Damage narrative should include HP status: {}", resolution.narrative);
    }

    #[test]
    fn test_damage_narrative_shows_unconscious() {
        let mut character = create_sample_fighter("Roland");
        character.hit_points.current = 5; // Low HP
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let intent = Intent::Damage {
            target_id: world.player_character.id,
            amount: 10, // More than current HP
            damage_type: DamageType::Slashing,
            source: "Goblin".to_string(),
        };

        let resolution = engine.resolve(&world, intent);
        // Narrative should indicate unconscious
        assert!(resolution.narrative.contains("UNCONSCIOUS"), "Lethal damage narrative should indicate UNCONSCIOUS: {}", resolution.narrative);
        // Effect should have dropped_to_zero
        assert!(resolution.effects.iter().any(|e| matches!(e, Effect::HpChanged { dropped_to_zero: true, .. })));
    }
}
