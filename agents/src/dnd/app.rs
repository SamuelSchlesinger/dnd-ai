//! Main application state and logic

use std::collections::VecDeque;

use crate::dnd::game::character::{create_sample_fighter, Character};
use crate::dnd::game::dice::{DiceExpression, RollResult};
use crate::dnd::game::state::{GameMode, GameWorld, NarrativeEntry, NarrativeType};
use crate::dnd::ui::render::{FocusedPanel, Overlay};
use crate::dnd::ui::theme::GameTheme;

/// Vim-style input modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputMode {
    /// Normal mode - navigation and hotkeys (default)
    #[default]
    Normal,
    /// Insert mode - free text input
    Insert,
    /// Command mode - entering : commands
    Command,
}

impl InputMode {
    /// Get a display string for the mode
    pub fn display(&self) -> &'static str {
        match self {
            InputMode::Normal => "NORMAL",
            InputMode::Insert => "INSERT",
            InputMode::Command => "COMMAND",
        }
    }
}

/// Main application state
pub struct AppState {
    // Game state
    pub game: GameWorld,

    // UI state
    pub theme: GameTheme,
    pub focused_panel: FocusedPanel,
    pub overlay: Option<Overlay>,

    // Narrative display
    pub narrative_history: Vec<NarrativeEntry>,
    pub narrative_scroll: usize,
    pub streaming_text: Option<String>,

    // Input state
    pub input_mode: InputMode,
    pub input_buffer: String,
    pub cursor_position: usize,
    pub input_history: VecDeque<String>,
    pub history_index: Option<usize>,

    // Status
    pub status_message: Option<String>,
    pub should_quit: bool,

    // Animation
    pub animation_frame: u8,
    pub pending_roll: Option<PendingRoll>,
}

/// A dice roll waiting to be displayed
pub struct PendingRoll {
    pub expression: String,
    pub purpose: String,
    pub dc: Option<i32>,
    pub result: Option<RollResult>,
}

impl AppState {
    /// Create a new application with a sample character
    pub fn new() -> Self {
        let character = create_sample_fighter("Hero");
        let game = GameWorld::new("The Adventure Begins", character);

        let mut state = Self {
            game,
            theme: GameTheme::default(),
            focused_panel: FocusedPanel::default(),
            overlay: None,
            narrative_history: Vec::new(),
            narrative_scroll: 0,
            streaming_text: None,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            cursor_position: 0,
            input_history: VecDeque::with_capacity(100),
            history_index: None,
            status_message: None,
            should_quit: false,
            animation_frame: 0,
            pending_roll: None,
        };

        // Add initial narrative
        state.add_narrative(
            "Welcome, adventurer! Your journey begins here.".to_string(),
            NarrativeType::DmNarration,
        );
        state.add_narrative(
            "Press 'i' to enter INSERT mode and type. Press '?' for help. Press ':' for commands.".to_string(),
            NarrativeType::System,
        );

        state
    }

    /// Create with a specific character
    pub fn with_character(character: Character, campaign_name: &str) -> Self {
        let game = GameWorld::new(campaign_name, character);

        Self {
            game,
            theme: GameTheme::default(),
            focused_panel: FocusedPanel::default(),
            overlay: None,
            narrative_history: Vec::new(),
            narrative_scroll: 0,
            streaming_text: None,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            cursor_position: 0,
            input_history: VecDeque::with_capacity(100),
            history_index: None,
            status_message: None,
            should_quit: false,
            animation_frame: 0,
            pending_roll: None,
        }
    }

    /// Enter insert mode
    pub fn enter_insert_mode(&mut self) {
        self.input_mode = InputMode::Insert;
    }

    /// Enter command mode (starts with :)
    pub fn enter_command_mode(&mut self) {
        self.input_mode = InputMode::Command;
        self.input_buffer.clear();
        self.input_buffer.push(':');
        self.cursor_position = 1;
    }

    /// Exit to normal mode
    pub fn enter_normal_mode(&mut self) {
        self.input_mode = InputMode::Normal;
        // Clear input buffer when leaving insert/command mode via Esc
        if self.input_mode != InputMode::Insert {
            self.input_buffer.clear();
            self.cursor_position = 0;
        }
    }

    /// Check if we're in a text input mode
    pub fn is_input_mode(&self) -> bool {
        matches!(self.input_mode, InputMode::Insert | InputMode::Command)
    }

    /// Add a narrative entry
    pub fn add_narrative(&mut self, content: String, entry_type: NarrativeType) {
        self.narrative_history.push(NarrativeEntry {
            content,
            entry_type,
            game_time: self.game.game_time.clone(),
        });
        // Auto-scroll to bottom
        self.scroll_to_bottom();
    }

    /// Scroll narrative to bottom
    pub fn scroll_to_bottom(&mut self) {
        self.narrative_scroll = self.narrative_history.len().saturating_sub(1);
    }

    /// Scroll narrative up
    pub fn scroll_up(&mut self, lines: usize) {
        self.narrative_scroll = self.narrative_scroll.saturating_sub(lines);
    }

    /// Scroll narrative down
    pub fn scroll_down(&mut self, lines: usize) {
        self.narrative_scroll = (self.narrative_scroll + lines)
            .min(self.narrative_history.len().saturating_sub(1));
    }

