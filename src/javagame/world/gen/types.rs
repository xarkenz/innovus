use innovus::tools::*;

use super::*;

fn smooth_step(a: f32, b: f32, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    (b - a) * (3.0 - 2.0 * t) * t * t + a
}

fn scramble(mut n: u64) -> u64 {
    n ^= n >> 33;
    n = n.wrapping_mul(0xFF51AFD7ED558CCD);
    n ^= n >> 33;
    n = n.wrapping_mul(0xC4CEB9FE1A85EC53);
    n ^= n >> 33;
    n
}

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
        if chunk.location().y() < 0 {
            let seed_dl = self.get_chunk_seed(chunk.location());
            let seed_dr = self.get_chunk_seed(chunk.location() + Vector([1, 0]));
            let seed_ul = self.get_chunk_seed(chunk.location() + Vector([0, 1]));
            let seed_ur = self.get_chunk_seed(chunk.location() + Vector([1, 1]));

            let [grad_dl, grad_dr, grad_ul, grad_ur] = [seed_dl, seed_dr, seed_ul, seed_ur].map(|seed| {
                let theta = scramble(seed) as f32 / u64::MAX as f32 * std::f32::consts::TAU;
                Vector([theta.cos(), theta.sin()]) / block::CHUNK_SIZE as f32
            });

            for y in 0..block::CHUNK_SIZE {
                for x in 0..block::CHUNK_SIZE {
                    let offset_dl = Vector([x as f32, y as f32]);
                    let offset_dr = Vector([-((block::CHUNK_SIZE - x) as f32), y as f32]);
                    let offset_ul = Vector([x as f32, -((block::CHUNK_SIZE - y) as f32)]);
                    let offset_ur = Vector([-((block::CHUNK_SIZE - x) as f32), -((block::CHUNK_SIZE - y) as f32)]);

                    let dot_dl = grad_dl.dot(&offset_dl);
                    let dot_dr = grad_dr.dot(&offset_dr);
                    let dot_ul = grad_ul.dot(&offset_ul);
                    let dot_ur = grad_ur.dot(&offset_ur);

                    let tx = x as f32 / block::CHUNK_SIZE as f32;
                    let ty = y as f32 / block::CHUNK_SIZE as f32;
                    let value = smooth_step(
                        smooth_step(dot_dl, dot_dr, tx),
                        smooth_step(dot_ul, dot_ur, tx),
                        ty,
                    );

                    let block_type = if value < -0.0 {
                        &block::types::STONE
                    } else if value < 0.2 {
                        &block::types::DIRT
                    } else {
                        &block::types::AIR
                    };
                    let block = block::Block::new(block_type, 0, 0);
                    chunk.set_block_at(x, y, block, chunk_map, physics);
                }
            }
        }

        Vec::new()
    }
}

#[derive(Debug)]
pub struct TestWorldGenerator {
    seed: u64,
}

impl TestWorldGenerator {
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
        }
    }
}

impl WorldGenerator for TestWorldGenerator {
    fn seed(&self) -> u64 {
        self.seed
    }

    fn generate_chunk(&self, chunk: &mut block::Chunk, chunk_map: &block::ChunkMap, physics: &mut phys::Physics) -> Vec<Box<dyn entity::Entity>> {
        if chunk.location().y() == -1 {
            for y in 0..15 {
                for x in 0..16 {
                    let block = block::Block::new(&block::types::DIRT, 0, 0);
                    chunk.set_block_at(x, y, block, chunk_map, physics);
                }
            }
            for x in 0..16 {
                let block = block::Block::new(&block::types::GRASSY_DIRT, 0, 0);
                chunk.set_block_at(x, 15, block, chunk_map, physics);
            }
        }
        else if chunk.location().y() < -1 {
            for y in 0..16 {
                for x in 0..16 {
                    let block = block::Block::new(&block::types::STONE, 0, 0);
                    chunk.set_block_at(x, y, block, chunk_map, physics);
                }
            }
        }
        else {
            for y in 0..16 {
                for x in 0..16 {
                    let block = block::Block::new(&block::types::AIR, 0, 0);
                    chunk.set_block_at(x, y, block, chunk_map, physics);
                }
            }
        }
        Vec::new()
    }
}
