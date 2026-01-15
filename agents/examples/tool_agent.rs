//! Tool-using agent example
//!
//! This example demonstrates how to build an agent that can use tools.
//! It includes a simple calculator tool and shows the full tool use flow.
//!
//! Run with: cargo run --bin tool_agent
//! (Make sure .env file has ANTHROPIC_API_KEY set)

use agentic::error::ToolError;
use agentic::llm::anthropic::AnthropicProvider;
use agentic::llm::{CompletionRequest, LlmProvider, StopReason, ToolChoice};
use agentic::message::{ContentBlock, Message, Role};
use agentic::tool::{Tool, ToolAnnotations, ToolContext, ToolDefinition, ToolOutput};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::io::{self, Write};

/// A simple calculator tool
struct CalculatorTool;

#[async_trait]
impl Tool for CalculatorTool {
    fn name(&self) -> &str {
        "calculator"
    }

    fn description(&self) -> &str {
        "Performs basic arithmetic operations. Supports add, subtract, multiply, and divide."
    }

    fn input_schema(&self) -> &Value {
        static SCHEMA: once_cell::sync::Lazy<Value> = once_cell::sync::Lazy::new(|| {
            json!({
                "type": "object",
                "properties": {
                    "operation": {
                        "type": "string",
                        "enum": ["add", "subtract", "multiply", "divide"],
                        "description": "The arithmetic operation to perform"
                    },
                    "a": {
                        "type": "number",
                        "description": "The first operand"
                    },
                    "b": {
                        "type": "number",
                        "description": "The second operand"
                    }
                },
                "required": ["operation", "a", "b"]
            })
        });
        &SCHEMA
    }

    fn annotations(&self) -> &ToolAnnotations {
        static ANNOTATIONS: ToolAnnotations = ToolAnnotations::read_only();
        &ANNOTATIONS
    }

    fn is_idempotent(&self) -> bool {
        true
    }

    async fn execute(&self, params: Value, _context: &ToolContext) -> Result<ToolOutput, ToolError> {
        let operation = params["operation"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidParameters {
                tool: "calculator".to_string(),
                reason: "operation must be a string".to_string(),
            })?;

        let a = params["a"].as_f64().ok_or_else(|| ToolError::InvalidParameters {
            tool: "calculator".to_string(),
            reason: "a must be a number".to_string(),
        })?;

        let b = params["b"].as_f64().ok_or_else(|| ToolError::InvalidParameters {
            tool: "calculator".to_string(),
            reason: "b must be a number".to_string(),
        })?;

        let result = match operation {
            "add" => a + b,
            "subtract" => a - b,
            "multiply" => a * b,
            "divide" => {
                if b == 0.0 {
                    return Ok(ToolOutput::error("Cannot divide by zero"));
                }
                a / b
            }
            _ => {
                return Err(ToolError::InvalidParameters {
                    tool: "calculator".to_string(),
                    reason: format!("Unknown operation: {}", operation),
                });
            }
        };

        Ok(ToolOutput::structured(
            format!("{}", result),
            json!({ "result": result }),
        ))
    }
}

/// Get current time tool
struct GetTimeTool;

#[async_trait]
impl Tool for GetTimeTool {
    fn name(&self) -> &str {
        "get_current_time"
    }

    fn description(&self) -> &str {
        "Returns the current date and time in UTC."
    }

    fn input_schema(&self) -> &Value {
        static SCHEMA: once_cell::sync::Lazy<Value> = once_cell::sync::Lazy::new(|| {
            json!({
                "type": "object",
                "properties": {},
                "required": []
            })
        });
        &SCHEMA
    }

    fn annotations(&self) -> &ToolAnnotations {
        static ANNOTATIONS: ToolAnnotations = ToolAnnotations::read_only();
        &ANNOTATIONS
    }

    fn is_idempotent(&self) -> bool {
        true
    }

