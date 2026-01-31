//! Overlay windows for inventory, character sheet, etc.

use bevy_egui::egui;
use dnd_core::spells::get_spell;
use dnd_core::world::{Ability, QuestStatus};

use crate::state::{ActiveOverlay, AppState, CharacterSaveList, GameSaveList};

/// Render the inventory overlay.
pub fn render_inventory(ctx: &egui::Context, app_state: &AppState) {
    // Use responsive sizing based on available screen
    let screen = ctx.screen_rect();
    let width = (screen.width() * 0.8).min(400.0).max(280.0);
    let height = (screen.height() * 0.7).min(450.0).max(300.0);

    egui::Window::new("Inventory")
        .collapsible(false)
        .resizable(true)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .default_size([width, height])
        .max_size([500.0, 600.0])
        .show(ctx, |ui| {
            // Gold
            ui.horizontal(|ui| {
                ui.label("Gold:");
                ui.label(
                    egui::RichText::new(format!("{:.0} gp", app_state.world.gold))
                        .color(egui::Color32::from_rgb(218, 165, 32))
                        .strong(),
                );
            });

            ui.separator();

            // Equipped items
            ui.heading("Equipped");
            ui.indent("equipped", |ui| {
                if let Some(ref weapon) = app_state.world.equipped_weapon {
                    ui.label(format!("Main Hand: {weapon}"));
                } else {
                    ui.label("Main Hand: (empty)");
                }
                if let Some(ref armor) = app_state.world.equipped_armor {
                    ui.label(format!("Armor: {armor}"));
                } else {
                    ui.label("Armor: (none)");
                }
            });

            ui.separator();

            // Inventory items
            ui.heading("Items");

            if app_state.world.inventory_items.is_empty() {
                ui.label(egui::RichText::new("Your pack is empty.").italics());
            } else {
                egui::ScrollArea::vertical()
                    .max_height(300.0)
                    .show(ui, |ui| {
                        for item in &app_state.world.inventory_items {
                            ui.horizontal(|ui| {
                                let name = if item.quantity > 1 {
                                    format!("{} x{}", item.name, item.quantity)
                                } else {
                                    item.name.clone()
                                };

                                let color = if item.magical {
                                    egui::Color32::from_rgb(138, 43, 226) // Purple for magical
                                } else {
                                    egui::Color32::WHITE
                                };

                                ui.label(egui::RichText::new(name).color(color));

                                if item.weight > 0.0 {
                                    ui.label(
                                        egui::RichText::new(format!("({:.1} lb)", item.weight))
                                            .color(egui::Color32::GRAY)
                                            .small(),
                                    );
                                }
                            });

                            if let Some(ref desc) = item.description {
                                ui.indent("item_desc", |ui| {
                                    ui.label(
                                        egui::RichText::new(desc)
                                            .color(egui::Color32::GRAY)
                                            .small()
                                            .italics(),
                                    );
                                });
                            }
                        }
                    });
            }

            ui.separator();
            ui.label(
                egui::RichText::new("Press I or Escape to close")
                    .small()
                    .color(egui::Color32::GRAY),
            );
        });
}

