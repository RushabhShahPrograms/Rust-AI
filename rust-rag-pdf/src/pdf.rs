/// Reads a PDF file from disk and returns all its text as a String
pub fn extract_text(path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let bytes = std::fs::read(path)?;
    let text = pdf_extract::extract_text_from_mem(&bytes)?;
    Ok(text)
}

/// Splits a big string into overlapping word-based chunks.
/// chunk_size = how many words per chunk
/// overlap     = how many words to repeat from the previous chunk
///               (overlap helps avoid cutting a sentence mid-thought)
pub fn chunk_text(text: &str, chunk_size: usize, overlap: usize) -> Vec<String> {
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut chunks = Vec::new();
    let mut i = 0;

    while i < words.len() {
        let end = (i + chunk_size).min(words.len());
        let chunk = words[i..end].join(" ");
        chunks.push(chunk);
        if end == words.len() {
            break;
        }
        i += chunk_size - overlap; // move forward but keep overlap words
    }
    chunks
}