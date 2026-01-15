# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands

```bash
# Build entire workspace
cargo build --workspace

# Run tests
cargo test --workspace

# Run a single test
cargo test test_name

# Run example agents (requires ANTHROPIC_API_KEY in .env)
cargo run --example simple_chat
cargo run --example tool_agent

# Run the D&D game (requires ANTHROPIC_API_KEY in .env)
cargo run --bin dnd_game
```

## Architecture Overview

This is an AI agent framework in Rust with two workspace members:

- **`lib/`** - Core `agentic` crate with traits and types
- **`agents/`** - Example agents using the framework

### Core Library Structure (`lib/src/`)

The framework follows a trait-based design with these key modules:

| Module | Purpose |
|--------|---------|
| `agent.rs` | `Agent` trait - central abstraction for processing messages |
| `tool.rs` | `Tool` trait with `ToolRegistry` for executable functions |
| `memory.rs` | Three memory types: `EpisodicMemory`, `SemanticMemory`, `ProceduralMemory` |
| `safety.rs` | `SafetyValidator`, `Guardrail`, `ApprovalWorkflow` traits with `SafetyPipeline` |
| `context.rs` | `ContextManager`, `Retriever`, `StatePersistence` for context window management |
| `llm/` | LLM providers - currently `AnthropicProvider` with streaming support |
| `id.rs` | Type-safe ID newtypes (AgentId, ToolId, MessageId, etc.) using UUID |
| `message.rs` | Message types with ContentBlock variants (Text, Image, ToolUse, ToolResult, Thinking) |
| `action.rs` | Action types for the safety validation pipeline |
| `error.rs` | Error types using thiserror |

### Key Design Patterns

1. **Type-safe IDs**: All IDs are newtypes around UUID to prevent mixing different ID types at compile time
2. **Async traits**: All major traits use `#[async_trait]` for async operations
3. **Builder pattern**: Configuration uses builder pattern (e.g., `CompletionRequest::new().with_system().with_messages()`)
4. **Content blocks**: Messages use `Vec<ContentBlock>` to support mixed content (text, images, tool calls)

### LLM Integration Flow

The Anthropic provider in `llm/anthropic.rs` handles:
- Converting internal `Message` types to API format
- Tool definitions with JSON schemas
- Streaming via SSE parsing
- Tool use loop (stop_reason == ToolUse triggers tool execution)

### Adding a New Tool

Implement the `Tool` trait:
```rust
#[async_trait]
impl Tool for MyTool {
    fn name(&self) -> &str { "my_tool" }
    fn description(&self) -> &str { "What it does" }
    fn input_schema(&self) -> &Value { /* JSON Schema */ }
    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        // Implementation
    }
}
```

## Research Reports

The `agents/research/` directory contains detailed research reports on:
- Planning (HTN, GOAP, Tree-of-Thought)
- Memory systems (episodic, semantic, procedural)
- Safety and security patterns
- Privacy and compliance
- Multi-agent coordination
- Industry survey (Anthropic MCP, OpenAI, Google, LangChain)

See `FRAMEWORK_DESIGN.md` for the full architectural vision synthesized from this research.

## D&D Dungeon Master Agent (`agents/src/dnd/`)

A comprehensive single-player D&D 5e experience with an AI Dungeon Master and TUI interface.

### Module Structure

```
agents/src/dnd/
├── mod.rs              # Module root
├── app.rs              # Application state, InputMode (vim-style)
├── events.rs           # Event handling for all input modes
├── game/               # D&D 5e game mechanics
│   ├── dice.rs         # Dice notation parser (2d6+3, 4d6kh3, advantage)
│   ├── character.rs    # Full character sheets, abilities, spells
│   ├── combat.rs       # Initiative tracking, turn management
│   ├── state.rs        # GameWorld, locations, quests, NPCs
│   ├── conditions.rs   # All 14 PHB conditions
│   └── skills.rs       # 18 skills with ability mappings
├── ui/                 # TUI with ratatui
│   ├── render.rs       # Main render orchestration
│   ├── layout.rs       # Exploration and combat layouts
│   ├── theme.rs        # Color schemes
│   └── widgets/        # NarrativeWidget, CombatTracker, DiceRoll, etc.
└── ai/                 # AI agents
    ├── dm_agent.rs     # Main DM agent with tool execution
    ├── tools.rs        # D&D tools (RollDice, SkillCheck, etc.)
    ├── prompts.rs      # System prompt builder
    └── subagents/      # Specialized agents (Combat, NPC, Rules)
```

### Vim-Style Input Modes

The TUI uses vim-style modal input:

- **NORMAL** (default): Navigation (`j`/`k`), hotkeys (`?` help, `r` rest), mode switching (`i`, `:`)
- **INSERT**: Free text input, `Esc` to return to normal, `Enter` to send
- **COMMAND**: `:q` quit, `:roll XdY` roll dice, `:rest` take rest

### Key Features

- Full D&D 5e dice notation with advantage/disadvantage
- Complete character sheets with spells, conditions, inventory
- Combat tracking with initiative and turn management
- Recursive subagent architecture for context-efficient AI responses
- Streaming narrative display with animated dice rolls
