//! UI module - egui-based interface panels.

mod input;
mod overlays;
mod panels;

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::character_creation::{CharacterCreation, ReadyToStart};
use crate::state::{
    ActiveOverlay, AppState, CharacterSaveList, GamePhase, GameSaveInfo, GameSaveList,
    PendingCharacterList, PendingGameList, PendingGameLoad,
};

/// Main UI system - renders all egui panels.
pub fn main_ui_system(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut app_state: ResMut<AppState>,
    game_phase: Res<State<GamePhase>>,
    mut next_phase: ResMut<NextState<GamePhase>>,
    mut char_creation: Option<ResMut<CharacterCreation>>,
    mut char_save_list: Option<ResMut<CharacterSaveList>>,
    mut game_save_list: Option<ResMut<GameSaveList>>,
    time: Res<Time>,
) {
    let ctx = contexts.ctx_mut();

    // Configure egui style
    configure_style(ctx);

    match game_phase.get() {
        GamePhase::MainMenu => {
            panels::render_main_menu(ctx, &mut next_phase, &mut app_state);

            // Render overlays (Settings and LoadCharacter can be accessed from main menu)
            match app_state.overlay {
                ActiveOverlay::Settings => {
                    overlays::render_settings(ctx, &mut app_state);
                }
                ActiveOverlay::LoadCharacter => {
                    if let Some(ref mut list) = char_save_list {
                        // Load saves if not already loaded and not currently loading
                        if !list.loaded && !list.loading && list.error.is_none() {
                            list.loading = true;
                            let (tx, rx) = std::sync::mpsc::channel();
                            std::thread::spawn(move || {
                                let result = crate::runtime::RUNTIME.block_on(
                                    dnd_core::persist::list_character_saves("saves/characters"),
                                );
                                let _ = tx.send(result);
                            });

                            // Store receiver to check later
                            commands.insert_resource(PendingCharacterList {
                                receiver: std::sync::Mutex::new(rx),
                            });
                        }

                        if let Some(character) =
                            overlays::render_load_character(ctx, &mut app_state, list)
                        {
                            // Start game with loaded character
                            commands.insert_resource(ReadyToStart {
                                character,
                                campaign_name: "The Dragon's Lair".to_string(),
                            });
                            next_phase.set(GamePhase::Playing);
                        }
                    }
                }
                ActiveOverlay::LoadGame => {
                    if let Some(ref mut list) = game_save_list {
                        // Load saves if not already loaded and not currently loading
                        if !list.loaded && !list.loading && list.error.is_none() {
                            list.loading = true;
                            let (tx, rx) = std::sync::mpsc::channel();
                            std::thread::spawn(move || {
                                let result =
                                    crate::runtime::RUNTIME.block_on(list_game_saves("saves"));
                                let _ = tx.send(result);
                            });

                            commands.insert_resource(PendingGameList {
                                receiver: std::sync::Mutex::new(rx),
                            });
                        }

                        if let Some(path) = overlays::render_load_game(ctx, &mut app_state, list) {
                            // Start loading the game
                            app_state.set_status_persistent("Loading game...");
                            let (tx, rx) = std::sync::mpsc::channel();
                            std::thread::spawn(move || {
                                let result = crate::runtime::RUNTIME.block_on(async {
                                    dnd_core::GameSession::load(&path)
                                        .await
                                        .map_err(|e| e.to_string())
                                });
                                let _ = tx.send(result);
                            });
                            commands.insert_resource(PendingGameLoad {
                                receiver: std::sync::Mutex::new(rx),
                            });
                        }
                    }
                }
                _ => {}
            }

            // Render error popup if present
            if app_state.error_message.is_some() {
                render_error_popup(ctx, &mut app_state);
            }
        }
        GamePhase::CharacterCreation => {
            if let Some(ref mut creation) = char_creation {
                crate::character_creation::render_character_creation(
                    ctx,
                    creation.as_mut(),
                    &mut next_phase,
                    &mut app_state,
                    &mut commands,
                );
            }
        }
        GamePhase::Playing => {
            // Render panels in correct order for egui layout:
            // 1. Top/bottom panels and side panels first (they claim space)
            // 2. CentralPanel last (fills remaining space)
            panels::render_top_bar(ctx, &mut app_state);
            panels::render_character_panel(ctx, &mut app_state);
            input::render_input_panel(ctx, &mut app_state);
            // CentralPanel must come after side/top/bottom panels
            panels::render_narrative_panel(ctx, &app_state, time.elapsed_secs_f64());
            // Windows can be rendered anytime (they float)
            panels::render_combat_panel(ctx, &app_state);

            // Render overlays if active
            match app_state.overlay {
                ActiveOverlay::None => {}
                ActiveOverlay::Inventory => overlays::render_inventory(ctx, &app_state),
                ActiveOverlay::CharacterSheet => {
                    overlays::render_character_sheet(ctx, &mut app_state)
                }
                ActiveOverlay::QuestLog => overlays::render_quest_log(ctx, &app_state),
                ActiveOverlay::Help => overlays::render_help(ctx),
                ActiveOverlay::Settings => {
                    if overlays::render_settings(ctx, &mut app_state) {
                        // User clicked "Return to Main Menu"
                        next_phase.set(GamePhase::MainMenu);
                    }
                }
                ActiveOverlay::LoadCharacter | ActiveOverlay::LoadGame => {
                    // These overlays are only used in MainMenu, close them if we're playing
                    app_state.overlay = ActiveOverlay::None;
                }
            }

            // Render spell detail popup if viewing a spell
            if app_state.viewing_spell.is_some() {
                overlays::render_spell_detail(ctx, &mut app_state);
            }

            // Render error popup if present
            if app_state.error_message.is_some() {
                render_error_popup(ctx, &mut app_state);
            }
        }
        GamePhase::GameOver => {
            panels::render_game_over(ctx, &app_state, &mut next_phase);
        }
    }
}

