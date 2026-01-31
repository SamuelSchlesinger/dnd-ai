//! Combat and spell visual effects.
//!
//! Provides visual feedback for combat events like hits,
//! misses, spell casting, etc.

use bevy::prelude::*;

use super::AnimationLifetime;

/// Types of combat effects.
#[derive(Clone, Copy, Debug)]
pub enum EffectType {
    /// Screen shake on hit.
    ScreenShake,
    /// Flash effect on damage taken.
    DamageFlash,
    /// Particle burst on spell cast.
    SpellCast,
    /// Critical hit effect.
    CriticalHit,
    /// Miss effect (whoosh).
    Miss,
    /// Heal effect.
    Heal,
    /// Death effect.
    Death,
    /// Level up celebration.
    LevelUp,
}

/// Component for combat effect animations.
#[derive(Component)]
#[allow(dead_code)]
pub struct CombatEffect {
    /// Type of effect.
    pub effect_type: EffectType,
    /// Effect intensity (0.0 to 1.0).
    pub intensity: f32,
    /// Animation duration.
    pub duration: f32,
    /// Time elapsed.
    pub elapsed: f32,
    /// Effect-specific data.
    pub position: Vec2,
}

/// Spawn a combat effect.
pub fn spawn_combat_effect(
    commands: &mut Commands,
    effect_type: EffectType,
    position: Vec2,
    intensity: f32,
) {
    let duration = match effect_type {
        EffectType::ScreenShake => 0.3,
        EffectType::DamageFlash => 0.2,
        EffectType::SpellCast => 0.8,
        EffectType::CriticalHit => 0.5,
        EffectType::Miss => 0.3,
        EffectType::Heal => 0.6,
        EffectType::Death => 1.0,
        EffectType::LevelUp => 2.0,
    };

    commands.spawn((
        CombatEffect {
            effect_type,
            intensity: intensity.clamp(0.0, 1.0),
            duration,
            elapsed: 0.0,
            position,
        },
        AnimationLifetime {
            remaining: duration + 0.1,
        },
    ));
}

/// System to animate combat effects.
pub fn animate_combat_effects(
    time: Res<Time>,
    mut query: Query<&mut CombatEffect>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
) {
    // Accumulate screen shake from all active shake effects
    let mut total_shake = Vec2::ZERO;

    for mut effect in query.iter_mut() {
        effect.elapsed += time.delta_secs();
        let progress = (effect.elapsed / effect.duration).min(1.0);

        match effect.effect_type {
            EffectType::ScreenShake => {
                // Decaying screen shake
                let shake_amount = effect.intensity * 10.0 * (1.0 - progress);
                let shake_x = (effect.elapsed * 50.0).sin() * shake_amount;
                let shake_y = (effect.elapsed * 43.0).cos() * shake_amount;
                total_shake += Vec2::new(shake_x, shake_y);
            }
            EffectType::CriticalHit => {
                // More intense screen shake for crits
                let shake_amount = effect.intensity * 20.0 * (1.0 - progress);
                let shake_x = (effect.elapsed * 80.0).sin() * shake_amount;
                let shake_y = (effect.elapsed * 67.0).cos() * shake_amount;
                total_shake += Vec2::new(shake_x, shake_y);
            }
            _ => {
                // Other effects don't shake the camera
            }
        }
    }

    // Apply screen shake to camera
    if total_shake != Vec2::ZERO {
        for mut transform in camera_query.iter_mut() {
            // Only shake, don't permanently move
            transform.translation.x = total_shake.x;
            transform.translation.y = total_shake.y;
        }
    } else {
        // Reset camera position when no shake
        for mut transform in camera_query.iter_mut() {
            transform.translation.x = 0.0;
            transform.translation.y = 0.0;
        }
    }
}

#[allow(dead_code)]
impl CombatEffect {
    /// Get the color for flash effects.
    pub fn flash_color(&self) -> Option<Color> {
        let progress = (self.elapsed / self.duration).min(1.0);
        let alpha = (1.0 - progress) * self.intensity * 0.5;

        match self.effect_type {
            EffectType::DamageFlash => Some(Color::srgba(1.0, 0.0, 0.0, alpha)),
            EffectType::Heal => Some(Color::srgba(0.0, 1.0, 0.0, alpha)),
            EffectType::SpellCast => Some(Color::srgba(0.5, 0.5, 1.0, alpha)),
            EffectType::LevelUp => Some(Color::srgba(1.0, 0.8, 0.0, alpha)),
            _ => None,
        }
    }
}
