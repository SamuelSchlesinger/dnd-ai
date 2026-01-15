//! Event handling for the D&D TUI

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

use crate::dnd::app::{AppState, InputMode};
use crate::dnd::game::state::{GameMode, NarrativeType};
use crate::dnd::ui::render::Overlay;

/// Result of handling an event
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventResult {
    Continue,
    Quit,
    NeedsRedraw,
}

/// Handle a terminal event
pub fn handle_event(state: &mut AppState, event: Event) -> EventResult {
    match event {
        Event::Key(key) => handle_key_event(state, key),
        Event::Resize(_, _) => EventResult::NeedsRedraw,
        _ => EventResult::Continue,
    }
}

/// Handle a key event
fn handle_key_event(state: &mut AppState, key: KeyEvent) -> EventResult {
    // Handle overlay keys first
    if state.overlay.is_some() {
        return handle_overlay_key(state, key);
    }

    // Global shortcuts (always work)
    if let (KeyCode::Char('c'), KeyModifiers::CONTROL) = (key.code, key.modifiers) {
        return EventResult::Quit;
    }

    // Route based on input mode
    match state.input_mode {
        InputMode::Normal => handle_normal_mode(state, key),
        InputMode::Insert => handle_insert_mode(state, key),
        InputMode::Command => handle_command_mode(state, key),
    }
}

/// Handle keys in NORMAL mode (vim-style navigation and hotkeys)
fn handle_normal_mode(state: &mut AppState, key: KeyEvent) -> EventResult {
    match key.code {
        // Mode switching
        KeyCode::Char('i') => {
            state.input_mode = InputMode::Insert;
            EventResult::NeedsRedraw
        }
        KeyCode::Char('a') => {
            // Append mode - go to insert at end
            state.input_mode = InputMode::Insert;
            state.cursor_position = state.input_buffer.len();
            EventResult::NeedsRedraw
        }
        KeyCode::Char(':') => {
            state.enter_command_mode();
            EventResult::NeedsRedraw
        }

        // Help
        KeyCode::Char('?') | KeyCode::F(1) => {
            state.toggle_help();
            EventResult::NeedsRedraw
        }

        // Navigation
        KeyCode::Char('j') | KeyCode::Down => {
            state.scroll_down(1);
            EventResult::NeedsRedraw
        }
        KeyCode::Char('k') | KeyCode::Up => {
            state.scroll_up(1);
            EventResult::NeedsRedraw
        }
        KeyCode::Char('G') => {
            state.scroll_to_bottom();
            EventResult::NeedsRedraw
        }
        KeyCode::Char('g') => {
            // gg to go to top (simplified: just g goes to top)
            state.narrative_scroll = 0;
            EventResult::NeedsRedraw
        }
        KeyCode::PageUp | KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.scroll_up(10);
            EventResult::NeedsRedraw
        }
        KeyCode::PageDown | KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.scroll_down(10);
            EventResult::NeedsRedraw
        }

        // Game mode specific hotkeys
        _ => handle_game_mode_hotkeys(state, key),
    }
}

/// Handle game-mode specific hotkeys (in normal mode)
fn handle_game_mode_hotkeys(state: &mut AppState, key: KeyEvent) -> EventResult {
    match state.game.mode {
        GameMode::Combat => handle_combat_hotkeys(state, key),
        GameMode::Dialogue => handle_dialogue_hotkeys(state, key),
        _ => handle_exploration_hotkeys(state, key),
    }
}

/// Handle exploration hotkeys (normal mode)
fn handle_exploration_hotkeys(state: &mut AppState, key: KeyEvent) -> EventResult {
    match key.code {
        KeyCode::Char('I') => {
            state.set_status("Inventory not yet implemented");
            EventResult::NeedsRedraw
        }
        KeyCode::Char('C') => {
            state.set_status("Character sheet not yet implemented");
            EventResult::NeedsRedraw
        }
        KeyCode::Char('Q') => {
            state.set_status("Quest log not yet implemented");
            EventResult::NeedsRedraw
        }
        KeyCode::Char('J') => {
            state.set_status("Journal not yet implemented");
            EventResult::NeedsRedraw
        }
        KeyCode::Char('r') => {
            state.game.short_rest();
            state.add_narrative(
                "You take a short rest to catch your breath.".to_string(),
                NarrativeType::System,
            );
            EventResult::NeedsRedraw
        }
        KeyCode::Char('R') => {
            state.game.long_rest();
            state.add_narrative(
                "You take a long rest, recovering fully.".to_string(),
                NarrativeType::System,
            );
            EventResult::NeedsRedraw
        }
        _ => EventResult::Continue,
    }
}

