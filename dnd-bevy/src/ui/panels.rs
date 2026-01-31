//! Main UI panels for the game interface.

use bevy::prelude::*;
use bevy_egui::egui;
use dnd_core::world::NarrativeType;

use crate::state::{ActiveOverlay, AppState, GamePhase, WorkerRequest};

/// Render the main menu screen.
pub fn render_main_menu(
    ctx: &egui::Context,
    next_phase: &mut NextState<GamePhase>,
    app_state: &mut AppState,
) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(100.0);

            // Title
            ui.heading(
                egui::RichText::new("D&D: AI Dungeon Master")
                    .size(48.0)
                    .color(egui::Color32::from_rgb(218, 165, 32)),
            );

            ui.add_space(20.0);
            ui.label(
                egui::RichText::new("A text-based adventure powered by AI")
                    .size(18.0)
                    .italics(),
            );

            ui.add_space(60.0);

            // Menu buttons
            let button_size = egui::vec2(200.0, 40.0);

            if ui
                .add_sized(button_size, egui::Button::new("New Game"))
                .clicked()
            {
                next_phase.set(GamePhase::CharacterCreation);
            }

            ui.add_space(10.0);

            if ui
                .add_sized(button_size, egui::Button::new("Load Character"))
                .on_hover_text("Load a saved character to start a new adventure")
                .clicked()
            {
                app_state.toggle_overlay(ActiveOverlay::LoadCharacter);
            }

            ui.add_space(10.0);

            if ui
                .add_sized(button_size, egui::Button::new("Load Game"))
                .on_hover_text("Continue a saved campaign")
                .clicked()
            {
                app_state.toggle_overlay(ActiveOverlay::LoadGame);
            }

            ui.add_space(10.0);

            if ui
                .add_sized(button_size, egui::Button::new("Settings"))
                .clicked()
            {
                app_state.toggle_overlay(ActiveOverlay::Settings);
            }

            ui.add_space(10.0);

            if ui
                .add_sized(button_size, egui::Button::new("Quit"))
                .clicked()
            {
                std::process::exit(0);
            }

            ui.add_space(40.0);

            // Footer
            ui.label(
                egui::RichText::new("Cmd+Q / Ctrl+Q to quit")
                    .size(12.0)
                    .color(egui::Color32::GRAY),
            );
        });
    });
}

