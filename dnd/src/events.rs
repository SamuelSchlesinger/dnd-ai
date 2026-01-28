//! Event handling for the D&D TUI

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};

use dnd_core::world::{GameMode, NarrativeType};

use crate::app::{App, InputMode};
use crate::ui::Overlay;

/// Result of handling an event
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventResult {
    Continue,
    Quit,
    NeedsRedraw,
    ProcessInput(bool), // bool indicates whether to await processing
}

/// Handle a terminal event
pub fn handle_event(app: &mut App, event: Event) -> EventResult {
    match event {
        Event::Key(key) => handle_key_event(app, key),
        Event::Mouse(mouse) => handle_mouse_event(app, mouse),
        Event::Resize(_, _) => EventResult::NeedsRedraw,
        _ => EventResult::Continue,
    }
}

/// Handle a mouse event
fn handle_mouse_event(app: &mut App, mouse: MouseEvent) -> EventResult {
    match mouse.kind {
        MouseEventKind::ScrollUp => {
            app.scroll_up(3);
            EventResult::NeedsRedraw
        }
        MouseEventKind::ScrollDown => {
            app.scroll_down(3);
            EventResult::NeedsRedraw
        }
        _ => EventResult::Continue,
    }
}

/// Handle a key event
fn handle_key_event(app: &mut App, key: KeyEvent) -> EventResult {
    // Handle overlay keys first
    if app.has_overlay() {
        return handle_overlay_key(app, key);
    }

    // Global shortcuts (always work)
    if let (KeyCode::Char('c'), KeyModifiers::CONTROL) = (key.code, key.modifiers) {
        return EventResult::Quit;
    }

    // Route based on input mode
    match app.input_mode {
        InputMode::Normal => handle_normal_mode(app, key),
        InputMode::Insert => handle_insert_mode(app, key),
        InputMode::Command => handle_command_mode(app, key),
    }
}

/// Handle keys in NORMAL mode (vim-style navigation and hotkeys)
fn handle_normal_mode(app: &mut App, key: KeyEvent) -> EventResult {
    match key.code {
        // Mode switching
        KeyCode::Char('i') => {
            app.input_mode = InputMode::Insert;
            EventResult::NeedsRedraw
        }
        KeyCode::Char('a') => {
            // Append mode - go to insert at end
            app.input_mode = InputMode::Insert;
            app.cursor_end();
            EventResult::NeedsRedraw
        }
        KeyCode::Char(':') => {
            app.enter_command_mode();
            EventResult::NeedsRedraw
        }

        // Help
        KeyCode::Char('?') | KeyCode::F(1) => {
            app.toggle_help();
            EventResult::NeedsRedraw
        }

        // Quit
        KeyCode::Char('q') => EventResult::Quit,

        // Navigation
        KeyCode::Char('j') | KeyCode::Down => {
            app.scroll_down(1);
            EventResult::NeedsRedraw
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.scroll_up(1);
            EventResult::NeedsRedraw
        }
        KeyCode::Char('G') => {
            app.scroll_to_bottom();
            EventResult::NeedsRedraw
        }
        KeyCode::Char('g') => {
            // gg to go to top (simplified: just g goes to top)
            app.narrative_scroll = 0;
            app.scroll_locked_to_bottom = false; // Unlock from auto-scroll
            EventResult::NeedsRedraw
        }
        KeyCode::PageUp | KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.scroll_up(10);
            EventResult::NeedsRedraw
        }
        KeyCode::PageDown | KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.scroll_down(10);
            EventResult::NeedsRedraw
        }

        // Panel focus cycling
        KeyCode::Tab => {
            app.cycle_focus();
            EventResult::NeedsRedraw
        }
        KeyCode::BackTab => {
            app.cycle_focus_reverse();
            EventResult::NeedsRedraw
        }

        // Game mode specific hotkeys
        _ => handle_game_mode_hotkeys(app, key),
    }
}

