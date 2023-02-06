use innovus::tools::*;

use super::*;

pub mod types;

pub trait WorldGenerator {
    fn new(seed: u64) -> Self
    where
        Self: Sized;

    fn seed(&self) -> u64;

    fn get_chunk(&self, location: block::ChunkLocation) -> block::Chunk {
        block::Chunk::new(location)
    }

    fn get_chunk_entities(&self, location: block::ChunkLocation) -> Vec<Box<dyn entity::Entity>> {
        Vec::new()
    }
}
