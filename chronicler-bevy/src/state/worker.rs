//! Worker thread communication types.

use chronicler_core::rules::Effect;
use chronicler_core::GameSession;
use tokio::sync::mpsc;

use super::WorldUpdate;

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
            Some(WorkerRequest::Load(path)) => {
                let client = session.dm().client().clone();
                match GameSession::load_with_client(&path, client).await {
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
                }
            }
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
    let effect_tx = response_tx.clone();

    let result = session
        .player_action_streaming_with_effects(
            input,
            |text| {
                let _ = stream_tx.try_send(WorkerResponse::StreamChunk(text.to_string()));
            },
            |effect| {
                // Stream effects in real-time for immediate sound/animation
                let _ = effect_tx.try_send(WorkerResponse::Effect(effect.clone()));
            },
        )
        .await;

    match result {
        Ok(response) => {
            // Effects already sent in real-time via callback

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
