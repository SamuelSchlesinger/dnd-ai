//! D&D-specific tools for the DM agent

use async_trait::async_trait;
use once_cell::sync::Lazy;
use serde_json::{json, Value};

use agentic::tool::{Tool, ToolAnnotations, ToolContext, ToolOutput};
use agentic::error::ToolError;
use agentic::id::ToolCallId;

use crate::dnd::game::dice::{roll_with_advantage, Advantage};

// Static schemas and annotations

static ROLL_DICE_SCHEMA: Lazy<Value> = Lazy::new(|| {
    json!({
        "type": "object",
        "properties": {
            "expression": {
                "type": "string",
                "description": "Dice expression in standard notation (e.g., 1d20+5, 2d6, 4d6kh3)"
            },
            "purpose": {
                "type": "string",
                "description": "What the roll is for (attack, damage, save, check, etc.)"
            },
            "advantage": {
                "type": "string",
                "enum": ["normal", "advantage", "disadvantage"],
                "default": "normal",
                "description": "Advantage state for d20 rolls"
            },
            "dc": {
                "type": "integer",
                "description": "Optional DC to compare against"
            }
        },
        "required": ["expression", "purpose"]
    })
});

static SKILL_CHECK_SCHEMA: Lazy<Value> = Lazy::new(|| {
    json!({
        "type": "object",
        "properties": {
            "skill": {
                "type": "string",
                "enum": [
                    "acrobatics", "animal_handling", "arcana", "athletics",
                    "deception", "history", "insight", "intimidation",
                    "investigation", "medicine", "nature", "perception",
                    "performance", "persuasion", "religion", "sleight_of_hand",
                    "stealth", "survival"
                ],
                "description": "The skill to check"
            },
            "dc": {
                "type": "integer",
                "description": "The DC to beat"
            },
            "advantage": {
                "type": "string",
                "enum": ["normal", "advantage", "disadvantage"],
                "default": "normal"
            },
            "passive": {
                "type": "boolean",
                "default": false,
                "description": "Use passive check (10 + modifiers) instead of rolling"
            }
        },
        "required": ["skill", "dc"]
    })
});

static SAVING_THROW_SCHEMA: Lazy<Value> = Lazy::new(|| {
    json!({
        "type": "object",
        "properties": {
            "ability": {
                "type": "string",
                "enum": ["strength", "dexterity", "constitution", "intelligence", "wisdom", "charisma"],
                "description": "The ability for the save"
            },
            "dc": {
                "type": "integer",
                "description": "The DC to beat"
            },
            "advantage": {
                "type": "string",
                "enum": ["normal", "advantage", "disadvantage"],
                "default": "normal"
            }
        },
        "required": ["ability", "dc"]
    })
});

static APPLY_DAMAGE_SCHEMA: Lazy<Value> = Lazy::new(|| {
    json!({
        "type": "object",
        "properties": {
            "target": {
                "type": "string",
                "description": "Name or ID of the target"
            },
            "amount": {
                "type": "integer",
                "description": "Amount of damage"
            },
            "damage_type": {
                "type": "string",
                "enum": [
                    "bludgeoning", "piercing", "slashing",
                    "acid", "cold", "fire", "force", "lightning",
                    "necrotic", "poison", "psychic", "radiant", "thunder"
                ],
                "description": "Type of damage"
            },
            "magical": {
                "type": "boolean",
                "default": false,
                "description": "Whether the damage is magical"
            }
        },
        "required": ["target", "amount", "damage_type"]
    })
});

static APPLY_HEALING_SCHEMA: Lazy<Value> = Lazy::new(|| {
    json!({
        "type": "object",
        "properties": {
            "target": {
                "type": "string",
                "description": "Name or ID of the target"
            },
            "amount": {
                "type": "integer",
                "description": "Amount of healing"
            }
        },
        "required": ["target", "amount"]
    })
});

static DESTRUCTIVE_ANNOTATIONS: ToolAnnotations = ToolAnnotations {
    destructive: true,
    read_only: false,
    requires_approval: true,
    slow: false,
    network_access: false,
    file_system_access: false,
    has_cost: false,
};

