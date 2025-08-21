use std::hash::{Hash, Hasher};
use fnv::FnvHasher;
use innovus::tools::*;

use super::*;

pub mod types;

pub trait WorldGenerator {
    fn seed(&self) -> u64;

    fn generate_chunk(&self, chunk: &mut block::Chunk, chunk_map: &block::ChunkMap, physics: &mut phys::Physics) -> Vec<Box<dyn entity::Entity>>;

    fn get_chunk_seed(&self, location: block::ChunkLocation) -> u64 {
        let mut hasher = SeedHasher::from_world_seed(self.seed());
        location.x().hash(&mut hasher);
        location.y().hash(&mut hasher);
        hasher.finish()
    }
}

pub struct SeedHasher {
    state: FnvHasher,
}

impl SeedHasher {
    pub fn from_world_seed(seed: u64) -> Self {
        Self {
            state: FnvHasher::with_key(seed),
        }
    }
}

impl Hasher for SeedHasher {
    fn finish(&self) -> u64 {
        self.state.finish()
    }

    fn write(&mut self, bytes: &[u8]) {
        self.state.write(bytes);
    }

    fn write_u16(&mut self, i: u16) {
        self.write(&i.to_be_bytes());
    }

    fn write_u32(&mut self, i: u32) {
        self.write(&i.to_be_bytes());
    }

    fn write_u64(&mut self, i: u64) {
        self.write(&i.to_be_bytes());
    }

    fn write_u128(&mut self, i: u128) {
        self.write(&i.to_be_bytes());
    }

    fn write_usize(&mut self, i: usize) {
        self.write(&i.to_be_bytes());
    }
}
