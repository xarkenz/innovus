use innovus::tools::*;

use super::*;

#[derive(Debug)]
pub struct OverworldGenerator {
    world_seed: u64,
    small_terrain: PerlinNoise1D,
    small_caves: PerlinNoise2D,
    big_caves: PerlinNoise2D,
}

impl OverworldGenerator {
    pub fn new(world_seed: u64) -> Self {
        let scramble_1 = scramble(world_seed);
        let scramble_2 = scramble(scramble_1);
        let scramble_3 = scramble(scramble_2);
        Self {
            world_seed,
            small_terrain: PerlinNoise1D::new(scramble_1),
            small_caves: PerlinNoise2D::new(scramble_2),
            big_caves: PerlinNoise2D::new(scramble_3),
        }
    }

    pub fn get_terrain_heights(&self, chunk_x: i64) -> Box<[(i64, usize)]> {
        let small_terrain_cell = self.small_terrain.get_cell(chunk_x);

        (0..block::CHUNK_SIZE)
            .map(|x_offset| {
                let small_terrain_value = small_terrain_cell.compute_value(x_offset as f32 / block::CHUNK_SIZE as f32, smooth_step);
                let raw_height = ((small_terrain_value + 1.0) * 30.0).round() as i64;
                (
                    raw_height.div_euclid(block::CHUNK_SIZE as i64),
                    raw_height.rem_euclid(block::CHUNK_SIZE as i64) as usize,
                )
            })
            .collect()
    }
}

impl WorldGenerator for OverworldGenerator {
    fn world_seed(&self) -> u64 {
        self.world_seed
    }

    fn generate_chunk(&self, chunk: &mut block::Chunk, chunk_map: &block::ChunkMap, physics: &mut phys::Physics) -> Vec<Box<dyn entity::Entity>> {
        let terrain_heights = self.get_terrain_heights(chunk.location().x());

        let big_offset = chunk.location().map(|x| x.rem_euclid(2) as f32 / 2.0);
        let big_location = chunk.location().map(|x| x.div_euclid(2));

        let small_caves_cell = self.small_caves.get_cell(chunk.location());
        let big_caves_cell = self.big_caves.get_cell(big_location);

        for y in 0..block::CHUNK_SIZE {
            for x in 0..block::CHUNK_SIZE {
                let (height_chunk, height_block) = terrain_heights[x];
                let block_type;

                if chunk.location().y() == height_chunk && y == height_block {
                    block_type = &block::types::GRASSY_DIRT;
                }
                else if chunk.location().y() < height_chunk || (chunk.location().y() == height_chunk && y < height_block) {
                    let small_offset = Vector([x as f32, y as f32]) / block::CHUNK_SIZE as f32;
                    let small_caves_value = small_caves_cell.compute_value(small_offset, smooth_step);
                    let big_offset = big_offset + Vector([x as f32, y as f32]) / (block::CHUNK_SIZE as f32 * 2.0);
                    let big_caves_value = big_caves_cell.compute_value(big_offset, smooth_step);
                    let value = 0.8 * small_caves_value + 0.5 * big_caves_value;

                    if value < 0.0 {
                        block_type = &block::types::SLATE;
                    }
                    else if value < 0.2 {
                        block_type = &block::types::STONE;
                    }
                    else {
                        block_type = &block::types::AIR;
                    }
                }
                else {
                    block_type = &block::types::AIR;
                }

                let block = block::Block::new(block_type, 0, 0);
                chunk.set_block_at(x, y, block, chunk_map, physics);
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
    fn world_seed(&self) -> u64 {
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
