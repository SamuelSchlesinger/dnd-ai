//! Dice rolling animation.
//!
//! Provides visual feedback for dice rolls with tumbling animation
//! and final result display.

use bevy::prelude::*;
use rand::Rng;

use super::AnimationLifetime;

/// Component for dice animation state.
#[derive(Component)]
#[allow(dead_code)]
pub struct DiceAnimation {
    /// The final result to display.
    pub result: i32,
    /// Current displayed value (animating).
    pub current_display: i32,
    /// Time remaining for tumble animation.
    pub tumble_time: f32,
    /// Total animation duration.
    pub total_duration: f32,
    /// Time since animation started.
    pub elapsed: f32,
    /// Dice type (d20, d6, etc).
    pub dice_type: DiceType,
    /// Purpose text for the roll.
    pub purpose: String,
    /// Position for rendering.
    pub position: Vec2,
    /// Whether this is a critical (nat 20) or fumble (nat 1).
    pub is_critical: bool,
    pub is_fumble: bool,
}

/// Types of dice for visual differentiation.
#[derive(Clone, Copy, Debug)]
pub enum DiceType {
    D4,
    D6,
    D8,
    D10,
    D12,
    D20,
    D100,
}

#[allow(dead_code)]
impl DiceType {
    /// Get the maximum value for this die.
    pub fn max_value(&self) -> i32 {
        match self {
            DiceType::D4 => 4,
            DiceType::D6 => 6,
            DiceType::D8 => 8,
            DiceType::D10 => 10,
            DiceType::D12 => 12,
            DiceType::D20 => 20,
            DiceType::D100 => 100,
        }
    }

    /// Get the color for this die type.
    pub fn color(&self) -> Color {
        match self {
            DiceType::D4 => Color::srgb(0.8, 0.2, 0.2),   // Red
            DiceType::D6 => Color::srgb(0.2, 0.8, 0.2),   // Green
            DiceType::D8 => Color::srgb(0.2, 0.2, 0.8),   // Blue
            DiceType::D10 => Color::srgb(0.8, 0.8, 0.2),  // Yellow
            DiceType::D12 => Color::srgb(0.8, 0.2, 0.8),  // Purple
            DiceType::D20 => Color::srgb(0.9, 0.6, 0.1),  // Gold
            DiceType::D100 => Color::srgb(0.5, 0.5, 0.5), // Gray
        }
    }
}

/// Spawn a new dice animation.
pub fn spawn_dice_animation(
    commands: &mut Commands,
    result: i32,
    dice_type: DiceType,
    purpose: String,
    position: Vec2,
) {
    let is_d20 = matches!(dice_type, DiceType::D20);
    let is_critical = is_d20 && result == 20;
    let is_fumble = is_d20 && result == 1;

    commands.spawn((
        DiceAnimation {
            result,
            current_display: 1,
            tumble_time: 0.8,
            total_duration: 1.5,
            elapsed: 0.0,
            dice_type,
            purpose,
            position,
            is_critical,
            is_fumble,
        },
        AnimationLifetime { remaining: 2.5 },
        Transform::from_translation(position.extend(10.0)),
        Visibility::default(),
    ));
}

/// System to animate dice rolls.
pub fn animate_dice(time: Res<Time>, mut query: Query<&mut DiceAnimation>) {
    let mut rng = rand::thread_rng();

    for mut dice in query.iter_mut() {
        dice.elapsed += time.delta_secs();

        // During tumble phase, randomize the display
        if dice.elapsed < dice.tumble_time {
            // Gradually slow down the tumble
            let progress = dice.elapsed / dice.tumble_time;
            let change_rate = 0.05 + (1.0 - progress) * 0.1;

            if rng.gen::<f32>() < change_rate {
                dice.current_display = rng.gen_range(1..=dice.dice_type.max_value());
            }
        } else {
            // Show final result
            dice.current_display = dice.result;
        }
    }
}

/// Parse a dice notation string to determine the primary die type.
pub fn parse_dice_type(notation: &str) -> DiceType {
    let notation = notation.to_lowercase();
    if notation.contains("d20") {
        DiceType::D20
    } else if notation.contains("d12") {
        DiceType::D12
    } else if notation.contains("d10") || notation.contains("d100") {
        if notation.contains("d100") {
            DiceType::D100
        } else {
            DiceType::D10
        }
    } else if notation.contains("d8") {
        DiceType::D8
    } else if notation.contains("d6") {
        DiceType::D6
    } else if notation.contains("d4") {
        DiceType::D4
    } else {
        DiceType::D20 // Default
    }
}
