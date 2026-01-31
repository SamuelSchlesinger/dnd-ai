//! Application state and AI worker integration.
//!
//! This module provides the GameState resource that holds all mutable
//! application state, and integrates with the async AI worker for
//! processing player actions.

use bevy::prelude::*;
use dnd_core::rules::Effect;
use dnd_core::world::{
    AbilityScores, CombatState, Condition, DeathSaves, GameMode, GameTime, HitPoints, Item,
    NarrativeType, Quest, Skill,
};
use dnd_core::GameSession;
use std::collections::HashMap;
use tokio::sync::mpsc;

/// Game phase state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, States)]
#[allow(dead_code)]
pub enum GamePhase {
    /// Main menu / title screen
    #[default]
    MainMenu,
    /// Character creation wizard
    CharacterCreation,
    /// Active gameplay
    Playing,
    /// Game over screen
    GameOver,
}

/// Request sent from the UI to the AI worker.
#[derive(Debug)]
#[allow(dead_code)]
pub enum WorkerRequest {
    /// Process a player action.
    PlayerAction(String),
    /// Cancel the current processing.
    Cancel,
    /// Save the game to a file.
    Save(std::path::PathBuf),
    /// Load a game from a file.
    Load(std::path::PathBuf),
    /// Shutdown the worker.
    Shutdown,
}

/// Response sent from the AI worker to the UI.
#[derive(Debug)]
#[allow(dead_code)]
pub enum WorkerResponse {
    /// A chunk of streaming text as it arrives.
    StreamChunk(String),
    /// A game effect to process.
    Effect(Effect),
    /// Processing completed successfully.
    Complete {
        /// The full narrative response.
        narrative: String,
        /// All effects that were applied.
        effects: Vec<Effect>,
        /// Updated world state for rendering.
        world_update: WorldUpdate,
        /// Whether combat is currently active.
        in_combat: bool,
        /// Whether it's the player's turn.
        is_player_turn: bool,
    },
    /// Processing was cancelled.
    Cancelled,
    /// An error occurred.
    Error(String),
    /// Save operation completed.
    SaveComplete(Result<std::path::PathBuf, String>),
    /// Load operation completed with new world state.
    LoadComplete(Result<WorldUpdate, String>),
}

/// World state snapshot for UI rendering.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct WorldUpdate {
    /// Player hit points.
    pub player_hp: HitPoints,
    /// Current combat state if any.
    pub combat: Option<CombatState>,
    /// Current game mode.
    pub mode: GameMode,
    /// Current game time.
    pub game_time: GameTime,
    /// Player name.
    pub player_name: String,
    /// Player class.
    pub player_class: Option<String>,
    /// Player level.
    pub player_level: u8,
    /// Player AC.
    pub player_ac: u8,
    /// Player initiative modifier.
    pub player_initiative: i8,
    /// Player speed.
    pub player_speed: u32,
    /// Current location name.
    pub current_location: String,
    /// Campaign name.
    pub campaign_name: String,
    /// Active conditions affecting the player.
    pub conditions: Vec<Condition>,
    /// Death save progress (when at 0 HP).
    pub death_saves: DeathSaves,
    /// Player's gold.
    pub gold: f32,
    /// Equipped weapon name (if any).
    pub equipped_weapon: Option<String>,
    /// Equipped armor name (if any).
    pub equipped_armor: Option<String>,
    /// Inventory items.
    pub inventory_items: Vec<Item>,
    /// Ability scores.
    pub ability_scores: AbilityScores,
    /// Skill proficiencies (skill -> proficiency level string).
    pub skill_proficiencies: HashMap<Skill, String>,
    /// Proficiency bonus.
    pub proficiency_bonus: i8,
    /// Active and completed quests.
    pub quests: Vec<Quest>,
    /// Spell slots (level 1-9): (available, total) for each level
    pub spell_slots: Vec<(u8, u8)>,
    /// Known/prepared spells
    pub known_spells: Vec<String>,
    /// Known cantrips
    pub cantrips: Vec<String>,
    /// Spellcasting ability (if any)
    pub spellcasting_ability: Option<String>,
    /// Spell save DC (if spellcaster)
    pub spell_save_dc: Option<u8>,
    /// Spell attack bonus (if spellcaster)
    pub spell_attack_bonus: Option<i8>,
}

