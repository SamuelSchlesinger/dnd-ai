//! D&D Bevy GUI - A visual interface for D&D with AI Dungeon Master.
//!
//! This application provides a polished, cross-platform GUI built with Bevy
//! and egui. It features:
//! - Text-based narrative gameplay
//! - Dice rolling animations
//! - Floating damage numbers
//! - Combat effects
//! - Character creation wizard

mod animations;
mod character_creation;
mod effects;
mod runtime;
mod state;
mod ui;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;

use crate::character_creation::{CharacterCreation, ReadyToStart};
use crate::state::{AppState, CharacterSaveList, GamePhase, GameSaveList, PendingSession};
use dnd_core::{GameSession, SessionConfig};

fn main() {
    // Load .env file if present
    dotenvy::dotenv().ok();

    // Create saves directories if they don't exist
    std::fs::create_dir_all("saves").ok();
    std::fs::create_dir_all("saves/characters").ok();

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "D&D: AI Dungeon Master".into(),
                resolution: (1280., 800.).into(),
                resizable: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin)
        // App state
        .init_state::<GamePhase>()
        .init_resource::<AppState>()
        .init_resource::<CharacterSaveList>()
        .init_resource::<GameSaveList>()
        // Startup systems
        .add_systems(Startup, setup)
        // State transition systems
        .add_systems(
            OnEnter(GamePhase::CharacterCreation),
            setup_character_creation,
        )
        .add_systems(
            OnExit(GamePhase::CharacterCreation),
            cleanup_character_creation,
        )
        // Update systems - UI
        .add_systems(Update, (ui::main_ui_system, ui::handle_keyboard_input))
        // Update systems - animations
        .add_systems(
            Update,
            (
                animations::animate_dice,
                animations::animate_damage_numbers,
                animations::animate_combat_effects,
                animations::cleanup_finished_animations,
            ),
        )
        // Update systems - AI worker and session management
        .add_systems(
            Update,
            (
                state::handle_worker_responses,
                state::check_pending_session,
                state::check_pending_character_list,
                state::check_pending_game_list,
                state::check_pending_game_load,
                state::clear_old_status,
                handle_ready_to_start,
            ),
        )
        .run();
}

/// Initial setup system.
fn setup(mut commands: Commands) {
    // Spawn 2D camera for animations
    commands.spawn(Camera2d);
}

/// Setup character creation when entering that state.
fn setup_character_creation(mut commands: Commands) {
    commands.insert_resource(CharacterCreation::new());
}

/// Cleanup character creation when exiting that state.
fn cleanup_character_creation(mut commands: Commands) {
    commands.remove_resource::<CharacterCreation>();
}

/// Handle ReadyToStart - spawn async session creation.
fn handle_ready_to_start(
    mut commands: Commands,
    ready: Option<Res<ReadyToStart>>,
    mut app_state: ResMut<AppState>,
) {
    let Some(ready) = ready else { return };

    // Create a channel to receive the session
    let (tx, rx) = std::sync::mpsc::channel();

    let character = ready.character.clone();
    let campaign_name = ready.campaign_name.clone();

    // Spawn async session creation
    std::thread::spawn(move || {
        let result = crate::runtime::RUNTIME.block_on(async {
            let config = SessionConfig::new(&campaign_name).with_character_name(&character.name);

            GameSession::new_with_character(config, character)
                .await
                .map_err(|e| e.to_string())
        });
        let _ = tx.send(result);
    });

    // Store the pending session receiver
    commands.insert_resource(PendingSession {
        receiver: std::sync::Mutex::new(rx),
    });

    // Remove ReadyToStart
    commands.remove_resource::<ReadyToStart>();

    // Show loading status
    app_state.set_status_persistent("Creating adventure...");
}
