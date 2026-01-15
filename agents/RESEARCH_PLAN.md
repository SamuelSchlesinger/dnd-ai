# Agentic Framework Research Program

## Overview
This document tracks a comprehensive research program to inform the design of the `agentic` Rust framework and its associated agents.

## Research Structure

### Phase 1: Industry Survey
- Anthropic (Claude, MCP, tool use patterns)
- Google/DeepMind (Gemini, Vertex AI, agent architectures)
- OpenAI (Assistants API, GPT agents, function calling)
- Other frameworks (LangChain, CrewAI, AutoGen, DSPy, etc.)

### Phase 2: Domain-Specific Research (Lieutenants)

#### Planning Lieutenant
- Task decomposition strategies
- Goal hierarchies and subgoal generation
- Plan verification and revision
- Multi-step reasoning architectures

#### Continual Learning Lieutenant
- Memory systems (episodic, semantic, procedural)
- Feedback incorporation
- Self-improvement mechanisms
- Knowledge distillation

#### Creativity Lieutenant
- Novel solution generation
- Analogical reasoning
- Exploration vs exploitation
- Creative problem-solving patterns

#### Context Management Lieutenant
- Context window optimization
- Summarization strategies
- Retrieval-augmented generation
- State management across sessions

#### Safety Lieutenant
- Alignment techniques
- Guardrails and constraints
- Safe exploration
- Failure modes and mitigations

#### Security Lieutenant
- Secure code generation
- Prompt injection defenses
- Sandboxing strategies
- Audit and logging

#### Privacy Lieutenant
- PII handling
- Data minimization
- Differential privacy in agents
- User consent and transparency

#### Miscellaneous Lieutenant
- Tool use patterns
- Multi-agent coordination
- Human-in-the-loop design
- Evaluation and benchmarking
- Deployment considerations

## Research Outputs
Each lieutenant will produce:
1. Key findings summary
2. Best practices identified
3. Recommended patterns for our framework
4. Open questions and areas for further investigation

---

## Findings

### Context Management Lieutenant - COMPLETED

**Report**: [context_management_report.md](./context_management_report.md)

**Key Findings Summary**:

1. **Context Window Optimization**
   - Priority-based selection is essential: Critical > High > Medium > Low priority segments
   - Dynamic pruning outperforms static truncation
   - Compression techniques range from simple (token removal) to advanced (learned compression)

2. **Summarization Strategies**
   - Progressive summarization creates compression levels based on age
   - Hierarchical summarization enables selective expansion
   - Critical to preserve: named entities, numbers, decisions, tool results, user corrections

3. **RAG Integration**
   - Hybrid retrieval (dense + sparse) outperforms either alone
   - Agent-specific needs: tool docs, examples, conversation history, error patterns
   - Self-RAG pattern: agent decides when/what to retrieve

4. **State Management**
   - Three-tier memory: working (in-context) + session + long-term
   - Checkpointing essential for reliability
   - Multi-session continuity via structured summaries

5. **Practical Systems Analysis**
   - Claude Code: conversation compaction, intelligent tool output handling, prompt caching
   - LangChain: Multiple memory types (buffer, window, summary, vector)
   - MemGPT: Agent-controlled memory operations as explicit tools

**Recommended Patterns for Agentic Framework**:
- `ContextManager` with priority-based segment management
- `HierarchicalMemory` (working/session/long-term)
- Pluggable `Summarizer` trait with preservation rules
- Integrated `Retriever` trait with hybrid strategies
- `AgentCheckpoint` for serializable state

**Rust Design Considerations**:
- Token-aware strings that cache counts
- Arena allocation for efficient segment storage
- Async-friendly context building
- Copy-on-write segments for memory efficiency
- Serde with version compatibility

**Open Questions**:
- Optimal compression-fidelity tradeoffs
- Automatic priority assignment learning
- Cross-model context transfer
- Graceful degradation strategies

---

### Privacy Lieutenant - COMPLETED

**Report**: [research/privacy/REPORT.md](./research/privacy/REPORT.md)

**Key Findings Summary**:

1. **PII Handling**
   - Hybrid detection: Pattern-based (regex) + NER-based for contextual PII
   - Sensitivity classification: Public < Internal < Confidential < Sensitive < Restricted
   - Data minimization: Collection, storage, and retention minimization principles
   - Redaction strategies: Placeholder, type indicator, partial mask, consistent hash, tokenization