    async fn execute(&self, _params: Value, _context: &ToolContext) -> Result<ToolOutput, ToolError> {
        let now = chrono::Utc::now();
        Ok(ToolOutput::text(format!(
            "Current UTC time: {}",
            now.format("%Y-%m-%d %H:%M:%S UTC")
        )))
    }
}

fn tool_to_definition<T: Tool>(tool: &T) -> ToolDefinition {
    ToolDefinition {
        name: tool.name().to_string(),
        description: tool.description().to_string(),
        input_schema: tool.input_schema().clone(),
        annotations: tool.annotations().clone(),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env file (try workspace root first, then current dir)
    if dotenvy::from_path("../.env").is_err() {
        let _ = dotenvy::dotenv();
    }

    // Create the Anthropic provider from environment variable
    let provider = match AnthropicProvider::from_env() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error: {}", e);
            eprintln!("Please set ANTHROPIC_API_KEY in .env file");
            std::process::exit(1);
        }
    };

    // Create our tools
    let calculator = CalculatorTool;
    let get_time = GetTimeTool;

    let tools = vec![
        tool_to_definition(&calculator),
        tool_to_definition(&get_time),
    ];

    println!("Tool-Using Agent");
    println!("================");
    println!("Available tools: calculator, get_current_time");
    println!("Try: 'What is 42 * 17?' or 'What time is it?'");
    println!("Type 'quit' to exit.\n");

    let mut messages: Vec<Message> = Vec::new();

    loop {
        // Print prompt
        print!("You: ");
        io::stdout().flush()?;

        // Read user input
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.eq_ignore_ascii_case("quit") {
            println!("Goodbye!");
            break;
        }

        if input.is_empty() {
            continue;
        }

        // Add user message
        messages.push(Message::user(input));

        // Tool use loop - continue until no more tool calls
        loop {
            // Create completion request with tools
            let request = CompletionRequest::new("claude-sonnet-4-20250514")
                .with_system("You are a helpful assistant with access to tools. Use the calculator for math and get_current_time for time queries. Always explain your reasoning.")
                .with_messages(messages.clone())
                .with_max_tokens(1024)
                .with_tools(tools.clone())
                .with_tool_choice(ToolChoice::Auto);

            // Get response
            let response = match provider.complete(request).await {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("\nError: {}\n", e);
                    break;
                }
            };

            // Check if model wants to use tools
            if response.stop_reason == StopReason::ToolUse {
                // Process tool calls
                let tool_uses: Vec<_> = response.message.tool_uses();

                // Add assistant message with tool uses
                messages.push(response.message.clone());

                // Execute each tool and collect results
                let mut tool_results: Vec<ContentBlock> = Vec::new();

                for (tool_id, tool_name, input) in tool_uses {
                    println!("\n[Using tool: {} with {:?}]", tool_name, input);

                    let result = match tool_name {
                        "calculator" => {
                            calculator
                                .execute(input.clone(), &ToolContext::default())
                                .await
                        }
                        "get_current_time" => {
                            get_time
                                .execute(input.clone(), &ToolContext::default())
                                .await
                        }
                        _ => Ok(ToolOutput::error(format!("Unknown tool: {}", tool_name))),
                    };

                    let (content, is_error) = match result {
                        Ok(output) => {
                            let is_err = output.is_error();
                            (output.content, is_err)
                        }
                        Err(e) => (format!("Error: {}", e), true),
                    };

                    println!("[Tool result: {}]", content);

                    tool_results.push(ContentBlock::ToolResult {
                        tool_use_id: *tool_id,
                        content,
                        is_error,
                    });
                }

                // Add tool results message
                messages.push(Message::new(Role::User, tool_results));
            } else {
                // No tool use, print the response
                let assistant_text = response.message.text_content();
                println!("\nAssistant: {}\n", assistant_text);

                // Add assistant message to history
                messages.push(response.message);

                // Show token usage
                println!(
                    "(tokens: {} in, {} out)\n",
                    response.usage.input_tokens, response.usage.output_tokens
                );
                break;
            }
        }
    }

    Ok(())
}
