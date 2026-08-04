//! Unit tests for the rules engine.

#![allow(clippy::module_inception)]

#[cfg(test)]
mod tests {
    use crate::dice::Advantage;
    use crate::rules::types::{CombatantInit, DamageType, Effect, Intent, RestType};
    use crate::rules::{apply_effect, apply_effects, RulesEngine};
    use crate::world::{create_sample_fighter, Ability, Condition, GameWorld, Skill};

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
        assert!(resolution
            .effects
            .iter()
            .any(|e| matches!(e, Effect::DiceRolled { .. })));
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
        assert!(resolution
            .effects
            .iter()
            .any(|e| matches!(e, Effect::HpChanged { amount, .. } if *amount == -10)));
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
        assert!(resolution
            .effects
            .iter()
            .any(|e| matches!(e, Effect::HpChanged { amount, .. } if *amount == 5)));
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
    fn test_healing_removes_unconscious() {
        let mut character = create_sample_fighter("Roland");
        character.hit_points.current = 0;
        character
            .conditions
            .push(crate::world::ActiveCondition::new(
                Condition::Unconscious,
                "Dropped to 0 HP",
            ));
        let mut world = GameWorld::new("Test", character);

        // Verify character is unconscious
        assert!(world
            .player_character
            .conditions
            .iter()
            .any(|c| c.condition == Condition::Unconscious));

        // Apply healing effect
        let effect = Effect::HpChanged {
            target_id: world.player_character.id,
            amount: 5,
            new_current: 5,
            new_max: world.player_character.hit_points.maximum,
            dropped_to_zero: false,
        };
        apply_effect(&mut world, &effect);

        // Verify unconscious is removed
        assert!(!world
            .player_character
            .conditions
            .iter()
            .any(|c| c.condition == Condition::Unconscious));
        assert_eq!(world.player_character.hit_points.current, 5);
    }

    #[test]
    fn test_massive_damage_detection() {
        let mut character = create_sample_fighter("Roland");
        character.hit_points.current = 10;
        character.hit_points.maximum = 30;
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        // Damage that would cause instant death (10 current + 30 max = need 40+ damage)
        let intent = Intent::Damage {
            target_id: world.player_character.id,
            amount: 50,
            damage_type: DamageType::Slashing,
            source: "Dragon".to_string(),
        };

        let resolution = engine.resolve(&world, intent);
        // Should mention instant death in the narrative
        assert!(resolution.narrative.contains("INSTANT DEATH"));
    }

    #[test]
    fn test_start_combat() {
        let character = create_sample_fighter("Roland");
        let world = GameWorld::new("Test", character.clone());
        let engine = RulesEngine::new();

        let intent = Intent::StartCombat {
            combatants: vec![CombatantInit {
                id: character.id,
                name: "Roland".to_string(),
                is_player: true,
                is_ally: true,
                current_hp: character.hit_points.current,
                max_hp: character.hit_points.maximum,
                armor_class: character.current_ac(),
                initiative_modifier: character.initiative_modifier(),
            }],
        };

        let resolution = engine.resolve(&world, intent);
        assert!(resolution
            .effects
            .iter()
            .any(|e| matches!(e, Effect::CombatStarted)));
        assert!(resolution
            .effects
            .iter()
            .any(|e| matches!(e, Effect::InitiativeRolled { .. })));
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
        assert!(resolution
            .effects
            .iter()
            .any(|e| matches!(e, Effect::DiceRolled { .. })));
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
        assert!(
            resolution.narrative.contains("HP:"),
            "Damage narrative should include HP status: {}",
            resolution.narrative
        );
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
        assert!(
            resolution.narrative.contains("UNCONSCIOUS"),
            "Lethal damage narrative should indicate UNCONSCIOUS: {}",
            resolution.narrative
        );
        // Effect should have dropped_to_zero
        assert!(resolution.effects.iter().any(|e| matches!(
            e,
            Effect::HpChanged {
                dropped_to_zero: true,
                ..
            }
        )));
    }

    #[test]
    fn test_short_rest_blocked_during_combat() {
        let character = create_sample_fighter("Roland");
        let mut world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        // Start combat (just need combat state to exist)
        world.combat = Some(crate::world::CombatState::new());

        let resolution = engine.resolve(&world, Intent::ShortRest);
        assert!(resolution.effects.is_empty());
        assert!(resolution.narrative.contains("Cannot take a short rest"));
    }

