//! Simple chat agent example
//!
//! This example demonstrates how to use the Anthropic provider directly
//! for a simple conversational agent.
//!
//! Run with: cargo run --bin simple_chat
//! (Make sure .env file has ANTHROPIC_API_KEY set)

use agentic::llm::anthropic::AnthropicProvider;
use agentic::llm::{CompletionRequest, LlmProvider};
use agentic::message::Message;
use std::io::{self, Write};

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

    println!("Simple Chat Agent");
    println!("=================");
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

        // Create completion request
        let request = CompletionRequest::new("claude-sonnet-4-20250514")
            .with_system("You are a helpful assistant. Be concise but friendly.")
            .with_messages(messages.clone())
            .with_max_tokens(1024);

        // Get response
        match provider.complete(request).await {
            Ok(response) => {
                let assistant_text = response.message.text_content();
                println!("\nAssistant: {}\n", assistant_text);

                // Add assistant message to history
                messages.push(response.message);

                // Show token usage
                println!(
                    "(tokens: {} in, {} out)\n",
                    response.usage.input_tokens, response.usage.output_tokens
                );
            }
            Err(e) => {
                eprintln!("\nError: {}\n", e);
            }
        }
    }

    Ok(())
}
