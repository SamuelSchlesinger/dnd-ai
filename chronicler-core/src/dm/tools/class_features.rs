//! Class-specific feature tools (Barbarian, Monk, Paladin, etc.).

use chronicler_llm::Tool;
use serde_json::json;

/// Barbarian enters a rage.
pub fn use_rage() -> Tool {
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

/// End the barbarian's current rage.
pub fn end_rage() -> Tool {
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

/// Monk spends ki points.
pub fn use_ki() -> Tool {
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

/// Paladin uses Lay on Hands.
pub fn use_lay_on_hands() -> Tool {
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

/// Paladin uses Divine Smite.
pub fn use_divine_smite() -> Tool {
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

/// Druid uses Wild Shape.
pub fn use_wild_shape() -> Tool {
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

/// End the druid's Wild Shape.
pub fn end_wild_shape() -> Tool {
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

/// Cleric or Paladin uses Channel Divinity.
pub fn use_channel_divinity() -> Tool {
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

/// Bard grants Bardic Inspiration.
pub fn use_bardic_inspiration() -> Tool {
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

/// Fighter uses Action Surge.
pub fn use_action_surge() -> Tool {
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

/// Fighter uses Second Wind.
pub fn use_second_wind() -> Tool {
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

/// Sorcerer spends sorcery points.
pub fn use_sorcery_points() -> Tool {
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