/// Render the character sheet overlay.
pub fn render_character_sheet(ctx: &egui::Context, app_state: &mut AppState) {
    // Use responsive sizing based on available screen
    let screen = ctx.screen_rect();
    let width = (screen.width() * 0.85).min(500.0).max(320.0);
    let height = (screen.height() * 0.8).min(550.0).max(350.0);

    egui::Window::new("Character Sheet")
        .collapsible(false)
        .resizable(true)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .default_size([width, height])
        .max_size([600.0, 700.0])
        .show(ctx, |ui| {
            // Header
            ui.horizontal(|ui| {
                ui.heading(
                    egui::RichText::new(&app_state.world.player_name)
                        .color(egui::Color32::from_rgb(218, 165, 32)),
                );
                if let Some(ref class) = app_state.world.player_class {
                    ui.label(format!("Level {} {}", app_state.world.player_level, class));
                }
            });

            ui.separator();

            // Two-column layout
            ui.columns(2, |columns| {
                // Left column: Ability scores
                columns[0].heading("Ability Scores");
                columns[0].separator();

                let abilities = [
                    (Ability::Strength, "STR"),
                    (Ability::Dexterity, "DEX"),
                    (Ability::Constitution, "CON"),
                    (Ability::Intelligence, "INT"),
                    (Ability::Wisdom, "WIS"),
                    (Ability::Charisma, "CHA"),
                ];

                for (ability, abbr) in abilities {
                    let score = app_state.world.ability_scores.get(ability);
                    let modifier = app_state.world.ability_scores.modifier(ability);
                    let mod_str = if modifier >= 0 {
                        format!("+{modifier}")
                    } else {
                        format!("{modifier}")
                    };

                    columns[0].horizontal(|ui| {
                        ui.label(egui::RichText::new(abbr).strong());
                        ui.label(format!("{score:2}"));
                        ui.label(
                            egui::RichText::new(format!("({mod_str})"))
                                .color(egui::Color32::from_rgb(100, 180, 255)),
                        );
                    });
                }

                columns[0].add_space(10.0);
                columns[0].separator();
                columns[0].heading("Combat Stats");
                columns[0].separator();

                columns[0].horizontal(|ui| {
                    ui.label("Armor Class:");
                    ui.label(
                        egui::RichText::new(format!("{}", app_state.world.player_ac)).strong(),
                    );
                });

                columns[0].horizontal(|ui| {
                    ui.label("Initiative:");
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

                columns[0].horizontal(|ui| {
                    ui.label("Speed:");
                    ui.label(format!("{} ft", app_state.world.player_speed));
                });

                columns[0].horizontal(|ui| {
                    ui.label("Proficiency Bonus:");
                    ui.label(
                        egui::RichText::new(format!("+{}", app_state.world.proficiency_bonus))
                            .strong(),
                    );
                });

                // Right column: Skills
                columns[1].heading("Skills");
                columns[1].separator();

                let mut skills: Vec<_> = app_state.world.skill_proficiencies.iter().collect();
                skills.sort_by_key(|(skill, _)| skill.name());

                for (skill, proficiency) in skills {
                    columns[1].horizontal(|ui| {
                        let is_proficient = proficiency != "NotProficient";
                        let marker = if is_proficient { "[*]" } else { "[ ]" };
                        let color = if is_proficient {
                            egui::Color32::GREEN
                        } else {
                            egui::Color32::DARK_GRAY
                        };

                        ui.label(egui::RichText::new(marker).color(color));
                        ui.label(skill.name());
                    });
                }
            });

            ui.separator();

            // Spellcasting section (if spellcaster)
            let has_spells = !app_state.world.cantrips.is_empty()
                || !app_state.world.known_spells.is_empty()
                || app_state.world.spell_slots.iter().any(|(_, t)| *t > 0);

            if has_spells {
                ui.heading("Spellcasting");

                // Spellcasting stats
                ui.horizontal(|ui| {
                    if let Some(ref ability) = app_state.world.spellcasting_ability {
                        ui.label(format!("Ability: {}", ability));
                    }
                    if let Some(dc) = app_state.world.spell_save_dc {
                        ui.separator();
                        ui.label(format!("Save DC: {}", dc));
                    }
                    if let Some(atk) = app_state.world.spell_attack_bonus {
                        ui.separator();
                        ui.label(format!("Attack: +{}", atk));
                    }
                });

                // Spell slots
                let has_slots = app_state.world.spell_slots.iter().any(|(_, t)| *t > 0);
                if has_slots {
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new("Spell Slots").strong());
                    ui.horizontal_wrapped(|ui| {
                        for (i, (available, total)) in
                            app_state.world.spell_slots.iter().enumerate()
                        {
                            if *total > 0 {
                                let level = i + 1;
                                let color = if *available > 0 {
                                    egui::Color32::from_rgb(100, 180, 255)
                                } else {
                                    egui::Color32::DARK_GRAY
                                };
                                ui.label(
                                    egui::RichText::new(format!(
                                        "Lv{}: {}/{}",
                                        level, available, total
                                    ))
                                    .color(color),
                                );
                                ui.add_space(8.0);
                            }
                        }
                    });
                }

                // Cantrips - clickable to view details
                if !app_state.world.cantrips.is_empty() {
                    ui.add_space(4.0);
                    let cantrips = app_state.world.cantrips.clone();
                    ui.collapsing(
                        egui::RichText::new(format!(
                            "Cantrips ({}) - click for details",
                            cantrips.len()
                        ))
                        .strong(),
                        |ui| {
                            for cantrip in &cantrips {
                                if ui.small_button(format!("• {}", cantrip)).clicked() {
                                    app_state.viewing_spell = Some(cantrip.clone());
                                }
                            }
                        },
                    );
                }

                // Known/Prepared Spells - clickable to view details
                if !app_state.world.known_spells.is_empty() {
                    ui.add_space(4.0);
                    let spells = app_state.world.known_spells.clone();
                    ui.collapsing(
                        egui::RichText::new(format!(
                            "Spells ({}) - click for details",
                            spells.len()
                        ))
                        .strong(),
                        |ui| {
                            egui::ScrollArea::vertical()
                                .max_height(150.0)
                                .show(ui, |ui| {
                                    for spell in &spells {
                                        if ui.small_button(format!("• {}", spell)).clicked() {
                                            app_state.viewing_spell = Some(spell.clone());
                                        }
                                    }
                                });
                        },
                    );
                }

                ui.separator();
            }

            // Conditions
            if !app_state.world.conditions.is_empty() {
                ui.heading("Active Conditions");
                ui.horizontal_wrapped(|ui| {
                    for condition in &app_state.world.conditions {
                        ui.label(
                            egui::RichText::new(format!("{condition}"))
                                .color(egui::Color32::YELLOW)
                                .background_color(egui::Color32::from_rgb(60, 50, 40)),
                        );
                    }
                });
                ui.separator();
            }

            ui.label(
                egui::RichText::new("Press C or Escape to close")
                    .small()
                    .color(egui::Color32::GRAY),
            );
        });
}

/// Render the quest log overlay.
pub fn render_quest_log(ctx: &egui::Context, app_state: &AppState) {
    let screen = ctx.screen_rect();
    let width = (screen.width() * 0.8).min(450.0).max(280.0);
    let height = (screen.height() * 0.7).min(450.0).max(300.0);

    egui::Window::new("Quest Log")
        .collapsible(false)
        .resizable(true)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .default_size([width, height])
        .max_size([550.0, 600.0])
        .show(ctx, |ui| {
            if app_state.world.quests.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(50.0);
                    ui.label(
                        egui::RichText::new("No quests yet.")
                            .italics()
                            .color(egui::Color32::GRAY),
                    );
                    ui.add_space(10.0);
                    ui.label("Your adventure awaits...");
                });
            } else {
                // Active quests
                let active_quests: Vec<_> = app_state
                    .world
                    .quests
                    .iter()
                    .filter(|q| q.status == QuestStatus::Active)
                    .collect();

                if !active_quests.is_empty() {
                    ui.heading(egui::RichText::new("Active Quests").color(egui::Color32::YELLOW));
                    ui.separator();

                    for quest in active_quests {
                        ui.group(|ui| {
                            ui.label(egui::RichText::new(&quest.name).strong());
                            ui.label(&quest.description);

                            // Objectives
                            if !quest.objectives.is_empty() {
                                ui.add_space(4.0);
                                for obj in &quest.objectives {
                                    let marker = if obj.completed { "[X]" } else { "[ ]" };
                                    let color = if obj.completed {
                                        egui::Color32::GREEN
                                    } else {
                                        egui::Color32::WHITE
                                    };
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "{} {}",
                                            marker, obj.description
                                        ))
                                        .color(color),
                                    );
                                }
                            }
                        });
                        ui.add_space(4.0);
                    }
                }

                // Completed quests
                let completed_quests: Vec<_> = app_state
                    .world
                    .quests
                    .iter()
                    .filter(|q| q.status == QuestStatus::Completed)
                    .collect();

                if !completed_quests.is_empty() {
                    ui.add_space(10.0);
                    ui.heading(egui::RichText::new("Completed Quests").color(egui::Color32::GREEN));
                    ui.separator();

                    for quest in completed_quests {
                        ui.label(
                            egui::RichText::new(format!("[Done] {}", quest.name))
                                .color(egui::Color32::from_rgb(100, 180, 100)),
                        );
                    }
                }

                // Failed quests
                let failed_quests: Vec<_> = app_state
                    .world
                    .quests
                    .iter()
                    .filter(|q| {
                        q.status == QuestStatus::Failed || q.status == QuestStatus::Abandoned
                    })
                    .collect();

                if !failed_quests.is_empty() {
                    ui.add_space(10.0);
                    ui.heading(egui::RichText::new("Failed Quests").color(egui::Color32::RED));
                    ui.separator();

                    for quest in failed_quests {
                        ui.label(
                            egui::RichText::new(format!("[Failed] {}", quest.name))
                                .color(egui::Color32::from_rgb(180, 100, 100)),
                        );
                    }
                }
            }

            ui.separator();
            ui.label(
                egui::RichText::new("Press Shift+Q or Escape to close")
                    .small()
                    .color(egui::Color32::GRAY),
            );
        });
}

