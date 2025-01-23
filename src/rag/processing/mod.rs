use chunking::{paragraph::hierarchical_chunking, simple::simple_word_chunking};

use super::{loading::loaded_data::LoadedFile, models::{chunks::Chunk, ChunkedFile}};

mod prepare;
mod dedup_embeddings;
mod prompt;
mod hype;
mod embedd_file;
mod summarize;
mod chunking;
mod recursive_prompt;

pub use dedup_embeddings::dedup;
pub use hype::hype;
pub use prompt::prompt;
pub use recursive_prompt::recursive_prompt;
pub use prepare::prepare_for_upload;

type ChunkSize = i32;
type ChunkOverlap = i32;

pub enum ChunkingStrategy {
    Word(ChunkSize, ChunkOverlap),
    Hierarchical(ChunkSize, ChunkOverlap),
    
}

pub fn chunk(file: LoadedFile, strategy: ChunkingStrategy) -> ChunkedFile<Chunk> {
    match &strategy {
        ChunkingStrategy::Word(size, overlap) => simple_word_chunking(file, size, overlap),
        ChunkingStrategy::Hierarchical(size, overlap) => hierarchical_chunking(file, size, overlap),
        
    }
}