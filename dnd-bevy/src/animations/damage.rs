//! Floating damage number animations.
//!
//! Numbers that float up and fade out, commonly used to show
//! damage dealt or healing received.

use bevy::prelude::*;

use super::AnimationLifetime;

/// Component for floating damage number.
#[derive(Component)]
#[allow(dead_code)]
pub struct DamageNumber {
    /// The amount to display (can be positive for healing, negative for damage).
    pub amount: i32,
    /// Whether this is healing (positive) or damage (negative display).
    pub is_healing: bool,
    /// Whether this was a critical hit.
    pub is_critical: bool,
    /// Starting position.
    pub start_position: Vec2,
    /// Current vertical offset.
    pub y_offset: f32,
    /// Current opacity (1.0 to 0.0).
    pub opacity: f32,
    /// Animation duration.
    pub duration: f32,
    /// Time elapsed.
    pub elapsed: f32,
}

/// Spawn a floating damage number.
pub fn spawn_damage_number(
    commands: &mut Commands,
    amount: i32,
    is_healing: bool,
    is_critical: bool,
    position: Vec2,
) {
    let duration = if is_critical { 1.5 } else { 1.0 };

    commands.spawn((
        DamageNumber {
            amount: amount.abs(),
            is_healing,
            is_critical,
            start_position: position,
            y_offset: 0.0,
            opacity: 1.0,
            duration,
            elapsed: 0.0,
        },
        AnimationLifetime {
            remaining: duration + 0.1,
        },
        Transform::from_translation(position.extend(15.0)),
        Visibility::default(),
    ));
}

/// System to animate floating damage numbers.
pub fn animate_damage_numbers(
    time: Res<Time>,
    mut query: Query<(&mut DamageNumber, &mut Transform)>,
) {
    for (mut damage, mut transform) in query.iter_mut() {
        damage.elapsed += time.delta_secs();
        let progress = (damage.elapsed / damage.duration).min(1.0);

        // Float upward with easing
        let rise_amount = if damage.is_critical { 100.0 } else { 60.0 };
        damage.y_offset = ease_out_quad(progress) * rise_amount;

        // Fade out in the second half
        damage.opacity = if progress > 0.5 {
            1.0 - (progress - 0.5) * 2.0
        } else {
            1.0
        };

        // Update transform
        transform.translation = Vec3::new(
            damage.start_position.x,
            damage.start_position.y + damage.y_offset,
            15.0,
        );

        // Scale up critical hits
        let scale = if damage.is_critical {
            1.0 + (1.0 - progress) * 0.5
        } else {
            1.0
        };
        transform.scale = Vec3::splat(scale);
    }
}

/// Quadratic ease-out function.
fn ease_out_quad(t: f32) -> f32 {
    1.0 - (1.0 - t) * (1.0 - t)
}

#[allow(dead_code)]
impl DamageNumber {
    /// Get the color for this damage number.
    pub fn color(&self) -> Color {
        let base_color = if self.is_healing {
            Color::srgb(0.2, 0.9, 0.2) // Green for healing
        } else if self.is_critical {
            Color::srgb(1.0, 0.8, 0.0) // Gold for crits
        } else {
            Color::srgb(1.0, 0.3, 0.3) // Red for damage
        };

        // Apply opacity
        base_color.with_alpha(self.opacity)
    }

    /// Get the text to display.
    pub fn text(&self) -> String {
        let prefix = if self.is_healing { "+" } else { "-" };
        format!("{}{}", prefix, self.amount)
    }

    /// Get the font size.
    pub fn font_size(&self) -> f32 {
        let base_size = 24.0;
        let amount_scale = (self.amount as f32 / 20.0).min(2.0);
        let crit_scale = if self.is_critical { 1.5 } else { 1.0 };
        base_size * (1.0 + amount_scale * 0.3) * crit_scale
    }
}
