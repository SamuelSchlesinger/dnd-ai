//! Input panel for player commands.

use bevy_egui::egui;
use dnd_core::world::NarrativeType;

use crate::state::AppState;

/// Render the input panel at the bottom of the screen.
pub fn render_input_panel(ctx: &egui::Context, app_state: &mut AppState) {
    egui::TopBottomPanel::bottom("input_panel")
        .min_height(60.0)
        .show(ctx, |ui| {
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                // Input label
                ui.label(">");
                ui.add_space(4.0);

                // Check for key presses before creating the text edit
                let enter_pressed = ctx.input(|i| i.key_pressed(egui::Key::Enter));
                let up_pressed = ctx.input(|i| i.key_pressed(egui::Key::ArrowUp));
                let down_pressed = ctx.input(|i| i.key_pressed(egui::Key::ArrowDown));

                // Text input - use proportional width for responsiveness
                let available = ui.available_width();
                let input_width = (available - 60.0).max(100.0); // Leave room for Send button, min 100px
                let response = ui.add_sized(
                    [input_width, 30.0],
                    egui::TextEdit::singleline(&mut app_state.input_text)
                        .hint_text("What do you do?")
                        .interactive(!app_state.is_processing),
                );

                // Handle history navigation when input has focus
                if response.has_focus() || response.lost_focus() {
                    if up_pressed {
                        app_state.history_up();
                    } else if down_pressed {
                        app_state.history_down();
                    }
                }

                // Submit on Enter when text field has focus (or just lost it due to Enter)
                let should_submit = enter_pressed
                    && (response.has_focus() || response.lost_focus())
                    && !app_state.input_text.trim().is_empty()
                    && !app_state.is_processing;

                if should_submit {
                    let action = std::mem::take(&mut app_state.input_text);
                    // Add to history
                    app_state.add_to_history(action.clone());
                    // Add player action to narrative
                    app_state.add_narrative(action.clone(), NarrativeType::PlayerAction, 0.0);
                    app_state.send_action(action);
                }

                // Auto-focus the input field (but not right after submitting)
                if !app_state.is_processing && !should_submit {
                    response.request_focus();
                }

                ui.add_space(8.0);

                // Send button
                let send_enabled =
                    !app_state.is_processing && !app_state.input_text.trim().is_empty();
                if ui
                    .add_enabled(send_enabled, egui::Button::new("Send"))
                    .clicked()
                {
                    let action = std::mem::take(&mut app_state.input_text);
                    app_state.add_to_history(action.clone());
                    app_state.add_narrative(action.clone(), NarrativeType::PlayerAction, 0.0);
                    app_state.send_action(action);
                }
            });

            ui.add_space(4.0);

            // Quick action buttons (disabled while processing)
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                ui.add_enabled_ui(!app_state.is_processing, |ui| {
                    ui.spacing_mut().item_spacing.x = 6.0; // Add spacing between buttons

                    // Combat actions (shown during combat)
                    if app_state.in_combat && app_state.is_player_turn {
                        if ui.small_button("Attack").clicked() {
                            app_state.input_text = "I attack ".to_string();
                        }
                        if ui.small_button("Dodge").clicked() {
                            let action = "I take the Dodge action".to_string();
                            app_state.add_narrative(
                                action.clone(),
                                NarrativeType::PlayerAction,
                                0.0,
                            );
                            app_state.send_action(action);
                        }
                        if ui.small_button("Disengage").clicked() {
                            let action = "I take the Disengage action".to_string();
                            app_state.add_narrative(
                                action.clone(),
                                NarrativeType::PlayerAction,
                                0.0,
                            );
                            app_state.send_action(action);
                        }
                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(8.0);
                    }

                    // General actions - only useful ones
                    if ui.small_button("Look").clicked() {
                        let action = "I look around".to_string();
                        app_state.add_narrative(action.clone(), NarrativeType::PlayerAction, 0.0);
                        app_state.send_action(action);
                    }
                    if ui.small_button("Search").clicked() {
                        app_state.input_text = "I search ".to_string();
                    }
                    if ui.small_button("Talk to").clicked() {
                        app_state.input_text = "I talk to ".to_string();
                    }

                    // Cast spell button (only show if character has spells)
                    let has_spells = !app_state.world.cantrips.is_empty()
                        || !app_state.world.known_spells.is_empty();
                    if has_spells {
                        ui.add_space(4.0);
                        ui.separator();
                        ui.add_space(4.0);
                        if ui.small_button("Cast...").clicked() {
                            app_state.input_text = "I cast ".to_string();
                        }
                    }
                });
            });
        });
}