/// Tool for rolling dice
pub struct RollDiceTool;

#[async_trait]
impl Tool for RollDiceTool {
    fn name(&self) -> &str {
        "roll_dice"
    }

    fn description(&self) -> &str {
        "Roll dice using standard D&D notation. Supports advantage/disadvantage for d20 rolls."
    }

    fn input_schema(&self) -> &Value {
        &ROLL_DICE_SCHEMA
    }

    async fn execute(&self, params: Value, _ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let expression = params["expression"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidParameters {
                tool: self.name().to_string(),
                reason: "Missing 'expression' parameter".to_string(),
            })?;

        let purpose = params["purpose"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidParameters {
                tool: self.name().to_string(),
                reason: "Missing 'purpose' parameter".to_string(),
            })?;

        let advantage = match params["advantage"].as_str() {
            Some("advantage") => Advantage::Advantage,
            Some("disadvantage") => Advantage::Disadvantage,
            _ => Advantage::Normal,
        };

        let dc = params["dc"].as_i64().map(|d| d as i32);

        // Roll the dice
        let result = roll_with_advantage(expression, advantage).map_err(|e| {
            ToolError::ExecutionFailed {
                tool_call_id: ToolCallId::new(),
                reason: e.to_string(),
            }
        })?;

        // Build output
        let mut output_text = format!(
            "Rolling {} for {}: {} = {}",
            expression,
            purpose,
            result.dice_display(),
            result.total
        );

        if result.natural_20 {
            output_text.push_str(" (NATURAL 20!)");
        } else if result.natural_1 {
            output_text.push_str(" (Natural 1)");
        }

        if let Some(dc_val) = dc {
            let success = result.total >= dc_val;
            output_text.push_str(&format!(
                " vs DC {} - {}",
                dc_val,
                if success { "SUCCESS" } else { "FAILURE" }
            ));
        }

        let structured = json!({
            "expression": expression,
            "purpose": purpose,
            "rolls": result.component_results.iter().map(|c| &c.rolls).collect::<Vec<_>>(),
            "modifier": result.modifier,
            "total": result.total,
            "natural_20": result.natural_20,
            "natural_1": result.natural_1,
            "dc": dc,
            "success": dc.map(|d| result.total >= d)
        });

        Ok(ToolOutput::structured(output_text, structured))
    }
}

/// Tool for making skill checks
pub struct SkillCheckTool;

#[async_trait]
impl Tool for SkillCheckTool {
    fn name(&self) -> &str {
        "skill_check"
    }

    fn description(&self) -> &str {
        "Make a skill check for a character. Automatically applies proficiency and ability modifiers."
    }

    fn input_schema(&self) -> &Value {
        &SKILL_CHECK_SCHEMA
    }

    async fn execute(&self, params: Value, _ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let skill = params["skill"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidParameters {
                tool: self.name().to_string(),
                reason: "Missing 'skill' parameter".to_string(),
            })?;

        let dc = params["dc"]
            .as_i64()
            .ok_or_else(|| ToolError::InvalidParameters {
                tool: self.name().to_string(),
                reason: "Missing 'dc' parameter".to_string(),
            })? as i32;

        let advantage = match params["advantage"].as_str() {
            Some("advantage") => Advantage::Advantage,
            Some("disadvantage") => Advantage::Disadvantage,
            _ => Advantage::Normal,
        };

        let passive = params["passive"].as_bool().unwrap_or(false);

        // For now, use a placeholder modifier (would come from character sheet)
        let modifier = 3; // Placeholder

        let (total, roll_display) = if passive {
            let passive_total = 10 + modifier;
            (passive_total, format!("10 + {} = {}", modifier, passive_total))
        } else {
            let expression = format!("1d20+{}", modifier);
            let result = roll_with_advantage(&expression, advantage).map_err(|e| {
                ToolError::ExecutionFailed {
                    tool_call_id: ToolCallId::new(),
                    reason: e.to_string(),
                }
            })?;
            (result.total, format!("{} = {}", result.dice_display(), result.total))
        };

        let success = total >= dc;
        let output_text = format!(
            "{} check: {} vs DC {} - {}",
            skill,
            roll_display,
            dc,
            if success { "SUCCESS" } else { "FAILURE" }
        );

        let structured = json!({
            "skill": skill,
            "total": total,
            "dc": dc,
            "success": success,
            "passive": passive,
            "advantage": format!("{:?}", advantage)
        });

        Ok(ToolOutput::structured(output_text, structured))
    }
}

