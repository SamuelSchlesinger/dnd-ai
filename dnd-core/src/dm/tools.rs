//! D&D tools for the AI Dungeon Master.
//!
//! These tools allow the AI to interact with game mechanics
//! by generating Intents that the RulesEngine resolves.

use crate::dice::Advantage;
use crate::rules::{CombatantInit, DamageType, Intent};
use crate::world::{Ability, CharacterId, Condition, GameWorld, Skill};
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
            Self::register_consequence(),
            // Inventory tools
            Self::give_item(),
            Self::remove_item(),
            Self::use_item(),
            Self::equip_item(),
            Self::unequip_item(),
            Self::adjust_gold(),
            Self::show_inventory(),
            Self::death_save(),
            Self::concentration_check(),
            Self::change_location(),
            // Class feature tools
            Self::use_rage(),
            Self::end_rage(),
            Self::use_ki(),
            Self::use_lay_on_hands(),
            Self::use_divine_smite(),
            Self::use_wild_shape(),
            Self::end_wild_shape(),
            Self::use_channel_divinity(),
            Self::use_bardic_inspiration(),
            Self::use_action_surge(),
            Self::use_second_wind(),
            Self::use_sorcery_points(),
            // Spellcasting
            Self::cast_spell(),
        ]
    }

    fn change_location() -> Tool {
        Tool {
            name: "change_location".to_string(),
            description: "Change the current location when the player travels somewhere new. Use this whenever the player moves to a different area, enters a building, or travels to a new place.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "new_location": {
                        "type": "string",
                        "description": "Name of the new location (e.g., 'The Dark Forest', 'Town Square', 'Goblin Cave')"
                    },
                    "location_type": {
                        "type": "string",
                        "enum": ["city", "town", "village", "dungeon", "wilderness", "building", "room", "other"],
                        "description": "Type of location"
                    },
                    "description": {
                        "type": "string",
                        "description": "Brief description of the location for future reference"
                    }
                },
                "required": ["new_location"]
            }),
        }
    }

    // ========================================================================
    // Class Feature Tools
    // ========================================================================

    fn use_rage() -> Tool {
        Tool {
            name: "use_rage".to_string(),
            description: "Barbarian enters a rage. Requires bonus action, can't be wearing heavy armor. Grants: advantage on STR checks/saves, rage damage bonus (+2 at levels 1-8, +3 at 9-15, +4 at 16+), resistance to bludgeoning/piercing/slashing. Lasts 1 minute, ends early if knocked unconscious or turn ends without attacking/taking damage.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        }
    }

    fn end_rage() -> Tool {
        Tool {
            name: "end_rage".to_string(),
            description: "End the barbarian's current rage. Use when: rage duration expires (10 rounds), character is knocked unconscious, or turn ends without attacking or taking damage.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "reason": {
                        "type": "string",
                        "enum": ["duration_expired", "unconscious", "no_combat_action", "voluntary"],
                        "description": "Why the rage is ending"
                    }
                },
                "required": ["reason"]
            }),
        }
    }

    fn use_ki() -> Tool {
        Tool {
            name: "use_ki".to_string(),
            description: "Monk spends ki points (called 'Monk's Focus' in SRD 5.2) to use abilities. Ki points = Monk level, recovered on long rest. Options: Flurry of Blows (1 ki, 2 bonus action unarmed strikes), Patient Defense (1 ki, Dodge as bonus action), Step of the Wind (1 ki, Disengage/Dash as bonus action + double jump), Stunning Strike (1 ki, target must CON save or be stunned).".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "points": {
                        "type": "integer",
                        "minimum": 1,
                        "description": "Number of ki points to spend"
                    },
                    "ability": {
                        "type": "string",
                        "enum": ["flurry_of_blows", "patient_defense", "step_of_the_wind", "stunning_strike", "other"],
                        "description": "The ki ability being used"
                    },
                    "description": {
                        "type": "string",
                        "description": "Description of the action if 'other' is selected"
                    }
                },
                "required": ["points", "ability"]
            }),
        }
    }

    fn use_lay_on_hands() -> Tool {
        Tool {
            name: "use_lay_on_hands".to_string(),
            description: "Paladin uses Lay on Hands to heal or cure. Pool = 5 × Paladin level, recovered on long rest. As an action: restore HP from pool (any amount up to remaining), OR spend 5 HP to cure one disease or neutralize one poison affecting a creature.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "hp_amount": {
                        "type": "integer",
                        "minimum": 0,
                        "description": "HP to restore from the pool (0 if curing disease/poison)"
                    },
                    "cure_disease": {
                        "type": "boolean",
                        "description": "Whether to cure a disease (costs 5 HP from pool)"
                    },
                    "neutralize_poison": {
                        "type": "boolean",
                        "description": "Whether to neutralize a poison (costs 5 HP from pool)"
                    },
                    "target": {
                        "type": "string",
                        "description": "Name of the creature being healed/cured"
                    }
                },
                "required": ["target"]
            }),
        }
    }

    fn use_divine_smite() -> Tool {
        Tool {
            name: "use_divine_smite".to_string(),
            description: "Paladin expends a spell slot to deal extra radiant damage when hitting with a melee weapon attack. Damage: 2d8 + 1d8 per slot level above 1st. Extra 1d8 vs undead or fiends. Maximum 5d8 (or 6d8 vs undead/fiends using 4th level slot).".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "spell_slot_level": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 5,
                        "description": "Level of spell slot to expend"
                    },
                    "target_is_undead_or_fiend": {
                        "type": "boolean",
                        "description": "Whether the target is undead or a fiend (extra 1d8 damage)"
                    }
                },
                "required": ["spell_slot_level"]
            }),
        }
    }

    fn use_wild_shape() -> Tool {
        Tool {
            name: "use_wild_shape".to_string(),
            description: "Druid uses Wild Shape to transform into a beast. Uses: 2 per short/long rest. Duration: hours = half druid level. Max CR: 1/4 (level 2-3), 1/2 (level 4-7), 1 (level 8+). Keep mental stats, proficiencies, features. Can't cast spells while transformed but can concentrate.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "beast_form": {
                        "type": "string",
                        "description": "Name of the beast to transform into (e.g., 'Wolf', 'Brown Bear', 'Giant Spider')"
                    },
                    "beast_hp": {
                        "type": "integer",
                        "minimum": 1,
                        "description": "HP of the beast form"
                    },
                    "beast_ac": {
                        "type": "integer",
                        "description": "AC of the beast form"
                    }
                },
                "required": ["beast_form", "beast_hp"]
            }),
        }
    }

    fn end_wild_shape() -> Tool {
        Tool {
            name: "end_wild_shape".to_string(),
            description: "End the druid's Wild Shape, reverting to normal form. Happens when: duration expires, beast HP drops to 0 (excess damage carries over), voluntarily ended as bonus action, or druid is incapacitated.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "reason": {
                        "type": "string",
                        "enum": ["duration_expired", "hp_zero", "voluntary", "incapacitated"],
                        "description": "Why Wild Shape is ending"
                    },
                    "excess_damage": {
                        "type": "integer",
                        "minimum": 0,
                        "description": "Damage that carries over to normal form if beast HP dropped to 0"
                    }
                },
                "required": ["reason"]
            }),
        }
    }

    fn use_channel_divinity() -> Tool {
        Tool {
            name: "use_channel_divinity".to_string(),
            description: "Cleric or Paladin uses Channel Divinity. Uses: 1 per short/long rest. Cleric options: Turn Undead (undead within 30ft WIS save or flee for 1 minute), Divine Spark (deal/heal 1d8 scaling damage). Paladin options vary by oath.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "option": {
                        "type": "string",
                        "description": "The Channel Divinity option being used (e.g., 'Turn Undead', 'Divine Spark', 'Sacred Weapon')"
                    },
                    "targets": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Names of targets affected"
                    }
                },
                "required": ["option"]
            }),
        }
    }

    fn use_bardic_inspiration() -> Tool {
        Tool {
            name: "use_bardic_inspiration".to_string(),
            description: "Bard grants Bardic Inspiration to a creature. Uses: CHA modifier per long rest (short rest at level 5+). Bonus action to grant one creature a die (d6, d8 at 5th, d10 at 10th, d12 at 15th) they can add to one ability check, attack roll, or saving throw within 10 minutes.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "target": {
                        "type": "string",
                        "description": "Name of the creature receiving inspiration"
                    },
                    "die_size": {
                        "type": "string",
                        "enum": ["d6", "d8", "d10", "d12"],
                        "description": "Size of the inspiration die (based on bard level)"
                    }
                },
                "required": ["target", "die_size"]
            }),
        }
    }

    fn use_action_surge() -> Tool {
        Tool {
            name: "use_action_surge".to_string(),
            description: "Fighter uses Action Surge to take an additional action on their turn. Uses: 1 per short/long rest (2 at level 17). The extra action can be used for any action (Attack, Cast a Spell, Dash, etc.).".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "action_taken": {
                        "type": "string",
                        "description": "What action the fighter takes with Action Surge"
                    }
                },
                "required": ["action_taken"]
            }),
        }
    }

    fn use_second_wind() -> Tool {
        Tool {
            name: "use_second_wind".to_string(),
            description: "Fighter uses Second Wind as a bonus action to regain hit points. Uses: 1 per short/long rest. Healing: 1d10 + fighter level HP.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        }
    }

    fn use_sorcery_points() -> Tool {
        Tool {
            name: "use_sorcery_points".to_string(),
            description: "Sorcerer spends sorcery points for Metamagic or converts between points and spell slots. Points = Sorcerer level (starting at level 2). Metamagic options: Careful (1 pt), Distant (1 pt), Empowered (1 pt), Extended (1 pt), Heightened (3 pts), Quickened (2 pts), Subtle (1 pt), Twinned (spell level pts).".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "points": {
                        "type": "integer",
                        "minimum": 1,
                        "description": "Number of sorcery points to spend"
                    },
                    "metamagic": {
                        "type": "string",
                        "enum": ["careful", "distant", "empowered", "extended", "heightened", "quickened", "subtle", "twinned", "convert_to_slot", "convert_from_slot"],
                        "description": "The Metamagic option or conversion being used"
                    },
                    "spell_name": {
                        "type": "string",
                        "description": "Name of the spell being modified (if using Metamagic)"
                    },
                    "slot_level": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 5,
                        "description": "Spell slot level for conversion (if converting)"
                    }
                },
                "required": ["points", "metamagic"]
            }),
        }
    }

    fn cast_spell() -> Tool {
        Tool {
            name: "cast_spell".to_string(),
            description: "Cast a spell. Handles spell slot consumption, attack rolls, saving throws, and damage/healing. For cantrips (level 0), no spell slot is consumed. For leveled spells, a spell slot of the appropriate level or higher must be available.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "spell_name": {
                        "type": "string",
                        "description": "Name of the spell to cast (e.g., 'Fireball', 'Cure Wounds', 'Fire Bolt')"
                    },
                    "slot_level": {
                        "type": "integer",
                        "minimum": 0,
                        "maximum": 9,
                        "description": "Spell slot level to use. Use 0 for cantrips. Can upcast by using a higher slot than the spell's base level."
                    },
                    "targets": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Names of targets for the spell (for targeted spells)"
                    }
                },
                "required": ["spell_name"]
            }),
        }
    }

    fn death_save() -> Tool {
        Tool {
            name: "death_save".to_string(),
            description: "Make a death saving throw for a character at 0 HP. Roll d20: 10+ = success, <10 = failure, nat 20 = regain 1 HP, nat 1 = 2 failures. 3 successes = stable, 3 failures = death.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        }
    }

    fn concentration_check() -> Tool {
        Tool {
            name: "concentration_check".to_string(),
            description: "Make a concentration check when a concentrating spellcaster takes damage. DC = max(10, damage/2). CON save to maintain concentration.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "damage_taken": {
                        "type": "integer",
                        "description": "Amount of damage taken that triggered the check"
                    },
                    "spell_name": {
                        "type": "string",
                        "description": "Name of the spell being concentrated on"
                    }
                },
                "required": ["damage_taken", "spell_name"]
            }),
        }
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

    fn register_consequence() -> Tool {
        Tool {
            name: "register_consequence".to_string(),
            description: "Register a future consequence based on player actions. Use this when something the player does should have future ramifications - like making an enemy, breaking a law, or triggering a curse. The consequence will be surfaced when relevant conditions arise.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "trigger_description": {
                        "type": "string",
                        "description": "Natural language description of when this consequence should trigger (e.g., 'Player enters Riverside village', 'Player encounters Baron Aldric', 'Player tries to sleep')"
                    },
                    "consequence_description": {
                        "type": "string",
                        "description": "Natural language description of what happens when triggered (e.g., 'Town guards attempt to arrest the player for crimes against the baron', 'The curse drains 1d6 HP')"
                    },
                    "severity": {
                        "type": "string",
                        "enum": ["minor", "moderate", "major", "critical"],
                        "description": "How severe this consequence is. Minor=flavor/inconvenience, Moderate=meaningful impact, Major=significant story impact, Critical=life-threatening"
                    },
                    "related_entities": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Names of entities involved (NPCs, locations, organizations)"
                    },
                    "importance": {
                        "type": "number",
                        "minimum": 0.1,
                        "maximum": 1.0,
                        "description": "How important this consequence is for relevance ranking (0.1-1.0, default based on severity)"
                    },
                    "expires_in_turns": {
                        "type": "integer",
                        "minimum": 1,
                        "description": "Number of turns until this consequence expires (omit for permanent consequences)"
                    }
                },
                "required": ["trigger_description", "consequence_description", "severity"]
            }),
        }
    }

    fn roll_dice() -> Tool {
        Tool {
            name: "roll_dice".to_string(),
            description: "Roll dice using standard D&D notation (e.g., '2d6+3', '1d20', '4d6kh3')."
                .to_string(),
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
            description: "Have a character make a raw ability check (not tied to a skill)."
                .to_string(),
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
            description: "Start a combat encounter. Initiative will be rolled for all combatants. Provide enemy stats based on D&D 5e SRD creatures."
                .to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "enemies": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "name": {
                                    "type": "string",
                                    "description": "Enemy name (e.g., 'Goblin', 'Orc', 'Wolf')"
                                },
                                "max_hp": {
                                    "type": "integer",
                                    "description": "Maximum hit points (e.g., Goblin: 7, Orc: 15, Wolf: 11)"
                                },
                                "armor_class": {
                                    "type": "integer",
                                    "description": "Armor class (e.g., Goblin: 15, Orc: 13, Wolf: 13)"
                                },
                                "initiative_modifier": {
                                    "type": "integer",
                                    "description": "Initiative modifier based on DEX (e.g., Goblin: +2, Orc: +1, Wolf: +2)"
                                }
                            },
                            "required": ["name"]
                        },
                        "description": "List of enemy combatants with their stats"
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

    // Inventory management tools

    fn give_item() -> Tool {
        Tool {
            name: "give_item".to_string(),
            description: "Give an item to the player. Use this when they find loot, receive rewards, or purchase items.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "item_name": {
                        "type": "string",
                        "description": "Name of the item (e.g., 'Longsword', 'Healing Potion', 'Rope')"
                    },
                    "quantity": {
                        "type": "integer",
                        "minimum": 1,
                        "description": "Number of items to give (default 1)"
                    },
                    "item_type": {
                        "type": "string",
                        "enum": ["weapon", "armor", "shield", "potion", "scroll", "wand", "ring", "wondrous", "adventuring", "tool", "other"],
                        "description": "Type of item"
                    },
                    "description": {
                        "type": "string",
                        "description": "Optional description of the item"
                    },
                    "magical": {
                        "type": "boolean",
                        "description": "Whether the item is magical (default false)"
                    },
                    "weight": {
                        "type": "number",
                        "description": "Weight in pounds (optional)"
                    },
                    "value_gp": {
                        "type": "number",
                        "description": "Value in gold pieces (optional)"
                    }
                },
                "required": ["item_name"]
            }),
        }
    }

    fn remove_item() -> Tool {
        Tool {
            name: "remove_item".to_string(),
            description: "Remove an item from the player's inventory. Use when items are consumed, lost, sold, or given away.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "item_name": {
                        "type": "string",
                        "description": "Name of the item to remove"
                    },
                    "quantity": {
                        "type": "integer",
                        "minimum": 1,
                        "description": "Number of items to remove (default 1)"
                    }
                },
                "required": ["item_name"]
            }),
        }
    }

    fn use_item() -> Tool {
        Tool {
            name: "use_item".to_string(),
            description: "Use a consumable item from inventory. Handles potions (healing), scrolls (spells), and other consumables.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "item_name": {
                        "type": "string",
                        "description": "Name of the item to use"
                    },
                    "target": {
                        "type": "string",
                        "enum": ["self", "ally", "enemy"],
                        "description": "Target of the item effect (default 'self')"
                    }
                },
                "required": ["item_name"]
            }),
        }
    }

    fn equip_item() -> Tool {
        Tool {
            name: "equip_item".to_string(),
            description:
                "Equip a weapon, armor, or shield from inventory. Affects AC and attack damage."
                    .to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "item_name": {
                        "type": "string",
                        "description": "Name of the item to equip"
                    }
                },
                "required": ["item_name"]
            }),
        }
    }

    fn unequip_item() -> Tool {
        Tool {
            name: "unequip_item".to_string(),
            description: "Unequip an item from a slot and return it to inventory.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "slot": {
                        "type": "string",
                        "enum": ["armor", "shield", "main_hand", "off_hand"],
                        "description": "Equipment slot to unequip from"
                    }
                },
                "required": ["slot"]
            }),
        }
    }

    fn adjust_gold() -> Tool {
        Tool {
            name: "adjust_gold".to_string(),
            description: "Add or remove gold from the player. Positive values add gold, negative values remove it.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "amount": {
                        "type": "number",
                        "description": "Amount of gold to add (positive) or remove (negative)"
                    },
                    "reason": {
                        "type": "string",
                        "description": "Reason for the gold change (e.g., 'looting chest', 'buying supplies', 'quest reward')"
                    }
                },
                "required": ["amount"]
            }),
        }
    }

    fn show_inventory() -> Tool {
        Tool {
            name: "show_inventory".to_string(),
            description: "Display the player's current inventory, equipment, and gold. Use this to check what items they have.".to_string(),
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
            // Validate damage is positive
            if amount <= 0 {
                return None;
            }
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
            // Validate healing is positive
            if amount <= 0 {
                return None;
            }
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
                armor_class: world.player_character.current_ac(),
                initiative_modifier: world.player_character.initiative_modifier(),
            }];

            for enemy in enemies {
                let name = enemy["name"].as_str().unwrap_or("Enemy").to_string();
                // Parse enemy HP if provided, default to 10/10 for basic enemies
                let max_hp = enemy["max_hp"].as_i64().unwrap_or(10) as i32;
                let current_hp = enemy["current_hp"].as_i64().unwrap_or(max_hp as i64) as i32;
                // Parse enemy AC if provided, default to 10 (unarmored)
                let armor_class = enemy["armor_class"].as_u64().unwrap_or(10) as u8;
                // Parse initiative modifier if provided, default to 0
                let initiative_modifier = enemy["initiative_modifier"].as_i64().unwrap_or(0) as i8;
                combatants.push(CombatantInit {
                    id: CharacterId::new(),
                    name,
                    is_player: false,
                    is_ally: false,
                    current_hp,
                    max_hp,
                    armor_class,
                    initiative_modifier,
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
        "register_consequence" => {
            let trigger_description = input["trigger_description"].as_str()?.to_string();
            let consequence_description = input["consequence_description"].as_str()?.to_string();
            let severity = input["severity"].as_str()?.to_string();
            let related_entities = input["related_entities"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();
            // Default importance based on severity
            let default_importance = match severity.as_str() {
                "minor" => 0.3,
                "moderate" => 0.5,
                "major" => 0.8,
                "critical" => 1.0,
                _ => 0.5,
            };
            let importance = input["importance"].as_f64().unwrap_or(default_importance) as f32;
            let expires_in_turns = input["expires_in_turns"].as_u64().map(|v| v as u32);

            Some(Intent::RegisterConsequence {
                trigger_description,
                consequence_description,
                severity,
                related_entities,
                importance,
                expires_in_turns,
            })
        }
        // Inventory tools
        "give_item" => {
            let item_name = input["item_name"].as_str()?.to_string();
            let quantity = input["quantity"].as_u64().unwrap_or(1) as u32;
            let item_type = input["item_type"].as_str().map(|s| s.to_string());
            let description = input["description"].as_str().map(|s| s.to_string());
            let magical = input["magical"].as_bool().unwrap_or(false);
            let weight = input["weight"].as_f64().map(|w| w as f32);
            let value_gp = input["value_gp"].as_f64().map(|v| v as f32);

            Some(Intent::AddItem {
                item_name,
                quantity,
                item_type,
                description,
                magical,
                weight,
                value_gp,
            })
        }
        "remove_item" => {
            let item_name = input["item_name"].as_str()?.to_string();
            let quantity = input["quantity"].as_u64().unwrap_or(1) as u32;

            Some(Intent::RemoveItem {
                item_name,
                quantity,
            })
        }
        "use_item" => {
            let item_name = input["item_name"].as_str()?.to_string();
            // Target handling would be more sophisticated in a full implementation
            let target_id = None;

            Some(Intent::UseItem {
                item_name,
                target_id,
            })
        }
        "equip_item" => {
            let item_name = input["item_name"].as_str()?.to_string();
            Some(Intent::EquipItem { item_name })
        }
        "unequip_item" => {
            let slot = input["slot"].as_str()?.to_string();
            Some(Intent::UnequipItem { slot })
        }
        "adjust_gold" => {
            let amount = input["amount"].as_f64()? as f32;
            let reason = input["reason"]
                .as_str()
                .unwrap_or("gold adjustment")
                .to_string();
            Some(Intent::AdjustGold { amount, reason })
        }
        "death_save" => Some(Intent::DeathSave {
            character_id: world.player_character.id,
        }),
        "concentration_check" => {
            let damage_taken = input["damage_taken"].as_i64()? as i32;
            let spell_name = input["spell_name"].as_str()?.to_string();
            Some(Intent::ConcentrationCheck {
                character_id: world.player_character.id,
                damage_taken,
                spell_name,
            })
        }
        "change_location" => {
            let new_location = input["new_location"].as_str()?.to_string();
            let location_type = input["location_type"].as_str().map(|s| s.to_string());
            let description = input["description"].as_str().map(|s| s.to_string());
            Some(Intent::ChangeLocation {
                new_location,
                location_type,
                description,
            })
        }

        // Class feature tools
        "use_rage" => Some(Intent::UseRage {
            character_id: world.player_character.id,
        }),
        "end_rage" => {
            let reason = input["reason"].as_str().unwrap_or("voluntary").to_string();
            Some(Intent::EndRage {
                character_id: world.player_character.id,
                reason,
            })
        }
        "use_ki" => {
            let points = input["points"].as_u64()? as u8;
            let ability = input["ability"].as_str()?.to_string();
            Some(Intent::UseKi {
                character_id: world.player_character.id,
                points,
                ability,
            })
        }
        "use_lay_on_hands" => {
            let target_name = input["target"].as_str()?.to_string();
            let hp_amount = input["hp_amount"].as_u64().unwrap_or(0) as u32;
            let cure_disease = input["cure_disease"].as_bool().unwrap_or(false);
            let neutralize_poison = input["neutralize_poison"].as_bool().unwrap_or(false);
            Some(Intent::UseLayOnHands {
                character_id: world.player_character.id,
                target_name,
                hp_amount,
                cure_disease,
                neutralize_poison,
            })
        }
        "use_divine_smite" => {
            let spell_slot_level = input["spell_slot_level"].as_u64()? as u8;
            let target_is_undead_or_fiend = input["target_is_undead_or_fiend"]
                .as_bool()
                .unwrap_or(false);
            Some(Intent::UseDivineSmite {
                character_id: world.player_character.id,
                spell_slot_level,
                target_is_undead_or_fiend,
            })
        }
        "use_wild_shape" => {
            let beast_form = input["beast_form"].as_str()?.to_string();
            let beast_hp = input["beast_hp"].as_i64()? as i32;
            let beast_ac = input["beast_ac"].as_u64().map(|ac| ac as u8);
            Some(Intent::UseWildShape {
                character_id: world.player_character.id,
                beast_form,
                beast_hp,
                beast_ac,
            })
        }
        "end_wild_shape" => {
            let reason = input["reason"].as_str()?.to_string();
            let excess_damage = input["excess_damage"].as_i64().unwrap_or(0) as i32;
            Some(Intent::EndWildShape {
                character_id: world.player_character.id,
                reason,
                excess_damage,
            })
        }
        "use_channel_divinity" => {
            let option = input["option"].as_str()?.to_string();
            let targets = input["targets"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();
            Some(Intent::UseChannelDivinity {
                character_id: world.player_character.id,
                option,
                targets,
            })
        }
        "use_bardic_inspiration" => {
            let target_name = input["target"].as_str()?.to_string();
            let die_size = input["die_size"].as_str()?.to_string();
            Some(Intent::UseBardicInspiration {
                character_id: world.player_character.id,
                target_name,
                die_size,
            })
        }
        "use_action_surge" => {
            let action_taken = input["action_taken"].as_str()?.to_string();
            Some(Intent::UseActionSurge {
                character_id: world.player_character.id,
                action_taken,
            })
        }
        "use_second_wind" => Some(Intent::UseSecondWind {
            character_id: world.player_character.id,
        }),
        "use_sorcery_points" => {
            let points = input["points"].as_u64()? as u8;
            let metamagic = input["metamagic"].as_str()?.to_string();
            let spell_name = input["spell_name"].as_str().map(|s| s.to_string());
            let slot_level = input["slot_level"].as_u64().map(|l| l as u8);
            Some(Intent::UseSorceryPoints {
                character_id: world.player_character.id,
                points,
                metamagic,
                spell_name,
                slot_level,
            })
        }

        "cast_spell" => {
            let spell_name = input["spell_name"].as_str()?.to_string();
            let slot_level = input["slot_level"].as_u64().unwrap_or(0) as u8;
            let targets = input["targets"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();

            Some(Intent::CastSpell {
                caster_id: world.player_character.id,
                spell_name,
                targets: vec![], // We pass target names separately
                spell_level: slot_level,
                target_names: targets,
            })
        }

        // show_inventory is handled specially via execute_info_tool
        _ => None,
    }
}

/// Execute an informational tool that returns data without creating an Intent.
/// Returns Some(result_string) if the tool is an info tool, None otherwise.
pub fn execute_info_tool(name: &str, _input: &Value, world: &GameWorld) -> Option<String> {
    match name {
        "show_inventory" => Some(format_inventory(world)),
        _ => None,
    }
}

/// Format the player's inventory for display.
fn format_inventory(world: &GameWorld) -> String {
    let character = &world.player_character;
    let mut output = String::new();

    output.push_str(&format!("=== {}'s Inventory ===\n\n", character.name));

    // Gold
    output.push_str(&format!("Gold: {:.0} gp\n\n", character.inventory.gold));

    // Current AC
    output.push_str(&format!("Current AC: {}\n\n", character.current_ac()));

    // Equipment
    output.push_str("Equipment:\n");
    if let Some(ref armor) = character.equipment.armor {
        let armor_type_str = match armor.armor_type {
            crate::world::ArmorType::Light => "Light",
            crate::world::ArmorType::Medium => "Medium",
            crate::world::ArmorType::Heavy => "Heavy",
        };
        let stealth_str = if armor.stealth_disadvantage {
            " [Stealth Disadvantage]"
        } else {
            ""
        };
        output.push_str(&format!(
            "  Armor: {} ({} armor, base AC {}){}\n",
            armor.base.name, armor_type_str, armor.base_ac, stealth_str
        ));
    } else {
        output.push_str("  Armor: None (unarmored)\n");
    }
    if let Some(ref shield) = character.equipment.shield {
        output.push_str(&format!("  Shield: {} (+2 AC)\n", shield.name));
    } else {
        output.push_str("  Shield: None\n");
    }
    if let Some(ref weapon) = character.equipment.main_hand {
        let two_handed = if weapon.is_two_handed() {
            " [Two-Handed]"
        } else {
            ""
        };
        output.push_str(&format!(
            "  Main Hand: {} ({} {}){}\n",
            weapon.base.name,
            weapon.damage_dice,
            weapon.damage_type.name(),
            two_handed
        ));
    } else {
        output.push_str("  Main Hand: Empty\n");
    }
    if let Some(ref item) = character.equipment.off_hand {
        output.push_str(&format!("  Off Hand: {}\n", item.name));
    }

    // Inventory items
    if character.inventory.items.is_empty() {
        output.push_str("\nInventory: Empty\n");
    } else {
        output.push_str("\nInventory:\n");
        for item in &character.inventory.items {
            let qty_str = if item.quantity > 1 {
                format!(" (x{})", item.quantity)
            } else {
                String::new()
            };
            let value_str = if item.value_gp > 0.0 {
                format!(" [{:.0} gp]", item.value_gp)
            } else {
                String::new()
            };
            output.push_str(&format!("  - {}{}{}\n", item.name, qty_str, value_str));
        }
    }

    output.push_str(&format!(
        "\nTotal Weight: {:.1} lb\n",
        character.inventory.total_weight()
    ));

    output
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::{Character, CharacterClass, ClassLevel, GameWorld};

    fn create_test_world() -> GameWorld {
        let mut character = Character::new("Test Hero");
        character.classes.push(ClassLevel {
            class: CharacterClass::Fighter,
            level: 1,
            subclass: None,
        });
        GameWorld::new("Test Campaign", character)
    }

    #[test]
    fn test_all_tools_have_valid_schemas() {
        let tools = DmTools::all();
        assert!(!tools.is_empty(), "Should have at least one tool");

        for tool in &tools {
            assert!(!tool.name.is_empty(), "Tool name should not be empty");
            assert!(
                !tool.description.is_empty(),
                "Tool {} should have a description",
                tool.name
            );
            assert!(
                tool.input_schema.get("type").is_some(),
                "Tool {} should have a type in schema",
                tool.name
            );
        }
    }

    #[test]
    fn test_tool_count() {
        let tools = DmTools::all();
        // Count all tools - should match the number in DmTools::all()
        assert!(
            tools.len() >= 30,
            "Should have at least 30 tools, got {}",
            tools.len()
        );
    }

    #[test]
    fn test_roll_dice_tool_schema() {
        let tools = DmTools::all();
        let roll_dice = tools.iter().find(|t| t.name == "roll_dice").unwrap();

        let props = roll_dice.input_schema["properties"].as_object().unwrap();
        assert!(
            props.contains_key("notation"),
            "roll_dice should have 'notation' property"
        );
        assert!(
            props.contains_key("purpose"),
            "roll_dice should have 'purpose' property"
        );

        let required = roll_dice.input_schema["required"].as_array().unwrap();
        assert!(
            required.iter().any(|v| v.as_str() == Some("notation")),
            "roll_dice should require 'notation'"
        );
    }

    #[test]
    fn test_skill_check_tool_schema() {
        let tools = DmTools::all();
        let skill_check = tools.iter().find(|t| t.name == "skill_check").unwrap();

        let props = skill_check.input_schema["properties"].as_object().unwrap();
        assert!(
            props.contains_key("skill"),
            "skill_check should have 'skill' property"
        );
        assert!(
            props.contains_key("dc"),
            "skill_check should have 'dc' property"
        );
    }

    #[test]
    fn test_apply_damage_tool_schema() {
        let tools = DmTools::all();
        let apply_damage = tools.iter().find(|t| t.name == "apply_damage").unwrap();

        let props = apply_damage.input_schema["properties"].as_object().unwrap();
        assert!(
            props.contains_key("amount"),
            "apply_damage should have 'amount' property"
        );
        assert!(
            props.contains_key("damage_type"),
            "apply_damage should have 'damage_type' property"
        );
    }

    #[test]
    fn test_parse_tool_call_roll_dice() {
        let world = create_test_world();
        let input = json!({
            "notation": "2d6+3",
            "purpose": "attack damage"
        });

        let intent = parse_tool_call("roll_dice", &input, &world);
        assert!(intent.is_some());

        if let Some(Intent::RollDice { notation, purpose }) = intent {
            assert_eq!(notation, "2d6+3");
            assert_eq!(purpose, "attack damage");
        } else {
            panic!("Expected RollDice intent");
        }
    }

    #[test]
    fn test_parse_tool_call_skill_check() {
        let world = create_test_world();
        let input = json!({
            "skill": "athletics",
            "dc": 15,
            "description": "climbing the wall"
        });

        let intent = parse_tool_call("skill_check", &input, &world);
        assert!(intent.is_some());

        if let Some(Intent::SkillCheck {
            skill,
            dc,
            description,
            ..
        }) = intent
        {
            assert_eq!(skill, Skill::Athletics);
            assert_eq!(dc, 15);
            assert_eq!(description, "climbing the wall");
        } else {
            panic!("Expected SkillCheck intent");
        }
    }

    #[test]
    fn test_parse_tool_call_apply_damage() {
        let world = create_test_world();
        let input = json!({
            "amount": 10,
            "damage_type": "slashing",
            "source": "sword"
        });

        let intent = parse_tool_call("apply_damage", &input, &world);
        assert!(intent.is_some());

        if let Some(Intent::Damage {
            amount,
            damage_type,
            source,
            ..
        }) = intent
        {
            assert_eq!(amount, 10);
            assert_eq!(damage_type, DamageType::Slashing);
            assert_eq!(source, "sword");
        } else {
            panic!("Expected Damage intent");
        }
    }

    #[test]
    fn test_parse_tool_call_invalid_damage_amount() {
        let world = create_test_world();
        let input = json!({
            "amount": 0,
            "damage_type": "slashing",
            "source": "sword"
        });

        let intent = parse_tool_call("apply_damage", &input, &world);
        assert!(intent.is_none(), "Should reject zero damage");

        let input = json!({
            "amount": -5,
            "damage_type": "slashing",
            "source": "sword"
        });

        let intent = parse_tool_call("apply_damage", &input, &world);
        assert!(intent.is_none(), "Should reject negative damage");
    }

    #[test]
    fn test_parse_tool_call_apply_healing() {
        let world = create_test_world();
        let input = json!({
            "amount": 8,
            "source": "potion"
        });

        let intent = parse_tool_call("apply_healing", &input, &world);
        assert!(intent.is_some());

        if let Some(Intent::Heal { amount, source, .. }) = intent {
            assert_eq!(amount, 8);
            assert_eq!(source, "potion");
        } else {
            panic!("Expected Heal intent");
        }
    }

    #[test]
    fn test_parse_tool_call_invalid_healing_amount() {
        let world = create_test_world();
        let input = json!({
            "amount": 0,
            "source": "potion"
        });

        let intent = parse_tool_call("apply_healing", &input, &world);
        assert!(intent.is_none(), "Should reject zero healing");
    }

    #[test]
    fn test_parse_tool_call_apply_condition() {
        let world = create_test_world();
        let input = json!({
            "condition": "poisoned",
            "source": "trap",
            "duration_rounds": 3
        });

        let intent = parse_tool_call("apply_condition", &input, &world);
        assert!(intent.is_some());

        if let Some(Intent::ApplyCondition {
            condition, source, ..
        }) = intent
        {
            assert_eq!(condition, Condition::Poisoned);
            assert_eq!(source, "trap");
        } else {
            panic!("Expected ApplyCondition intent");
        }
    }

    #[test]
    fn test_parse_tool_call_unknown_tool() {
        let world = create_test_world();
        let input = json!({});

        let intent = parse_tool_call("unknown_tool", &input, &world);
        assert!(intent.is_none());
    }

    #[test]
    fn test_parse_ability() {
        assert_eq!(parse_ability("strength"), Some(Ability::Strength));
        assert_eq!(parse_ability("str"), Some(Ability::Strength));
        assert_eq!(parse_ability("STR"), Some(Ability::Strength));
        assert_eq!(parse_ability("dexterity"), Some(Ability::Dexterity));
        assert_eq!(parse_ability("dex"), Some(Ability::Dexterity));
        assert_eq!(parse_ability("constitution"), Some(Ability::Constitution));
        assert_eq!(parse_ability("con"), Some(Ability::Constitution));
        assert_eq!(parse_ability("intelligence"), Some(Ability::Intelligence));
        assert_eq!(parse_ability("int"), Some(Ability::Intelligence));
        assert_eq!(parse_ability("wisdom"), Some(Ability::Wisdom));
        assert_eq!(parse_ability("wis"), Some(Ability::Wisdom));
        assert_eq!(parse_ability("charisma"), Some(Ability::Charisma));
        assert_eq!(parse_ability("cha"), Some(Ability::Charisma));
        assert_eq!(parse_ability("invalid"), None);
    }

    #[test]
    fn test_parse_skill() {
        assert_eq!(parse_skill("athletics"), Some(Skill::Athletics));
        assert_eq!(parse_skill("stealth"), Some(Skill::Stealth));
        assert_eq!(parse_skill("perception"), Some(Skill::Perception));
        assert_eq!(parse_skill("persuasion"), Some(Skill::Persuasion));
        assert_eq!(parse_skill("invalid"), None);
    }

    #[test]
    fn test_parse_advantage() {
        assert_eq!(parse_advantage(Some("advantage")), Advantage::Advantage);
        assert_eq!(
            parse_advantage(Some("disadvantage")),
            Advantage::Disadvantage
        );
        assert_eq!(parse_advantage(Some("normal")), Advantage::Normal);
        assert_eq!(parse_advantage(None), Advantage::Normal);
    }

    #[test]
    fn test_parse_damage_type() {
        assert_eq!(parse_damage_type("slashing"), Some(DamageType::Slashing));
        assert_eq!(parse_damage_type("SLASHING"), Some(DamageType::Slashing));
        assert_eq!(parse_damage_type("piercing"), Some(DamageType::Piercing));
        assert_eq!(
            parse_damage_type("bludgeoning"),
            Some(DamageType::Bludgeoning)
        );
        assert_eq!(parse_damage_type("fire"), Some(DamageType::Fire));
        assert_eq!(parse_damage_type("cold"), Some(DamageType::Cold));
        assert_eq!(parse_damage_type("lightning"), Some(DamageType::Lightning));
        assert_eq!(parse_damage_type("psychic"), Some(DamageType::Psychic));
        assert_eq!(parse_damage_type("invalid"), None);
    }

    #[test]
    fn test_parse_condition() {
        assert_eq!(parse_condition("blinded"), Some(Condition::Blinded));
        assert_eq!(parse_condition("BLINDED"), Some(Condition::Blinded));
        assert_eq!(parse_condition("charmed"), Some(Condition::Charmed));
        assert_eq!(parse_condition("frightened"), Some(Condition::Frightened));
        assert_eq!(parse_condition("grappled"), Some(Condition::Grappled));
        assert_eq!(
            parse_condition("incapacitated"),
            Some(Condition::Incapacitated)
        );
        assert_eq!(parse_condition("invisible"), Some(Condition::Invisible));
        assert_eq!(parse_condition("paralyzed"), Some(Condition::Paralyzed));
        assert_eq!(parse_condition("poisoned"), Some(Condition::Poisoned));
        assert_eq!(parse_condition("prone"), Some(Condition::Prone));
        assert_eq!(parse_condition("stunned"), Some(Condition::Stunned));
        assert_eq!(parse_condition("unconscious"), Some(Condition::Unconscious));
        assert_eq!(parse_condition("invalid"), None);
    }

    #[test]
    fn test_info_tool_show_inventory() {
        let world = create_test_world();
        let input = json!({});

        let result = execute_info_tool("show_inventory", &input, &world);
        assert!(result.is_some());

        let inventory = result.unwrap();
        assert!(inventory.contains("Gold"));
    }
}
