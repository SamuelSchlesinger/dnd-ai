//! Character creation flow for Bevy GUI.
//!
//! Adapts the TUI character creation wizard to egui.

use bevy::prelude::*;
use bevy_egui::egui;
use dnd_core::spells::{spells_by_level, SpellClass};
use dnd_core::world::{
    Ability, AbilityScores, Background, Character, CharacterClass, ClassLevel, HitPoints,
    ProficiencyLevel, Race, RaceType, Skill, SlotInfo, Speed, SpellSlots, SpellcastingData,
};
use dnd_core::{AbilityMethod, CharacterBuilder};
use std::collections::HashSet;

use crate::state::{AppState, GamePhase};

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

/// Character creation step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CreationStep {
    #[default]
    Name,
    Race,
    Class,
    Background,
    AbilityMethod,
    AbilityScores,
    Skills,
    Spells,
    Backstory,
    Review,
}

#[allow(dead_code)]
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
            CreationStep::Spells => "Choose Spells",
            CreationStep::Backstory => "Write Your Backstory",
            CreationStep::Review => "Review Character",
        }
    }

    /// Get next step, considering whether the class is a spellcaster.
    pub fn next_for_class(&self, class: Option<CharacterClass>) -> Option<CreationStep> {
        match self {
            CreationStep::Name => Some(CreationStep::Race),
            CreationStep::Race => Some(CreationStep::Class),
            CreationStep::Class => Some(CreationStep::Background),
            CreationStep::Background => Some(CreationStep::AbilityMethod),
            CreationStep::AbilityMethod => Some(CreationStep::AbilityScores),
            CreationStep::AbilityScores => Some(CreationStep::Skills),
            CreationStep::Skills => {
                // Only show Spells step for spellcasting classes
                if class.map(|c| c.is_spellcaster()).unwrap_or(false) {
                    Some(CreationStep::Spells)
                } else {
                    Some(CreationStep::Backstory)
                }
            }
            CreationStep::Spells => Some(CreationStep::Backstory),
            CreationStep::Backstory => Some(CreationStep::Review),
            CreationStep::Review => None,
        }
    }

    /// Get previous step, considering whether the class is a spellcaster.
    pub fn prev_for_class(&self, class: Option<CharacterClass>) -> Option<CreationStep> {
        match self {
            CreationStep::Name => None,
            CreationStep::Race => Some(CreationStep::Name),
            CreationStep::Class => Some(CreationStep::Race),
            CreationStep::Background => Some(CreationStep::Class),
            CreationStep::AbilityMethod => Some(CreationStep::Background),
            CreationStep::AbilityScores => Some(CreationStep::AbilityMethod),
            CreationStep::Skills => Some(CreationStep::AbilityScores),
            CreationStep::Spells => Some(CreationStep::Skills),
            CreationStep::Backstory => {
                // Only show Spells step for spellcasting classes
                if class.map(|c| c.is_spellcaster()).unwrap_or(false) {
                    Some(CreationStep::Spells)
                } else {
                    Some(CreationStep::Skills)
                }
            }
            CreationStep::Review => Some(CreationStep::Backstory),
        }
    }

    pub fn next(&self) -> Option<CreationStep> {
        self.next_for_class(None)
    }

    pub fn prev(&self) -> Option<CreationStep> {
        self.prev_for_class(None)
    }
}

/// Holds a character ready to start the game with.
#[derive(Resource)]
pub struct ReadyToStart {
    pub character: Character,
    pub campaign_name: String,
}

/// Character creation state resource.
#[derive(Resource, Default)]
pub struct CharacterCreation {
    pub step: CreationStep,
    pub name: String,
    pub race: Option<RaceType>,
    pub class: Option<CharacterClass>,
    pub background: Option<Background>,
    pub ability_method: AbilityMethod,
    pub ability_scores: AbilityScores,
    pub ability_assignment: [Option<Ability>; 6],
    pub rolled_scores: [u8; 6],
    pub point_buy_points: u8,
    pub selected_skills: Vec<Skill>,
    pub available_skills: Vec<Skill>,
    pub required_skill_count: usize,
    // Spell selection
    pub selected_cantrips: Vec<String>,
    pub selected_spells: Vec<String>,
    pub available_cantrips: Vec<String>,
    pub available_spells: Vec<String>,
    pub required_cantrip_count: usize,
    pub required_spell_count: usize,
    pub backstory: String,
    pub error_message: Option<String>,
    pub save_message: Option<String>,
}

