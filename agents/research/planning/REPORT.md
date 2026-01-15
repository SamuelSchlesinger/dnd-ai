# Planning and Task Decomposition in AI Agents: Research Report

## Executive Summary

Planning is the cognitive backbone of intelligent agents. While language models can generate impressive outputs, converting high-level goals into executable action sequences remains a fundamental challenge. This report synthesizes research and best practices across three critical domains: **task decomposition** (breaking down complex problems), **plan verification and replanning** (ensuring robustness), and **goal management** (handling multiple competing objectives). We provide actionable patterns for implementing these capabilities in a Rust-based agent framework.

---

## 1. Task Decomposition Architectures

### 1.1 The Fundamental Challenge

Complex tasks require breaking down into manageable subtasks. An agent asked to "build a web application" must decompose this into hundreds of concrete steps: setting up a project, defining data models, implementing routes, writing tests, etc. The quality of this decomposition directly impacts agent success.

Key questions for any decomposition approach:
- **Granularity**: How fine-grained should subtasks be?
- **Ordering**: What dependencies exist between subtasks?
- **Adaptability**: How to handle unexpected situations during execution?

### 1.2 Hierarchical Task Networks (HTN)

HTN planning is a classical AI approach that decomposes tasks through a hierarchy of methods.

**Core Concepts**:
```
Task Types:
- Primitive Tasks: Directly executable actions
- Compound Tasks: Abstract tasks requiring decomposition

Methods:
- Each compound task has one or more decomposition methods
- Methods specify how to break down a compound task into subtasks
- Method selection can depend on current world state
```

**Example HTN Structure**:
```
Compound: BuildWebApp
├── Method 1 (if using React):
│   ├── SetupReactProject (compound)
│   ├── ImplementComponents (compound)
│   └── DeployToVercel (primitive)
└── Method 2 (if using static site):
    ├── SetupHugo (primitive)
    ├── CreateContent (compound)
    └── DeployToNetlify (primitive)

Compound: SetupReactProject
└── Method:
    ├── CreateViteProject (primitive)
    ├── InstallDependencies (primitive)
    └── ConfigureESLint (primitive)
```

**Strengths**:
- Natural representation of expert knowledge
- Handles complex dependencies through ordering constraints
- Supports multiple decomposition strategies
- Efficient plan generation when good methods exist

**Weaknesses**:
- Requires pre-defined method library
- Brittleness when encountering novel situations
- Method engineering is labor-intensive
- Combinatorial explosion with many methods

**When to Use**:
- Domains with well-understood task structures
- Repetitive workflows that can be templated
- When expert knowledge is available for encoding

**Rust Implementation Pattern**:
```rust
/// A task that can be either primitive (executable) or compound (needs decomposition)
pub enum Task {
    Primitive(PrimitiveTask),
    Compound(CompoundTask),
}

pub struct PrimitiveTask {
    pub name: String,
    pub preconditions: Vec<Condition>,
    pub effects: Vec<Effect>,
    pub executor: Box<dyn TaskExecutor>,
}

pub struct CompoundTask {
    pub name: String,
    pub methods: Vec<DecompositionMethod>,
}

pub struct DecompositionMethod {
    pub name: String,
    /// Conditions under which this method applies
    pub applicability: Vec<Condition>,
    /// Subtasks this method produces (ordered)
    pub subtasks: Vec<Task>,
    /// Priority when multiple methods apply
    pub priority: i32,
}

pub trait HTNPlanner {
    /// Decompose a task into primitive actions given current state
    fn plan(&self, task: &Task, state: &WorldState) -> Result<Vec<PrimitiveTask>, PlanError>;

    /// Check if a method is applicable in current state
    fn method_applicable(&self, method: &DecompositionMethod, state: &WorldState) -> bool;

    /// Select best method when multiple apply
    fn select_method(&self, methods: &[&DecompositionMethod], state: &WorldState) -> Option<&DecompositionMethod>;
}
```

### 1.3 Goal-Oriented Action Planning (GOAP)

GOAP originated in game AI and uses state-space search to find action sequences that achieve goals.

**Core Concepts**:
```
World State: Set of boolean or symbolic facts about the world
Goal: Desired world state (partial specification)
Action:
  - Preconditions: State facts that must be true to execute
  - Effects: State changes that result from execution
  - Cost: Resource cost for planning optimization

Planning: Search backward from goal or forward from current state
```

**Example GOAP Setup**:
```
World State: {
    hasCode: false,
    testsPass: false,
    isDeployed: false,
    hasDatabase: false
}

Goal: { isDeployed: true, testsPass: true }

Actions:
- WriteCode:
    preconditions: {}
    effects: { hasCode: true }
    cost: 10

- SetupDatabase:
    preconditions: {}
    effects: { hasDatabase: true }
    cost: 5

- RunTests:
    preconditions: { hasCode: true, hasDatabase: true }
    effects: { testsPass: true }
    cost: 2

- Deploy:
    preconditions: { hasCode: true, testsPass: true }
    effects: { isDeployed: true }
    cost: 3

Generated Plan: SetupDatabase -> WriteCode -> RunTests -> Deploy
```

**Strengths**:
- Flexible: no pre-defined task hierarchies needed
- Optimal: can find lowest-cost plans
- Reactive: easy to replan when state changes
- Emergent behavior from simple action definitions

**Weaknesses**:
- Combinatorial explosion in large state spaces
- Requires accurate world state modeling
- Planning can be slow for complex goals
- No natural support for partial-order planning

**When to Use**:
- Dynamic environments where conditions change
- When action effects are well-defined
- Game AI and robotics applications
- When you want emergent behavior

**Rust Implementation Pattern**:
```rust
use std::collections::{HashMap, BinaryHeap, HashSet};

/// Symbolic world state as a set of facts
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct WorldState {
    facts: HashMap<String, bool>,
}

impl WorldState {
    pub fn satisfies(&self, goal: &WorldState) -> bool {
        goal.facts.iter().all(|(k, v)| self.facts.get(k) == Some(v))
    }

    pub fn apply_effects(&mut self, effects: &HashMap<String, bool>) {
        for (k, v) in effects {
            self.facts.insert(k.clone(), *v);
        }
    }
}

pub struct GOAPAction {
    pub name: String,
    pub preconditions: HashMap<String, bool>,
    pub effects: HashMap<String, bool>,
    pub cost: u32,
}

impl GOAPAction {
    pub fn is_applicable(&self, state: &WorldState) -> bool {
        self.preconditions.iter().all(|(k, v)| state.facts.get(k) == Some(v))
    }
}

pub struct GOAPPlanner {
    actions: Vec<GOAPAction>,
}

impl GOAPPlanner {
    /// A* search from current state to goal
    pub fn plan(&self, start: WorldState, goal: WorldState) -> Option<Vec<String>> {
        // Use A* with heuristic = number of unsatisfied goal conditions
        let mut open_set = BinaryHeap::new();
        let mut came_from: HashMap<WorldState, (WorldState, String)> = HashMap::new();
        let mut g_score: HashMap<WorldState, u32> = HashMap::new();

        g_score.insert(start.clone(), 0);
        open_set.push(PlanNode { state: start.clone(), f_score: 0 });

        while let Some(current) = open_set.pop() {
            if current.state.satisfies(&goal) {
                return Some(self.reconstruct_path(&came_from, &current.state));
            }

            for action in &self.actions {
                if action.is_applicable(&current.state) {
                    let mut next_state = current.state.clone();
                    next_state.apply_effects(&action.effects);

                    let tentative_g = g_score.get(&current.state).unwrap() + action.cost;

                    if tentative_g < *g_score.get(&next_state).unwrap_or(&u32::MAX) {
                        came_from.insert(next_state.clone(), (current.state.clone(), action.name.clone()));
                        g_score.insert(next_state.clone(), tentative_g);
                        let h = self.heuristic(&next_state, &goal);
                        open_set.push(PlanNode { state: next_state, f_score: tentative_g + h });
                    }
                }
            }
        }

        None
    }

    fn heuristic(&self, state: &WorldState, goal: &WorldState) -> u32 {
        goal.facts.iter()
            .filter(|(k, v)| state.facts.get(*k) != Some(*v))
            .count() as u32
    }

    fn reconstruct_path(&self, came_from: &HashMap<WorldState, (WorldState, String)>, goal: &WorldState) -> Vec<String> {
        let mut path = Vec::new();
        let mut current = goal.clone();
        while let Some((prev, action)) = came_from.get(&current) {
            path.push(action.clone());
            current = prev.clone();
        }
        path.reverse();
        path
    }
}

#[derive(Eq, PartialEq)]
struct PlanNode {
    state: WorldState,
    f_score: u32,
}

impl Ord for PlanNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.f_score.cmp(&self.f_score) // Reverse for min-heap
    }
}
impl PartialOrd for PlanNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
```