/// Render the top bar with title and status.
pub fn render_top_bar(ctx: &egui::Context, app_state: &mut AppState) {
    egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            // Game title / campaign name
            ui.heading(
                egui::RichText::new(&app_state.world.campaign_name)
                    .color(egui::Color32::from_rgb(218, 165, 32)),
            );

            ui.separator();

            // Location
            ui.label(format!("Location: {}", app_state.world.current_location));

            ui.separator();

            // Game time
            let time = &app_state.world.game_time;
            ui.label(format!(
                "Day {}, {}:{:02}",
                time.day, time.hour, time.minute
            ));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Use consistent spacing between buttons
                ui.spacing_mut().item_spacing.x = 6.0;

                // Help button
                if ui.button("?").on_hover_text("Help (F1)").clicked() {
                    app_state.toggle_overlay(ActiveOverlay::Help);
                }

                // Settings button
                if ui
                    .button("Settings")
                    .on_hover_text("Game settings")
                    .clicked()
                {
                    app_state.toggle_overlay(ActiveOverlay::Settings);
                }

                // Character Sheet button
                if ui
                    .button("Character")
                    .on_hover_text("View full character sheet")
                    .clicked()
                {
                    app_state.toggle_overlay(ActiveOverlay::CharacterSheet);
                }

                ui.add_space(4.0);
                ui.separator();
                ui.add_space(4.0);

                // Load button
                let load_enabled =
                    !app_state.is_loading && !app_state.is_processing && app_state.has_session();
                let autosave_path =
                    dnd_core::persist::auto_save_path("saves", &app_state.world.campaign_name);
                let autosave_exists = autosave_path.exists();
                if ui
                    .add_enabled(load_enabled && autosave_exists, egui::Button::new("Load"))
                    .on_hover_text(if autosave_exists {
                        "Load last save (Ctrl+L)"
                    } else {
                        "No save found - save first"
                    })
                    .clicked()
                {
                    if let Some(tx) = &app_state.request_tx {
                        let _ = tx.try_send(WorkerRequest::Load(autosave_path));
                        app_state.is_loading = true;
                        app_state.set_status_persistent("Loading...");
                    }
                }

                // Save button
                let save_enabled =
                    !app_state.is_saving && !app_state.is_processing && app_state.has_session();
                if ui
                    .add_enabled(save_enabled, egui::Button::new("Save"))
                    .on_hover_text("Save game (Ctrl+S)")
                    .clicked()
                {
                    if let Some(tx) = &app_state.request_tx {
                        let path = dnd_core::persist::auto_save_path(
                            "saves",
                            &app_state.world.campaign_name,
                        );
                        let _ = tx.try_send(WorkerRequest::Save(path));
                        app_state.is_saving = true;
                        app_state.set_status_persistent("Saving...");
                    }
                }

                ui.add_space(10.0);

                // Status message with improved visibility
                if let Some(ref status) = app_state.status_message {
                    // Show spinner for save/load operations
                    if app_state.is_saving || app_state.is_loading {
                        ui.spinner();
                    }

                    egui::Frame::none()
                        .fill(egui::Color32::from_rgba_unmultiplied(218, 165, 32, 40))
                        .inner_margin(egui::Margin::symmetric(6.0, 2.0))
                        .rounding(egui::Rounding::same(3.0))
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new(status)
                                    .color(egui::Color32::from_rgb(255, 215, 0))
                                    .strong(),
                            );
                        });
                }

                // Processing indicator
                if app_state.is_processing && !app_state.is_saving && !app_state.is_loading {
                    ui.spinner();
                    ui.label("Thinking...");
                }
            });
        });
    });
}

/// Render the narrative panel (main story area).
pub fn render_narrative_panel(ctx: &egui::Context, app_state: &AppState, _current_time: f64) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Adventure Log");
        ui.separator();

        // Scrollable narrative area
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .stick_to_bottom(true)
            .show(ui, |ui| {
                for entry in &app_state.narrative {
                    let color = match entry.entry_type {
                        NarrativeType::DmNarration => egui::Color32::from_rgb(230, 220, 200), // Parchment
                        NarrativeType::PlayerAction => egui::Color32::from_rgb(100, 180, 255), // Blue
                        NarrativeType::NpcDialogue => egui::Color32::from_rgb(200, 200, 150), // Tan
                        NarrativeType::Combat => egui::Color32::from_rgb(255, 100, 100),      // Red
                        NarrativeType::System => egui::Color32::from_rgb(180, 180, 180), // Gray
                    };

                    let prefix = match entry.entry_type {
                        NarrativeType::DmNarration => "",
                        NarrativeType::PlayerAction => "> ",
                        NarrativeType::NpcDialogue => "\"",
                        NarrativeType::Combat => "[Combat] ",
                        NarrativeType::System => "[System] ",
                    };

                    // Split by paragraph breaks (double newlines first, then single)
                    // and render each paragraph with proper visual spacing
                    let text_with_prefix = format!("{}{}", prefix, entry.text);

                    // Check if there are double newlines (proper paragraphs)
                    if text_with_prefix.contains("\n\n") {
                        let paragraphs: Vec<&str> = text_with_prefix.split("\n\n").collect();
                        for (i, paragraph) in paragraphs.iter().enumerate() {
                            let text = paragraph.trim();
                            if !text.is_empty() {
                                ui.add(
                                    egui::Label::new(egui::RichText::new(text).color(color)).wrap(),
                                );
                                // Add space between paragraphs
                                if i < paragraphs.len() - 1 {
                                    ui.add_space(8.0);
                                }
                            }
                        }
                    } else if text_with_prefix.contains('\n') {
                        // Single newlines - render each line with smaller spacing
                        let lines: Vec<&str> = text_with_prefix.split('\n').collect();
                        for (i, line) in lines.iter().enumerate() {
                            let text = line.trim();
                            if !text.is_empty() {
                                ui.add(
                                    egui::Label::new(egui::RichText::new(text).color(color)).wrap(),
                                );
                                if i < lines.len() - 1 {
                                    ui.add_space(4.0);
                                }
                            }
                        }
                    } else {
                        // No newlines - render as a single block
                        ui.add(
                            egui::Label::new(
                                egui::RichText::new(text_with_prefix.trim()).color(color),
                            )
                            .wrap(),
                        );
                    }
                    ui.add_space(12.0);
                }

                // Show streaming text if any
                if !app_state.streaming_text.is_empty() {
                    let streaming_color = egui::Color32::from_rgb(230, 220, 200);

                    // Check if there are double newlines (proper paragraphs)
                    if app_state.streaming_text.contains("\n\n") {
                        let paragraphs: Vec<&str> =
                            app_state.streaming_text.split("\n\n").collect();
                        for (i, paragraph) in paragraphs.iter().enumerate() {
                            let text = paragraph.trim();
                            if !text.is_empty() {
                                ui.add(
                                    egui::Label::new(
                                        egui::RichText::new(text).color(streaming_color).italics(),
                                    )
                                    .wrap(),
                                );
                                if i < paragraphs.len() - 1 {
                                    ui.add_space(8.0);
                                }
                            }
                        }
                    } else if app_state.streaming_text.contains('\n') {
                        let lines: Vec<&str> = app_state.streaming_text.split('\n').collect();
                        for (i, line) in lines.iter().enumerate() {
                            let text = line.trim();
                            if !text.is_empty() {
                                ui.add(
                                    egui::Label::new(
                                        egui::RichText::new(text).color(streaming_color).italics(),
                                    )
                                    .wrap(),
                                );
                                if i < lines.len() - 1 {
                                    ui.add_space(4.0);
                                }
                            }
                        }
                    } else {
                        ui.add(
                            egui::Label::new(
                                egui::RichText::new(app_state.streaming_text.trim())
                                    .color(streaming_color)
                                    .italics(),
                            )
                            .wrap(),
                        );
                    }
                    ui.label(
                        egui::RichText::new("...")
                            .color(egui::Color32::GRAY)
                            .italics(),
                    );
                }
            });
    });
}

