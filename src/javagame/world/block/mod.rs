use std::cell::RefCell;
use std::collections::BTreeMap;
use innovus::{gfx::*, tools::*};
use std::fmt::Formatter;
use crate::tools::asset::AssetPool;

pub mod types;

pub use types::BLOCK_TYPES;

#[derive(Debug)]
pub enum AttributeType {
    Bool(bool),
    U8(u8),
    I8(i8),
    U32(u32),
    I32(i32),
    String(&'static str),
    // default variant index, variant display names
    Enum(u8, &'static [&'static str]),
}

#[derive(Clone, Debug)]
pub enum AttributeValue {
    Bool(bool),
    U8(u8),
    I8(i8),
    U32(u32),
    I32(i32),
    String(String),
}

pub struct BlockType {
    pub name: &'static str,
    pub attributes: &'static [(&'static str, AttributeType)],
    pub colliders: &'static [Rectangle<i32>],
    pub is_full_block: bool,
    pub light_emission: u8,
    pub connector: fn(&Self, &Self) -> bool,
}

impl BlockType {
    pub fn default_attributes(&self) -> Vec<AttributeValue> {
        self.attributes
            .iter()
            .map(|(_, t)| match t {
                &AttributeType::Bool(b) => AttributeValue::Bool(b),
                &AttributeType::U8(n) => AttributeValue::U8(n),
                &AttributeType::I8(n) => AttributeValue::I8(n),
                &AttributeType::U32(n) => AttributeValue::U32(n),
                &AttributeType::I32(n) => AttributeValue::I32(n),
                &AttributeType::String(s) => AttributeValue::String(s.to_string()),
                &AttributeType::Enum(n, _) => AttributeValue::U8(n),
            })
            .collect()
    }

    pub fn connects_to(&self, other: &Self) -> bool {
        (self.connector)(self, other)
    }
}

impl PartialEq for BlockType {
    fn eq(&self, other: &Self) -> bool {
        // Comparing pointers is sufficient
        self as *const Self == other as *const Self
    }
}

impl std::fmt::Debug for BlockType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "BlockType({})", self.name)
    }
}

pub const BLOCK_QUAD_OFFSETS: [Vector2f; 4] = [
    Vector([0.0, 0.5]), // Top left
    Vector([0.5, 0.5]), // Top right
    Vector([0.0, 0.0]), // Bottom left
    Vector([0.5, 0.0]), // Bottom right
];
pub const BLOCK_QUAD_VERTEX_OFFSETS: [Vector2f; 4] = [
    Vector([0.0, 0.0]), // Bottom left
    Vector([0.0, 0.5]), // Top left
    Vector([0.5, 0.5]), // Top right
    Vector([0.5, 0.0]), // Bottom right
];
pub const VERTICES_PER_BLOCK: usize = BLOCK_QUAD_OFFSETS.len() * BLOCK_QUAD_VERTEX_OFFSETS.len();

#[derive(Debug)]
pub struct BlockAppearance {
    pub block_type: &'static BlockType,
    pub offset: Vector<u32, 2>,
    pub resolution: u32,
}

impl BlockAppearance {
    fn get_connect(&self, chunk_map: &ChunkMap, chunk: &Chunk, x: isize, y: isize) -> bool {
        let other_type = chunk
            .with_block(x, y, chunk_map, |block| block.block_type)
            .unwrap_or(&types::AIR);
        self.block_type.connects_to(other_type)
    }

    fn get_quad_type(&self, chunk_map: &ChunkMap, chunk: &Chunk, x: isize, y: isize, x_connect: bool, y_connect: bool) -> u32 {
        match (x_connect, y_connect) {
            (false, false) => 0,
            (true, false) => 1,
            (false, true) => 2,
            (true, true) => if self.get_connect(chunk_map, chunk, x, y) { 4 } else { 3 },
        }
    }

    pub fn get_quad_uv_offsets(&self, chunk_map: &ChunkMap, chunk: &Chunk, x: usize, y: usize) -> [Vector<u32, 2>; 4] {
        let left_x = x as isize - 1;
        let right_x = x as isize + 1;
        let down_y = y as isize - 1;
        let up_y = y as isize + 1;

        let left_connect = self.get_connect(chunk_map, chunk, left_x, y as isize);
        let right_connect = self.get_connect(chunk_map, chunk, right_x, y as isize);
        let down_connect = self.get_connect(chunk_map, chunk, x as isize, down_y);
        let up_connect = self.get_connect(chunk_map, chunk, x as isize, up_y);

        let quad_types: [u32; 4] = [
            self.get_quad_type(chunk_map, chunk, left_x, up_y, left_connect, up_connect),
            self.get_quad_type(chunk_map, chunk, right_x, up_y, right_connect, up_connect),
            self.get_quad_type(chunk_map, chunk, left_x, down_y, left_connect, down_connect),
            self.get_quad_type(chunk_map, chunk, right_x, down_y, right_connect, down_connect),
        ];

        quad_types.map(|quad_type| Vector([
            self.offset.x() + quad_type * self.resolution,
            self.offset.y(),
        ]))
    }
}