/// Configure egui visual style.
fn configure_style(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    // Increase default font sizes
    use egui::{FontId, TextStyle};
    style.text_styles = [
        (TextStyle::Small, FontId::proportional(14.0)),
        (TextStyle::Body, FontId::proportional(16.0)),
        (TextStyle::Monospace, FontId::monospace(15.0)),
        (TextStyle::Button, FontId::proportional(16.0)),
        (TextStyle::Heading, FontId::proportional(22.0)),
    ]
    .into();

    // Dark theme with D&D colors
    let visuals = &mut style.visuals;
    visuals.dark_mode = true;
    visuals.override_text_color = Some(egui::Color32::from_rgb(230, 220, 200)); // Parchment
    visuals.window_fill = egui::Color32::from_rgb(30, 25, 20); // Dark brown
    visuals.panel_fill = egui::Color32::from_rgb(40, 35, 30);
    visuals.faint_bg_color = egui::Color32::from_rgb(50, 45, 40);
    visuals.extreme_bg_color = egui::Color32::from_rgb(20, 15, 10);

    // Accent colors
    visuals.selection.bg_fill = egui::Color32::from_rgb(139, 69, 19); // Saddle brown
    visuals.hyperlink_color = egui::Color32::from_rgb(218, 165, 32); // Goldenrod

    // Widget colors
    visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(45, 40, 35);
    visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(55, 50, 45);

    // Enhanced hover state with gold tint
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(90, 75, 50); // Brighter with gold tint
    visuals.widgets.hovered.bg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(218, 165, 32)); // Gold stroke
    visuals.widgets.hovered.expansion = 1.0; // Slight expansion on hover

    // Active/pressed state
    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(139, 69, 19); // Saddle brown
    visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(255, 215, 0)); // Bright gold stroke

    ctx.set_style(style);
}

/// Render error popup.
fn render_error_popup(ctx: &egui::Context, app_state: &mut AppState) {
    let mut open = true;

    egui::Window::new("Error")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .open(&mut open)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);
                if let Some(ref msg) = app_state.error_message {
                    ui.colored_label(egui::Color32::RED, msg);
                }
                ui.add_space(10.0);
                if ui.button("OK").clicked() {
                    app_state.error_message = None;
                }
            });
        });

    if !open {
        app_state.error_message = None;
    }
}