/// Render the character panel (right sidebar).
pub fn render_character_panel(ctx: &egui::Context, app_state: &mut AppState) {
    let panel_width = if app_state.character_panel_expanded {
        220.0
    } else {
        120.0
    };

    egui::SidePanel::right("character_panel")
        .min_width(panel_width)
        .max_width(panel_width)
        .resizable(false)
        .show(ctx, |ui| {
            // Header with character name
            ui.horizontal(|ui| {
                // Character name
                ui.heading(
                    egui::RichText::new(&app_state.world.player_name)
                        .color(egui::Color32::from_rgb(218, 165, 32)),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Collapse/expand button
                    let button_text = if app_state.character_panel_expanded {
                        "−"
                    } else {
                        "+"
                    };
                    if ui
                        .small_button(button_text)
                        .on_hover_text(if app_state.character_panel_expanded {
                            "Collapse panel"
                        } else {
                            "Expand panel"
                        })
                        .clicked()
                    {
                        app_state.character_panel_expanded = !app_state.character_panel_expanded;
                    }
                });
            });

            // HP Bar (always shown)
            let hp = &app_state.world.player_hp;
            let hp_ratio = hp.ratio().clamp(0.0, 1.0);
            let hp_color = if hp_ratio > 0.5 {
                egui::Color32::from_rgb(34, 139, 34)
            } else if hp_ratio > 0.25 {
                egui::Color32::from_rgb(204, 153, 0)
            } else {
                egui::Color32::from_rgb(178, 34, 34)
            };

            let progress_bar = egui::ProgressBar::new(hp_ratio)
                .text(format!("{}/{}", hp.current.max(0), hp.maximum))
                .fill(hp_color);
            ui.add(progress_bar);

            // Collapsed view - just show basic stats
            if !app_state.character_panel_expanded {
                ui.horizontal(|ui| {
                    ui.label(format!("AC:{}", app_state.world.player_ac));
                    ui.label(format!("Lv{}", app_state.world.player_level));
                });
                return;
            }

            // Expanded view
            if let Some(ref class) = app_state.world.player_class {
                ui.label(format!("Level {} {}", app_state.world.player_level, class));
            }

            // Death saves if at 0 HP
            if hp.current <= 0 {
                ui.add_space(4.0);
                let saves = &app_state.world.death_saves;
                ui.horizontal(|ui| {
                    ui.label("Death Saves: ");
                    for i in 0..3 {
                        let filled = i < saves.successes;
                        let color = if filled {
                            egui::Color32::from_rgb(34, 139, 34)
                        } else {
                            egui::Color32::DARK_GRAY
                        };
                        ui.colored_label(color, if filled { "[X]" } else { "[ ]" });
                    }
                    ui.label("/");
                    for i in 0..3 {
                        let filled = i < saves.failures;
                        let color = if filled {
                            egui::Color32::RED
                        } else {
                            egui::Color32::DARK_GRAY
                        };
                        ui.colored_label(color, if filled { "[X]" } else { "[ ]" });
                    }
                });
            }

            ui.separator();

            // Combat stats
            ui.horizontal(|ui| {
                ui.label("AC:");
                ui.label(egui::RichText::new(format!("{}", app_state.world.player_ac)).strong());
                ui.separator();
                ui.label("Init:");
                let init = app_state.world.player_initiative;
                ui.label(
                    egui::RichText::new(if init >= 0 {
                        format!("+{init}")
                    } else {
                        format!("{init}")
                    })
                    .strong(),
                );
            });

            ui.horizontal(|ui| {
                ui.label("Speed:");
                ui.label(format!("{} ft", app_state.world.player_speed));
            });

            // Conditions
            if !app_state.world.conditions.is_empty() {
                ui.separator();
                ui.label(
                    egui::RichText::new("Conditions")
                        .color(egui::Color32::YELLOW)
                        .strong(),
                );
                for condition in &app_state.world.conditions {
                    ui.label(format!("  {condition}"));
                }
            }

            ui.separator();

            // Equipment
            ui.label(egui::RichText::new("Equipment").strong());
            if let Some(ref weapon) = app_state.world.equipped_weapon {
                ui.label(format!("  {weapon}"));
            } else {
                ui.label("  (no weapon)");
            }
            if let Some(ref armor) = app_state.world.equipped_armor {
                ui.label(format!("  {armor}"));
            }

            // Spell Slots (if spellcaster)
            let has_spell_slots = app_state
                .world
                .spell_slots
                .iter()
                .any(|(_, total)| *total > 0);
            if has_spell_slots || !app_state.world.cantrips.is_empty() {
                ui.separator();
                ui.label(egui::RichText::new("Spellcasting").strong());

                // Show spell save DC and attack bonus
                if let (Some(dc), Some(atk)) = (
                    app_state.world.spell_save_dc,
                    app_state.world.spell_attack_bonus,
                ) {
                    ui.horizontal(|ui| {
                        ui.label(format!("DC {} | ", dc));
                        ui.label(format!("+{} atk", atk));
                    });
                }

                // Spell slots in compact format
                if has_spell_slots {
                    ui.horizontal_wrapped(|ui| {
                        ui.label("Slots: ");
                        for (i, (available, total)) in
                            app_state.world.spell_slots.iter().enumerate()
                        {
                            if *total > 0 {
                                let level = i + 1;
                                let color = if *available > 0 {
                                    egui::Color32::from_rgb(100, 180, 255)
                                } else {
                                    egui::Color32::GRAY
                                };
                                ui.label(
                                    egui::RichText::new(format!(
                                        "L{}: {}/{}",
                                        level, available, total
                                    ))
                                    .color(color)
                                    .small(),
                                );
                            }
                        }
                    });
                }

                // Cantrips count
                if !app_state.world.cantrips.is_empty() {
                    ui.label(
                        egui::RichText::new(format!(
                            "  {} cantrips",
                            app_state.world.cantrips.len()
                        ))
                        .small()
                        .color(egui::Color32::LIGHT_GRAY),
                    );
                }
            }

            ui.separator();

            // Gold
            ui.horizontal(|ui| {
                ui.label("Gold:");
                ui.label(
                    egui::RichText::new(format!("{:.0} gp", app_state.world.gold))
                        .color(egui::Color32::from_rgb(218, 165, 32)),
                );
            });

            // Inventory section (collapsible)
            ui.separator();
            ui.collapsing("Inventory", |ui| {
                if app_state.world.inventory_items.is_empty() {
                    ui.label(
                        egui::RichText::new("Empty")
                            .italics()
                            .color(egui::Color32::GRAY),
                    );
                } else {
                    egui::ScrollArea::vertical()
                        .max_height(150.0)
                        .show(ui, |ui| {
                            for item in &app_state.world.inventory_items {
                                let name = if item.quantity > 1 {
                                    format!("{} (x{})", item.name, item.quantity)
                                } else {
                                    item.name.clone()
                                };
                                let color = if item.magical {
                                    egui::Color32::from_rgb(138, 43, 226)
                                } else {
                                    egui::Color32::LIGHT_GRAY
                                };
                                ui.label(
                                    egui::RichText::new(format!("• {name}"))
                                        .color(color)
                                        .small(),
                                );
                            }
                        });
                }
            });
        });
}

