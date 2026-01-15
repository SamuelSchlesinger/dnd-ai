//! Main render function for the D&D TUI

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
    Frame,
};

use super::layout::{centered_rect, AppLayout, CombatLayout};
use super::theme::GameTheme;
use super::widgets::*;
use crate::dnd::app::AppState;
use crate::dnd::game::state::GameMode;

/// Render the main application
pub fn render(frame: &mut Frame, state: &AppState) {
    let area = frame.area();

    match state.game.mode {
        GameMode::Combat => render_combat_mode(frame, area, state),
        _ => render_exploration_mode(frame, area, state),
    }

    // Render overlay if present
    if let Some(ref overlay) = state.overlay {
        render_overlay(frame, area, state, overlay);
    }
}

/// Render exploration mode layout
fn render_exploration_mode(frame: &mut Frame, area: Rect, state: &AppState) {
    let layout = AppLayout::calculate(area);
    let theme = &state.theme;

    // Title bar
    render_title_bar(frame, layout.title_area, state);

    // Narrative area
    let narrative_items: Vec<narrative::NarrativeItem> = state
        .narrative_history
        .iter()
        .map(|entry| narrative::NarrativeItem {
            content: entry.content.clone(),
            entry_type: entry.entry_type,
        })
        .collect();

    let narrative = NarrativeWidget::new(&narrative_items, theme)
        .scroll(state.narrative_scroll)
        .focused(state.focused_panel == FocusedPanel::Narrative)
        .streaming(state.streaming_text.as_deref());
    frame.render_widget(narrative, layout.narrative_area);

    // Character panel
    let char_panel = CharacterPanelWidget::new(&state.game.player_character, theme)
        .focused(state.focused_panel == FocusedPanel::Sidebar);
    frame.render_widget(char_panel, layout.sidebar_area);

    // Status bar
    let status = StatusBarWidget::new(&state.game.player_character, state.game.mode, state.input_mode, theme)
        .message(state.status_message.as_deref());
    frame.render_widget(status, layout.status_bar);

    // Hotkey bar
    let hotkeys = status_bar::HotkeyBarWidget::new(state.game.mode, state.input_mode, theme);
    frame.render_widget(hotkeys, layout.hotkey_bar);

    // Input field
    let input = InputWidget::new(&state.input_buffer, theme)
        .cursor_position(state.cursor_position);
    frame.render_widget(input, layout.input_area);
}

/// Render combat mode layout
fn render_combat_mode(frame: &mut Frame, area: Rect, state: &AppState) {
    let layout = AppLayout::calculate_combat(area);
    let theme = &state.theme;

    // Title bar
    render_title_bar(frame, layout.title_area, state);

    // Narrative area (combat log)
    let narrative_items: Vec<narrative::NarrativeItem> = state
        .narrative_history
        .iter()
        .map(|entry| narrative::NarrativeItem {
            content: entry.content.clone(),
            entry_type: entry.entry_type,
        })
        .collect();

    let narrative = NarrativeWidget::new(&narrative_items, theme)
        .scroll(state.narrative_scroll)
        .focused(state.focused_panel == FocusedPanel::Narrative)
        .streaming(state.streaming_text.as_deref());
    frame.render_widget(narrative, layout.narrative_area);

    // Initiative tracker
    if let Some(ref combat) = state.game.combat {
        let tracker = CombatTrackerWidget::new(combat, theme);
        frame.render_widget(tracker, layout.initiative_area);

        // Enemy HP display
        let enemies = build_enemy_status(state);
        let enemy_widget = combat_tracker::EnemyHpWidget::new(enemies, theme);
        frame.render_widget(enemy_widget, layout.enemy_hp_area);
    }

    // Status bar (with combat actions remaining)
    let status = StatusBarWidget::new(&state.game.player_character, state.game.mode, state.input_mode, theme)
        .message(state.status_message.as_deref());
    frame.render_widget(status, layout.status_bar);

    // Hotkey bar (combat actions)
    let hotkeys = status_bar::HotkeyBarWidget::new(state.game.mode, state.input_mode, theme);
    frame.render_widget(hotkeys, layout.hotkey_bar);

    // Input field
    let input = InputWidget::new(&state.input_buffer, theme)
        .cursor_position(state.cursor_position);
    frame.render_widget(input, layout.input_area);
}

