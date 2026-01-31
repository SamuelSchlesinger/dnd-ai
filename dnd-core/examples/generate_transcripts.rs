//! Generate dramatic example transcripts for the README.
//!
//! Run with: `cargo run -p dnd-core --example generate_transcripts`

use dnd_core::headless::{HeadlessConfig, HeadlessGame};
use dnd_core::world::{Background, CharacterClass, RaceType};
use std::fs;
use std::path::Path;

/// A scenario to play out and record.
struct Scenario {
    title: &'static str,
    filename: &'static str,
    description: &'static str,
    character_name: &'static str,
    race: RaceType,
    class: CharacterClass,
    background: Background,
    starting_location: &'static str,
    campaign_name: &'static str,
    actions: Vec<&'static str>,
}

fn scenarios() -> Vec<Scenario> {
    vec![
        Scenario {
            title: "The Goblin Ambush",
            filename: "goblin_ambush.md",
            description: "A lone fighter stumbles into a goblin trap on the forest road.",
            character_name: "Kira Stonefist",
            race: RaceType::Dwarf,
            class: CharacterClass::Fighter,
            background: Background::Soldier,
            starting_location: "The Forest Road",
            campaign_name: "The Goblin Ambush",
            actions: vec![
                "I cautiously walk down the forest path, keeping my hand on my axe.",
                "I stop and listen carefully. Is something moving in the bushes?",
                "I draw my axe and call out: 'Show yourselves, cowards!'",
                "I charge at the nearest goblin with a battle cry!",
                "I swing my axe at the goblin in a wide arc.",
                "I try to kick the goblin away and look for others.",
            ],
        },
        Scenario {
            title: "The Wizard's Bargain",
            filename: "wizards_bargain.md",
            description: "An elf wizard seeks forbidden knowledge from a mysterious stranger.",
            character_name: "Vaeris Moonwhisper",
            race: RaceType::Elf,
            class: CharacterClass::Wizard,
            background: Background::Sage,
            starting_location: "The Dusty Archives",
            campaign_name: "The Wizard's Bargain",
            actions: vec![
                "I search the ancient shelves for any texts on planar magic.",
                "I examine the strange tome more closely. What language is it written in?",
                "I attempt to read the inscription on the cover using my knowledge of arcane languages.",
                "Who approaches? I turn to face the stranger, one hand ready to cast Shield.",
                "I listen carefully to the stranger's offer. What exactly does he want in exchange?",
                "I consider the bargain. 'Knowledge for knowledge seems fair, but what assurance do I have?'",
            ],
        },
        Scenario {
            title: "Tavern Trouble",
            filename: "tavern_trouble.md",
            description: "A charming bard's performance takes an unexpected turn.",
            character_name: "Lyra Silvervine",
            race: RaceType::Human,
            class: CharacterClass::Bard,
            background: Background::Entertainer,
            starting_location: "The Laughing Dragon Tavern",
            campaign_name: "Tavern Trouble",
            actions: vec![
                "I survey the tavern crowd, tuning my lute. Who looks like they need cheering up?",
                "I begin playing a lively tune and watch the crowd's reaction.",
                "I use my Bardic Inspiration! I add a heroic verse about the sad-looking warrior in the corner.",
                "I approach the warrior during my break. 'That's quite a sword you carry. There must be a story there.'",
                "I lean in closer. 'Tell me about the bounty. Perhaps I can help.'",
                "I consider the danger but also the opportunity. 'A song worth singing, then. Count me in.'",
            ],
        },
        Scenario {
            title: "Into the Crypt",
            filename: "into_the_crypt.md",
            description: "A cleric descends into darkness to put restless souls to peace.",
            character_name: "Brother Marcus",
            race: RaceType::Human,
            class: CharacterClass::Cleric,
            background: Background::Acolyte,
            starting_location: "The Abandoned Cemetery",
            campaign_name: "Into the Crypt",
            actions: vec![
                "I hold my holy symbol aloft and pray for guidance as I approach the crypt entrance.",
                "I cast Light on my holy symbol and descend the stairs into the darkness.",
                "I stop and listen. What sounds do I hear from the depths?",
                "I ready my mace and call out: 'In the name of the Light, I come to grant you rest!'",
                "I hold forth my holy symbol and attempt to Turn Undead!",
                "I pursue the remaining undead, determined to cleanse this place.",
            ],
        },
        Scenario {
            title: "The Heist",
            filename: "the_heist.md",
            description: "A cunning rogue infiltrates the merchant lord's manor.",
            character_name: "Shadow",
            race: RaceType::Halfling,
            class: CharacterClass::Rogue,
            background: Background::Criminal,
            starting_location: "Outside the Manor Walls",
            campaign_name: "The Heist",
            actions: vec![
                "I observe the manor from the shadows, watching the guard patrol patterns.",
                "I wait for the guards to pass, then scale the garden wall.",
                "I press myself against the wall and search for an unlocked window or servant's entrance.",
                "I carefully pick the lock, taking my time to avoid making noise.",
                "I slip inside and let my eyes adjust. What room am I in?",
                "I search for the merchant's study. That's where the documents would be.",
            ],
        },
        Scenario {
            title: "Blood and Thunder",
            filename: "blood_and_thunder.md",
            description: "A half-orc barbarian faces a deadly beast in the arena.",
            character_name: "Grukk the Unbroken",
            race: RaceType::HalfOrc,
            class: CharacterClass::Barbarian,
            background: Background::Outlander,
            starting_location: "The Gladiator's Arena",
            campaign_name: "Blood and Thunder",
            actions: vec![
                "I enter the arena and roar to the crowd, beating my chest!",
                "I watch as the gates open. What beast do they release?",
                "I enter my RAGE! My muscles bulge as fury overtakes me!",
                "I charge the beast, greataxe raised high!",
                "I strike with all my might, channeling my rage into the blow!",
                "I refuse to fall! I use my Relentless Endurance and strike back!",
            ],
        },
    ]
}

