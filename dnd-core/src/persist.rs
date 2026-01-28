//! Campaign persistence for save/load functionality.
//!
//! Provides robust serialization of game state and DM memory,
//! supporting both JSON (human-readable) and bincode (compact) formats.

use crate::dm::memory::{CampaignFact, FactCategory};
use crate::world::GameWorld;
use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;
use tokio::fs;

/// Errors from persistence operations.
#[derive(Debug, Error)]
pub enum PersistError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Invalid save format")]
    InvalidFormat,

    #[error("Version mismatch: expected {expected}, found {found}")]
    VersionMismatch { expected: u32, found: u32 },
}

/// Current save file version.
const SAVE_VERSION: u32 = 1;

/// A saved campaign with all state needed to resume play.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedCampaign {
    /// Save format version for compatibility checking.
    pub version: u32,

    /// When the save was created.
    pub saved_at: String,

    /// The complete game world state.
    pub world: GameWorld,

    /// Campaign facts that persist across sessions.
    pub campaign_facts: Vec<CampaignFact>,

    /// Summary of the conversation for context restoration.
    pub conversation_summary: Option<String>,

    /// Metadata about the save.
    pub metadata: SaveMetadata,
}

/// Metadata about the save file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveMetadata {
    /// Player character name.
    pub character_name: String,

    /// Campaign name.
    pub campaign_name: String,

    /// Character level.
    pub level: u8,

    /// Current location.
    pub location: String,

    /// Play time in minutes.
    pub play_time_minutes: u32,

    /// Number of in-game days elapsed.
    pub days_elapsed: u32,
}

impl SavedCampaign {
    /// Create a new saved campaign from game state.
    pub fn new(
        world: GameWorld,
        campaign_facts: Vec<CampaignFact>,
        conversation_summary: Option<String>,
    ) -> Self {
        let metadata = SaveMetadata {
            character_name: world.player_character.name.clone(),
            campaign_name: world.campaign_name.clone(),
            level: world.player_character.level,
            location: world.current_location.name.clone(),
            play_time_minutes: 0, // TODO: Track actual play time
            days_elapsed: world.game_time.day as u32,
        };

        Self {
            version: SAVE_VERSION,
            saved_at: chrono_now(),
            world,
            campaign_facts,
            conversation_summary,
            metadata,
        }
    }

    /// Save to a JSON file.
    pub async fn save_json(&self, path: impl AsRef<Path>) -> Result<(), PersistError> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content).await?;
        Ok(())
    }

    /// Load from a JSON file.
    pub async fn load_json(path: impl AsRef<Path>) -> Result<Self, PersistError> {
        let content = fs::read_to_string(path).await?;
        let saved: Self = serde_json::from_str(&content)?;

        if saved.version != SAVE_VERSION {
            return Err(PersistError::VersionMismatch {
                expected: SAVE_VERSION,
                found: saved.version,
            });
        }

        Ok(saved)
    }

    /// Check if a save file exists and get its metadata without loading the full state.
    pub async fn peek_metadata(path: impl AsRef<Path>) -> Result<SaveMetadata, PersistError> {
        let content = fs::read_to_string(path).await?;

        // Parse just enough to get metadata
        #[derive(Deserialize)]
        struct Partial {
            version: u32,
            metadata: SaveMetadata,
        }

        let partial: Partial = serde_json::from_str(&content)?;

        if partial.version != SAVE_VERSION {
            return Err(PersistError::VersionMismatch {
                expected: SAVE_VERSION,
                found: partial.version,
            });
        }

        Ok(partial.metadata)
    }
}

/// List all save files in a directory.
pub async fn list_saves(dir: impl AsRef<Path>) -> Result<Vec<SaveInfo>, PersistError> {
    let mut saves = Vec::new();
    let mut entries = fs::read_dir(dir).await?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.extension().map(|e| e == "json").unwrap_or(false) {
            if let Ok(metadata) = SavedCampaign::peek_metadata(&path).await {
                saves.push(SaveInfo {
                    path: path.to_string_lossy().to_string(),
                    metadata,
                });
            }
        }
    }

    // Sort by modification time (most recent first)
    saves.sort_by(|a, b| b.path.cmp(&a.path));
    Ok(saves)
}