/// Render the help overlay.
pub fn render_help(ctx: &egui::Context) {
    let screen = ctx.screen_rect();
    let width = (screen.width() * 0.8).min(450.0).max(300.0);
    let height = (screen.height() * 0.75).min(480.0).max(320.0);

    egui::Window::new("Help")
        .collapsible(false)
        .resizable(true)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .default_size([width, height])
        .max_size([550.0, 600.0])
        .show(ctx, |ui| {
            ui.heading("D&D: AI Dungeon Master");
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("How to Play");
                ui.label("Type natural language commands to interact with the world.");
                ui.label("The AI Dungeon Master will respond to your actions.");
                ui.add_space(10.0);

                ui.heading("Example Commands");
                ui.label("• \"I look around the room\"");
                ui.label("• \"I attack the goblin with my sword\"");
                ui.label("• \"I try to pick the lock\"");
                ui.label("• \"I cast fireball at the enemies\"");
                ui.label("• \"I search the chest\"");
                ui.add_space(10.0);

                ui.heading("Keyboard Shortcuts");
                ui.add_space(4.0);

                ui.label(
                    egui::RichText::new("Global:")
                        .strong()
                        .color(egui::Color32::from_rgb(218, 165, 32)),
                );
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Ctrl+Q / Cmd+Q").strong());
                    ui.label("- Quit game");
                });
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Ctrl+S / Cmd+S").strong());
                    ui.label("- Quick Save");
                });
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Escape").strong());
                    ui.label("- Close overlay / Cancel");
                });

                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new("Input:")
                        .strong()
                        .color(egui::Color32::from_rgb(218, 165, 32)),
                );
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Enter").strong());
                    ui.label("- Send command");
                });
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Up / Down").strong());
                    ui.label("- Browse command history");
                });

                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new("Overlays (when not typing):")
                        .strong()
                        .color(egui::Color32::from_rgb(218, 165, 32)),
                );
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("I").strong());
                    ui.label("- Inventory");
                });
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("C").strong());
                    ui.label("- Character Sheet");
                });
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Shift+Q").strong());
                    ui.label("- Quest Log");
                });
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("F1 / ?").strong());
                    ui.label("- Help (this screen)");
                });

                ui.add_space(10.0);
                ui.heading("Tips");
                ui.label("• Be descriptive - the DM understands natural language");
                ui.label("• Check your inventory before adventures");
                ui.label("• Save often using Ctrl+S");
                ui.label("• Use the quick action buttons for common actions");
            });

            ui.separator();
            ui.label(
                egui::RichText::new("Press F1, ?, or Escape to close")
                    .small()
                    .color(egui::Color32::GRAY),
            );
        });
}