    #[test]
    fn test_long_rest_blocked_during_combat() {
        let character = create_sample_fighter("Roland");
        let mut world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        // Start combat (just need combat state to exist)
        world.combat = Some(crate::world::CombatState::new());

        let resolution = engine.resolve(&world, Intent::LongRest);
        assert!(resolution.effects.is_empty());
        assert!(resolution.narrative.contains("Cannot take a long rest"));
    }

    #[test]
    fn test_rest_allowed_outside_combat() {
        let character = create_sample_fighter("Roland");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        // No combat active
        assert!(world.combat.is_none());

        let short_rest = engine.resolve(&world, Intent::ShortRest);
        assert!(!short_rest.effects.is_empty());
        assert!(short_rest.effects.iter().any(|e| matches!(
            e,
            Effect::RestCompleted {
                rest_type: RestType::Short
            }
        )));

        let long_rest = engine.resolve(&world, Intent::LongRest);
        assert!(!long_rest.effects.is_empty());
        assert!(long_rest.effects.iter().any(|e| matches!(
            e,
            Effect::RestCompleted {
                rest_type: RestType::Long
            }
        )));
    }

    #[test]
    fn test_unconscious_cannot_attack() {
        let mut character = create_sample_fighter("Roland");
        character.hit_points.current = 0;
        character
            .conditions
            .push(crate::world::ActiveCondition::new(
                Condition::Unconscious,
                "Dropped to 0 HP",
            ));
        let world = GameWorld::new("Test", character.clone());
        let engine = RulesEngine::new();

        let resolution = engine.resolve(
            &world,
            Intent::Attack {
                attacker_id: character.id,
                target_id: crate::world::CharacterId::new(),
                weapon_name: "Longsword".to_string(),
                advantage: Advantage::Normal,
            },
        );

        assert!(resolution.effects.is_empty());
        assert!(resolution.narrative.contains("unconscious"));
        assert!(resolution.narrative.contains("cannot attack"));
    }

    #[test]
    fn test_unconscious_auto_fails_str_dex_checks() {
        let mut character = create_sample_fighter("Roland");
        character.hit_points.current = 0;
        character
            .conditions
            .push(crate::world::ActiveCondition::new(
                Condition::Unconscious,
                "Dropped to 0 HP",
            ));
        let world = GameWorld::new("Test", character.clone());
        let engine = RulesEngine::new();

        // Athletics is a Strength skill - should auto-fail
        let athletics_check = engine.resolve(
            &world,
            Intent::SkillCheck {
                character_id: character.id,
                skill: Skill::Athletics,
                dc: 10,
                advantage: Advantage::Normal,
                description: "Climbing".to_string(),
            },
        );
        assert!(athletics_check.narrative.contains("unconscious"));
        assert!(athletics_check.narrative.contains("automatically fails"));

        // Acrobatics is a Dexterity skill - should auto-fail
        let acrobatics_check = engine.resolve(
            &world,
            Intent::SkillCheck {
                character_id: character.id,
                skill: Skill::Acrobatics,
                dc: 10,
                advantage: Advantage::Normal,
                description: "Tumbling".to_string(),
            },
        );
        assert!(acrobatics_check.narrative.contains("unconscious"));
        assert!(acrobatics_check.narrative.contains("automatically fails"));

        // Perception is a Wisdom skill - should NOT auto-fail
        let perception_check = engine.resolve(
            &world,
            Intent::SkillCheck {
                character_id: character.id,
                skill: Skill::Perception,
                dc: 10,
                advantage: Advantage::Normal,
                description: "Noticing".to_string(),
            },
        );
        // Should actually roll (won't auto-fail since it's Wisdom-based)
        assert!(perception_check
            .effects
            .iter()
            .any(|e| matches!(e, Effect::DiceRolled { .. })));
    }

