//! D&D tools for the AI Dungeon Master.
//!
//! These tools allow the AI to interact with game mechanics
//! by generating Intents that the RulesEngine resolves.
//!
//! Tools are organized into categories:
//! - `checks` - Dice rolls, skill checks, ability checks, saving throws
//! - `combat` - Damage, healing, conditions, combat flow
//! - `inventory` - Items and currency management
//! - `class_features` - Class-specific abilities (rage, ki, smite, etc.)
//! - `world` - Rest, story memory, spells, progression, time, ability scores
//! - `locations` - Location creation, connections, and state updates
//! - `npc` - NPC creation, updates, movement, and removal
//! - `quests` - Quest creation, objectives, and completion tracking
//! - `state` - Declarative state assertions (disposition, location, status, relationships)

mod checks;
mod class_features;
mod combat;
mod converters;
mod info;
mod inventory;
mod knowledge;
mod locations;
mod npc;
mod parsing;
mod quests;
mod schedule;
mod state;
mod world;

pub use info::execute_info_tool_with_memory;
pub use parsing::parse_tool_call;

use chronicler_llm::Tool;

/// Collection of D&D tools for the DM.
pub struct DmTools;

impl DmTools {
    /// Get all tool definitions for the configured LLM API.
    pub fn all() -> Vec<Tool> {
        vec![
            // Checks
            checks::roll_dice(),
            checks::skill_check(),
            checks::ability_check(),
            checks::saving_throw(),
            // Combat
            combat::attack(),
            combat::apply_damage(),
            combat::apply_healing(),
            combat::apply_condition(),
            combat::remove_condition(),
            combat::start_combat(),
            combat::end_combat(),
            combat::next_turn(),
            combat::death_save(),
            combat::concentration_check(),
            // Inventory
            inventory::give_item(),
            inventory::remove_item(),
            inventory::use_item(),
            inventory::equip_item(),
            inventory::unequip_item(),
            inventory::adjust_gold(),
            inventory::adjust_silver(),
            inventory::show_inventory(),
            // Class features
            class_features::use_rage(),
            class_features::end_rage(),
            class_features::use_ki(),
            class_features::use_lay_on_hands(),
            class_features::use_divine_smite(),
            class_features::use_wild_shape(),
            class_features::end_wild_shape(),
            class_features::use_channel_divinity(),
            class_features::use_bardic_inspiration(),
            class_features::use_action_surge(),
            class_features::use_second_wind(),
            class_features::use_sorcery_points(),
            // World
            world::short_rest(),
            world::long_rest(),
            world::change_location(),
            world::remember_fact(),
            world::register_consequence(),
            world::cast_spell(),
            world::award_experience(),
            world::modify_ability_score(),
            world::advance_time(),
            world::restore_spell_slot(),
            // Locations
            locations::create_location(),
            locations::connect_locations(),
            locations::update_location(),
            // NPCs
            npc::create_npc(),
            npc::update_npc(),
            npc::move_npc(),
            npc::remove_npc(),
            // Quests
            quests::create_quest(),
            quests::add_quest_objective(),
            quests::complete_objective(),
            quests::complete_quest(),
            quests::fail_quest(),
            quests::update_quest(),
            // State assertions
            state::assert_state(),
            state::query_state(),
            // Knowledge tracking
            knowledge::share_knowledge(),
            knowledge::query_knowledge(),
            // Scheduled events
            schedule::schedule_event(),
            schedule::check_schedule(),
            schedule::cancel_event(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_tools_by_category() {
        let tools = DmTools::all();

        // Check that we have tools from each category
        let check_tools = ["roll_dice", "skill_check", "ability_check", "saving_throw"];
        for name in check_tools {
            assert!(
                tools.iter().any(|t| t.name == name),
                "Missing check tool: {}",
                name
            );
        }

        let combat_tools = [
            "attack",
            "apply_damage",
            "apply_healing",
            "start_combat",
            "end_combat",
        ];
        for name in combat_tools {
            assert!(
                tools.iter().any(|t| t.name == name),
                "Missing combat tool: {}",
                name
            );
        }

        let inventory_tools = ["give_item", "remove_item", "adjust_gold", "show_inventory"];
        for name in inventory_tools {
            assert!(
                tools.iter().any(|t| t.name == name),
                "Missing inventory tool: {}",
                name
            );
        }

        let class_tools = ["use_rage", "use_ki", "use_divine_smite", "use_action_surge"];
        for name in class_tools {
            assert!(
                tools.iter().any(|t| t.name == name),
                "Missing class feature tool: {}",
                name
            );
        }

        let world_tools = [
            "short_rest",
            "long_rest",
            "change_location",
            "cast_spell",
            "award_experience",
        ];
        for name in world_tools {
            assert!(
                tools.iter().any(|t| t.name == name),
                "Missing world tool: {}",
                name
            );
        }
    }
}
