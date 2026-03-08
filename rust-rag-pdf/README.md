# 🦀 Rust PDF RAG

Chat with any PDF using local LLMs — no cloud, no API keys, fully private.

## How it works

```
PDF → Extract Text → Chunk → Embed (qwen3-embedding)
                                        ↓
Your Question → Embed → Find Similar Chunks → LLM (ministral-3:8b) → Answer
```

## Requirements

- [Rust](https://rustup.rs) 1.75+
- [Ollama](https://ollama.com) running locally

## Setup

```bash
# Pull required models
ollama pull ministral-3:8b
ollama pull qwen3-embedding:0.6b

# Clone and run
git clone https://github.com/RushabhShahPrograms/Rust-AI
cd rust-rag-pdf
cargo run
```

## Usage

```
📄 Enter path to your PDF: /path/to/document.pdf
🧮 Generating embeddings...
💬 Ask anything about the PDF. Type 'exit' to quit.

You: What is this document about?
AI: (streams answer from PDF context only)
```

## Project Structure

```
src/
├── main.rs         # Entry point + CLI loop
├── pdf.rs          # PDF extraction + chunking
├── embeddings.rs   # Vector embeddings + cosine similarity
└── rag.rs          # RAG pipeline + prompt builder
```

## Stack

| Library | Purpose |
|---|---|
| `ollama-rs` | LLM inference + embeddings |
| `pdf-extract` | PDF text extraction |
| `tokio` | Async runtime |
| `tokio-stream` | Streaming responses |