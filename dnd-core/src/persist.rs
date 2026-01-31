//! Campaign persistence for save/load functionality.
//!
//! Provides robust serialization of game state and DM memory,
//! supporting both JSON (human-readable) and bincode (compact) formats.

use crate::dm::memory::{CampaignFact, FactCategory};
use crate::world::{Character, GameWorld};
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

    /// When the save was created (duplicated from parent for peek access).
    #[serde(default)]
    pub saved_at: String,
}

impl SavedCampaign {
    /// Create a new saved campaign from game state.
    pub fn new(
        world: GameWorld,
        campaign_facts: Vec<CampaignFact>,
        conversation_summary: Option<String>,
    ) -> Self {
        let saved_at = chrono_now();
        let metadata = SaveMetadata {
            character_name: world.player_character.name.clone(),
            campaign_name: world.campaign_name.clone(),
            level: world.player_character.level,
            location: world.current_location.name.clone(),
            play_time_minutes: 0, // TODO: Track actual play time
            days_elapsed: world.game_time.day as u32,
            saved_at: saved_at.clone(),
        };

        Self {
            version: SAVE_VERSION,
            saved_at,
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
                name: f
                    .content
                    .split(':')
                    .next()
                    .unwrap_or(&f.content)
                    .to_string(),
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
                // Use character count for unicode-safe truncation
                let char_count = n.content.chars().count();
                if char_count > 100 {
                    let truncated: String = n.content.chars().take(100).collect();
                    format!("{truncated}...")
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

// ============================================================================
// Character Persistence
// ============================================================================

/// Current character save file version.
const CHARACTER_SAVE_VERSION: u32 = 1;

/// A saved character that can be reused across sessions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedCharacter {
    /// Save format version for compatibility checking.
    pub version: u32,

    /// When the character was saved.
    pub saved_at: String,

    /// The complete character data.
    pub character: Character,

    /// Quick-access metadata about the character.
    pub metadata: CharacterMetadata,
}

/// Metadata about a saved character for quick display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterMetadata {
    /// Character name.
    pub name: String,

    /// Race name.
    pub race: String,

    /// Class name.
    pub class: String,

    /// Character level.
    pub level: u8,

    /// Whether the character has a backstory.
    pub has_backstory: bool,
}

impl SavedCharacter {
    /// Create a new saved character from a Character.
    pub fn new(character: Character) -> Self {
        let metadata = CharacterMetadata {
            name: character.name.clone(),
            race: character.race.name.clone(),
            class: character
                .classes
                .first()
                .map(|c| c.class.name().to_string())
                .unwrap_or_else(|| "Unknown".to_string()),
            level: character.level,
            has_backstory: character.backstory.is_some(),
        };

        Self {
            version: CHARACTER_SAVE_VERSION,
            saved_at: chrono_now(),
            character,
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

        if saved.version != CHARACTER_SAVE_VERSION {
            return Err(PersistError::VersionMismatch {
                expected: CHARACTER_SAVE_VERSION,
                found: saved.version,
            });
        }

        Ok(saved)
    }

    /// Get metadata without loading the full character.
    pub async fn peek_metadata(path: impl AsRef<Path>) -> Result<CharacterMetadata, PersistError> {
        let content = fs::read_to_string(path).await?;

        #[derive(Deserialize)]
        struct Partial {
            version: u32,
            metadata: CharacterMetadata,
        }

        let partial: Partial = serde_json::from_str(&content)?;

        if partial.version != CHARACTER_SAVE_VERSION {
            return Err(PersistError::VersionMismatch {
                expected: CHARACTER_SAVE_VERSION,
                found: partial.version,
            });
        }

        Ok(partial.metadata)
    }
}

/// Information about a character save file.
#[derive(Debug, Clone)]
pub struct CharacterSaveInfo {
    /// Path to the save file.
    pub path: String,

    /// Character metadata.
    pub metadata: CharacterMetadata,
}

/// List all character save files in a directory.
pub async fn list_character_saves(
    dir: impl AsRef<Path>,
) -> Result<Vec<CharacterSaveInfo>, PersistError> {
    let mut saves = Vec::new();

    // Create the directory if it doesn't exist
    let dir_path = dir.as_ref();
    if !dir_path.exists() {
        fs::create_dir_all(dir_path).await?;
        return Ok(saves);
    }

    let mut entries = fs::read_dir(dir_path).await?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.extension().map(|e| e == "json").unwrap_or(false) {
            if let Ok(metadata) = SavedCharacter::peek_metadata(&path).await {
                saves.push(CharacterSaveInfo {
                    path: path.to_string_lossy().to_string(),
                    metadata,
                });
            }
        }
    }

    // Sort by name
    saves.sort_by(|a, b| a.metadata.name.cmp(&b.metadata.name));
    Ok(saves)
}

/// Generate a save path for a character.
pub fn character_save_path(dir: impl AsRef<Path>, name: &str) -> std::path::PathBuf {
    let sanitized = name
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect::<String>();
    dir.as_ref().join(format!("{sanitized}.json"))
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

    #[test]
    fn test_saved_character_creation() {
        let character = create_sample_fighter("Thorin Ironfist");
        let saved = SavedCharacter::new(character);

        assert_eq!(saved.version, CHARACTER_SAVE_VERSION);
        assert_eq!(saved.metadata.name, "Thorin Ironfist");
        assert_eq!(saved.metadata.class, "Fighter");
        assert_eq!(saved.metadata.level, 3); // create_sample_fighter creates level 3
        assert!(!saved.metadata.has_backstory);
    }

    #[test]
    fn test_saved_character_with_backstory() {
        let mut character = create_sample_fighter("Aria Shadowblade");
        character.backstory = Some("A mysterious ranger from the northern forests.".to_string());

        let saved = SavedCharacter::new(character);

        assert!(saved.metadata.has_backstory);
        assert!(saved.character.backstory.is_some());
        assert_eq!(
            saved.character.backstory.as_ref().unwrap(),
            "A mysterious ranger from the northern forests."
        );
    }

    #[test]
    fn test_character_metadata() {
        let mut character = create_sample_fighter("Test Character");
        character.backstory = Some("Has a backstory".to_string());

        let saved = SavedCharacter::new(character);
        let metadata = &saved.metadata;

        assert_eq!(metadata.name, "Test Character");
        assert_eq!(metadata.race, "Human"); // create_sample_fighter uses Human
        assert_eq!(metadata.class, "Fighter");
        assert_eq!(metadata.level, 3);
        assert!(metadata.has_backstory);
    }

    #[test]
    fn test_character_save_path() {
        let path = character_save_path("/saves/characters", "Sir Reginald");
        assert!(path.to_string_lossy().contains("Sir_Reginald"));
        assert!(path.to_string_lossy().ends_with(".json"));
    }

    #[test]
    fn test_character_save_path_special_chars() {
        let path = character_save_path("saves/characters", "Bob's Character!@#");
        // Special characters should be replaced with underscores
        assert!(path.to_string_lossy().contains("Bob_s_Character"));
        assert!(!path.to_string_lossy().contains("!"));
        assert!(!path.to_string_lossy().contains("@"));
    }

    #[tokio::test]
    async fn test_saved_character_save_and_load() {
        use tempfile::TempDir;

        // Create a temp directory for testing
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let save_path = temp_dir.path().join("test_character.json");

        // Create and save a character
        let mut character = create_sample_fighter("Save Test Hero");
        character.backstory = Some("Testing save functionality.".to_string());

        let saved = SavedCharacter::new(character);
        saved
            .save_json(&save_path)
            .await
            .expect("Save should succeed");

        // Verify file exists
        assert!(save_path.exists());

        // Load the character
        let loaded = SavedCharacter::load_json(&save_path)
            .await
            .expect("Load should succeed");

        assert_eq!(loaded.character.name, "Save Test Hero");
        assert_eq!(loaded.metadata.name, "Save Test Hero");
        assert!(loaded.metadata.has_backstory);
        assert_eq!(
            loaded.character.backstory.as_ref().unwrap(),
            "Testing save functionality."
        );
    }

    #[tokio::test]
    async fn test_peek_character_metadata() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let save_path = temp_dir.path().join("peek_test.json");

        let character = create_sample_fighter("Peek Test");
        let saved = SavedCharacter::new(character);
        saved
            .save_json(&save_path)
            .await
            .expect("Save should succeed");

        // Peek at metadata without loading full character
        let metadata = SavedCharacter::peek_metadata(&save_path)
            .await
            .expect("Peek should succeed");

        assert_eq!(metadata.name, "Peek Test");
        assert_eq!(metadata.class, "Fighter");
    }

    #[tokio::test]
    async fn test_list_character_saves() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let char_dir = temp_dir.path().join("characters");
        std::fs::create_dir_all(&char_dir).expect("Create dir should succeed");

        // Save multiple characters
        let characters = vec![
            create_sample_fighter("Alpha"),
            create_sample_fighter("Beta"),
            create_sample_fighter("Charlie"),
        ];

        for character in characters {
            let saved = SavedCharacter::new(character.clone());
            let path = character_save_path(&char_dir, &character.name);
            saved.save_json(&path).await.expect("Save should succeed");
        }

        // List saves
        let saves = list_character_saves(&char_dir)
            .await
            .expect("List should succeed");

        assert_eq!(saves.len(), 3);

        // Should be sorted by name
        let names: Vec<_> = saves.iter().map(|s| s.metadata.name.as_str()).collect();
        assert_eq!(names, vec!["Alpha", "Beta", "Charlie"]);
    }

    #[tokio::test]
    async fn test_list_character_saves_empty_dir() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let char_dir = temp_dir.path().join("empty_characters");

        // List on non-existent directory should return empty vec and create the directory
        let saves = list_character_saves(&char_dir)
            .await
            .expect("List should succeed");

        assert!(saves.is_empty());
        assert!(char_dir.exists()); // Directory should be created
    }
}