### 1.4 Tree-of-Thought (ToT)

Tree-of-Thought extends chain-of-thought prompting with explicit tree search over reasoning paths.

**Core Concepts**:
```
Thought: Intermediate reasoning step
Tree: Multiple thoughts branch from each state
Evaluation: Score thoughts for promise/correctness
Search: BFS, DFS, or beam search through thought tree
```

**ToT Process**:
```
Problem: "Implement a rate limiter"

Root
├── Thought 1: "Use token bucket algorithm"
│   ├── Thought 1.1: "Fixed window token bucket"
│   │   └── [Evaluation: 0.7 - simple but bursty]
│   └── Thought 1.2: "Sliding window token bucket"
│       └── [Evaluation: 0.9 - smooth limiting]
├── Thought 2: "Use leaky bucket algorithm"
│   └── [Evaluation: 0.6 - good for smoothing but complex]
└── Thought 3: "Use fixed window counter"
    └── [Evaluation: 0.5 - simple but edge case issues]

Selected Path: 1 -> 1.2 (Sliding window token bucket)
```

**Strengths**:
- Explores multiple solution paths
- Can backtrack from dead ends
- Naturally handles problems with many valid approaches
- Self-evaluation improves quality

**Weaknesses**:
- High token cost (multiple LLM calls)
- Latency from tree traversal
- Evaluation quality depends on LLM judgment
- May explore irrelevant branches

**When to Use**:
- Complex reasoning problems
- Tasks with multiple valid approaches
- When wrong paths are recoverable
- Mathematical or logical reasoning

**Rust Implementation Pattern**:
```rust
use std::collections::VecDeque;

pub struct ThoughtNode {
    pub content: String,
    pub evaluation_score: f32,
    pub children: Vec<ThoughtNode>,
    pub depth: usize,
}

pub struct TreeOfThought<L: LLMClient> {
    llm: L,
    max_depth: usize,
    branching_factor: usize,
    search_strategy: SearchStrategy,
}

pub enum SearchStrategy {
    BFS,
    DFS { max_depth: usize },
    BeamSearch { beam_width: usize },
}

impl<L: LLMClient> TreeOfThought<L> {
    pub async fn solve(&self, problem: &str) -> Result<String, PlanError> {
        let root = self.generate_initial_thoughts(problem).await?;

        match self.search_strategy {
            SearchStrategy::BFS => self.bfs_search(root).await,
            SearchStrategy::DFS { max_depth } => self.dfs_search(root, max_depth).await,
            SearchStrategy::BeamSearch { beam_width } => self.beam_search(root, beam_width).await,
        }
    }

    async fn generate_initial_thoughts(&self, problem: &str) -> Result<ThoughtNode, PlanError> {
        let prompt = format!(
            "Problem: {}\n\nGenerate {} different approaches to solve this problem.",
            problem, self.branching_factor
        );

        let thoughts = self.llm.generate(&prompt).await?;
        let children = self.parse_and_evaluate_thoughts(&thoughts).await?;

        Ok(ThoughtNode {
            content: problem.to_string(),
            evaluation_score: 1.0,
            children,
            depth: 0,
        })
    }

    async fn evaluate_thought(&self, thought: &str, context: &str) -> Result<f32, PlanError> {
        let prompt = format!(
            "Context: {}\nThought: {}\n\nRate this thought from 0.0 to 1.0 based on:\n\
             - Correctness\n- Progress toward solution\n- Feasibility\n\nScore:",
            context, thought
        );

        let response = self.llm.generate(&prompt).await?;
        response.trim().parse().map_err(|_| PlanError::EvaluationFailed)
    }

    async fn beam_search(&self, root: ThoughtNode, beam_width: usize) -> Result<String, PlanError> {
        let mut beam = vec![root];

        for depth in 0..self.max_depth {
            let mut candidates = Vec::new();

            for node in &beam {
                if self.is_solution(&node).await? {
                    return Ok(self.extract_solution(node));
                }

                let children = self.expand_node(node).await?;
                candidates.extend(children);
            }

            // Keep top beam_width candidates
            candidates.sort_by(|a, b| b.evaluation_score.partial_cmp(&a.evaluation_score).unwrap());
            beam = candidates.into_iter().take(beam_width).collect();
        }

        // Return best path found
        beam.into_iter()
            .max_by(|a, b| a.evaluation_score.partial_cmp(&b.evaluation_score).unwrap())
            .map(|n| self.extract_solution(&n))
            .ok_or(PlanError::NoSolutionFound)
    }
}
```

### 1.5 Graph-of-Thought (GoT)

Graph-of-Thought generalizes ToT by allowing non-linear thought structures and thought aggregation.

**Core Concepts**:
```
Operations:
- Generate: Create new thoughts from existing ones
- Aggregate: Combine multiple thoughts into one
- Refine: Improve a thought based on feedback
- Score: Evaluate thought quality

Graph Structure:
- Nodes: Individual thoughts
- Edges: Derivation relationships (can be many-to-many)
- Cycles: Iterative refinement allowed
```

**GoT Example**:
```
Task: "Design a caching system"

[T1: Consider write-through]  [T2: Consider write-back]  [T3: Consider read-through]
         \                          |                         /
          \                         |                        /
           \                        v                       /
            -----> [T4: Compare write strategies] <--------
                           |
                           v
            [T5: Hybrid approach - write-back with periodic sync]
                           |
                           v
                    [T6: Add TTL]
                           |
                           v
                    [T7: Implementation plan]
```

**Strengths**:
- Supports thought aggregation (combining insights)
- Allows iterative refinement loops
- More natural representation of complex reasoning
- Can reuse partial solutions

**Weaknesses**:
- More complex orchestration
- Higher computational overhead
- Difficult to determine when to aggregate vs. explore
- Risk of cycles without progress

**When to Use**:
- Problems benefiting from solution combination
- Multi-perspective analysis
- Complex design decisions
- When partial solutions can be merged

**Rust Implementation Pattern**:
```rust
use std::collections::HashMap;
use petgraph::graph::{DiGraph, NodeIndex};

pub struct Thought {
    pub id: ThoughtId,
    pub content: String,
    pub score: f32,
    pub thought_type: ThoughtType,
}

pub enum ThoughtType {
    Initial,
    Generated,
    Aggregated { sources: Vec<ThoughtId> },
    Refined { from: ThoughtId },
}

pub struct GraphOfThought<L: LLMClient> {
    llm: L,
    graph: DiGraph<Thought, EdgeType>,
    thoughts: HashMap<ThoughtId, NodeIndex>,
}

pub enum EdgeType {
    Generates,      // Parent generates child
    Aggregates,     // Multiple parents combine into child
    Refines,        // Parent is refined into child
}

pub enum Operation {
    Generate { from: ThoughtId, count: usize },
    Aggregate { sources: Vec<ThoughtId> },
    Refine { thought: ThoughtId },
    Score { thought: ThoughtId },
}

impl<L: LLMClient> GraphOfThought<L> {
    pub async fn execute(&mut self, op: Operation) -> Result<Vec<ThoughtId>, PlanError> {
        match op {
            Operation::Generate { from, count } => {
                let parent = self.get_thought(&from)?;
                let new_thoughts = self.llm_generate(&parent.content, count).await?;

                let mut new_ids = Vec::new();
                for content in new_thoughts {
                    let id = self.add_thought(content, ThoughtType::Generated);
                    self.add_edge(&from, &id, EdgeType::Generates);
                    new_ids.push(id);
                }
                Ok(new_ids)
            }

            Operation::Aggregate { sources } => {
                let contents: Vec<_> = sources.iter()
                    .filter_map(|id| self.get_thought(id).ok())
                    .map(|t| t.content.as_str())
                    .collect();

                let aggregated = self.llm_aggregate(&contents).await?;
                let id = self.add_thought(aggregated, ThoughtType::Aggregated {
                    sources: sources.clone()
                });

                for source in &sources {
                    self.add_edge(source, &id, EdgeType::Aggregates);
                }
                Ok(vec![id])
            }

            Operation::Refine { thought } => {
                let original = self.get_thought(&thought)?;
                let refined = self.llm_refine(&original.content).await?;
                let id = self.add_thought(refined, ThoughtType::Refined { from: thought.clone() });
                self.add_edge(&thought, &id, EdgeType::Refines);
                Ok(vec![id])
            }

            Operation::Score { thought } => {
                let t = self.get_thought_mut(&thought)?;
                t.score = self.llm_evaluate(&t.content).await?;
                Ok(vec![thought])
            }
        }
    }

    /// Get the best thought path to a solution
    pub fn best_path(&self) -> Vec<&Thought> {
        // Find highest-scored leaf nodes and trace back to root
        // Implementation depends on specific scoring strategy
        todo!()
    }
}
```