    #[test]
    fn test_unconscious_auto_fails_str_dex_saves() {
        let mut character = create_sample_fighter("Roland");
        character.hit_points.current = 0;
        character
            .conditions
            .push(crate::world::ActiveCondition::new(
                Condition::Unconscious,
                "Dropped to 0 HP",
            ));
        let world = GameWorld::new("Test", character.clone());
        let engine = RulesEngine::new();

        // Dexterity save - should auto-fail
        let dex_save = engine.resolve(
            &world,
            Intent::SavingThrow {
                character_id: character.id,
                ability: Ability::Dexterity,
                dc: 15,
                advantage: Advantage::Normal,
                source: "Fireball".to_string(),
            },
        );
        assert!(dex_save.narrative.contains("unconscious"));
        assert!(dex_save.narrative.contains("automatically fails"));

        // Constitution save - should NOT auto-fail
        let con_save = engine.resolve(
            &world,
            Intent::SavingThrow {
                character_id: character.id,
                ability: Ability::Constitution,
                dc: 15,
                advantage: Advantage::Normal,
                source: "Poison".to_string(),
            },
        );
        // Should actually roll
        assert!(con_save
            .effects
            .iter()
            .any(|e| matches!(e, Effect::DiceRolled { .. })));
    }

    #[test]
    fn test_damage_at_zero_hp_causes_death_save_failure() {
        let mut character = create_sample_fighter("Roland");
        character.hit_points.current = 0;
        character
            .conditions
            .push(crate::world::ActiveCondition::new(
                Condition::Unconscious,
                "Dropped to 0 HP",
            ));
        let world = GameWorld::new("Test", character.clone());
        let engine = RulesEngine::new();

        let resolution = engine.resolve(
            &world,
            Intent::Damage {
                target_id: character.id,
                amount: 5,
                damage_type: DamageType::Slashing,
                source: "Goblin".to_string(),
            },
        );

        // Should have death save failure effect
        assert!(resolution.effects.iter().any(|e| matches!(
            e,
            Effect::DeathSaveFailure {
                failures: 1,
                total_failures: 1,
                ..
            }
        )));
        assert!(resolution.narrative.contains("death save failure"));
    }

    #[test]
    fn test_massive_damage_at_zero_hp_causes_instant_death() {
        let mut character = create_sample_fighter("Roland");
        character.hit_points.current = 0;
        character.hit_points.maximum = 30;
        character
            .conditions
            .push(crate::world::ActiveCondition::new(
                Condition::Unconscious,
                "Dropped to 0 HP",
            ));
        let world = GameWorld::new("Test", character.clone());
        let engine = RulesEngine::new();

        // Damage >= max HP while at 0 HP = instant death
        let resolution = engine.resolve(
            &world,
            Intent::Damage {
                target_id: character.id,
                amount: 30,
                damage_type: DamageType::Slashing,
                source: "Dragon".to_string(),
            },
        );

        // Should have character died effect
        assert!(resolution
            .effects
            .iter()
            .any(|e| matches!(e, Effect::CharacterDied { .. })));
        assert!(resolution.narrative.contains("INSTANT DEATH"));
    }

    #[test]
    fn test_healing_resets_death_saves() {
        let mut character = create_sample_fighter("Roland");
        character.hit_points.current = 0;
        character.death_saves.failures = 2; // 2 failures already
        character
            .conditions
            .push(crate::world::ActiveCondition::new(
                Condition::Unconscious,
                "Dropped to 0 HP",
            ));
        let mut world = GameWorld::new("Test", character);

        // Apply healing effect
        let effect = Effect::HpChanged {
            target_id: world.player_character.id,
            amount: 5,
            new_current: 5,
            new_max: world.player_character.hit_points.maximum,
            dropped_to_zero: false,
        };
        apply_effect(&mut world, &effect);

        // Death saves should be reset
        assert_eq!(world.player_character.death_saves.failures, 0);
        assert_eq!(world.player_character.death_saves.successes, 0);
        // Unconscious should be removed
        assert!(!world
            .player_character
            .conditions
            .iter()
            .any(|c| c.condition == Condition::Unconscious));
    }

    #[test]
    fn test_three_death_save_failures_causes_death() {
        let mut character = create_sample_fighter("Roland");
        character.hit_points.current = 0;
        character.death_saves.failures = 2; // Already have 2 failures
        character
            .conditions
            .push(crate::world::ActiveCondition::new(
                Condition::Unconscious,
                "Dropped to 0 HP",
            ));
        let world = GameWorld::new("Test", character.clone());
        let engine = RulesEngine::new();

        // Take damage at 0 HP - should cause 3rd failure and death
        let resolution = engine.resolve(
            &world,
            Intent::Damage {
                target_id: character.id,
                amount: 5,
                damage_type: DamageType::Slashing,
                source: "Goblin".to_string(),
            },
        );

        // Should have both death save failure and character died effects
        assert!(resolution.effects.iter().any(|e| matches!(
            e,
            Effect::DeathSaveFailure {
                total_failures: 3,
                ..
            }
        )));
        assert!(resolution
            .effects
            .iter()
            .any(|e| matches!(e, Effect::CharacterDied { .. })));
        assert!(resolution.narrative.contains("DIES"));
    }

