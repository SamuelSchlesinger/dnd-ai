//! Live tool-loop and streaming smoke test for OpenRouter or Ollama.
//!
//! ```text
//! cargo run -p chronicler-llm --example provider_smoke -- openrouter
//! cargo run -p chronicler-llm --example provider_smoke -- ollama [model]
//! ```
//!
//! OpenRouter credentials are read from the process environment or parsed
//! directly from `~/.generalist.env`; they are never printed or persisted.

use chronicler_llm::{
    Client, ContentBlock, Message, Request, Role, StopReason, StreamEvent, Tool, ToolChoice,
    LOCAL_DEFAULT_MODEL, OPENROUTER_BASE_URL,
};
use futures::StreamExt;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let provider = args.next().unwrap_or_else(|| "ollama".to_string());
    let (client, label) = match provider.as_str() {
        "openrouter" => {
            load_generalist_environment();
            let key = std::env::var("OPENROUTER_API_KEY")
                .map_err(|_| "OPENROUTER_API_KEY is not configured")?;
            let client = Client::openrouter(key)?;
            assert_eq!(client.base_url(), OPENROUTER_BASE_URL);
            (client, "OpenRouter")
        }
        "ollama" => {
            let model = args
                .next()
                .unwrap_or_else(|| LOCAL_DEFAULT_MODEL.to_string());
            (Client::local(model)?, "Ollama")
        }
        other => return Err(format!("unknown provider '{other}'").into()),
    };

    let tool = Tool {
        name: "multiply".to_string(),
        description: "Multiply two integers".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "left": {"type": "integer"},
                "right": {"type": "integer"}
            },
            "required": ["left", "right"],
            "additionalProperties": false
        }),
    };
    let mut messages = vec![Message::user(
        "Call multiply with 111 and 111. After receiving its result, reply with exactly RESULT=12321.",
    )];
    let first = client
        .complete(
            Request::new(messages.clone())
                .with_system("Follow the requested tool protocol and output format exactly.")
                .with_max_tokens(512)
                .with_temperature(0.0)
                .with_tools(vec![tool.clone()])
                .with_tool_choice(ToolChoice::Auto),
        )
        .await?;
    let tool_calls: Vec<_> = first
        .content
        .iter()
        .filter_map(|block| match block {
            ContentBlock::ToolUse { id, name, input } => {
                Some((id.clone(), name.clone(), input.clone()))
            }
            _ => None,
        })
        .collect();
    if tool_calls.is_empty() {
        return Err(format!(
            "model did not call multiply (stop={:?}, text={:?})",
            first.stop_reason,
            first.text()
        )
        .into());
    }
    if tool_calls.iter().any(|(_, name, _)| name != "multiply") {
        return Err("model called an unexpected tool".into());
    }

    let preserved_reasoning = first
        .content
        .iter()
        .any(|block| matches!(block, ContentBlock::ReasoningDetails { .. }));
    messages.push(Message {
        role: Role::Assistant,
        content: first.content,
    });
    messages.push(Message {
        role: Role::User,
        content: tool_calls
            .into_iter()
            .map(|(id, _, _)| ContentBlock::ToolResult {
                tool_use_id: id,
                content: "12321".to_string(),
                is_error: false,
            })
            .collect(),
    });

    let mut stream = client
        .stream(
            Request::new(messages)
                .with_system("Follow the requested tool protocol and output format exactly.")
                .with_max_tokens(256)
                .with_temperature(0.0)
                .with_tools(vec![tool])
                .with_tool_choice(ToolChoice::Auto),
        )
        .await?;
    let mut answer = String::new();
    let mut stop_reason = None;
    while let Some(event) = stream.next().await {
        match event? {
            StreamEvent::TextDelta { text, .. } => answer.push_str(&text),
            StreamEvent::MessageDelta {
                stop_reason: reason,
            } => stop_reason = reason,
            StreamEvent::Error { message } => return Err(message.into()),
            _ => {}
        }
    }
    if !answer.replace([' ', '\n'], "").contains("RESULT=12321") {
        return Err(format!("unexpected final answer: {answer:?}").into());
    }
    if stop_reason == Some(StopReason::ToolUse) {
        return Err("model requested another tool instead of finishing".into());
    }

    println!(
        "SMOKE OK - {label}/{} (streaming + tool loop; reasoning details preserved: {preserved_reasoning})",
        client.model()
    );
    Ok(())
}

fn load_generalist_environment() {
    let Some(home) = std::env::var_os("HOME") else {
        return;
    };
    let path = std::path::PathBuf::from(home).join(".generalist.env");
    if path.is_file() {
        dotenvy::from_path(path).ok();
    }
}
