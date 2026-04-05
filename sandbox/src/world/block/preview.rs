use innovus::gfx::Gfx;
use innovus::gfx::mesh::{MeshRenderer, Vertex2D};
use innovus::gfx::pipeline::BindGroup;
use innovus::tools::Vector;
use crate::tools::asset::AssetPool;
use crate::world::block::{Block, BlockSide, ChunkMap, CHUNK_SIZE, QUADRANT_OFFSETS, QUADRANT_VERTEX_OFFSETS};
use crate::world::block::types::AIR;
use crate::world::item::ItemType;

pub struct BlockPreview {
    position: Option<Vector<f32, 2>>,
    item_type: &'static ItemType,
    opacity: f32,
    mesh: MeshRenderer<Vertex2D>,
}

impl BlockPreview {
    pub fn create(gfx: &Gfx, position: Option<Vector<f32, 2>>, item_type: &'static ItemType, opacity: f32) -> Self {
        Self {
            position,
            item_type,
            opacity,
            mesh: MeshRenderer::create(gfx),
        }
    }

    pub fn position(&self) -> Option<Vector<f32, 2>> {
        self.position
    }

    pub fn set_position(&mut self, position: Option<Vector<f32, 2>>) {
        self.position = position;
    }

    pub fn item_type(&self) -> &'static ItemType {
        self.item_type
    }

    pub fn set_item_type(&mut self, item_type: &'static ItemType) {
        self.item_type = item_type;
    }

    pub fn opacity(&self) -> f32 {
        self.opacity
    }

    pub fn set_opacity(&mut self, opacity: f32) {
        self.opacity = opacity;
    }

    pub fn render(&mut self, render_pass: &mut wgpu::RenderPass, assets: &AssetPool, chunks: &ChunkMap) {
        let Some(position) = self.position else {
            return;
        };

        if let Some(block_type) = self.item_type.block_type() {
            let chunk_location = Vector([
                position.x().div_euclid(CHUNK_SIZE as f32) as i64,
                position.y().div_euclid(CHUNK_SIZE as f32) as i64,
            ]);
            let Some(chunk) = chunks.get(chunk_location) else {
                return;
            };

            let block_x = position.x().rem_euclid(CHUNK_SIZE as f32) as usize;
            let block_y = position.y().rem_euclid(CHUNK_SIZE as f32) as usize;
            if block_x >= 16 || block_y >= 16 {
                // FIXME: at one point the game crashed, supposedly because this case occurred.
                //        i have no idea how it would have happened, but hopefully this will help
                //        figure it out the next time it happens
                eprintln!("x={block_x}, y={block_y}, fx={:.8}, fy={:.8}", position.x(), position.y());
            }
            let slot = chunk.block_slot_at(block_x, block_y);
            if slot.block().block_type() != &AIR {
                return;
            }
            let light_value = slot.light_value();

            let block = Block::new(block_type, BlockSide::from_position(position));
            if let Some(image) = assets.get_block_image(&block, chunk_location, block_x, block_y) {
                let atlas_offsets = image.get_quadrant_atlas_offsets(chunks, &*chunk, &block, block_x, block_y);
                let block_origin = position.map(f32::floor);

                let mut vertices = Vec::new();
                let mut faces = Vec::new();
                for (quadrant_offset, atlas_offset) in std::iter::zip(QUADRANT_OFFSETS, atlas_offsets) {
                    let index = vertices.len() as u32;
                    faces.push([index + 0, index + 1, index + 2]);
                    faces.push([index + 2, index + 3, index + 0]);
                    for vertex_offset in QUADRANT_VERTEX_OFFSETS {
                        let total_offset = quadrant_offset + vertex_offset;
                        vertices.push(Vertex2D::new(
                            (block_origin + total_offset).with_z(1.0),
                            Some(Vector([light_value, light_value, light_value, self.opacity])),
                            Some(Vector([
                                atlas_offset.x() as f32 + total_offset.x() * image.size() as f32,
                                atlas_offset.y() as f32 + (1.0 - total_offset.y()) * image.size() as f32,
                            ])),
                        ));
                    }
                }

                self.mesh.clear();
                self.mesh.add(&vertices, &faces);

                render_pass.set_pipeline(assets.block_pipeline());
                render_pass.set_bind_group(0, assets.block_texture().bind_group(), &[]);
                self.mesh.render(render_pass);
            }
        }
    }
}