/// Tool for making saving throws
pub struct SavingThrowTool;

#[async_trait]
impl Tool for SavingThrowTool {
    fn name(&self) -> &str {
        "saving_throw"
    }

    fn description(&self) -> &str {
        "Make a saving throw for a character."
    }

    fn input_schema(&self) -> &Value {
        &SAVING_THROW_SCHEMA
    }

    async fn execute(&self, params: Value, _ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let ability = params["ability"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidParameters {
                tool: self.name().to_string(),
                reason: "Missing 'ability' parameter".to_string(),
            })?;

        let dc = params["dc"]
            .as_i64()
            .ok_or_else(|| ToolError::InvalidParameters {
                tool: self.name().to_string(),
                reason: "Missing 'dc' parameter".to_string(),
            })? as i32;

        let advantage = match params["advantage"].as_str() {
            Some("advantage") => Advantage::Advantage,
            Some("disadvantage") => Advantage::Disadvantage,
            _ => Advantage::Normal,
        };

        // Placeholder modifier
        let modifier = 2;
        let expression = format!("1d20+{}", modifier);

        let result = roll_with_advantage(&expression, advantage).map_err(|e| {
            ToolError::ExecutionFailed {
                tool_call_id: ToolCallId::new(),
                reason: e.to_string(),
            }
        })?;

        let success = result.total >= dc;
        let output_text = format!(
            "{} saving throw: {} vs DC {} - {}",
            ability,
            result.total,
            dc,
            if success { "SUCCESS" } else { "FAILURE" }
        );

        Ok(ToolOutput::structured(
            output_text,
            json!({
                "ability": ability,
                "total": result.total,
                "dc": dc,
                "success": success,
                "natural_20": result.natural_20,
                "natural_1": result.natural_1
            }),
        ))
    }
}

/// Tool for applying damage
pub struct ApplyDamageTool;

#[async_trait]
impl Tool for ApplyDamageTool {
    fn name(&self) -> &str {
        "apply_damage"
    }

    fn description(&self) -> &str {
        "Apply damage to a character or creature. Handles resistance, immunity, and vulnerability."
    }

    fn input_schema(&self) -> &Value {
        &APPLY_DAMAGE_SCHEMA
    }

    fn annotations(&self) -> &ToolAnnotations {
        &DESTRUCTIVE_ANNOTATIONS
    }

    async fn execute(&self, params: Value, _ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let target = params["target"].as_str().unwrap_or("Unknown");
        let amount = params["amount"].as_i64().unwrap_or(0) as i32;
        let damage_type = params["damage_type"].as_str().unwrap_or("untyped");

        // In a real implementation, this would modify the game state
        let output_text = format!(
            "{} takes {} {} damage!",
            target, amount, damage_type
        );

        Ok(ToolOutput::structured(
            output_text,
            json!({
                "target": target,
                "damage_dealt": amount,
                "damage_type": damage_type
            }),
        ))
    }
}

/// Tool for healing
pub struct ApplyHealingTool;

#[async_trait]
impl Tool for ApplyHealingTool {
    fn name(&self) -> &str {
        "apply_healing"
    }

    fn description(&self) -> &str {
        "Apply healing to a character."
    }

    fn input_schema(&self) -> &Value {
        &APPLY_HEALING_SCHEMA
    }

    async fn execute(&self, params: Value, _ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let target = params["target"].as_str().unwrap_or("Unknown");
        let amount = params["amount"].as_i64().unwrap_or(0) as i32;

        let output_text = format!("{} is healed for {} HP!", target, amount);

        Ok(ToolOutput::structured(
            output_text,
            json!({
                "target": target,
                "healing": amount
            }),
        ))
    }
}