    // ========================================================================
    // World Building Tool Tests
    // ========================================================================

    // ------------------------------------------------------------------------
    // NPC Tools
    // ------------------------------------------------------------------------

    #[test]
    fn test_create_npc() {
        let character = create_sample_fighter("Roland");
        let mut world = GameWorld::new("Test Campaign", character);
        let engine = RulesEngine::new();

        // Verify world starts with no NPCs
        assert!(world.npcs.is_empty());

        let intent = Intent::CreateNpc {
            name: "Gundren Rockseeker".to_string(),
            description: "A grizzled dwarf merchant".to_string(),
            personality: "Gruff but loyal".to_string(),
            occupation: Some("Merchant".to_string()),
            disposition: "Friendly".to_string(),
            location: None,
            known_information: vec!["Has a map to Wave Echo Cave".to_string()],
        };

        let resolution = engine.resolve(&world, intent);

        // Verify we got the NpcCreated effect
        assert!(resolution
            .effects
            .iter()
            .any(|e| matches!(e, Effect::NpcCreated { name, .. } if name == "Gundren Rockseeker")));

        // Apply effects and verify the NPC was added
        apply_effects(&mut world, &resolution.effects);
        assert_eq!(world.npcs.len(), 1);

        let npc = world.npcs.values().next().unwrap();
        assert_eq!(npc.name, "Gundren Rockseeker");
    }

    #[test]
    fn test_create_npc_duplicate_detection() {
        let character = create_sample_fighter("Roland");
        let mut world = GameWorld::new("Test Campaign", character);
        let engine = RulesEngine::new();

        // Create an NPC
        let intent = Intent::CreateNpc {
            name: "Thorek Ironbrew".to_string(),
            description: "A grizzled dwarf innkeeper".to_string(),
            personality: "Gruff but fair".to_string(),
            occupation: Some("Innkeeper".to_string()),
            disposition: "Neutral".to_string(),
            location: Some("The Crossroads Inn".to_string()),
            known_information: vec![],
        };

        let resolution = engine.resolve(&world, intent);
        apply_effects(&mut world, &resolution.effects);
        assert_eq!(world.npcs.len(), 1);

        // Try to create the same NPC again (should fail with helpful message)
        let duplicate_intent = Intent::CreateNpc {
            name: "Thorek Ironbrew".to_string(), // Same name
            description: "A different description".to_string(),
            personality: "Different personality".to_string(),
            occupation: Some("Barkeep".to_string()),
            disposition: "Friendly".to_string(),
            location: None,
            known_information: vec![],
        };

        let duplicate_resolution = engine.resolve(&world, duplicate_intent);

        // Verify no NpcCreated effect was produced
        assert!(
            !duplicate_resolution
                .effects
                .iter()
                .any(|e| matches!(e, Effect::NpcCreated { .. })),
            "Should not produce NpcCreated effect for duplicate NPC"
        );

        // Verify the narrative contains the error message
        assert!(
            duplicate_resolution
                .narrative
                .contains("DUPLICATE NPC ERROR"),
            "Should contain DUPLICATE NPC ERROR in narrative"
        );
        assert!(
            duplicate_resolution.narrative.contains("update_npc"),
            "Should suggest using update_npc"
        );

        // Verify NPC count didn't change
        apply_effects(&mut world, &duplicate_resolution.effects);
        assert_eq!(world.npcs.len(), 1, "NPC count should not increase");
    }

