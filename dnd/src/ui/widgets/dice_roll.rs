//! Animated dice roll display widget

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

use dnd_core::dice::RollResult;

use crate::ui::theme::GameTheme;

/// Animation state for dice roll
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum DiceAnimationState {
    Rolling { frame: u8 },
    Revealing,
    Complete,
}

/// Widget for displaying dice roll results with animation
pub struct DiceRollWidget<'a> {
    result: Option<&'a RollResult>,
    purpose: &'a str,
    dc: Option<i32>,
    animation_state: DiceAnimationState,
    theme: &'a GameTheme,
}

impl<'a> DiceRollWidget<'a> {
    pub fn new(theme: &'a GameTheme) -> Self {
        Self {
            result: None,
            purpose: "",
            dc: None,
            animation_state: DiceAnimationState::Complete,
            theme,
        }
    }

    pub fn result(mut self, result: &'a RollResult) -> Self {
        self.result = Some(result);
        self
    }

    pub fn purpose(mut self, purpose: &'a str) -> Self {
        self.purpose = purpose;
        self
    }

    pub fn dc(mut self, dc: Option<i32>) -> Self {
        self.dc = dc;
        self
    }

    pub fn animation_state(mut self, state: DiceAnimationState) -> Self {
        self.animation_state = state;
        self
    }
}

impl Widget for DiceRollWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(" Dice Roll ")
            .borders(Borders::ALL)
            .border_style(self.theme.border_style(true));

        let inner = block.inner(area);
        block.render(area, buf);

        let mut lines: Vec<Line> = Vec::new();

        // Purpose line
        if !self.purpose.is_empty() {
            lines.push(Line::from(Span::styled(
                self.purpose,
                Style::default().add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));
        }

        match &self.animation_state {
            DiceAnimationState::Rolling { frame } => {
                // Show rolling animation
                let spin_chars = ['|', '/', '-', '\\'];
                let spin = spin_chars[(*frame as usize) % 4];

                lines.push(Line::from("  ╭───╮"));
                lines.push(Line::from(format!("  │ {spin} │")));
                lines.push(Line::from("  ╰───╯"));
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "Rolling...",
                    Style::default().add_modifier(Modifier::DIM),
                )));
            }
            DiceAnimationState::Revealing | DiceAnimationState::Complete => {
                if let Some(result) = self.result {
                    // Determine style based on result
                    let success = self.dc.map(|dc| result.total >= dc);
                    let style =
                        self.theme
                            .roll_result_style(result.natural_20, result.natural_1, success);

                    // Show the result
                    if result.natural_20 {
                        lines.push(Line::from(Span::styled(
                            "  .--===--.",
                            style,
                        )));
                        lines.push(Line::from(Span::styled(
                            " / NAT 20! \\",
                            style,
                        )));
                        lines.push(Line::from(Span::styled(
                            "|  CRITICAL |",
                            style,
                        )));
                        lines.push(Line::from(Span::styled(
                            " \\   HIT!  /",
                            style,
                        )));
                        lines.push(Line::from(Span::styled(
                            "  '---==---'",
                            style,
                        )));
                    } else if result.natural_1 {
                        lines.push(Line::from(Span::styled(
                            "  .-------.",
                            style,
                        )));
                        lines.push(Line::from(Span::styled(
                            " / NAT  1  \\",
                            style,
                        )));
                        lines.push(Line::from(Span::styled(
                            "|  FUMBLE!  |",
                            style,
                        )));
                        lines.push(Line::from(Span::styled(
                            " \\        /",
                            style,
                        )));
                        lines.push(Line::from(Span::styled(
                            "  '-------'",
                            style,
                        )));
                    } else {
                        // Normal result
                        lines.push(Line::from("  ╭─────╮"));
                        lines.push(Line::from(Span::styled(
                            format!("  │ {:3} │", result.total),
                            style.add_modifier(Modifier::BOLD),
                        )));
                        lines.push(Line::from("  ╰─────╯"));
                    }

                    lines.push(Line::from(""));

                    // Show breakdown
                    lines.push(Line::from(Span::styled(
                        result.dice_display(),
                        Style::default().add_modifier(Modifier::DIM),
                    )));

                    // Show DC comparison if present
                    if let Some(dc) = self.dc {
                        let outcome = if result.total >= dc {
                            Span::styled("SUCCESS!", self.theme.roll_result_style(false, false, Some(true)))
                        } else {
                            Span::styled("FAILURE", self.theme.roll_result_style(false, false, Some(false)))
                        };

                        lines.push(Line::from(vec![
                            Span::raw(format!("vs DC {dc} - ")),
                            outcome,
                        ]));
                    }
                } else {
                    lines.push(Line::from("No roll to display"));
                }
            }
        }

        let paragraph = Paragraph::new(lines).alignment(Alignment::Center);
        paragraph.render(inner, buf);
    }
}