#[derive(Clone, Debug)]
pub struct Block {
    pub block_type: &'static BlockType,
    pub attributes: Vec<AttributeValue>,
    pub block_light: u8,
    pub sky_light: u8,
}

impl Block {
    pub fn new(block_type: &'static BlockType, block_light: u8, sky_light: u8) -> Self {
        Self {
            block_type,
            attributes: block_type.default_attributes(),
            block_light,
            sky_light,
        }
    }
}

impl Default for Block {
    fn default() -> Self {
        Self::new(&types::AIR, 0, 15)
    }
}

pub const CHUNK_SIZE: usize = 16;
const CHUNK_SIZE_F32: f32 = CHUNK_SIZE as f32;

fn resolve_relative_coordinate(value: isize) -> (i64, usize) {
    if value < 0 {
        (-1, (value + CHUNK_SIZE as isize) as usize)
    }
    else if value >= CHUNK_SIZE as isize {
        (1, (value - CHUNK_SIZE as isize) as usize)
    }
    else {
        (0, value as usize)
    }
}

pub type ChunkLocation = Vector<i64, 2>;
pub type ChunkMap = BTreeMap<ChunkLocation, RefCell<Chunk>>;

#[derive(Debug)]
pub struct Chunk {
    location: ChunkLocation,
    blocks: [[Block; CHUNK_SIZE]; CHUNK_SIZE],
    block_colliders: [[Box<[phys::ColliderHandle]>; CHUNK_SIZE]; CHUNK_SIZE],
    blocks_dirty: [[bool; CHUNK_SIZE]; CHUNK_SIZE],
    dirty: bool,
    geometry: Geometry<Vertex2D>,
}

impl Chunk {
    pub fn new(location: ChunkLocation) -> Self {
        Self {
            location,
            blocks: Default::default(),
            block_colliders: [(); CHUNK_SIZE].map(|_| [(); CHUNK_SIZE].map(|_| Box::from([]))),
            blocks_dirty: [[true; CHUNK_SIZE]; CHUNK_SIZE],
            dirty: true,
            geometry: Geometry::new_render().unwrap(),
        }
    }

    pub fn location(&self) -> ChunkLocation {
        self.location
    }

    pub fn block_at(&self, x: usize, y: usize) -> &Block {
        &self.blocks[y][x]
    }

    pub fn set_block_at(&mut self, x: usize, y: usize, block: Block, chunk_map: &ChunkMap, physics: &mut phys::Physics) {
        // Add new physics colliders and remove the old ones
        let block_origin = Vector([
            self.location.x() as f32 * CHUNK_SIZE_F32 + x as f32,
            self.location.y() as f32 * CHUNK_SIZE_F32 + y as f32,
        ]);
        let colliders = block.block_type.colliders
            .iter()
            .map(|&bounds| {
                let mut collider_bounds = Rectangle::new(
                    Vector([
                        bounds.min_x() as f32 / 32.0,
                        bounds.min_y() as f32 / 32.0,
                    ]),
                    Vector([
                        bounds.max_x() as f32 / 32.0,
                        bounds.max_y() as f32 / 32.0,
                    ]),
                );
                collider_bounds.shift_by(block_origin);
                physics.add_collider(phys::Collider::new_fixed(collider_bounds))
            })
            .collect();
        let old_colliders = std::mem::replace(&mut self.block_colliders[y][x], colliders);
        for handle in old_colliders {
            physics.remove_collider(handle);
        }

        self.blocks[y][x] = block;
        self.set_dirty_at(x, y, true);
        // Propagate the dirty flag to the surrounding blocks in order to update their appearances
        self.propagate_dirty(x, y, chunk_map);
    }

    pub fn with_block<F, T>(&self, x: isize, y: isize, chunk_map: &ChunkMap, f: F) -> Option<T>
    where
        F: FnOnce(&Block) -> T,
    {
        let (chunk_offset_x, block_x) = resolve_relative_coordinate(x);
        let (chunk_offset_y, block_y) = resolve_relative_coordinate(y);

        if chunk_offset_x == 0 && chunk_offset_y == 0 {
            Some(f(self.block_at(block_x, block_y)))
        }
        else {
            let other_chunk_location = Vector([
                self.location.x() + chunk_offset_x,
                self.location.y() + chunk_offset_y,
            ]);
            chunk_map.get(&other_chunk_location).map(|other_chunk| {
                f(other_chunk.borrow().block_at(block_x, block_y))
            })
        }
    }