impl Default for WorldUpdate {
    fn default() -> Self {
        Self {
            player_hp: HitPoints::new(10),
            combat: None,
            mode: GameMode::Exploration,
            game_time: GameTime::default(),
            player_name: "???".to_string(),
            player_class: None,
            player_level: 1,
            player_ac: 10,
            player_initiative: 0,
            player_speed: 30,
            current_location: "Unknown".to_string(),
            campaign_name: "New Campaign".to_string(),
            conditions: Vec::new(),
            death_saves: DeathSaves::default(),
            gold: 0.0,
            equipped_weapon: None,
            equipped_armor: None,
            inventory_items: Vec::new(),
            ability_scores: AbilityScores::default(),
            skill_proficiencies: HashMap::new(),
            proficiency_bonus: 2,
            quests: Vec::new(),
            spell_slots: Vec::new(),
            known_spells: Vec::new(),
            cantrips: Vec::new(),
            spellcasting_ability: None,
            spell_save_dc: None,
            spell_attack_bonus: None,
        }
    }
}

impl WorldUpdate {
    /// Create a WorldUpdate snapshot from a GameSession.
    pub fn from_session(session: &GameSession) -> Self {
        let world = session.world();
        let character = &world.player_character;
        Self {
            player_hp: character.hit_points.clone(),
            combat: world.combat.clone(),
            mode: world.mode,
            game_time: world.game_time.clone(),
            player_name: character.name.clone(),
            player_class: character
                .classes
                .first()
                .map(|c| c.class.name().to_string()),
            player_level: character.level,
            player_ac: character.current_ac(),
            player_initiative: character.initiative_modifier(),
            player_speed: character.speed.walk,
            current_location: world.current_location.name.clone(),
            campaign_name: world.campaign_name.clone(),
            conditions: character.conditions.iter().map(|c| c.condition).collect(),
            death_saves: character.death_saves.clone(),
            gold: character.inventory.gold,
            equipped_weapon: character
                .equipment
                .main_hand
                .as_ref()
                .map(|w| w.base.name.clone()),
            equipped_armor: character
                .equipment
                .armor
                .as_ref()
                .map(|a| a.base.name.clone()),
            inventory_items: character.inventory.items.clone(),
            ability_scores: character.ability_scores.clone(),
            skill_proficiencies: character
                .skill_proficiencies
                .iter()
                .map(|(skill, level)| (*skill, format!("{level:?}")))
                .collect(),
            proficiency_bonus: character.proficiency_bonus(),
            quests: world.quests.clone(),
            spell_slots: character
                .spellcasting
                .as_ref()
                .map(|sc| {
                    sc.spell_slots
                        .slots
                        .iter()
                        .map(|slot| (slot.available(), slot.total))
                        .collect()
                })
                .unwrap_or_default(),
            known_spells: character
                .spellcasting
                .as_ref()
                .map(|sc| sc.spells_known.clone())
                .unwrap_or_default(),
            cantrips: character
                .spellcasting
                .as_ref()
                .map(|sc| sc.cantrips_known.clone())
                .unwrap_or_default(),
            spellcasting_ability: character
                .spellcasting
                .as_ref()
                .map(|sc| sc.ability.name().to_string()),
            spell_save_dc: character.spellcasting.as_ref().map(|sc| {
                let mod_ = character.ability_scores.modifier(sc.ability);
                (8 + mod_ + character.proficiency_bonus()) as u8
            }),
            spell_attack_bonus: character.spellcasting.as_ref().map(|sc| {
                let mod_ = character.ability_scores.modifier(sc.ability);
                mod_ + character.proficiency_bonus()
            }),
        }
    }
}

/// A narrative entry with styling.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct NarrativeEntry {
    pub text: String,
    pub entry_type: NarrativeType,
    pub timestamp: f64,
}

/// Active overlay screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ActiveOverlay {
    #[default]
    None,
    Inventory,
    CharacterSheet,
    QuestLog,
    Help,
    Settings,
    LoadCharacter,
    LoadGame,
}

/// Pending session creation - holds the receiver for async session creation.
#[derive(Resource)]
pub struct PendingSession {
    pub receiver: std::sync::Mutex<std::sync::mpsc::Receiver<Result<GameSession, String>>>,
}

/// List of saved characters for the load character overlay.
#[derive(Resource, Default)]
pub struct CharacterSaveList {
    /// The list of character saves.
    pub saves: Vec<dnd_core::CharacterSaveInfo>,
    /// Whether the list is being loaded.
    pub loading: bool,
    /// Whether the list has been loaded at least once.
    pub loaded: bool,
    /// Selected character index.
    pub selected: Option<usize>,
    /// Error message if loading failed.
    pub error: Option<String>,
}

