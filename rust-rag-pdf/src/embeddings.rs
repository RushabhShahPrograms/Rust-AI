use ollama_rs::Ollama;
use ollama_rs::generation::embeddings::request::{EmbeddingsInput, GenerateEmbeddingsRequest};

const EMBED_MODEL: &str = "qwen3-embedding:0.6b";

/// Calls Ollama to turn a text string into a vector of floats (embedding)
pub async fn embed(ollama: &Ollama, text: &str) -> Vec<f32> {
    let req = GenerateEmbeddingsRequest::new(
        EMBED_MODEL.to_string(),
        EmbeddingsInput::Single(text.to_string()),
    );

    let res = ollama
        .generate_embeddings(req)
        .await
        .expect("Failed to generate embedding");

    // Ollama returns f64, we cast to f32 to save memory
    res.embeddings[0].iter().map(|&x| x as f32).collect()
}

/// Measures how similar two vectors are (returns value between -1 and 1)
/// 1.0 = identical meaning, 0.0 = unrelated
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let mag_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let mag_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if mag_a == 0.0 || mag_b == 0.0 {
        return 0.0;
    }
    dot / (mag_a * mag_b)
}

/// Given a query embedding, returns the top-k most relevant chunk texts
pub fn find_top_chunks<'a>(
    query_emb: &[f32],
    chunks: &'a [String],
    chunk_embeddings: &[Vec<f32>],
    k: usize,
) -> Vec<&'a str> {
    // Score every chunk against the query
    let mut scores: Vec<(f32, usize)> = chunk_embeddings
        .iter()
        .enumerate()
        .map(|(i, emb)| (cosine_similarity(query_emb, emb), i))
        .collect();

    // Sort by highest similarity first
    scores.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

    // Return top-k chunk texts
    scores
        .iter()
        .take(k)
        .map(|(_, i)| chunks[*i].as_str())
        .collect()
}