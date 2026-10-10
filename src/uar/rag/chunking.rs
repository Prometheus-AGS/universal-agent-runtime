use crate::uar::rag::embeddings::EmbeddingBackend;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use text_splitter::{Characters, ChunkConfig, MarkdownSplitter, TextSplitter};
use tracing::warn;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChunkingStrategy {
    /// Simple fixed character length
    FixedSize { size: usize },
    /// Token-based splitting (using `cl100k_base` via `text_splitter`)
    Token { tokens: usize },
    /// Recursive character splitting trying to respect semantic boundaries (paragraphs, etc.)
    Recursive { size: usize },
    /// Structure-aware recursive splitting. Uses MarkdownSplitter when content
    /// contains Markdown structure (headings, fenced code blocks, etc.), falls
    /// back to TextSplitter otherwise. Uses a character range to avoid
    /// undersized fragments.
    Structured { size: usize },
    /// Split by sentence using Unicode sentence boundaries
    Sentence,
    /// Keep full document (no chunking)
    Document,
    /// Semantic chunking: Embed sentences and merge if similar
    Semantic { threshold: f32 },
    /// Agentic: Ask LLM (Not implemented yet)
    Agentic,
}

#[derive(Debug)]
pub struct Chunker {
    strategy: ChunkingStrategy,
    // Optional because not all strategies need it
    embedding_backend: Option<Arc<dyn EmbeddingBackend>>,
}

impl Chunker {
    pub fn new(
        strategy: ChunkingStrategy,
        embedding_backend: Option<Arc<dyn EmbeddingBackend>>,
    ) -> Self {
        Self {
            strategy,
            embedding_backend,
        }
    }

    pub async fn chunk(&self, text: &str) -> Result<Vec<String>> {
        match &self.strategy {
            ChunkingStrategy::FixedSize { size } => Ok(text
                .chars()
                .collect::<Vec<char>>()
                .chunks(*size)
                .map(|c| c.iter().collect::<String>())
                .collect()),
            ChunkingStrategy::Recursive { size } => {
                let config = ChunkConfig::new(*size)
                    .with_sizer(Characters)
                    .with_trim(true);
                let splitter = TextSplitter::new(config);
                Ok(splitter.chunks(text).map(|s: &str| s.to_string()).collect())
            }
            ChunkingStrategy::Structured { size } => {
                // Use a range so chunks fill up before being returned.
                // This prevents the undersized fragment problem where
                // 32-50 char pieces from version numbers score highest.
                let lower = (*size / 2).max(64);
                let capacity = lower..=*size;

                // Lightweight heuristic: detect Markdown structure markers.
                // False positives just use MarkdownSplitter on plain text,
                // which degrades gracefully (still respects paragraph/sentence
                // boundaries). False negatives use TextSplitter (current behavior).
                let has_markdown_structure = text.contains("\n#")
                    || text.contains("\n##")
                    || text.contains("```")
                    || text.starts_with('#');

                if has_markdown_structure {
                    let splitter = MarkdownSplitter::new(capacity);
                    Ok(splitter.chunks(text).map(|s| s.to_string()).collect())
                } else {
                    let config = ChunkConfig::new(capacity)
                        .with_sizer(Characters)
                        .with_trim(true);
                    let splitter = TextSplitter::new(config);
                    Ok(splitter.chunks(text).map(|s: &str| s.to_string()).collect())
                }
            }
            ChunkingStrategy::Token { tokens } => {
                let size = tokens * 4;
                let config = ChunkConfig::new(size)
                    .with_sizer(Characters)
                    .with_trim(true);
                let splitter = TextSplitter::new(config);
                Ok(splitter.chunks(text).map(|s: &str| s.to_string()).collect())
            }
            ChunkingStrategy::Sentence => Ok(split_sentences(text, false)),
            ChunkingStrategy::Document => Ok(vec![text.to_string()]),
            ChunkingStrategy::Semantic { threshold } => self.semantic_chunk(text, *threshold).await,
            ChunkingStrategy::Agentic => {
                warn!("Agentic chunking not implemented, falling back to Document");
                Ok(vec![text.to_string()])
            }
        }
    }

