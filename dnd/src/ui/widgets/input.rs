//! Input field widget

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::ui::theme::GameTheme;

/// Input field widget
pub struct InputWidget<'a> {
    content: &'a str,
    cursor_position: usize,
    theme: &'a GameTheme,
    placeholder: &'a str,
    is_active: bool,
    is_command_mode: bool,
}

impl<'a> InputWidget<'a> {
    pub fn new(content: &'a str, theme: &'a GameTheme) -> Self {
        Self {
            content,
            cursor_position: content.len(),
            theme,
            placeholder: "Enter your action...",
            is_active: true,
            is_command_mode: false,
        }
    }

    pub fn cursor_position(mut self, pos: usize) -> Self {
        self.cursor_position = pos;
        self
    }

    pub fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = placeholder;
        self
    }

    pub fn active(mut self, active: bool) -> Self {
        self.is_active = active;
        self
    }

    pub fn command_mode(mut self, is_command: bool) -> Self {
        self.is_command_mode = is_command;
        self
    }
}

impl Widget for InputWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(self.theme.border_style(self.is_active));

        let inner = block.inner(area);
        block.render(area, buf);

        // Build the input line with cursor
        let line = if self.content.is_empty() && !self.is_command_mode {
            Line::from(vec![
                Span::styled("> ", self.theme.player_style()),
                Span::styled(
                    self.placeholder,
                    Style::default().add_modifier(Modifier::DIM),
                ),
            ])
        } else {
            let prefix = if self.is_command_mode { ":" } else { "> " };
            let display_content = if self.is_command_mode && self.content.starts_with(':') {
                &self.content[1..]
            } else {
                self.content
            };

            let adjusted_cursor = if self.is_command_mode && self.cursor_position > 0 {
                self.cursor_position.saturating_sub(1)
            } else {
                self.cursor_position
            };

            // Use character-based slicing for unicode safety
            let before_cursor: String = display_content.chars().take(adjusted_cursor).collect();
            let at_cursor = display_content
                .chars()
                .nth(adjusted_cursor)
                .map(|c| c.to_string())
                .unwrap_or_else(|| " ".to_string());
            let char_count = display_content.chars().count();
            let after_cursor = if adjusted_cursor < char_count {
                display_content
                    .chars()
                    .skip(adjusted_cursor + 1)
                    .collect::<String>()
            } else {
                String::new()
            };

            Line::from(vec![
                Span::styled(prefix, self.theme.player_style()),
                Span::raw(before_cursor),
                Span::styled(
                    at_cursor,
                    Style::default()
                        .add_modifier(Modifier::UNDERLINED | Modifier::BOLD)
                        .fg(self.theme.player_text),
                ),
                Span::raw(after_cursor),
            ])
        };

        let paragraph = Paragraph::new(line);
        paragraph.render(inner, buf);
    }
}
