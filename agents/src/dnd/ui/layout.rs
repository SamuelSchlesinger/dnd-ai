//! Layout calculations for the D&D TUI

use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Calculate the main layout areas
pub struct AppLayout {
    pub title_area: Rect,
    pub narrative_area: Rect,
    pub sidebar_area: Rect,
    pub status_bar: Rect,
    pub hotkey_bar: Rect,
    pub input_area: Rect,
}

impl AppLayout {
    /// Calculate layout based on terminal size
    pub fn calculate(area: Rect) -> Self {
        // Main vertical split
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),  // Title bar
                Constraint::Min(8),     // Main content
                Constraint::Length(1),  // Status bar
                Constraint::Length(1),  // Hotkey bar
                Constraint::Length(3),  // Input area
            ])
            .split(area);

        // Content area: narrative + sidebar
        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(70),
                Constraint::Percentage(30),
            ])
            .split(main_chunks[1]);

        Self {
            title_area: main_chunks[0],
            narrative_area: content_chunks[0],
            sidebar_area: content_chunks[1],
            status_bar: main_chunks[2],
            hotkey_bar: main_chunks[3],
            input_area: main_chunks[4],
        }
    }

    /// Calculate combat-specific layout
    pub fn calculate_combat(area: Rect) -> CombatLayout {
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),  // Title
                Constraint::Min(8),     // Content
                Constraint::Length(3),  // Status
                Constraint::Length(1),  // Hotkeys
                Constraint::Length(3),  // Input
            ])
            .split(area);

        // Combat content: narrative + tracker
        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(65),
                Constraint::Percentage(35),
            ])
            .split(main_chunks[1]);

        // Split tracker into initiative and enemy HP
        let tracker_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(content_chunks[1]);

        CombatLayout {
            title_area: main_chunks[0],
            narrative_area: content_chunks[0],
            initiative_area: tracker_chunks[0],
            enemy_hp_area: tracker_chunks[1],
            status_bar: main_chunks[2],
            hotkey_bar: main_chunks[3],
            input_area: main_chunks[4],
        }
    }
}

/// Combat-specific layout
pub struct CombatLayout {
    pub title_area: Rect,
    pub narrative_area: Rect,
    pub initiative_area: Rect,
    pub enemy_hp_area: Rect,
    pub status_bar: Rect,
    pub hotkey_bar: Rect,
    pub input_area: Rect,
}

/// Calculate centered popup area
pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Calculate fixed-size centered popup
pub fn centered_rect_fixed(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;

    Rect::new(x, y, width.min(area.width), height.min(area.height))
}
