//! Narrative display widget

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    symbols::scrollbar,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Widget, Wrap},
};

use dnd_core::world::NarrativeType;

use crate::ui::theme::GameTheme;

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
        // Build title with scroll indicator
        let title = if self.focused {
            " Narrative [j/k scroll] "
        } else {
            " Narrative "
        };

        let block = Block::default()
            .title(title)
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

        let paragraph = Paragraph::new(lines.clone())
            .scroll((scroll as u16, 0))
            .wrap(Wrap { trim: false });

        paragraph.render(inner, buf);

        // Render scrollbar if content exceeds visible area
        if total_lines > visible_height {
            // Create scrollbar area (inside the block, on the right)
            let scrollbar_area = Rect {
                x: inner.x + inner.width.saturating_sub(1),
                y: inner.y,
                width: 1,
                height: inner.height,
            };

            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .symbols(scrollbar::VERTICAL)
                .thumb_style(Style::default().fg(Color::DarkGray))
                .track_style(Style::default().fg(Color::Black))
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"));

            let mut scrollbar_state = ScrollbarState::new(max_scroll)
                .position(scroll);

            scrollbar.render(scrollbar_area, buf, &mut scrollbar_state);

            // Add scroll position hint at top if scrolled down
            if scroll > 0 {
                let hint = format!(" ↑{scroll} ");
                let hint_x = inner.x;
                let hint_style = Style::default().fg(Color::DarkGray).add_modifier(Modifier::DIM);
                for (i, ch) in hint.chars().enumerate() {
                    let x = hint_x + (i as u16);
                    if x < inner.x + inner.width.saturating_sub(2) {
                        buf[(x, inner.y)].set_char(ch).set_style(hint_style);
                    }
                }
            }

            // Add hint at bottom if more content below
            if scroll < max_scroll {
                let remaining = max_scroll - scroll;
                let hint = format!(" ↓{remaining} more ");
                let hint_x = inner.x;
                let hint_y = inner.y + inner.height.saturating_sub(1);
                let hint_style = Style::default().fg(Color::DarkGray).add_modifier(Modifier::DIM);
                for (i, ch) in hint.chars().enumerate() {
                    let x = hint_x + (i as u16);
                    if x < inner.x + inner.width.saturating_sub(2) {
                        buf[(x, hint_y)].set_char(ch).set_style(hint_style);
                    }
                }
            }
        }
    }
}
