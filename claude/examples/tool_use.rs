//! Tool use example demonstrating the complete_with_tools helper.
//!
//! Run with: cargo run -p claude --example tool_use

use chronicler_llm::{Client, Message, Request, Tool, ToolChoice, ToolResult, ToolUse};
use serde_json::json;

fn calculator_tool() -> Tool {
    Tool {
        name: "calculator".to_string(),
        description: "Performs arithmetic: add, subtract, multiply, divide".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": ["add", "subtract", "multiply", "divide"]
                },
                "a": { "type": "number" },
                "b": { "type": "number" }
            },
            "required": ["operation", "a", "b"]
        }),
    }
}

fn execute_tool(tool: ToolUse) -> ToolResult {
    match tool.name.as_str() {
        "calculator" => {
            let op = tool.input["operation"].as_str().unwrap_or("");
            let a = tool.input["a"].as_f64().unwrap_or(0.0);
            let b = tool.input["b"].as_f64().unwrap_or(0.0);

            let result = match op {
                "add" => a + b,
                "subtract" => a - b,
                "multiply" => a * b,
                "divide" if b != 0.0 => a / b,
                "divide" => return ToolResult::error("Division by zero"),
                _ => return ToolResult::error(format!("Unknown operation: {op}")),
            };
            ToolResult::success(format!("{result}"))
        }
        _ => ToolResult::error(format!("Unknown tool: {}", tool.name)),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let client = Client::from_env()?;

    let request = Request::new(vec![Message::user("What is 42 * 17 + 5?")])
        .with_system("Use the calculator tool for math.")
        .with_tools(vec![calculator_tool()])
        .with_tool_choice(ToolChoice::Auto);

    let response = client
        .complete_with_tools(request, |tool| async move {
            println!("[Tool: {} with {:?}]", tool.name, tool.input);
            execute_tool(tool)
        })
        .await?;

    println!("\nResult: {}", response.text());
    Ok(())
}
