//! Render orchestration for the D&D TUI

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use dnd_core::dice::RollResult;
use dnd_core::world::GameMode;

use crate::app::{App, InputMode};
use crate::ui::layout::{centered_rect_fixed, AppLayout, CombatLayout};
use crate::ui::widgets::{
    CharacterPanelWidget, CombatTrackerWidget, DiceAnimationState, DiceRollWidget,
    EnemyHpWidget, EnemyStatus, HpEstimate, HotkeyBarWidget, InputWidget, NarrativeWidget,
    StatusBarWidget,
};

/// Which panel is focused
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FocusedPanel {
    #[default]
    Narrative,
    Character,
    Combat,
}

/// Overlay types
#[derive(Debug, Clone)]
pub enum Overlay {
    Help,
    DiceRoll {
        result: Option<RollResult>,
        purpose: String,
        dc: Option<i32>,
    },
}

/// Main render function
pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Choose layout based on game mode
    match app.session.world().mode {
        GameMode::Combat => render_combat_layout(frame, app, area),
        _ => render_exploration_layout(frame, app, area),
    }

    // Render overlay if present
    if let Some(overlay) = app.overlay() {
        render_overlay(frame, app, overlay, area);
    }
}

/// Render exploration layout (70/30 split)
fn render_exploration_layout(frame: &mut Frame, app: &App, area: Rect) {
    let layout = AppLayout::calculate(area);

    // Title bar
    render_title_bar(frame, app, layout.title_area);

    // Narrative panel
    let narrative_widget = NarrativeWidget::new(&app.narrative_history, &app.theme)
        .scroll(app.narrative_scroll)
        .focused(matches!(app.focused_panel, FocusedPanel::Narrative))
        .streaming(app.streaming_text.as_deref());
    frame.render_widget(narrative_widget, layout.narrative_area);

    // Character panel
    let character_widget =
        CharacterPanelWidget::new(&app.session.world().player_character, &app.theme)
            .focused(matches!(app.focused_panel, FocusedPanel::Character));
    frame.render_widget(character_widget, layout.sidebar_area);

    // Status bar
    render_status_bar(frame, app, layout.status_bar);

    // Hotkey bar
    render_hotkey_bar(frame, app, layout.hotkey_bar);

    // Input area
    render_input(frame, app, layout.input_area);
}

/// Render combat layout (65/35 split)
fn render_combat_layout(frame: &mut Frame, app: &App, area: Rect) {
    let layout = CombatLayout::calculate(area);

    // Title bar (combat mode)
    render_combat_title(frame, app, layout.title_area);

    // Narrative/combat log
    let narrative_widget = NarrativeWidget::new(&app.narrative_history, &app.theme)
        .scroll(app.narrative_scroll)
        .focused(matches!(app.focused_panel, FocusedPanel::Narrative))
        .streaming(app.streaming_text.as_deref());
    frame.render_widget(narrative_widget, layout.narrative_area);

    // Initiative tracker
    if let Some(ref combat) = app.session.world().combat {
        let combat_widget = CombatTrackerWidget::new(combat, &app.theme)
            .focused(matches!(app.focused_panel, FocusedPanel::Combat));
        frame.render_widget(combat_widget, layout.initiative_area);

        // Build enemy status list from combatants
        let enemies: Vec<EnemyStatus> = combat
            .get_enemies()
            .iter()
            .map(|c| {
                let hp_ratio = if c.max_hp > 0 {
                    c.current_hp as f64 / c.max_hp as f64
                } else {
                    1.0
                };
                let hp_estimate = if c.current_hp <= 0 {
                    HpEstimate::Dead
                } else if hp_ratio <= 0.25 {
                    HpEstimate::Critical
                } else if hp_ratio <= 0.5 {
                    HpEstimate::Bloodied
                } else {
                    HpEstimate::Healthy
                };
                EnemyStatus {
                    name: c.name.clone(),
                    hp_estimate,
                    conditions: vec![],
                }
            })
            .collect();

        let enemy_widget = EnemyHpWidget::new(enemies, &app.theme);
        frame.render_widget(enemy_widget, layout.enemy_hp_area);
    }

    // Status bar (with combat info)
    render_combat_status(frame, app, layout.status_bar);

    // Hotkey bar
    render_hotkey_bar(frame, app, layout.hotkey_bar);

    // Input area
    render_input(frame, app, layout.input_area);
}

/// Render the title bar
fn render_title_bar(frame: &mut Frame, app: &App, area: Rect) {
    let world = app.session.world();
    let time = &world.game_time;
    let title = format!(
        " {} | {} | {}:{:02} {} ",
        world.current_location.name,
        time.time_of_day(),
        time.hour,
        time.minute,
        if time.is_daytime() { "☀" } else { "☽" }
    );

    let line = Line::from(Span::styled(
        title,
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    ));
    frame.render_widget(Paragraph::new(line), area);
}

/// Render the combat title bar
fn render_combat_title(frame: &mut Frame, app: &App, area: Rect) {
    let round = app
        .session
        .world()
        .combat
        .as_ref()
        .map(|c| c.round)
        .unwrap_or(1);

    let title = format!(" ⚔ COMBAT - Round {round} ⚔ ");

    let line = Line::from(Span::styled(
        title,
        Style::default()
            .fg(Color::LightRed)
            .add_modifier(Modifier::BOLD),
    ));
    frame.render_widget(Paragraph::new(line), area);
}

/// Render the status bar
fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let status_widget = StatusBarWidget::new(
        &app.session.world().player_character,
        app.session.world().mode,
        app.input_mode,
        &app.theme,
    )
    .message(app.status_message());

    frame.render_widget(status_widget, area);
}