/// Render the combat panel (shows when in combat).
pub fn render_combat_panel(ctx: &egui::Context, app_state: &AppState) {
    if !app_state.in_combat {
        return;
    }

    if let Some(ref combat) = app_state.world.combat {
        // Position combat window below top bar, use responsive width
        let screen = ctx.screen_rect();
        let max_width = (screen.width() * 0.3).min(250.0).max(150.0);

        egui::Window::new("Combat")
            .collapsible(true)
            .resizable(true)
            .default_pos([10.0, 50.0])
            .default_width(max_width)
            .show(ctx, |ui| {
                ui.label(format!("Round {}", combat.round));
                ui.separator();

                // Initiative order
                for (i, combatant) in combat.combatants.iter().enumerate() {
                    let is_current = i == combat.turn_index;
                    let prefix = if is_current { "> " } else { "  " };

                    let hp_text = format!("{}/{}", combatant.current_hp, combatant.max_hp);
                    let hp_color = if combatant.current_hp as f32 / combatant.max_hp as f32 > 0.5 {
                        egui::Color32::GREEN
                    } else if combatant.current_hp > 0 {
                        egui::Color32::YELLOW
                    } else {
                        egui::Color32::RED
                    };

                    ui.horizontal(|ui| {
                        if is_current {
                            ui.label(
                                egui::RichText::new(prefix)
                                    .color(egui::Color32::YELLOW)
                                    .strong(),
                            );
                        } else {
                            ui.label(prefix);
                        }

                        let name_color = if combatant.is_player {
                            egui::Color32::from_rgb(100, 180, 255)
                        } else if combatant.is_ally {
                            egui::Color32::GREEN
                        } else {
                            egui::Color32::RED
                        };

                        ui.label(egui::RichText::new(&combatant.name).color(name_color));
                        ui.label(format!("({})", combatant.initiative));
                        ui.label(egui::RichText::new(hp_text).color(hp_color));
                    });
                }

                if app_state.is_player_turn {
                    ui.separator();
                    ui.label(
                        egui::RichText::new("Your turn!")
                            .color(egui::Color32::YELLOW)
                            .strong(),
                    );
                }
            });
    }
}

/// Render the game over screen.
pub fn render_game_over(
    ctx: &egui::Context,
    app_state: &AppState,
    next_phase: &mut NextState<GamePhase>,
) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(100.0);

            ui.heading(
                egui::RichText::new("Game Over")
                    .size(48.0)
                    .color(egui::Color32::RED),
            );

            ui.add_space(20.0);

            ui.label(format!("{} has fallen.", app_state.world.player_name));

            ui.add_space(40.0);

            if ui.button("Return to Main Menu").clicked() {
                next_phase.set(GamePhase::MainMenu);
            }
        });
    });
}