/// Render the settings overlay. Returns true if user wants to return to main menu.
pub fn render_settings(ctx: &egui::Context, app_state: &mut AppState) -> bool {
    let mut return_to_menu = false;

    let screen = ctx.screen_rect();
    let width = (screen.width() * 0.75).min(400.0).max(280.0);
    let height = (screen.height() * 0.65).min(380.0).max(280.0);

    egui::Window::new("Settings")
        .collapsible(false)
        .resizable(true)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .default_size([width, height])
        .max_size([500.0, 500.0])
        .show(ctx, |ui| {
            ui.heading("Settings");
            ui.separator();

            // Display section
            ui.collapsing(egui::RichText::new("Display").strong(), |ui| {
                ui.horizontal(|ui| {
                    ui.label("Character panel:");
                    if ui
                        .selectable_label(app_state.character_panel_expanded, "Expanded")
                        .clicked()
                    {
                        app_state.character_panel_expanded = true;
                    }
                    if ui
                        .selectable_label(!app_state.character_panel_expanded, "Collapsed")
                        .clicked()
                    {
                        app_state.character_panel_expanded = false;
                    }
                });
            });

            ui.add_space(8.0);

            // Save files section
            ui.collapsing(egui::RichText::new("Save Files").strong(), |ui| {
                ui.label("Save directory: saves/");
                ui.label("Character saves: saves/characters/");

                ui.add_space(4.0);

                if ui.button("Open saves folder").clicked() {
                    #[cfg(target_os = "macos")]
                    {
                        let _ = std::process::Command::new("open").arg("saves").spawn();
                    }
                    #[cfg(target_os = "windows")]
                    {
                        let _ = std::process::Command::new("explorer").arg("saves").spawn();
                    }
                    #[cfg(target_os = "linux")]
                    {
                        let _ = std::process::Command::new("xdg-open").arg("saves").spawn();
                    }
                }
            });

            ui.add_space(8.0);

            // Keyboard shortcuts
            ui.collapsing(egui::RichText::new("Keyboard Shortcuts").strong(), |ui| {
                ui.label("Ctrl+S / Cmd+S - Save game");
                ui.label("Ctrl+Q / Cmd+Q - Quit game");
                ui.label("I - Inventory");
                ui.label("C - Character sheet");
                ui.label("Shift+Q - Quest log");
                ui.label("F1 / ? - Help");
                ui.label("Escape - Close overlay");
            });

            ui.add_space(8.0);

            // About section
            ui.collapsing(egui::RichText::new("About").strong(), |ui| {
                ui.label(
                    egui::RichText::new("D&D: AI Dungeon Master")
                        .color(egui::Color32::from_rgb(218, 165, 32)),
                );
                ui.label("A text-based adventure powered by AI");
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new("Built with Rust, Bevy, and Claude")
                        .small()
                        .color(egui::Color32::GRAY),
                );
            });

            ui.add_space(16.0);
            ui.separator();

            // Game actions
            ui.horizontal(|ui| {
                if ui.button("Return to Main Menu").clicked() {
                    return_to_menu = true;
                    app_state.overlay = ActiveOverlay::None;
                }

                if ui
                    .button(
                        egui::RichText::new("Quit Game")
                            .color(egui::Color32::from_rgb(200, 100, 100)),
                    )
                    .clicked()
                {
                    std::process::exit(0);
                }
            });

            ui.add_space(8.0);
            ui.label(
                egui::RichText::new("Press Escape to close")
                    .small()
                    .color(egui::Color32::GRAY),
            );
        });

    return_to_menu
}

