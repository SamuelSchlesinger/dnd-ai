//! Character panel widget for sidebar display

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph, Widget},
};

use dnd_core::world::{Ability, Character};

use crate::ui::theme::GameTheme;

/// Compact character panel for sidebar
pub struct CharacterPanelWidget<'a> {
    character: &'a Character,
    theme: &'a GameTheme,
    focused: bool,
}

impl<'a> CharacterPanelWidget<'a> {
    pub fn new(character: &'a Character, theme: &'a GameTheme) -> Self {
        Self {
            character,
            theme,
            focused: false,
        }
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }
}

impl Widget for CharacterPanelWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(format!(" {} ", self.character.name))
            .borders(Borders::ALL)
            .border_style(self.theme.border_style(self.focused));

        let inner = block.inner(area);
        block.render(area, buf);

        // Split into sections
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Class/Level
                Constraint::Length(2), // HP bar
                Constraint::Length(2), // AC/Init/Speed (2 lines now)
                Constraint::Length(6), // Ability scores
                Constraint::Min(0),    // Features/Conditions
            ])
            .split(inner);

        // Class and level
        let class_text = if let Some(class) = self.character.classes.first() {
            format!("Level {} {}", class.level, class.class)
        } else {
            format!("Level {}", self.character.level)
        };
        let class_line = Line::from(Span::styled(
            class_text,
            Style::default().add_modifier(Modifier::DIM),
        ));
        Paragraph::new(class_line).render(chunks[0], buf);

        // HP bar
        let hp = &self.character.hit_points;
        let hp_ratio = hp.ratio();
        let hp_color = self.theme.hp_color(hp_ratio);

        let hp_label = if hp.temporary > 0 {
            format!("HP: {}/{} (+{})", hp.current, hp.maximum, hp.temporary)
        } else {
            format!("HP: {}/{}", hp.current, hp.maximum)
        };

        let gauge = Gauge::default()
            .block(Block::default())
            .gauge_style(Style::default().fg(hp_color))
            .ratio(hp_ratio as f64)
            .label(hp_label);
        gauge.render(chunks[1], buf);

        // AC, Initiative, Speed - on two lines to avoid cutoff
        let ac = self.character.current_ac();
        let init = self.character.initiative_modifier();
        let speed = self.character.speed.walk;

        let init_str = if init >= 0 {
            format!("+{init}")
        } else {
            format!("{init}")
        };

        let combat_stats = vec![
            Line::from(vec![
                Span::raw("AC: "),
                Span::styled(
                    format!("{ac}"),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw("  Init: "),
                Span::styled(init_str, Style::default()),
            ]),
            Line::from(vec![
                Span::raw("Speed: "),
                Span::styled(format!("{speed} ft"), Style::default()),
            ]),
        ];
        Paragraph::new(combat_stats).render(chunks[2], buf);

        // Ability scores
        let abilities_text = render_ability_scores(self.character);
        Paragraph::new(abilities_text).render(chunks[3], buf);

        // Features and conditions
        if chunks[4].height > 0 {
            let mut lines = Vec::new();

            // Show limited-use features
            for feature in &self.character.features {
                if let Some(ref uses) = feature.uses {
                    let status = if uses.current > 0 { "[X]" } else { "[ ]" };
                    lines.push(Line::from(format!(
                        "{} {} ({}/{})",
                        status, feature.name, uses.current, uses.maximum
                    )));
                }
            }

            // Show conditions
            if !self.character.conditions.is_empty() {
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "Conditions:",
                    Style::default().add_modifier(Modifier::BOLD),
                )));
                for condition in &self.character.conditions {
                    lines.push(Line::from(Span::styled(
                        format!("  {}", condition.condition),
                        self.theme.combat_style(),
                    )));
                }
            }

            Paragraph::new(lines).render(chunks[4], buf);
        }
    }
}

fn render_ability_scores(character: &Character) -> Vec<Line<'static>> {
    let abilities = [
        (Ability::Strength, "STR"),
        (Ability::Dexterity, "DEX"),
        (Ability::Constitution, "CON"),
        (Ability::Intelligence, "INT"),
        (Ability::Wisdom, "WIS"),
        (Ability::Charisma, "CHA"),
    ];

    abilities
        .iter()
        .map(|(ability, abbr)| {
            let score = character.ability_scores.get(*ability);
            let modifier = character.ability_scores.modifier(*ability);
            let mod_str = if modifier >= 0 {
                format!("+{modifier}")
            } else {
                format!("{modifier}")
            };

            Line::from(format!("{abbr}: {score:2} ({mod_str})"))
        })
        .collect()
}