/// Handle combat hotkeys (normal mode)
fn handle_combat_hotkeys(state: &mut AppState, key: KeyEvent) -> EventResult {
    match key.code {
        KeyCode::Char('A') => {
            state.set_status("Select target for attack");
            EventResult::NeedsRedraw
        }
        KeyCode::Char('c') => {
            state.set_status("Spellcasting not yet implemented");
            EventResult::NeedsRedraw
        }
        KeyCode::Char('d') => {
            state.add_narrative("You dash!".to_string(), NarrativeType::Combat);
            EventResult::NeedsRedraw
        }
        KeyCode::Char('e') => {
            if let Some(ref mut combat) = state.game.combat {
                combat.next_turn();
                state.add_narrative("You end your turn.".to_string(), NarrativeType::Combat);
            }
            EventResult::NeedsRedraw
        }
        KeyCode::Char(c @ '1'..='9') => {
            let target_idx = c.to_digit(10).unwrap() as usize;
            state.set_status(format!("Selected target {}", target_idx));
            EventResult::NeedsRedraw
        }
        _ => EventResult::Continue,
    }
}

/// Handle dialogue hotkeys (normal mode)
fn handle_dialogue_hotkeys(state: &mut AppState, key: KeyEvent) -> EventResult {
    match key.code {
        KeyCode::Char(c @ '1'..='9') => {
            let option_idx = c.to_digit(10).unwrap() as usize;
            state.add_narrative(
                format!("You choose option {}.", option_idx),
                NarrativeType::PlayerAction,
            );
            EventResult::NeedsRedraw
        }
        KeyCode::Esc => {
            state.game.end_dialogue();
            state.add_narrative(
                "You end the conversation.".to_string(),
                NarrativeType::System,
            );
            EventResult::NeedsRedraw
        }
        _ => EventResult::Continue,
    }
}

/// Handle keys in INSERT mode (free text input)
fn handle_insert_mode(state: &mut AppState, key: KeyEvent) -> EventResult {
    match key.code {
        // Exit insert mode
        KeyCode::Esc => {
            state.input_mode = InputMode::Normal;
            EventResult::NeedsRedraw
        }

        // Submit input
        KeyCode::Enter => {
            if let Some(input) = state.submit_input() {
                process_player_input(state, &input);
            }
            // Stay in insert mode after submitting
            EventResult::NeedsRedraw
        }

        // Input editing
        KeyCode::Left => {
            state.cursor_left();
            EventResult::NeedsRedraw
        }
        KeyCode::Right => {
            state.cursor_right();
            EventResult::NeedsRedraw
        }
        KeyCode::Home => {
            state.cursor_home();
            EventResult::NeedsRedraw
        }
        KeyCode::End => {
            state.cursor_end();
            EventResult::NeedsRedraw
        }
        KeyCode::Backspace => {
            state.backspace();
            EventResult::NeedsRedraw
        }
        KeyCode::Delete => {
            state.delete();
            EventResult::NeedsRedraw
        }
        KeyCode::Up => {
            state.history_prev();
            EventResult::NeedsRedraw
        }
        KeyCode::Down => {
            state.history_next();
            EventResult::NeedsRedraw
        }

        // Character input
        KeyCode::Char(c) => {
            state.type_char(c);
            EventResult::NeedsRedraw
        }

        _ => EventResult::Continue,
    }
}

