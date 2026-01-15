//! D&D 5e combat tracking
//!
//! Initiative, turn order, and combat state management.

use super::character::CharacterId;
use super::conditions::Condition;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Unique identifier for combat encounters
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CombatId(pub Uuid);

impl CombatId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for CombatId {
    fn default() -> Self {
        Self::new()
    }
}

/// Combat encounter state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatState {
    pub id: CombatId,
    pub active: bool,
    pub round: u32,
    pub turn_index: usize,
    pub initiative_order: Vec<InitiativeEntry>,
    pub combatants: HashMap<CharacterId, CombatantStatus>,
    pub environment: CombatEnvironment,
    pub log: Vec<CombatLogEntry>,
}

impl CombatState {
    pub fn new() -> Self {
        Self {
            id: CombatId::new(),
            active: false,
            round: 0,
            turn_index: 0,
            initiative_order: Vec::new(),
            combatants: HashMap::new(),
            environment: CombatEnvironment::default(),
            log: Vec::new(),
        }
    }

    /// Start combat with the rolled initiatives
    pub fn start(&mut self) {
        self.active = true;
        self.round = 1;
        self.turn_index = 0;
        self.sort_initiative();

        // Reset all combatants for first turn
        for status in self.combatants.values_mut() {
            status.reset_turn();
        }

        self.log.push(CombatLogEntry {
            round: self.round,
            entry_type: CombatLogType::CombatStart,
            description: "Combat begins!".to_string(),
        });
    }

    /// Add a combatant with their initiative roll
    pub fn add_combatant(
        &mut self,
        character_id: CharacterId,
        name: String,
        initiative_roll: i32,
        initiative_modifier: i8,
        is_player: bool,
    ) {
        self.initiative_order.push(InitiativeEntry {
            character_id,
            name: name.clone(),
            initiative_total: initiative_roll,
            initiative_modifier,
            is_player,
        });

        self.combatants.insert(
            character_id,
            CombatantStatus {
                character_id,
                name,
                has_action: true,
                has_bonus_action: true,
                has_reaction: true,
                movement_remaining: 30, // Default, should be set properly
                has_object_interaction: true,
                is_surprised: false,
                temporary_conditions: Vec::new(),
            },
        );

        if self.active {
            self.sort_initiative();
        }
    }

    /// Sort initiative order (highest first, dex mod for ties)
    fn sort_initiative(&mut self) {
        self.initiative_order.sort_by(|a, b| {
            b.initiative_total
                .cmp(&a.initiative_total)
                .then_with(|| b.initiative_modifier.cmp(&a.initiative_modifier))
        });
    }

    /// Get the current combatant
    pub fn current_combatant(&self) -> Option<&InitiativeEntry> {
        self.initiative_order.get(self.turn_index)
    }

    /// Get current combatant's status
    pub fn current_combatant_status(&self) -> Option<&CombatantStatus> {
        self.current_combatant()
            .and_then(|entry| self.combatants.get(&entry.character_id))
    }

    /// Get mutable current combatant's status
    pub fn current_combatant_status_mut(&mut self) -> Option<&mut CombatantStatus> {
        let id = self.current_combatant()?.character_id;
        self.combatants.get_mut(&id)
    }

    /// Advance to the next turn
    pub fn next_turn(&mut self) {
        if !self.active || self.initiative_order.is_empty() {
            return;
        }

        // Log end of current turn
        if let Some(current) = self.current_combatant() {
            self.log.push(CombatLogEntry {
                round: self.round,
                entry_type: CombatLogType::TurnEnd {
                    character: current.name.clone(),
                },
                description: format!("{}'s turn ends.", current.name),
            });
        }

        self.turn_index += 1;

        // Check for new round
        if self.turn_index >= self.initiative_order.len() {
            self.turn_index = 0;
            self.round += 1;

            self.log.push(CombatLogEntry {
                round: self.round,
                entry_type: CombatLogType::RoundStart,
                description: format!("Round {} begins!", self.round),
            });
        }

        // Reset the new combatant's turn resources
        if let Some(status) = self.current_combatant_status_mut() {
            status.reset_turn();
        }

        // Log start of new turn
        if let Some(current) = self.current_combatant() {
            self.log.push(CombatLogEntry {
                round: self.round,
                entry_type: CombatLogType::TurnStart {
                    character: current.name.clone(),
                },
                description: format!("{}'s turn begins.", current.name),
            });
        }
    }