/// Pending character list load - holds the receiver for async character list loading.
#[derive(Resource)]
pub struct PendingCharacterList {
    pub receiver: std::sync::Mutex<
        std::sync::mpsc::Receiver<
            Result<Vec<dnd_core::CharacterSaveInfo>, dnd_core::persist::PersistError>,
        >,
    >,
}

/// Information about a game save file.
#[derive(Debug, Clone)]
pub struct GameSaveInfo {
    pub path: String,
    pub campaign_name: String,
    pub character_name: String,
    pub character_level: u8,
    pub saved_at: String,
}

/// List of saved games for the load game overlay.
#[derive(Resource, Default)]
pub struct GameSaveList {
    pub saves: Vec<GameSaveInfo>,
    pub loading: bool,
    pub loaded: bool,
    pub selected: Option<usize>,
    pub error: Option<String>,
}

/// Pending game list load.
#[derive(Resource)]
pub struct PendingGameList {
    pub receiver: std::sync::Mutex<std::sync::mpsc::Receiver<Result<Vec<GameSaveInfo>, String>>>,
}

/// Pending game session load from a save file.
#[derive(Resource)]
pub struct PendingGameLoad {
    pub receiver: std::sync::Mutex<std::sync::mpsc::Receiver<Result<GameSession, String>>>,
}

/// Main application state resource.
#[derive(Resource)]
#[allow(dead_code)]
pub struct AppState {
    /// Current world state snapshot.
    pub world: WorldUpdate,
    /// Narrative history.
    pub narrative: Vec<NarrativeEntry>,
    /// Current streaming text (not yet complete).
    pub streaming_text: String,
    /// Player input text.
    pub input_text: String,
    /// Whether we're waiting for AI response.
    pub is_processing: bool,
    /// Status bar message.
    pub status_message: Option<String>,
    /// Current overlay.
    pub overlay: ActiveOverlay,
    /// Request channel sender.
    pub request_tx: Option<mpsc::Sender<WorkerRequest>>,
    /// Response channel receiver.
    pub response_rx: Option<mpsc::Receiver<WorkerResponse>>,
    /// Whether in combat.
    pub in_combat: bool,
    /// Whether it's the player's turn.
    pub is_player_turn: bool,
    /// Error message to display.
    pub error_message: Option<String>,
    /// Time since last effect (for animation timing).
    pub last_effect_time: f64,
    /// Whether the character panel is expanded.
    pub character_panel_expanded: bool,
    /// Whether a save operation is in progress.
    pub is_saving: bool,
    /// Whether a load operation is in progress.
    pub is_loading: bool,
    /// When the status message was set (for auto-clear).
    pub status_set_time: Option<f64>,
    /// History of player commands for up/down navigation.
    pub input_history: Vec<String>,
    /// Current position in input history (-1 means not browsing history).
    pub history_index: i32,
    /// Saved input text when browsing history.
    pub saved_input: String,
    /// Spell currently being viewed in detail (None if not viewing any).
    pub viewing_spell: Option<String>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            world: WorldUpdate::default(),
            narrative: Vec::new(),
            streaming_text: String::new(),
            input_text: String::new(),
            is_processing: false,
            status_message: None,
            overlay: ActiveOverlay::None,
            request_tx: None,
            response_rx: None,
            in_combat: false,
            is_player_turn: false,
            error_message: None,
            last_effect_time: 0.0,
            character_panel_expanded: true,
            is_saving: false,
            is_loading: false,
            status_set_time: None,
            input_history: Vec::new(),
            history_index: -1,
            saved_input: String::new(),
            viewing_spell: None,
        }
    }
}

impl AppState {
    /// Add a narrative entry.
    pub fn add_narrative(&mut self, text: String, entry_type: NarrativeType, time: f64) {
        self.narrative.push(NarrativeEntry {
            text,
            entry_type,
            timestamp: time,
        });
        // Keep narrative history bounded
        if self.narrative.len() > 500 {
            self.narrative.remove(0);
        }
    }

    /// Set a status message (with timestamp for auto-clear).
    pub fn set_status(&mut self, message: impl Into<String>, current_time: f64) {
        self.status_message = Some(message.into());
        self.status_set_time = Some(current_time);
    }

    /// Set a status message without timestamp (won't auto-clear).
    pub fn set_status_persistent(&mut self, message: impl Into<String>) {
        self.status_message = Some(message.into());
        self.status_set_time = None;
    }