/// Render the load character overlay.
pub fn render_load_character(
    ctx: &egui::Context,
    app_state: &mut AppState,
    save_list: &mut CharacterSaveList,
) -> Option<dnd_core::world::Character> {
    let mut selected_character = None;

    let screen = ctx.screen_rect();
    let width = (screen.width() * 0.8).min(450.0).max(300.0);
    let height = (screen.height() * 0.65).min(400.0).max(280.0);

    egui::Window::new("Load Character")
        .collapsible(false)
        .resizable(true)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .default_size([width, height])
        .max_size([550.0, 500.0])
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Saved Characters");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Refresh").clicked() {
                        // Reset to trigger a reload
                        save_list.saves.clear();
                        save_list.loaded = false;
                        save_list.loading = false;
                        save_list.error = None;
                        save_list.selected = None;
                    }
                });
            });
            ui.separator();

            if save_list.loading {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("Loading saved characters...");
                });
            } else if let Some(ref err) = save_list.error {
                ui.colored_label(egui::Color32::RED, err);
            } else if save_list.saves.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(50.0);
                    ui.label(
                        egui::RichText::new("No saved characters found.")
                            .italics()
                            .color(egui::Color32::GRAY),
                    );
                    ui.add_space(10.0);
                    ui.label("Create a character first!");
                });
            } else {
                egui::ScrollArea::vertical()
                    .max_height(280.0)
                    .show(ui, |ui| {
                        for (i, save) in save_list.saves.iter().enumerate() {
                            let is_selected = save_list.selected == Some(i);
                            let meta = &save.metadata;

                            let text = format!(
                                "{} - Level {} {} {}{}",
                                meta.name,
                                meta.level,
                                meta.race,
                                meta.class,
                                if meta.has_backstory {
                                    " (has backstory)"
                                } else {
                                    ""
                                }
                            );

                            if ui.selectable_label(is_selected, text).clicked() {
                                save_list.selected = Some(i);
                            }
                        }
                    });

                ui.separator();

                ui.horizontal(|ui| {
                    let can_load = save_list.selected.is_some();

                    if ui
                        .add_enabled(can_load, egui::Button::new("Load & Play"))
                        .clicked()
                    {
                        if let Some(idx) = save_list.selected {
                            let path = save_list.saves[idx].path.clone();
                            // Load the character using the shared runtime
                            match crate::runtime::RUNTIME
                                .block_on(dnd_core::SavedCharacter::load_json(&path))
                            {
                                Ok(saved) => {
                                    selected_character = Some(saved.character);
                                    app_state.overlay = ActiveOverlay::None;
                                }
                                Err(e) => {
                                    save_list.error = Some(format!("Failed to load: {e}"));
                                }
                            }
                        }
                    }

                    if ui.button("Cancel").clicked() {
                        app_state.overlay = ActiveOverlay::None;
                    }
                });
            }

            ui.separator();
            ui.label(
                egui::RichText::new("Press Escape to close")
                    .small()
                    .color(egui::Color32::GRAY),
            );
        });

    selected_character
}

