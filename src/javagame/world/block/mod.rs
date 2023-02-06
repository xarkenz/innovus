use innovus::{gfx::*, tools::*};
use std::fmt::Formatter;

pub mod types;

#[derive(Debug)]
pub enum AttributeType {
    Bool(bool),
    U8(u8),
    I8(i8),
    U32(u32),
    I32(i32),
    String(&'static str),
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
    pub full_block: bool,
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

impl std::fmt::Debug for BlockType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "BlockType({})", self.name)
    }
}

#[derive(Clone, Debug)]
pub struct Block {
    id: &'static BlockType,
    attributes: Vec<AttributeValue>,
    block_light: u8,
    sky_light: u8,
}

impl Block {
    pub fn new(id: &'static BlockType, block_light: u8, sky_light: u8) -> Self {
        Block {
            id,
            attributes: id.default_attributes(),
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

pub type ChunkLocation = Vector<i64, 2>;

#[derive(Clone, Debug)]
pub struct Chunk {
    location: ChunkLocation,
    blocks: [[Block; CHUNK_SIZE]; CHUNK_SIZE],
    blocks_dirty: [[bool; CHUNK_SIZE]; CHUNK_SIZE],
    geometry: Geometry<Vertex2D>,
}

impl Chunk {
    pub fn new(location: ChunkLocation) -> Self {
        Chunk {
            location,
            blocks: Default::default(),
            blocks_dirty: [[true; CHUNK_SIZE]; CHUNK_SIZE],
            geometry: Geometry::new_render().unwrap(),
        }
    }

    pub fn location(&self) -> ChunkLocation {
        self.location
    }

    pub fn block_at(&self, x: usize, y: usize) -> &Block {
        &self.blocks[y][x]
    }

    pub fn set_block_at(&mut self, x: usize, y: usize, block: Block) {
        self.blocks[y][x] = block;
    }

    pub fn dirty_at(&self, x: usize, y: usize) -> bool {
        self.blocks_dirty[y][x]
    }

    pub fn set_dirty_at(&mut self, x: usize, y: usize, dirty: bool) {
        self.blocks_dirty[y][x] = dirty;
    }

    pub fn dirty(&self) -> bool {
        self.blocks_dirty.iter().any(|row| row.iter().any(|b| *b))
    }

    pub fn set_dirty(&mut self, dirty: bool) {
        for row in &mut self.blocks_dirty {
            row.fill(dirty);
        }
    }

    pub fn update(&mut self, _dt: f32) {
        if self.geometry.is_empty() {
            let l = self.location.x() as f32 * CHUNK_SIZE_F32;
            let b = self.location.y() as f32 * CHUNK_SIZE_F32;
            let r = l + CHUNK_SIZE_F32;
            let t = b + CHUNK_SIZE_F32;
            self.geometry.add(
                &[
                    Vertex2D::new([l, b, 0.0], Some([1.0, 0.0, 0.0, 1.0]), None),
                    Vertex2D::new([r, t, 0.0], Some([1.0, 0.0, 0.0, 1.0]), None),
                    Vertex2D::new([l, t, 0.0], Some([1.0, 0.0, 0.0, 1.0]), None),
                    Vertex2D::new([l, b, 0.0], Some([0.0, 1.0, 0.0, 1.0]), None),
                    Vertex2D::new([r, b, 0.0], Some([0.0, 1.0, 0.0, 1.0]), None),
                    Vertex2D::new([r, t, 0.0], Some([0.0, 1.0, 0.0, 1.0]), None),
                ],
                &[[0, 1, 2], [3, 4, 5]],
            );
        }
    }

    pub fn render(&self) {
        self.geometry.render();
    }
}