    #[test]
    fn test_create_npc_duplicate_case_insensitive() {
        let character = create_sample_fighter("Roland");
        let mut world = GameWorld::new("Test Campaign", character);
        let engine = RulesEngine::new();

        // Create an NPC
        let intent = Intent::CreateNpc {
            name: "Mira the Smith".to_string(),
            description: "A skilled blacksmith".to_string(),
            personality: "Hardworking".to_string(),
            occupation: Some("Blacksmith".to_string()),
            disposition: "Neutral".to_string(),
            location: None,
            known_information: vec![],
        };

        let resolution = engine.resolve(&world, intent);
        apply_effects(&mut world, &resolution.effects);

        // Try with different casing (should still detect as duplicate)
        let duplicate_intent = Intent::CreateNpc {
            name: "MIRA THE SMITH".to_string(), // Different case
            description: "Another description".to_string(),
            personality: "Different".to_string(),
            occupation: None,
            disposition: "Friendly".to_string(),
            location: None,
            known_information: vec![],
        };

        let duplicate_resolution = engine.resolve(&world, duplicate_intent);

        // Should detect as duplicate (case-insensitive)
        assert!(
            duplicate_resolution
                .narrative
                .contains("DUPLICATE NPC ERROR"),
            "Should detect case-insensitive duplicate"
        );
    }

    #[test]
    fn test_update_npc_disposition() {
        let character = create_sample_fighter("Roland");
        let mut world = GameWorld::new("Test Campaign", character);
        let engine = RulesEngine::new();

        // First create an NPC
        let create_intent = Intent::CreateNpc {
            name: "Sildar Hallwinter".to_string(),
            description: "A human warrior".to_string(),
            personality: "Honorable and brave".to_string(),
            occupation: Some("Knight".to_string()),
            disposition: "Neutral".to_string(),
            location: None,
            known_information: vec![],
        };
        let create_resolution = engine.resolve(&world, create_intent);
        apply_effects(&mut world, &create_resolution.effects);

        // Now update the NPC's disposition
        let update_intent = Intent::UpdateNpc {
            npc_name: "Sildar Hallwinter".to_string(),
            disposition: Some("Friendly".to_string()),
            add_information: vec!["Knows about the Redbrands".to_string()],
            new_description: None,
            new_personality: None,
        };

        let update_resolution = engine.resolve(&world, update_intent);

        // Verify we got the NpcUpdated effect
        assert!(update_resolution.effects.iter().any(|e| matches!(
            e,
            Effect::NpcUpdated { npc_name, changes }
            if npc_name == "Sildar Hallwinter" && changes.contains("disposition changed")
        )));

        // Verify narrative mentions the update
        assert!(update_resolution.narrative.contains("Sildar Hallwinter"));
        assert!(update_resolution.narrative.contains("updated"));
    }

    #[test]
    fn test_update_nonexistent_npc() {
        let character = create_sample_fighter("Roland");
        let world = GameWorld::new("Test Campaign", character);
        let engine = RulesEngine::new();

        // Try to update an NPC that doesn't exist
        let update_intent = Intent::UpdateNpc {
            npc_name: "Ghost NPC".to_string(),
            disposition: Some("Hostile".to_string()),
            add_information: vec![],
            new_description: None,
            new_personality: None,
        };

        let resolution = engine.resolve(&world, update_intent);

        // Should have no effects (NPC not found)
        assert!(resolution.effects.is_empty());
        assert!(resolution.narrative.contains("not found"));
    }

    #[test]
    fn test_move_npc() {
        use crate::world::{Location, LocationType};

        let character = create_sample_fighter("Roland");
        let mut world = GameWorld::new("Test Campaign", character);
        let engine = RulesEngine::new();

        // Create a destination location
        let tavern = Location::new("The Sleeping Giant", LocationType::Building);
        let tavern_id = tavern.id;
        world.known_locations.insert(tavern_id, tavern);

        // Create an NPC
        let create_intent = Intent::CreateNpc {
            name: "Barthen".to_string(),
            description: "A shopkeeper".to_string(),
            personality: "Helpful".to_string(),
            occupation: Some("Shopkeeper".to_string()),
            disposition: "Friendly".to_string(),
            location: None,
            known_information: vec![],
        };
        let create_resolution = engine.resolve(&world, create_intent);
        apply_effects(&mut world, &create_resolution.effects);

        // Move the NPC to the tavern
        let move_intent = Intent::MoveNpc {
            npc_name: "Barthen".to_string(),
            destination: "The Sleeping Giant".to_string(),
            reason: Some("Going for a drink".to_string()),
        };

        let move_resolution = engine.resolve(&world, move_intent);

        // Verify we got the NpcMoved effect
        assert!(move_resolution.effects.iter().any(|e| matches!(
            e,
            Effect::NpcMoved { npc_name, to_location, .. }
            if npc_name == "Barthen" && to_location == "The Sleeping Giant"
        )));

        // Apply effects and verify the NPC's location changed
        apply_effects(&mut world, &move_resolution.effects);

        let npc = world.npcs.values().find(|n| n.name == "Barthen").unwrap();
        assert_eq!(npc.location_id, Some(tavern_id));
    }