### 1.6 LLM-Native Decomposition Patterns

Modern LLM agents use several prompting patterns for task decomposition:

**Chain-of-Thought (CoT)**:
```
Simple sequential reasoning
Step 1 -> Step 2 -> Step 3 -> Answer

Best for: Linear problems, math, logic
```

**Self-Ask**:
```
Q: Main question
  -> Q: What subquestion do I need to answer first?
  -> A: Subquestion answer
  -> Q: What next?
  -> A: ...
  -> Final Answer

Best for: Multi-hop reasoning, research tasks
```

**Decomposed Prompting**:
```
Task -> [Subtask 1] -> [Subtask 2] -> [Subtask 3] -> Combined Result

Each subtask can use specialized prompt/tool
Best for: Modular tasks, when subtasks need different capabilities
```

**Least-to-Most Prompting**:
```
1. Decompose into subproblems (smallest to largest)
2. Solve smallest subproblem
3. Use solution to solve next larger subproblem
4. Continue until original problem solved

Best for: Problems with natural size progression
```

**Rust Implementation Pattern**:
```rust
/// Unified interface for decomposition strategies
pub trait Decomposer: Send + Sync {
    /// Break a task into subtasks
    fn decompose(&self, task: &Task, context: &Context)
        -> impl Future<Output = Result<Vec<Subtask>, DecomposeError>>;

    /// Check if further decomposition is needed
    fn needs_decomposition(&self, task: &Task) -> bool;
}

pub struct ChainOfThoughtDecomposer<L: LLMClient> {
    llm: L,
    max_steps: usize,
}

pub struct SelfAskDecomposer<L: LLMClient> {
    llm: L,
    max_depth: usize,
}

pub struct LeastToMostDecomposer<L: LLMClient> {
    llm: L,
}

/// Composable decomposition strategy
pub struct HybridDecomposer {
    strategies: Vec<(Box<dyn Decomposer>, TaskMatcher)>,
    default: Box<dyn Decomposer>,
}

impl Decomposer for HybridDecomposer {
    async fn decompose(&self, task: &Task, context: &Context) -> Result<Vec<Subtask>, DecomposeError> {
        // Find matching strategy for this task type
        for (strategy, matcher) in &self.strategies {
            if matcher.matches(task) {
                return strategy.decompose(task, context).await;
            }
        }
        self.default.decompose(task, context).await
    }
}
```

### 1.7 Decomposition Comparison Matrix

| Approach | Planning Speed | Flexibility | Token Cost | Best For |
|----------|---------------|-------------|------------|----------|
| HTN | Fast (if methods exist) | Low | Low | Known domains |
| GOAP | Slow (search) | High | Low | Dynamic environments |
| ToT | Slow (tree search) | Medium | High | Complex reasoning |
| GoT | Slow (graph ops) | High | Very High | Solution synthesis |
| CoT | Fast | Low | Medium | Linear problems |
| Self-Ask | Medium | Medium | Medium | Multi-hop reasoning |

---

## 2. Plan Verification and Replanning

### 2.1 The Robustness Challenge

Plans fail. Actions have unexpected outcomes. Preconditions change. Environment states drift from assumptions. Robust agents must:
- Detect when plans are no longer valid
- Decide whether to repair or regenerate plans
- Handle failures gracefully
- Learn from failures to improve future planning

### 2.2 Plan Validation Techniques

**Pre-Execution Validation**:

```rust
/// Validate a plan before execution
pub trait PlanValidator {
    /// Check if plan is valid given current state
    fn validate(&self, plan: &Plan, state: &WorldState) -> ValidationResult;

    /// Estimate probability of plan success
    fn estimate_success(&self, plan: &Plan, state: &WorldState) -> f32;
}

pub struct ValidationResult {
    pub valid: bool,
    pub issues: Vec<ValidationIssue>,
}

pub enum ValidationIssue {
    /// A precondition cannot be satisfied
    UnsatisfiablePrecondition { step: usize, condition: Condition },

    /// Steps are in wrong order (dependency violation)
    OrderingViolation { step_a: usize, step_b: usize },

    /// Resource conflict between steps
    ResourceConflict { steps: Vec<usize>, resource: String },

    /// Missing step needed to achieve goal
    IncompleteGoal { missing: Vec<Condition> },

    /// Potential side effect conflict
    SideEffectConflict { step: usize, effect: Effect },
}

/// Multi-layer validation
pub struct LayeredValidator {
    validators: Vec<Box<dyn PlanValidator>>,
}

impl PlanValidator for LayeredValidator {
    fn validate(&self, plan: &Plan, state: &WorldState) -> ValidationResult {
        let mut all_issues = Vec::new();

        for validator in &self.validators {
            let result = validator.validate(plan, state);
            all_issues.extend(result.issues);
        }

        ValidationResult {
            valid: all_issues.is_empty(),
            issues: all_issues,
        }
    }
}
```

**Validation Layers**:

1. **Syntactic Validation**: Plan structure is well-formed
2. **Precondition Checking**: Each step's preconditions can be met
3. **Effect Analysis**: Step effects don't conflict
4. **Resource Validation**: Required resources are available
5. **Temporal Validation**: Deadlines can be met
6. **Safety Validation**: No dangerous states are reached

**Simulation-Based Validation**:
```rust
/// Simulate plan execution to validate
pub struct PlanSimulator {
    world_model: Box<dyn WorldModel>,
    max_steps: usize,
}

impl PlanSimulator {
    pub fn simulate(&self, plan: &Plan, initial_state: WorldState) -> SimulationResult {
        let mut state = initial_state;
        let mut trace = Vec::new();

        for (i, step) in plan.steps.iter().enumerate() {
            // Check preconditions
            if !self.check_preconditions(step, &state) {
                return SimulationResult::Failed {
                    step: i,
                    reason: "Precondition not met".into(),
                    trace,
                };
            }

            // Apply effects (possibly with uncertainty)
            match self.world_model.apply_action(step, &state) {
                Ok(new_state) => {
                    trace.push(StateTransition {
                        step: i,
                        action: step.clone(),
                        before: state.clone(),
                        after: new_state.clone(),
                    });
                    state = new_state;
                }
                Err(e) => {
                    return SimulationResult::Failed {
                        step: i,
                        reason: e.to_string(),
                        trace,
                    };
                }
            }
        }

        // Check goal satisfaction
        if plan.goal.is_satisfied_by(&state) {
            SimulationResult::Success { final_state: state, trace }
        } else {
            SimulationResult::IncompleteGoal { final_state: state, trace }
        }
    }
}
```

### 2.3 Runtime Monitoring

**Execution Monitor Pattern**:
```rust
/// Monitor plan execution in real-time
pub trait ExecutionMonitor: Send + Sync {
    /// Check if current state matches expectations
    fn check_state(&self, expected: &WorldState, actual: &WorldState) -> MonitorResult;

    /// Called before each step
    fn pre_step(&mut self, step: &PlanStep, state: &WorldState) -> PreStepDecision;

    /// Called after each step
    fn post_step(&mut self, step: &PlanStep, result: &StepResult, state: &WorldState) -> PostStepDecision;
}

pub enum PreStepDecision {
    Proceed,
    Skip { reason: String },
    Abort { reason: String },
    Replan { reason: String },
}

pub enum PostStepDecision {
    Continue,
    Retry { modified_step: PlanStep },
    InsertSteps { steps: Vec<PlanStep> },
    Replan { from_step: usize },
    Abort { reason: String },
}

pub struct MonitorResult {
    pub state_match: StateMatch,
    pub deviations: Vec<StateDeviation>,
    pub risk_level: RiskLevel,
}

pub enum StateMatch {
    Exact,
    Compatible,    // Different but plan still valid
    Divergent,     // Significant deviation
    Unknown,       // Cannot determine
}

pub struct StateDeviation {
    pub fact: String,
    pub expected: Option<Value>,
    pub actual: Option<Value>,
    pub severity: Severity,
}
```

**Monitoring Strategies**:

1. **Precondition Monitoring**: Check preconditions before each step
2. **Effect Verification**: Verify expected effects after each step
3. **Invariant Monitoring**: Check system invariants continuously
4. **Progress Monitoring**: Track progress toward goal
5. **Resource Monitoring**: Track resource consumption

