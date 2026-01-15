# AI Agent Frameworks Industry Survey Report

**Date:** January 2026
**Scope:** Major AI providers and open-source frameworks for building AI agents

---

## Executive Summary

This report surveys the landscape of AI agent frameworks from major providers (Anthropic, Google, OpenAI) and prominent open-source projects (LangChain/LangGraph, CrewAI, AutoGen, DSPy, Semantic Kernel). The analysis identifies common patterns, unique innovations, and provides recommendations for building a Rust-based agent framework.

---

## 1. Anthropic (Claude)

### 1.1 Tool Use Architecture

Claude's tool use follows a **request-response model** where:
- Tools are defined via JSON Schema with `name`, `description`, and `input_schema`
- Claude generates `tool_use` content blocks when it wants to call a tool
- The client executes the tool and returns a `tool_result` content block
- Claude incorporates results and continues reasoning

**Key Features:**
- **Streaming support**: Tool calls can be streamed as they're generated
- **Parallel tool calls**: Multiple tools can be requested in a single response
- **Tool choice modes**: `auto`, `any`, `tool` (force specific tool), `none`
- **Structured outputs**: Schema-validated JSON responses

### 1.2 Model Context Protocol (MCP)

MCP is Anthropic's open standard for connecting AI applications to external systems.

**Architecture (Client-Server):**
```
MCP Host (e.g., Claude Desktop)
    └── MCP Client
            └── MCP Server (exposes tools, resources, prompts)
```

**Core Primitives:**
| Primitive | Purpose | Control Model |
|-----------|---------|---------------|
| **Tools** | Executable functions | Model-controlled (AI decides when to use) |
| **Resources** | Data sources (files, DBs, APIs) | Application-controlled |
| **Prompts** | Reusable templates | User-controlled (explicit selection) |

**Protocol Details:**
- JSON-RPC 2.0 based messaging
- Two transport options: **stdio** (local subprocess) and **Streamable HTTP** (remote)
- Dynamic capability discovery via `tools/list`, `resources/list`, `prompts/list`
- Real-time notifications when capabilities change
- Session management for HTTP transport

**Advanced Features:**
- **Sampling**: Servers can request LLM completions from the client
- **Elicitation**: Servers can request user input
- **Resource subscriptions**: Real-time updates when resources change

### 1.3 Claude Code Patterns

Claude Code demonstrates production agent patterns:
- **Agentic loop**: Tool calls -> execution -> result integration -> continue
- **Sandboxed execution**: File and command execution in restricted environment
- **Context management**: Efficient handling of conversation history
- **Progressive disclosure**: Tools reveal information as needed

### 1.4 Limitations
- Tool descriptions consume context window
- No native persistent state between conversations
- Rate limits on tool calls in high-frequency scenarios

---

## 2. Google (Gemini)

### 2.1 Function Calling

Gemini supports function calling with similar patterns to Claude:
- Function declarations with JSON Schema parameters
- Automatic function detection and invocation
- Support for parallel function calls
- Grounding with Google Search for real-time data

**Key Features:**
- **Code execution**: Built-in Python sandbox for generated code
- **Grounding**: Direct access to Google Search and custom data stores
- **Multi-modal**: Functions can process images, audio, and video

### 2.2 Vertex AI Agent Builder

Google's enterprise platform for building agents:
- **Visual agent builder**: No-code/low-code interface
- **Playbooks**: Define agent behavior through natural language instructions
- **Data stores**: Connect agents to enterprise data sources
- **Extensions**: Pre-built integrations (Google Workspace, databases)

### 2.3 Agent Development Kit (ADK)

Google's open-source framework for building agents:
- **Agent composition**: Hierarchical agent structures
- **Tool integration**: Native tools, extensions, and custom functions
- **Multi-agent orchestration**: Coordinate multiple specialized agents
- **Evaluation framework**: Test and measure agent performance

### 2.4 Unique Innovations
- **Grounding API**: Real-time factual accuracy via Google Search
- **Native multi-modality**: Seamless handling of text, images, audio, video
- **Vertex AI integration**: Enterprise-grade deployment and monitoring

### 2.5 Limitations
- Tighter coupling to Google Cloud ecosystem
- Less community tooling compared to OpenAI
- Grounding requires Vertex AI (not available in base API)

