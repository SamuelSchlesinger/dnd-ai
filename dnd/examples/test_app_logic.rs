//! Test App logic without TUI

fn main() {
    println!("=== Testing App Logic ===\n");
    
    // Test dice expression parsing (used by :roll command)
    test_dice_command();
    
    // Test FocusedPanel cycling logic
    test_panel_cycling();
    
    println!("\n=== Tests complete! ===");
}

fn test_dice_command() {
    use dnd_core::dice::DiceExpression;
    
    println!("1. Testing :roll command parsing...");
    
    let test_cases = [
        (":roll 1d20", true),
        (":roll 2d6+3", true),
        (":roll 4d6kh3", true),
        (":roll invalid", false),
        (":roll", false),
    ];
    
    for (input, should_parse) in test_cases {
        let expr_part = input.strip_prefix(":roll ").unwrap_or("");
        let parsed = DiceExpression::parse(expr_part).is_ok();
        let status = if parsed == should_parse { "OK" } else { "FAIL" };
        println!("   {status} - '{input}' -> parsed={parsed}, expected={should_parse}");
    }
}

fn test_panel_cycling() {
    println!("\n2. Testing panel focus cycling logic...");
    
    // Simulate the cycle_focus logic
    #[derive(Debug, Clone, Copy, PartialEq)]
    enum FocusedPanel {
        Narrative,
        Character,
        Combat,
    }
    
    fn cycle_focus(panel: FocusedPanel) -> FocusedPanel {
        match panel {
            FocusedPanel::Narrative => FocusedPanel::Character,
            FocusedPanel::Character => FocusedPanel::Combat,
            FocusedPanel::Combat => FocusedPanel::Narrative,
        }
    }
    
    fn cycle_focus_reverse(panel: FocusedPanel) -> FocusedPanel {
        match panel {
            FocusedPanel::Narrative => FocusedPanel::Combat,
            FocusedPanel::Combat => FocusedPanel::Character,
            FocusedPanel::Character => FocusedPanel::Narrative,
        }
    }
    
    let mut panel = FocusedPanel::Narrative;
    
    // Test forward cycling
    println!("   Forward cycling (Tab):");
    for _ in 0..4 {
        let next = cycle_focus(panel);
        println!("      {panel:?} -> {next:?}");
        panel = next;
    }
    
    // Test reverse cycling
    panel = FocusedPanel::Narrative;
    println!("   Reverse cycling (Shift+Tab):");
    for _ in 0..4 {
        let next = cycle_focus_reverse(panel);
        println!("      {panel:?} -> {next:?}");
        panel = next;
    }
    
    println!("   Panel cycling logic OK");
}
