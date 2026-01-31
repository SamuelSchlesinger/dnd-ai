//! D&D dice rolling system.
//!
//! Supports standard dice notation: XdY+Z, advantage/disadvantage,
//! keep highest/lowest, and more.

use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use thiserror::Error;

/// Error type for dice parsing and rolling.
#[derive(Debug, Error)]
pub enum DiceError {
    #[error("Invalid dice notation: {0}")]
    InvalidNotation(String),
    #[error("Invalid die size: {0}")]
    InvalidDieSize(u32),
    #[error("No dice specified")]
    NoDice,
    #[error("Cannot keep {keep} dice when only rolling {count} (in {notation})")]
    InvalidKeepCount {
        keep: u32,
        count: u32,
        notation: String,
    },
}

/// Advantage state for d20 rolls.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Advantage {
    #[default]
    Normal,
    Advantage,
    Disadvantage,
}

impl Advantage {
    /// Combine two advantage states (advantage + disadvantage = normal).
    pub fn combine(self, other: Advantage) -> Advantage {
        match (self, other) {
            (Advantage::Normal, x) | (x, Advantage::Normal) => x,
            (Advantage::Advantage, Advantage::Disadvantage) => Advantage::Normal,
            (Advantage::Disadvantage, Advantage::Advantage) => Advantage::Normal,
            (Advantage::Advantage, Advantage::Advantage) => Advantage::Advantage,
            (Advantage::Disadvantage, Advantage::Disadvantage) => Advantage::Disadvantage,
        }
    }
}

/// Standard D&D die types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DieType {
    D4,
    D6,
    D8,
    D10,
    D12,
    D20,
    D100,
}

impl DieType {
    pub fn sides(&self) -> u32 {
        match self {
            DieType::D4 => 4,
            DieType::D6 => 6,
            DieType::D8 => 8,
            DieType::D10 => 10,
            DieType::D12 => 12,
            DieType::D20 => 20,
            DieType::D100 => 100,
        }
    }

    pub fn from_sides(sides: u32) -> Option<DieType> {
        match sides {
            4 => Some(DieType::D4),
            6 => Some(DieType::D6),
            8 => Some(DieType::D8),
            10 => Some(DieType::D10),
            12 => Some(DieType::D12),
            20 => Some(DieType::D20),
            100 => Some(DieType::D100),
            _ => None,
        }
    }
}

impl fmt::Display for DieType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "d{}", self.sides())
    }
}

/// A single die component of a dice expression.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiceComponent {
    pub count: u32,
    pub die_type: DieType,
    pub keep_highest: Option<u32>,
    pub keep_lowest: Option<u32>,
}

/// A complete dice expression (e.g., 2d6+3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiceExpression {
    pub components: Vec<DiceComponent>,
    pub modifier: i32,
    pub original: String,
}

impl DiceExpression {
    /// Parse a dice notation string.
    pub fn parse(notation: &str) -> Result<Self, DiceError> {
        let notation = notation.trim().to_lowercase();
        if notation.is_empty() {
            return Err(DiceError::NoDice);
        }

        let mut components = Vec::new();
        let mut modifier: i32 = 0;
        let mut current = String::new();
        let mut sign: i32 = 1;

        for ch in notation.chars() {
            match ch {
                '+' | '-' => {
                    if !current.is_empty() {
                        Self::parse_component(&current, sign, &mut components, &mut modifier)?;
                        current.clear();
                    }
                    sign = if ch == '+' { 1 } else { -1 };
                }
                ' ' => continue,
                _ => current.push(ch),
            }
        }

        if !current.is_empty() {
            Self::parse_component(&current, sign, &mut components, &mut modifier)?;
        }

        if components.is_empty() && modifier == 0 {
            return Err(DiceError::NoDice);
        }

        Ok(DiceExpression {
            components,
            modifier,
            original: notation,
        })
    }

    fn parse_component(
        s: &str,
        sign: i32,
        components: &mut Vec<DiceComponent>,
        modifier: &mut i32,
    ) -> Result<(), DiceError> {
        if let Some(d_pos) = s.find('d') {
            let count_str = &s[..d_pos];
            let rest = &s[d_pos + 1..];

            let count: u32 = if count_str.is_empty() {
                1
            } else {
                count_str
                    .parse()
                    .map_err(|_| DiceError::InvalidNotation(s.to_string()))?
            };

            let (sides_str, keep_highest, keep_lowest) = if let Some(kh_pos) = rest.find("kh") {
                let sides = &rest[..kh_pos];
                let keep: u32 = rest[kh_pos + 2..]
                    .parse()
                    .map_err(|_| DiceError::InvalidNotation(s.to_string()))?;
                (sides, Some(keep), None)
            } else if let Some(kl_pos) = rest.find("kl") {
                let sides = &rest[..kl_pos];
                let keep: u32 = rest[kl_pos + 2..]
                    .parse()
                    .map_err(|_| DiceError::InvalidNotation(s.to_string()))?;
                (sides, None, Some(keep))
            } else {
                (rest, None, None)
            };

            let sides: u32 = sides_str
                .parse()
                .map_err(|_| DiceError::InvalidNotation(s.to_string()))?;

            let die_type = DieType::from_sides(sides).ok_or(DiceError::InvalidDieSize(sides))?;

            // Validate keep count doesn't exceed dice count
            if let Some(keep) = keep_highest.or(keep_lowest) {
                if keep > count {
                    return Err(DiceError::InvalidKeepCount {
                        keep,
                        count,
                        notation: s.to_string(),
                    });
                }
            }

            components.push(DiceComponent {
                count,
                die_type,
                keep_highest,
                keep_lowest,
            });
        } else {
            let value: i32 = s
                .parse()
                .map_err(|_| DiceError::InvalidNotation(s.to_string()))?;
            *modifier += sign * value;
        }

        Ok(())
    }