/// Render title bar
fn render_title_bar(frame: &mut Frame, area: Rect, state: &AppState) {
    let location = &state.game.current_location.name;
    let time = state.game.game_time.time_of_day();

    let title = Line::from(vec![
        Span::styled(
            format!(" {} ", location),
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw("─"),
        Span::styled(
            format!(" {} ", time),
            Style::default().add_modifier(Modifier::DIM),
        ),
    ]);

    let paragraph = Paragraph::new(title);
    frame.render_widget(paragraph, area);
}

/// Render overlay
fn render_overlay(frame: &mut Frame, area: Rect, state: &AppState, overlay: &Overlay) {
    match overlay {
        Overlay::Help => render_help_overlay(frame, area, state),
        Overlay::DiceRoll { result, purpose, dc } => {
            render_dice_overlay(frame, area, state, result, purpose, *dc)
        }
        _ => {}
    }
}

/// Render help overlay
fn render_help_overlay(frame: &mut Frame, area: Rect, state: &AppState) {
    let popup_area = centered_rect(60, 70, area);

    // Clear the background
    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(" Help - Press Esc to close ")
        .borders(Borders::ALL)
        .border_style(state.theme.border_style(true));

    let help_text = vec![
        Line::from(Span::styled(
            "Global Keys",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  ? / F1    - Show this help"),
        Line::from("  Esc       - Close overlay / Cancel"),
        Line::from("  Ctrl+C    - Quit game"),
        Line::from("  Up/Down   - Scroll narrative"),
        Line::from(""),
        Line::from(Span::styled(
            "Exploration Mode",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  I         - Open inventory"),
        Line::from("  C         - View character sheet"),
        Line::from("  Q         - View quest log"),
        Line::from("  J         - Open journal"),
        Line::from("  R         - Request rest"),
        Line::from(""),
        Line::from(Span::styled(
            "Combat Mode",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  A         - Attack"),
        Line::from("  C         - Cast spell"),
        Line::from("  D         - Dash"),
        Line::from("  U         - Use item"),
        Line::from("  E         - End turn"),
        Line::from(""),
        Line::from(Span::styled(
            "Commands",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  /roll XdY - Roll dice (e.g., /roll 1d20+5)"),
        Line::from("  /rest     - Request rest"),
        Line::from("  /save     - Save game"),
        Line::from("  /quit     - Quit game"),
    ];

    let paragraph = Paragraph::new(help_text).block(block);
    frame.render_widget(paragraph, popup_area);
}

/// Render dice roll overlay
fn render_dice_overlay(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    result: &Option<crate::dnd::game::dice::RollResult>,
    purpose: &str,
    dc: Option<i32>,
) {
    let popup_area = centered_rect(40, 50, area);

    frame.render_widget(Clear, popup_area);

    let mut widget = DiceRollWidget::new(&state.theme).purpose(purpose).dc(dc);

    if let Some(r) = result {
        widget = widget
            .result(r)
            .animation_state(dice_roll::DiceAnimationState::Complete);
    } else {
        widget = widget.animation_state(dice_roll::DiceAnimationState::Rolling {
            frame: state.animation_frame,
        });
    }

    frame.render_widget(widget, popup_area);
}

/// Build enemy status list for display
fn build_enemy_status(state: &AppState) -> Vec<combat_tracker::EnemyStatus> {
    // In a real implementation, this would track enemy HP estimates
    // For now, return placeholder data
    if let Some(ref combat) = state.game.combat {
        combat
            .initiative_order
            .iter()
            .filter(|e| !e.is_player)
            .map(|e| combat_tracker::EnemyStatus {
                name: e.name.clone(),
                hp_estimate: combat_tracker::HpEstimate::Unknown,
                conditions: Vec::new(),
            })
            .collect()
    } else {
        Vec::new()
    }
}

/// UI overlay types
#[derive(Debug, Clone)]
pub enum Overlay {
    Help,
    CharacterSheet,
    Inventory,
    QuestLog,
    DiceRoll {
        result: Option<crate::dnd::game::dice::RollResult>,
        purpose: String,
        dc: Option<i32>,
    },
}

/// Panel that can be focused
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FocusedPanel {
    #[default]
    Narrative,
    Sidebar,
    Input,
}