impl CharacterCreation {
    pub fn new() -> Self {
        Self {
            point_buy_points: 27,
            ability_scores: AbilityScores::new(10, 10, 10, 10, 10, 10),
            ..Default::default()
        }
    }

    /// Build the character from current selections.
    pub fn build_character(&self) -> Result<Character, String> {
        let mut builder = CharacterBuilder::new()
            .name(&self.name)
            .race(self.race.ok_or("No race selected")?)
            .class(self.class.ok_or("No class selected")?)
            .background(self.background.ok_or("No background selected")?)
            .ability_scores(self.ability_scores.clone())
            .skills(self.selected_skills.clone());

        // Handle Half-Elf bonus abilities
        if self.race == Some(RaceType::HalfElf) {
            builder = builder.half_elf_bonuses([Ability::Strength, Ability::Constitution]);
        }

        // Add backstory if provided
        if !self.backstory.trim().is_empty() {
            builder = builder.backstory(&self.backstory);
        }

        let mut character = builder.build().map_err(|e| e.to_string())?;

        // Add spellcasting if class is a spellcaster
        if let Some(class) = self.class {
            if class.is_spellcaster() {
                if let Some(ability) = class.spellcasting_ability() {
                    character.spellcasting = Some(SpellcastingData {
                        ability,
                        spells_known: self.selected_spells.clone(),
                        spells_prepared: self.selected_spells.clone(), // For simplicity, prepared = known at level 1
                        cantrips_known: self.selected_cantrips.clone(),
                        spell_slots: create_level_1_spell_slots(class),
                    });
                }
            }
        }

        Ok(character)
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
            character.saving_throw_proficiencies = HashSet::new();
            for ability in data.saving_throws {
                character.saving_throw_proficiencies.insert(ability);
            }
            character.features = data.level_1_features;
        }

        // Apply background
        if let Some(bg) = self.background {
            character.background = bg;
            character.background_name = bg.name().to_string();
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

        // Apply spellcasting if selected
        if let Some(class) = self.class {
            if class.is_spellcaster() {
                if let Some(ability) = class.spellcasting_ability() {
                    character.spellcasting = Some(SpellcastingData {
                        ability,
                        spells_known: self.selected_spells.clone(),
                        spells_prepared: self.selected_spells.clone(),
                        cantrips_known: self.selected_cantrips.clone(),
                        spell_slots: create_level_1_spell_slots(class),
                    });
                }
            }
        }

        character
    }
}

/// Create level 1 spell slots for a given class.
fn create_level_1_spell_slots(class: CharacterClass) -> SpellSlots {
    let first_level_slots = match class {
        CharacterClass::Bard
        | CharacterClass::Cleric
        | CharacterClass::Druid
        | CharacterClass::Sorcerer
        | CharacterClass::Wizard => 2,
        CharacterClass::Warlock => 1, // Pact Magic
        _ => 0,
    };

    SpellSlots {
        slots: [
            SlotInfo {
                total: first_level_slots,
                used: 0,
            },
            SlotInfo { total: 0, used: 0 },
            SlotInfo { total: 0, used: 0 },
            SlotInfo { total: 0, used: 0 },
            SlotInfo { total: 0, used: 0 },
            SlotInfo { total: 0, used: 0 },
            SlotInfo { total: 0, used: 0 },
            SlotInfo { total: 0, used: 0 },
            SlotInfo { total: 0, used: 0 },
        ],
    }
}