    /// End combat
    pub fn end_combat(&mut self) {
        self.active = false;
        self.log.push(CombatLogEntry {
            round: self.round,
            entry_type: CombatLogType::CombatEnd,
            description: "Combat ends!".to_string(),
        });
    }

    /// Remove a combatant (death, fled, etc.)
    pub fn remove_combatant(&mut self, character_id: CharacterId) {
        self.initiative_order
            .retain(|e| e.character_id != character_id);
        self.combatants.remove(&character_id);

        // Adjust turn index if needed
        if self.turn_index >= self.initiative_order.len() && !self.initiative_order.is_empty() {
            self.turn_index = 0;
            self.round += 1;
        }
    }

    /// Get combatants in initiative order
    pub fn get_ordered_combatants(&self) -> &[InitiativeEntry] {
        &self.initiative_order
    }

    /// Check if it's a specific character's turn
    pub fn is_turn(&self, character_id: CharacterId) -> bool {
        self.current_combatant()
            .map(|c| c.character_id == character_id)
            .unwrap_or(false)
    }

    /// Log a combat action
    pub fn log_action(&mut self, entry_type: CombatLogType, description: String) {
        self.log.push(CombatLogEntry {
            round: self.round,
            entry_type,
            description,
        });
    }
}

impl Default for CombatState {
    fn default() -> Self {
        Self::new()
    }
}

/// Entry in the initiative order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitiativeEntry {
    pub character_id: CharacterId,
    pub name: String,
    pub initiative_total: i32,
    pub initiative_modifier: i8,
    pub is_player: bool,
}

/// Per-combatant status tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatantStatus {
    pub character_id: CharacterId,
    pub name: String,
    pub has_action: bool,
    pub has_bonus_action: bool,
    pub has_reaction: bool,
    pub movement_remaining: u32,
    pub has_object_interaction: bool,
    pub is_surprised: bool,
    /// Conditions that only last for this combat
    pub temporary_conditions: Vec<Condition>,
}

impl CombatantStatus {
    /// Reset turn resources
    pub fn reset_turn(&mut self) {
        self.has_action = !self.is_surprised;
        self.has_bonus_action = !self.is_surprised;
        self.has_reaction = true;
        self.movement_remaining = 30; // Should be set from character speed
        self.has_object_interaction = true;
        self.is_surprised = false; // Surprise only lasts first round
    }

    /// Use action
    pub fn use_action(&mut self) -> bool {
        if self.has_action {
            self.has_action = false;
            true
        } else {
            false
        }
    }

    /// Use bonus action
    pub fn use_bonus_action(&mut self) -> bool {
        if self.has_bonus_action {
            self.has_bonus_action = false;
            true
        } else {
            false
        }
    }

    /// Use reaction
    pub fn use_reaction(&mut self) -> bool {
        if self.has_reaction {
            self.has_reaction = false;
            true
        } else {
            false
        }
    }

    /// Use movement
    pub fn use_movement(&mut self, feet: u32) -> bool {
        if self.movement_remaining >= feet {
            self.movement_remaining -= feet;
            true
        } else {
            false
        }
    }
}

/// Combat environment details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatEnvironment {
    pub lighting: LightingLevel,
    pub terrain: Vec<TerrainFeature>,
    pub description: String,
}

impl Default for CombatEnvironment {
    fn default() -> Self {
        Self {
            lighting: LightingLevel::BrightLight,
            terrain: Vec::new(),
            description: "An open area".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LightingLevel {
    BrightLight,
    DimLight,
    Darkness,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainFeature {
    pub name: String,
    pub feature_type: TerrainType,
    pub description: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TerrainType {
    DifficultTerrain,
    HalfCover,
    ThreeQuartersCover,
    FullCover,
    Hazard,
    Obstacle,
}

/// Combat log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatLogEntry {
    pub round: u32,
    pub entry_type: CombatLogType,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CombatLogType {
    CombatStart,
    CombatEnd,
    RoundStart,
    TurnStart { character: String },
    TurnEnd { character: String },
    Attack { attacker: String, target: String, hit: bool, damage: Option<i32> },
    Spell { caster: String, spell: String },
    Movement { character: String },
    Healing { target: String, amount: i32 },
    Condition { target: String, condition: String, applied: bool },
    DeathSave { character: String, success: bool },
    Death { character: String },
    Other,
}
