
use crate::rag::{
    loading::loaded_data::LoadedFile,
    models::{chunks::Chunk, ChunkedFile}, processing::{ChunkOverlap, ChunkSize},
};


/// A hierarchical chunking strategy that:
/// 1. Splits into paragraphs
/// 2. Splits paragraphs into sentences
/// 3. Accumulates sentences into chunks up to `chunk_size` words
/// 4. Applies word-level overlap between chunks
pub fn hierarchical_chunking(
    file: LoadedFile,
    chunk_size: &ChunkSize,
    overlap: &ChunkOverlap,
) -> ChunkedFile<Chunk> {
    let chunk_size = *chunk_size as usize;
    let overlap = *overlap as usize;

    // 1) Split the document by paragraphs
    let paragraphs = split_into_paragraphs(&file.content);

    let mut chunks = Vec::new();
    let mut chunk_id = 0;

    for paragraph in paragraphs {
        // 2) Split paragraph into sentences
        let sentences = split_into_sentences(&paragraph);

        // 3) Flatten sentences into a word-level list, tagging which sentence each word belongs to
        let flattened: Vec<(String, usize)> = sentences
            .iter()
            .enumerate()
            .flat_map(|(s_idx, sentence)| {
                sentence
                    .split_whitespace()
                    .map(move |word| (word.to_owned(), s_idx))
            })
            .collect();

        // We'll keep a sliding index in "word-space" to accumulate chunk_size words
        // while preserving entire sentences if possible
        let mut start_word_index = 0;

        while start_word_index < flattened.len() {
            // We accumulate until we reach chunk_size or run out of text.
            let mut current_word_count = 0;
            let mut end_word_index = start_word_index;

            while end_word_index < flattened.len() && current_word_count < chunk_size {
                let sentence_idx = flattened[end_word_index].1;

                // Count how many words remain in the current sentence
                let sentence_word_count = count_words_in_sentence(&flattened, end_word_index, sentence_idx);

                // If including the entire sentence does not exceed chunk_size too badly, include it.
                // If it drastically exceeds chunk_size, we'll chunk the sentence itself.
                // Accept the entire sentence
                current_word_count += sentence_word_count;
                end_word_index += sentence_word_count;
                
            }

            // If we haven't advanced at all (an extremely large single word?), push 1 word to avoid infinite loop
            if end_word_index == start_word_index {
                end_word_index = std::cmp::min(start_word_index + 1, flattened.len());
            }

            // Gather words for this chunk
            let chunk_words: Vec<&str> = flattened[start_word_index..end_word_index]
                .iter()
                .map(|(w, _)| w.as_str())
                .collect();
            let text = chunk_words.join(" ");

            // Push chunk
            chunks.push(Chunk {
                seq_num: chunk_id,
                text,
                embedding_vector: None,
            });
            chunk_id += 1;

            if end_word_index >= flattened.len() {
                break;
            }

            // Overlap in "word space"
            let step = if chunk_size > overlap {
                chunk_size - overlap
            } else {
                // fallback to ensure we don't get stuck
                1
            };
            start_word_index += step;
        }
    }

    (file, chunks).into()
}

/// Split a string into paragraphs by double newlines or some heuristic
fn split_into_paragraphs(content: &str) -> Vec<String> {
    content
        .split("\n\n") // naive: double-newline as a paragraph delimiter
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

/// Naively split a paragraph into sentences by '.', '?', '!'.
/// You could replace this with a more robust solution.
fn split_into_sentences(paragraph: &str) -> Vec<String> {
    let mut sentences = paragraph
        .split(|c: char| c == '.' || c == '?' || c == '!')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_owned())
        .collect::<Vec<String>>();

    // Optionally reinsert punctuation or at least add a period if needed
    // for clarity in the final chunk.
    for sentence in sentences.iter_mut() {
        if !sentence.ends_with('.') && !sentence.ends_with('?') && !sentence.ends_with('!') {
            sentence.push('.');
        }
    }

    sentences
}

/// Count how many words remain in the `start`-th position up to the end of the given sentence_index.
fn count_words_in_sentence(
    flattened: &[(String, usize)],
    start: usize,
    target_sentence_idx: usize,
) -> usize {
    flattened
        .iter()
        .skip(start)
        .take_while(|(_, s_idx)| *s_idx == target_sentence_idx)
        .count()
}
