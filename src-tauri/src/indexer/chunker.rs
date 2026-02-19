/// Approximate token count by splitting on whitespace.
/// This is a rough estimate (~1.3 tokens per word for English).
#[allow(dead_code)]
pub fn estimate_tokens(text: &str) -> usize {
    text.split_whitespace().count()
}

/// Chunk a document's text into overlapping segments.
///
/// - `chunk_size`: target tokens per chunk (default 512)
/// - `overlap`: tokens of overlap between chunks (default 64)
pub fn chunk_text(text: &str, chunk_size: usize, overlap: usize) -> Vec<ChunkInfo> {
    let words: Vec<&str> = text.split_whitespace().collect();

    if words.is_empty() {
        return Vec::new();
    }

    if words.len() <= chunk_size {
        return vec![ChunkInfo {
            index: 0,
            content: words.join(" "),
            token_count: words.len() as i32,
        }];
    }

    let mut chunks = Vec::new();
    let step = chunk_size.saturating_sub(overlap).max(1);
    let mut start = 0;
    let mut index = 0;

    while start < words.len() {
        let end = (start + chunk_size).min(words.len());
        let chunk_words = &words[start..end];
        let content = chunk_words.join(" ");

        chunks.push(ChunkInfo {
            index,
            content,
            token_count: chunk_words.len() as i32,
        });

        start += step;
        index += 1;

        if end >= words.len() {
            break;
        }
    }

    chunks
}

/// Default chunking with 512 tokens and 64 overlap.
pub fn chunk_text_default(text: &str) -> Vec<ChunkInfo> {
    chunk_text(text, 512, 64)
}

#[derive(Debug, Clone)]
pub struct ChunkInfo {
    pub index: i32,
    pub content: String,
    pub token_count: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_text() {
        let chunks = chunk_text_default("");
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_short_text() {
        let chunks = chunk_text_default("hello world");
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].content, "hello world");
        assert_eq!(chunks[0].token_count, 2);
    }

    #[test]
    fn test_chunking_with_overlap() {
        // Create a text with exactly 1024 words
        let words: Vec<String> = (0..1024).map(|i| format!("word{}", i)).collect();
        let text = words.join(" ");

        let chunks = chunk_text(&text, 512, 64);

        // Should produce at least 2 chunks
        assert!(chunks.len() >= 2);

        // First chunk should have 512 tokens
        assert_eq!(chunks[0].token_count, 512);

        // Chunks should be indexed
        assert_eq!(chunks[0].index, 0);
        assert_eq!(chunks[1].index, 1);
    }
}
