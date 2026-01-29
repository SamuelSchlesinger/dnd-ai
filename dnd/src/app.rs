//! Main application state and logic

use std::collections::VecDeque;
use std::path::PathBuf;

use dnd_core::dice::{DiceExpression, RollResult};
use dnd_core::world::{GameMode, NarrativeType};
use tokio::sync::mpsc;

use crate::ai_worker::{WorkerRequest, WorkerResponse, WorldUpdate};
use crate::ui::theme::GameTheme;
use crate::ui::widgets::narrative::NarrativeItem;
use crate::ui::{FocusedPanel, Overlay};

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

/// State for a dice roll in progress (for animation)
#[derive(Debug, Clone)]
pub struct RollingDice {
    /// The dice expression string (e.g., "2d6+3")
    pub expression: String,
    /// Purpose of the roll (e.g., "Attack Roll")
    pub purpose: String,
    /// Difficulty class if this is a check
    pub dc: Option<i32>,
    /// Number of animation frames elapsed
    pub frames_elapsed: u8,
    /// The result once calculated (after animation delay)
    pub result: Option<RollResult>,
}

/// Main application state
pub struct App {
    // Channel communication with AI worker
    pub request_tx: mpsc::Sender<WorkerRequest>,
    pub response_rx: mpsc::Receiver<WorkerResponse>,

    // Local world state snapshot for rendering
    pub world: WorldUpdate,

    // UI state
    pub theme: GameTheme,
    pub focused_panel: FocusedPanel,
    overlay: Option<Overlay>,

    // Narrative display
    pub narrative_history: Vec<NarrativeItem>,
    pub narrative_scroll: usize,
    pub streaming_text: Option<String>,
    pub scroll_locked_to_bottom: bool, // True = auto-scroll on new content

    // Input state
    pub input_mode: InputMode,
    input_buffer: String,
    cursor_position: usize,
    pub input_history: VecDeque<String>,
    pub history_index: Option<usize>,
    pub saved_input: Option<String>, // Saved current input when browsing history

    // Status
    status_message: Option<String>,
    pub should_quit: bool,
    pub quit_after_save: bool,

    // Animation
    pub animation_frame: u8,
    rolling_dice: Option<RollingDice>,

    // AI processing
    pub ai_processing: bool,
}

impl App {
    /// Create a new application with channel endpoints and initial world state
    pub fn new(
        request_tx: mpsc::Sender<WorkerRequest>,
        response_rx: mpsc::Receiver<WorkerResponse>,
        world: WorldUpdate,
    ) -> Self {
        let class_str = world.player_class.as_deref().unwrap_or("adventurer");
        let welcome = format!(
            "{} the {} steps into {}.\n\nThe familiar sounds and smells of the inn surround you. What would you like to do?",
            world.player_name,
            class_str,
            world.current_location
        );

        let mut app = Self {
            request_tx,
            response_rx,
            world,
            theme: GameTheme::default(),
            focused_panel: FocusedPanel::default(),
            overlay: None,
            narrative_history: Vec::new(),
            narrative_scroll: 0,
            streaming_text: None,
            scroll_locked_to_bottom: true,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            cursor_position: 0,
            input_history: VecDeque::with_capacity(100),
            history_index: None,
            saved_input: None,
            status_message: None,
            should_quit: false,
            quit_after_save: false,
            animation_frame: 0,
            rolling_dice: None,
            ai_processing: false,
        };

        app.add_narrative(welcome, NarrativeType::DmNarration);
        app.add_narrative(
            "Press 'i' to describe your action, '?' for help, or scroll with j/k".to_string(),
            NarrativeType::System,
        );

        app
    }

    /// Get the current game mode
    pub fn game_mode(&self) -> GameMode {
        self.world.mode
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
        // Clear command buffer when leaving command mode
        if self.input_buffer.starts_with(':') {
            self.input_buffer.clear();
            self.cursor_position = 0;
        }
    }

    /// Add a narrative entry
    pub fn add_narrative(&mut self, content: String, entry_type: NarrativeType) {
        // Don't auto-scroll for player actions - let them be visible where user is looking
        // Only auto-scroll for DM/system responses
        let should_scroll = self.scroll_locked_to_bottom
            && !matches!(entry_type, NarrativeType::PlayerAction);

        self.narrative_history.push(NarrativeItem {
            content,
            entry_type,
        });

        if should_scroll {
            self.scroll_to_bottom();
        }
    }

    /// Append text to the streaming buffer
    pub fn append_streaming_text(&mut self, text: &str) {
        match &mut self.streaming_text {
            Some(existing) => existing.push_str(text),
            None => self.streaming_text = Some(text.to_string()),
        }
        // Auto-scroll when streaming
        if self.scroll_locked_to_bottom {
            self.scroll_to_bottom();
        }
    }

