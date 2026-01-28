//! Character creation TUI wizard.
//!
//! A multi-step interface for creating D&D 5e characters.

use crossterm::event::{Event, KeyCode, KeyEvent};
use dnd_core::world::{
    Ability, AbilityScores, Background, Character, CharacterClass, ClassLevel, HitPoints,
    ProficiencyLevel, Race, RaceType, Skill, Speed,
};
use dnd_core::{AbilityMethod, CharacterBuilder};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Gauge, List, ListItem, ListState, Paragraph, Wrap};
use std::collections::HashSet;

/// Point buy costs for each score value.
fn point_buy_cost(score: u8) -> u8 {
    match score {
        8 => 0,
        9 => 1,
        10 => 2,
        11 => 3,
        12 => 4,
        13 => 5,
        14 => 7,
        15 => 9,
        _ => 0,
    }
}

/// Steps in character creation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreationStep {
    Name,
    Race,
    Class,
    Background,
    AbilityMethod,
    AbilityScores,
    Skills,
    Review,
}

impl CreationStep {
    pub fn title(&self) -> &'static str {
        match self {
            CreationStep::Name => "Enter Your Name",
            CreationStep::Race => "Choose Your Race",
            CreationStep::Class => "Choose Your Class",
            CreationStep::Background => "Choose Your Background",
            CreationStep::AbilityMethod => "Ability Score Method",
            CreationStep::AbilityScores => "Assign Ability Scores",
            CreationStep::Skills => "Choose Skills",
            CreationStep::Review => "Review Character",
        }
    }

    pub fn next(&self) -> Option<CreationStep> {
        match self {
            CreationStep::Name => Some(CreationStep::Race),
            CreationStep::Race => Some(CreationStep::Class),
            CreationStep::Class => Some(CreationStep::Background),
            CreationStep::Background => Some(CreationStep::AbilityMethod),
            CreationStep::AbilityMethod => Some(CreationStep::AbilityScores),
            CreationStep::AbilityScores => Some(CreationStep::Skills),
            CreationStep::Skills => Some(CreationStep::Review),
            CreationStep::Review => None,
        }
    }

    pub fn prev(&self) -> Option<CreationStep> {
        match self {
            CreationStep::Name => None,
            CreationStep::Race => Some(CreationStep::Name),
            CreationStep::Class => Some(CreationStep::Race),
            CreationStep::Background => Some(CreationStep::Class),
            CreationStep::AbilityMethod => Some(CreationStep::Background),
            CreationStep::AbilityScores => Some(CreationStep::AbilityMethod),
            CreationStep::Skills => Some(CreationStep::AbilityScores),
            CreationStep::Review => Some(CreationStep::Skills),
        }
    }
}

/// Character creation state.
pub struct CharacterCreation {
    pub step: CreationStep,
    pub name: String,
    pub race: Option<RaceType>,
    pub class: Option<CharacterClass>,
    pub background: Option<Background>,
    pub ability_method: AbilityMethod,
    pub ability_scores: AbilityScores,
    pub ability_assignment: [Option<Ability>; 6], // For standard array: which ability gets each value
    pub rolled_scores: [u8; 6],                   // For rolled: the 6 rolled values
    pub point_buy_points: u8,                     // For point buy: remaining points
    pub selected_skills: Vec<Skill>,
    pub available_skills: Vec<Skill>,
    pub required_skill_count: usize,

    // UI state
    pub list_state: ListState,
    pub assignment_index: usize, // Current ability being edited
    pub cursor_position: usize,  // For name input
    pub finished: bool,
    pub cancelled: bool,
}

impl CharacterCreation {
    pub fn new() -> Self {
        Self {
            step: CreationStep::Name,
            name: String::new(),
            race: None,
            class: None,
            background: None,
            ability_method: AbilityMethod::StandardArray,
            ability_scores: AbilityScores::default(),
            ability_assignment: [None; 6],
            rolled_scores: [0; 6],
            point_buy_points: 27,
            selected_skills: Vec::new(),
            available_skills: Vec::new(),
            required_skill_count: 0,
            list_state: ListState::default(),
            assignment_index: 0,
            cursor_position: 0,
            finished: false,
            cancelled: false,
        }
    }

    /// Handle keyboard input.
    pub fn handle_event(&mut self, event: Event) {
        if let Event::Key(key) = event {
            match self.step {
                CreationStep::Name => self.handle_name_input(key),
                CreationStep::Race => self.handle_list_selection(key, RaceType::all().len()),
                CreationStep::Class => {
                    self.handle_list_selection(key, CharacterClass::all().len())
                }
                CreationStep::Background => {
                    self.handle_list_selection(key, Background::all().len())
                }
                CreationStep::AbilityMethod => {
                    self.handle_list_selection(key, AbilityMethod::all().len())
                }
                CreationStep::AbilityScores => self.handle_ability_assignment(key),
                CreationStep::Skills => self.handle_skill_selection(key),
                CreationStep::Review => self.handle_review(key),
            }
        }
    }

