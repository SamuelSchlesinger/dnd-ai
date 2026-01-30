# dnd-ai

A D&D 5e game with an AI Dungeon Master powered by Claude. Create a character and embark on procedurally-generated adventures with an AI that remembers your story, tracks consequences, and adapts to your choices.

![dnd-ai screenshot](docs/screenshots/gameplay.png)

## Quick Start

**1. Get an API key** from [Anthropic Console](https://console.anthropic.com/)

**2. Set up and run:**

```bash
# Clone and enter the repo
git clone https://github.com/yourusername/dnd-ai.git
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

## Development

```bash
cargo build --workspace    # Build all crates
cargo test --workspace     # Run tests
```

## License

This project is licensed under [CC BY-NC 4.0](LICENSE) - free to use and modify for non-commercial purposes.

D&D content is from the [SRD 5.2](https://dnd.wizards.com/resources/systems-reference-document) (CC BY 4.0, Wizards of the Coast).
