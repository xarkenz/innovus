use std::collections::HashMap;
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

impl std::fmt::Debug for BlockType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "BlockType({})", self.name)
    }
}

pub const BLOCK_QUAD_OFFSETS: [Vector<f32, 2>; 4] = [
    Vector([0.0, 0.5]), // Top left
    Vector([0.5, 0.5]), // Top right
    Vector([0.0, 0.0]), // Bottom left
    Vector([0.5, 0.0]), // Bottom right
];
pub const BLOCK_QUAD_VERTEX_OFFSETS: [Vector<f32, 2>; 4] = [
    Vector([0.0, 0.0]), // Bottom left
    Vector([0.0, 0.5]), // Top left
    Vector([0.5, 0.5]), // Top right
    Vector([0.5, 0.0]), // Bottom right
];
pub const VERTICES_PER_BLOCK: usize = BLOCK_QUAD_OFFSETS.len() * BLOCK_QUAD_VERTEX_OFFSETS.len();

#[derive(Debug)]
pub struct BlockAppearance {
    pub offset: Vector<u32, 2>,
}

pub struct BlockRenderer {
    pub atlas: Image,
    texture: Texture2D,
    appearances: HashMap<*const BlockType, BlockAppearance>,
}

impl BlockRenderer {
    pub fn new() -> Result<Self, String> {
        let (images, block_types): (Vec<Image>, Vec<&BlockType>) = types::BLOCK_TYPES.iter()
            .filter_map(|&block_type| {
                let path = format!("src/javagame/assets/textures/block/{}.png", block_type.name);
                Image::from_file(&path).ok().map(|image| (image, block_type))
            })
            .unzip();

        let (atlas, offsets) = Image::new_atlas(&images);

        let appearances = HashMap::from_iter(block_types.iter()
            .zip(offsets)
            .map(|(&block_type, offset)| {
                let appearance = BlockAppearance {
                    offset,
                };
                (block_type as *const BlockType, appearance)
            }));

        let mut texture = Texture2D::new(0);
        texture.set_minify_filter(TextureSampling::Nearest);
        texture.set_magnify_filter(TextureSampling::Nearest);
        texture.set_wrap_s(TextureWrap::Repeat);
        texture.set_wrap_t(TextureWrap::Repeat);
        texture.load_from_image(&atlas);

        Ok(Self {
            atlas,
            texture,
            appearances,
        })
    }

    pub fn get_appearance(&self, block_type: &'static BlockType) -> Option<&BlockAppearance> {
        self.appearances.get(&(block_type as *const BlockType))
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

pub type ChunkLocation = Vector<i64, 2>;

#[derive(Clone, Debug)]
pub struct Chunk {
    location: ChunkLocation,
    blocks: [[Block; CHUNK_SIZE]; CHUNK_SIZE],
    blocks_dirty: [[bool; CHUNK_SIZE]; CHUNK_SIZE],
    dirty: bool,
    geometry: Geometry<Vertex2D>,
}

impl Chunk {
    pub fn new(location: ChunkLocation) -> Self {
        Self {
            location,
            blocks: Default::default(),
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

    pub fn set_block_at(&mut self, x: usize, y: usize, block: Block) {
        self.blocks[y][x] = block;
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

    pub fn set_dirty(&mut self, dirty: bool) {
        self.dirty = dirty;
    }

    pub fn update(&mut self, dt: f32) {
        // TODO
    }

    pub fn render(&mut self, dt: f32, renderer: &BlockRenderer) {
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
            let mut dirty_count: usize = 0;
            for y in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    if self.is_dirty_at(x, y) {
                        self.update_block_vertices(x, y, renderer);
                        self.set_dirty_at(x, y, false);
                        dirty_count += 1;
                    }
                }
            }
            self.geometry.update_vertex_buffer();
            self.set_dirty(false);
        }

        self.geometry.render();
    }

    fn update_block_vertices(&mut self, x: usize, y: usize, renderer: &BlockRenderer) {
        // (y * CHUNK_SIZE + x) blocks in, 4 quads per block, 4 vertices per quad
        let first_index = (y * CHUNK_SIZE + x) * VERTICES_PER_BLOCK;
        if let Some(appearance) = renderer.get_appearance(self.block_at(x, y).block_type) {
            let mut index = first_index;
            for quad_offset in BLOCK_QUAD_OFFSETS {
                for vertex_offset in BLOCK_QUAD_VERTEX_OFFSETS {
                    let mut vertex = self.geometry.get_vertex(index);
                    vertex.color = [1.0; 4];
                    vertex.tex = true;
                    let pos = quad_offset + vertex_offset;
                    vertex.uv = [
                        appearance.offset.x() as f32 + pos.x() * 16.0,
                        appearance.offset.y() as f32 + (1.0 - pos.y()) * 16.0,
                    ];
                    self.geometry.set_vertex(index, &vertex);
                    index += 1;
                }
            }
        }
        else {
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
}