    fn handle_name_input(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char(c) => {
                if self.name.len() < 30 {
                    self.name.insert(self.cursor_position, c);
                    self.cursor_position += 1;
                }
            }
            KeyCode::Backspace => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                    self.name.remove(self.cursor_position);
                }
            }
            KeyCode::Delete => {
                if self.cursor_position < self.name.len() {
                    self.name.remove(self.cursor_position);
                }
            }
            KeyCode::Left => {
                self.cursor_position = self.cursor_position.saturating_sub(1);
            }
            KeyCode::Right => {
                self.cursor_position = (self.cursor_position + 1).min(self.name.len());
            }
            KeyCode::Enter => {
                if !self.name.trim().is_empty() {
                    self.advance_step();
                }
            }
            KeyCode::Esc => {
                self.cancelled = true;
            }
            _ => {}
        }
    }

    fn handle_list_selection(&mut self, key: KeyEvent, max_items: usize) {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                let i = self.list_state.selected().unwrap_or(0);
                self.list_state
                    .select(Some(if i == 0 { max_items - 1 } else { i - 1 }));
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let i = self.list_state.selected().unwrap_or(0);
                self.list_state.select(Some((i + 1) % max_items));
            }
            KeyCode::Enter => {
                self.confirm_selection();
            }
            KeyCode::Esc => {
                if let Some(prev) = self.step.prev() {
                    self.step = prev;
                    self.list_state.select(Some(0));
                } else {
                    self.cancelled = true;
                }
            }
            _ => {}
        }
    }

    fn handle_ability_assignment(&mut self, key: KeyEvent) {
        match self.ability_method {
            AbilityMethod::StandardArray => self.handle_standard_array(key),
            AbilityMethod::PointBuy => self.handle_point_buy(key),
            AbilityMethod::Rolled => self.handle_rolled(key),
        }
    }

    fn handle_standard_array(&mut self, key: KeyEvent) {
        let standard_array = [15u8, 14, 13, 12, 10, 8];

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.assignment_index = self.assignment_index.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.assignment_index = (self.assignment_index + 1).min(5);
            }
            KeyCode::Char('1') => self.assign_array_value(0, standard_array[0]),
            KeyCode::Char('2') => self.assign_array_value(1, standard_array[1]),
            KeyCode::Char('3') => self.assign_array_value(2, standard_array[2]),
            KeyCode::Char('4') => self.assign_array_value(3, standard_array[3]),
            KeyCode::Char('5') => self.assign_array_value(4, standard_array[4]),
            KeyCode::Char('6') => self.assign_array_value(5, standard_array[5]),
            KeyCode::Enter => {
                if self.ability_assignment.iter().all(|a| a.is_some()) {
                    self.advance_step();
                }
            }
            KeyCode::Esc => {
                if let Some(prev) = self.step.prev() {
                    self.step = prev;
                    self.ability_assignment = [None; 6];
                    self.list_state.select(Some(0));
                }
            }
            _ => {}
        }
    }

    fn handle_point_buy(&mut self, key: KeyEvent) {
        let abilities = Ability::all();
        let current_ability = abilities[self.assignment_index];
        let current_score = self.ability_scores.get(current_ability);

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.assignment_index = self.assignment_index.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.assignment_index = (self.assignment_index + 1).min(5);
            }
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Char('+') | KeyCode::Char('=') => {
                // Increase score
                if current_score < 15 {
                    let current_cost = point_buy_cost(current_score);
                    let new_cost = point_buy_cost(current_score + 1);
                    let cost_diff = new_cost - current_cost;
                    if self.point_buy_points >= cost_diff {
                        self.ability_scores.set(current_ability, current_score + 1);
                        self.point_buy_points -= cost_diff;
                    }
                }
            }
            KeyCode::Left | KeyCode::Char('h') | KeyCode::Char('-') => {
                // Decrease score
                if current_score > 8 {
                    let current_cost = point_buy_cost(current_score);
                    let new_cost = point_buy_cost(current_score - 1);
                    let refund = current_cost - new_cost;
                    self.ability_scores.set(current_ability, current_score - 1);
                    self.point_buy_points += refund;
                }
            }
            KeyCode::Enter => {
                // Point buy is always "complete" - can proceed anytime
                self.advance_step();
            }
            KeyCode::Esc => {
                if let Some(prev) = self.step.prev() {
                    self.step = prev;
                    self.list_state.select(Some(0));
                }
            }
            _ => {}
        }
    }

    fn handle_rolled(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.assignment_index = self.assignment_index.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.assignment_index = (self.assignment_index + 1).min(5);
            }
            KeyCode::Char('1') => self.assign_array_value(0, self.rolled_scores[0]),
            KeyCode::Char('2') => self.assign_array_value(1, self.rolled_scores[1]),
            KeyCode::Char('3') => self.assign_array_value(2, self.rolled_scores[2]),
            KeyCode::Char('4') => self.assign_array_value(3, self.rolled_scores[3]),
            KeyCode::Char('5') => self.assign_array_value(4, self.rolled_scores[4]),
            KeyCode::Char('6') => self.assign_array_value(5, self.rolled_scores[5]),
            KeyCode::Char('r') => {
                // Re-roll all scores
                self.rolled_scores = dnd_core::character_builder::roll_ability_scores();
                self.ability_assignment = [None; 6];
                self.ability_scores = AbilityScores::default();
            }
            KeyCode::Enter => {
                if self.ability_assignment.iter().all(|a| a.is_some()) {
                    self.advance_step();
                }
            }
            KeyCode::Esc => {
                if let Some(prev) = self.step.prev() {
                    self.step = prev;
                    self.ability_assignment = [None; 6];
                    self.list_state.select(Some(0));
                }
            }
            _ => {}
        }
    }

    fn assign_array_value(&mut self, array_index: usize, value: u8) {
        let abilities = Ability::all();
        let target_ability = abilities[self.assignment_index];

        // Check if this value is already assigned to another ability
        if self.ability_assignment[array_index].is_some() {
            return;
        }

        // Clear any previous assignment to this ability
        for i in 0..6 {
            if self.ability_assignment[i] == Some(target_ability) {
                self.ability_assignment[i] = None;
            }
        }

        // Assign
        self.ability_assignment[array_index] = Some(target_ability);
        self.ability_scores.set(target_ability, value);

        // Move to next unassigned ability
        for i in 0..6 {
            let ability = abilities[i];
            let already_assigned = self.ability_assignment.contains(&Some(ability));
            if !already_assigned {
                self.assignment_index = i;
                break;
            }
        }
    }

    fn handle_skill_selection(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                let i = self.list_state.selected().unwrap_or(0);
                let max = self.available_skills.len();
                self.list_state
                    .select(Some(if i == 0 { max.saturating_sub(1) } else { i - 1 }));
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let i = self.list_state.selected().unwrap_or(0);
                self.list_state
                    .select(Some((i + 1) % self.available_skills.len().max(1)));
            }
            KeyCode::Char(' ') | KeyCode::Enter => {
                if let Some(i) = self.list_state.selected() {
                    if i < self.available_skills.len() {
                        let skill = self.available_skills[i];
                        if self.selected_skills.contains(&skill) {
                            self.selected_skills.retain(|s| *s != skill);
                        } else if self.selected_skills.len() < self.required_skill_count {
                            self.selected_skills.push(skill);
                        }
                    }
                }

                // Auto-advance if we have enough skills
                if key.code == KeyCode::Enter
                    && self.selected_skills.len() == self.required_skill_count
                {
                    self.advance_step();
                }
            }
            KeyCode::Tab => {
                if self.selected_skills.len() == self.required_skill_count {
                    self.advance_step();
                }
            }
            KeyCode::Esc => {
                if let Some(prev) = self.step.prev() {
                    self.step = prev;
                    self.selected_skills.clear();
                    self.list_state.select(Some(0));
                }
            }
            _ => {}
        }
    }

    fn handle_review(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Enter | KeyCode::Char('y') => {
                self.finished = true;
            }
            KeyCode::Esc | KeyCode::Char('n') => {
                if let Some(prev) = self.step.prev() {
                    self.step = prev;
                    self.list_state.select(Some(0));
                }
            }
            _ => {}
        }
    }

    fn confirm_selection(&mut self) {
        match self.step {
            CreationStep::Race => {
                if let Some(i) = self.list_state.selected() {
                    self.race = Some(RaceType::all()[i]);
                    self.advance_step();
                }
            }
            CreationStep::Class => {
                if let Some(i) = self.list_state.selected() {
                    self.class = Some(CharacterClass::all()[i]);
                    self.advance_step();
                }
            }
            CreationStep::Background => {
                if let Some(i) = self.list_state.selected() {
                    self.background = Some(Background::all()[i]);
                    self.advance_step();
                }
            }
            CreationStep::AbilityMethod => {
                if let Some(i) = self.list_state.selected() {
                    self.ability_method = AbilityMethod::all()[i];

                    // Initialize state based on method
                    match self.ability_method {
                        AbilityMethod::StandardArray => {
                            self.ability_scores = AbilityScores::default();
                            self.ability_assignment = [None; 6];
                        }
                        AbilityMethod::PointBuy => {
                            self.ability_scores = AbilityScores::new(8, 8, 8, 8, 8, 8);
                            self.point_buy_points = 27;
                        }
                        AbilityMethod::Rolled => {
                            // Roll 6 scores
                            self.rolled_scores = dnd_core::character_builder::roll_ability_scores();
                            self.ability_scores = AbilityScores::default();
                            self.ability_assignment = [None; 6];
                        }
                    }
                    self.assignment_index = 0;
                    self.advance_step();
                }
            }
            _ => {}
        }
    }

    fn advance_step(&mut self) {
        if let Some(next) = self.step.next() {
            self.step = next;
            self.list_state.select(Some(0));

            // Set up skills when entering skill selection
            if next == CreationStep::Skills {
                if let Some(class) = self.class {
                    let data = class.data();
                    self.available_skills = data.skill_options.to_vec();
                    self.required_skill_count = data.skill_count;
                    self.selected_skills.clear();
                }
            }
        }
    }

    /// Build the character from current selections.
    pub fn build_character(&self) -> Result<dnd_core::world::Character, String> {
        let mut builder = CharacterBuilder::new()
            .name(&self.name)
            .race(self.race.ok_or("No race selected")?)
            .class(self.class.ok_or("No class selected")?)
            .background(self.background.ok_or("No background selected")?)
            .ability_scores(self.ability_scores.clone())
            .skills(self.selected_skills.clone());

        // Handle Half-Elf bonus abilities (simplified: +1 to STR and CON)
        if self.race == Some(RaceType::HalfElf) {
            builder = builder.half_elf_bonuses([Ability::Strength, Ability::Constitution]);
        }

        builder.build().map_err(|e| e.to_string())
    }

    /// Build a preview character from current selections.
    pub fn build_preview_character(&self) -> Character {
        let name = if self.name.is_empty() {
            "???".to_string()
        } else {
            self.name.clone()
        };

        let mut character = Character::new(&name);

        // Apply race
        if let Some(race) = self.race {
            character.race = Race {
                name: race.name().to_string(),
                subrace: None,
                race_type: Some(race),
            };
            character.race_type = race;
            character.speed = Speed::new(race.base_speed());
        }

        // Apply class
        if let Some(class) = self.class {
            character.classes = vec![ClassLevel {
                class,
                level: 1,
                subclass: None,
            }];

            let data = class.data();

            // Set saving throws
            character.saving_throw_proficiencies = HashSet::new();
            for ability in data.saving_throws {
                character.saving_throw_proficiencies.insert(ability);
            }

            // Set features
            character.features = data.level_1_features;
        }

        // Apply background
        if let Some(bg) = self.background {
            character.background = bg;
            character.background_name = bg.name().to_string();

            // Add background skills
            for skill in bg.skill_proficiencies() {
                character
                    .skill_proficiencies
                    .entry(skill)
                    .or_insert(ProficiencyLevel::Proficient);
            }
        }

        // Apply ability scores with racial bonuses
        let mut scores = self.ability_scores.clone();
        if let Some(race) = self.race {
            race.apply_ability_bonuses(&mut scores);
        }
        character.ability_scores = scores;

        // Calculate HP
        if let Some(class) = self.class {
            let base_hp = class.data().base_hp;
            let con_mod = character.ability_scores.modifier(Ability::Constitution);
            let hp = (base_hp + con_mod as i32).max(1);
            character.hit_points = HitPoints::new(hp);
        }

        // Apply selected skills
        for skill in &self.selected_skills {
            character
                .skill_proficiencies
                .insert(*skill, ProficiencyLevel::Proficient);
        }

        character
    }

    /// Render the character creation UI.
    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        // Clear background
        frame.render_widget(Clear, area);

        // For review step, show full-width character sheet
        if self.step == CreationStep::Review {
            self.render_review(frame, area);
            return;
        }

        // Two-column layout: left for creation steps, right for preview
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
            .split(area);

        // Left side: current step
        let left_block = Block::default()
            .title(format!(" {} ", self.step.title()))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let left_inner = left_block.inner(columns[0]);
        frame.render_widget(left_block, columns[0]);

        match self.step {
            CreationStep::Name => self.render_name_input(frame, left_inner),
            CreationStep::Race => self.render_race_selection(frame, left_inner),
            CreationStep::Class => self.render_class_selection(frame, left_inner),
            CreationStep::Background => self.render_background_selection(frame, left_inner),
            CreationStep::AbilityMethod => self.render_ability_method(frame, left_inner),
            CreationStep::AbilityScores => self.render_ability_scores(frame, left_inner),
            CreationStep::Skills => self.render_skill_selection(frame, left_inner),
            CreationStep::Review => {} // Handled above
        }

        // Right side: character preview
        self.render_preview(frame, columns[1]);
    }

    /// Render the character preview panel.
    fn render_preview(&self, frame: &mut Frame, area: Rect) {
        let character = self.build_preview_character();

        let block = Block::default()
            .title(format!(" {} ", if self.name.is_empty() { "Preview" } else { &self.name }))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Split into sections
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Class/Level
                Constraint::Length(2), // HP bar
                Constraint::Length(2), // AC/Init/Speed
                Constraint::Length(6), // Ability scores
                Constraint::Min(0),    // Skills/Features
            ])
            .split(inner);

        // Class and level
        let class_text = if let Some(class) = character.classes.first() {
            format!("Level {} {}", class.level, class.class)
        } else {
            "No class selected".to_string()
        };
        let class_line = Line::from(Span::styled(
            class_text,
            Style::default().add_modifier(Modifier::DIM),
        ));
        frame.render_widget(Paragraph::new(class_line), chunks[0]);

        // HP bar
        let hp = &character.hit_points;
        let hp_ratio = hp.ratio();
        let hp_color = if hp_ratio > 0.5 {
            Color::Green
        } else if hp_ratio > 0.25 {
            Color::Yellow
        } else {
            Color::Red
        };

        let hp_label = format!("HP: {}/{}", hp.current, hp.maximum);
        let gauge = Gauge::default()
            .block(Block::default())
            .gauge_style(Style::default().fg(hp_color))
            .ratio(hp_ratio as f64)
            .label(hp_label);
        frame.render_widget(gauge, chunks[1]);

        // AC, Initiative, Speed
        let ac = character.current_ac();
        let init = character.initiative_modifier();
        let speed = character.speed.walk;

        let init_str = if init >= 0 {
            format!("+{init}")
        } else {
            format!("{init}")
        };

        let combat_stats = vec![
            Line::from(vec![
                Span::raw("AC: "),
                Span::styled(format!("{ac}"), Style::default().add_modifier(Modifier::BOLD)),
                Span::raw("  Init: "),
                Span::styled(init_str, Style::default()),
            ]),
            Line::from(vec![
                Span::raw("Speed: "),
                Span::styled(format!("{speed} ft"), Style::default()),
            ]),
        ];
        frame.render_widget(Paragraph::new(combat_stats), chunks[2]);

        // Ability scores
        let abilities = [
            (Ability::Strength, "STR"),
            (Ability::Dexterity, "DEX"),
            (Ability::Constitution, "CON"),
            (Ability::Intelligence, "INT"),
            (Ability::Wisdom, "WIS"),
            (Ability::Charisma, "CHA"),
        ];

        let ability_lines: Vec<Line> = abilities
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
            .collect();
        frame.render_widget(Paragraph::new(ability_lines), chunks[3]);

        // Skills and features
        if chunks[4].height > 0 {
            let mut lines = Vec::new();

            // Show skill proficiencies
            if !character.skill_proficiencies.is_empty() {
                lines.push(Line::from(Span::styled(
                    "Skills:",
                    Style::default().add_modifier(Modifier::BOLD),
                )));
                let mut skills: Vec<&str> = character
                    .skill_proficiencies
                    .keys()
                    .map(|s| s.name())
                    .collect();
                skills.sort(); // Stable order
                for skill in skills.iter().take(4) {
                    lines.push(Line::from(format!("  {skill}")));
                }
                if skills.len() > 4 {
                    lines.push(Line::from(format!("  +{} more", skills.len() - 4)));
                }
            }

            frame.render_widget(Paragraph::new(lines), chunks[4]);
        }
    }

    fn render_name_input(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .split(area);

        let prompt = Paragraph::new("What is your character's name?")
            .style(Style::default().fg(Color::Yellow));
        frame.render_widget(prompt, chunks[0]);

        let input = Paragraph::new(format!("{}█", self.name))
            .block(Block::default().borders(Borders::ALL).title(" Name "))
            .style(Style::default().fg(Color::White));
        frame.render_widget(input, chunks[1]);

        let help = Paragraph::new("Press Enter to continue, Esc to cancel")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(help, chunks[2]);
    }

    fn render_race_selection(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        // Race list
        let items: Vec<ListItem> = RaceType::all()
            .iter()
            .map(|r| {
                ListItem::new(format!("{} ({})", r.name(), r.ability_bonuses()))
                    .style(Style::default().fg(Color::White))
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(" Races "))
            .highlight_style(Style::default().bg(Color::Blue).fg(Color::White))
            .highlight_symbol("> ");

        frame.render_stateful_widget(list, chunks[0], &mut self.list_state);

        // Description
        if let Some(i) = self.list_state.selected() {
            let race = RaceType::all()[i];
            let desc = Paragraph::new(format!(
                "{}\n\nAbility Bonuses: {}\nSpeed: {} ft",
                race.description(),
                race.ability_bonuses(),
                race.base_speed()
            ))
            .block(Block::default().borders(Borders::ALL).title(" Description "))
            .wrap(Wrap { trim: true });
            frame.render_widget(desc, chunks[1]);
        }
    }

    fn render_class_selection(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        let items: Vec<ListItem> = CharacterClass::all()
            .iter()
            .map(|c| {
                ListItem::new(format!("{} (d{})", c.name(), c.hit_die().sides()))
                    .style(Style::default().fg(Color::White))
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(" Classes "))
            .highlight_style(Style::default().bg(Color::Blue).fg(Color::White))
            .highlight_symbol("> ");

        frame.render_stateful_widget(list, chunks[0], &mut self.list_state);

        if let Some(i) = self.list_state.selected() {
            let class = CharacterClass::all()[i];
            let data = class.data();
            let saves: Vec<&str> = data.saving_throws.iter().map(|a| a.abbreviation()).collect();
            let desc = Paragraph::new(format!(
                "{}\n\nHit Die: d{}\nSaving Throws: {}\nSkills: {} from {} options",
                class.description(),
                class.hit_die().sides(),
                saves.join(", "),
                data.skill_count,
                data.skill_options.len()
            ))
            .block(Block::default().borders(Borders::ALL).title(" Description "))
            .wrap(Wrap { trim: true });
            frame.render_widget(desc, chunks[1]);
        }
    }

    fn render_background_selection(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        let items: Vec<ListItem> = Background::all()
            .iter()
            .map(|b| ListItem::new(b.name()).style(Style::default().fg(Color::White)))
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Backgrounds "),
            )
            .highlight_style(Style::default().bg(Color::Blue).fg(Color::White))
            .highlight_symbol("> ");

        frame.render_stateful_widget(list, chunks[0], &mut self.list_state);

        if let Some(i) = self.list_state.selected() {
            let bg = Background::all()[i];
            let skills = bg.skill_proficiencies();
            let desc = Paragraph::new(format!(
                "{}\n\nSkill Proficiencies:\n  {}, {}",
                bg.description(),
                skills[0].name(),
                skills[1].name()
            ))
            .block(Block::default().borders(Borders::ALL).title(" Description "))
            .wrap(Wrap { trim: true });
            frame.render_widget(desc, chunks[1]);
        }
    }

    fn render_ability_method(&mut self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = AbilityMethod::all()
            .iter()
            .map(|m| {
                ListItem::new(format!("{}: {}", m.name(), m.description()))
                    .style(Style::default().fg(Color::White))
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Choose Method "),
            )
            .highlight_style(Style::default().bg(Color::Blue).fg(Color::White))
            .highlight_symbol("> ");

        frame.render_stateful_widget(list, area, &mut self.list_state);
    }

    fn render_ability_scores(&self, frame: &mut Frame, area: Rect) {
        match self.ability_method {
            AbilityMethod::StandardArray => self.render_standard_array(frame, area),
            AbilityMethod::PointBuy => self.render_point_buy(frame, area),
            AbilityMethod::Rolled => self.render_rolled(frame, area),
        }
    }

    fn render_standard_array(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(10),
                Constraint::Min(0),
            ])
            .split(area);

        // Instructions
        let instructions = Paragraph::new(
            "Press 1-6 to assign array values to the highlighted ability. Use j/k to navigate.",
        )
        .style(Style::default().fg(Color::Yellow))
        .wrap(Wrap { trim: true });
        frame.render_widget(instructions, chunks[0]);

        // Abilities and assignments
        let abilities = Ability::all();
        let standard_array = [15u8, 14, 13, 12, 10, 8];

        let mut lines = Vec::new();
        for (i, ability) in abilities.iter().enumerate() {
            let value = self.ability_scores.get(*ability);
            let marker = if i == self.assignment_index {
                "> "
            } else {
                "  "
            };
            let assigned = if self.ability_assignment.contains(&Some(*ability)) {
                format!("{value}")
            } else {
                "___".to_string()
            };
            lines.push(format!(
                "{}{}: {}",
                marker,
                ability.abbreviation(),
                assigned
            ));
        }

        // Show array values
        lines.push(String::new());
        lines.push("Available values:".to_string());
        let available: Vec<String> = standard_array
            .iter()
            .enumerate()
            .map(|(i, v)| {
                if self.ability_assignment[i].is_some() {
                    format!("[{v}] (used)")
                } else {
                    format!("[{}] press {}", v, i + 1)
                }
            })
            .collect();
        lines.push(available.join("  "));

        let abilities_text = Paragraph::new(lines.join("\n"))
            .block(Block::default().borders(Borders::ALL).title(" Standard Array "));
        frame.render_widget(abilities_text, chunks[1]);

        // Help
        let all_assigned = self.ability_assignment.iter().all(|a| a.is_some());
        let help_text = if all_assigned {
            "Press Enter to continue"
        } else {
            "Assign all abilities to continue"
        };
        let help = Paragraph::new(help_text).style(Style::default().fg(Color::DarkGray));
        frame.render_widget(help, chunks[2]);
    }

    fn render_point_buy(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(10),
                Constraint::Min(0),
            ])
            .split(area);

        // Instructions
        let instructions = Paragraph::new(format!(
            "Points remaining: {}. Use ←/→ or h/l to adjust scores. j/k to navigate.",
            self.point_buy_points
        ))
        .style(Style::default().fg(Color::Yellow))
        .wrap(Wrap { trim: true });
        frame.render_widget(instructions, chunks[0]);

        // Abilities with current scores and costs
        let abilities = Ability::all();
        let mut lines = Vec::new();

        for (i, ability) in abilities.iter().enumerate() {
            let score = self.ability_scores.get(*ability);
            let modifier = self.ability_scores.modifier(*ability);
            let mod_str = if modifier >= 0 {
                format!("+{modifier}")
            } else {
                format!("{modifier}")
            };

            let marker = if i == self.assignment_index {
                "> "
            } else {
                "  "
            };

            // Show cost for current score
            let cost = point_buy_cost(score);
            lines.push(format!(
                "{}{}: {:2} ({})  [cost: {}]",
                marker,
                ability.abbreviation(),
                score,
                mod_str,
                cost
            ));
        }

        // Show cost table
        lines.push(String::new());
        lines.push("Cost table: 8=0, 9=1, 10=2, 11=3, 12=4, 13=5, 14=7, 15=9".to_string());

        let abilities_text = Paragraph::new(lines.join("\n"))
            .block(Block::default().borders(Borders::ALL).title(" Point Buy "));
        frame.render_widget(abilities_text, chunks[1]);

        // Help
        let help = Paragraph::new("Press Enter to continue (you can proceed with any allocation)")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(help, chunks[2]);
    }

    fn render_rolled(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(10),
                Constraint::Min(0),
            ])
            .split(area);

        // Instructions
        let instructions = Paragraph::new(
            "Press 1-6 to assign rolled values. Use j/k to navigate. Press 'r' to re-roll all.",
        )
        .style(Style::default().fg(Color::Yellow))
        .wrap(Wrap { trim: true });
        frame.render_widget(instructions, chunks[0]);

        // Abilities and assignments
        let abilities = Ability::all();

        let mut lines = Vec::new();
        for (i, ability) in abilities.iter().enumerate() {
            let value = self.ability_scores.get(*ability);
            let marker = if i == self.assignment_index {
                "> "
            } else {
                "  "
            };
            let assigned = if self.ability_assignment.contains(&Some(*ability)) {
                format!("{value}")
            } else {
                "___".to_string()
            };
            lines.push(format!(
                "{}{}: {}",
                marker,
                ability.abbreviation(),
                assigned
            ));
        }

        // Show rolled values
        lines.push(String::new());
        lines.push("Rolled values (4d6 drop lowest):".to_string());
        let rolled: Vec<String> = self.rolled_scores
            .iter()
            .enumerate()
            .map(|(i, v)| {
                if self.ability_assignment[i].is_some() {
                    format!("[{v}] (used)")
                } else {
                    format!("[{}] press {}", v, i + 1)
                }
            })
            .collect();
        lines.push(rolled.join("  "));

        let abilities_text = Paragraph::new(lines.join("\n"))
            .block(Block::default().borders(Borders::ALL).title(" Rolled Scores "));
        frame.render_widget(abilities_text, chunks[1]);

        // Help
        let all_assigned = self.ability_assignment.iter().all(|a| a.is_some());
        let help_text = if all_assigned {
            "Press Enter to continue"
        } else {
            "Assign all abilities to continue, or 'r' to re-roll"
        };
        let help = Paragraph::new(help_text).style(Style::default().fg(Color::DarkGray));
        frame.render_widget(help, chunks[2]);
    }

    fn render_skill_selection(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(2), Constraint::Min(0)])
            .split(area);

        let header = Paragraph::new(format!(
            "Select {} skills (Space to toggle, Tab when done) - {}/{} selected",
            self.required_skill_count,
            self.selected_skills.len(),
            self.required_skill_count
        ))
        .style(Style::default().fg(Color::Yellow));
        frame.render_widget(header, chunks[0]);

        let items: Vec<ListItem> = self
            .available_skills
            .iter()
            .map(|s| {
                let selected = self.selected_skills.contains(s);
                let marker = if selected { "[X]" } else { "[ ]" };
                ListItem::new(format!("{} {}", marker, s.name())).style(if selected {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::White)
                })
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(" Skills "))
            .highlight_style(Style::default().bg(Color::Blue))
            .highlight_symbol("> ");

        frame.render_stateful_widget(list, chunks[1], &mut self.list_state);
    }

    fn render_review(&self, frame: &mut Frame, area: Rect) {
        let character = self.build_preview_character();

        // Two-column layout: character sheet on left, confirmation on right
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(area);

        // Left: Full character sheet
        let sheet_block = Block::default()
            .title(format!(" {} ", self.name))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let sheet_inner = sheet_block.inner(columns[0]);
        frame.render_widget(sheet_block, columns[0]);

        let sheet_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Race/Class/Background
                Constraint::Length(3), // HP bar
                Constraint::Length(3), // Combat stats
                Constraint::Length(8), // Ability scores
                Constraint::Min(0),    // Skills and features
            ])
            .split(sheet_inner);

        // Race/Class/Background
        let race_name = self.race.map(|r| r.name()).unwrap_or("None");
        let class_name = self.class.map(|c| c.name()).unwrap_or("None");
        let bg_name = self.background.map(|b| b.name()).unwrap_or("None");

        let identity = vec![
            Line::from(vec![
                Span::styled("Race: ", Style::default().fg(Color::DarkGray)),
                Span::styled(race_name, Style::default().fg(Color::White)),
                Span::raw("  "),
                Span::styled("Class: ", Style::default().fg(Color::DarkGray)),
                Span::styled(class_name, Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("Background: ", Style::default().fg(Color::DarkGray)),
                Span::styled(bg_name, Style::default().fg(Color::White)),
            ]),
        ];
        frame.render_widget(Paragraph::new(identity), sheet_chunks[0]);

        // HP bar
        let hp = &character.hit_points;
        let hp_color = Color::Green;
        let hp_label = format!("HP: {}/{}", hp.current, hp.maximum);
        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title(" Hit Points "))
            .gauge_style(Style::default().fg(hp_color))
            .ratio(1.0)
            .label(hp_label);
        frame.render_widget(gauge, sheet_chunks[1]);

        // Combat stats
        let ac = character.current_ac();
        let init = character.initiative_modifier();
        let speed = character.speed.walk;
        let init_str = if init >= 0 { format!("+{init}") } else { format!("{init}") };

        let combat = vec![
            Line::from(vec![
                Span::styled("AC: ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{ac}"), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                Span::raw("    "),
                Span::styled("Initiative: ", Style::default().fg(Color::DarkGray)),
                Span::styled(init_str, Style::default().fg(Color::White)),
                Span::raw("    "),
                Span::styled("Speed: ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{speed} ft"), Style::default().fg(Color::White)),
            ]),
        ];
        frame.render_widget(
            Paragraph::new(combat).block(Block::default().borders(Borders::ALL).title(" Combat ")),
            sheet_chunks[2],
        );

        // Ability scores in a nice grid
        let abilities = [
            (Ability::Strength, "STR"),
            (Ability::Dexterity, "DEX"),
            (Ability::Constitution, "CON"),
            (Ability::Intelligence, "INT"),
            (Ability::Wisdom, "WIS"),
            (Ability::Charisma, "CHA"),
        ];

        let mut ability_lines = Vec::new();
        for (ability, abbr) in abilities {
            let score = character.ability_scores.get(ability);
            let modifier = character.ability_scores.modifier(ability);
            let mod_str = if modifier >= 0 {
                format!("+{modifier}")
            } else {
                format!("{modifier}")
            };
            ability_lines.push(Line::from(vec![
                Span::styled(format!("{abbr}: "), Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{score:2}"), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                Span::styled(format!(" ({mod_str})"), Style::default().fg(Color::Cyan)),
            ]));
        }
        frame.render_widget(
            Paragraph::new(ability_lines).block(Block::default().borders(Borders::ALL).title(" Abilities ")),
            sheet_chunks[3],
        );

        // Skills
        let mut skill_names: Vec<&str> = character.skill_proficiencies.keys().map(|s| s.name()).collect();
        skill_names.sort(); // Stable order
        let mut skill_lines = Vec::new();
        for chunk in skill_names.chunks(2) {
            let line_text = chunk.join(", ");
            skill_lines.push(Line::from(Span::styled(line_text, Style::default().fg(Color::White))));
        }
        frame.render_widget(
            Paragraph::new(skill_lines).block(Block::default().borders(Borders::ALL).title(" Skills ")),
            sheet_chunks[4],
        );

        // Right: Confirmation panel
        let confirm_block = Block::default()
            .title(" Confirm ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green));

        let confirm_inner = confirm_block.inner(columns[1]);
        frame.render_widget(confirm_block, columns[1]);

        let confirm_text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "Ready to begin your adventure?",
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Enter", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw(" or "),
                Span::styled("Y", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw(" - Create character"),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Esc", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::raw(" or "),
                Span::styled("N", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::raw(" - Go back"),
            ]),
        ];
        frame.render_widget(Paragraph::new(confirm_text), confirm_inner);
    }
}