    #[test]
    fn test_remove_npc() {
        let character = create_sample_fighter("Roland");
        let mut world = GameWorld::new("Test Campaign", character);
        let engine = RulesEngine::new();

        // Create an NPC
        let create_intent = Intent::CreateNpc {
            name: "Glasstaff".to_string(),
            description: "A villainous wizard".to_string(),
            personality: "Cunning and cruel".to_string(),
            occupation: Some("Wizard".to_string()),
            disposition: "Hostile".to_string(),
            location: None,
            known_information: vec![],
        };
        let create_resolution = engine.resolve(&world, create_intent);
        apply_effects(&mut world, &create_resolution.effects);

        assert_eq!(world.npcs.len(), 1);

        // Remove the NPC
        let remove_intent = Intent::RemoveNpc {
            npc_name: "Glasstaff".to_string(),
            reason: "Defeated by the party".to_string(),
            permanent: true,
        };

        let remove_resolution = engine.resolve(&world, remove_intent);

        // Verify we got the NpcRemoved effect
        assert!(remove_resolution.effects.iter().any(|e| matches!(
            e,
            Effect::NpcRemoved { npc_name, reason }
            if npc_name == "Glasstaff" && reason == "Defeated by the party"
        )));

        // Apply effects and verify the NPC was removed
        apply_effects(&mut world, &remove_resolution.effects);
        assert!(world.npcs.is_empty());
    }

    // ------------------------------------------------------------------------
    // Location Tools
    // ------------------------------------------------------------------------

    #[test]
    fn test_create_location() {
        let character = create_sample_fighter("Roland");
        let mut world = GameWorld::new("Test Campaign", character);
        let engine = RulesEngine::new();

        // World starts with the starting location
        let initial_location_count = world.known_locations.len();

        let intent = Intent::CreateLocation {
            name: "Phandalin".to_string(),
            location_type: "Town".to_string(),
            description: "A small frontier town".to_string(),
            parent_location: None,
            items: vec!["Notice Board".to_string()],
            npcs_present: vec![],
        };

        let resolution = engine.resolve(&world, intent);

        // Verify we got the LocationCreated effect
        assert!(resolution.effects.iter().any(|e| matches!(
            e,
            Effect::LocationCreated { name, location_type }
            if name == "Phandalin" && location_type == "Town"
        )));

        // Apply effects and verify the location was added
        apply_effects(&mut world, &resolution.effects);
        assert_eq!(world.known_locations.len(), initial_location_count + 1);

        // Find the new location
        let phandalin = world
            .known_locations
            .values()
            .find(|l| l.name == "Phandalin");
        assert!(phandalin.is_some());
    }

    #[test]
    fn test_connect_locations() {
        use crate::world::{Location, LocationType};

        let character = create_sample_fighter("Roland");
        let mut world = GameWorld::new("Test Campaign", character);
        let engine = RulesEngine::new();

        // Create two locations
        let town = Location::new("Phandalin", LocationType::Town);
        let town_id = town.id;
        world.known_locations.insert(town_id, town);

        let cave = Location::new("Wave Echo Cave", LocationType::Cave);
        let cave_id = cave.id;
        world.known_locations.insert(cave_id, cave);

        // Connect them
        let intent = Intent::ConnectLocations {
            from_location: "Phandalin".to_string(),
            to_location: "Wave Echo Cave".to_string(),
            direction: Some("east".to_string()),
            travel_time_minutes: Some(120),
            bidirectional: true,
        };

        let resolution = engine.resolve(&world, intent);

        // Verify we got the LocationsConnected effect
        assert!(resolution.effects.iter().any(|e| matches!(
            e,
            Effect::LocationsConnected { from, to, direction }
            if from == "Phandalin" && to == "Wave Echo Cave" && direction.as_deref() == Some("east")
        )));

        // Apply effects and verify the connection was added
        apply_effects(&mut world, &resolution.effects);

        let phandalin = world.known_locations.get(&town_id).unwrap();
        assert_eq!(phandalin.connections.len(), 1);
        assert_eq!(phandalin.connections[0].destination_id, cave_id);
        assert_eq!(phandalin.connections[0].destination_name, "Wave Echo Cave");
        assert_eq!(phandalin.connections[0].direction, Some("east".to_string()));
    }

