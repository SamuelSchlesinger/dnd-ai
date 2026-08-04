//! Simple chat example using the configured LLM provider.
//!
//! Run with: cargo run -p claude --example simple_chat

use chronicler_llm::{Client, Message, Request};
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let client = Client::from_env()?;
    let mut messages: Vec<Message> = Vec::new();

    println!("Simple Chat (type 'quit' to exit)\n");

    loop {
        print!("You: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.eq_ignore_ascii_case("quit") {
            break;
        }
        if input.is_empty() {
            continue;
        }

        messages.push(Message::user(input));

        let request = Request::new(messages.clone())
            .with_system("You are a helpful assistant. Be concise.")
            .with_max_tokens(1024);

        let response = client.complete(request).await?;
        println!("\nAssistant: {}\n", response.text());

        messages.push(Message::assistant(response.text()));
    }

    Ok(())
}
