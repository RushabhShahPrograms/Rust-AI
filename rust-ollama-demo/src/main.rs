use anyhow::{Context, Result};
use ollama_rs::generation::chat::request::ChatMessageRequest;
use ollama_rs::generation::chat::{ChatMessage, MessageRole};
use ollama_rs::Ollama;
use std::io::{stdin, BufRead};
use tokio::io::{self, AsyncWriteExt};
use tokio_stream::StreamExt;

// ── Entry point ────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    // Drive all errors up to here so we print them cleanly instead of panicking.
    if let Err(e) = run().await {
        eprintln!("\n❌  Error: {e:#}");
        std::process::exit(1);
    }
}

// ── Core application ───────────────────────────────────────────────────────────

async fn run() -> Result<()> {
    let ollama = Ollama::default();

    print_banner();

    // ── Step 1: fetch & display installed models ──────────────────────────────
    println!("🔍 Fetching models from Ollama...\n");

    let models = ollama
        .list_local_models()
        .await
        .context("Could not reach Ollama. Is it running? (try: ollama serve)")?;

    if models.is_empty() {
        anyhow::bail!(
            "No models installed. Pull one first:\n  ollama pull llama3\n  ollama pull mistral"
        );
    }

    println!("Available models:");
    for (i, m) in models.iter().enumerate() {
        println!("  {}. {}", i + 1, m.name);
    }

    // ── Step 2: let the user pick ─────────────────────────────────────────────
    let model = select_model(&models)?;
    println!("\n✅  Using model: {model}\n");

    // ── Step 3: chat loop ─────────────────────────────────────────────────────
    let stdin = stdin();
    let mut history: Vec<ChatMessage> = Vec::new();

    println!("Type your message and press Enter. Type 'exit' or press Ctrl+C to quit.\n");

    loop {
        // Prompt
        print!("You: ");
        flush_stdout();

        let mut raw = String::new();
        stdin
            .lock()
            .read_line(&mut raw)
            .context("Failed to read from stdin")?;

        let user_input = raw.trim().to_string();

        // Exit commands
        if user_input.eq_ignore_ascii_case("exit")
            || user_input.eq_ignore_ascii_case("quit")
            || user_input.eq_ignore_ascii_case("q")
        {
            println!("\nGoodbye! 👋");
            break;
        }

        if user_input.is_empty() {
            continue;
        }

        // Add user turn to history
        history.push(ChatMessage {
            role: MessageRole::User,
            content: user_input.clone(),
            images: None,
            tool_calls: vec![],
            thinking: None,
        });

        // Stream the AI response
        let full_response = stream_response(&ollama, &model, &history).await?;

        // Add assistant turn to history (keeps multi-turn context alive)
        history.push(ChatMessage {
            role: MessageRole::Assistant,
            content: full_response,
            images: None,
            tool_calls: vec![],
            thinking: None,
        });

        println!("\n");
    }

    Ok(())
}

// ── Helpers ────────────────────────────────────────────────────────────────────

/// Ask user to pick a model by number. Defaults to 1 on bad input.
fn select_model(models: &[ollama_rs::models::LocalModel]) -> Result<String> {
    println!();
    print!("Select a model (1-{}, default 1): ", models.len());
    flush_stdout();

    let stdin = stdin();
    let mut input = String::new();
    stdin
        .lock()
        .read_line(&mut input)
        .context("Failed to read model selection")?;

    let trimmed = input.trim();

    // Empty input → default to 1
    if trimmed.is_empty() {
        return Ok(models[0].name.clone());
    }

    match trimmed.parse::<usize>() {
        Ok(n) if n >= 1 && n <= models.len() => Ok(models[n - 1].name.clone()),
        _ => {
            println!("⚠️  Invalid choice, defaulting to model 1.");
            Ok(models[0].name.clone())
        }
    }
}

/// Stream tokens from the model and print them in real time.
/// Returns the complete assembled response.
async fn stream_response(
    ollama: &Ollama,
    model: &str,
    history: &[ChatMessage],
) -> Result<String> {
    print!("\nAI: ");

    let request = ChatMessageRequest::new(model.to_string(), history.to_vec());

    let mut stream = ollama
        .send_chat_messages_stream(request)
        .await
        .context("Failed to start stream — is the model loaded?")?;

    let mut stdout = io::stdout();
    let mut full_response = String::new();

    while let Some(result) = stream.next().await {
        match result {
            Ok(response) => {
                let token = &response.message.content;
                full_response.push_str(token);

                stdout
                    .write_all(token.as_bytes())
                    .await
                    .context("Failed to write token to stdout")?;

                stdout.flush().await.context("Failed to flush stdout")?;
            }
            Err(e) => {
                // Don't abort the whole app on a stream error — log and break.
                eprintln!("\n⚠️  Stream interrupted: {e:?}");
                break;
            }
        }
    }

    Ok(full_response)
}

fn flush_stdout() {
    use std::io::Write;
    std::io::stdout().flush().ok();
}

fn print_banner() {
    println!("╔══════════════════════════════════════╗");
    println!("║     🦀 Rust Ollama Chat (CLI)        ║");
    println!("╚══════════════════════════════════════╝");
    println!();
}