    /// Clear status message.
    pub fn clear_status(&mut self) {
        self.status_message = None;
        self.status_set_time = None;
    }

    /// Toggle an overlay.
    pub fn toggle_overlay(&mut self, overlay: ActiveOverlay) {
        if self.overlay == overlay {
            self.overlay = ActiveOverlay::None;
        } else {
            self.overlay = overlay;
        }
    }

    /// Send a player action to the AI worker.
    pub fn send_action(&mut self, action: String) {
        if let Some(tx) = &self.request_tx {
            if !action.trim().is_empty() && !self.is_processing {
                let _ = tx.try_send(WorkerRequest::PlayerAction(action));
                self.is_processing = true;
                self.streaming_text.clear();
            }
        }
    }

    /// Check if the game session is active.
    pub fn has_session(&self) -> bool {
        self.request_tx.is_some()
    }

    /// Add a command to input history.
    pub fn add_to_history(&mut self, command: String) {
        // Don't add empty or duplicate consecutive commands
        if !command.trim().is_empty() && self.input_history.last() != Some(&command) {
            self.input_history.push(command);
        }
        // Keep history bounded
        if self.input_history.len() > 100 {
            self.input_history.remove(0);
        }
        // Reset history navigation
        self.history_index = -1;
        self.saved_input.clear();
    }

    /// Navigate up in history (older commands).
    pub fn history_up(&mut self) {
        if self.input_history.is_empty() {
            return;
        }

        if self.history_index == -1 {
            // Starting to browse history, save current input
            self.saved_input = self.input_text.clone();
            self.history_index = self.input_history.len() as i32 - 1;
        } else if self.history_index > 0 {
            self.history_index -= 1;
        }

        if let Some(cmd) = self.input_history.get(self.history_index as usize) {
            self.input_text = cmd.clone();
        }
    }

    /// Navigate down in history (newer commands).
    pub fn history_down(&mut self) {
        if self.history_index == -1 {
            return; // Not browsing history
        }

        self.history_index += 1;

        if self.history_index >= self.input_history.len() as i32 {
            // Back to current input
            self.history_index = -1;
            self.input_text = std::mem::take(&mut self.saved_input);
        } else if let Some(cmd) = self.input_history.get(self.history_index as usize) {
            self.input_text = cmd.clone();
        }
    }
}

/// System to clear old status messages after 3 seconds.
pub fn clear_old_status(mut app_state: ResMut<AppState>, time: Res<Time>) {
    if let Some(set_time) = app_state.status_set_time {
        let elapsed = time.elapsed_secs_f64() - set_time;
        if elapsed > 3.0 {
            app_state.clear_status();
        }
    }
}

/// System to handle responses from the AI worker.
pub fn handle_worker_responses(
    mut app_state: ResMut<AppState>,
    time: Res<Time>,
    mut commands: Commands,
) {
    // Take the receiver temporarily to check for messages
    let response = if let Some(rx) = &mut app_state.response_rx {
        rx.try_recv().ok()
    } else {
        None
    };

    if let Some(response) = response {
        match response {
            WorkerResponse::StreamChunk(text) => {
                app_state.streaming_text.push_str(&text);
            }
            WorkerResponse::Effect(effect) => {
                crate::effects::process_effect(
                    &mut app_state,
                    &effect,
                    &mut commands,
                    time.elapsed_secs_f64(),
                );
            }
            WorkerResponse::Complete {
                narrative,
                effects: _,
                world_update,
                in_combat,
                is_player_turn,
            } => {
                // Add the complete narrative
                if !narrative.is_empty() {
                    app_state.add_narrative(
                        narrative,
                        NarrativeType::DmNarration,
                        time.elapsed_secs_f64(),
                    );
                }
                app_state.streaming_text.clear();
                app_state.world = world_update;
                app_state.in_combat = in_combat;
                app_state.is_player_turn = is_player_turn;
                app_state.is_processing = false;
            }
            WorkerResponse::Cancelled => {
                app_state.is_processing = false;
                app_state.streaming_text.clear();
            }
            WorkerResponse::Error(err) => {
                app_state.error_message = Some(err);
                app_state.is_processing = false;
            }
            WorkerResponse::SaveComplete(result) => {
                app_state.is_saving = false;
                match result {
                    Ok(path) => {
                        app_state.set_status(format!("Saved to {path:?}"), time.elapsed_secs_f64());
                    }
                    Err(e) => {
                        app_state.error_message = Some(format!("Save failed: {e}"));
                    }
                }
            }
            WorkerResponse::LoadComplete(result) => {
                app_state.is_loading = false;
                match result {
                    Ok(world_update) => {
                        app_state.world = world_update;
                        app_state.set_status("Game loaded", time.elapsed_secs_f64());
                    }
                    Err(e) => {
                        app_state.error_message = Some(format!("Load failed: {e}"));
                    }
                }
            }
        }
    }
}