```rust
/// Composite monitor combining multiple strategies
pub struct CompositeMonitor {
    precondition_monitor: PreconditionMonitor,
    effect_monitor: EffectMonitor,
    invariant_monitor: InvariantMonitor,
    progress_monitor: ProgressMonitor,
}

impl ExecutionMonitor for CompositeMonitor {
    fn post_step(&mut self, step: &PlanStep, result: &StepResult, state: &WorldState) -> PostStepDecision {
        // Check effects achieved
        if let Some(issue) = self.effect_monitor.check(step, result, state) {
            if issue.severity == Severity::Critical {
                return PostStepDecision::Replan { from_step: 0 };
            }
        }

        // Check invariants not violated
        if let Some(violation) = self.invariant_monitor.check(state) {
            return PostStepDecision::Abort {
                reason: format!("Invariant violated: {}", violation)
            };
        }

        // Update progress
        self.progress_monitor.update(step, state);
        if self.progress_monitor.is_stuck() {
            return PostStepDecision::Replan { from_step: self.progress_monitor.last_progress_step() };
        }

        PostStepDecision::Continue
    }
}
```

### 2.4 Replanning Strategies

**When to Replan**:
```rust
pub enum ReplanTrigger {
    /// Precondition of upcoming step not met
    PreconditionFailure { step: usize },

    /// Step execution failed
    ExecutionFailure { step: usize, error: String },

    /// State deviated too far from expected
    StateDivergence { deviation_score: f32 },

    /// New information makes current plan suboptimal
    BetterPathFound { improvement: f32 },

    /// Goal changed
    GoalChange { new_goal: Goal },

    /// Resource constraints changed
    ResourceChange { resource: String },

    /// Timeout approaching
    TimeoutRisk { remaining: Duration },
}
```

**Replanning Decision Logic**:
```rust
pub struct ReplanningPolicy {
    /// Maximum replanning attempts
    max_replans: usize,

    /// Minimum improvement to justify replanning
    min_improvement_threshold: f32,

    /// Prefer plan repair over full replanning
    prefer_repair: bool,

    /// Maximum repair attempts before full replan
    max_repair_attempts: usize,
}

impl ReplanningPolicy {
    pub fn should_replan(&self, trigger: &ReplanTrigger, history: &ReplanHistory) -> ReplanDecision {
        // Check if we've hit replan limit
        if history.replan_count >= self.max_replans {
            return ReplanDecision::Abort { reason: "Max replans exceeded".into() };
        }

        match trigger {
            ReplanTrigger::PreconditionFailure { step } => {
                if self.prefer_repair && history.repair_count < self.max_repair_attempts {
                    ReplanDecision::Repair { from_step: *step }
                } else {
                    ReplanDecision::FullReplan
                }
            }

            ReplanTrigger::ExecutionFailure { step, error } => {
                // Analyze error to determine if retryable
                if is_transient_error(error) {
                    ReplanDecision::Retry { step: *step, backoff: calculate_backoff(history) }
                } else {
                    ReplanDecision::FullReplan
                }
            }

            ReplanTrigger::BetterPathFound { improvement } => {
                if *improvement > self.min_improvement_threshold {
                    ReplanDecision::FullReplan
                } else {
                    ReplanDecision::Continue
                }
            }

            ReplanTrigger::GoalChange { .. } => ReplanDecision::FullReplan,

            _ => ReplanDecision::Repair { from_step: 0 },
        }
    }
}

pub enum ReplanDecision {
    Continue,
    Retry { step: usize, backoff: Duration },
    Repair { from_step: usize },
    FullReplan,
    Abort { reason: String },
}
```

**Plan Repair vs. Full Replanning**:
```rust
pub trait PlanRepairer {
    /// Attempt to repair plan from given step
    fn repair(&self, plan: &Plan, from_step: usize, state: &WorldState) -> Result<Plan, RepairError>;
}

pub struct LocalRepairer<P: Planner> {
    planner: P,
    max_lookahead: usize,
}

impl<P: Planner> PlanRepairer for LocalRepairer<P> {
    fn repair(&self, plan: &Plan, from_step: usize, state: &WorldState) -> Result<Plan, RepairError> {
        // Extract local goal from next few steps
        let local_goal = self.extract_local_goal(plan, from_step, self.max_lookahead);

        // Plan locally
        let local_plan = self.planner.plan(state, &local_goal)?;

        // Splice into original plan
        let mut repaired = plan.clone();
        repaired.steps.splice(from_step..from_step + self.max_lookahead, local_plan.steps);

        // Validate repaired plan
        if self.validate(&repaired, state) {
            Ok(repaired)
        } else {
            Err(RepairError::ValidationFailed)
        }
    }
}
```

### 2.5 Failure Handling Patterns

**Graceful Degradation**:
```rust
pub struct GracefulDegradation {
    /// Ordered list of fallback goals (most to least ambitious)
    fallback_goals: Vec<Goal>,

    /// Minimum acceptable goal
    minimum_goal: Goal,
}

impl GracefulDegradation {
    pub fn degrade(&self, current_goal_idx: usize) -> Option<&Goal> {
        self.fallback_goals.get(current_goal_idx + 1)
    }

    pub fn at_minimum(&self, current_goal: &Goal) -> bool {
        current_goal == &self.minimum_goal
    }
}
```

**Retry with Exponential Backoff**:
```rust
pub struct RetryPolicy {
    pub max_attempts: usize,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub multiplier: f32,
    pub jitter: bool,
}

impl RetryPolicy {
    pub fn delay_for_attempt(&self, attempt: usize) -> Duration {
        let delay = self.initial_delay.mul_f32(self.multiplier.powi(attempt as i32));
        let capped = delay.min(self.max_delay);

        if self.jitter {
            let jitter_factor = rand::random::<f32>() * 0.3 + 0.85; // 85-115%
            capped.mul_f32(jitter_factor)
        } else {
            capped
        }
    }
}
```

**Error Classification and Handling**:
```rust
pub enum ErrorClass {
    /// Transient error, retry likely to succeed
    Transient,

    /// Precondition error, need different approach
    Precondition,

    /// Resource error, need to wait or acquire
    Resource,

    /// Logic error, plan is fundamentally flawed
    Logic,

    /// External error, system outside our control
    External,

    /// Unknown, be conservative
    Unknown,
}

pub struct ErrorHandler {
    classifiers: Vec<Box<dyn ErrorClassifier>>,
    handlers: HashMap<ErrorClass, Box<dyn ErrorHandlingStrategy>>,
}

impl ErrorHandler {
    pub fn handle(&self, error: &dyn Error, context: &ExecutionContext) -> ErrorResponse {
        let class = self.classify(error);

        if let Some(handler) = self.handlers.get(&class) {
            handler.handle(error, context)
        } else {
            ErrorResponse::Abort { reason: error.to_string() }
        }
    }

    fn classify(&self, error: &dyn Error) -> ErrorClass {
        for classifier in &self.classifiers {
            if let Some(class) = classifier.classify(error) {
                return class;
            }
        }
        ErrorClass::Unknown
    }
}
```

### 2.6 Uncertainty Handling

**Probabilistic Planning**:
```rust
/// Action with uncertain outcomes
pub struct ProbabilisticAction {
    pub name: String,
    pub preconditions: Vec<Condition>,
    pub outcomes: Vec<ProbabilisticOutcome>,
}

pub struct ProbabilisticOutcome {
    pub probability: f32,
    pub effects: Vec<Effect>,
}

/// Plan with confidence scores
pub struct ProbabilisticPlan {
    pub steps: Vec<ProbabilisticAction>,
    pub success_probability: f32,
    pub expected_cost: f32,
}

pub trait ProbabilisticPlanner {
    fn plan(&self, state: &WorldState, goal: &Goal) -> Result<ProbabilisticPlan, PlanError>;

    /// Plan for worst-case outcomes
    fn plan_robust(&self, state: &WorldState, goal: &Goal, min_success_prob: f32)
        -> Result<ProbabilisticPlan, PlanError>;
}
```

**Contingency Planning**:
```rust
/// Plan with contingencies for likely failures
pub struct ContingentPlan {
    pub main_plan: Plan,
    pub contingencies: Vec<Contingency>,
}

pub struct Contingency {
    /// Condition that triggers this contingency
    pub trigger: ContingencyTrigger,

    /// Alternative plan to execute
    pub alternative: Plan,

    /// Whether to resume main plan after contingency
    pub resume_point: Option<usize>,
}

pub enum ContingencyTrigger {
    /// Specific step fails
    StepFailure { step: usize },

    /// State condition becomes true
    StateCondition(Condition),

    /// Resource drops below threshold
    ResourceLow { resource: String, threshold: f32 },

    /// Time constraint at risk
    TimeoutRisk { remaining: Duration },
}

impl ContingentPlan {
    pub fn check_contingencies(&self, state: &WorldState, step: usize) -> Option<&Contingency> {
        self.contingencies.iter().find(|c| c.trigger.is_triggered(state, step))
    }
}
```