    /// Finalize the streaming response into a narrative entry
    pub fn finalize_streaming(&mut self) {
        if let Some(text) = self.streaming_text.take() {
            if !text.is_empty() {
                self.add_narrative(text, NarrativeType::DmNarration);
            }
        }
    }

    /// Scroll narrative to bottom and lock to bottom
    pub fn scroll_to_bottom(&mut self) {
        // Set to max value - the widget will cap it to actual max_scroll
        self.narrative_scroll = usize::MAX / 2;
        self.scroll_locked_to_bottom = true;
    }

    /// Estimate max scroll based on narrative content
    /// Uses conservative estimate assuming ~60 char effective width
    fn estimate_max_scroll(&self) -> usize {
        const ESTIMATED_WIDTH: usize = 60;
        const ESTIMATED_VISIBLE_HEIGHT: usize = 20;

        let estimated_lines: usize = self
            .narrative_history
            .iter()
            .map(|item| {
                // Count lines in content, estimate wrapping for each
                item.content
                    .lines()
                    .map(|line| (line.len() / ESTIMATED_WIDTH).max(1))
                    .sum::<usize>()
                    + 1 // blank line between entries
            })
            .sum();

        estimated_lines.saturating_sub(ESTIMATED_VISIBLE_HEIGHT)
    }

    /// Scroll narrative up (unlocks from bottom)
    pub fn scroll_up(&mut self, lines: usize) {
        // If scroll is at a huge "bottom" value, reset to estimated max first
        let max_scroll = self.estimate_max_scroll();
        if self.narrative_scroll > max_scroll {
            self.narrative_scroll = max_scroll;
        }
        self.narrative_scroll = self.narrative_scroll.saturating_sub(lines);
        // User manually scrolled up, unlock from bottom
        self.scroll_locked_to_bottom = false;
    }

    /// Scroll narrative down
    pub fn scroll_down(&mut self, lines: usize) {
        self.narrative_scroll = self.narrative_scroll.saturating_add(lines);
        // Cap to reasonable max to prevent overflow issues
        let max_scroll = self.estimate_max_scroll();
        self.narrative_scroll = self.narrative_scroll.min(max_scroll + 100);
        // Note: we don't re-lock to bottom here - user must press G to re-lock
    }

    /// Submit current input
    pub fn submit_input(&mut self) -> Option<String> {
        if self.input_buffer.is_empty() {
            return None;
        }

        let input = std::mem::take(&mut self.input_buffer);
        self.cursor_position = 0;

        // Add to history (if not a command)
        if !input.starts_with(':') {
            self.input_history.push_front(input.clone());
            if self.input_history.len() > 100 {
                self.input_history.pop_back();
            }
        }
        self.history_index = None;
        self.saved_input = None; // Clear any saved input

        Some(input)
    }

    /// Handle a typed character (unicode-safe)
    pub fn type_char(&mut self, c: char) {
        // Convert cursor position (character index) to byte index
        let byte_pos = self
            .input_buffer
            .char_indices()
            .nth(self.cursor_position)
            .map(|(i, _)| i)
            .unwrap_or(self.input_buffer.len());
        self.input_buffer.insert(byte_pos, c);
        self.cursor_position += 1;
    }