/// System to check for pending game list load.
pub fn check_pending_game_list(
    mut commands: Commands,
    pending: Option<Res<PendingGameList>>,
    mut save_list: Option<ResMut<GameSaveList>>,
) {
    let Some(pending) = pending else { return };
    let Some(ref mut list) = save_list else {
        return;
    };

    let result = {
        let receiver = pending.receiver.lock().unwrap();
        receiver.try_recv()
    };

    match result {
        Ok(Ok(saves)) => {
            list.saves = saves;
            list.loading = false;
            list.loaded = true;
            commands.remove_resource::<PendingGameList>();
        }
        Ok(Err(e)) => {
            list.error = Some(e);
            list.loading = false;
            list.loaded = true;
            commands.remove_resource::<PendingGameList>();
        }
        Err(std::sync::mpsc::TryRecvError::Empty) => {}
        Err(std::sync::mpsc::TryRecvError::Disconnected) => {
            list.error = Some("Game list load failed unexpectedly".to_string());
            list.loading = false;
            list.loaded = true;
            commands.remove_resource::<PendingGameList>();
        }
    }
}

/// System to check for pending game load and start the session.
pub fn check_pending_game_load(
    mut commands: Commands,
    pending: Option<Res<PendingGameLoad>>,
    mut app_state: ResMut<AppState>,
    mut next_phase: ResMut<NextState<GamePhase>>,
) {
    let Some(pending) = pending else { return };

    let result = {
        let receiver = pending.receiver.lock().unwrap();
        receiver.try_recv()
    };

    match result {
        Ok(Ok(session)) => {
            // Session loaded successfully - spawn the worker
            let (request_tx, response_rx, initial_world) = spawn_worker(session);
            app_state.request_tx = Some(request_tx);
            app_state.response_rx = Some(response_rx);
            app_state.world = initial_world;
            app_state.set_status_persistent("Game loaded!");
            app_state.overlay = ActiveOverlay::None;

            // Transition to playing
            next_phase.set(GamePhase::Playing);
            commands.remove_resource::<PendingGameLoad>();
        }
        Ok(Err(e)) => {
            app_state.error_message = Some(format!("Failed to load game: {e}"));
            commands.remove_resource::<PendingGameLoad>();
        }
        Err(std::sync::mpsc::TryRecvError::Empty) => {}
        Err(std::sync::mpsc::TryRecvError::Disconnected) => {
            app_state.error_message = Some("Game load failed unexpectedly".to_string());
            commands.remove_resource::<PendingGameLoad>();
        }
    }
}

/// System to check for pending character list load.
pub fn check_pending_character_list(
    mut commands: Commands,
    pending: Option<Res<PendingCharacterList>>,
    mut save_list: Option<ResMut<CharacterSaveList>>,
) {
    let Some(pending) = pending else { return };
    let Some(ref mut list) = save_list else {
        return;
    };

    let result = {
        let receiver = pending.receiver.lock().unwrap();
        receiver.try_recv()
    };

    match result {
        Ok(Ok(saves)) => {
            list.saves = saves;
            list.loading = false;
            list.loaded = true;
            commands.remove_resource::<PendingCharacterList>();
        }
        Ok(Err(e)) => {
            list.error = Some(format!("Failed to load saves: {e}"));
            list.loading = false;
            list.loaded = true;
            commands.remove_resource::<PendingCharacterList>();
        }
        Err(std::sync::mpsc::TryRecvError::Empty) => {
            // Still loading
        }
        Err(std::sync::mpsc::TryRecvError::Disconnected) => {
            list.error = Some("Character list load failed unexpectedly".to_string());
            list.loading = false;
            list.loaded = true;
            commands.remove_resource::<PendingCharacterList>();
        }
    }
}