/// Handle keys in COMMAND mode (: commands)
fn handle_command_mode(state: &mut AppState, key: KeyEvent) -> EventResult {
    match key.code {
        // Exit command mode
        KeyCode::Esc => {
            state.input_mode = InputMode::Normal;
            state.input_buffer.clear();
            state.cursor_position = 0;
            EventResult::NeedsRedraw
        }

        // Execute command
        KeyCode::Enter => {
            let command = state.input_buffer.clone();
            state.input_buffer.clear();
            state.cursor_position = 0;
            state.input_mode = InputMode::Normal;

            // Process the command (strip the leading :)
            if command.len() > 1 {
                process_colon_command(state, &command[1..]);
            }
            EventResult::NeedsRedraw
        }

        // Input editing
        KeyCode::Left => {
            if state.cursor_position > 1 {
                state.cursor_left();
            }
            EventResult::NeedsRedraw
        }
        KeyCode::Right => {
            state.cursor_right();
            EventResult::NeedsRedraw
        }
        KeyCode::Backspace => {
            if state.cursor_position > 1 {
                state.backspace();
            } else {
                // Backspace on just ":" exits command mode
                state.input_mode = InputMode::Normal;
                state.input_buffer.clear();
                state.cursor_position = 0;
            }
            EventResult::NeedsRedraw
        }

        // Character input
        KeyCode::Char(c) => {
            state.type_char(c);
            EventResult::NeedsRedraw
        }

        _ => EventResult::Continue,
    }
}

/// Process : commands (like :quit, :roll, etc.)
fn process_colon_command(state: &mut AppState, command: &str) {
    let parts: Vec<&str> = command.trim().split_whitespace().collect();
    if parts.is_empty() {
        return;
    }

    match parts[0] {
        "q" | "quit" | "exit" => {
            state.should_quit = true;
        }
        "w" | "save" => {
            state.set_status("Save not yet implemented");
        }
        "wq" => {
            state.set_status("Saving and quitting...");
            state.should_quit = true;
        }
        "help" | "h" => {
            state.toggle_help();
        }
        "roll" | "r" => {
            if parts.len() > 1 {
                let expression = parts[1..].join("");
                state.show_dice_roll(&expression, "Manual Roll", None);
            } else {
                state.set_status("Usage: :roll XdY+Z");
            }
        }
        "rest" => {
            if parts.len() > 1 && parts[1] == "long" {
                state.game.long_rest();
                state.add_narrative(
                    "You take a long rest, recovering fully.".to_string(),
                    NarrativeType::System,
                );
            } else {
                state.game.short_rest();
                state.add_narrative(
                    "You take a short rest.".to_string(),
                    NarrativeType::System,
                );
            }
        }
        _ => {
            state.set_status(format!("Unknown command: {}", parts[0]));
        }
    }
}

/// Handle key when overlay is open
fn handle_overlay_key(state: &mut AppState, key: KeyEvent) -> EventResult {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            state.close_overlay();
            EventResult::NeedsRedraw
        }
        KeyCode::Enter | KeyCode::Char(' ') => {
            // Close dice roll overlay on any key
            if matches!(state.overlay, Some(Overlay::DiceRoll { .. })) {
                state.close_overlay();
            }
            EventResult::NeedsRedraw
        }
        _ => EventResult::Continue,
    }
}

/// Process player input text (from insert mode)
fn process_player_input(state: &mut AppState, input: &str) {
    let input = input.trim();
    if input.is_empty() {
        return;
    }

    // Add player action to narrative
    state.add_narrative(format!("> {}", input), NarrativeType::PlayerAction);

    // In a full implementation, this would send to the AI DM
    // For now, just acknowledge
    state.add_narrative(
        "The DM considers your action...".to_string(),
        NarrativeType::System,
    );

    // Demo: simple keyword responses
    let lower = input.to_lowercase();
    if lower.contains("attack") || lower.contains("hit") || lower.contains("strike") {
        state.show_dice_roll("1d20+5", "Attack Roll", Some(15));
    } else if lower.contains("look") || lower.contains("examine") || lower.contains("search") {
        state.show_dice_roll("1d20+3", "Perception Check", Some(12));
    } else if lower.contains("sneak") || lower.contains("hide") || lower.contains("stealth") {
        state.show_dice_roll("1d20+2", "Stealth Check", Some(10));
    } else {
        state.add_narrative(
            "You proceed with your action.".to_string(),
            NarrativeType::DmNarration,
        );
    }
}