---

## 3. Goal Management

### 3.1 The Multi-Goal Challenge

Real agents juggle multiple goals simultaneously:
- User's immediate request
- Ongoing background tasks
- System maintenance goals
- Safety constraints
- Learned preferences

These goals may conflict, have different priorities, and change over time.

### 3.2 Goal Representation

**Basic Goal Structure**:
```rust
#[derive(Clone, Debug)]
pub struct Goal {
    pub id: GoalId,
    pub description: String,
    pub conditions: Vec<GoalCondition>,
    pub priority: Priority,
    pub deadline: Option<Deadline>,
    pub source: GoalSource,
    pub status: GoalStatus,
}

#[derive(Clone, Debug)]
pub enum GoalCondition {
    /// State fact must be true
    StateFact { fact: String, value: Value },

    /// Action must have been performed
    ActionPerformed { action: String },

    /// Resource at certain level
    ResourceLevel { resource: String, min: f32 },

    /// Custom condition
    Custom(Box<dyn Fn(&WorldState) -> bool + Send + Sync>),
}

#[derive(Clone, Debug)]
pub enum GoalSource {
    User,           // Explicitly requested
    System,         // System-generated
    Inferred,       // Inferred from context
    Subgoal { parent: GoalId },  // Derived from another goal
}

#[derive(Clone, Debug)]
pub enum GoalStatus {
    Pending,
    Active,
    Achieved,
    Failed { reason: String },
    Abandoned { reason: String },
    Suspended,
}
```

### 3.3 Goal Hierarchies

**Goal Decomposition**:
```rust
pub struct GoalHierarchy {
    goals: HashMap<GoalId, Goal>,
    children: HashMap<GoalId, Vec<GoalId>>,
    parent: HashMap<GoalId, GoalId>,
}

impl GoalHierarchy {
    /// Add a subgoal under a parent
    pub fn add_subgoal(&mut self, parent: &GoalId, subgoal: Goal) {
        let subgoal_id = subgoal.id.clone();
        self.goals.insert(subgoal_id.clone(), subgoal);
        self.children.entry(parent.clone()).or_default().push(subgoal_id.clone());
        self.parent.insert(subgoal_id, parent.clone());
    }

    /// Check if goal is achieved (recursively checks subgoals)
    pub fn is_achieved(&self, goal_id: &GoalId, state: &WorldState) -> bool {
        let goal = match self.goals.get(goal_id) {
            Some(g) => g,
            None => return false,
        };

        // Check own conditions
        let own_achieved = goal.conditions.iter().all(|c| c.is_satisfied(state));

        // Check all subgoals achieved
        let subgoals = self.children.get(goal_id).map(|v| v.as_slice()).unwrap_or(&[]);
        let subgoals_achieved = subgoals.iter().all(|sg| self.is_achieved(sg, state));

        own_achieved && subgoals_achieved
    }

    /// Get all leaf goals (actionable)
    pub fn get_actionable_goals(&self) -> Vec<&Goal> {
        self.goals.values()
            .filter(|g| {
                let has_children = self.children.get(&g.id).map(|c| !c.is_empty()).unwrap_or(false);
                !has_children && matches!(g.status, GoalStatus::Active | GoalStatus::Pending)
            })
            .collect()
    }

    /// Propagate status up the hierarchy
    pub fn propagate_status(&mut self, goal_id: &GoalId) {
        if let Some(parent_id) = self.parent.get(goal_id).cloned() {
            let siblings = self.children.get(&parent_id).cloned().unwrap_or_default();

            let all_achieved = siblings.iter()
                .all(|s| matches!(self.goals.get(s).map(|g| &g.status), Some(GoalStatus::Achieved)));

            let any_failed = siblings.iter()
                .any(|s| matches!(self.goals.get(s).map(|g| &g.status), Some(GoalStatus::Failed { .. })));

            if let Some(parent) = self.goals.get_mut(&parent_id) {
                if all_achieved {
                    parent.status = GoalStatus::Achieved;
                } else if any_failed {
                    parent.status = GoalStatus::Failed { reason: "Subgoal failed".into() };
                }
            }

            // Recurse up
            self.propagate_status(&parent_id);
        }
    }
}
```

### 3.4 Goal Prioritization

**Priority Schemes**:
```rust
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Priority {
    /// Urgency: how time-sensitive (higher = more urgent)
    pub urgency: u8,

    /// Importance: how valuable (higher = more important)
    pub importance: u8,

    /// Category priority for tie-breaking
    pub category: PriorityCategory,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PriorityCategory {
    Safety,      // Highest - safety constraints
    UserDirect,  // Direct user requests
    UserInferred,// Inferred user needs
    System,      // System maintenance
    Background,  // Background optimization
}

impl Priority {
    /// Eisenhower matrix scoring
    pub fn eisenhower_score(&self) -> u16 {
        (self.importance as u16) * 10 + (self.urgency as u16)
    }

    /// Combined score with category weighting
    pub fn weighted_score(&self) -> u32 {
        let category_weight = match self.category {
            PriorityCategory::Safety => 10000,
            PriorityCategory::UserDirect => 1000,
            PriorityCategory::UserInferred => 100,
            PriorityCategory::System => 10,
            PriorityCategory::Background => 1,
        };
        category_weight * (self.eisenhower_score() as u32)
    }
}

/// Priority queue for goals
pub struct GoalQueue {
    goals: BinaryHeap<PrioritizedGoal>,
}

struct PrioritizedGoal {
    goal: Goal,
    computed_priority: u32,
}

impl Ord for PrioritizedGoal {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.computed_priority.cmp(&other.computed_priority)
    }
}
```

**Dynamic Prioritization**:
```rust
pub trait Prioritizer: Send + Sync {
    /// Compute priority for a goal given current context
    fn prioritize(&self, goal: &Goal, context: &PrioritizationContext) -> Priority;
}

pub struct PrioritizationContext {
    pub current_state: WorldState,
    pub active_goals: Vec<Goal>,
    pub resource_availability: HashMap<String, f32>,
    pub time_constraints: Vec<TimeConstraint>,
    pub user_preferences: UserPreferences,
}

/// Prioritizer that adjusts based on deadlines
pub struct DeadlineAwarePrioritizer {
    base_prioritizer: Box<dyn Prioritizer>,
    urgency_curve: UrgencyCurve,
}

impl Prioritizer for DeadlineAwarePrioritizer {
    fn prioritize(&self, goal: &Goal, context: &PrioritizationContext) -> Priority {
        let mut priority = self.base_prioritizer.prioritize(goal, context);

        if let Some(deadline) = &goal.deadline {
            let time_remaining = deadline.time_until();
            let urgency_boost = self.urgency_curve.compute(time_remaining, deadline.flexibility);
            priority.urgency = priority.urgency.saturating_add(urgency_boost);
        }

        priority
    }
}
```

### 3.5 Conflict Detection and Resolution

**Conflict Types**:
```rust
pub enum GoalConflict {
    /// Goals require mutually exclusive states
    StateConflict {
        goal_a: GoalId,
        goal_b: GoalId,
        fact: String,
        value_a: Value,
        value_b: Value,
    },

    /// Goals compete for limited resource
    ResourceConflict {
        goals: Vec<GoalId>,
        resource: String,
        total_required: f32,
        available: f32,
    },

    /// Goals have incompatible temporal constraints
    TemporalConflict {
        goal_a: GoalId,
        goal_b: GoalId,
        constraint_type: TemporalConstraintType,
    },

    /// Achieving one goal prevents another
    CausalConflict {
        achiever: GoalId,
        prevented: GoalId,
        reason: String,
    },
}

pub struct ConflictDetector {
    state_analyzer: StateConflictAnalyzer,
    resource_analyzer: ResourceConflictAnalyzer,
    temporal_analyzer: TemporalConflictAnalyzer,
}

impl ConflictDetector {
    pub fn detect_conflicts(&self, goals: &[Goal], state: &WorldState) -> Vec<GoalConflict> {
        let mut conflicts = Vec::new();

        conflicts.extend(self.state_analyzer.analyze(goals, state));
        conflicts.extend(self.resource_analyzer.analyze(goals, state));
        conflicts.extend(self.temporal_analyzer.analyze(goals));

        conflicts
    }
}
```