/// Render the character creation UI.
pub fn render_character_creation(
    ctx: &egui::Context,
    creation: &mut CharacterCreation,
    next_phase: &mut NextState<GamePhase>,
    app_state: &mut AppState,
    commands: &mut Commands,
) {
    egui::CentralPanel::default().show(ctx, |ui| {
        // Progress bar - conditionally include Spells step for spellcasters
        ui.horizontal(|ui| {
            let is_spellcaster = creation.class.map(|c| c.is_spellcaster()).unwrap_or(false);
            let mut steps: Vec<CreationStep> = vec![
                CreationStep::Name,
                CreationStep::Race,
                CreationStep::Class,
                CreationStep::Background,
                CreationStep::AbilityMethod,
                CreationStep::AbilityScores,
                CreationStep::Skills,
            ];
            if is_spellcaster {
                steps.push(CreationStep::Spells);
            }
            steps.push(CreationStep::Backstory);
            steps.push(CreationStep::Review);

            for (i, step) in steps.iter().enumerate() {
                let is_current = *step == creation.step;
                let is_completed = steps.iter().position(|s| *s == creation.step).unwrap_or(0) > i;

                let color = if is_current {
                    egui::Color32::from_rgb(218, 165, 32) // Gold
                } else if is_completed {
                    egui::Color32::from_rgb(34, 139, 34) // Forest green
                } else {
                    egui::Color32::DARK_GRAY
                };

                ui.label(egui::RichText::new(format!("{}", i + 1)).color(color));
                if i < steps.len() - 1 {
                    ui.label(egui::RichText::new("-").color(color));
                }
            }
        });

        ui.separator();

        // Step title
        ui.heading(creation.step.title());
        ui.add_space(10.0);

        // Two-column layout for most steps
        ui.columns(2, |columns| {
            // Left column: current step content
            match creation.step {
                CreationStep::Name => render_name_step(&mut columns[0], creation),
                CreationStep::Race => render_race_step(&mut columns[0], creation),
                CreationStep::Class => render_class_step(&mut columns[0], creation),
                CreationStep::Background => render_background_step(&mut columns[0], creation),
                CreationStep::AbilityMethod => {
                    render_ability_method_step(&mut columns[0], creation)
                }
                CreationStep::AbilityScores => {
                    render_ability_scores_step(&mut columns[0], creation)
                }
                CreationStep::Skills => render_skills_step(&mut columns[0], creation),
                CreationStep::Spells => render_spells_step(&mut columns[0], creation),
                CreationStep::Backstory => render_backstory_step(&mut columns[0], creation),
                CreationStep::Review => {
                    render_review_step(&mut columns[0], creation, next_phase, app_state, commands)
                }
            }

            // Right column: character preview
            render_preview(&mut columns[1], creation);
        });

        // Error message
        if let Some(ref msg) = creation.error_message {
            ui.separator();
            ui.colored_label(egui::Color32::RED, msg);
        }

        // Navigation buttons
        ui.separator();
        ui.horizontal(|ui| {
            if creation.step != CreationStep::Name && ui.button("< Back").clicked() {
                if let Some(prev) = creation.step.prev() {
                    creation.step = prev;
                    creation.error_message = None;
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Cancel").clicked() {
                    next_phase.set(GamePhase::MainMenu);
                }
            });
        });
    });
}

fn render_name_step(ui: &mut egui::Ui, creation: &mut CharacterCreation) {
    ui.label("What is your character's name?");
    ui.add_space(10.0);

    let response = ui.add(
        egui::TextEdit::singleline(&mut creation.name)
            .hint_text("Enter name...")
            .desired_width(300.0),
    );

    if response.lost_focus()
        && ui.input(|i| i.key_pressed(egui::Key::Enter))
        && !creation.name.trim().is_empty()
    {
        creation.step = CreationStep::Race;
    }

    ui.add_space(10.0);
    if !creation.name.trim().is_empty() && ui.button("Next >").clicked() {
        creation.step = CreationStep::Race;
    }
}