/// Handle game-mode specific hotkeys (in normal mode)
fn handle_game_mode_hotkeys(app: &mut App, key: KeyEvent) -> EventResult {
    match app.game_mode() {
        GameMode::Combat => handle_combat_hotkeys(app, key),
        GameMode::Dialogue => handle_dialogue_hotkeys(app, key),
        _ => handle_exploration_hotkeys(app, key),
    }
}

/// Handle exploration hotkeys (normal mode)
fn handle_exploration_hotkeys(app: &mut App, key: KeyEvent) -> EventResult {
    match key.code {
        KeyCode::Char('I') => {
            app.set_status("Inventory not yet implemented");
            EventResult::NeedsRedraw
        }
        KeyCode::Char('C') => {
            app.set_status("Character sheet not yet implemented");
            EventResult::NeedsRedraw
        }
        KeyCode::Char('Q') => {
            app.set_status("Quest log not yet implemented");
            EventResult::NeedsRedraw
        }
        KeyCode::Char('J') => {
            app.set_status("Journal not yet implemented");
            EventResult::NeedsRedraw
        }
        KeyCode::Char('r') => {
            app.short_rest();
            EventResult::NeedsRedraw
        }
        KeyCode::Char('R') => {
            app.long_rest();
            EventResult::NeedsRedraw
        }
        _ => EventResult::Continue,
    }
}

/// Handle combat hotkeys (normal mode)
fn handle_combat_hotkeys(app: &mut App, key: KeyEvent) -> EventResult {
    match key.code {
        // Attack - prefill input with attack command
        KeyCode::Char('A') => {
            app.set_input("I attack ");
            app.input_mode = InputMode::Insert;
            app.set_status("Type target name, then press Enter");
            EventResult::NeedsRedraw
        }
        // Cast spell - prefill input with cast command
        KeyCode::Char('c') => {
            app.set_input("I cast ");
            app.input_mode = InputMode::Insert;
            app.set_status("Type spell name and target, then press Enter");
            EventResult::NeedsRedraw
        }
        // Dash action
        KeyCode::Char('d') => {
            app.set_input("I take the Dash action");
            app.input_mode = InputMode::Insert;
            EventResult::NeedsRedraw
        }
        // Dodge action
        KeyCode::Char('D') => {
            app.set_input("I take the Dodge action");
            app.input_mode = InputMode::Insert;
            EventResult::NeedsRedraw
        }
        // Help action
        KeyCode::Char('H') => {
            app.set_input("I take the Help action to assist ");
            app.input_mode = InputMode::Insert;
            EventResult::NeedsRedraw
        }
        // End turn
        KeyCode::Char('e') => {
            if let Some(ref mut combat) = app.session.world_mut().combat {
                combat.next_turn();
                app.add_narrative("You end your turn.".to_string(), NarrativeType::Combat);
            }
            EventResult::NeedsRedraw
        }
        // Use item
        KeyCode::Char('u') => {
            app.set_input("I use ");
            app.input_mode = InputMode::Insert;
            app.set_status("Type item name, then press Enter");
            EventResult::NeedsRedraw
        }
        // Target selection (1-9 keys)
        KeyCode::Char(c @ '1'..='9') => {
            let target_idx = c.to_digit(10).unwrap() as usize;
            // Get combatant name if possible
            let combatant_name = app
                .session
                .world()
                .combat
                .as_ref()
                .and_then(|combat| combat.combatants.get(target_idx - 1))
                .map(|c| c.name.clone());

            if let Some(name) = combatant_name {
                app.set_input(format!("I attack {name}"));
                app.input_mode = InputMode::Insert;
                app.set_status(format!("Target: {name}"));
            } else {
                app.set_status(format!("No target at position {target_idx}"));
            }
            EventResult::NeedsRedraw
        }
        _ => EventResult::Continue,
    }
}

