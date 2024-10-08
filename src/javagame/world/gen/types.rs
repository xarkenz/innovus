use innovus::tools::*;

use super::*;
use crate::world::*;

#[derive(Debug)]
pub struct OverworldGenerator {
    seed: u64,
}

impl WorldGenerator for OverworldGenerator {
    fn new(seed: u64) -> Self {
        Self { seed }
    }

    fn seed(&self) -> u64 {
        self.seed
    }

    fn get_chunk(&self, location: block::ChunkLocation) -> block::Chunk {
        if location == Vector([0, -1]) || location == Vector([-1, -1]) {
            let mut chunk = block::Chunk::new(location);
            for y in 8..15 {
                for x in 0..16 {
                    chunk.set_block_at(x, y, block::Block::new(&block::types::DIRT, 0, 15));
                }
            }
            for x in 0..16 {
                chunk.set_block_at(x, 15, block::Block::new(&block::types::GRASSY_DIRT, 0, 15));
            }
            chunk
        }
        else {
            block::Chunk::new(location)
        }
    }
}