/// System to check for pending session creation and connect it when ready.
pub fn check_pending_session(
    mut commands: Commands,
    pending: Option<Res<PendingSession>>,
    mut app_state: ResMut<AppState>,
) {
    let Some(pending) = pending else { return };

    // Try to receive without blocking
    let result = {
        let receiver = pending.receiver.lock().unwrap();
        receiver.try_recv()
    };

    match result {
        Ok(Ok(session)) => {
            // Session created successfully - spawn the worker
            let (request_tx, response_rx, initial_world) = spawn_worker(session);
            app_state.request_tx = Some(request_tx);
            app_state.response_rx = Some(response_rx);
            app_state.world = initial_world;
            app_state.set_status_persistent("Adventure begins!");

            // Send initial action to get the DM's opening narration
            app_state.send_action(
                "I begin my adventure. Set the scene and describe where I am.".to_string(),
            );

            // Remove the pending resource
            commands.remove_resource::<PendingSession>();
        }
        Ok(Err(e)) => {
            app_state.error_message = Some(format!("Failed to create session: {e}"));
            commands.remove_resource::<PendingSession>();
        }
        Err(std::sync::mpsc::TryRecvError::Empty) => {
            // Still waiting
        }
        Err(std::sync::mpsc::TryRecvError::Disconnected) => {
            app_state.error_message = Some("Session creation failed unexpectedly".to_string());
            commands.remove_resource::<PendingSession>();
        }
    }
}

/// Spawn the AI worker and return channel endpoints.
pub fn spawn_worker(
    session: GameSession,
) -> (
    mpsc::Sender<WorkerRequest>,
    mpsc::Receiver<WorkerResponse>,
    WorldUpdate,
) {
    let (request_tx, request_rx) = mpsc::channel(8);
    let (response_tx, response_rx) = mpsc::channel(64);

    // Get initial world state before spawning
    let initial_world = WorldUpdate::from_session(&session);

    // Spawn the worker task
    std::thread::spawn(move || {
        crate::runtime::RUNTIME.block_on(worker_loop(session, request_rx, response_tx));
    });

    (request_tx, response_rx, initial_world)
}

/// The main worker loop that processes requests.
async fn worker_loop(
    mut session: GameSession,
    mut request_rx: mpsc::Receiver<WorkerRequest>,
    response_tx: mpsc::Sender<WorkerResponse>,
) {
    loop {
        match request_rx.recv().await {
            Some(WorkerRequest::PlayerAction(input)) => {
                process_player_action(&mut session, &input, &response_tx).await;
            }
            Some(WorkerRequest::Cancel) => {
                let _ = response_tx.send(WorkerResponse::Cancelled).await;
            }
            Some(WorkerRequest::Save(path)) => {
                let result = session.save(&path).await;
                let response = match result {
                    Ok(()) => WorkerResponse::SaveComplete(Ok(path)),
                    Err(e) => WorkerResponse::SaveComplete(Err(e.to_string())),
                };
                let _ = response_tx.send(response).await;
            }
            Some(WorkerRequest::Load(path)) => match GameSession::load(&path).await {
                Ok(new_session) => {
                    session = new_session;
                    let world_update = WorldUpdate::from_session(&session);
                    let _ = response_tx
                        .send(WorkerResponse::LoadComplete(Ok(world_update)))
                        .await;
                }
                Err(e) => {
                    let _ = response_tx
                        .send(WorkerResponse::LoadComplete(Err(e.to_string())))
                        .await;
                }
            },
            Some(WorkerRequest::Shutdown) | None => {
                break;
            }
        }
    }
}

/// Process a player action and send responses with streaming.
async fn process_player_action(
    session: &mut GameSession,
    input: &str,
    response_tx: &mpsc::Sender<WorkerResponse>,
) {
    let input = input.trim();
    if input.is_empty() {
        return;
    }

    let stream_tx = response_tx.clone();

    let result = session
        .player_action_streaming(input, |text| {
            let _ = stream_tx.try_send(WorkerResponse::StreamChunk(text.to_string()));
        })
        .await;

    match result {
        Ok(response) => {
            // Send individual effects for immediate UI updates
            for effect in &response.effects {
                let _ = response_tx
                    .send(WorkerResponse::Effect(effect.clone()))
                    .await;
            }

            // Build world update
            let world_update = WorldUpdate::from_session(session);

            // Send complete response
            let _ = response_tx
                .send(WorkerResponse::Complete {
                    narrative: response.narrative,
                    effects: response.effects,
                    world_update,
                    in_combat: response.in_combat,
                    is_player_turn: response.is_player_turn,
                })
                .await;
        }
        Err(e) => {
            let _ = response_tx.send(WorkerResponse::Error(e.to_string())).await;
        }
    }
}