**Resolution Strategies**:
```rust
pub trait ConflictResolver: Send + Sync {
    fn resolve(&self, conflict: &GoalConflict, goals: &mut [Goal], context: &ResolutionContext)
        -> ResolutionResult;
}

pub enum ResolutionResult {
    /// Conflict resolved by adjusting goal
    Resolved { modified_goals: Vec<GoalId> },

    /// One goal must be abandoned
    Abandon { goal: GoalId, reason: String },

    /// Goals must be sequenced differently
    Reorder { new_order: Vec<GoalId> },

    /// User input needed
    NeedsInput { question: String },

    /// Cannot resolve
    Unresolvable { reason: String },
}

/// Priority-based resolution: higher priority wins
pub struct PriorityResolver;

impl ConflictResolver for PriorityResolver {
    fn resolve(&self, conflict: &GoalConflict, goals: &mut [Goal], _context: &ResolutionContext)
        -> ResolutionResult {
        match conflict {
            GoalConflict::StateConflict { goal_a, goal_b, .. } => {
                let priority_a = goals.iter().find(|g| &g.id == goal_a).map(|g| &g.priority);
                let priority_b = goals.iter().find(|g| &g.id == goal_b).map(|g| &g.priority);

                match (priority_a, priority_b) {
                    (Some(a), Some(b)) if a > b => {
                        ResolutionResult::Abandon {
                            goal: goal_b.clone(),
                            reason: "Lower priority goal conflicts".into()
                        }
                    }
                    (Some(a), Some(b)) if b > a => {
                        ResolutionResult::Abandon {
                            goal: goal_a.clone(),
                            reason: "Lower priority goal conflicts".into()
                        }
                    }
                    _ => ResolutionResult::NeedsInput {
                        question: "Two equally important goals conflict. Which should take priority?".into()
                    },
                }
            }

            GoalConflict::ResourceConflict { goals: conflicting, resource, .. } => {
                // Sort by priority, allocate to highest priority first
                ResolutionResult::Reorder {
                    new_order: sort_by_priority(conflicting, goals)
                }
            }

            _ => ResolutionResult::Unresolvable {
                reason: "Strategy not implemented for this conflict type".into()
            },
        }
    }
}

/// Negotiation-based resolution for multi-agent scenarios
pub struct NegotiationResolver {
    max_rounds: usize,
}

/// Constraint relaxation: try to find partial satisfaction
pub struct RelaxationResolver {
    relaxation_strategies: Vec<Box<dyn RelaxationStrategy>>,
}
```

### 3.6 Goal Lifecycle Management

**Goal State Machine**:
```rust
pub struct GoalManager {
    hierarchy: GoalHierarchy,
    prioritizer: Box<dyn Prioritizer>,
    conflict_detector: ConflictDetector,
    conflict_resolver: Box<dyn ConflictResolver>,
    lifecycle_hooks: GoalLifecycleHooks,
}

impl GoalManager {
    /// Add a new goal
    pub fn add_goal(&mut self, goal: Goal, parent: Option<GoalId>) -> Result<GoalId, GoalError> {
        // Check for conflicts with existing goals
        let existing: Vec<_> = self.hierarchy.goals.values().cloned().collect();
        let mut all_goals = existing;
        all_goals.push(goal.clone());

        let conflicts = self.conflict_detector.detect_conflicts(&all_goals, &self.current_state());

        // Attempt to resolve conflicts
        for conflict in conflicts {
            match self.conflict_resolver.resolve(&conflict, &mut all_goals, &self.context()) {
                ResolutionResult::Unresolvable { reason } => {
                    return Err(GoalError::ConflictUnresolvable { reason });
                }
                ResolutionResult::NeedsInput { question } => {
                    return Err(GoalError::NeedsUserInput { question });
                }
                _ => {}
            }
        }

        // Add to hierarchy
        let goal_id = goal.id.clone();
        if let Some(parent_id) = parent {
            self.hierarchy.add_subgoal(&parent_id, goal);
        } else {
            self.hierarchy.goals.insert(goal_id.clone(), goal);
        }

        self.lifecycle_hooks.on_goal_added(&goal_id);
        Ok(goal_id)
    }

    /// Update goal status
    pub fn update_status(&mut self, goal_id: &GoalId, status: GoalStatus) {
        if let Some(goal) = self.hierarchy.goals.get_mut(goal_id) {
            let old_status = goal.status.clone();
            goal.status = status.clone();

            self.lifecycle_hooks.on_status_change(goal_id, &old_status, &status);

            // Propagate up hierarchy if needed
            if matches!(status, GoalStatus::Achieved | GoalStatus::Failed { .. }) {
                self.hierarchy.propagate_status(goal_id);
            }
        }
    }

    /// Get next goal to work on
    pub fn get_next_goal(&self) -> Option<&Goal> {
        let actionable = self.hierarchy.get_actionable_goals();
        let context = self.prioritization_context();

        actionable.into_iter()
            .max_by_key(|g| self.prioritizer.prioritize(g, &context).weighted_score())
    }

    /// Abandon a goal and its subgoals
    pub fn abandon_goal(&mut self, goal_id: &GoalId, reason: &str) {
        // Abandon all descendants first
        if let Some(children) = self.hierarchy.children.get(goal_id).cloned() {
            for child in children {
                self.abandon_goal(&child, "Parent goal abandoned");
            }
        }

        self.update_status(goal_id, GoalStatus::Abandoned { reason: reason.into() });
    }

    /// Suspend a goal (can be resumed later)
    pub fn suspend_goal(&mut self, goal_id: &GoalId) {
        self.update_status(goal_id, GoalStatus::Suspended);
    }

    /// Resume a suspended goal
    pub fn resume_goal(&mut self, goal_id: &GoalId) {
        if let Some(goal) = self.hierarchy.goals.get(goal_id) {
            if matches!(goal.status, GoalStatus::Suspended) {
                self.update_status(goal_id, GoalStatus::Active);
            }
        }
    }
}
```

### 3.7 BDI Architecture Integration

The Belief-Desire-Intention (BDI) model from agent theory provides a proven framework for goal-directed behavior.

```rust
/// BDI-style agent architecture
pub struct BDIAgent {
    /// Current beliefs about the world
    beliefs: BeliefBase,

    /// Desires/goals the agent wants to achieve
    desires: GoalManager,

    /// Current commitments/plans being executed
    intentions: IntentionStack,

    /// Reasoning components
    deliberator: Box<dyn Deliberator>,
    planner: Box<dyn Planner>,
}

pub struct BeliefBase {
    facts: HashMap<String, Value>,
    uncertainty: HashMap<String, f32>,  // Confidence in each belief
    sources: HashMap<String, BeliefSource>,
}

pub struct IntentionStack {
    /// Stack of active intentions (plans being executed)
    stack: Vec<Intention>,

    /// Intention persistence policy
    persistence: IntentionPersistence,
}

pub struct Intention {
    pub goal: Goal,
    pub plan: Plan,
    pub current_step: usize,
    pub commitment_level: CommitmentLevel,
}

#[derive(Clone, Debug)]
pub enum CommitmentLevel {
    /// Will persist until goal achieved or impossible
    Fanatical,
    /// Will persist unless significantly better opportunity
    Strong,
    /// Will reconsider periodically
    Moderate,
    /// Will abandon if any difficulty
    Weak,
}

impl BDIAgent {
    /// Main reasoning cycle
    pub async fn cycle(&mut self) -> Result<(), AgentError> {
        // 1. Update beliefs from perception
        self.update_beliefs().await?;

        // 2. Generate options (potential new goals)
        let options = self.deliberator.generate_options(&self.beliefs, &self.desires).await?;

        // 3. Filter options (which goals to adopt)
        let adopted = self.deliberator.filter_options(&options, &self.beliefs, &self.intentions)?;

        // 4. Update desires with new goals
        for goal in adopted {
            self.desires.add_goal(goal, None)?;
        }

        // 5. Select intention to pursue
        if let Some(goal) = self.desires.get_next_goal() {
            if !self.intentions.has_intention_for(&goal.id) {
                let plan = self.planner.plan(&self.beliefs.to_world_state(), goal).await?;
                self.intentions.push(Intention {
                    goal: goal.clone(),
                    plan,
                    current_step: 0,
                    commitment_level: self.assess_commitment(goal),
                });
            }
        }

        // 6. Execute one step of current intention
        if let Some(intention) = self.intentions.current_mut() {
            let step = &intention.plan.steps[intention.current_step];
            match self.execute_step(step).await {
                Ok(_) => {
                    intention.current_step += 1;
                    if intention.current_step >= intention.plan.steps.len() {
                        self.desires.update_status(&intention.goal.id, GoalStatus::Achieved);
                        self.intentions.pop();
                    }
                }
                Err(e) => {
                    self.handle_execution_failure(e).await?;
                }
            }
        }

        Ok(())
    }

    /// Reconsider intentions based on commitment level
    fn reconsider_intentions(&mut self) {
        self.intentions.stack.retain(|intention| {
            match intention.commitment_level {
                CommitmentLevel::Fanatical => true,
                CommitmentLevel::Strong => {
                    // Only drop if goal becomes impossible
                    !self.is_impossible(&intention.goal)
                }
                CommitmentLevel::Moderate => {
                    // Drop if better opportunity exists
                    !self.has_better_opportunity(&intention.goal)
                }
                CommitmentLevel::Weak => {
                    // Drop on any difficulty
                    !self.has_difficulties(&intention.goal)
                }
            }
        });
    }
}
```