fn render_race_step(ui: &mut egui::Ui, creation: &mut CharacterCreation) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        for race in RaceType::all() {
            let is_selected = creation.race == Some(*race);
            let response = ui.selectable_label(
                is_selected,
                format!("{} ({})", race.name(), race.ability_bonuses()),
            );

            if response.clicked() {
                creation.race = Some(*race);
            }

            if response.double_clicked() && creation.race.is_some() {
                creation.step = CreationStep::Class;
            }
        }
    });

    ui.add_space(10.0);
    if creation.race.is_some() && ui.button("Next >").clicked() {
        creation.step = CreationStep::Class;
    }
}

fn render_class_step(ui: &mut egui::Ui, creation: &mut CharacterCreation) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        for class in CharacterClass::all() {
            let is_selected = creation.class == Some(*class);
            let response = ui.selectable_label(
                is_selected,
                format!("{} (d{})", class.name(), class.hit_die().sides()),
            );

            if response.clicked() {
                creation.class = Some(*class);
            }

            if response.double_clicked() && creation.class.is_some() {
                creation.step = CreationStep::Background;
            }
        }
    });

    ui.add_space(10.0);
    if creation.class.is_some() && ui.button("Next >").clicked() {
        creation.step = CreationStep::Background;
    }
}

fn render_background_step(ui: &mut egui::Ui, creation: &mut CharacterCreation) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        for bg in Background::all() {
            let is_selected = creation.background == Some(*bg);
            let response = ui.selectable_label(is_selected, bg.name());

            if response.clicked() {
                creation.background = Some(*bg);
            }

            if response.double_clicked() && creation.background.is_some() {
                creation.step = CreationStep::AbilityMethod;
            }
        }
    });

    ui.add_space(10.0);
    if creation.background.is_some() && ui.button("Next >").clicked() {
        creation.step = CreationStep::AbilityMethod;
    }
}

fn render_ability_method_step(ui: &mut egui::Ui, creation: &mut CharacterCreation) {
    for method in AbilityMethod::all() {
        let is_selected = creation.ability_method == *method;
        let response = ui.selectable_label(
            is_selected,
            format!("{}: {}", method.name(), method.description()),
        );

        if response.clicked() {
            creation.ability_method = *method;
            // Initialize state based on method
            match *method {
                AbilityMethod::StandardArray => {
                    creation.ability_scores = AbilityScores::default();
                    creation.ability_assignment = [None; 6];
                }
                AbilityMethod::PointBuy => {
                    creation.ability_scores = AbilityScores::new(8, 8, 8, 8, 8, 8);
                    creation.point_buy_points = 27;
                }
                AbilityMethod::Rolled => {
                    creation.rolled_scores = dnd_core::character_builder::roll_ability_scores();
                    creation.ability_scores = AbilityScores::default();
                    creation.ability_assignment = [None; 6];
                }
            }
        }

        if response.double_clicked() {
            creation.step = CreationStep::AbilityScores;
        }
    }

    ui.add_space(10.0);
    if ui.button("Next >").clicked() {
        creation.step = CreationStep::AbilityScores;
    }
}

fn render_ability_scores_step(ui: &mut egui::Ui, creation: &mut CharacterCreation) {
    match creation.ability_method {
        AbilityMethod::StandardArray => render_standard_array(ui, creation),
        AbilityMethod::PointBuy => render_point_buy(ui, creation),
        AbilityMethod::Rolled => render_rolled(ui, creation),
    }

    // Check if complete
    let complete = match creation.ability_method {
        AbilityMethod::StandardArray | AbilityMethod::Rolled => {
            creation.ability_assignment.iter().all(|a| a.is_some())
        }
        AbilityMethod::PointBuy => true, // Always complete
    };

    ui.add_space(10.0);
    if complete && ui.button("Next >").clicked() {
        // Set up skills for next step
        if let Some(class) = creation.class {
            let data = class.data();
            creation.available_skills = data.skill_options.to_vec();
            creation.required_skill_count = data.skill_count;
            creation.selected_skills.clear();
        }
        creation.step = CreationStep::Skills;
    }
}

