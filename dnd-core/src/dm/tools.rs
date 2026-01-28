//! D&D tools for the AI Dungeon Master.
//!
//! These tools allow the AI to interact with game mechanics
//! by generating Intents that the RulesEngine resolves.

use crate::dice::Advantage;
use crate::rules::{DamageType, Intent, CombatantInit};
use crate::world::{Ability, CharacterId, Condition, Skill, GameWorld};
use claude::Tool;
use serde_json::{json, Value};

/// Collection of D&D tools for the DM.
pub struct DmTools;

impl DmTools {
    /// Get all tool definitions for the Claude API.
    pub fn all() -> Vec<Tool> {
        vec![
            Self::roll_dice(),
            Self::skill_check(),
            Self::ability_check(),
            Self::saving_throw(),
            Self::apply_damage(),
            Self::apply_healing(),
            Self::apply_condition(),
            Self::remove_condition(),
            Self::start_combat(),
            Self::end_combat(),
            Self::next_turn(),
            Self::short_rest(),
            Self::long_rest(),
            Self::remember_fact(),
        ]
    }

    fn remember_fact() -> Tool {
        Tool {
            name: "remember_fact".to_string(),
            description: "Record an important story fact for future reference. Use this when introducing NPCs, establishing locations, recording player decisions, or revealing plot points. Facts are indexed and used to maintain narrative consistency.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "subject_name": {
                        "type": "string",
                        "description": "Name of the entity this fact is about (NPC name, location name, item name, etc.)"
                    },
                    "subject_type": {
                        "type": "string",
                        "enum": ["npc", "location", "item", "quest", "organization", "event", "creature"],
                        "description": "Type of entity"
                    },
                    "fact": {
                        "type": "string",
                        "description": "The fact to record in natural language"
                    },
                    "category": {
                        "type": "string",
                        "enum": ["appearance", "personality", "event", "relationship", "backstory", "motivation", "capability", "location", "possession", "status", "secret"],
                        "description": "Category of the fact"
                    },
                    "related_entities": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Names of other entities mentioned in this fact (optional)"
                    },
                    "importance": {
                        "type": "number",
                        "minimum": 0.1,
                        "maximum": 1.0,
                        "description": "How important this fact is (0.1-1.0, default 0.7)"
                    }
                },
                "required": ["subject_name", "subject_type", "fact", "category"]
            }),
        }
    }

    fn roll_dice() -> Tool {
        Tool {
            name: "roll_dice".to_string(),
            description: "Roll dice using standard D&D notation (e.g., '2d6+3', '1d20', '4d6kh3').".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "notation": {
                        "type": "string",
                        "description": "Dice notation (e.g., '2d6+3', '1d20+5', '4d6kh3')"
                    },
                    "purpose": {
                        "type": "string",
                        "description": "What the roll is for (e.g., 'damage', 'initiative')"
                    }
                },
                "required": ["notation", "purpose"]
            }),
        }
    }

    fn skill_check() -> Tool {
        Tool {
            name: "skill_check".to_string(),
            description: "Have a character make a skill check against a DC.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "skill": {
                        "type": "string",
                        "enum": ["athletics", "acrobatics", "sleight_of_hand", "stealth",
                                "arcana", "history", "investigation", "nature", "religion",
                                "animal_handling", "insight", "medicine", "perception", "survival",
                                "deception", "intimidation", "performance", "persuasion"],
                        "description": "The skill to check"
                    },
                    "dc": {
                        "type": "integer",
                        "description": "Difficulty Class for the check"
                    },
                    "description": {
                        "type": "string",
                        "description": "What the character is attempting"
                    },
                    "advantage": {
                        "type": "string",
                        "enum": ["normal", "advantage", "disadvantage"],
                        "description": "Advantage state for the roll"
                    }
                },
                "required": ["skill", "dc", "description"]
            }),
        }
    }

    fn ability_check() -> Tool {
        Tool {
            name: "ability_check".to_string(),
            description: "Have a character make a raw ability check (not tied to a skill).".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "ability": {
                        "type": "string",
                        "enum": ["strength", "dexterity", "constitution", "intelligence", "wisdom", "charisma"],
                        "description": "The ability to check"
                    },
                    "dc": {
                        "type": "integer",
                        "description": "Difficulty Class for the check"
                    },
                    "description": {
                        "type": "string",
                        "description": "What the character is attempting"
                    },
                    "advantage": {
                        "type": "string",
                        "enum": ["normal", "advantage", "disadvantage"],
                        "description": "Advantage state for the roll"
                    }
                },
                "required": ["ability", "dc", "description"]
            }),
        }
    }

    fn saving_throw() -> Tool {
        Tool {
            name: "saving_throw".to_string(),
            description: "Have a character make a saving throw.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "ability": {
                        "type": "string",
                        "enum": ["strength", "dexterity", "constitution", "intelligence", "wisdom", "charisma"],
                        "description": "The ability for the save"
                    },
                    "dc": {
                        "type": "integer",
                        "description": "Difficulty Class for the save"
                    },
                    "source": {
                        "type": "string",
                        "description": "What is causing the saving throw"
                    },
                    "advantage": {
                        "type": "string",
                        "enum": ["normal", "advantage", "disadvantage"],
                        "description": "Advantage state for the roll"
                    }
                },
                "required": ["ability", "dc", "source"]
            }),
        }
    }

    fn apply_damage() -> Tool {
        Tool {
            name: "apply_damage".to_string(),
            description: "Apply damage to a character or creature.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "amount": {
                        "type": "integer",
                        "description": "Amount of damage to apply"
                    },
                    "damage_type": {
                        "type": "string",
                        "enum": ["slashing", "piercing", "bludgeoning", "fire", "cold",
                                "lightning", "thunder", "acid", "poison", "necrotic",
                                "radiant", "force", "psychic"],
                        "description": "Type of damage"
                    },
                    "source": {
                        "type": "string",
                        "description": "Source of the damage"
                    },
                    "target": {
                        "type": "string",
                        "enum": ["player", "npc"],
                        "description": "Who receives the damage"
                    }
                },
                "required": ["amount", "damage_type", "source"]
            }),
        }
    }

    fn apply_healing() -> Tool {
        Tool {
            name: "apply_healing".to_string(),
            description: "Heal a character.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "amount": {
                        "type": "integer",
                        "description": "Amount of HP to restore"
                    },
                    "source": {
                        "type": "string",
                        "description": "Source of the healing"
                    }
                },
                "required": ["amount", "source"]
            }),
        }
    }

    fn apply_condition() -> Tool {
        Tool {
            name: "apply_condition".to_string(),
            description: "Apply a condition to a character.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "condition": {
                        "type": "string",
                        "enum": ["blinded", "charmed", "deafened", "frightened", "grappled",
                                "incapacitated", "invisible", "paralyzed", "petrified", "poisoned",
                                "prone", "restrained", "stunned", "unconscious"],
                        "description": "The condition to apply"
                    },
                    "source": {
                        "type": "string",
                        "description": "What caused the condition"
                    },
                    "duration_rounds": {
                        "type": "integer",
                        "description": "How many rounds the condition lasts (omit for indefinite)"
                    }
                },
                "required": ["condition", "source"]
            }),
        }
    }

    fn remove_condition() -> Tool {
        Tool {
            name: "remove_condition".to_string(),
            description: "Remove a condition from a character.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "condition": {
                        "type": "string",
                        "enum": ["blinded", "charmed", "deafened", "frightened", "grappled",
                                "incapacitated", "invisible", "paralyzed", "petrified", "poisoned",
                                "prone", "restrained", "stunned", "unconscious"],
                        "description": "The condition to remove"
                    }
                },
                "required": ["condition"]
            }),
        }
    }

    fn start_combat() -> Tool {
        Tool {
            name: "start_combat".to_string(),
            description: "Start a combat encounter. Initiative will be rolled for all combatants.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "enemies": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "name": { "type": "string" }
                            },
                            "required": ["name"]
                        },
                        "description": "List of enemy combatants"
                    }
                },
                "required": ["enemies"]
            }),
        }
    }

    fn end_combat() -> Tool {
        Tool {
            name: "end_combat".to_string(),
            description: "End the current combat encounter.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        }
    }

    fn next_turn() -> Tool {
        Tool {
            name: "next_turn".to_string(),
            description: "Advance to the next turn in combat.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        }
    }

    fn short_rest() -> Tool {
        Tool {
            name: "short_rest".to_string(),
            description: "Take a short rest (1 hour). Recover some abilities.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        }
    }

    fn long_rest() -> Tool {
        Tool {
            name: "long_rest".to_string(),
            description: "Take a long rest (8 hours). Fully recover HP and abilities.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        }
    }
}

/// Parse a tool call into an Intent.
pub fn parse_tool_call(name: &str, input: &Value, world: &GameWorld) -> Option<Intent> {
    match name {
        "roll_dice" => {
            let notation = input["notation"].as_str()?;
            let purpose = input["purpose"].as_str().unwrap_or("general roll");
            Some(Intent::RollDice {
                notation: notation.to_string(),
                purpose: purpose.to_string(),
            })
        }
        "skill_check" => {
            let skill = parse_skill(input["skill"].as_str()?)?;
            let dc = input["dc"].as_i64()? as i32;
            let description = input["description"].as_str().unwrap_or("").to_string();
            let advantage = parse_advantage(input["advantage"].as_str());
            Some(Intent::SkillCheck {
                character_id: world.player_character.id,
                skill,
                dc,
                advantage,
                description,
            })
        }
        "ability_check" => {
            let ability = parse_ability(input["ability"].as_str()?)?;
            let dc = input["dc"].as_i64()? as i32;
            let description = input["description"].as_str().unwrap_or("").to_string();
            let advantage = parse_advantage(input["advantage"].as_str());
            Some(Intent::AbilityCheck {
                character_id: world.player_character.id,
                ability,
                dc,
                advantage,
                description,
            })
        }
        "saving_throw" => {
            let ability = parse_ability(input["ability"].as_str()?)?;
            let dc = input["dc"].as_i64()? as i32;
            let source = input["source"].as_str().unwrap_or("unknown").to_string();
            let advantage = parse_advantage(input["advantage"].as_str());
            Some(Intent::SavingThrow {
                character_id: world.player_character.id,
                ability,
                dc,
                advantage,
                source,
            })
        }
        "apply_damage" => {
            let amount = input["amount"].as_i64()? as i32;
            let damage_type = parse_damage_type(input["damage_type"].as_str()?)?;
            let source = input["source"].as_str().unwrap_or("unknown").to_string();
            Some(Intent::Damage {
                target_id: world.player_character.id,
                amount,
                damage_type,
                source,
            })
        }
        "apply_healing" => {
            let amount = input["amount"].as_i64()? as i32;
            let source = input["source"].as_str().unwrap_or("healing").to_string();
            Some(Intent::Heal {
                target_id: world.player_character.id,
                amount,
                source,
            })
        }
        "apply_condition" => {
            let condition = parse_condition(input["condition"].as_str()?)?;
            let source = input["source"].as_str().unwrap_or("unknown").to_string();
            let duration_rounds = input["duration_rounds"].as_i64().map(|d| d as u32);
            Some(Intent::ApplyCondition {
                target_id: world.player_character.id,
                condition,
                source,
                duration_rounds,
            })
        }
        "remove_condition" => {
            let condition = parse_condition(input["condition"].as_str()?)?;
            Some(Intent::RemoveCondition {
                target_id: world.player_character.id,
                condition,
            })
        }
        "start_combat" => {
            let enemies = input["enemies"].as_array()?;
            let player_hp = &world.player_character.hit_points;
            let mut combatants = vec![CombatantInit {
                id: world.player_character.id,
                name: world.player_character.name.clone(),
                is_player: true,
                is_ally: true,
                current_hp: player_hp.current,
                max_hp: player_hp.maximum,
            }];

            for enemy in enemies {
                let name = enemy["name"].as_str().unwrap_or("Enemy").to_string();
                // Parse enemy HP if provided, default to 10/10 for basic enemies
                let max_hp = enemy["max_hp"].as_i64().unwrap_or(10) as i32;
                let current_hp = enemy["current_hp"].as_i64().unwrap_or(max_hp as i64) as i32;
                combatants.push(CombatantInit {
                    id: CharacterId::new(),
                    name,
                    is_player: false,
                    is_ally: false,
                    current_hp,
                    max_hp,
                });
            }

            Some(Intent::StartCombat { combatants })
        }
        "end_combat" => Some(Intent::EndCombat),
        "next_turn" => Some(Intent::NextTurn),
        "short_rest" => Some(Intent::ShortRest),
        "long_rest" => Some(Intent::LongRest),
        "remember_fact" => {
            let subject_name = input["subject_name"].as_str()?.to_string();
            let subject_type = input["subject_type"].as_str()?.to_string();
            let fact = input["fact"].as_str()?.to_string();
            let category = input["category"].as_str()?.to_string();
            let related_entities = input["related_entities"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();
            let importance = input["importance"].as_f64().unwrap_or(0.7) as f32;

            Some(Intent::RememberFact {
                subject_name,
                subject_type,
                fact,
                category,
                related_entities,
                importance,
            })
        }
        _ => None,
    }
}

fn parse_skill(s: &str) -> Option<Skill> {
    match s.to_lowercase().replace('_', "").as_str() {
        "athletics" => Some(Skill::Athletics),
        "acrobatics" => Some(Skill::Acrobatics),
        "sleightofhand" => Some(Skill::SleightOfHand),
        "stealth" => Some(Skill::Stealth),
        "arcana" => Some(Skill::Arcana),
        "history" => Some(Skill::History),
        "investigation" => Some(Skill::Investigation),
        "nature" => Some(Skill::Nature),
        "religion" => Some(Skill::Religion),
        "animalhandling" => Some(Skill::AnimalHandling),
        "insight" => Some(Skill::Insight),
        "medicine" => Some(Skill::Medicine),
        "perception" => Some(Skill::Perception),
        "survival" => Some(Skill::Survival),
        "deception" => Some(Skill::Deception),
        "intimidation" => Some(Skill::Intimidation),
        "performance" => Some(Skill::Performance),
        "persuasion" => Some(Skill::Persuasion),
        _ => None,
    }
}

fn parse_ability(s: &str) -> Option<Ability> {
    match s.to_lowercase().as_str() {
        "strength" | "str" => Some(Ability::Strength),
        "dexterity" | "dex" => Some(Ability::Dexterity),
        "constitution" | "con" => Some(Ability::Constitution),
        "intelligence" | "int" => Some(Ability::Intelligence),
        "wisdom" | "wis" => Some(Ability::Wisdom),
        "charisma" | "cha" => Some(Ability::Charisma),
        _ => None,
    }
}

fn parse_advantage(s: Option<&str>) -> Advantage {
    match s {
        Some("advantage") => Advantage::Advantage,
        Some("disadvantage") => Advantage::Disadvantage,
        _ => Advantage::Normal,
    }
}

fn parse_damage_type(s: &str) -> Option<DamageType> {
    match s.to_lowercase().as_str() {
        "slashing" => Some(DamageType::Slashing),
        "piercing" => Some(DamageType::Piercing),
        "bludgeoning" => Some(DamageType::Bludgeoning),
        "fire" => Some(DamageType::Fire),
        "cold" => Some(DamageType::Cold),
        "lightning" => Some(DamageType::Lightning),
        "thunder" => Some(DamageType::Thunder),
        "acid" => Some(DamageType::Acid),
        "poison" => Some(DamageType::Poison),
        "necrotic" => Some(DamageType::Necrotic),
        "radiant" => Some(DamageType::Radiant),
        "force" => Some(DamageType::Force),
        "psychic" => Some(DamageType::Psychic),
        _ => None,
    }
}

fn parse_condition(s: &str) -> Option<Condition> {
    match s.to_lowercase().as_str() {
        "blinded" => Some(Condition::Blinded),
        "charmed" => Some(Condition::Charmed),
        "deafened" => Some(Condition::Deafened),
        "frightened" => Some(Condition::Frightened),
        "grappled" => Some(Condition::Grappled),
        "incapacitated" => Some(Condition::Incapacitated),
        "invisible" => Some(Condition::Invisible),
        "paralyzed" => Some(Condition::Paralyzed),
        "petrified" => Some(Condition::Petrified),
        "poisoned" => Some(Condition::Poisoned),
        "prone" => Some(Condition::Prone),
        "restrained" => Some(Condition::Restrained),
        "stunned" => Some(Condition::Stunned),
        "unconscious" => Some(Condition::Unconscious),
        _ => None,
    }
}
