//! Color theme and styling for the D&D TUI

use ratatui::style::{Color, Modifier, Style};

/// Game UI color theme
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct GameTheme {
    // Base colors
    pub background: Color,
    pub foreground: Color,
    pub border: Color,
    pub border_focused: Color,

    // HP colors
    pub hp_healthy: Color,
    pub hp_wounded: Color,
    pub hp_critical: Color,

    // Roll result colors
    pub crit_success: Color,
    pub success: Color,
    pub failure: Color,
    pub crit_failure: Color,

    // Text colors
    pub player_text: Color,
    pub dm_text: Color,
    pub npc_dialogue: Color,
    pub combat_text: Color,
    pub system_text: Color,

    // Combat colors
    pub player_turn: Color,
    pub enemy_turn: Color,
    pub ally_turn: Color,
}

impl Default for GameTheme {
    fn default() -> Self {
        Self {
            background: Color::Reset,
            foreground: Color::White,
            border: Color::DarkGray,
            border_focused: Color::Cyan,

            hp_healthy: Color::Green,
            hp_wounded: Color::Yellow,
            hp_critical: Color::Red,

            crit_success: Color::Yellow,
            success: Color::Green,
            failure: Color::DarkGray,
            crit_failure: Color::Red,

            player_text: Color::Cyan,
            dm_text: Color::White,
            npc_dialogue: Color::Yellow,
            combat_text: Color::LightRed,
            system_text: Color::DarkGray,

            player_turn: Color::LightGreen,
            enemy_turn: Color::LightRed,
            ally_turn: Color::LightBlue,
        }
    }
}

impl GameTheme {
    /// Get style for normal text
    #[allow(dead_code)]
    pub fn text_style(&self) -> Style {
        Style::default().fg(self.foreground)
    }

    /// Get style for DM narration
    pub fn dm_style(&self) -> Style {
        Style::default().fg(self.dm_text)
    }

    /// Get style for player actions
    pub fn player_style(&self) -> Style {
        Style::default()
            .fg(self.player_text)
            .add_modifier(Modifier::ITALIC)
    }

    /// Get style for NPC dialogue
    pub fn npc_style(&self) -> Style {
        Style::default().fg(self.npc_dialogue)
    }

    /// Get style for combat text
    pub fn combat_style(&self) -> Style {
        Style::default().fg(self.combat_text)
    }

    /// Get style for system messages
    pub fn system_style(&self) -> Style {
        Style::default()
            .fg(self.system_text)
            .add_modifier(Modifier::DIM)
    }

    /// Get HP bar color based on ratio
    pub fn hp_color(&self, ratio: f32) -> Color {
        if ratio > 0.5 {
            self.hp_healthy
        } else if ratio > 0.25 {
            self.hp_wounded
        } else {
            self.hp_critical
        }
    }

    /// Get style for dice roll result
    pub fn roll_result_style(&self, is_crit: bool, is_fumble: bool, success: Option<bool>) -> Style {
        if is_crit {
            Style::default()
                .fg(self.crit_success)
                .add_modifier(Modifier::BOLD)
        } else if is_fumble {
            Style::default()
                .fg(self.crit_failure)
                .add_modifier(Modifier::BOLD)
        } else {
            match success {
                Some(true) => Style::default().fg(self.success),
                Some(false) => Style::default().fg(self.failure),
                None => Style::default().fg(self.foreground),
            }
        }
    }

    /// Get border style
    pub fn border_style(&self, focused: bool) -> Style {
        Style::default().fg(if focused {
            self.border_focused
        } else {
            self.border
        })
    }

    /// Get title style
    #[allow(dead_code)]
    pub fn title_style(&self, focused: bool) -> Style {
        let style = Style::default().fg(if focused {
            self.border_focused
        } else {
            self.foreground
        });

        if focused {
            style.add_modifier(Modifier::BOLD)
        } else {
            style
        }
    }

    /// Get initiative entry style
    pub fn initiative_style(&self, is_current: bool, is_player: bool) -> Style {
        let color = if is_player {
            self.player_turn
        } else {
            self.enemy_turn
        };

        let style = Style::default().fg(color);
        if is_current {
            style.add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
        } else {
            style
        }
    }
}
