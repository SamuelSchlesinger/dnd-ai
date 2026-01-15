//! Game world state management
//!
//! Tracks the overall game state including location, time, quests, and NPCs.

use super::character::{Character, CharacterId};
use super::combat::CombatState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Unique identifier for locations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LocationId(pub Uuid);

impl LocationId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for LocationId {
    fn default() -> Self {
        Self::new()
    }
}

/// Current game mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameMode {
    /// Exploring, investigating, traveling
    Exploration,
    /// Active combat encounter
    Combat,
    /// Conversation with NPC
    Dialogue,
    /// Short or long rest
    Rest,
    /// Shopping/trading
    Shopping,
    /// Character creation or level up
    CharacterManagement,
}

impl Default for GameMode {
    fn default() -> Self {
        GameMode::Exploration
    }
}

/// In-game time tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameTime {
    pub year: i32,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
}

impl GameTime {
    pub fn new(year: i32, month: u8, day: u8, hour: u8, minute: u8) -> Self {
        Self { year, month, day, hour, minute }
    }

    /// Advance time by minutes
    pub fn advance_minutes(&mut self, minutes: u32) {
        let total_minutes = self.minute as u32 + minutes;
        self.minute = (total_minutes % 60) as u8;

        let hours_to_add = total_minutes / 60;
        self.advance_hours(hours_to_add);
    }

    /// Advance time by hours
    pub fn advance_hours(&mut self, hours: u32) {
        let total_hours = self.hour as u32 + hours;
        self.hour = (total_hours % 24) as u8;

        let days_to_add = total_hours / 24;
        self.advance_days(days_to_add);
    }

    /// Advance time by days
    pub fn advance_days(&mut self, days: u32) {
        // Simplified calendar (30 days per month, 12 months per year)
        let total_days = self.day as u32 + days;
        self.day = ((total_days - 1) % 30 + 1) as u8;

        let months_to_add = (total_days - 1) / 30;
        self.advance_months(months_to_add);
    }

    /// Advance time by months
    pub fn advance_months(&mut self, months: u32) {
        let total_months = self.month as u32 + months;
        self.month = ((total_months - 1) % 12 + 1) as u8;

        let years_to_add = (total_months - 1) / 12;
        self.year += years_to_add as i32;
    }

    /// Check if it's daytime (6am - 6pm)
    pub fn is_daytime(&self) -> bool {
        self.hour >= 6 && self.hour < 18
    }

    /// Get time of day description
    pub fn time_of_day(&self) -> &'static str {
        match self.hour {
            5..=7 => "dawn",
            8..=11 => "morning",
            12..=13 => "midday",
            14..=17 => "afternoon",
            18..=20 => "evening",
            21..=23 | 0..=4 => "night",
            _ => "unknown",
        }
    }

    /// Format as string
    pub fn to_string_detailed(&self) -> String {
        let month_name = match self.month {
            1 => "Hammer",
            2 => "Alturiak",
            3 => "Ches",
            4 => "Tarsakh",
            5 => "Mirtul",
            6 => "Kythorn",
            7 => "Flamerule",
            8 => "Eleasis",
            9 => "Eleint",
            10 => "Marpenoth",
            11 => "Uktar",
            12 => "Nightal",
            _ => "Unknown",
        };

        format!(
            "{} {}, {} DR - {}:{:02}",
            month_name, self.day, self.year, self.hour, self.minute
        )
    }
}

impl Default for GameTime {
    fn default() -> Self {
        // Default to a common Forgotten Realms starting date
        Self::new(1492, 3, 15, 10, 0) // Mid-morning, 15th of Ches, 1492 DR
    }
}

/// A location in the game world
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub id: LocationId,
    pub name: String,
    pub location_type: LocationType,
    pub description: String,
    pub connections: Vec<LocationConnection>,
    pub npcs_present: Vec<CharacterId>,
    pub items: Vec<String>,
    pub discovered_secrets: Vec<String>,
}