---

## 3. OpenAI (GPT)

### 3.1 Function Calling

OpenAI pioneered the function calling paradigm:
- JSON Schema tool definitions
- `tool_choice` parameter for control (`auto`, `required`, `none`, specific tool)
- Parallel function calling (multiple in single response)
- Structured outputs with `strict: true` mode

**Evolution:**
1. **Functions** (legacy): Single function per response
2. **Tools** (current): Multiple tools, parallel calls, richer metadata
3. **Structured Outputs**: Guaranteed JSON schema compliance

### 3.2 Assistants API

Managed agent infrastructure:
- **Threads**: Persistent conversation history
- **Runs**: Execution instances with status tracking
- **Built-in tools**: Code Interpreter, File Search, Function Calling
- **Vector stores**: Automatic RAG for uploaded files

**Key Features:**
- State management handled by OpenAI
- Automatic context window management
- File handling and code execution sandbox

### 3.3 Swarm Framework

OpenAI's experimental multi-agent pattern:
- **Agents**: Defined by instructions and tools
- **Handoffs**: Agents transfer control to other agents
- **Routines**: Multi-step processes with agent coordination
- **Context variables**: Shared state across agents

**Core Concepts:**
```python
Agent(
    name="Triage",
    instructions="Route user to appropriate specialist",
    functions=[transfer_to_sales, transfer_to_support]
)
```

**Handoff Pattern:**
- Agent returns another agent to transfer control
- Context variables persist across handoffs
- Explicit vs implicit handoff strategies

### 3.4 Unique Innovations
- **Code Interpreter**: Sandboxed Python with file I/O
- **File Search**: Automatic RAG with vector stores
- **Assistants threads**: Managed conversation persistence

### 3.5 Limitations
- Assistants API latency for complex operations
- Limited control over RAG behavior in File Search
- Swarm is experimental, not production-ready

---

## 4. Open-Source Frameworks

### 4.1 LangChain / LangGraph

**LangChain Core:**
- **Chains**: Sequential processing pipelines
- **Agents**: LLM-driven tool selection and execution
- **Memory**: Conversation history management
- **Retrievers**: Document retrieval for RAG

**LangGraph:**
- **Graph-based workflows**: Nodes, edges, conditional routing
- **State management**: Typed state schemas, checkpointing
- **Human-in-the-loop**: Interrupt and resume capabilities
- **Multi-agent patterns**: Supervisor, hierarchical, swarm

**Key Patterns:**
```
StateGraph -> Add Nodes -> Add Edges -> Compile -> Execute
```

**Strengths:**
- Most extensive ecosystem of integrations
- Production-ready with LangSmith observability
- Flexible composition patterns

**Limitations:**
- Complexity for simple use cases
- Performance overhead from abstraction layers
- Python-centric (JS/TS secondary)

### 4.2 CrewAI

**Architecture:**
- **Agents**: Role-based with goals and backstories
- **Tasks**: Specific objectives with expected outputs
- **Crews**: Coordinated groups of agents
- **Tools**: Shared or agent-specific capabilities

**Unique Features:**
- **Role-playing**: Agents adopt personas for better task execution
- **Delegation**: Agents can delegate subtasks
- **Sequential/Parallel processes**: Flexible execution order
- **Memory**: Short-term, long-term, and entity memory

**Example:**
```python
researcher = Agent(role="Research Analyst", goal="Find accurate data")
writer = Agent(role="Content Writer", goal="Create compelling content")
crew = Crew(agents=[researcher, writer], tasks=[research_task, write_task])
```

### 4.3 AutoGen (Microsoft)

**Architecture:**
- **Conversable Agents**: Agents that communicate via messages
- **Group Chat**: Multi-agent conversations with managers
- **Code Execution**: Sandboxed execution environments
- **Human Proxy**: Human-in-the-loop integration

**Key Patterns:**
- **Two-agent chat**: Simple back-and-forth dialogue
- **Group chat**: Multiple agents with speaker selection
- **Nested chat**: Sub-conversations within main flow
- **Sequential chat**: Ordered agent interactions

**Strengths:**
- Natural conversational paradigm
- Flexible human-in-the-loop patterns
- Strong code generation and execution

### 4.4 DSPy