    #[test]
    fn test_update_location() {
        use crate::world::{Location, LocationType};

        let character = create_sample_fighter("Roland");
        let mut world = GameWorld::new("Test Campaign", character);
        let engine = RulesEngine::new();

        // Create a location
        let tavern = Location::new("Stonehill Inn", LocationType::Building);
        world.known_locations.insert(tavern.id, tavern);

        // Update the location
        let intent = Intent::UpdateLocation {
            location_name: "Stonehill Inn".to_string(),
            new_description: Some("A cozy inn with warm hearth".to_string()),
            add_items: vec!["Ale Barrel".to_string(), "Fireplace".to_string()],
            remove_items: vec![],
            add_npcs: vec!["Toblen Stonehill".to_string()],
            remove_npcs: vec![],
        };

        let resolution = engine.resolve(&world, intent);

        // Verify we got the LocationUpdated effect
        assert!(resolution.effects.iter().any(|e| matches!(
            e,
            Effect::LocationUpdated { location_name, changes }
            if location_name == "Stonehill Inn" && changes.contains("description updated")
        )));

        // Verify narrative mentions the changes
        assert!(resolution.narrative.contains("Stonehill Inn"));
        assert!(resolution.narrative.contains("updated"));
    }

    #[test]
    fn test_update_nonexistent_location() {
        let character = create_sample_fighter("Roland");
        let world = GameWorld::new("Test Campaign", character);
        let engine = RulesEngine::new();

        // Try to update a location that doesn't exist
        let intent = Intent::UpdateLocation {
            location_name: "Ghost Town".to_string(),
            new_description: Some("A spooky place".to_string()),
            add_items: vec![],
            remove_items: vec![],
            add_npcs: vec![],
            remove_npcs: vec![],
        };

        let resolution = engine.resolve(&world, intent);

        // Should have no effects (location not found)
        assert!(resolution.effects.is_empty());
        assert!(resolution.narrative.contains("not found"));
    }

    // ------------------------------------------------------------------------
    // Gameplay Tools
    // ------------------------------------------------------------------------

    #[test]
    fn test_modify_ability_score() {
        let character = create_sample_fighter("Roland");
        let mut world = GameWorld::new("Test Campaign", character);
        let engine = RulesEngine::new();

        // Get initial strength
        let initial_strength = world.player_character.ability_scores.strength;

        let intent = Intent::ModifyAbilityScore {
            ability: Ability::Strength,
            modifier: 2,
            source: "Bull's Strength".to_string(),
            duration: Some("1 hour".to_string()),
        };

        let resolution = engine.resolve(&world, intent);

        // Verify we got the AbilityScoreModified effect
        assert!(resolution.effects.iter().any(|e| matches!(
            e,
            Effect::AbilityScoreModified { ability, modifier, source }
            if *ability == Ability::Strength && *modifier == 2 && source == "Bull's Strength"
        )));

        // Apply effects and verify the ability score changed
        apply_effects(&mut world, &resolution.effects);
        assert_eq!(
            world.player_character.ability_scores.strength,
            initial_strength + 2
        );
    }

    #[test]
    fn test_modify_ability_score_negative() {
        let character = create_sample_fighter("Roland");
        let mut world = GameWorld::new("Test Campaign", character);
        let engine = RulesEngine::new();

        // Get initial dexterity
        let initial_dexterity = world.player_character.ability_scores.dexterity;

        let intent = Intent::ModifyAbilityScore {
            ability: Ability::Dexterity,
            modifier: -4,
            source: "Ray of Enfeeblement".to_string(),
            duration: Some("1 minute".to_string()),
        };

        let resolution = engine.resolve(&world, intent);

        // Apply effects and verify the ability score decreased
        apply_effects(&mut world, &resolution.effects);
        assert_eq!(
            world.player_character.ability_scores.dexterity,
            initial_dexterity - 4
        );
    }

