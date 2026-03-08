use crate::embeddings::{embed, find_top_chunks};
use ollama_rs::Ollama;

const LLM_MODEL: &str = "ministral-3:8b";

pub struct RagEngine {
    pub ollama: Ollama,
    pub chunks: Vec<String>,
    pub embeddings: Vec<Vec<f32>>,
}

impl RagEngine {
    pub fn new(chunks: Vec<String>, embeddings: Vec<Vec<f32>>) -> Self {
        RagEngine {
            ollama: Ollama::default(),
            chunks,
            embeddings,
        }
    }

    /// Takes the user question and returns a fully-formed prompt
    /// injected with the most relevant PDF chunks
    pub async fn build_prompt(&self, question: &str) -> String {
        let query_emb = embed(&self.ollama, question).await;
        let relevant = find_top_chunks(&query_emb, &self.chunks, &self.embeddings, 3);
        let context = relevant.join("\n\n---\n\n");

        // ⚠️ This strict prompt is what prevents hallucination
        format!(
            "You are a document assistant. \
            Answer ONLY using the CONTEXT below. \
            If the answer is not present, say: 'I cannot find this in the document.' \
            Never use outside knowledge. Never guess.\n\n\
            CONTEXT:\n{context}\n\n\
            QUESTION: {question}\n\n\
            ANSWER:",
        )
    }

    pub fn llm_model(&self) -> String {
        LLM_MODEL.to_string()
    }
}