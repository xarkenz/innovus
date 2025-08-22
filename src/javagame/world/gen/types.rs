use innovus::tools::*;
use crate::world::block::BlockCoord;
use super::*;

#[derive(Debug)]
pub struct OverworldGenerator {
    world_seed: u64,
    small_terrain: PerlinNoise1D,
    big_terrain: PerlinNoise1D,
    small_caves: PerlinNoise2D,
    big_caves: PerlinNoise2D,
}

impl OverworldGenerator {
    pub fn new(world_seed: u64) -> Self {
        let scramble_1 = scramble(world_seed);
        let scramble_2 = scramble(scramble_1);
        let scramble_3 = scramble(scramble_2);
        let scramble_4 = scramble(scramble_3);
        Self {
            world_seed,
            small_terrain: PerlinNoise1D::new(scramble_1),
            big_terrain: PerlinNoise1D::new(scramble_2),
            small_caves: PerlinNoise2D::new(scramble_3),
            big_caves: PerlinNoise2D::new(scramble_4),
        }
    }

    pub fn get_height_map(&self, chunk_x: i64) -> [i64; block::CHUNK_SIZE] {
        let small_terrain_cell = self.small_terrain.get_cell(chunk_x.div_euclid(2));
        let small_offset_base = chunk_x.rem_euclid(2) as f32 / 2.0;
        let big_terrain_cell = self.big_terrain.get_cell(chunk_x.div_euclid(4));
        let big_offset_base = chunk_x.rem_euclid(4) as f32 / 4.0;

        let mut x_offset = 0;
        [(); block::CHUNK_SIZE].map(|_| {
            let offset = x_offset as f32 / block::CHUNK_SIZE as f32;
            let small_offset = small_offset_base + offset / 2.0;
            let small_terrain_value = small_terrain_cell.compute_value(small_offset, smooth_step);
            let big_offset = big_offset_base + offset / 4.0;
            let big_terrain_value = big_terrain_cell.compute_value(big_offset, smooth_step);
            x_offset += 1;
            (small_terrain_value * 20.0 + big_terrain_value * 50.0).round() as i64
        })
    }
}

impl WorldGenerator for OverworldGenerator {
    fn world_seed(&self) -> u64 {
        self.world_seed
    }

    fn generate_chunk(&self, chunk: &mut block::Chunk, chunk_map: &block::ChunkMap, physics: &mut phys::Physics) -> Vec<Box<dyn entity::Entity>> {
        let height_map = self.get_height_map(chunk.location().x());
        chunk.set_height_map(height_map);

        let big_offset_base = chunk.location().map(|x| x.rem_euclid(2) as f32 / 2.0);
        let big_location = chunk.location().map(|x| x.div_euclid(2));

        let small_caves_cell = self.small_caves.get_cell(chunk.location());
        let big_caves_cell = self.big_caves.get_cell(big_location);

        for y in 0..block::CHUNK_SIZE {
            for x in 0..block::CHUNK_SIZE {
                let block_y = i64::from(BlockCoord::new(chunk.location().y(), y));
                let terrain_height = height_map[x];

                let block_type;
                if block_y > terrain_height {
                    block_type = &block::types::AIR;
                }
                else if block_y == terrain_height {
                    block_type = &block::types::GRASSY_DIRT;
                }
                else if terrain_height - block_y <= 4 {
                    block_type = &block::types::DIRT;
                }
                else {
                    let small_offset = Vector([x as f32, y as f32]) / block::CHUNK_SIZE as f32;
                    let small_caves_value = small_caves_cell.compute_value(small_offset, smooth_step);
                    let big_offset = big_offset_base + small_offset / 2.0;
                    let big_caves_value = big_caves_cell.compute_value(big_offset, smooth_step);
                    let mut value = 0.8 * small_caves_value + 0.5 * big_caves_value;

                    if terrain_height - block_y <= 25 {
                        value *= smooth_step(0.0, 1.0, (terrain_height - block_y - 5) as f32 / 20.0);
                    }

                    if value < -0.1 {
                        block_type = &block::types::SLATE;
                    }
                    else if value < 0.2 {
                        block_type = &block::types::STONE;
                    }
                    else {
                        block_type = &block::types::AIR;
                    }
                }

                let block = block::Block::new(block_type);
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
                    let block = block::Block::new(&block::types::DIRT);
                    chunk.set_block_at(x, y, block, chunk_map, physics);
                }
            }
            for x in 0..16 {
                let block = block::Block::new(&block::types::GRASSY_DIRT);
                chunk.set_block_at(x, 15, block, chunk_map, physics);
            }
        }
        else if chunk.location().y() < -1 {
            for y in 0..16 {
                for x in 0..16 {
                    let block = block::Block::new(&block::types::STONE);
                    chunk.set_block_at(x, y, block, chunk_map, physics);
                }
            }
        }
        else {
            for y in 0..16 {
                for x in 0..16 {
                    let block = block::Block::new(&block::types::AIR);
                    chunk.set_block_at(x, y, block, chunk_map, physics);
                }
            }
        }
        Vec::new()
    }
}