    pub fn propagate_dirty(&mut self, x: usize, y: usize, chunk_map: &ChunkMap) {
        for dy in [-1, 0, 1] {
            for dx in [-1, 0, 1] {
                if dx == 0 && dy == 0 {
                    continue;
                }

                let (chunk_offset_x, dirty_x) = resolve_relative_coordinate(x as isize + dx);
                let (chunk_offset_y, dirty_y) = resolve_relative_coordinate(y as isize + dy);

                if chunk_offset_x == 0 && chunk_offset_y == 0 {
                    self.set_dirty_at(dirty_x, dirty_y, true);
                }
                else if let Some(chunk) = chunk_map.get(&Vector([
                    self.location.x() + chunk_offset_x,
                    self.location.y() + chunk_offset_y,
                ])) {
                    chunk.borrow_mut().set_dirty_at(dirty_x, dirty_y, true);
                }
            }
        }
    }

    pub fn is_dirty_at(&self, x: usize, y: usize) -> bool {
        self.blocks_dirty[y][x]
    }

    pub fn set_dirty_at(&mut self, x: usize, y: usize, dirty: bool) {
        self.blocks_dirty[y][x] = dirty;
        self.dirty = self.dirty || dirty;
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn set_dirty(&mut self) {
        self.dirty = true;
        // Force a re-render for all blocks in the chunk
        for row in &mut self.blocks_dirty {
            row.fill(true);
        }
    }

    pub fn update(&mut self, dt: f32) {
        let _ = dt;
    }

    pub fn render(&mut self, dt: f32, assets: &AssetPool, chunk_map: &ChunkMap) {
        let _ = dt;
        if self.geometry.is_empty() {
            let mut vertices = Vec::new();
            let mut faces = Vec::new();
            for block_y in 0..CHUNK_SIZE {
                for block_x in 0..CHUNK_SIZE {
                    for offset in BLOCK_QUAD_OFFSETS {
                        let index = vertices.len() as u32;
                        faces.push([index + 0, index + 1, index + 2]);
                        faces.push([index + 2, index + 3, index + 0]);
                        // TODO: obviously lossy
                        let x = self.location.x() as f32 * CHUNK_SIZE_F32 + block_x as f32 + offset.x();
                        let y = self.location.y() as f32 * CHUNK_SIZE_F32 + block_y as f32 + offset.y();
                        for vertex_offset in BLOCK_QUAD_VERTEX_OFFSETS {
                            vertices.push(Vertex2D::new(
                                [x + vertex_offset.x(), y + vertex_offset.y(), 0.0],
                                Some([0.0, 0.0, 0.0, 0.0]),
                                None,
                            ));
                        }
                    }
                }
            }
            self.geometry.add(&vertices, &faces);
        }

        if self.is_dirty() {
            for y in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    if self.is_dirty_at(x, y) {
                        self.update_block_vertices(x, y, assets, chunk_map);
                        self.blocks_dirty[y][x] = false;
                    }
                }
            }
            self.geometry.update_vertex_buffer();
            self.dirty = false;
        }

        self.geometry.render();
    }

    fn update_block_vertices(&mut self, x: usize, y: usize, assets: &AssetPool, chunk_map: &ChunkMap) {
        // (y * CHUNK_SIZE + x) blocks in, 4 quads per block, 4 vertices per quad
        let first_index = (y * CHUNK_SIZE + x) * VERTICES_PER_BLOCK;

        if let Some(appearance) = assets.get_block_appearance(self.block_at(x, y).block_type) {
            let mut index = first_index;
            let uv_offsets = appearance.get_quad_uv_offsets(chunk_map, self, x, y);
            for (quad_offset, uv_offset) in std::iter::zip(BLOCK_QUAD_OFFSETS, uv_offsets) {
                for vertex_offset in BLOCK_QUAD_VERTEX_OFFSETS {
                    let mut vertex = self.geometry.get_vertex(index);
                    vertex.color = [1.0; 4];
                    vertex.tex = true;
                    let pos = quad_offset + vertex_offset;
                    vertex.uv = [
                        uv_offset.x() as f32 + pos.x() * appearance.resolution as f32,
                        uv_offset.y() as f32 + (1.0 - pos.y()) * appearance.resolution as f32,
                    ];
                    self.geometry.set_vertex(index, &vertex);
                    index += 1;
                }
            }
        }
        else {
            // Make block invisible since it has no appearance (e.g. air)
            let mut index = first_index;
            for _ in 0..VERTICES_PER_BLOCK {
                let mut vertex = self.geometry.get_vertex(index);
                vertex.color = [0.0; 4];
                vertex.tex = false;
                vertex.uv = [0.0; 2];
                self.geometry.set_vertex(index, &vertex);
                index += 1;
            }
        }
    }

    pub fn detach(self, physics: &mut phys::Physics) {
        for row in self.block_colliders {
            for colliders in row {
                for collider in colliders {
                    physics.remove_collider(collider);
                }
            }
        }
    }
}