    /// Roll the dice expression and return the result.
    pub fn roll(&self) -> RollResult {
        self.roll_with_rng(&mut rand::thread_rng())
    }

    /// Roll with a specific RNG (useful for testing).
    pub fn roll_with_rng<R: Rng>(&self, rng: &mut R) -> RollResult {
        let mut all_rolls = Vec::new();
        let mut component_results = Vec::new();

        for component in &self.components {
            let mut rolls: Vec<u32> = (0..component.count)
                .map(|_| rng.gen_range(1..=component.die_type.sides()))
                .collect();

            let kept_rolls = if let Some(keep) = component.keep_highest {
                rolls.sort_by(|a, b| b.cmp(a));
                rolls.truncate(keep as usize);
                rolls.clone()
            } else if let Some(keep) = component.keep_lowest {
                rolls.sort();
                rolls.truncate(keep as usize);
                rolls.clone()
            } else {
                rolls.clone()
            };

            let subtotal: u32 = kept_rolls.iter().sum();
            component_results.push(ComponentResult {
                die_type: component.die_type,
                rolls: rolls.clone(),
                kept: kept_rolls,
                subtotal,
            });
            all_rolls.extend(rolls);
        }

        let dice_total: i32 = component_results.iter().map(|c| c.subtotal as i32).sum();
        let total = dice_total + self.modifier;

        // Find the d20 result for natural 20/1 detection
        // (only matters for single d20 components, used in attack rolls)
        let d20_roll = component_results
            .iter()
            .find(|c| c.die_type == DieType::D20 && c.rolls.len() == 1)
            .and_then(|c| c.rolls.first().copied());

        RollResult {
            expression: self.clone(),
            component_results,
            modifier: self.modifier,
            total,
            natural_20: d20_roll == Some(20),
            natural_1: d20_roll == Some(1),
        }
    }

    /// Roll with advantage/disadvantage (only applies to single d20 rolls).
    pub fn roll_with_advantage(&self, advantage: Advantage) -> RollResult {
        self.roll_with_advantage_rng(advantage, &mut rand::thread_rng())
    }

    pub fn roll_with_advantage_rng<R: Rng>(&self, advantage: Advantage, rng: &mut R) -> RollResult {
        match advantage {
            Advantage::Normal => self.roll_with_rng(rng),
            Advantage::Advantage | Advantage::Disadvantage => {
                if !self.is_single_d20() {
                    return self.roll_with_rng(rng);
                }

                let roll1 = rng.gen_range(1..=20u32);
                let roll2 = rng.gen_range(1..=20u32);

                let (chosen, _other) = match advantage {
                    Advantage::Advantage => {
                        if roll1 >= roll2 {
                            (roll1, roll2)
                        } else {
                            (roll2, roll1)
                        }
                    }
                    Advantage::Disadvantage => {
                        if roll1 <= roll2 {
                            (roll1, roll2)
                        } else {
                            (roll2, roll1)
                        }
                    }
                    Advantage::Normal => unreachable!(),
                };

                let total = chosen as i32 + self.modifier;

                RollResult {
                    expression: self.clone(),
                    component_results: vec![ComponentResult {
                        die_type: DieType::D20,
                        rolls: vec![roll1, roll2],
                        kept: vec![chosen],
                        subtotal: chosen,
                    }],
                    modifier: self.modifier,
                    total,
                    natural_20: chosen == 20,
                    natural_1: chosen == 1,
                }
            }
        }
    }

    fn is_single_d20(&self) -> bool {
        self.components.len() == 1
            && self.components[0].count == 1
            && self.components[0].die_type == DieType::D20
    }
}

impl FromStr for DiceExpression {
    type Err = DiceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        DiceExpression::parse(s)
    }
}

impl fmt::Display for DiceExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.original)
    }
}

/// Result of rolling a single dice component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentResult {
    pub die_type: DieType,
    pub rolls: Vec<u32>,
    pub kept: Vec<u32>,
    pub subtotal: u32,
}