**Philosophy:** Programming (not prompting) LLMs

**Core Concepts:**
- **Signatures**: Typed input/output specifications
- **Modules**: Composable LLM operations
- **Optimizers**: Automatic prompt optimization
- **Assertions**: Runtime constraints on outputs

**Example:**
```python
class RAG(dspy.Module):
    def __init__(self):
        self.retrieve = dspy.Retrieve(k=3)
        self.generate = dspy.ChainOfThought("context, question -> answer")

    def forward(self, question):
        context = self.retrieve(question)
        return self.generate(context=context, question=question)
```

**Unique Innovation:** Treats prompts as learnable parameters, optimized via examples.

### 4.5 Semantic Kernel (Microsoft)

**Architecture:**
- **Kernel**: Central orchestrator
- **Plugins**: Collections of functions (native or OpenAPI)
- **Memory**: Vector stores for semantic search
- **Planners**: Automatic task decomposition

**Plugin System:**
```csharp
[KernelFunction("get_weather")]
[Description("Get current weather for a location")]
public async Task<Weather> GetWeatherAsync(string location) { }
```

**Key Features:**
- Enterprise-focused design
- Multi-language support (C#, Python, Java)
- Native integration with Azure AI services
- Function calling abstraction

---

## 5. Common Patterns Across Providers

### 5.1 Tool/Function Definition
All frameworks use JSON Schema for tool definitions:
```json
{
  "name": "tool_name",
  "description": "What the tool does",
  "parameters": {
    "type": "object",
    "properties": { ... },
    "required": [ ... ]
  }
}
```

### 5.2 Agent Loop Pattern
```
┌─────────────────────────────────────────────┐
│ User Input                                  │
└─────────────────┬───────────────────────────┘
                  ▼
┌─────────────────────────────────────────────┐
│ LLM Reasoning                               │
│ (Decide: respond or use tool)               │
└─────────────────┬───────────────────────────┘
                  ▼
        ┌─────────┴─────────┐
        │                   │
        ▼                   ▼
┌───────────────┐   ┌───────────────┐
│ Tool Call     │   │ Final Response│
└───────┬───────┘   └───────────────┘
        ▼
┌───────────────┐
│ Tool Result   │
└───────┬───────┘
        │
        └────────► (back to LLM Reasoning)
```

### 5.3 Multi-Agent Patterns

| Pattern | Description | Used By |
|---------|-------------|---------|
| **Supervisor** | Central agent routes to specialists | LangGraph, CrewAI |
| **Handoff** | Agents transfer control explicitly | Swarm, AutoGen |
| **Hierarchical** | Manager agents coordinate workers | CrewAI, AutoGen |
| **Collaborative** | Agents work in parallel, results merged | LangGraph |
| **Debate** | Agents argue to refine answers | AutoGen |

### 5.4 State Management

| Approach | Framework | Description |
|----------|-----------|-------------|
| **External persistence** | Claude, Gemini | Client manages state |
| **Managed threads** | OpenAI Assistants | Platform manages state |
| **Graph state** | LangGraph | Typed state in graph |
| **Context variables** | Swarm | Dict passed between agents |
| **Memory stores** | CrewAI, SK | Short/long-term memory |

### 5.5 Human-in-the-Loop

All frameworks support human intervention:
- **Tool approval**: Confirm before execution
- **Breakpoints**: Pause at specific steps
- **Edit and continue**: Modify intermediate results
- **Escalation**: Route to human for complex decisions

---

## 6. Unique Innovations Summary

| Provider | Key Innovation |
|----------|----------------|
| **Anthropic** | MCP protocol for universal tool integration |
| **Google** | Native multi-modality and grounding |
| **OpenAI** | Code Interpreter and managed vector stores |
| **LangGraph** | Graph-based workflow composition |
| **CrewAI** | Role-playing agent personas |
| **AutoGen** | Conversational agent paradigm |
| **DSPy** | Prompt optimization as machine learning |
| **Semantic Kernel** | Enterprise plugin architecture |

---

## 7. Recommendations for Rust Framework

### 7.1 Core Architecture

```rust
// Trait-based tool definition
trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters(&self) -> JsonSchema;
    async fn execute(&self, args: Value) -> Result<ToolResult>;
}

// Agent trait
trait Agent: Send + Sync {
    async fn step(&mut self, context: &Context) -> Result<AgentAction>;
}

enum AgentAction {
    Respond(String),
    ToolCall(ToolCall),
    Handoff(Box<dyn Agent>),
    Terminate,
}
```

### 7.2 Recommended Patterns

1. **Implement MCP Protocol**
   - Use MCP as the standard for tool integration
   - Benefits: Ecosystem compatibility, proven design
   - Implementation: JSON-RPC over stdio/HTTP

2. **Graph-Based Workflows**
   - Model agent flows as directed graphs
   - Nodes: Agent steps, tool calls, conditionals
   - Edges: Control flow with conditions
   - State: Typed state machine

3. **Typed Tool Schemas**
   - Leverage Rust's type system
   - Derive JSON Schema from Rust types
   - Compile-time validation of tool definitions

4. **Async-First Design**
   - All tool execution async
   - Parallel tool calls with `join!`
   - Streaming responses via `Stream` trait

5. **Pluggable LLM Backends**
   - Abstract over providers (Claude, GPT, Gemini)
   - Unified tool calling interface
   - Provider-specific optimizations

### 7.3 Key Traits to Implement

```rust
// Core traits for the framework
trait LLM {
    async fn complete(&self, messages: &[Message], tools: &[Tool])
        -> Result<Response>;
}

trait Memory {
    async fn store(&self, key: &str, value: Value) -> Result<()>;
    async fn retrieve(&self, key: &str) -> Result<Option<Value>>;
    async fn search(&self, query: &str, k: usize) -> Result<Vec<Value>>;
}

trait Transport {
    async fn send(&self, message: JsonRpcMessage) -> Result<()>;
    async fn receive(&self) -> Result<JsonRpcMessage>;
}
```

### 7.4 Priority Features

| Priority | Feature | Rationale |
|----------|---------|-----------|
| **P0** | Tool definition and execution | Core agent capability |
| **P0** | MCP client support | Ecosystem compatibility |
| **P1** | Streaming responses | User experience |
| **P1** | Multi-agent coordination | Complex workflows |
| **P2** | MCP server support | Framework as tool provider |
| **P2** | Built-in tools (FS, HTTP) | Common operations |
| **P3** | Optimization/evaluation | Production quality |

---

## 8. Comparative Analysis

| Feature | Claude/MCP | OpenAI | Google | LangGraph | CrewAI |
|---------|------------|--------|--------|-----------|--------|
| **Tool Definition** | JSON Schema | JSON Schema | JSON Schema | Python dict | Python class |
| **Parallel Tools** | Yes | Yes | Yes | Yes | Yes |
| **Streaming** | Yes | Yes | Yes | Yes | Limited |
| **Multi-Agent** | Manual | Swarm | ADK | Native | Native |
| **State Management** | Client-side | Managed | Client-side | Graph state | Memory |
| **Protocol Standard** | MCP | None | None | None | None |
| **Open Source** | MCP only | Swarm only | ADK | Yes | Yes |
| **Rust Support** | MCP SDK | None | None | None | None |
| **Production Ready** | Yes | Yes | Yes | Yes | Growing |

---

## 9. Conclusion

The AI agent framework landscape shows convergence on core patterns (JSON Schema tools, agentic loops, multi-agent coordination) while each provider adds unique innovations. For a Rust framework, we recommend:

1. **Adopt MCP** as the standard protocol for tool integration
2. **Use graph-based workflows** for complex agent coordination
3. **Leverage Rust's type system** for compile-time safety
4. **Build abstractions** that work across LLM providers
5. **Prioritize streaming and async** for responsive agents

The combination of Rust's performance and safety guarantees with proven agent patterns from the industry leaders positions the framework well for production use cases requiring reliability and efficiency.

---

## References

- Anthropic MCP Documentation: https://modelcontextprotocol.io
- OpenAI API Documentation: https://platform.openai.com/docs
- Google AI Documentation: https://ai.google.dev
- LangChain Documentation: https://python.langchain.com
- Semantic Kernel Documentation: https://learn.microsoft.com/semantic-kernel
- CrewAI Documentation: https://docs.crewai.com
- AutoGen Documentation: https://microsoft.github.io/autogen
- DSPy Documentation: https://dspy.ai