    #[test]
    fn test_modify_ability_score_clamped() {
        let character = create_sample_fighter("Roland");
        let mut world = GameWorld::new("Test Campaign", character);
        let engine = RulesEngine::new();

        // Try to reduce intelligence below 1
        let intent = Intent::ModifyAbilityScore {
            ability: Ability::Intelligence,
            modifier: -20, // Character has 10 INT, this would make it -10
            source: "Feeblemind".to_string(),
            duration: None,
        };

        let resolution = engine.resolve(&world, intent);
        apply_effects(&mut world, &resolution.effects);

        // Should be clamped to minimum of 1
        assert_eq!(world.player_character.ability_scores.intelligence, 1);
    }

    #[test]
    fn test_restore_spell_slot() {
        use crate::world::{SlotInfo, SpellSlots, SpellcastingData};

        let mut character = create_sample_fighter("Roland");
        // Give the fighter some spellcasting (e.g., Eldritch Knight)
        character.spellcasting = Some(SpellcastingData {
            ability: Ability::Intelligence,
            spells_known: vec!["Shield".to_string()],
            spells_prepared: vec![],
            cantrips_known: vec!["Fire Bolt".to_string()],
            spell_slots: SpellSlots {
                slots: [
                    SlotInfo { total: 3, used: 2 }, // Level 1: 1 remaining
                    SlotInfo { total: 2, used: 2 }, // Level 2: 0 remaining
                    SlotInfo { total: 0, used: 0 }, // Level 3
                    SlotInfo { total: 0, used: 0 }, // Level 4
                    SlotInfo { total: 0, used: 0 }, // Level 5
                    SlotInfo { total: 0, used: 0 }, // Level 6
                    SlotInfo { total: 0, used: 0 }, // Level 7
                    SlotInfo { total: 0, used: 0 }, // Level 8
                    SlotInfo { total: 0, used: 0 }, // Level 9
                ],
            },
        });

        let mut world = GameWorld::new("Test Campaign", character);
        let engine = RulesEngine::new();

        // Get initial available slots
        let initial_level1_used = world
            .player_character
            .spellcasting
            .as_ref()
            .unwrap()
            .spell_slots
            .slots[0]
            .used;

        let intent = Intent::RestoreSpellSlot {
            slot_level: 1,
            source: "Arcane Recovery".to_string(),
        };

        let resolution = engine.resolve(&world, intent);

        // Verify we got the SpellSlotRestored effect
        assert!(resolution
            .effects
            .iter()
            .any(|e| matches!(e, Effect::SpellSlotRestored { level: 1, .. })));

        // Apply effects and verify the spell slot was restored
        apply_effects(&mut world, &resolution.effects);

        let new_level1_used = world
            .player_character
            .spellcasting
            .as_ref()
            .unwrap()
            .spell_slots
            .slots[0]
            .used;

        assert_eq!(new_level1_used, initial_level1_used - 1);
    }

    #[test]
    fn test_restore_spell_slot_invalid_level() {
        let character = create_sample_fighter("Roland");
        let world = GameWorld::new("Test Campaign", character);
        let engine = RulesEngine::new();

        // Try to restore an invalid spell slot level
        let intent = Intent::RestoreSpellSlot {
            slot_level: 10, // Invalid: max is 9
            source: "Invalid Source".to_string(),
        };

        let resolution = engine.resolve(&world, intent);

        // Should have no effects (invalid level)
        assert!(resolution.effects.is_empty());
        assert!(resolution.narrative.contains("Invalid"));
    }

    #[test]
    fn test_create_npc_with_location() {
        use crate::world::{Location, LocationType};

        let character = create_sample_fighter("Roland");
        let mut world = GameWorld::new("Test Campaign", character);
        let engine = RulesEngine::new();

        // Create a location first
        let tavern = Location::new("The Yawning Portal", LocationType::Building);
        let tavern_id = tavern.id;
        world.known_locations.insert(tavern_id, tavern);

        // Create an NPC at that location
        let intent = Intent::CreateNpc {
            name: "Durnan".to_string(),
            description: "A retired adventurer and tavern keeper".to_string(),
            personality: "Stoic and mysterious".to_string(),
            occupation: Some("Tavern Keeper".to_string()),
            disposition: "Neutral".to_string(),
            location: Some("The Yawning Portal".to_string()),
            known_information: vec!["Knows about Undermountain".to_string()],
        };

        let resolution = engine.resolve(&world, intent);
        apply_effects(&mut world, &resolution.effects);

        // Verify NPC was created at the correct location
        let npc = world.npcs.values().find(|n| n.name == "Durnan").unwrap();
        assert_eq!(npc.location_id, Some(tavern_id));
    }
}