/// Handle keyboard input for navigation and shortcuts.
pub fn handle_keyboard_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut app_state: ResMut<AppState>,
    game_phase: Res<State<GamePhase>>,
    mut contexts: EguiContexts,
) {
    let ctx = contexts.ctx_mut();

    // Ctrl+Q / Cmd+Q to quit (works anywhere)
    let ctrl_pressed = keys.pressed(KeyCode::ControlLeft)
        || keys.pressed(KeyCode::ControlRight)
        || keys.pressed(KeyCode::SuperLeft)
        || keys.pressed(KeyCode::SuperRight);

    if ctrl_pressed && keys.just_pressed(KeyCode::KeyQ) {
        std::process::exit(0);
    }

    // Close overlays with Escape (works in any phase)
    if keys.just_pressed(KeyCode::Escape) && app_state.overlay != ActiveOverlay::None {
        app_state.overlay = ActiveOverlay::None;
        return;
    }

    // Only handle other shortcuts during gameplay
    if *game_phase.get() != GamePhase::Playing {
        return;
    }

    // Ctrl+S for quick save (works even while typing)
    if ctrl_pressed
        && keys.just_pressed(KeyCode::KeyS)
        && !app_state.is_saving
        && !app_state.is_processing
        && app_state.has_session()
    {
        if let Some(tx) = &app_state.request_tx {
            let path = dnd_core::persist::auto_save_path("saves", &app_state.world.campaign_name);
            let _ = tx.try_send(crate::state::WorkerRequest::Save(path));
            app_state.is_saving = true;
            app_state.set_status_persistent("Saving...");
        }
    }

    // Don't handle other shortcuts if egui wants keyboard input (user is typing)
    if ctx.wants_keyboard_input() {
        return;
    }

    // Toggle overlays with hotkeys (when no overlay is open)
    if app_state.overlay == ActiveOverlay::None {
        if keys.just_pressed(KeyCode::KeyI) {
            app_state.toggle_overlay(ActiveOverlay::Inventory);
        }
        if keys.just_pressed(KeyCode::KeyC) {
            app_state.toggle_overlay(ActiveOverlay::CharacterSheet);
        }
        if keys.just_pressed(KeyCode::KeyQ) && keys.pressed(KeyCode::ShiftLeft) {
            app_state.toggle_overlay(ActiveOverlay::QuestLog);
        }
        if keys.just_pressed(KeyCode::F1) || keys.just_pressed(KeyCode::Slash) {
            app_state.toggle_overlay(ActiveOverlay::Help);
        }
    }
}

/// List all game saves in the saves directory.
async fn list_game_saves(dir: &str) -> Result<Vec<GameSaveInfo>, String> {
    use tokio::fs;

    let mut saves = Vec::new();
    let dir_path = std::path::Path::new(dir);

    if !dir_path.exists() {
        return Ok(saves);
    }

    let mut entries = fs::read_dir(dir_path).await.map_err(|e| e.to_string())?;

    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();

        // Skip directories and non-json files
        if path.is_dir() || path.extension().map(|e| e != "json").unwrap_or(true) {
            continue;
        }

        // Skip character saves (they're in saves/characters/)
        if path
            .file_name()
            .map(|n| n.to_string_lossy().contains("character"))
            .unwrap_or(false)
        {
            continue;
        }

        // Try to peek at the save metadata
        if let Ok(metadata) = dnd_core::persist::SavedCampaign::peek_metadata(&path).await {
            saves.push(GameSaveInfo {
                path: path.to_string_lossy().to_string(),
                campaign_name: metadata.campaign_name,
                character_name: metadata.character_name,
                character_level: metadata.level,
                saved_at: metadata.saved_at,
            });
        }
    }

    // Sort by save time (most recent first)
    saves.sort_by(|a, b| b.saved_at.cmp(&a.saved_at));

    Ok(saves)
}
