//! Main Dungeon Master agent implementation

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::json;

use agentic::agent::{Agent, AgentMetadata, Capabilities, Context, Response};
use agentic::error::{AgentError, ToolError};
use agentic::tool::ToolOutput;
use agentic::id::AgentId;
use agentic::llm::anthropic::AnthropicProvider;
use agentic::llm::{CompletionRequest, LlmProvider, StopReason, ToolChoice};
use agentic::message::{ContentBlock, Message, Role};
use agentic::tool::{Tool, ToolContext, ToolRegistry};

use super::prompts::build_dm_system_prompt;
use super::tools::{ApplyDamageTool, ApplyHealingTool, RollDiceTool, SavingThrowTool, SkillCheckTool};
use crate::dnd::game::state::GameWorld;

/// The main Dungeon Master agent
pub struct DungeonMasterAgent {
    id: AgentId,
    metadata: AgentMetadata,
    capabilities: Capabilities,
    llm: Arc<AnthropicProvider>,
    tools: ToolRegistry,
}

impl DungeonMasterAgent {
    /// Create a new DM agent
    pub fn new(llm: Arc<AnthropicProvider>) -> Self {
        let mut tools = ToolRegistry::new();

        // Register D&D tools
        tools.register(RollDiceTool);
        tools.register(SkillCheckTool);
        tools.register(SavingThrowTool);
        tools.register(ApplyDamageTool);
        tools.register(ApplyHealingTool);

        Self {
            id: AgentId::new(),
            metadata: AgentMetadata::new("Dungeon Master", "AI Dungeon Master for D&D 5e")
                .with_version("0.1.0"),
            capabilities: Capabilities {
                tool_use: true,
                memory: true,
                planning: false,
                streaming: true,
                vision: false,
                multi_agent: true,
                max_context_tokens: Some(200_000),
                supported_content: vec!["text".to_string(), "tool_use".to_string()],
            },
            llm,
            tools,
        }
    }

    /// Process a player action and generate a response
    pub async fn process_action(
        &self,
        player_input: &str,
        game: &GameWorld,
        conversation_history: &[Message],
    ) -> Result<DmResponse, AgentError> {
        // Build system prompt with game state
        let system_prompt = build_dm_system_prompt(game);

        // Build messages
        let mut messages = conversation_history.to_vec();
        messages.push(Message::user(player_input));

        // Create completion request
        let request = CompletionRequest::new("claude-sonnet-4-20250514")
            .with_system(&system_prompt)
            .with_messages(messages.clone())
            .with_max_tokens(2048)
            .with_temperature(0.7)
            .with_tools(self.tools.tool_definitions())
            .with_tool_choice(ToolChoice::Auto);

        // Execute with tool loop
        let (response, tool_calls) = self.execute_with_tools(request, &mut messages).await?;

        Ok(DmResponse {
            narrative: response.text_content(),
            tool_results: tool_calls,
        })
    }

    /// Execute a request, handling any tool calls
    async fn execute_with_tools(
        &self,
        mut request: CompletionRequest,
        messages: &mut Vec<Message>,
    ) -> Result<(Message, Vec<ToolCallResult>), AgentError> {
        let mut all_tool_calls = Vec::new();
        let max_iterations = 10;

        for _ in 0..max_iterations {
            let response = self
                .llm
                .complete(request.clone())
                .await
                .map_err(|e| AgentError::ProcessingFailed {
                    reason: e.to_string(),
                })?;

            // Check if we need to execute tools
            if response.stop_reason == StopReason::ToolUse {
                // Extract tool calls
                let tool_uses = response.message.tool_uses();

                if tool_uses.is_empty() {
                    // No actual tool calls, return the response
                    return Ok((response.message, all_tool_calls));
                }

                // Add assistant message with tool calls
                messages.push(response.message.clone());

                // Execute each tool
                let mut tool_results = Vec::new();
                for (tool_id, tool_name, input) in tool_uses {
                    let result = self.execute_tool(tool_name, input.clone()).await;

                    let (content, is_error) = match &result {
                        Ok(output) => (output.content.clone(), output.is_error()),
                        Err(e) => (format!("Error: {}", e), true),
                    };

                    all_tool_calls.push(ToolCallResult {
                        tool_name: tool_name.to_string(),
                        input: input.clone(),
                        output: content.clone(),
                        is_error,
                    });

                    tool_results.push(ContentBlock::tool_result(*tool_id, content, is_error));
                }

                // Add tool results as user message
                messages.push(Message::new(Role::User, tool_results));

                // Update request with new messages
                request = request.with_messages(messages.clone());
            } else {
                // No more tool calls, return the final response
                return Ok((response.message, all_tool_calls));
            }
        }

        Err(AgentError::ProcessingFailed {
            reason: "Too many tool call iterations".to_string(),
        })
    }

    /// Execute a single tool
    async fn execute_tool(
        &self,
        tool_name: &str,
        input: serde_json::Value,
    ) -> Result<ToolOutput, ToolError> {
        let tool = self.tools.get(tool_name).ok_or_else(|| {
            ToolError::NotFound {
                name: tool_name.to_string(),
            }
        })?;

        let ctx = ToolContext::default();
        tool.execute(input, &ctx).await
    }
}

#[async_trait]
impl Agent for DungeonMasterAgent {
    fn id(&self) -> &AgentId {
        &self.id
    }

    fn metadata(&self) -> &AgentMetadata {
        &self.metadata
    }

    fn capabilities(&self) -> &Capabilities {
        &self.capabilities
    }

    fn tools(&self) -> &[Arc<dyn Tool>] {
        // Return empty slice - tools are managed internally
        &[]
    }

    async fn process(&self, message: Message, context: &mut Context) -> Result<Response, AgentError> {
        // This would integrate with the full context management
        // For now, create a simple response
        let response_text = format!("Received: {}", message.text_content());

        Ok(Response::text(response_text))
    }

    async fn initialize(&mut self) -> Result<(), AgentError> {
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), AgentError> {
        Ok(())
    }
}

/// Response from the DM agent
#[derive(Debug, Clone)]
pub struct DmResponse {
    pub narrative: String,
    pub tool_results: Vec<ToolCallResult>,
}

/// Result of a tool call
#[derive(Debug, Clone)]
pub struct ToolCallResult {
    pub tool_name: String,
    pub input: serde_json::Value,
    pub output: String,
    pub is_error: bool,
}