fn render_standard_array(ui: &mut egui::Ui, creation: &mut CharacterCreation) {
    let standard_array = [15u8, 14, 13, 12, 10, 8];
    let abilities = Ability::all();

    ui.label("Assign values to abilities (click ability, then click a value):");
    ui.add_space(10.0);

    // Show abilities
    for ability in &abilities {
        let current_value = creation.ability_scores.get(*ability);
        let assigned = creation.ability_assignment.contains(&Some(*ability));

        ui.horizontal(|ui| {
            ui.label(format!("{}: ", ability.abbreviation()));
            if assigned {
                ui.label(egui::RichText::new(current_value.to_string()).strong());
            } else {
                ui.label("___");
            }
        });
    }

    ui.add_space(10.0);
    ui.label("Available values:");

    ui.horizontal(|ui| {
        for (i, value) in standard_array.iter().enumerate() {
            let used = creation.ability_assignment[i].is_some();
            if ui
                .add_enabled(!used, egui::Button::new(value.to_string()))
                .clicked()
            {
                // Find first unassigned ability
                for ability in &abilities {
                    if !creation.ability_assignment.contains(&Some(*ability)) {
                        creation.ability_assignment[i] = Some(*ability);
                        creation.ability_scores.set(*ability, *value);
                        break;
                    }
                }
            }
        }
    });

    if ui.button("Reset").clicked() {
        creation.ability_assignment = [None; 6];
        creation.ability_scores = AbilityScores::default();
    }
}

fn render_point_buy(ui: &mut egui::Ui, creation: &mut CharacterCreation) {
    ui.label(format!("Points remaining: {}", creation.point_buy_points));
    ui.add_space(10.0);

    let abilities = Ability::all();

    for ability in &abilities {
        let score = creation.ability_scores.get(*ability);
        let modifier = creation.ability_scores.modifier(*ability);
        let mod_str = if modifier >= 0 {
            format!("+{modifier}")
        } else {
            modifier.to_string()
        };

        ui.horizontal(|ui| {
            ui.label(format!("{}: ", ability.abbreviation()));

            if ui.button("-").clicked() && score > 8 {
                let current_cost = point_buy_cost(score);
                let new_cost = point_buy_cost(score - 1);
                let refund = current_cost - new_cost;
                creation.ability_scores.set(*ability, score - 1);
                creation.point_buy_points += refund;
            }

            ui.label(egui::RichText::new(format!("{score:2}")).strong());

            let can_increase = score < 15 && {
                let current_cost = point_buy_cost(score);
                let new_cost = point_buy_cost(score + 1);
                creation.point_buy_points >= new_cost - current_cost
            };

            if ui
                .add_enabled(can_increase, egui::Button::new("+"))
                .clicked()
            {
                let current_cost = point_buy_cost(score);
                let new_cost = point_buy_cost(score + 1);
                let cost_diff = new_cost - current_cost;
                creation.ability_scores.set(*ability, score + 1);
                creation.point_buy_points -= cost_diff;
            }

            ui.label(format!("({mod_str})"));
        });
    }

    ui.add_space(10.0);
    ui.label(
        egui::RichText::new("Cost: 8=0, 9=1, 10=2, 11=3, 12=4, 13=5, 14=7, 15=9")
            .small()
            .color(egui::Color32::GRAY),
    );
}

