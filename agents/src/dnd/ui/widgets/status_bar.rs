//! Status bar widget

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use crate::dnd::app::InputMode;
use crate::dnd::game::character::Character;
use crate::dnd::game::state::GameMode;
use crate::dnd::ui::theme::GameTheme;

/// Status bar widget showing quick stats
pub struct StatusBarWidget<'a> {
    character: &'a Character,
    game_mode: GameMode,
    input_mode: InputMode,
    theme: &'a GameTheme,
    message: Option<&'a str>,
}

impl<'a> StatusBarWidget<'a> {
    pub fn new(character: &'a Character, game_mode: GameMode, input_mode: InputMode, theme: &'a GameTheme) -> Self {
        Self {
            character,
            game_mode,
            input_mode,
            theme,
            message: None,
        }
    }

    pub fn message(mut self, message: Option<&'a str>) -> Self {
        self.message = message;
        self
    }
}

impl Widget for StatusBarWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let hp = &self.character.hit_points;
        let hp_ratio = hp.ratio();
        let hp_color = self.theme.hp_color(hp_ratio);

        // HP display
        let hp_text = if hp.temporary > 0 {
            format!("HP: {}/{} (+{})", hp.current, hp.maximum, hp.temporary)
        } else {
            format!("HP: {}/{}", hp.current, hp.maximum)
        };

        // AC display
        let ac = self.character.current_ac();

        // Game mode indicator
        let game_mode_text = match self.game_mode {
            GameMode::Exploration => "EXPLORE",
            GameMode::Combat => "COMBAT",
            GameMode::Dialogue => "DIALOGUE",
            GameMode::Rest => "RESTING",
            GameMode::Shopping => "SHOP",
            GameMode::CharacterManagement => "CHARACTER",
        };

        let game_mode_style = match self.game_mode {
            GameMode::Combat => Style::default()
                .fg(self.theme.combat_text)
                .add_modifier(Modifier::BOLD),
            GameMode::Dialogue => Style::default().fg(self.theme.npc_dialogue),
            _ => Style::default().fg(self.theme.system_text),
        };

        // Input mode indicator (vim-style)
        let (input_mode_text, input_mode_style) = match self.input_mode {
            InputMode::Normal => ("NORMAL", Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
            InputMode::Insert => ("INSERT", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            InputMode::Command => ("COMMAND", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        };

        let mut spans = vec![
            Span::styled(format!("-- {} --", input_mode_text), input_mode_style),
            Span::raw(" | "),
            Span::styled(hp_text, Style::default().fg(hp_color)),
            Span::raw(" | "),
            Span::styled(format!("AC: {}", ac), Style::default()),
            Span::raw(" | "),
            Span::styled(game_mode_text, game_mode_style),
        ];

        // Add message if present
        if let Some(msg) = self.message {
            spans.push(Span::raw(" | "));
            spans.push(Span::styled(
                msg,
                Style::default().add_modifier(Modifier::DIM),
            ));
        }

        let line = Line::from(spans);
        let paragraph = Paragraph::new(line);
        paragraph.render(area, buf);
    }
}

/// Hotkey bar widget
pub struct HotkeyBarWidget<'a> {
    game_mode: GameMode,
    input_mode: InputMode,
    theme: &'a GameTheme,
}

impl<'a> HotkeyBarWidget<'a> {
    pub fn new(game_mode: GameMode, input_mode: InputMode, theme: &'a GameTheme) -> Self {
        Self { game_mode, input_mode, theme }
    }
}

impl Widget for HotkeyBarWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let hotkeys = match self.input_mode {
            InputMode::Normal => match self.game_mode {
                GameMode::Combat => vec![
                    ("i:insert", true),
                    ("A:attack", true),
                    ("c:cast", true),
                    ("d:dash", true),
                    ("e:end", true),
                    ("?:help", false),
                ],
                GameMode::Dialogue => vec![
                    ("i:insert", true),
                    ("1-9:response", true),
                    ("Esc:leave", false),
                    ("?:help", false),
                ],
                _ => vec![
                    ("i:insert", true),
                    ("::command", true),
                    ("j/k:scroll", true),
                    ("r:rest", true),
                    ("R:long rest", true),
                    ("?:help", false),
                ],
            },
            InputMode::Insert => vec![
                ("Esc:normal", true),
                ("Enter:send", true),
                ("↑↓:history", false),
            ],
            InputMode::Command => vec![
                ("Esc:cancel", true),
                ("Enter:execute", true),
                (":q quit", false),
                (":roll XdY", false),
                (":help", false),
            ],
        };

        let spans: Vec<Span> = hotkeys
            .iter()
            .flat_map(|(text, primary)| {
                let style = if *primary {
                    Style::default()
                } else {
                    Style::default().add_modifier(Modifier::DIM)
                };
                vec![Span::styled(*text, style), Span::raw("  ")]
            })
            .collect();

        let line = Line::from(spans);
        let paragraph = Paragraph::new(line);
        paragraph.render(area, buf);
    }
}
