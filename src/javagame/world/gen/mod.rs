use innovus::tools::*;

use super::*;

pub mod types;

pub trait WorldGenerator {
    fn new(seed: u64) -> Self
    where
        Self: Sized;

    fn seed(&self) -> u64;

    fn generate_chunk(&self, chunk: &mut block::Chunk, chunk_map: &block::ChunkMap, physics: &mut phys::Physics) -> Vec<Box<dyn entity::Entity>> {
        let _ = (chunk, chunk_map, physics);
        Vec::new()
    }
}