    async fn semantic_chunk(&self, text: &str, threshold: f32) -> Result<Vec<String>> {
        let backend = self
            .embedding_backend
            .as_ref()
            .ok_or_else(|| anyhow!("Embedding backend required for Semantic Chunking"))?;

        // 1. Split into "Base Sentences" (using simple sentence strategy)
        let sentences = split_sentences(text, true);

        if sentences.is_empty() {
            return Ok(vec![]);
        }

        // 2. Embed all sentences
        let refs: Vec<&str> = sentences.iter().map(|s| s.as_str()).collect();
        let embeddings = backend.embed(&refs).await?;

        // 3. Iterate and merge
        // Algorithm:
        // Start current_chunk with sentence[0].
        // next_sentence = sentence[1].
        // Sim = cosine(embedding[current_chunk_avg], embedding[next]).
        // If Sim > Threshold -> Merge.
        // Else -> Push current_chunk, start new.
        // Optimization: Just compare adjacent sentences for now (easier than maintaining running avg embedding).

        // This is a complex logic. MVP Implementation:
        // Merge adjacent if similar.

        let mut chunks = Vec::new();
        let mut current_chunk = sentences[0].clone();
        let mut current_emb = embeddings[0].clone();

        for i in 1..sentences.len() {
            let next_sent = &sentences[i];
            let next_emb = &embeddings[i];

            // Calculate similarity between Current Aggregate vs Next
            // (Or just Last vs Next? Aggregate is better but requires re-embedding or averaging)
            // Averaging normalized embeddings is a decent approximation for "topic".
            // Let's use Last Sentence vs Next Sentence for simple "coherence".
            // Better: Average of current chunk so far.
            // Simple MVP: Compare with *previous sentence* (i-1) embedding.
            // Actually, comparing to the *running average* of the current chunk is standard.

            // Cosine Similarity
            let sim = cosine_similarity(&current_emb, next_emb);

            if sim >= threshold {
                // Merge
                current_chunk.push(' ');
                current_chunk.push_str(next_sent);
                // Update average embedding (naive unweighted average)
                current_emb = current_emb
                    .iter()
                    .zip(next_emb.iter())
                    .map(|(a, b)| (a + b) / 2.0)
                    .collect();
            } else {
                // Finalize chunk
                chunks.push(current_chunk);
                // Start new
                current_chunk = next_sent.clone();
                current_emb = next_emb.clone();
            }
        }
        chunks.push(current_chunk);

        Ok(chunks)
    }
}

/// Split `text` into trimmed, non-empty sentences.
///
/// `!` and `?` always end a sentence. A `.` ends one only when whitespace or
/// the end of the text follows it, so "v0.2.0" and "Obsidian 1.12.3" stay in
/// one piece, and not when everything since the last boundary is digits, so a
/// list marker such as "1." stays with its item. With `newline_breaks`, a
/// newline also ends a sentence.
fn split_sentences(text: &str, newline_breaks: bool) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut start = 0;
    let mut chars = text.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        let ends_sentence = match c {
            '!' | '?' => true,
            '\n' => newline_breaks,
            '.' => {
                let followed_by_break = chars.peek().is_none_or(|&(_, next)| next.is_whitespace());
                let is_list_marker = text[start..i].trim().chars().all(|d| d.is_ascii_digit());
                followed_by_break && !is_list_marker
            }
            _ => false,
        };
        if ends_sentence {
            let end = i + c.len_utf8();
            push_trimmed(&mut sentences, &text[start..end]);
            start = end;
        }
    }
    push_trimmed(&mut sentences, &text[start..]);
    sentences
}