fn render_rolled(ui: &mut egui::Ui, creation: &mut CharacterCreation) {
    let abilities = Ability::all();

    ui.label("Assign rolled values to abilities:");
    ui.add_space(10.0);

    // Show abilities
    for ability in &abilities {
        let current_value = creation.ability_scores.get(*ability);
        let assigned = creation.ability_assignment.contains(&Some(*ability));

        ui.horizontal(|ui| {
            ui.label(format!("{}: ", ability.abbreviation()));
            if assigned {
                ui.label(egui::RichText::new(current_value.to_string()).strong());
            } else {
                ui.label("___");
            }
        });
    }

    ui.add_space(10.0);
    ui.label("Rolled values (4d6 drop lowest):");

    ui.horizontal(|ui| {
        for (i, value) in creation.rolled_scores.iter().enumerate() {
            let used = creation.ability_assignment[i].is_some();
            if ui
                .add_enabled(!used, egui::Button::new(value.to_string()))
                .clicked()
            {
                // Find first unassigned ability
                for ability in &abilities {
                    if !creation.ability_assignment.contains(&Some(*ability)) {
                        creation.ability_assignment[i] = Some(*ability);
                        creation.ability_scores.set(*ability, *value);
                        break;
                    }
                }
            }
        }
    });

    ui.horizontal(|ui| {
        if ui.button("Reset").clicked() {
            creation.ability_assignment = [None; 6];
            creation.ability_scores = AbilityScores::default();
        }
        if ui.button("Re-roll").clicked() {
            creation.rolled_scores = dnd_core::character_builder::roll_ability_scores();
            creation.ability_assignment = [None; 6];
            creation.ability_scores = AbilityScores::default();
        }
    });
}

fn render_skills_step(ui: &mut egui::Ui, creation: &mut CharacterCreation) {
    ui.label(format!(
        "Select {} skills ({}/{} selected):",
        creation.required_skill_count,
        creation.selected_skills.len(),
        creation.required_skill_count
    ));
    ui.add_space(10.0);

    egui::ScrollArea::vertical().show(ui, |ui| {
        for skill in &creation.available_skills.clone() {
            let is_selected = creation.selected_skills.contains(skill);
            let can_select =
                is_selected || creation.selected_skills.len() < creation.required_skill_count;

            let text = if is_selected {
                format!("[X] {}", skill.name())
            } else {
                format!("[ ] {}", skill.name())
            };

            if ui
                .add_enabled(can_select, egui::SelectableLabel::new(is_selected, text))
                .clicked()
            {
                if is_selected {
                    creation.selected_skills.retain(|s| s != skill);
                } else {
                    creation.selected_skills.push(*skill);
                }
            }
        }
    });

    ui.add_space(10.0);
    if creation.selected_skills.len() == creation.required_skill_count
        && ui.button("Next >").clicked()
    {
        // Check if class is a spellcaster
        if let Some(class) = creation.class {
            if class.is_spellcaster() {
                // Set up spell selection
                setup_spell_selection(creation, class);
                creation.step = CreationStep::Spells;
            } else {
                creation.step = CreationStep::Backstory;
            }
        } else {
            creation.step = CreationStep::Backstory;
        }
    }
}

/// Set up available spells for selection based on class.
fn setup_spell_selection(creation: &mut CharacterCreation, class: CharacterClass) {
    // Convert CharacterClass to SpellClass
    let spell_class = match class {
        CharacterClass::Bard => SpellClass::Bard,
        CharacterClass::Cleric => SpellClass::Cleric,
        CharacterClass::Druid => SpellClass::Druid,
        CharacterClass::Sorcerer => SpellClass::Sorcerer,
        CharacterClass::Warlock => SpellClass::Warlock,
        CharacterClass::Wizard => SpellClass::Wizard,
        CharacterClass::Paladin => SpellClass::Paladin,
        CharacterClass::Ranger => SpellClass::Ranger,
        _ => return,
    };

    // Get cantrips (level 0) for this class
    creation.available_cantrips = spells_by_level(0)
        .filter(|spell| spell.classes.contains(&spell_class))
        .map(|spell| spell.name.clone())
        .collect();

    // Get 1st level spells for this class
    creation.available_spells = spells_by_level(1)
        .filter(|spell| spell.classes.contains(&spell_class))
        .map(|spell| spell.name.clone())
        .collect();

    // Set required counts
    creation.required_cantrip_count = class.cantrips_known_at_level_1();
    creation.required_spell_count = class.spells_known_at_level_1();

    // Clear previous selections
    creation.selected_cantrips.clear();
    creation.selected_spells.clear();
}