async fn run_scenario(
    scenario: &Scenario,
    output_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n{}", "=".repeat(60));
    println!("Running: {}", scenario.title);
    println!("{}\n", "=".repeat(60));

    let config = HeadlessConfig::custom(
        scenario.character_name,
        scenario.race,
        scenario.class,
        scenario.background,
    )
    .with_campaign_name(scenario.campaign_name)
    .with_starting_location(scenario.starting_location);

    let mut game = HeadlessGame::new(config).await?;

    let mut transcript = String::new();

    // Header
    transcript.push_str(&format!("# {}\n\n", scenario.title));
    transcript.push_str(&format!("*{}*\n\n", scenario.description));
    transcript.push_str("---\n\n");

    // Character info
    transcript.push_str(&format!(
        "**Character:** {} ({} {})\n",
        scenario.character_name,
        scenario.race.name(),
        scenario.class.name()
    ));
    transcript.push_str(&format!("**Background:** {}\n", scenario.background.name()));
    transcript.push_str(&format!("**Location:** {}\n\n", scenario.starting_location));
    transcript.push_str("---\n\n");

    // Play through the scenario
    for (i, action) in scenario.actions.iter().enumerate() {
        println!("Turn {}: {}", i + 1, action);

        let response = game.send(action).await?;

        transcript.push_str(&format!("### Turn {}\n\n", i + 1));
        transcript.push_str(&format!("**Player:** {}\n\n", action));
        transcript.push_str(&format!("**DM:** {}\n\n", response.narrative));

        if response.in_combat {
            transcript.push_str(&format!(
                "*[Combat: HP {}/{}, Player's turn: {}]*\n\n",
                response.current_hp, response.max_hp, response.is_player_turn
            ));
        }

        transcript.push_str("---\n\n");

        println!(
            "  Response: {}...\n",
            &response.narrative[..response.narrative.len().min(100)]
        );
    }

    // Footer
    transcript.push_str(&format!(
        "*This transcript was generated by [dnd-ai](https://github.com/yourusername/dnd-ai), \
        an AI Dungeon Master powered by Claude.*\n"
    ));

    // Write the file
    let output_path = output_dir.join(scenario.filename);
    fs::write(&output_path, transcript)?;
    println!("Wrote transcript to: {}", output_path.display());

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env file
    let _ = dotenvy::dotenv();

    // Check for API key
    if std::env::var("ANTHROPIC_API_KEY").is_err() {
        eprintln!("Error: ANTHROPIC_API_KEY environment variable not set");
        eprintln!("Set it in your environment or create a .env file");
        std::process::exit(1);
    }

    let output_dir = Path::new("docs/transcripts");
    fs::create_dir_all(output_dir)?;

    let scenarios = scenarios();

    println!("Generating {} transcripts...\n", scenarios.len());

    for scenario in &scenarios {
        if let Err(e) = run_scenario(scenario, output_dir).await {
            eprintln!("Error running '{}': {}", scenario.title, e);
            // Continue with other scenarios
        }
    }

    println!("\nDone! Check docs/transcripts/ for the generated files.");
    Ok(())
}
