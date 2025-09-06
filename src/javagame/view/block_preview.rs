use innovus::gfx::{Geometry, Vertex2D};
use innovus::tools::Vector;
use crate::tools::asset::AssetPool;
use crate::world::block::{self, Block, BlockType, ChunkMap, CHUNK_SIZE, QUADRANT_OFFSETS, QUADRANT_VERTEX_OFFSETS};

pub struct BlockPreview {
    position: Vector<f32, 2>,
    block_type: &'static BlockType,
    opacity: f32,
    geometry: Geometry<Vertex2D>,
}

impl BlockPreview {
    pub fn new(position: Vector<f32, 2>, block_type: &'static BlockType, opacity: f32) -> Self {
        Self {
            position,
            block_type,
            opacity,
            geometry: Geometry::new_render().unwrap(),
        }
    }

    pub fn position(&self) -> Vector<f32, 2> {
        self.position
    }

    pub fn set_position(&mut self, position: Vector<f32, 2>) {
        self.position = position;
    }

    pub fn block_type(&self) -> &'static BlockType {
        self.block_type
    }

    pub fn set_block_type(&mut self, block_type: &'static BlockType) {
        self.block_type = block_type;
    }

    pub fn opacity(&self) -> f32 {
        self.opacity
    }

    pub fn set_opacity(&mut self, opacity: f32) {
        self.opacity = opacity;
    }

    pub fn render(&mut self, assets: &AssetPool, chunks: &ChunkMap) {
        let chunk_location = Vector([
            self.position.x().div_euclid(CHUNK_SIZE as f32) as i64,
            self.position.y().div_euclid(CHUNK_SIZE as f32) as i64,
        ]);
        let Some(chunk) = chunks.get(chunk_location) else {
            return;
        };

        let block_x = self.position.x().rem_euclid(CHUNK_SIZE as f32) as usize;
        let block_y = self.position.y().rem_euclid(CHUNK_SIZE as f32) as usize;
        let slot = chunk.block_slot_at(block_x, block_y);
        if slot.block().block_type() != &block::types::AIR {
            return;
        }
        let light_value = slot.light_value();

        let block = Block::new(self.block_type);
        if let Some(image) = assets.get_block_image(&block, chunk_location, block_x, block_y) {
            let atlas_offsets = image.get_quadrant_atlas_offsets(chunks, &*chunk, &block, block_x, block_y);
            let block_origin = self.position.map(f32::floor);

            let mut vertices = Vec::new();
            let mut faces = Vec::new();
            for (quadrant_offset, atlas_offset) in std::iter::zip(QUADRANT_OFFSETS, atlas_offsets) {
                let index = vertices.len() as u32;
                faces.push([index + 0, index + 1, index + 2]);
                faces.push([index + 2, index + 3, index + 0]);
                for vertex_offset in QUADRANT_VERTEX_OFFSETS {
                    let total_offset = quadrant_offset + vertex_offset;
                    vertices.push(Vertex2D::new(
                        (block_origin + total_offset).with_z(0.0),
                        Some(Vector([light_value, light_value, light_value, self.opacity])),
                        Some(Vector([
                            atlas_offset.x() as f32 + total_offset.x() * image.size() as f32,
                            atlas_offset.y() as f32 + (1.0 - total_offset.y()) * image.size() as f32,
                        ])),
                    ));
                }
            }

            self.geometry.clear();
            self.geometry.add(&vertices, &faces);

            assets.block_texture().bind();
            self.geometry.render();
        }
    }
}