---

## 4. Recommended Patterns for Rust Framework

### 4.1 Unified Planner Trait

```rust
/// Core planning abstraction
#[async_trait]
pub trait Planner: Send + Sync {
    /// Generate a plan to achieve goal from current state
    async fn plan(&self, state: &WorldState, goal: &Goal) -> Result<Plan, PlanError>;

    /// Check if planner can handle this type of goal
    fn can_plan_for(&self, goal: &Goal) -> bool;

    /// Estimate planning time/complexity
    fn estimate_complexity(&self, state: &WorldState, goal: &Goal) -> PlanningComplexity;
}

/// Compose multiple planners
pub struct HierarchicalPlanner {
    planners: Vec<(Box<dyn Planner>, GoalMatcher)>,
    fallback: Box<dyn Planner>,
}

#[async_trait]
impl Planner for HierarchicalPlanner {
    async fn plan(&self, state: &WorldState, goal: &Goal) -> Result<Plan, PlanError> {
        for (planner, matcher) in &self.planners {
            if matcher.matches(goal) && planner.can_plan_for(goal) {
                return planner.plan(state, goal).await;
            }
        }
        self.fallback.plan(state, goal).await
    }
}
```

### 4.2 Type-Safe State Representation

```rust
/// Trait for world state values
pub trait StateValue: Clone + Send + Sync + 'static {
    fn type_name() -> &'static str;
    fn to_any(&self) -> Box<dyn Any + Send + Sync>;
}

/// Type-safe world state with compile-time key types
pub struct TypedWorldState {
    values: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
    string_keys: HashMap<String, Box<dyn Any + Send + Sync>>,
}

impl TypedWorldState {
    pub fn get<T: StateValue>(&self) -> Option<&T> {
        self.values.get(&TypeId::of::<T>())
            .and_then(|v| v.downcast_ref())
    }

    pub fn set<T: StateValue>(&mut self, value: T) {
        self.values.insert(TypeId::of::<T>(), Box::new(value));
    }
}

/// Macro for defining typed state keys
#[macro_export]
macro_rules! define_state_key {
    ($name:ident : $type:ty) => {
        pub struct $name;
        impl $crate::StateKey for $name {
            type Value = $type;
            fn name() -> &'static str { stringify!($name) }
        }
    };
}

// Usage:
define_state_key!(HasCode: bool);
define_state_key!(TestsPassing: bool);
define_state_key!(CurrentFile: Option<String>);
```

### 4.3 Plan Execution Engine

```rust
/// Execute plans with monitoring and error handling
pub struct PlanExecutor<M: ExecutionMonitor, R: PlanRepairer> {
    monitor: M,
    repairer: R,
    retry_policy: RetryPolicy,
    replanning_policy: ReplanningPolicy,
}

impl<M: ExecutionMonitor, R: PlanRepairer> PlanExecutor<M, R> {
    pub async fn execute(&mut self, plan: Plan, state: &mut WorldState) -> ExecutionResult {
        let mut current_plan = plan;
        let mut step_idx = 0;
        let mut replan_count = 0;

        while step_idx < current_plan.steps.len() {
            let step = &current_plan.steps[step_idx];

            // Pre-step check
            match self.monitor.pre_step(step, state) {
                PreStepDecision::Proceed => {}
                PreStepDecision::Skip { reason } => {
                    log::info!("Skipping step {}: {}", step_idx, reason);
                    step_idx += 1;
                    continue;
                }
                PreStepDecision::Abort { reason } => {
                    return ExecutionResult::Aborted { step: step_idx, reason };
                }
                PreStepDecision::Replan { reason } => {
                    if let Some(new_plan) = self.try_replan(&current_plan, step_idx, state, &mut replan_count).await {
                        current_plan = new_plan;
                        step_idx = 0;
                        continue;
                    } else {
                        return ExecutionResult::ReplanFailed { step: step_idx, reason };
                    }
                }
            }

            // Execute step with retries
            let result = self.execute_step_with_retry(step, state).await;

            // Post-step check
            match self.monitor.post_step(step, &result, state) {
                PostStepDecision::Continue => {
                    step_idx += 1;
                }
                PostStepDecision::Retry { modified_step } => {
                    current_plan.steps[step_idx] = modified_step;
                    // Don't increment, retry same step
                }
                PostStepDecision::InsertSteps { steps } => {
                    // Insert steps after current
                    current_plan.steps.splice(step_idx + 1..step_idx + 1, steps);
                    step_idx += 1;
                }
                PostStepDecision::Replan { from_step } => {
                    if let Some(new_plan) = self.try_replan(&current_plan, from_step, state, &mut replan_count).await {
                        current_plan = new_plan;
                        step_idx = from_step;
                    } else {
                        return ExecutionResult::ReplanFailed { step: step_idx, reason: "Replan triggered but failed".into() };
                    }
                }
                PostStepDecision::Abort { reason } => {
                    return ExecutionResult::Aborted { step: step_idx, reason };
                }
            }
        }

        ExecutionResult::Success {
            steps_executed: current_plan.steps.len(),
            replans: replan_count
        }
    }
}
```

### 4.4 Goal-Oriented Agent Framework

```rust
/// High-level agent combining all planning components
pub struct GoalOrientedAgent {
    /// Goal management
    goals: GoalManager,

    /// Planning capabilities
    planner: Box<dyn Planner>,

    /// Plan execution
    executor: Box<dyn Executor>,

    /// Task decomposition
    decomposer: Box<dyn Decomposer>,

    /// Current world state
    state: WorldState,

    /// Agent configuration
    config: AgentConfig,
}

impl GoalOrientedAgent {
    /// Main agent loop
    pub async fn run(&mut self) -> Result<(), AgentError> {
        loop {
            // Get highest priority goal
            let Some(goal) = self.goals.get_next_goal() else {
                // No goals, wait for new input
                self.wait_for_input().await?;
                continue;
            };

            // Decompose if needed
            if self.decomposer.needs_decomposition(goal) {
                let subtasks = self.decomposer.decompose(goal, &self.context()).await?;
                for subtask in subtasks {
                    self.goals.add_goal(subtask.into_goal(), Some(goal.id.clone()))?;
                }
                continue;
            }

            // Generate plan
            let plan = match self.planner.plan(&self.state, goal).await {
                Ok(p) => p,
                Err(PlanError::Impossible { reason }) => {
                    self.goals.update_status(&goal.id, GoalStatus::Failed { reason });
                    continue;
                }
                Err(e) => return Err(e.into()),
            };

            // Execute plan
            match self.executor.execute(plan, &mut self.state).await {
                ExecutionResult::Success { .. } => {
                    self.goals.update_status(&goal.id, GoalStatus::Achieved);
                }
                ExecutionResult::Aborted { reason, .. } => {
                    self.goals.update_status(&goal.id, GoalStatus::Failed { reason });
                }
                ExecutionResult::ReplanFailed { reason, .. } => {
                    self.goals.update_status(&goal.id, GoalStatus::Failed { reason });
                }
            }
        }
    }
}
```

---

## 5. Rust-Specific Design Considerations

### 5.1 Type System Leverage

**Phantom Types for Plan States**:
```rust
/// Type-level plan state
pub struct Validated;
pub struct Unvalidated;
pub struct Executing;
pub struct Completed;

pub struct Plan<State = Unvalidated> {
    steps: Vec<PlanStep>,
    _state: PhantomData<State>,
}

impl Plan<Unvalidated> {
    pub fn validate(self, validator: &dyn PlanValidator) -> Result<Plan<Validated>, ValidationError> {
        validator.validate(&self)?;
        Ok(Plan {
            steps: self.steps,
            _state: PhantomData,
        })
    }
}

impl Plan<Validated> {
    pub fn begin_execution(self) -> Plan<Executing> {
        Plan {
            steps: self.steps,
            _state: PhantomData,
        }
    }
}

// Only validated plans can be executed
impl Executor {
    pub fn execute(&self, plan: Plan<Validated>) -> ExecutionHandle {
        // ...
    }
}
```