fn render_spells_step(ui: &mut egui::Ui, creation: &mut CharacterCreation) {
    let class_name = creation.class.map(|c| c.name()).unwrap_or("Unknown");

    ui.label(format!("Choose spells for your {} (Level 1):", class_name));
    ui.add_space(10.0);

    // Cantrip selection
    if creation.required_cantrip_count > 0 {
        ui.heading("Cantrips");
        ui.label(format!(
            "Select {} cantrip{} ({} selected):",
            creation.required_cantrip_count,
            if creation.required_cantrip_count == 1 {
                ""
            } else {
                "s"
            },
            creation.selected_cantrips.len()
        ));
        ui.add_space(5.0);

        egui::ScrollArea::vertical()
            .id_salt("cantrips")
            .max_height(150.0)
            .show(ui, |ui| {
                for cantrip in creation.available_cantrips.clone() {
                    let is_selected = creation.selected_cantrips.contains(&cantrip);
                    let can_select = is_selected
                        || creation.selected_cantrips.len() < creation.required_cantrip_count;

                    let label = if is_selected {
                        format!("[X] {}", cantrip)
                    } else {
                        format!("[ ] {}", cantrip)
                    };

                    if can_select {
                        if ui.selectable_label(is_selected, &label).clicked() {
                            if is_selected {
                                creation.selected_cantrips.retain(|s| s != &cantrip);
                            } else {
                                creation.selected_cantrips.push(cantrip);
                            }
                        }
                    } else {
                        ui.add_enabled(false, egui::Label::new(&label));
                    }
                }
            });
        ui.add_space(10.0);
    }

    // Spell selection (for classes that learn specific spells)
    if creation.required_spell_count > 0 {
        ui.heading("1st Level Spells");
        ui.label(format!(
            "Select {} spell{} ({} selected):",
            creation.required_spell_count,
            if creation.required_spell_count == 1 {
                ""
            } else {
                "s"
            },
            creation.selected_spells.len()
        ));
        ui.add_space(5.0);

        egui::ScrollArea::vertical()
            .id_salt("spells")
            .max_height(200.0)
            .show(ui, |ui| {
                for spell in creation.available_spells.clone() {
                    let is_selected = creation.selected_spells.contains(&spell);
                    let can_select = is_selected
                        || creation.selected_spells.len() < creation.required_spell_count;

                    let label = if is_selected {
                        format!("[X] {}", spell)
                    } else {
                        format!("[ ] {}", spell)
                    };

                    if can_select {
                        if ui.selectable_label(is_selected, &label).clicked() {
                            if is_selected {
                                creation.selected_spells.retain(|s| s != &spell);
                            } else {
                                creation.selected_spells.push(spell);
                            }
                        }
                    } else {
                        ui.add_enabled(false, egui::Label::new(&label));
                    }
                }
            });
    } else if creation.class == Some(CharacterClass::Cleric)
        || creation.class == Some(CharacterClass::Druid)
    {
        // Clerics and Druids prepare spells from the entire list
        ui.label(
            egui::RichText::new(
                "As a prepared caster, you can prepare different spells each day from your entire class spell list.",
            )
            .small()
            .color(egui::Color32::GRAY),
        );
    }

    ui.add_space(10.0);

    // Navigation
    let cantrips_complete = creation.selected_cantrips.len() >= creation.required_cantrip_count;
    let spells_complete = creation.selected_spells.len() >= creation.required_spell_count;

    if cantrips_complete && spells_complete && ui.button("Next >").clicked() {
        creation.step = CreationStep::Backstory;
    }
}

fn render_backstory_step(ui: &mut egui::Ui, creation: &mut CharacterCreation) {
    ui.label("Write your character's backstory (optional):");
    ui.add_space(5.0);
    ui.label(
        egui::RichText::new("This helps the DM understand your character's motivations and goals.")
            .small()
            .color(egui::Color32::GRAY),
    );
    ui.add_space(10.0);

    egui::ScrollArea::vertical()
        .max_height(300.0)
        .show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(&mut creation.backstory)
                    .hint_text("Who is your character? Where do they come from? What drives them?")
                    .desired_width(f32::INFINITY)
                    .desired_rows(12),
            );
        });

    ui.add_space(10.0);
    ui.horizontal(|ui| {
        if ui.button("Skip").clicked() {
            creation.backstory.clear();
            creation.step = CreationStep::Review;
        }
        if ui.button("Next >").clicked() {
            creation.step = CreationStep::Review;
        }
    });
}

