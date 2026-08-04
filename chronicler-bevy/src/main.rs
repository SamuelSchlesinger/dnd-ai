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
mod sound;
mod state;
mod ui;
mod window;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use chronicler_core::{GameSession, LlmClient, SessionConfig, LOCAL_DEFAULT_MODEL};
use serde::Deserialize;

enum CliAction {
    RunFromEnvironment,
    RunLocal(String),
    Help,
}

fn parse_cli_args<I>(args: I) -> Result<CliAction, String>
where
    I: IntoIterator<Item = String>,
{
    let mut args = args.into_iter().peekable();
    let Some(first) = args.next() else {
        return Ok(CliAction::RunFromEnvironment);
    };
    let action = match first.as_str() {
        "-h" | "--help" => CliAction::Help,
        "--local" => {
            let model = args
                .next_if(|next| !next.starts_with('-'))
                .unwrap_or_else(|| LOCAL_DEFAULT_MODEL.to_string());
            CliAction::RunLocal(model)
        }
        value if value.starts_with("--local=") => {
            let model = value.trim_start_matches("--local=");
            if model.is_empty() {
                return Err("--local= requires a model name".to_string());
            }
            CliAction::RunLocal(model.to_string())
        }
        other => return Err(format!("unknown argument '{other}' (try --help)")),
    };
    if let Some(extra) = args.next() {
        return Err(format!("unexpected argument '{extra}'"));
    }
    Ok(action)
}

fn print_help() {
    println!(
        "Chronicler\n\nUSAGE:\n    chronicler [--local [MODEL]]\n\nOPTIONS:\n    --local [MODEL]  Use Ollama at http://localhost:11434/v1\n                     [default model: {LOCAL_DEFAULT_MODEL}]\n    -h, --help       Print this help\n\nWithout --local, provider settings are read from the environment."
    );
}

/// Application configuration loaded from config.toml
#[derive(Deserialize)]
struct ConfigFile {
    paths: PathsConfig,
}

#[derive(Deserialize, Clone)]
struct PathsConfig {
    asset_path: String,
    saves_path: String,
}

/// Runtime configuration resource available throughout the app
#[derive(Resource, Clone)]
pub struct AppConfig {
    pub saves_path: String,
    llm_client: Option<LlmClient>,
    llm_error: Option<String>,
}

impl AppConfig {
    pub fn characters_path(&self) -> String {
        format!("{}/characters", self.saves_path)
    }

    pub fn llm_client(&self) -> Result<LlmClient, String> {
        self.llm_client.clone().ok_or_else(|| {
            self.llm_error
                .clone()
                .unwrap_or_else(|| "LLM is not configured".to_string())
        })
    }
}

use crate::character_creation::{CharacterCreation, ReadyToStart};
use crate::state::{
    AppState, CharacterSaveList, GamePhase, GameSaveList, OnboardingState, PendingSession,
};
fn main() {
    // Load .env file if present
    dotenvy::dotenv().ok();

    let cli = match parse_cli_args(std::env::args().skip(1)) {
        Ok(CliAction::Help) => {
            print_help();
            return;
        }
        Ok(action) => action,
        Err(error) => {
            eprintln!("Error: {error}");
            print_help();
            std::process::exit(2);
        }
    };
    let llm_result = match cli {
        CliAction::RunFromEnvironment => LlmClient::from_env(),
        CliAction::RunLocal(model) => LlmClient::local(model),
        CliAction::Help => unreachable!(),
    };
    let (llm_client, llm_error) = match llm_result {
        Ok(client) => {
            eprintln!(
                "LLM: {} model {} at {}",
                client.provider(),
                client.model(),
                client.base_url()
            );
            (Some(client), None)
        }
        Err(error) => (None, Some(error.to_string())),
    };

    // Load configuration from config.toml
    let config: ConfigFile = std::fs::read_to_string("config.toml")
        .map_err(|e| format!("Failed to read config.toml: {e}"))
        .and_then(|s| toml::from_str(&s).map_err(|e| format!("Failed to parse config.toml: {e}")))
        .expect("config.toml must exist and be valid. Run from workspace root.");

    let asset_path = config.paths.asset_path;
    let saves_path = config.paths.saves_path.clone();

    // Create saves directories if they don't exist
    std::fs::create_dir_all(&saves_path).ok();
    std::fs::create_dir_all(format!("{}/characters", saves_path)).ok();

    let app_config = AppConfig {
        saves_path: saves_path.clone(),
        llm_client,
        llm_error,
    };

    // Load settings from disk
    let window_settings = window::load_settings(&saves_path);
    let sound_settings = sound::load_settings(&saves_path);
    let onboarding_state = OnboardingState::load(&saves_path);

    // Always use windowed mode (fullscreen disabled due to macOS issues)
    let initial_window_mode = bevy::window::WindowMode::Windowed;

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "D&D: AI Dungeon Master".into(),
                        resolution: (window_settings.width, window_settings.height).into(),
                        resizable: true,
                        mode: initial_window_mode,
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    file_path: asset_path,
                    ..default()
                }),
        )
        .add_plugins(EguiPlugin)
        .add_plugins(sound::SoundPlugin)
        .add_plugins(window::WindowSettingsPlugin)
        .insert_resource(app_config)
        .insert_resource(window_settings)
        .insert_resource(sound_settings)
        .insert_resource(onboarding_state)
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
                animations::animate_screen_shake,
                animations::cleanup_finished_animations,
            ),
        )
        // Update systems - AI worker and session management
        .add_systems(
            Update,
            (
                state::handle_worker_responses,
                state::process_pending_sounds,
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
    app_config: Res<AppConfig>,
) {
    let Some(ready) = ready else { return };

    // Create a channel to receive the session
    let (tx, rx) = std::sync::mpsc::channel();

    let character = ready.character.clone();
    let campaign_name = ready.campaign_name.clone();
    let llm_client = app_config.llm_client();

    // Spawn async session creation
    std::thread::spawn(move || {
        let result = crate::runtime::RUNTIME.block_on(async {
            let client = llm_client?;
            let config = SessionConfig::new(&campaign_name).with_character_name(&character.name);

            GameSession::new_with_character_and_client(config, character, client)
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

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    #[test]
    fn local_defaults_to_generalist_model() {
        match parse_cli_args(args(&["--local"])).expect("args") {
            CliAction::RunLocal(model) => assert_eq!(model, "qwen3.6:35b-a3b"),
            _ => panic!("expected local mode"),
        }
    }

    #[test]
    fn local_accepts_model_argument() {
        match parse_cli_args(args(&["--local", "qwen3:8b"])).expect("args") {
            CliAction::RunLocal(model) => assert_eq!(model, "qwen3:8b"),
            _ => panic!("expected local mode"),
        }
    }

    #[test]
    fn rejects_unknown_arguments() {
        assert!(parse_cli_args(args(&["--remote"])).is_err());
    }
}
