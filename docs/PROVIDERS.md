# Model Providers

Chronicler is not tied to one model vendor. The game engine, rules, tools,
memory, and save format use a provider-neutral LLM interface. A provider is
chosen when the application starts, so an existing campaign can be resumed
with a different provider or model without migrating the save.

Two transports currently implement that interface:

- Native Anthropic Messages for Anthropic models
- OpenAI-compatible Chat Completions for OpenAI, OpenRouter, Ollama, and other
  compatible servers

## Local Qwen in One Flag

Install Ollama and make sure the model is available once:

```bash
ollama pull qwen3.6:35b-a3b
```

With Ollama running, launch Chronicler from the workspace root:

```bash
cargo run -p chronicler -- --local
```

A bare `--local` means:

- use Ollama at `http://localhost:11434/v1`
- use `qwen3.6:35b-a3b` for both narrative and background work
- do not require or send an API key

The flag selects the local endpoint and model; it does not install Ollama,
download the model, or start the Ollama service. To choose another installed
model:

```bash
cargo run -p chronicler -- --local MODEL_NAME
```

## Supported Configurations

| Configuration | Protocol | Main model | Fast model | Credential |
|---|---|---|---|---|
| OpenRouter | OpenAI-compatible | `moonshotai/kimi-k3` | same | `OPENROUTER_API_KEY` |
| Anthropic | Anthropic Messages | `claude-sonnet-4-6` | `claude-haiku-4-5-20251001` | `ANTHROPIC_API_KEY` |
| OpenAI | OpenAI Chat Completions | `gpt-5.6-sol` | `gpt-5.6-luna` | `OPENAI_API_KEY` |
| Ollama | OpenAI-compatible | `qwen3.6:35b-a3b` | same | none |
| Custom server | OpenAI-compatible | configured by you | configured by you | server-dependent |

The **main model** narrates and chooses game tools. The **fast model** performs
background relevance checks and state inference. They can be two different
models or the same model, as in the default local and OpenRouter configurations.

## Selecting a Provider

The desktop application loads a repository-root `.env` file if present.
Start with:

```bash
cp .env.example .env
```

Then select one configuration:

```bash
# OpenRouter / Kimi K3
CHRONICLER_PROVIDER=openrouter
OPENROUTER_API_KEY=...

# Anthropic
CHRONICLER_PROVIDER=anthropic
ANTHROPIC_API_KEY=...

# OpenAI
CHRONICLER_PROVIDER=openai
OPENAI_API_KEY=...

# Ollama through environment configuration
CHRONICLER_PROVIDER=local
OLLAMA_MODEL=qwen3.6:35b-a3b
```

Selection is deterministic:

1. `--local [MODEL]` explicitly selects local Ollama and ignores cloud-provider
   selection.
2. Otherwise, `CHRONICLER_PROVIDER` selects `openrouter`, `anthropic`, `openai`,
   or `local` (`ollama` is accepted as a synonym).
3. Without an explicit selection, Chronicler picks the first configured
   credential in this order: OpenRouter, Anthropic, then OpenAI. This is
   startup selection, not runtime failover.

Use `CHRONICLER_MODEL` and `CHRONICLER_FAST_MODEL` to override both model roles
independently of provider. Provider-specific overrides are also available:

| Provider | Main model | Fast model | Base URL |
|---|---|---|---|
| OpenRouter | `OPENROUTER_MODEL` | `OPENROUTER_FAST_MODEL` | `OPENROUTER_BASE_URL` |
| Anthropic | `ANTHROPIC_MODEL` | `ANTHROPIC_FAST_MODEL` | `ANTHROPIC_BASE_URL` |
| OpenAI/custom | `OPENAI_MODEL` | `OPENAI_FAST_MODEL` | `OPENAI_BASE_URL` |
| Ollama | `OLLAMA_MODEL` | `OLLAMA_FAST_MODEL` | `OLLAMA_BASE_URL` |

The explicit `--local` shortcut always uses the built-in localhost URL. Use
`CHRONICLER_PROVIDER=local` when you need to override `OLLAMA_BASE_URL`.

## Other OpenAI-Compatible Servers

An OpenAI-compatible server generally needs no Chronicler code changes:

```bash
CHRONICLER_PROVIDER=openai
OPENAI_BASE_URL=http://localhost:8000/v1
OPENAI_MODEL=your-model
OPENAI_FAST_MODEL=your-model
```

A real API key is optional for a custom base URL unless the server requires
one. Compatibility means more than accepting plain chat text: the selected
model must reliably support JSON-schema function/tool calling and assistant
tool-call continuation. Streaming support is needed for incremental narrative
output. Text-only chat models may answer a basic prompt but cannot run the
game correctly.

At the provider boundary, Chronicler normalizes:

- system, user, assistant, image, and tool-result messages
- tool definitions, tool choices, and parallel tool calls
- streaming text and stop reasons
- provider usage accounting
- opaque OpenRouter reasoning details that must be replayed during a tool loop

Provider-specific wire data stays inside `chronicler-llm`; game code depends
only on the normalized request, response, content-block, and stream-event types.

## Programmatic Configuration

Applications embedding `chronicler-core` can use environment selection or
inject a client explicitly:

```rust
use chronicler_core::{
    GameSession, LlmClient, LlmConfig, SessionConfig, LOCAL_DEFAULT_MODEL,
};

// Resolve CHRONICLER_PROVIDER and related environment variables.
let environment_client = LlmClient::from_env()?;

// Or bypass environment selection completely.
let local_client = LlmClient::local(LOCAL_DEFAULT_MODEL)?;
let openrouter_client = LlmClient::from_config(
    LlmConfig::openrouter(openrouter_key)
        .with_model("moonshotai/kimi-k3")
        .with_fast_model("moonshotai/kimi-k3"),
)?;

let session = GameSession::new_with_client(
    SessionConfig::new("A Local Campaign"),
    local_client,
).await?;
```

`GameSession::load_with_client` provides the equivalent explicit-client path
when resuming a save. This dependency-injection boundary is also useful for
tests and host applications that manage credentials outside environment
variables.

## Saves and Provider Switching

Provider configuration and credentials are runtime dependencies, not campaign
state. Save files contain the world, character, conversation summary, and story
memory, but no API key or provider selection. You can therefore stop the game,
change provider configuration, and load the same campaign again.

Output quality and tool reliability can differ between models. Switching the
backend preserves game state, not identical prose or decisions.

## Credential Boundary

- Credentials are not logged or written to save files.
- A cloud credential is attached only to requests for the selected base URL.
- Custom base-URL overrides deliberately change that destination; review them
  before supplying a real credential.
- `--local` sends requests only to the built-in localhost Ollama URL and uses
  no credential.
- There is no automatic cloud fallback from a failed local request.

## Provider Smoke Tests

The reproducible smoke test exercises a real tool call, tool-result
continuation, and streaming response:

```bash
# Local Ollama; defaults to qwen3.6:35b-a3b
cargo run -p chronicler-llm --example provider_smoke -- ollama

# OpenRouter; requires OPENROUTER_API_KEY
cargo run -p chronicler-llm --example provider_smoke -- openrouter
```