2. **Regulatory Compliance**
   - GDPR: Lawful basis, data subject rights (access, rectification, erasure, portability, objection)
   - CCPA: Consumer rights, Do Not Sell, non-discrimination
   - Audit requirements: Comprehensive logging with integrity protection
   - Cross-border transfers: Adequacy decisions, SCCs, explicit consent

3. **Privacy-Enhancing Techniques**
   - Differential privacy: Laplace/Gaussian mechanisms, privacy budget accounting
   - Federated learning: On-device processing, secure aggregation, gradient privacy
   - Privacy by design: Seven foundational principles, default privacy settings
   - Encryption: At-rest, in-transit, secure memory handling with zeroization

4. **Agent-Specific Privacy Patterns**
   - Input filtering and output sanitization pipelines
   - Memory privacy controls and data lineage tracking
   - Tool execution privacy controls for external API calls
   - Consent-aware processing with audit trails

**Recommended Patterns for Agentic Framework**:
- `PiiDetector` trait with hybrid pattern + NER implementation
- `SensitiveData<T>` wrapper with access tracking and lineage
- `SecretString` with auto-zeroization and debug-safe formatting
- `PrivacyModule` for wrapping agents with privacy protections
- `ConsentManager` for GDPR/CCPA compliance
- `AuditLog` with tamper-evident event recording

**Rust Design Considerations**:
- Type-level sensitivity enforcement with marker traits
- `Zeroizing<T>` for secure memory handling
- Privacy-safe builders that require explicit review
- Compile-time data flow restrictions

**Privacy Checklist Categories**:
- Design phase: Data inventory, PIA, legal basis, privacy by design
- Implementation: PII handling, security controls, consent, rights
- Runtime: Input scanning, memory encryption, tool validation, output sanitization
- Audit: Logging, reporting, testing, incident response

**Open Questions**:
- Semantic PII detection in natural language
- Privacy-utility tradeoffs for differential privacy epsilon values
- Verifiable deletion in vector stores
- AI Act implications for agent privacy

---

### Planning Lieutenant - COMPLETED

**Report**: [research/planning/REPORT.md](./research/planning/REPORT.md)

**Key Findings Summary**:

1. **Task Decomposition Architectures**
   - HTN (Hierarchical Task Networks): Best for known domains with expert-defined decomposition methods
   - GOAP (Goal-Oriented Action Planning): Best for dynamic environments with well-defined action effects
   - Tree-of-Thought (ToT): Best for complex reasoning with branching solution paths
   - Graph-of-Thought (GoT): Best for problems requiring solution aggregation/synthesis
   - LLM-native patterns: Chain-of-thought, self-ask, decomposed prompting, least-to-most

2. **Plan Verification & Replanning**
   - Layered validation: syntactic, precondition, effect, resource, temporal, safety
   - Runtime monitoring: precondition checking, effect verification, invariant monitoring
   - Replanning strategies: plan repair (local) vs. full replanning
   - Failure handling: graceful degradation, retry with backoff, error classification
   - Uncertainty handling: probabilistic planning, contingency plans

3. **Goal Management**
   - Goal hierarchies with parent-child relationships and status propagation
   - Priority schemes: Eisenhower matrix (urgency + importance), category weighting
   - Conflict types: state conflicts, resource conflicts, temporal conflicts, causal conflicts
   - Resolution strategies: priority-based, negotiation-based, constraint relaxation
   - BDI (Belief-Desire-Intention) architecture for goal-directed agents

**Recommended Patterns for Agentic Framework**:
- `Planner` trait with hierarchical composition
- `PlanValidator` with layered validation passes
- `ExecutionMonitor` for runtime checking
- `GoalManager` with hierarchy and conflict detection
- `PlanExecutor` with retry and replanning policies
- BDI-style agent loop for goal-directed behavior

**Rust Design Considerations**:
- Phantom types for plan state (Validated/Unvalidated/Executing)
- Type-safe world state with `TypeId`-based storage
- Builder pattern for complex goal construction
- `thiserror` for comprehensive error handling
- Async-friendly planning with cancellation support
- String interning for memory efficiency

**Open Questions**:
- Optimal LLM decomposition granularity
- When to use classical vs. neural planners
- Plan caching and reuse strategies
- Learning prioritization from user feedback
- Goal safety verification

---

(Additional findings to be populated by other research agents)