**Builder Pattern for Complex Types**:
```rust
pub struct GoalBuilder {
    description: Option<String>,
    conditions: Vec<GoalCondition>,
    priority: Priority,
    deadline: Option<Deadline>,
}

impl GoalBuilder {
    pub fn new() -> Self {
        Self {
            description: None,
            conditions: Vec::new(),
            priority: Priority::default(),
            deadline: None,
        }
    }

    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn condition(mut self, cond: GoalCondition) -> Self {
        self.conditions.push(cond);
        self
    }

    pub fn priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }

    pub fn deadline(mut self, deadline: Deadline) -> Self {
        self.deadline = Some(deadline);
        self
    }

    pub fn build(self) -> Result<Goal, BuildError> {
        let description = self.description.ok_or(BuildError::MissingDescription)?;
        if self.conditions.is_empty() {
            return Err(BuildError::NoConditions);
        }

        Ok(Goal {
            id: GoalId::new(),
            description,
            conditions: self.conditions,
            priority: self.priority,
            deadline: self.deadline,
            source: GoalSource::User,
            status: GoalStatus::Pending,
        })
    }
}
```

### 5.2 Error Handling

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PlanError {
    #[error("Goal is impossible to achieve: {reason}")]
    Impossible { reason: String },

    #[error("Planning timed out after {elapsed:?}")]
    Timeout { elapsed: Duration },

    #[error("No valid decomposition found for task")]
    NoDecomposition,

    #[error("State precondition not met: {condition}")]
    PreconditionFailed { condition: String },

    #[error("Resource unavailable: {resource}")]
    ResourceUnavailable { resource: String },

    #[error("LLM error during planning: {0}")]
    LLMError(#[from] LLMError),
}

#[derive(Error, Debug)]
pub enum ExecutionError {
    #[error("Step {step} failed: {reason}")]
    StepFailed { step: usize, reason: String },

    #[error("Replanning failed after {attempts} attempts")]
    ReplanExhausted { attempts: usize },

    #[error("Execution aborted: {reason}")]
    Aborted { reason: String },

    #[error("Invariant violated: {invariant}")]
    InvariantViolation { invariant: String },
}

/// Result type aliases
pub type PlanResult<T> = Result<T, PlanError>;
pub type ExecResult<T> = Result<T, ExecutionError>;
```

### 5.3 Async Architecture

```rust
/// Async-friendly planning with cancellation
pub struct AsyncPlanner {
    inner: Box<dyn Planner>,
    timeout: Duration,
}

impl AsyncPlanner {
    pub async fn plan_with_cancellation(
        &self,
        state: &WorldState,
        goal: &Goal,
        cancel: CancellationToken,
    ) -> PlanResult<Plan> {
        tokio::select! {
            result = self.inner.plan(state, goal) => result,
            _ = cancel.cancelled() => Err(PlanError::Cancelled),
            _ = tokio::time::sleep(self.timeout) => Err(PlanError::Timeout { elapsed: self.timeout }),
        }
    }
}

/// Parallel plan generation (race multiple strategies)
pub async fn race_planners(
    planners: Vec<Box<dyn Planner>>,
    state: &WorldState,
    goal: &Goal,
) -> PlanResult<Plan> {
    let futures: Vec<_> = planners.iter()
        .map(|p| p.plan(state, goal))
        .collect();

    // Return first successful result
    let (result, _, _) = futures::future::select_all(futures).await;
    result
}
```

### 5.4 Memory Efficiency

```rust
/// Interned strings for frequently repeated values
pub struct InternedString {
    index: u32,
}

pub struct StringInterner {
    strings: Vec<String>,
    lookup: HashMap<String, u32>,
}

impl StringInterner {
    pub fn intern(&mut self, s: &str) -> InternedString {
        if let Some(&idx) = self.lookup.get(s) {
            return InternedString { index: idx };
        }

        let idx = self.strings.len() as u32;
        self.strings.push(s.to_string());
        self.lookup.insert(s.to_string(), idx);
        InternedString { index: idx }
    }

    pub fn resolve(&self, interned: InternedString) -> &str {
        &self.strings[interned.index as usize]
    }
}

/// Arena allocation for plan steps
pub struct PlanArena {
    steps: typed_arena::Arena<PlanStep>,
    plans: Vec<&'static [PlanStep]>,
}
```

---

## 6. Open Research Questions

### 6.1 LLM-Specific Planning Challenges

1. **Decomposition Granularity**: How fine-grained should LLM task decomposition be? Too fine loses context; too coarse risks failures.

2. **Plan Verbalization**: What's the optimal way to represent plans in natural language for LLMs to follow?

3. **Self-Monitoring**: Can LLMs accurately assess their own plan execution progress?

4. **Learned Heuristics**: Can we train LLMs to have better planning heuristics for specific domains?

### 6.2 Hybrid Architecture Questions

1. **Classical + Neural**: When to use classical planners (HTN, GOAP) vs. neural planners (ToT, GoT)?

2. **Verification Integration**: How to integrate formal verification with LLM-generated plans?

3. **Multi-Agent Coordination**: How do multiple planning agents coordinate without conflicts?

### 6.3 Practical Implementation Questions

1. **Plan Caching**: When is it safe to reuse previously generated plans?

2. **Incremental Replanning**: How to minimize disruption when replanning mid-execution?

3. **User Preference Learning**: How to learn prioritization preferences from user feedback?

4. **Failure Attribution**: How to determine root causes when plans fail?

### 6.4 Safety and Alignment

1. **Goal Safety**: How to ensure agent-generated subgoals don't violate safety constraints?

2. **Commitment Bounds**: How to prevent agents from being overly committed to bad plans?

3. **Human Override**: How to design planning systems that respect human intervention?

---

## 7. Summary and Recommendations

### Key Takeaways

1. **Multiple decomposition strategies needed**: No single approach works for all tasks. HTN for known domains, GOAP for dynamic environments, ToT/GoT for complex reasoning.

2. **Verification is essential**: Plans fail. Build robust validation, monitoring, and replanning from the start.

3. **Goals are hierarchical and dynamic**: Support goal decomposition, prioritization, and conflict resolution.

4. **Rust's type system is an asset**: Use phantom types, builders, and strong error types to prevent invalid states.

5. **BDI provides a solid foundation**: The Belief-Desire-Intention model offers a proven architecture for goal-directed agents.

### Recommended Architecture for Agentic Framework

1. **Pluggable Planner Trait**: Support multiple planning backends
2. **Type-Safe State Management**: Leverage Rust's type system for world state
3. **Composable Decomposition**: Chain decomposition strategies
4. **Layered Validation**: Multiple validation passes before execution
5. **Adaptive Monitoring**: Monitor execution with configurable policies
6. **Hierarchical Goals**: First-class goal hierarchy support
7. **Graceful Degradation**: Built-in fallback and recovery mechanisms

### Priority Implementation Order

1. **Core Types**: WorldState, Goal, Plan, PlanStep
2. **Basic Planner Trait**: Simple planning interface
3. **Plan Executor**: Step-by-step execution with basic monitoring
4. **Goal Manager**: Goal hierarchy and prioritization
5. **Validation Layer**: Pre-execution plan validation
6. **Replanning**: Failure detection and recovery
7. **Advanced Planners**: HTN, GOAP, ToT implementations
8. **Conflict Resolution**: Multi-goal conflict handling

---

## References and Further Reading

### Classical Planning
- Ghallab, Nau, Traverso: "Automated Planning: Theory and Practice" (2004)
- HTN Planning: SHOP2, PANDA systems
- GOAP: Jeff Orkin's original GDC papers

### LLM-Based Planning
- "Tree of Thoughts: Deliberate Problem Solving with Large Language Models" (Yao et al., 2023)
- "Graph of Thoughts: Solving Elaborate Problems with Large Language Models" (Besta et al., 2023)
- "Chain-of-Thought Prompting Elicits Reasoning in Large Language Models" (Wei et al., 2022)
- "ReAct: Synergizing Reasoning and Acting in Language Models" (Yao et al., 2023)

### Agent Architectures
- "Belief-Desire-Intention Model" (Rao & Georgeff, 1995)
- "An Introduction to Multiagent Systems" (Wooldridge, 2009)
- "Artificial Intelligence: A Modern Approach" - Planning chapters (Russell & Norvig)

### Modern Agent Systems
- AutoGPT, BabyAGI - early LLM agent implementations
- LangChain Agents documentation
- Claude's tool use and agentic patterns

### Rust Resources
- "Programming Rust" (Blandy, Orendorff, Tindall)
- Tokio async runtime documentation
- Serde serialization framework