/// Handle dialogue hotkeys (normal mode)
fn handle_dialogue_hotkeys(app: &mut App, key: KeyEvent) -> EventResult {
    match key.code {
        KeyCode::Char(c @ '1'..='9') => {
            let option_idx = c.to_digit(10).unwrap() as usize;
            app.add_narrative(
                format!("You choose option {option_idx}."),
                NarrativeType::PlayerAction,
            );
            EventResult::NeedsRedraw
        }
        KeyCode::Esc => {
            app.session.world_mut().mode = GameMode::Exploration;
            app.add_narrative(
                "You end the conversation.".to_string(),
                NarrativeType::System,
            );
            EventResult::NeedsRedraw
        }
        _ => EventResult::Continue,
    }
}

/// Handle keys in INSERT mode (free text input)
fn handle_insert_mode(app: &mut App, key: KeyEvent) -> EventResult {
    match key.code {
        // Exit insert mode
        KeyCode::Esc => {
            app.input_mode = InputMode::Normal;
            EventResult::NeedsRedraw
        }

        // Submit input
        KeyCode::Enter => {
            if let Some(_input) = app.submit_input() {
                // Signal that we need to process input
                return EventResult::ProcessInput(true);
            }
            EventResult::NeedsRedraw
        }

        // Input editing
        KeyCode::Left => {
            app.cursor_left();
            EventResult::NeedsRedraw
        }
        KeyCode::Right => {
            app.cursor_right();
            EventResult::NeedsRedraw
        }
        KeyCode::Home => {
            app.cursor_home();
            EventResult::NeedsRedraw
        }
        KeyCode::End => {
            app.cursor_end();
            EventResult::NeedsRedraw
        }
        KeyCode::Backspace => {
            app.backspace();
            EventResult::NeedsRedraw
        }
        KeyCode::Delete => {
            app.delete();
            EventResult::NeedsRedraw
        }
        KeyCode::Up => {
            app.history_prev();
            EventResult::NeedsRedraw
        }
        KeyCode::Down => {
            app.history_next();
            EventResult::NeedsRedraw
        }

        // Character input
        KeyCode::Char(c) => {
            app.type_char(c);
            EventResult::NeedsRedraw
        }

        _ => EventResult::Continue,
    }
}

/// Handle keys in COMMAND mode (: commands)
fn handle_command_mode(app: &mut App, key: KeyEvent) -> EventResult {
    match key.code {
        // Exit command mode
        KeyCode::Esc => {
            app.input_mode = InputMode::Normal;
            app.clear_input();
            EventResult::NeedsRedraw
        }

        // Execute command
        KeyCode::Enter => {
            let command = app.input_buffer().to_string();
            app.clear_input();
            app.input_mode = InputMode::Normal;

            // Process the command
            if command.len() > 1 {
                app.process_command(&command);
            }

            if app.should_quit {
                EventResult::Quit
            } else {
                EventResult::NeedsRedraw
            }
        }

        // Input editing
        KeyCode::Left => {
            if app.cursor_position() > 1 {
                app.cursor_left();
            }
            EventResult::NeedsRedraw
        }
        KeyCode::Right => {
            app.cursor_right();
            EventResult::NeedsRedraw
        }
        KeyCode::Backspace => {
            if app.cursor_position() > 1 {
                app.backspace();
            } else {
                // Backspace on just ":" exits command mode
                app.input_mode = InputMode::Normal;
                app.clear_input();
            }
            EventResult::NeedsRedraw
        }

        // Character input
        KeyCode::Char(c) => {
            app.type_char(c);
            EventResult::NeedsRedraw
        }

        _ => EventResult::Continue,
    }
}

/// Handle key when overlay is open
fn handle_overlay_key(app: &mut App, key: KeyEvent) -> EventResult {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.close_overlay();
            EventResult::NeedsRedraw
        }
        KeyCode::Enter | KeyCode::Char(' ') => {
            // Close dice roll overlay on any key
            if matches!(app.overlay(), Some(Overlay::DiceRoll { .. })) {
                app.close_overlay();
            }
            EventResult::NeedsRedraw
        }
        _ => EventResult::Continue,
    }
}
