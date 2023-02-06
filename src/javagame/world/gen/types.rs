use innovus::tools::*;

use super::*;
use crate::world::*;

#[derive(Debug)]
pub struct OverworldGenerator {
    seed: u64,
}

impl WorldGenerator for OverworldGenerator {
    fn new(seed: u64) -> Self {
        OverworldGenerator { seed }
    }

    fn seed(&self) -> u64 {
        self.seed
    }

    fn get_chunk(&self, location: block::ChunkLocation) -> block::Chunk {
        if location == Vector([0, -1]) || location == Vector([-1, -1]) {
            let mut chunk = block::Chunk::new(location);
            for y in 0..block::CHUNK_SIZE {
                for x in 0..block::CHUNK_SIZE {
                    chunk.set_block_at(x, y, block::Block::new(&block::types::AIR, 0, 15));
                }
            }
            chunk
        } else {
            block::Chunk::new(location)
        }
    }
}