fn render_review_step(
    ui: &mut egui::Ui,
    creation: &mut CharacterCreation,
    next_phase: &mut NextState<GamePhase>,
    _app_state: &mut AppState,
    commands: &mut Commands,
) {
    ui.label("Review your character and begin your adventure!");
    ui.add_space(10.0);

    // Show backstory if present
    if !creation.backstory.trim().is_empty() {
        ui.collapsing("Backstory", |ui| {
            ui.label(&creation.backstory);
        });
        ui.add_space(10.0);
    }

    // Save message
    if let Some(ref msg) = creation.save_message {
        ui.colored_label(egui::Color32::GREEN, msg);
        ui.add_space(5.0);
    }

    ui.horizontal(|ui| {
        if ui.button("Save Character").clicked() {
            match creation.build_character() {
                Ok(character) => {
                    let saved = dnd_core::SavedCharacter::new(character);
                    let path =
                        dnd_core::persist::character_save_path("saves/characters", &creation.name);

                    // Spawn async save task
                    let path_clone = path.clone();
                    std::thread::spawn(move || {
                        crate::runtime::RUNTIME.block_on(async {
                            let _ = saved.save_json(&path_clone).await;
                        });
                    });

                    creation.save_message = Some(format!("Character saved to {}", path.display()));
                }
                Err(e) => {
                    creation.error_message = Some(format!("Failed to save: {e}"));
                }
            }
        }

        if ui.button("Start Adventure!").clicked() {
            match creation.build_character() {
                Ok(character) => {
                    // Store the character ready to start
                    commands.insert_resource(ReadyToStart {
                        character,
                        campaign_name: "The Dragon's Lair".to_string(),
                    });

                    // Transition to playing - the session will be created by a system
                    next_phase.set(GamePhase::Playing);
                }
                Err(e) => {
                    creation.error_message = Some(format!("Failed to create character: {e}"));
                }
            }
        }
    });
}

fn render_preview(ui: &mut egui::Ui, creation: &CharacterCreation) {
    let character = creation.build_preview_character();

    ui.group(|ui| {
        ui.heading(
            egui::RichText::new(&character.name).color(egui::Color32::from_rgb(218, 165, 32)),
        );

        // Race/Class
        if let Some(class) = character.classes.first() {
            ui.label(format!("Level {} {}", class.level, class.class.name()));
        }
        if let Some(race) = creation.race {
            ui.label(race.name());
        }

        ui.separator();

        // HP
        let hp = &character.hit_points;
        ui.label(format!("HP: {}/{}", hp.current, hp.maximum));

        // Combat stats
        ui.horizontal(|ui| {
            ui.label(format!("AC: {}", character.current_ac()));
            ui.label(format!("Init: {:+}", character.initiative_modifier()));
        });
        ui.label(format!("Speed: {} ft", character.speed.walk));

        ui.separator();

        // Ability scores
        let abilities = [
            (Ability::Strength, "STR"),
            (Ability::Dexterity, "DEX"),
            (Ability::Constitution, "CON"),
            (Ability::Intelligence, "INT"),
            (Ability::Wisdom, "WIS"),
            (Ability::Charisma, "CHA"),
        ];

        for (ability, abbr) in abilities {
            let score = character.ability_scores.get(ability);
            let modifier = character.ability_scores.modifier(ability);
            let mod_str = if modifier >= 0 {
                format!("+{modifier}")
            } else {
                format!("{modifier}")
            };
            ui.horizontal(|ui| {
                ui.label(format!("{abbr}: {score:2} ({mod_str})"));
            });
        }

        // Skills
        if !creation.selected_skills.is_empty() {
            ui.separator();
            ui.label("Skills:");
            for skill in &creation.selected_skills {
                ui.label(format!("  {}", skill.name()));
            }
        }
    });
}
