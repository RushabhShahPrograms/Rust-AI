# Rust Ollama Demo

A CLI chat app in Rust that talks to a local Ollama instance — with **dynamic model selection** and a **self-contained portable binary**.

---

## Prerequisites

- [Ollama](https://ollama.com/) running locally (`ollama serve`)
- At least one model pulled (e.g. `ollama pull llama3`)

---

## Running in development

```bash
cargo run
```

---

## Building a portable binary

### Key concept

> Rust compiles to a **native binary**. The output file does **not** need Rust installed to run — you can copy it to any machine with the same OS/architecture.

### Option A — Standard release build (recommended for most cases)

```bash
cargo build --release
```

The binary is at `./target/release/rust-ollama-demo`.  
Copy it anywhere on the same OS — it just works.

```bash
# Copy and run on the same Linux/macOS machine
cp target/release/rust-ollama-demo ~/bin/ollama-chat
ollama-chat
```

---

### Option B — Fully static binary (works on ANY Linux, no glibc needed)

A standard Linux build still depends on the system's C library (glibc).  
To make a truly standalone binary that runs on *any* Linux (Alpine, old Ubuntu, Docker scratch, etc.), compile against **musl**:

```bash
# 1. Add the musl target (one-time setup)
rustup target add x86_64-unknown-linux-musl

# On Ubuntu/Debian you also need the musl linker:
sudo apt install musl-tools

# 2. Build
cargo build --release --target x86_64-unknown-linux-musl

# Binary is at:
./target/x86_64-unknown-linux-musl/release/rust-ollama-demo
```

This single file can be:
- SCP'd to any Linux server
- Added to a `FROM scratch` Docker image
- Dropped onto an Alpine container
- Zipped and shared — no installer needed

---

### Option C — Cross-compile for other platforms

Use [cross](https://github.com/cross-rs/cross) (requires Docker):

```bash
cargo install cross

# Build for ARM64 Linux (e.g. Raspberry Pi, AWS Graviton)
cross build --release --target aarch64-unknown-linux-musl

# Build for Windows from Linux/macOS
cross build --release --target x86_64-pc-windows-gnu
```

---

## What the `[profile.release]` flags do

| Flag | Effect |
|------|--------|
| `opt-level = 3` | Maximum compiler optimizations |
| `lto = true` | Link-Time Optimization — dead code removal across crates |
| `codegen-units = 1` | Better optimization (slower compile, but a one-time cost) |
| `strip = true` | Removes debug symbols → **~60-80% smaller binary** |
| `panic = "abort"` | Removes stack unwinding machinery → smaller binary |

---

## Usage

```
╔══════════════════════════════════════╗
║     🦀 Rust Ollama Chat (CLI)        ║
╚══════════════════════════════════════╝

🔍 Fetching models from Ollama...

Available models:
  1. llama3:latest
  2. mistral:latest
  3. codellama:13b

Select a model (1-3, default 1): 2

✅  Using model: mistral:latest

Type your message and press Enter. Type 'exit' or Ctrl+C to quit.

You: What is ownership in Rust?
AI: Ownership is Rust's memory management system...
```