impl Location {
    pub fn new(name: impl Into<String>, location_type: LocationType) -> Self {
        Self {
            id: LocationId::new(),
            name: name.into(),
            location_type,
            description: String::new(),
            connections: Vec::new(),
            npcs_present: Vec::new(),
            items: Vec::new(),
            discovered_secrets: Vec::new(),
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LocationType {
    Wilderness,
    Town,
    City,
    Dungeon,
    Building,
    Room,
    Road,
    Cave,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationConnection {
    pub destination_id: LocationId,
    pub destination_name: String,
    pub direction: Option<String>,
    pub travel_time_minutes: u32,
    pub description: String,
}

/// A quest or objective
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quest {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub status: QuestStatus,
    pub objectives: Vec<QuestObjective>,
    pub rewards: Vec<String>,
    pub giver: Option<String>,
}

impl Quest {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            description: description.into(),
            status: QuestStatus::Active,
            objectives: Vec::new(),
            rewards: Vec::new(),
            giver: None,
        }
    }

    pub fn is_complete(&self) -> bool {
        !self.objectives.is_empty() && self.objectives.iter().all(|o| o.completed)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuestStatus {
    Active,
    Completed,
    Failed,
    Abandoned,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestObjective {
    pub description: String,
    pub completed: bool,
    pub optional: bool,
}

/// An NPC in the game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NPC {
    pub id: CharacterId,
    pub name: String,
    pub description: String,
    pub personality: String,
    pub occupation: Option<String>,
    pub location_id: Option<LocationId>,
    pub disposition: Disposition,
    pub known_information: Vec<String>,
    pub dialogue_history: Vec<String>,
}

impl NPC {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: CharacterId::new(),
            name: name.into(),
            description: String::new(),
            personality: String::new(),
            occupation: None,
            location_id: None,
            disposition: Disposition::Neutral,
            known_information: Vec::new(),
            dialogue_history: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Disposition {
    Hostile,
    Unfriendly,
    Neutral,
    Friendly,
    Helpful,
}

/// The complete game world state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameWorld {
    pub session_id: Uuid,
    pub campaign_name: String,

    // Player character
    pub player_character: Character,

    // NPCs
    pub npcs: HashMap<CharacterId, NPC>,

    // Current state
    pub mode: GameMode,
    pub combat: Option<CombatState>,
    pub current_location: Location,
    pub game_time: GameTime,

    // Locations
    pub known_locations: HashMap<LocationId, Location>,

    // Campaign progress
    pub quests: Vec<Quest>,
    pub journal_entries: Vec<JournalEntry>,

    // Session tracking
    pub session_number: u32,
    pub narrative_history: Vec<NarrativeEntry>,
}

impl GameWorld {
    pub fn new(campaign_name: impl Into<String>, player_character: Character) -> Self {
        let starting_location = Location::new("Starting Location", LocationType::Town)
            .with_description("A quiet place where your adventure begins.");

        let mut known_locations = HashMap::new();
        known_locations.insert(starting_location.id, starting_location.clone());

        Self {
            session_id: Uuid::new_v4(),
            campaign_name: campaign_name.into(),
            player_character,
            npcs: HashMap::new(),
            mode: GameMode::Exploration,
            combat: None,
            current_location: starting_location,
            game_time: GameTime::default(),
            known_locations,
            quests: Vec::new(),
            journal_entries: Vec::new(),
            session_number: 1,
            narrative_history: Vec::new(),
        }
    }

    /// Start combat
    pub fn start_combat(&mut self) -> &mut CombatState {
        self.mode = GameMode::Combat;
        self.combat = Some(CombatState::new());
        self.combat.as_mut().unwrap()
    }

    /// End combat
    pub fn end_combat(&mut self) {
        if let Some(ref mut combat) = self.combat {
            combat.end_combat();
        }
        self.mode = GameMode::Exploration;
    }

    /// Enter dialogue mode
    pub fn start_dialogue(&mut self) {
        self.mode = GameMode::Dialogue;
    }

    /// End dialogue
    pub fn end_dialogue(&mut self) {
        self.mode = GameMode::Exploration;
    }

    /// Take a short rest
    pub fn short_rest(&mut self) {
        self.mode = GameMode::Rest;
        self.game_time.advance_hours(1);

        // Recover hit dice spending opportunity handled by UI/agent
        self.mode = GameMode::Exploration;
    }

    /// Take a long rest
    pub fn long_rest(&mut self) {
        self.mode = GameMode::Rest;
        self.game_time.advance_hours(8);

        // Full HP recovery
        let max_hp = self.player_character.hit_points.maximum;
        self.player_character.hit_points.current = max_hp;

        // Recover half hit dice
        self.player_character.hit_dice.recover_half();

        // Recover spell slots
        if let Some(ref mut spellcasting) = self.player_character.spellcasting {
            spellcasting.spell_slots.recover_all();
        }

        // Reset feature uses that recharge on long rest
        for feature in &mut self.player_character.features {
            if let Some(ref mut uses) = feature.uses {
                if matches!(uses.recharge, super::character::RechargeType::LongRest) {
                    uses.current = uses.maximum;
                }
            }
        }

        self.mode = GameMode::Exploration;
    }

    /// Add a narrative entry to history
    pub fn add_narrative(&mut self, content: String, entry_type: NarrativeType) {
        self.narrative_history.push(NarrativeEntry {
            content,
            entry_type,
            game_time: self.game_time.clone(),
        });
    }

    /// Get recent narrative for context
    pub fn recent_narrative(&self, count: usize) -> Vec<&NarrativeEntry> {
        self.narrative_history
            .iter()
            .rev()
            .take(count)
            .collect()
    }
}

/// Journal entry for player notes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntry {
    pub title: String,
    pub content: String,
    pub game_time: GameTime,
    pub location: String,
}

/// Entry in the narrative history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeEntry {
    pub content: String,
    pub entry_type: NarrativeType,
    pub game_time: GameTime,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum NarrativeType {
    /// DM narration
    DmNarration,
    /// Player action
    PlayerAction,
    /// NPC dialogue
    NpcDialogue,
    /// Combat description
    Combat,
    /// System message
    System,
}
