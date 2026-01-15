//! Input field widget

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::dnd::ui::theme::GameTheme;

/// Input mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Insert,
}

/// Input field widget
pub struct InputWidget<'a> {
    content: &'a str,
    cursor_position: usize,
    mode: InputMode,
    theme: &'a GameTheme,
    placeholder: &'a str,
}

impl<'a> InputWidget<'a> {
    pub fn new(content: &'a str, theme: &'a GameTheme) -> Self {
        Self {
            content,
            cursor_position: content.len(),
            mode: InputMode::Insert,
            theme,
            placeholder: "Enter your action...",
        }
    }

    pub fn cursor_position(mut self, pos: usize) -> Self {
        self.cursor_position = pos;
        self
    }

    pub fn mode(mut self, mode: InputMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = placeholder;
        self
    }
}

impl Widget for InputWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(self.theme.border_style(true));

        let inner = block.inner(area);
        block.render(area, buf);

        // Build the input line with cursor
        let line = if self.content.is_empty() {
            Line::from(vec![
                Span::styled("> ", self.theme.player_style()),
                Span::styled(
                    self.placeholder,
                    Style::default().add_modifier(Modifier::DIM),
                ),
            ])
        } else {
            let before_cursor = &self.content[..self.cursor_position.min(self.content.len())];
            let at_cursor = self
                .content
                .chars()
                .nth(self.cursor_position)
                .map(|c| c.to_string())
                .unwrap_or_else(|| " ".to_string());
            let after_cursor = if self.cursor_position < self.content.len() {
                &self.content[self.cursor_position + 1..]
            } else {
                ""
            };

            Line::from(vec![
                Span::styled("> ", self.theme.player_style()),
                Span::raw(before_cursor),
                Span::styled(
                    at_cursor,
                    Style::default()
                        .add_modifier(Modifier::REVERSED)
                        .fg(self.theme.player_text),
                ),
                Span::raw(after_cursor),
            ])
        };

        let paragraph = Paragraph::new(line);
        paragraph.render(inner, buf);
    }
}
