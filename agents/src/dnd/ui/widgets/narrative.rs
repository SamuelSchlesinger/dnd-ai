//! Narrative display widget

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};

use crate::dnd::game::state::NarrativeType;
use crate::dnd::ui::theme::GameTheme;

/// A single entry in the narrative display
#[derive(Debug, Clone)]
pub struct NarrativeItem {
    pub content: String,
    pub entry_type: NarrativeType,
}

/// Widget for displaying narrative text
pub struct NarrativeWidget<'a> {
    items: &'a [NarrativeItem],
    scroll: usize,
    theme: &'a GameTheme,
    focused: bool,
    streaming_text: Option<&'a str>,
}

impl<'a> NarrativeWidget<'a> {
    pub fn new(items: &'a [NarrativeItem], theme: &'a GameTheme) -> Self {
        Self {
            items,
            scroll: 0,
            theme,
            focused: false,
            streaming_text: None,
        }
    }

    pub fn scroll(mut self, scroll: usize) -> Self {
        self.scroll = scroll;
        self
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    pub fn streaming(mut self, text: Option<&'a str>) -> Self {
        self.streaming_text = text;
        self
    }

    fn style_for_type(&self, entry_type: NarrativeType) -> Style {
        match entry_type {
            NarrativeType::DmNarration => self.theme.dm_style(),
            NarrativeType::PlayerAction => self.theme.player_style(),
            NarrativeType::NpcDialogue => self.theme.npc_style(),
            NarrativeType::Combat => self.theme.combat_style(),
            NarrativeType::System => self.theme.system_style(),
        }
    }
}

impl Widget for NarrativeWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(" Narrative ")
            .borders(Borders::ALL)
            .border_style(self.theme.border_style(self.focused));

        let inner = block.inner(area);
        block.render(area, buf);

        // Build lines from narrative items
        let mut lines: Vec<Line> = Vec::new();

        for item in self.items {
            let style = self.style_for_type(item.entry_type);

            // Add prefix based on type
            let prefix = match item.entry_type {
                NarrativeType::PlayerAction => "> ",
                NarrativeType::NpcDialogue => "",
                NarrativeType::System => "[ ",
                _ => "",
            };

            let suffix = match item.entry_type {
                NarrativeType::System => " ]",
                _ => "",
            };

            let text = format!("{}{}{}", prefix, item.content, suffix);

            // Word wrap manually for long lines
            for line in text.lines() {
                lines.push(Line::from(Span::styled(line.to_string(), style)));
            }

            // Add blank line between entries
            lines.push(Line::from(""));
        }

        // Add streaming text if present
        if let Some(streaming) = self.streaming_text {
            let style = self.theme.dm_style().add_modifier(Modifier::DIM);
            for line in streaming.lines() {
                lines.push(Line::from(Span::styled(line.to_string(), style)));
            }
            // Add cursor indicator
            lines.push(Line::from(Span::styled("▌", style)));
        }

        // Calculate scroll position
        let visible_height = inner.height as usize;
        let total_lines = lines.len();
        let max_scroll = total_lines.saturating_sub(visible_height);
        let scroll = self.scroll.min(max_scroll);

        let paragraph = Paragraph::new(lines)
            .scroll((scroll as u16, 0))
            .wrap(Wrap { trim: false });

        paragraph.render(inner, buf);
    }
}
