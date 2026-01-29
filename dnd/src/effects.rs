//! Effect-to-UI mapping for game effects

use dnd_core::rules::Effect;
use dnd_core::world::NarrativeType;

use crate::app::App;
use crate::ui::Overlay;

/// Process a game effect and update UI state accordingly
pub fn process_effect(app: &mut App, effect: &Effect) {
    match effect {
        Effect::DiceRolled { roll, purpose } => {
            // Show dice roll overlay
            app.set_overlay(Overlay::DiceRoll {
                result: Some(roll.clone()),
                purpose: purpose.clone(),
                dc: None,
            });

            // Add to narrative
            let result_text = format!("{}: {} = {}", purpose, roll.expression, roll.total);
            app.add_narrative(result_text, NarrativeType::System);
        }

        Effect::AttackHit {
            attacker_name,
            target_name,
            attack_roll,
            target_ac,
            is_critical,
        } => {
            if *is_critical {
                app.add_narrative(
                    format!(
                        "CRITICAL HIT! {attacker_name} rolls {attack_roll} vs AC {target_ac} and strikes {target_name}!"
                    ),
                    NarrativeType::Combat,
                );
            } else {
                app.add_narrative(
                    format!(
                        "{attacker_name} rolls {attack_roll} vs AC {target_ac} and hits {target_name}!"
                    ),
                    NarrativeType::Combat,
                );
            }
        }

        Effect::AttackMissed {
            attacker_name,
            target_name,
            attack_roll,
            target_ac,
        } => {
            app.add_narrative(
                format!(
                    "{attacker_name} rolls {attack_roll} vs AC {target_ac} and misses {target_name}!"
                ),
                NarrativeType::Combat,
            );
        }

        Effect::HpChanged {
            amount,
            new_current,
            dropped_to_zero,
            ..
        } => {
            if *amount < 0 {
                app.add_narrative(
                    format!("Takes {} damage! (HP: {})", -amount, new_current),
                    NarrativeType::Combat,
                );
            } else if *amount > 0 {
                app.add_narrative(
                    format!("Heals {amount} HP! (HP: {new_current})"),
                    NarrativeType::System,
                );
            }

            if *dropped_to_zero {
                app.set_status("You fall unconscious!");
            }
        }

        Effect::ConditionApplied {
            condition, source, ..
        } => {
            app.add_narrative(
                format!("Now {condition} from {source}!"),
                NarrativeType::Combat,
            );
        }

        Effect::ConditionRemoved { condition, .. } => {
            app.add_narrative(format!("No longer {condition}."), NarrativeType::System);
        }

        Effect::CombatStarted => {
            app.add_narrative("Combat begins!".to_string(), NarrativeType::Combat);
            app.set_status("Roll for initiative!");
        }

        Effect::CombatEnded => {
            app.add_narrative("Combat ends.".to_string(), NarrativeType::System);
        }

        Effect::TurnAdvanced {
            round,
            current_combatant,
        } => {
            app.add_narrative(
                format!("Round {round} - {current_combatant}'s turn."),
                NarrativeType::Combat,
            );
        }

        Effect::InitiativeRolled {
            name, roll, total, ..
        } => {
            app.add_narrative(
                format!("{name} rolls {roll} for initiative (total: {total})"),
                NarrativeType::System,
            );
        }

        Effect::CombatantAdded {
            name, initiative, ..
        } => {
            app.add_narrative(
                format!("{name} enters combat with initiative {initiative}."),
                NarrativeType::Combat,
            );
        }

        Effect::TimeAdvanced { minutes } => {
            if *minutes >= 60 {
                let hours = minutes / 60;
                let mins = minutes % 60;
                if mins > 0 {
                    app.add_narrative(
                        format!("{hours} hours and {mins} minutes pass."),
                        NarrativeType::System,
                    );
                } else {
                    app.add_narrative(format!("{hours} hours pass."), NarrativeType::System);
                }
            } else {
                app.add_narrative(format!("{minutes} minutes pass."), NarrativeType::System);
            }
        }

        Effect::ExperienceGained { amount, new_total } => {
            app.add_narrative(
                format!("Gained {amount} XP! (Total: {new_total} XP)"),
                NarrativeType::System,
            );
        }

        Effect::LevelUp { new_level } => {
            app.add_narrative(
                format!("LEVEL UP! You are now level {new_level}!"),
                NarrativeType::System,
            );
            app.set_status(format!("Level up! Now level {new_level}!"));
        }

        Effect::FeatureUsed {
            feature_name,
            uses_remaining,
        } => {
            app.add_narrative(
                format!("Used {feature_name}. ({uses_remaining} uses remaining)"),
                NarrativeType::System,
            );
        }

        Effect::SpellSlotUsed { level, remaining } => {
            app.add_narrative(
                format!("Used a level {level} spell slot. ({remaining} remaining)"),
                NarrativeType::System,
            );
        }

        Effect::RestCompleted { rest_type } => {
            let rest_name = match rest_type {
                dnd_core::rules::RestType::Short => "short",
                dnd_core::rules::RestType::Long => "long",
            };
            app.add_narrative(
                format!("Completed a {rest_name} rest."),
                NarrativeType::System,
            );
        }

        Effect::CheckSucceeded {
            check_type,
            roll,
            dc,
        } => {
            // Note: roll is just the total (i32), not a full RollResult
            app.add_narrative(
                format!("{check_type} check succeeded! ({roll} vs DC {dc})"),
                NarrativeType::System,
            );
            // Use lower priority so it doesn't overwrite critical messages
            app.set_status_if_empty(format!("{check_type} SUCCESS: {roll} vs DC {dc}"));
        }

        Effect::CheckFailed {
            check_type,
            roll,
            dc,
        } => {
            // Note: roll is just the total (i32), not a full RollResult
            app.add_narrative(
                format!("{check_type} check failed. ({roll} vs DC {dc})"),
                NarrativeType::System,
            );
            // Use lower priority so it doesn't overwrite critical messages
            app.set_status_if_empty(format!("{check_type} FAILED: {roll} vs DC {dc}"));
        }

        // FactRemembered is handled by the DM agent internally - no UI effect needed
        Effect::FactRemembered { .. } => {
            // Story memory storage is handled in the DM agent
        }

        // ConsequenceRegistered is handled by the DM agent internally
        Effect::ConsequenceRegistered { .. } => {
            // Consequence storage is handled in the DM agent
        }

        // ConsequenceTriggered means a stored consequence was activated
        Effect::ConsequenceTriggered {
            consequence_description,
            ..
        } => {
            app.add_narrative(
                format!("CONSEQUENCE: {consequence_description}"),
                NarrativeType::System,
            );
        }

        // Inventory effects
        Effect::ItemAdded {
            item_name,
            quantity,
            new_total,
        } => {
            let qty_str = if *quantity > 1 {
                format!("{quantity} x ")
            } else {
                String::new()
            };
            app.add_narrative(
                format!("Received {qty_str}{item_name}! (now have {new_total})"),
                NarrativeType::System,
            );
        }

        Effect::ItemRemoved {
            item_name,
            quantity,
            remaining,
        } => {
            let qty_str = if *quantity > 1 {
                format!("{quantity} x ")
            } else {
                String::new()
            };
            if *remaining > 0 {
                app.add_narrative(
                    format!("Lost {qty_str}{item_name}. ({remaining} remaining)"),
                    NarrativeType::System,
                );
            } else {
                app.add_narrative(
                    format!("Lost {qty_str}{item_name}."),
                    NarrativeType::System,
                );
            }
        }

        Effect::ItemEquipped { item_name, slot } => {
            app.add_narrative(
                format!("Equipped {item_name} in {slot} slot."),
                NarrativeType::System,
            );
        }

        Effect::ItemUnequipped { item_name, slot } => {
            app.add_narrative(
                format!("Unequipped {item_name} from {slot} slot."),
                NarrativeType::System,
            );
        }

        Effect::ItemUsed { item_name, result } => {
            app.add_narrative(
                format!("Used {item_name}. {result}"),
                NarrativeType::System,
            );
        }

        Effect::GoldChanged {
            amount,
            new_total,
            reason,
        } => {
            let action = if *amount >= 0.0 { "Gained" } else { "Spent" };
            app.add_narrative(
                format!(
                    "{} {:.0} gp ({}). Total: {:.0} gp",
                    action,
                    amount.abs(),
                    reason,
                    new_total
                ),
                NarrativeType::System,
            );
        }

        Effect::AcChanged { new_ac, source } => {
            app.add_narrative(
                format!("AC changed to {new_ac} ({source})"),
                NarrativeType::System,
            );
        }

        Effect::DeathSaveFailure {
            total_failures,
            source,
            ..
        } => {
            app.add_narrative(
                format!(
                    "DEATH SAVE FAILURE from {source}! ({total_failures}/3 failures)"
                ),
                NarrativeType::Combat,
            );
            if *total_failures >= 3 {
                app.set_status("You have died!");
            } else {
                app.set_status(format!("Death saves: {total_failures}/3 failures"));
            }
        }

        Effect::DeathSavesReset { .. } => {
            app.add_narrative(
                "Death saves reset - you're stable!".to_string(),
                NarrativeType::System,
            );
        }

        Effect::CharacterDied { cause, .. } => {
            app.add_narrative(
                format!("YOU HAVE DIED! Cause: {cause}"),
                NarrativeType::Combat,
            );
            app.set_status("GAME OVER - Your character has died.");
        }

        Effect::DeathSaveSuccess {
            roll,
            total_successes,
            ..
        } => {
            app.add_narrative(
                format!(
                    "Death save SUCCESS! Rolled {roll} ({total_successes}/3 successes)"
                ),
                NarrativeType::Combat,
            );
            app.set_status(format!("Death saves: {total_successes}/3 successes"));
        }

        Effect::Stabilized { .. } => {
            app.add_narrative(
                "You have stabilized! No longer dying.".to_string(),
                NarrativeType::Combat,
            );
            app.set_status("Stabilized - unconscious but stable");
        }

        Effect::ConcentrationBroken {
            spell_name,
            damage_taken,
            roll,
            dc,
            ..
        } => {
            app.add_narrative(
                format!(
                    "CONCENTRATION BROKEN! Took {damage_taken} damage, rolled {roll} vs DC {dc} - {spell_name} ends!"
                ),
                NarrativeType::Combat,
            );
            app.set_status(format!("Lost concentration on {spell_name}!"));
        }

        Effect::ConcentrationMaintained {
            spell_name,
            roll,
            dc,
            ..
        } => {
            app.add_narrative(
                format!(
                    "Concentration maintained! Rolled {roll} vs DC {dc} - {spell_name} continues."
                ),
                NarrativeType::System,
            );
        }

        Effect::LocationChanged {
            previous_location,
            new_location,
        } => {
            app.add_narrative(
                format!("Traveled from {previous_location} to {new_location}."),
                NarrativeType::System,
            );
            app.set_status(format!("Now at: {new_location}"));
        }
    }
}

// Note: process_effects is not used in the async architecture.
// Effects are processed individually via WorkerResponse::Effect.