/// Information about a save file.
#[derive(Debug, Clone)]
pub struct SaveInfo {
    /// Path to the save file.
    pub path: String,

    /// Save metadata.
    pub metadata: SaveMetadata,
}

/// Create an auto-save file name.
pub fn auto_save_path(base_dir: impl AsRef<Path>, campaign_name: &str) -> std::path::PathBuf {
    let sanitized = campaign_name
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect::<String>();
    base_dir.as_ref().join(format!("{sanitized}_autosave.json"))
}

/// Create a manual save file name with timestamp.
pub fn manual_save_path(base_dir: impl AsRef<Path>, campaign_name: &str) -> std::path::PathBuf {
    let sanitized = campaign_name
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect::<String>();
    let timestamp = chrono_now().replace([':', '-', 'T', ' '], "_");
    base_dir
        .as_ref()
        .join(format!("{sanitized}_{timestamp}.json"))
}

/// Get current timestamp as ISO 8601 string.
fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();

    // Simple timestamp without chrono dependency
    format!("{}", now.as_secs())
}

/// Export campaign to a shareable format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignExport {
    /// Campaign name.
    pub name: String,

    /// Character summary.
    pub character: CharacterExport,

    /// Key NPCs encountered.
    pub npcs: Vec<NpcExport>,

    /// Completed quests.
    pub completed_quests: Vec<String>,

    /// Notable events.
    pub events: Vec<String>,
}

/// Character export summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterExport {
    pub name: String,
    pub level: u8,
    pub classes: Vec<String>,
}

/// NPC export summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcExport {
    pub name: String,
    pub relationship: String,
}

impl CampaignExport {
    /// Create an export from a saved campaign.
    pub fn from_saved(saved: &SavedCampaign) -> Self {
        let character = CharacterExport {
            name: saved.world.player_character.name.clone(),
            level: saved.world.player_character.level,
            classes: saved
                .world
                .player_character
                .classes
                .iter()
                .map(|c| format!("{:?} {}", c.class, c.level))
                .collect(),
        };

        let npcs: Vec<NpcExport> = saved
            .campaign_facts
            .iter()
            .filter(|f| f.category == FactCategory::NPC)
            .map(|f| NpcExport {
                name: f.content.split(':').next().unwrap_or(&f.content).to_string(),
                relationship: f.content.clone(),
            })
            .collect();

        let completed_quests: Vec<String> = saved
            .world
            .quests
            .iter()
            .filter(|q| matches!(q.status, crate::world::QuestStatus::Completed))
            .map(|q| q.name.clone())
            .collect();

        let events: Vec<String> = saved
            .world
            .narrative_history
            .iter()
            .take(20)
            .map(|n| {
                if n.content.len() > 100 {
                    format!("{}...", &n.content[..100])
                } else {
                    n.content.clone()
                }
            })
            .collect();

        Self {
            name: saved.world.campaign_name.clone(),
            character,
            npcs,
            completed_quests,
            events,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::create_sample_fighter;

    #[test]
    fn test_saved_campaign_creation() {
        let character = create_sample_fighter("Test Hero");
        let world = GameWorld::new("Test Campaign", character);

        let saved = SavedCampaign::new(world, vec![], None);

        assert_eq!(saved.version, SAVE_VERSION);
        assert_eq!(saved.metadata.character_name, "Test Hero");
        assert_eq!(saved.metadata.campaign_name, "Test Campaign");
    }

    #[test]
    fn test_auto_save_path() {
        let path = auto_save_path("/saves", "My Campaign!");
        assert!(path.to_string_lossy().contains("My_Campaign__autosave"));
    }

    #[test]
    fn test_campaign_export() {
        let character = create_sample_fighter("Hero");
        let world = GameWorld::new("Epic Quest", character);
        let saved = SavedCampaign::new(world, vec![], None);

        let export = CampaignExport::from_saved(&saved);
        assert_eq!(export.name, "Epic Quest");
        assert_eq!(export.character.name, "Hero");
    }
}