/// Render combat-specific status bar
fn render_combat_status(frame: &mut Frame, app: &App, area: Rect) {
    let world = app.session.world();
    let character = &world.player_character;
    let hp = &character.hit_points;
    let hp_color = app.theme.hp_color(hp.ratio());

    // Build combat-specific status
    let is_player_turn = world
        .combat
        .as_ref()
        .and_then(|c| c.current_combatant())
        .map(|c| c.is_player)
        .unwrap_or(false);

    let turn_indicator = if is_player_turn {
        Span::styled(
            "YOUR TURN",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled("Waiting...", Style::default().fg(Color::DarkGray))
    };

    let line = Line::from(vec![
        Span::styled(
            format!("HP: {}/{}", hp.current, hp.maximum),
            Style::default().fg(hp_color),
        ),
        Span::raw(" | "),
        Span::styled(format!("AC: {}", character.current_ac()), Style::default()),
        Span::raw(" | "),
        turn_indicator,
        Span::raw(" | "),
        Span::styled(
            "Actions: 1  Bonus: 1  Movement: 30ft",
            Style::default().add_modifier(Modifier::DIM),
        ),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(app.theme.border_style(false));

    frame.render_widget(Paragraph::new(line).block(block), area);
}

/// Render the hotkey bar
fn render_hotkey_bar(frame: &mut Frame, app: &App, area: Rect) {
    let hotkey_widget =
        HotkeyBarWidget::new(app.session.world().mode, app.input_mode, &app.theme);
    frame.render_widget(hotkey_widget, area);
}

/// Render the input area
fn render_input(frame: &mut Frame, app: &App, area: Rect) {
    let is_active = matches!(app.input_mode, InputMode::Insert | InputMode::Command);
    let is_command = matches!(app.input_mode, InputMode::Command);

    let placeholder = if app.ai_processing {
        "Processing..."
    } else {
        "Enter your action..."
    };

    let input_widget = InputWidget::new(app.input_buffer(), &app.theme)
        .cursor_position(app.cursor_position())
        .active(is_active)
        .command_mode(is_command)
        .placeholder(placeholder);

    frame.render_widget(input_widget, area);
}

/// Render overlay
fn render_overlay(frame: &mut Frame, app: &App, overlay: &Overlay, area: Rect) {
    match overlay {
        Overlay::Help => render_help_overlay(frame, app, area),
        Overlay::DiceRoll { result, purpose, dc } => {
            render_dice_overlay(frame, app, result.as_ref(), purpose, *dc, area)
        }
    }
}

/// Render help overlay
fn render_help_overlay(frame: &mut Frame, app: &App, area: Rect) {
    let popup_area = centered_rect_fixed(50, 20, area);

    // Clear the background
    frame.render_widget(Clear, popup_area);

    let help_text = vec![
        Line::from(Span::styled(
            " D&D Dungeon Master - Help ",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Input Modes:",
            Style::default().add_modifier(Modifier::UNDERLINED),
        )),
        Line::from("  i       Enter INSERT mode (type actions)"),
        Line::from("  :       Enter COMMAND mode"),
        Line::from("  Esc     Return to NORMAL mode"),
        Line::from(""),
        Line::from(Span::styled(
            "Navigation (NORMAL mode):",
            Style::default().add_modifier(Modifier::UNDERLINED),
        )),
        Line::from("  j/k or ↑/↓     Scroll up/down"),
        Line::from("  PgUp/PgDn      Scroll by page"),
        Line::from("  Ctrl+u/d       Scroll by half page"),
        Line::from("  g/G            Jump to top/bottom"),
        Line::from("  Tab            Cycle panel focus"),
        Line::from("  Mouse wheel    Scroll narrative"),
        Line::from(""),
        Line::from(Span::styled(
            "Actions:",
            Style::default().add_modifier(Modifier::UNDERLINED),
        )),
        Line::from("  r/R     Short/long rest"),
        Line::from("  q       Quit"),
        Line::from(""),
        Line::from(Span::styled(
            "Commands:",
            Style::default().add_modifier(Modifier::UNDERLINED),
        )),
        Line::from("  :q      Quit"),
        Line::from("  :roll   Roll dice (e.g., :roll 2d6+3)"),
        Line::from("  :w      Save game"),
        Line::from(""),
        Line::from(Span::styled(
            "Press Esc or q to close",
            Style::default().add_modifier(Modifier::DIM),
        )),
    ];

    let block = Block::default()
        .title(" Help ")
        .borders(Borders::ALL)
        .border_style(app.theme.border_style(true));

    let paragraph = Paragraph::new(help_text)
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, popup_area);
}

/// Render dice roll overlay
fn render_dice_overlay(
    frame: &mut Frame,
    app: &App,
    result: Option<&RollResult>,
    purpose: &str,
    dc: Option<i32>,
    area: Rect,
) {
    let popup_area = centered_rect_fixed(30, 15, area);

    // Clear the background
    frame.render_widget(Clear, popup_area);

    // Determine animation state based on whether we have a result
    let animation_state = if result.is_some() {
        DiceAnimationState::Complete
    } else {
        DiceAnimationState::Rolling {
            frame: app.animation_frame,
        }
    };

    let mut dice_widget = DiceRollWidget::new(&app.theme)
        .purpose(purpose)
        .dc(dc)
        .animation_state(animation_state);

    if let Some(r) = result {
        dice_widget = dice_widget.result(r);
    }

    frame.render_widget(dice_widget, popup_area);
}
