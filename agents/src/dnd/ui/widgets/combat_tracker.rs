//! Combat tracker widget

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Widget},
};

use crate::dnd::game::combat::CombatState;
use crate::dnd::ui::theme::GameTheme;

/// Widget displaying initiative order and combat status
pub struct CombatTrackerWidget<'a> {
    combat: &'a CombatState,
    theme: &'a GameTheme,
    focused: bool,
}

impl<'a> CombatTrackerWidget<'a> {
    pub fn new(combat: &'a CombatState, theme: &'a GameTheme) -> Self {
        Self {
            combat,
            theme,
            focused: false,
        }
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }
}

impl Widget for CombatTrackerWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = format!(" Combat - Round {} ", self.combat.round);
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(self.theme.border_style(self.focused));

        let inner = block.inner(area);
        block.render(area, buf);

        // Build initiative list
        let items: Vec<ListItem> = self
            .combat
            .initiative_order
            .iter()
            .enumerate()
            .map(|(idx, entry)| {
                let is_current = idx == self.combat.turn_index;
                let style = self.theme.initiative_style(is_current, entry.is_player);

                let indicator = if is_current { "> " } else { "  " };
                let init_display = format!("[{:2}]", entry.initiative_total);

                let line = Line::from(vec![
                    Span::styled(indicator, style),
                    Span::styled(&entry.name, style),
                    Span::raw(" "),
                    Span::styled(init_display, Style::default().add_modifier(Modifier::DIM)),
                ]);

                ListItem::new(line)
            })
            .collect();

        let list = List::new(items);
        Widget::render(list, inner, buf);
    }
}

/// Widget showing enemy HP estimates
pub struct EnemyHpWidget<'a> {
    enemies: Vec<EnemyStatus>,
    theme: &'a GameTheme,
}

/// Status of an enemy for display
pub struct EnemyStatus {
    pub name: String,
    pub hp_estimate: HpEstimate,
    pub conditions: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum HpEstimate {
    /// Full health
    Healthy,
    /// Bloodied (half or less)
    Bloodied,
    /// Near death
    Critical,
    /// Dead
    Dead,
    /// Unknown
    Unknown,
}

impl<'a> EnemyHpWidget<'a> {
    pub fn new(enemies: Vec<EnemyStatus>, theme: &'a GameTheme) -> Self {
        Self { enemies, theme }
    }
}

impl Widget for EnemyHpWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(" Enemies ")
            .borders(Borders::ALL)
            .border_style(self.theme.border_style(false));

        let inner = block.inner(area);
        block.render(area, buf);

        let items: Vec<ListItem> = self
            .enemies
            .iter()
            .map(|enemy| {
                let hp_indicator = match enemy.hp_estimate {
                    HpEstimate::Healthy => "[####]",
                    HpEstimate::Bloodied => "[##--]",
                    HpEstimate::Critical => "[#---]",
                    HpEstimate::Dead => "[DEAD]",
                    HpEstimate::Unknown => "[????]",
                };

                let hp_color = match enemy.hp_estimate {
                    HpEstimate::Healthy => self.theme.hp_healthy,
                    HpEstimate::Bloodied => self.theme.hp_wounded,
                    HpEstimate::Critical => self.theme.hp_critical,
                    HpEstimate::Dead => self.theme.failure,
                    HpEstimate::Unknown => self.theme.system_text,
                };

                let line = Line::from(vec![
                    Span::raw(&enemy.name),
                    Span::raw(": "),
                    Span::styled(hp_indicator, Style::default().fg(hp_color)),
                ]);

                ListItem::new(line)
            })
            .collect();

        let list = List::new(items);
        Widget::render(list, inner, buf);
    }
}
