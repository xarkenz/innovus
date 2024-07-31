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
            if location == Vector([-1, -1]) {
                chunk.set_block_at(3, 8, block::Block::new(&block::types::COPPER_BLOCK, 0, 15));
                chunk.set_block_at(3, 7, block::Block::new(&block::types::COPPER_WIRE, 0, 15));
                chunk.set_block_at(3, 6, block::Block::new(&block::types::COPPER_WIRE, 0, 15));
                chunk.set_block_at(4, 6, block::Block::new(&block::types::COPPER_WIRE, 0, 15));
                chunk.set_block_at(5, 6, block::Block::new(&block::types::COPPER_WIRE, 0, 15));
                chunk.set_block_at(6, 6, block::Block::new(&block::types::COPPER_WIRE, 0, 15));
                chunk.set_block_at(6, 5, block::Block::new(&block::types::COPPER_WIRE, 0, 15));
                chunk.set_block_at(6, 7, block::Block::new(&block::types::COPPER_WIRE, 0, 15));
                chunk.set_block_at(7, 7, block::Block::new(&block::types::COPPER_WIRE, 0, 15));
                chunk.set_block_at(8, 7, block::Block::new(&block::types::COPPER_WIRE, 0, 15));
                chunk.set_block_at(8, 8, block::Block::new(&block::types::COPPER_BLOCK, 0, 15));
                chunk.set_block_at(15, 7, block::Block::new(&block::types::DIRT, 0, 15));
            }
            chunk
        }
        else {
            block::Chunk::new(location)
        }
    }
}