    /// Submit current input
    pub fn submit_input(&mut self) -> Option<String> {
        if self.input_buffer.is_empty() {
            return None;
        }

        let input = std::mem::take(&mut self.input_buffer);
        self.cursor_position = 0;

        // Add to history
        self.input_history.push_front(input.clone());
        if self.input_history.len() > 100 {
            self.input_history.pop_back();
        }
        self.history_index = None;

        Some(input)
    }

    /// Handle a typed character
    pub fn type_char(&mut self, c: char) {
        self.input_buffer.insert(self.cursor_position, c);
        self.cursor_position += 1;
    }

    /// Handle backspace
    pub fn backspace(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.input_buffer.remove(self.cursor_position);
        }
    }

    /// Handle delete
    pub fn delete(&mut self) {
        if self.cursor_position < self.input_buffer.len() {
            self.input_buffer.remove(self.cursor_position);
        }
    }

    /// Move cursor left
    pub fn cursor_left(&mut self) {
        self.cursor_position = self.cursor_position.saturating_sub(1);
    }

    /// Move cursor right
    pub fn cursor_right(&mut self) {
        self.cursor_position = (self.cursor_position + 1).min(self.input_buffer.len());
    }

    /// Move cursor to start
    pub fn cursor_home(&mut self) {
        self.cursor_position = 0;
    }

    /// Move cursor to end
    pub fn cursor_end(&mut self) {
        self.cursor_position = self.input_buffer.len();
    }

    /// Navigate to previous input in history
    pub fn history_prev(&mut self) {
        if self.input_history.is_empty() {
            return;
        }

        let new_index = match self.history_index {
            None => Some(0),
            Some(i) if i + 1 < self.input_history.len() => Some(i + 1),
            Some(i) => Some(i),
        };

        if let Some(idx) = new_index {
            if let Some(entry) = self.input_history.get(idx) {
                self.input_buffer = entry.clone();
                self.cursor_position = self.input_buffer.len();
                self.history_index = new_index;
            }
        }
    }

    /// Navigate to next input in history
    pub fn history_next(&mut self) {
        match self.history_index {
            None => {}
            Some(0) => {
                self.input_buffer.clear();
                self.cursor_position = 0;
                self.history_index = None;
            }
            Some(i) => {
                if let Some(entry) = self.input_history.get(i - 1) {
                    self.input_buffer = entry.clone();
                    self.cursor_position = self.input_buffer.len();
                    self.history_index = Some(i - 1);
                }
            }
        }
    }

    /// Toggle help overlay
    pub fn toggle_help(&mut self) {
        if matches!(self.overlay, Some(Overlay::Help)) {
            self.overlay = None;
        } else {
            self.overlay = Some(Overlay::Help);
        }
    }

    /// Close any open overlay
    pub fn close_overlay(&mut self) {
        self.overlay = None;
    }

    /// Show a dice roll
    pub fn show_dice_roll(&mut self, expression: &str, purpose: &str, dc: Option<i32>) {
        // Parse and roll
        if let Ok(expr) = DiceExpression::parse(expression) {
            let result = expr.roll();
            self.overlay = Some(Overlay::DiceRoll {
                result: Some(result.clone()),
                purpose: purpose.to_string(),
                dc,
            });

            // Also add to narrative
            let result_text = if let Some(dc_val) = dc {
                let outcome = if result.total >= dc_val {
                    "SUCCESS"
                } else {
                    "FAILURE"
                };
                format!(
                    "{}: {} = {} vs DC {} - {}",
                    purpose,
                    expression,
                    result.total,
                    dc_val,
                    outcome
                )
            } else {
                format!("{}: {} = {}", purpose, expression, result.total)
            };

            self.add_narrative(result_text, NarrativeType::System);
        }
    }

    /// Process a slash command
    pub fn process_command(&mut self, command: &str) -> bool {
        let parts: Vec<&str> = command.trim().split_whitespace().collect();
        if parts.is_empty() {
            return false;
        }

        match parts[0] {
            "/roll" => {
                if parts.len() > 1 {
                    let expression = parts[1..].join("");
                    self.show_dice_roll(&expression, "Manual Roll", None);
                } else {
                    self.status_message = Some("Usage: /roll XdY+Z".to_string());
                }
                true
            }
            "/quit" | "/exit" => {
                self.should_quit = true;
                true
            }
            "/help" => {
                self.toggle_help();
                true
            }
            "/rest" => {
                if parts.len() > 1 && parts[1] == "long" {
                    self.game.long_rest();
                    self.add_narrative(
                        "You take a long rest, recovering fully.".to_string(),
                        NarrativeType::System,
                    );
                } else {
                    self.game.short_rest();
                    self.add_narrative(
                        "You take a short rest.".to_string(),
                        NarrativeType::System,
                    );
                }
                true
            }
            _ => {
                self.status_message = Some(format!("Unknown command: {}", parts[0]));
                false
            }
        }
    }

    /// Tick for animations
    pub fn tick(&mut self) {
        self.animation_frame = self.animation_frame.wrapping_add(1);
    }

    /// Set status message (clears after display)
    pub fn set_status(&mut self, message: impl Into<String>) {
        self.status_message = Some(message.into());
    }

    /// Clear status message
    pub fn clear_status(&mut self) {
        self.status_message = None;
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
