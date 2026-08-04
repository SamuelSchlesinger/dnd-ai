# Contributing to Chronicler

Thanks for your interest in contributing! This document will help you get started.

## Development Setup

**Prerequisites:**
- Rust toolchain ([rustup.rs](https://rustup.rs/))
- A supported cloud-provider key, or Ollama for local integration testing

**Setup:**
```bash
git clone https://github.com/SamuelSchlesinger/chronicler.git
cd chronicler
cp .env.example .env
# Edit .env and configure one provider, or use --local
```

**Build & Test:**
```bash
cargo build --workspace     # Build all crates
cargo test --workspace      # Run all tests
cargo clippy --workspace    # Check for lints
cargo fmt --check           # Check formatting
```

**Run the game:**
```bash
cargo run -p chronicler
# Local Qwen default; no API key:
cargo run -p chronicler -- --local
```

Provider setup, selection precedence, and custom compatible endpoints are
documented in [docs/PROVIDERS.md](docs/PROVIDERS.md).

## Project Structure

| Crate | Path | Purpose |
|-------|------|---------|
| `chronicler-llm` | `claude/` | Anthropic and OpenAI-compatible LLM client |
| `chronicler-core` | `chronicler-core/` | Game engine, rules, AI DM |
| `chronicler` | `chronicler-bevy/` | Bevy GUI application |

## Provider Boundary

Game code should depend on the provider-neutral types exported by
`chronicler-llm`. Do not put Anthropic, OpenAI, OpenRouter, or Ollama wire
payloads in `chronicler-core`.

- A new OpenAI-compatible service usually needs configuration and tests, not a
  new transport.
- A genuinely different protocol belongs behind a transport adapter in
  `chronicler-llm`.
- Preserve assistant tool-call content between a tool request and its result.
  Some providers attach opaque reasoning metadata that must also be replayed.
- A game-capable model must support JSON-schema tool calling; a successful
  plain-text completion is not a sufficient integration test.

Use the live smoke test to exercise tool continuation and streaming:

```bash
cargo run -p chronicler-llm --example provider_smoke -- ollama
cargo run -p chronicler-llm --example provider_smoke -- openrouter
```

## Code Style

- Run `cargo fmt` before committing
- Run `cargo clippy` and fix warnings
- Write doc comments for public APIs
- Add tests for new functionality

## Adding a New DM Tool

Tools let the AI DM interact with game mechanics. To add one:

```rust
use chronicler_llm::Tool;
use serde_json::json;

pub fn your_tool() -> Tool {
    Tool {
        name: "your_tool_name".to_string(),
        description: "Brief description of what the tool does.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "some_param": {
                    "type": "string",
                    "description": "Description of this parameter"
                },
                "optional_param": {
                    "type": "integer",
                    "description": "Optional parameters aren't in required"
                }
            },
            "required": ["some_param"]
        }),
    }
}
```

Then implement the tool handler in `chronicler-core/src/dm/agent.rs`.

## D&D Content Guidelines

This project uses **SRD 5.2** content under Creative Commons. When adding D&D content:

- **Use only SRD content** - Check `docs/SRD_CC_v5.2.pdf` if unsure
- **Safe:** 9 SRD races, 12 base classes, SRD spells/monsters
- **Not safe:** Content from PHB, MM, or other sourcebooks beyond the SRD

See `CLAUDE.md` for detailed licensing guidance.

## Pull Requests

1. Fork the repo and create a feature branch
2. Make your changes with clear commit messages
3. Ensure `cargo test`, `cargo clippy`, and `cargo fmt --check` pass
4. Open a PR with a description of what you changed and why

## Questions?

Open an issue for bugs, feature requests, or questions about the codebase.
