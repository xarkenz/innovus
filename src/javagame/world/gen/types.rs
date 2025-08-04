use innovus::tools::*;

use super::*;
use crate::world::*;

#[derive(Debug)]
pub struct OverworldGenerator {
    seed: u64,
}

impl OverworldGenerator {
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
        }
    }
}

impl WorldGenerator for OverworldGenerator {
    fn seed(&self) -> u64 {
        self.seed
    }

    fn generate_chunk(&self, chunk: &mut block::Chunk, chunk_map: &block::ChunkMap, physics: &mut phys::Physics) -> Vec<Box<dyn entity::Entity>> {
        if chunk.location() == Vector([0, -1]) || chunk.location() == Vector([-1, -1]) {
            for y in 8..15 {
                for x in 0..16 {
                    let block = block::Block::new(&block::types::DIRT, 0, 15);
                    chunk.set_block_at(x, y, block, chunk_map, physics);
                }
            }
            for x in 0..16 {
                let block = block::Block::new(&block::types::GRASSY_DIRT, 0, 15);
                chunk.set_block_at(x, 15, block, chunk_map, physics);
            }
        }
        Vec::new()
    }
}
