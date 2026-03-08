mod pdf;
mod embeddings;
mod rag;

use crate::embeddings::embed;
use crate::rag::RagEngine;
use ollama_rs::generation::chat::{ChatMessage, MessageRole};
use ollama_rs::generation::chat::request::ChatMessageRequest;
use ollama_rs::Ollama;
use tokio::io::{self, AsyncWriteExt};
use tokio_stream::StreamExt;
use std::io::{stdin, BufRead};

#[tokio::main]
async fn main() {
    println!("╔══════════════════════════════════════════════╗");
    println!("║     🦀 Rust PDF RAG — Chat with your PDF     ║");
    println!("╚══════════════════════════════════════════════╝\n");

    // --- 1. Get PDF path ---
    print!("📄 Enter path to your PDF: ");
    std::io::Write::flush(&mut std::io::stdout()).unwrap();

    let stdin = stdin();
    let mut pdf_path = String::new();
    stdin.lock().read_line(&mut pdf_path).unwrap();
    let pdf_path = pdf_path.trim();

    // --- 2. Extract and chunk text ---
    println!("\n🔍 Extracting text from PDF...");
    let text = match pdf::extract_text(pdf_path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("❌ Failed to read PDF: {e}");
            return;
        }
    };
    println!("✅ Extracted {} characters", text.len());

    // 200 words per chunk, 30 word overlap
    let chunks = pdf::chunk_text(&text, 200, 30);
    println!("✂️  Split into {} chunks\n", chunks.len());

    // --- 3. Embed all chunks ---
    let ollama = Ollama::default();
    println!("🧮 Generating embeddings for all chunks...");
    let mut chunk_embeddings: Vec<Vec<f32>> = Vec::new();

    for (i, chunk) in chunks.iter().enumerate() {
        print!("\r   Progress: {}/{}", i + 1, chunks.len());
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
        let emb = embed(&ollama, chunk).await;
        chunk_embeddings.push(emb);
    }
    println!("\n✅ Embeddings ready!");
    println!("\n💬 Ask anything about the PDF. Type 'exit' to quit.\n");

    // --- 4. Build RAG engine ---
    let engine = RagEngine::new(chunks, chunk_embeddings);

    // --- 5. Chat loop ---
    loop {
        print!("You: ");
        std::io::Write::flush(&mut std::io::stdout()).unwrap();

        let mut question = String::new();
        stdin.lock().read_line(&mut question).unwrap();
        let question = question.trim().to_string();

        if question.to_lowercase() == "exit" {
            println!("Goodbye! 👋");
            break;
        }
        if question.is_empty() {
            continue;
        }

        // Build the RAG-injected prompt
        let prompt = engine.build_prompt(&question).await;

        print!("\nAI: ");
        let mut stdout = io::stdout();

        let messages = vec![ChatMessage {
            role: MessageRole::User,
            content: prompt,
            images: None,
            tool_calls: vec![],
            thinking: None,
        }];

        let request = ChatMessageRequest::new(engine.llm_model(), messages);

        let mut stream = match engine.ollama.send_chat_messages_stream(request).await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("❌ Error: {:?}", e);
                continue;
            }
        };

        while let Some(result) = stream.next().await {
            if let Ok(response) = result {
                let token = &response.message.content;
                stdout.write_all(token.as_bytes()).await.unwrap();
                stdout.flush().await.unwrap();
            }
        }
        println!("\n");
    }
}