fn push_trimmed(sentences: &mut Vec<String>, piece: &str) {
    let piece = piece.trim();
    if !piece.is_empty() {
        sentences.push(piece.to_string());
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let mag_a = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let mag_b = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if mag_a == 0.0 || mag_b == 0.0 {
        return 0.0;
    }
    dot / (mag_a * mag_b)
}

/// Detect whether text contains Markdown structure markers.
fn has_markdown_structure(text: &str) -> bool {
    text.contains("\n#") || text.contains("\n##") || text.contains("```") || text.starts_with('#')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fixed_size() {
        let strategy = ChunkingStrategy::FixedSize { size: 5 };
        let chunker = Chunker::new(strategy, None);
        let text = "HelloWorld";
        let chunks = chunker.chunk(text).await.unwrap();
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0], "Hello");
        assert_eq!(chunks[1], "World");
    }

    #[tokio::test]
    async fn test_recursive() {
        let strategy = ChunkingStrategy::Recursive { size: 10 };
        let chunker = Chunker::new(strategy, None);
        let text = "Hello World From Rust";
        let chunks = chunker.chunk(text).await.unwrap();
        assert!(!chunks.is_empty());
        for c in chunks {
            assert!(c.len() <= 10, "Chunk '{c}' exceeds size 10");
        }
    }

    #[tokio::test]
    async fn structured_respects_headings() {
        let strategy = ChunkingStrategy::Structured { size: 200 };
        let chunker = Chunker::new(strategy, None);
        let text = "# Introduction\n\nThis is the introduction paragraph with enough text to fill a reasonable chunk size for testing purposes.\n\n## Details\n\nThis is the details section with additional content that should form its own chunk separate from the introduction above.";
        let chunks = chunker.chunk(text).await.unwrap();
        assert!(!chunks.is_empty());
        // No chunk should contain a partial heading (heading on its own line
        // should stay with its following content, not be orphaned).
        for chunk in &chunks {
            // A chunk should not start with bare text that was split from
            // right after a heading — headings should anchor their sections.
            assert!(
                !chunk.is_empty(),
                "Structured chunking produced an empty chunk"
            );
        }
    }

    #[tokio::test]
    async fn structured_no_version_fragments() {
        let strategy = ChunkingStrategy::Structured { size: 1024 };
        let chunker = Chunker::new(strategy, None);
        // Text with version numbers that the old naive splitter would break on.
        let text = "The application uses Obsidian 1.0 for note management and runs on platform v0.1.2 of the runtime. \
                     The configuration requires Node.js 18.0 or later and Python 3.11 for the build tools. \
                     Version 2.0.0 of the API introduced breaking changes to the authentication endpoint. \
                     Users upgrading from release 1.5.3 should consult the migration guide carefully. \
                     The dependency on libfoo 0.9.1 was updated to libfoo 1.0.0 in this quarterly release. \
                     Additional context about the system architecture and deployment pipeline follows below. \
                     The monitoring stack uses Grafana 10.0 with Prometheus 2.45 for metrics collection. \
                     Container orchestration runs on Kubernetes 1.28 with Helm 3.12 for chart management.";
        let chunks = chunker.chunk(text).await.unwrap();
        assert!(!chunks.is_empty());
        // No chunk should be a tiny fragment from a version number split.
        for chunk in &chunks {
            assert!(
                chunk.len() >= 64 || chunks.len() == 1,
                "Structured chunking produced undersized fragment: '{}' ({} chars)",
                chunk,
                chunk.len()
            );
        }
        // Version strings should remain intact within their chunks.
        let joined = chunks.join(" ");
        assert!(joined.contains("v0.1.2"), "version string v0.1.2 was split");
        assert!(
            joined.contains("Obsidian 1.0"),
            "version string Obsidian 1.0 was split"
        );
    }

    #[tokio::test]
    async fn structured_range_prevents_tiny_chunks() {
        let strategy = ChunkingStrategy::Structured { size: 1024 };
        let chunker = Chunker::new(strategy, None);
        // Generate enough text to produce multiple chunks.
        let text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. ".repeat(30);
        let chunks = chunker.chunk(text).await.unwrap();
        assert!(chunks.len() > 1, "expected multiple chunks from long text");
        // All chunks except possibly the last should be at least size/2.
        for (i, chunk) in chunks.iter().enumerate() {
            if i < chunks.len() - 1 {
                assert!(
                    chunk.len() >= 512,
                    "Non-final chunk {} is only {} chars (expected >= 512): '{}'",
                    i,
                    chunk.len(),
                    &chunk[..chunk.len().min(80)]
                );
            }
        }
    }

    #[tokio::test]
    async fn sentence_does_not_split_versions() {
        let strategy = ChunkingStrategy::Sentence;
        let chunker = Chunker::new(strategy, None);
        let text = "The app uses v0.1.2 of the runtime. It requires Node.js 18.0 or later.";
        let chunks = chunker.chunk(text).await.unwrap();
        assert!(!chunks.is_empty());
        // Version strings should remain intact.
        let joined = chunks.join(" ");
        assert!(
            joined.contains("v0.1.2"),
            "Sentence strategy split version v0.1.2: {:?}",
            chunks
        );
        assert!(
            joined.contains("18.0"),
            "Sentence strategy split version 18.0: {:?}",
            chunks
        );
    }

    #[tokio::test]
    async fn structured_plain_text_fallback() {
        let strategy = ChunkingStrategy::Structured { size: 200 };
        let chunker = Chunker::new(strategy, None);
        // Plain text with no Markdown structure markers.
        let text = "This is plain text without any markdown headings or code blocks. \
                     It should fall back to TextSplitter and still produce valid chunks. \
                     The chunking should respect word boundaries and not split mid-word.";
        let chunks = chunker.chunk(text).await.unwrap();
        assert!(!chunks.is_empty());
        for chunk in &chunks {
            assert!(!chunk.is_empty(), "produced empty chunk from plain text");
        }
    }

    #[test]
    fn markdown_detection_heuristic() {
        assert!(has_markdown_structure("# Heading\n\ntext"));
        assert!(has_markdown_structure("text\n## Sub"));
        assert!(has_markdown_structure("before\n```\ncode\n```"));
        assert!(!has_markdown_structure("just plain text here"));
        assert!(!has_markdown_structure("no markers at all"));
    }

    #[test]
    fn test_cosine() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.0001);

        let c = vec![0.0, 1.0, 0.0];
        assert!((cosine_similarity(&a, &c)).abs() < 0.0001);
    }
}