/// Complete result of a dice roll.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollResult {
    pub expression: DiceExpression,
    pub component_results: Vec<ComponentResult>,
    pub modifier: i32,
    pub total: i32,
    pub natural_20: bool,
    pub natural_1: bool,
}

impl RollResult {
    /// Format the individual dice results for display.
    pub fn dice_display(&self) -> String {
        let dice_parts: Vec<String> = self
            .component_results
            .iter()
            .map(|c| {
                if c.rolls.len() > c.kept.len() {
                    let mut kept_used = vec![false; c.kept.len()];
                    let mut shown = Vec::new();

                    for &roll in &c.rolls {
                        let is_kept = c.kept.iter().enumerate().any(|(i, &k)| {
                            if k == roll && !kept_used[i] {
                                kept_used[i] = true;
                                true
                            } else {
                                false
                            }
                        });

                        if is_kept {
                            shown.push(format!("{roll}"));
                        } else {
                            shown.push(format!("({roll})"));
                        }
                    }
                    format!("[{}]", shown.join(", "))
                } else {
                    format!(
                        "[{}]",
                        c.rolls
                            .iter()
                            .map(|r| r.to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                }
            })
            .collect();

        let dice_str = dice_parts.join(" + ");
        if self.modifier != 0 {
            if self.modifier > 0 {
                format!("{} + {}", dice_str, self.modifier)
            } else {
                format!("{} - {}", dice_str, self.modifier.abs())
            }
        } else {
            dice_str
        }
    }

    /// Check if the roll meets or exceeds a DC.
    pub fn meets_dc(&self, dc: i32) -> bool {
        self.total >= dc
    }

    /// Check if this was a critical hit (natural 20 on attack).
    pub fn is_critical(&self) -> bool {
        self.natural_20
    }

    /// Check if this was a critical failure (natural 1 on attack).
    pub fn is_fumble(&self) -> bool {
        self.natural_1
    }
}

impl fmt::Display for RollResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} = {}", self.dice_display(), self.total)
    }
}

/// Convenience function to roll dice from a notation string.
pub fn roll(notation: &str) -> Result<RollResult, DiceError> {
    let expr = DiceExpression::parse(notation)?;
    Ok(expr.roll())
}

/// Roll with advantage/disadvantage.
pub fn roll_with_advantage(notation: &str, advantage: Advantage) -> Result<RollResult, DiceError> {
    let expr = DiceExpression::parse(notation)?;
    Ok(expr.roll_with_advantage(advantage))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let expr = DiceExpression::parse("1d20").unwrap();
        assert_eq!(expr.components.len(), 1);
        assert_eq!(expr.components[0].count, 1);
        assert_eq!(expr.components[0].die_type, DieType::D20);
        assert_eq!(expr.modifier, 0);
    }

    #[test]
    fn test_parse_with_modifier() {
        let expr = DiceExpression::parse("1d20+5").unwrap();
        assert_eq!(expr.modifier, 5);

        let expr = DiceExpression::parse("2d6-2").unwrap();
        assert_eq!(expr.modifier, -2);
    }

    #[test]
    fn test_parse_multiple_dice() {
        let expr = DiceExpression::parse("2d6+1d4+3").unwrap();
        assert_eq!(expr.components.len(), 2);
        assert_eq!(expr.modifier, 3);
    }

    #[test]
    fn test_parse_keep_highest() {
        let expr = DiceExpression::parse("4d6kh3").unwrap();
        assert_eq!(expr.components[0].count, 4);
        assert_eq!(expr.components[0].keep_highest, Some(3));
    }

    #[test]
    fn test_invalid_keep_count() {
        // Can't keep more dice than you roll
        let result = DiceExpression::parse("4d6kh5");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            DiceError::InvalidKeepCount {
                keep: 5,
                count: 4,
                ..
            }
        ));

        // Keep lowest also validates
        let result = DiceExpression::parse("2d20kl3");
        assert!(result.is_err());

        // Equal is fine
        let result = DiceExpression::parse("4d6kh4");
        assert!(result.is_ok());
    }

    #[test]
    fn test_roll_range() {
        for _ in 0..100 {
            let result = roll("1d20").unwrap();
            assert!(result.total >= 1 && result.total <= 20);
        }
    }

    #[test]
    fn test_roll_with_modifier() {
        for _ in 0..100 {
            let result = roll("1d20+5").unwrap();
            assert!(result.total >= 6 && result.total <= 25);
        }
    }

    #[test]
    fn test_advantage_combine() {
        assert_eq!(
            Advantage::Normal.combine(Advantage::Advantage),
            Advantage::Advantage
        );
        assert_eq!(
            Advantage::Advantage.combine(Advantage::Disadvantage),
            Advantage::Normal
        );
        assert_eq!(
            Advantage::Advantage.combine(Advantage::Advantage),
            Advantage::Advantage
        );
    }
}
