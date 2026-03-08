use ollama_rs::Ollama;
use ollama_rs::generation::chat::request::ChatMessageRequest;
use ollama_rs::generation::chat::{ChatMessage, MessageRole};
use tokio::io::{self, AsyncWriteExt};
use tokio_stream::StreamExt;
use std::io::{stdin, BufRead};

#[tokio::main]
async fn main() {
    let ollama = Ollama::default();
    let model = "ministral-3:8b".to_string();

    let mut history: Vec<ChatMessage> = Vec::new();

    println!("╔══════════════════════════════════════╗");
    println!("║     🦀 Rust Ollama Chat (CLI)        ║");
    println!("║     Model: ministral-3:8b            ║");
    println!("║     Type 'exit' to quit              ║");
    println!("╚══════════════════════════════════════╝");
    println!();

    let stdin = stdin();

    loop {
        print!("You: ");
        std::io::Write::flush(&mut std::io::stdout()).unwrap();

        let mut user_input = String::new();
        stdin.lock().read_line(&mut user_input).unwrap();
        let user_input = user_input.trim().to_string();

        if user_input.to_lowercase() == "exit" {
            println!("Goodbye! 👋");
            break;
        }

        if user_input.is_empty() {
            continue;
        }

        history.push(ChatMessage {
            role: MessageRole::User,
            content: user_input.clone(),
            images: None,
            tool_calls: vec![],
            thinking: None,
        });

        print!("\nAI: ");
        let mut stdout = io::stdout();

        let request = ChatMessageRequest::new(model.clone(), history.clone());

        let mut stream = match ollama.send_chat_messages_stream(request).await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Error connecting: {:?}", e);
                continue;
            }
        };

        let mut full_response = String::new();

        while let Some(result) = stream.next().await {
            match result {
                Ok(response) => {
                    let token = &response.message.content;
                    full_response.push_str(token);
                    stdout.write_all(token.as_bytes()).await.unwrap();
                    stdout.flush().await.unwrap();
                }
                Err(e) => {
                    eprintln!("\nStream error: {:?}", e);
                    break;
                }
            }
        }

        history.push(ChatMessage {
            role: MessageRole::Assistant,
            content: full_response,
            images: None,
            tool_calls: vec![],
            thinking: None,
        });

        println!("\n");
    }
}