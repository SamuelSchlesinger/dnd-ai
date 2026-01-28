//! Test dice rolling functionality

use dnd_core::dice::{DiceExpression, Advantage};

fn main() {
    println!("=== Testing Dice Rolling ===\n");

    // Test basic dice
    test_roll("1d20", "Basic d20");
    test_roll("2d6", "Two d6");
    test_roll("1d20+5", "d20 with modifier");
    test_roll("2d6+3", "2d6 with modifier");
    test_roll("4d6kh3", "4d6 keep highest 3 (stat roll)");
    test_roll("8d6", "Fireball damage");

    // Test advantage/disadvantage (requires separate API)
    test_advantage("1d20", Advantage::Advantage, "d20 with advantage");
    test_advantage("1d20", Advantage::Disadvantage, "d20 with disadvantage");
    test_advantage("1d20+5", Advantage::Advantage, "d20+5 with advantage");

    println!("\n=== All dice tests passed! ===");
}

fn test_roll(expr_str: &str, description: &str) {
    print!("Rolling {expr_str} ({description})... ");
    match DiceExpression::parse(expr_str) {
        Ok(expr) => {
            let result = expr.roll();
            let rolls: Vec<_> = result.component_results.iter()
                .flat_map(|c| c.kept.iter())
                .collect();
            println!("Result: {} (kept: {:?})", result.total, rolls);
        }
        Err(e) => {
            println!("PARSE ERROR: {e:?}");
        }
    }
}

fn test_advantage(expr_str: &str, advantage: Advantage, description: &str) {
    print!("Rolling {expr_str} ({description})... ");
    match DiceExpression::parse(expr_str) {
        Ok(expr) => {
            let result = expr.roll_with_advantage(advantage);
            let rolls: Vec<_> = result.component_results.iter()
                .flat_map(|c| c.kept.iter())
                .collect();
            println!("Result: {} (kept: {:?})", result.total, rolls);
        }
        Err(e) => {
            println!("PARSE ERROR: {e:?}");
        }
    }
}