    /// Handle backspace (unicode-safe)
    pub fn backspace(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            // Get byte range of the character at cursor position
            if let Some((byte_pos, ch)) = self.input_buffer.char_indices().nth(self.cursor_position)
            {
                self.input_buffer
                    .replace_range(byte_pos..byte_pos + ch.len_utf8(), "");
            }
        }
    }

    /// Handle delete (unicode-safe)
    pub fn delete(&mut self) {
        let char_count = self.input_buffer.chars().count();
        if self.cursor_position < char_count {
            if let Some((byte_pos, ch)) = self.input_buffer.char_indices().nth(self.cursor_position)
            {
                self.input_buffer
                    .replace_range(byte_pos..byte_pos + ch.len_utf8(), "");
            }
        }
    }

    /// Move cursor left
    pub fn cursor_left(&mut self) {
        self.cursor_position = self.cursor_position.saturating_sub(1);
    }

    /// Move cursor right
    pub fn cursor_right(&mut self) {
        let char_count = self.input_buffer.chars().count();
        self.cursor_position = (self.cursor_position + 1).min(char_count);
    }

    /// Move cursor to start
    pub fn cursor_home(&mut self) {
        self.cursor_position = 0;
    }

    /// Move cursor to end (unicode-safe)
    pub fn cursor_end(&mut self) {
        self.cursor_position = self.input_buffer.chars().count();
    }

    /// Navigate to previous input in history
    pub fn history_prev(&mut self) {
        if self.input_history.is_empty() {
            return;
        }

        // Save current input if we're just starting to browse history
        if self.history_index.is_none() && !self.input_buffer.is_empty() {
            self.saved_input = Some(self.input_buffer.clone());
        }

        let new_index = match self.history_index {
            None => Some(0),
            Some(i) if i + 1 < self.input_history.len() => Some(i + 1),
            Some(i) => Some(i), // Already at oldest
        };

        if let Some(idx) = new_index {
            if let Some(entry) = self.input_history.get(idx) {
                self.input_buffer = entry.clone();
                self.cursor_position = self.input_buffer.chars().count();
                self.history_index = new_index;
            }
        }
    }

    /// Navigate to next input in history
    pub fn history_next(&mut self) {
        match self.history_index {
            None => {
                // Already at "new input", nothing to do
            }
            Some(0) => {
                // Return to saved input or empty
                self.input_buffer = self.saved_input.take().unwrap_or_default();
                self.cursor_position = self.input_buffer.chars().count();
                self.history_index = None;
            }
            Some(i) => {
                if let Some(entry) = self.input_history.get(i - 1) {
                    self.input_buffer = entry.clone();
                    self.cursor_position = self.input_buffer.chars().count();
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

    /// Toggle inventory overlay
    pub fn toggle_inventory(&mut self) {
        if matches!(self.overlay, Some(Overlay::Inventory)) {
            self.overlay = None;
        } else {
            self.overlay = Some(Overlay::Inventory);
        }
    }

    /// Toggle character sheet overlay
    pub fn toggle_character_sheet(&mut self) {
        if matches!(self.overlay, Some(Overlay::CharacterSheet)) {
            self.overlay = None;
        } else {
            self.overlay = Some(Overlay::CharacterSheet);
        }
    }

    /// Toggle quest log overlay
    pub fn toggle_quest_log(&mut self) {
        if matches!(self.overlay, Some(Overlay::QuestLog)) {
            self.overlay = None;
        } else {
            self.overlay = Some(Overlay::QuestLog);
        }
    }

    /// Close any open overlay
    pub fn close_overlay(&mut self) {
        self.overlay = None;
    }

    /// Show a dice roll with animation
    pub fn show_dice_roll(&mut self, expression: &str, purpose: &str, dc: Option<i32>) {
        // Validate the expression parses
        if DiceExpression::parse(expression).is_ok() {
            // Start rolling animation - don't calculate result yet
            self.rolling_dice = Some(RollingDice {
                expression: expression.to_string(),
                purpose: purpose.to_string(),
                dc,
                frames_elapsed: 0,
                result: None,
            });

            // Show overlay with no result (triggers Rolling animation state)
            self.set_overlay(Overlay::DiceRoll {
                result: None,
                purpose: purpose.to_string(),
                dc,
            });
        }
    }

    /// Process a colon command
    /// Returns (handled, needs_worker_request)
    pub fn process_command(&mut self, command: &str) -> (bool, Option<WorkerRequest>) {
        let cmd = command.trim_start_matches(':');
        let parts: Vec<&str> = cmd.split_whitespace().collect();

        if parts.is_empty() {
            return (false, None);
        }

        match parts[0] {
            "q" | "quit" | "exit" => {
                self.should_quit = true;
                (true, None)
            }
            "w" | "save" => {
                let filename = if parts.len() > 1 {
                    parts[1].to_string()
                } else {
                    "campaign.json".to_string()
                };
                self.set_status("Saving...");
                (true, Some(WorkerRequest::Save(PathBuf::from(filename))))
            }
            "load" => {
                let filename = if parts.len() > 1 {
                    parts[1].to_string()
                } else {
                    "campaign.json".to_string()
                };
                self.set_status("Loading...");
                (true, Some(WorkerRequest::Load(PathBuf::from(filename))))
            }
            "wq" => {
                self.set_status("Saving and quitting...");
                self.quit_after_save = true;
                (
                    true,
                    Some(WorkerRequest::Save(PathBuf::from("campaign.json"))),
                )
            }
            "help" | "h" => {
                self.toggle_help();
                (true, None)
            }
            "roll" | "r" => {
                if parts.len() > 1 {
                    let expression = parts[1..].join("");
                    self.show_dice_roll(&expression, "Manual Roll", None);
                } else {
                    self.set_status("Usage: :roll XdY+Z");
                }
                (true, None)
            }
            "rest" => {
                // Rest commands can't be done directly anymore since we don't own session
                // We'd need to send a request to the worker
                // For now, just show a message that this should be done via player action
                self.set_status("Use 'I take a short rest' or 'I take a long rest' in-game");
                (true, None)
            }
            _ => {
                self.set_status(format!("Unknown command: {}", parts[0]));
                (false, None)
            }
        }
    }

    /// Send a player action to the AI worker
    pub fn send_player_action(&mut self, input: String) {
        if input.trim().is_empty() {
            return;
        }

        self.ai_processing = true;
        self.set_status("Processing...");

        // Try to send the request (non-blocking)
        if self.request_tx.try_send(WorkerRequest::PlayerAction(input)).is_err() {
            self.set_status("Worker busy, please wait...");
            self.ai_processing = false;
        }
    }

    /// Send a cancel request to the AI worker
    pub fn cancel_processing(&mut self) {
        if self.ai_processing {
            let _ = self.request_tx.try_send(WorkerRequest::Cancel);
            self.set_status("Cancelling...");
        }
    }

    /// Update world state from a WorldUpdate
    pub fn apply_world_update(&mut self, update: WorldUpdate) {
        self.world = update;
    }

    /// Tick for animations
    pub fn tick(&mut self) {
        self.animation_frame = self.animation_frame.wrapping_add(1);

        // Update rolling dice animation
        if let Some(ref mut rolling) = self.rolling_dice {
            rolling.frames_elapsed += 1;

            // After ~8 frames (~0.8 sec at 100ms poll), calculate result
            if rolling.frames_elapsed >= 8 && rolling.result.is_none() {
                if let Ok(expr) = DiceExpression::parse(&rolling.expression) {
                    rolling.result = Some(expr.roll());
                }
            }

            // After ~10 frames, reveal result
            if rolling.frames_elapsed >= 10 {
                if let Some(result) = rolling.result.take() {
                    // Update overlay with result
                    self.overlay = Some(Overlay::DiceRoll {
                        result: Some(result.clone()),
                        purpose: rolling.purpose.clone(),
                        dc: rolling.dc,
                    });

                    // Add to narrative
                    let result_text = if let Some(dc_val) = rolling.dc {
                        let outcome = if result.total >= dc_val {
                            "SUCCESS"
                        } else {
                            "FAILURE"
                        };
                        format!(
                            "{}: {} = {} vs DC {} - {}",
                            rolling.purpose, rolling.expression, result.total, dc_val, outcome
                        )
                    } else {
                        format!(
                            "{}: {} = {}",
                            rolling.purpose, rolling.expression, result.total
                        )
                    };
                    self.add_narrative(result_text, NarrativeType::System);

                    self.rolling_dice = None;
                }
            } else {
                // Update overlay with current animation frame
                self.overlay = Some(Overlay::DiceRoll {
                    result: None,
                    purpose: rolling.purpose.clone(),
                    dc: rolling.dc,
                });
            }
        }
    }

    /// Set status message (always overwrites)
    pub fn set_status(&mut self, message: impl Into<String>) {
        self.status_message = Some(message.into());
    }

    /// Set status message only if no message is currently shown
    /// Use this for lower-priority messages that shouldn't overwrite critical ones
    pub fn set_status_if_empty(&mut self, message: impl Into<String>) {
        if self.status_message.is_none() {
            self.status_message = Some(message.into());
        }
    }

    /// Clear status message
    pub fn clear_status(&mut self) {
        self.status_message = None;
    }

    // =========================================================================
    // Getters for private fields
    // =========================================================================

    /// Get the current overlay
    pub fn overlay(&self) -> Option<&Overlay> {
        self.overlay.as_ref()
    }

    /// Get the current status message
    pub fn status_message(&self) -> Option<&str> {
        self.status_message.as_deref()
    }

    /// Get the current input buffer
    pub fn input_buffer(&self) -> &str {
        &self.input_buffer
    }

    /// Get the current cursor position
    pub fn cursor_position(&self) -> usize {
        self.cursor_position
    }

    // =========================================================================
    // Setters for private fields
    // =========================================================================

    /// Set the overlay
    pub fn set_overlay(&mut self, overlay: Overlay) {
        self.overlay = Some(overlay);
    }

    /// Check if an overlay is currently open
    pub fn has_overlay(&self) -> bool {
        self.overlay.is_some()
    }

    /// Cycle to next focused panel
    pub fn cycle_focus(&mut self) {
        self.focused_panel = match self.focused_panel {
            FocusedPanel::Narrative => FocusedPanel::Character,
            FocusedPanel::Character => FocusedPanel::Combat,
            FocusedPanel::Combat => FocusedPanel::Narrative,
        };
    }

    /// Cycle to previous focused panel
    pub fn cycle_focus_reverse(&mut self) {
        self.focused_panel = match self.focused_panel {
            FocusedPanel::Narrative => FocusedPanel::Combat,
            FocusedPanel::Combat => FocusedPanel::Character,
            FocusedPanel::Character => FocusedPanel::Narrative,
        };
    }

    /// Set input buffer content and move cursor to end (unicode-safe)
    pub fn set_input(&mut self, content: impl Into<String>) {
        self.input_buffer = content.into();
        self.cursor_position = self.input_buffer.chars().count();
    }

    /// Clear the input buffer
    pub fn clear_input(&mut self) {
        self.input_buffer.clear();
        self.cursor_position = 0;
    }
}