/// Render the load game overlay. Returns the path to load if a game is selected.
pub fn render_load_game(
    ctx: &egui::Context,
    app_state: &mut AppState,
    save_list: &mut GameSaveList,
) -> Option<String> {
    let mut selected_path = None;

    let screen = ctx.screen_rect();
    let width = (screen.width() * 0.8).min(480.0).max(300.0);
    let height = (screen.height() * 0.65).min(400.0).max(280.0);

    egui::Window::new("Load Game")
        .collapsible(false)
        .resizable(true)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .default_size([width, height])
        .max_size([600.0, 500.0])
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Saved Games");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Refresh").clicked() {
                        save_list.saves.clear();
                        save_list.loaded = false;
                        save_list.loading = false;
                        save_list.error = None;
                        save_list.selected = None;
                    }
                });
            });
            ui.separator();

            if save_list.loading {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("Loading saved games...");
                });
            } else if let Some(ref err) = save_list.error {
                ui.colored_label(egui::Color32::RED, err);
            } else if save_list.saves.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(50.0);
                    ui.label(
                        egui::RichText::new("No saved games found.")
                            .italics()
                            .color(egui::Color32::GRAY),
                    );
                    ui.add_space(10.0);
                    ui.label("Start a New Game and save your progress!");
                });
            } else {
                egui::ScrollArea::vertical()
                    .max_height(280.0)
                    .show(ui, |ui| {
                        for (i, save) in save_list.saves.iter().enumerate() {
                            let is_selected = save_list.selected == Some(i);

                            let text = format!(
                                "{} - {} (Level {})\nSaved: {}",
                                save.campaign_name,
                                save.character_name,
                                save.character_level,
                                save.saved_at
                            );

                            if ui.selectable_label(is_selected, text).clicked() {
                                save_list.selected = Some(i);
                            }
                        }
                    });

                ui.separator();

                ui.horizontal(|ui| {
                    let can_load = save_list.selected.is_some();

                    if ui
                        .add_enabled(can_load, egui::Button::new("Load Game"))
                        .clicked()
                    {
                        if let Some(idx) = save_list.selected {
                            selected_path = Some(save_list.saves[idx].path.clone());
                        }
                    }

                    if ui.button("Cancel").clicked() {
                        app_state.overlay = ActiveOverlay::None;
                    }
                });
            }

            ui.separator();
            ui.label(
                egui::RichText::new("Press Escape to close")
                    .small()
                    .color(egui::Color32::GRAY),
            );
        });

    selected_path
}

