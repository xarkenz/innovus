use std::hash::{Hash, Hasher};
use innovus::tools::*;
use crate::tools::noise::SimpleHasher;
use super::*;

pub mod types;

pub trait WorldGenerator {
    fn world_seed(&self) -> u64;

    fn generate_chunk(&self, chunk: &mut block::Chunk, chunk_map: &block::ChunkMap, physics: &mut phys::Physics) -> Vec<Box<dyn entity::Entity>>;

    fn get_chunk_seed(&self, location: block::ChunkLocation) -> u64 {
        let mut hasher = SimpleHasher::with_seed(self.world_seed());
        location.x().hash(&mut hasher);
        location.y().hash(&mut hasher);
        hasher.finish()
    }
}
