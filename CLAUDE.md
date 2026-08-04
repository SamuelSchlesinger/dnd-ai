# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

At the end of each session, if you learned something important about the codebase that isn't documented here, add it.

## Licensing Compliance

This project uses D&D content under the **SRD 5.2 (Creative Commons Attribution 4.0)** license. When adding or modifying D&D-related content:

**Only use content from the SRD 5.2:**
- Reference: https://dnd.wizards.com/resources/systems-reference-document
- The SRD includes: 9 races, 12 classes, basic spells, monsters, and core mechanics

**Do NOT include copyrighted content from:**
- Player's Handbook (beyond SRD content)
- Monster Manual (beyond SRD creatures)
- Other D&D sourcebooks (Xanathar's, Tasha's, etc.)
- Setting-specific content (Forgotten Realms lore, named NPCs, etc.)

**Safe to use:**
- All races: Human, Elf, Dwarf, Halfling, Half-Orc, Half-Elf, Tiefling, Gnome, Dragonborn
- All 12 base classes with one subclass each (per SRD)
- Spells, monsters, and magic items listed in the SRD
- Core game mechanics (ability scores, skills, combat rules, etc.)
- Original content you create

**When in doubt:** Check the SRD document directly or use generic/original content instead.

## Build Commands

```bash
# Build entire workspace
cargo build --workspace

# Run tests
cargo test --workspace

# Run a single test
cargo test test_name

# Run configured-provider examples
cargo run -p chronicler-llm --example simple_chat
cargo run -p chronicler-llm --example tool_use
cargo run -p chronicler-llm --example provider_smoke -- ollama

# Run the game using environment configuration or Ollama
cargo run -p chronicler
cargo run -p chronicler -- --local
```

## Pre-Commit Requirements

**Before committing any changes, ALL of these must pass:**

```bash
cargo fmt --all          # Format code
cargo clippy --workspace -- -D warnings # Lint check (warnings are errors)
cargo test --workspace   # All tests must pass
```

**Do not commit if any of these fail.** Fix issues before committing.

### Git Hook (Recommended)

Install the pre-commit hook to automatically run these checks:

```bash
./scripts/install-hooks.sh
```

This will block commits that fail formatting, clippy, or tests.

## Testing Strategy

### Philosophy

- **Unit tests**: Deterministic rules engine tests with real assertions (run in CI)
- **Integration tests**: API connectivity tests requiring a configured provider (marked `#[ignore]`, run manually)

The AI DM's interpretation of player input is non-deterministic and cannot be reliably unit tested. We test the rules engine directly instead.

### Running Tests

```bash
cargo test --workspace                    # Run all unit tests
cargo test -p chronicler-core test_name   # Run specific test
cargo tarpaulin --workspace               # Generate coverage report
```

### Coverage Status

Current coverage: **61.39%** (10803/17598 lines)

**Well-covered modules (>80%):**

| File | Coverage |
|------|----------|
| `rules/resolve/class_features.rs` | 91.30% |
| `rules/resolve/inventory.rs` | 86.92% |
| `rules/resolve/quests.rs` | 100% |
| `rules/resolve/checks.rs` | 88.06% |
| `rules/resolve/world.rs` | 84.15% |
| `rules/helpers.rs` | 100% |
| `dm/tools/parsing/class_features.rs` | 100% |
| `dm/tools/parsing/inventory.rs` | 100% |
| Spell definitions | 100% |
| Items database | 98% |

**Needs improvement:**

| File | Coverage | Priority |
|------|----------|----------|
| `dm/agent.rs` | 18% | Medium - core DM logic |
| `session.rs` | 11% | Medium - public API |
| `rules/effects.rs` | 35% | Medium - effect application |
| `rules/engine.rs` | 31% | Medium - intent resolution |
| `headless.rs` | 23% | Low - integration API |
| `dm/tools/parsing/world.rs` | 0% | Low - world-building parsing |

## Workspace Structure

This workspace contains 3 crates:

| Crate | Path | Description |
|-------|------|-------------|
| `chronicler-llm` | `claude/` | Anthropic and OpenAI-compatible LLM client |
| `chronicler-core` | `chronicler-core/` | Tabletop RPG game engine compatible with D&D 5e, with AI Dungeon Master |
| `chronicler` | `chronicler-bevy/` | Bevy GUI application |

## LLM Client (`claude/src/`)

A provider-neutral client with native Anthropic Messages and OpenAI-compatible Chat Completions transports:

```rust
use chronicler_llm::{Client, Request, Message};

let client = Client::from_env()?;
let response = client.complete(
    Request::new(vec![Message::user("Hello")])
        .with_system("You are helpful.")
).await?;
```

Features:
- Non-streaming and streaming completions
- Tool use with automatic execution loop (`complete_with_tools`)
- Anthropic, OpenAI, OpenRouter/Kimi K3, and Ollama support
- OpenRouter reasoning-metadata preservation across tool turns

Provider invariants:
- Keep provider wire types inside `chronicler-llm`; core game code uses only
  normalized messages, content blocks, tools, responses, and stream events.
- Provider/model configuration is injected when a session is created or loaded
  and is never serialized into a campaign save.
- OpenAI-compatible does not mean text-only: a game model must support
  JSON-schema tools and tool-result continuation.
- See `docs/PROVIDERS.md` for selection precedence, endpoint overrides, local
  Qwen usage, and the credential boundary.

## Game Engine (`chronicler-core/src/`)

The tabletop RPG game engine (compatible with D&D 5e) provides:

| Module | Purpose |
|--------|---------|
| `session.rs` | `GameSession` - main public API |
| `rules.rs` | D&D 5e rules engine |
| `world.rs` | Game state, characters, locations |
| `dice.rs` | Dice notation parser (2d6+3, 4d6kh3, advantage) |
| `character_builder.rs` | Character creation |
| `persist.rs` | Save/load campaigns |
| `dm/` | AI Dungeon Master implementation |

### AI Dungeon Master (`chronicler-core/src/dm/`)

```
dm/
├── agent.rs          # Main DM agent with tool execution
├── tools.rs          # RPG tools (dice, skill checks, etc.)
├── memory.rs         # Context management and summarization
├── prompts/          # System prompt templates (.txt files)
└── story_memory/     # Fact, entity, and relationship tracking
```

## Adding a New Tool

Create a function that returns a `chronicler_llm::Tool` with a JSON schema:

```rust
use chronicler_llm::Tool;
use serde_json::json;

pub fn roll_dice() -> Tool {
    Tool {
        name: "roll_dice".to_string(),
        description: "Roll dice using standard D&D notation.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "notation": {
                    "type": "string",
                    "description": "Dice notation like '2d6+3'"
                },
                "purpose": {
                    "type": "string",
                    "description": "Optional purpose for the roll"
                }
            },
            "required": ["notation"]
        }),
    }
}
```