/// Render the spell detail popup.
pub fn render_spell_detail(ctx: &egui::Context, app_state: &mut AppState) {
    let spell_name = match &app_state.viewing_spell {
        Some(name) => name.clone(),
        None => return,
    };

    let spell_data = get_spell(&spell_name);

    egui::Window::new("Spell Details")
        .collapsible(false)
        .resizable(true)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .default_size([400.0, 350.0])
        .max_size([500.0, 500.0])
        .show(ctx, |ui| {
            match spell_data {
                Some(spell) => {
                    // Spell name and level
                    ui.heading(
                        egui::RichText::new(&spell.name)
                            .color(egui::Color32::from_rgb(100, 180, 255)),
                    );

                    let level_text = if spell.level == 0 {
                        format!("{} cantrip", spell.school.name())
                    } else {
                        let suffix = match spell.level {
                            1 => "st",
                            2 => "nd",
                            3 => "rd",
                            _ => "th",
                        };
                        format!("{}{}-level {}", spell.level, suffix, spell.school.name())
                    };

                    ui.label(
                        egui::RichText::new(level_text)
                            .italics()
                            .color(egui::Color32::LIGHT_GRAY),
                    );

                    if spell.ritual {
                        ui.label(
                            egui::RichText::new("(ritual)")
                                .italics()
                                .color(egui::Color32::from_rgb(180, 140, 255)),
                        );
                    }

                    ui.separator();

                    // Spell properties in a grid
                    egui::Grid::new("spell_properties")
                        .num_columns(2)
                        .spacing([20.0, 4.0])
                        .show(ui, |ui| {
                            ui.label(egui::RichText::new("Casting Time:").strong());
                            ui.label(spell.casting_time.description());
                            ui.end_row();

                            ui.label(egui::RichText::new("Range:").strong());
                            ui.label(spell.range.description());
                            ui.end_row();

                            ui.label(egui::RichText::new("Components:").strong());
                            ui.label(spell.components.description());
                            ui.end_row();

                            ui.label(egui::RichText::new("Duration:").strong());
                            let duration_text = if spell.concentration {
                                format!("Concentration, {}", spell.duration.description())
                            } else {
                                spell.duration.description().to_string()
                            };
                            ui.label(duration_text);
                            ui.end_row();
                        });

                    ui.separator();

                    // Description
                    egui::ScrollArea::vertical()
                        .max_height(180.0)
                        .show(ui, |ui| {
                            ui.label(&spell.description);
                        });

                    // Combat info if applicable
                    if spell.damage_dice.is_some()
                        || spell.healing_dice.is_some()
                        || spell.save_type.is_some()
                    {
                        ui.separator();
                        ui.label(egui::RichText::new("Combat").strong());

                        if let Some(ref dice) = spell.damage_dice {
                            let damage_text = if let Some(ref dtype) = spell.damage_type {
                                format!("Damage: {} {}", dice, dtype.name())
                            } else {
                                format!("Damage: {}", dice)
                            };
                            ui.label(damage_text);
                        }

                        if let Some(ref dice) = spell.healing_dice {
                            ui.label(format!("Healing: {}", dice));
                        }

                        if let Some(ref save) = spell.save_type {
                            let save_text = if let Some(ref effect) = spell.save_effect {
                                format!("{} save: {}", save.abbreviation(), effect)
                            } else {
                                format!("{} save", save.abbreviation())
                            };
                            ui.label(save_text);
                        }

                        if let Some(ref attack) = spell.attack_type {
                            let attack_name = match attack {
                                dnd_core::spells::SpellAttackType::Melee => "Melee",
                                dnd_core::spells::SpellAttackType::Ranged => "Ranged",
                            };
                            ui.label(format!("Attack: {} spell attack", attack_name));
                        }
                    }
                }
                None => {
                    ui.heading(&spell_name);
                    ui.separator();
                    ui.label(
                        egui::RichText::new("Spell details not found in database.")
                            .italics()
                            .color(egui::Color32::GRAY),
                    );
                    ui.add_space(8.0);
                    ui.label(
                        "This spell may be from a source not yet added to the SRD 5.2 database.",
                    );
                }
            }

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Close").clicked() {
                    app_state.viewing_spell = None;
                }
                ui.label(
                    egui::RichText::new("(or press Escape)")
                        .small()
                        .color(egui::Color32::GRAY),
                );
            });
        });

    // Close on Escape key
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        app_state.viewing_spell = None;
    }
}
