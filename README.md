# dnd-ai

> *"Another seeker of forbidden knowledge, I presume? How... interesting."*
>
> *The woman's smile widens, revealing teeth that seem just a bit too sharp.*
>
> — [The Wizard's Bargain](docs/transcripts/wizards_bargain.md)

---

> *Your divine power surges through the chamber like a thunderclap of pure faith! The holy light becomes almost blinding as it washes over the undead.*
>
> — [Into the Crypt](docs/transcripts/into_the_crypt.md)

---

> *"They were drained, Lyra. Not torn apart like an animal attack. Drained. Like something had sucked the very life from them."*
>
> — [Tavern Trouble](docs/transcripts/tavern_trouble.md)

---

> *The roar that erupts from your throat echoes off the stone walls like the bellow of a primordial beast. Your fists pound against your chest in a primal display of dominance. The crowd erupts!*
>
> — [Blood and Thunder](docs/transcripts/blood_and_thunder.md)

---

A D&D 5e game with an AI Dungeon Master powered by Claude. Create a character and embark on procedurally-generated adventures with an AI that remembers your story, tracks consequences, and adapts to your choices.

**Bring Your Own Key** — This is a local application that runs on your machine. You provide your own [Anthropic API key](https://console.anthropic.com/), so you control your costs and your data never passes through a third party.

## Quick Start

**1. Get an API key** from [Anthropic Console](https://console.anthropic.com/)

**2. Set up and run:**

```bash
# Clone and enter the repo
git clone https://github.com/SamuelSchlesinger/dnd-ai.git
cd dnd-ai

# Set your API key (or create a .env file)
export ANTHROPIC_API_KEY=your_key_here

# Run the game
cargo run -p dnd
```

**3. Create your character** and start playing!

## Controls

| Action | How |
|--------|-----|
| Type messages | Click the input box and type |
| Send message | Press `Enter` or click Send |
| Scroll history | Mouse wheel or drag |

The AI Dungeon Master will narrate your adventure, describe scenes, run combat, and respond to whatever you try. Just type what your character does!

## Features

- **AI Dungeon Master** - Claude narrates your adventure, runs NPCs, and adjudicates actions
- **Full D&D 5e Rules** - Ability checks, saving throws, combat, conditions, and more
- **Story Memory** - The AI remembers characters, locations, and events from your adventure
- **Consequence System** - Your actions have lasting effects that resurface naturally
- **Save/Load** - Continue your campaign across sessions
- **Character Creation** - Choose from SRD races, classes, and backgrounds

## Example Transcripts

See the AI Dungeon Master in action:

| Adventure | Description |
|-----------|-------------|
| [**The Goblin Ambush**](docs/transcripts/goblin_ambush.md) | A dwarf fighter springs a trap on the forest road. Death saves ensue. |
| [**Tavern Trouble**](docs/transcripts/tavern_trouble.md) | A bard's performance wins hearts and uncovers a deadly mystery. |
| [**The Wizard's Bargain**](docs/transcripts/wizards_bargain.md) | An elf seeks forbidden knowledge. A stranger offers a deal. |
| [**Into the Crypt**](docs/transcripts/into_the_crypt.md) | A cleric descends into darkness to face the restless dead. |
| [**The Heist**](docs/transcripts/the_heist.md) | A halfling rogue infiltrates a merchant lord's manor. |
| [**Blood and Thunder**](docs/transcripts/blood_and_thunder.md) | A half-orc barbarian faces certain death in the gladiator's arena. |

## Requirements

- Rust toolchain ([install](https://rustup.rs/))
- Anthropic API key ([get one](https://console.anthropic.com/))

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     dnd (application)                       │
│                    GUI (Bevy + egui)                        │
└─────────────────────────────┬───────────────────────────────┘
                              │
┌─────────────────────────────▼───────────────────────────────┐
│                    dnd-core (library)                       │
│  GameSession, Rules Engine, AI DM, Persistence              │
└─────────────────────────────┬───────────────────────────────┘
                              │
┌─────────────────────────────▼───────────────────────────────┐
│                     claude (library)                        │
│  Anthropic API: completions, streaming, tool use            │
└─────────────────────────────────────────────────────────────┘
```

## How It Works

Curious about the AI behind the DM screen? See **[How the AI Dungeon Master Works](docs/HOW_IT_WORKS.md)** for a peek at the prompt design, tool system, and story memory.

## Development

```bash
cargo build --workspace    # Build all crates
cargo test --workspace     # Run tests
```

## License

This project is licensed under [CC BY-NC 4.0](LICENSE) - free to use and modify for non-commercial purposes.

D&D content is from the [SRD 5.2](https://dnd.wizards.com/resources/systems-reference-document) (CC BY 4.0, Wizards of the Coast).
