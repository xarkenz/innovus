use std::collections::HashMap;
use innovus::{gfx::*, tools::*};
use std::fmt::Formatter;
use num::Integer;
use crate::world::World;

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

    fn get_block_type(world: &World, chunk: &Chunk, x: isize, y: isize) -> &'static BlockType {
        let (chunk_offset_x, x) = Self::resolve_relative_coordinate(x);
        let (chunk_offset_y, y) = Self::resolve_relative_coordinate(y);

        if chunk_offset_x == 0 && chunk_offset_y == 0 {
            chunk.block_at(x, y).block_type
        }
        else {
            match world.get_chunk(Vector([
                chunk.location().x() + chunk_offset_x,
                chunk.location().y() + chunk_offset_y,
            ])) {
                Some(chunk) => chunk.block_at(x, y).block_type,
                None => &types::AIR,
            }
        }
    }

    fn get_connect(&self, world: &World, chunk: &Chunk, x: isize, y: isize) -> bool {
        self.block_type.connects_to(Self::get_block_type(world, chunk, x, y))
    }

    fn get_quad_type(&self, world: &World, chunk: &Chunk, x: isize, y: isize, x_connect: bool, y_connect: bool) -> u32 {
        match (x_connect, y_connect) {
            (false, false) => 0,
            (true, false) => 1,
            (false, true) => 2,
            (true, true) => if self.get_connect(world, chunk, x, y) { 4 } else { 3 },
        }
    }

    pub fn get_quad_uv_offsets(&self, world: &World, chunk: &Chunk, x: usize, y: usize) -> [Vector<u32, 2>; 4] {
        let left_x = x as isize - 1;
        let right_x = x as isize + 1;
        let down_y = y as isize - 1;
        let up_y = y as isize + 1;

        let left_connect = self.get_connect(world, chunk, left_x, y as isize);
        let right_connect = self.get_connect(world, chunk, right_x, y as isize);
        let down_connect = self.get_connect(world, chunk, x as isize, down_y);
        let up_connect = self.get_connect(world, chunk, x as isize, up_y);

        let quad_types: [u32; 4] = [
            self.get_quad_type(world, chunk, left_x, up_y, left_connect, up_connect),
            self.get_quad_type(world, chunk, right_x, up_y, right_connect, up_connect),
            self.get_quad_type(world, chunk, left_x, down_y, left_connect, down_connect),
            self.get_quad_type(world, chunk, right_x, down_y, right_connect, down_connect),
        ];

        quad_types.map(|quad_type| Vector([
            self.offset.x() + quad_type * self.resolution,
            self.offset.y(),
        ]))
    }
}

pub struct BlockRenderer {
    atlas: Image,
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
                    block_type,
                    offset,
                    resolution: 16,
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

    pub fn atlas(&self) -> &Image {
        &self.atlas
    }

    pub fn texture(&self) -> &Texture2D {
        &self.texture
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
        self.set_dirty_at(x, y, true);
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

    pub fn render(&mut self, dt: f32, renderer: &BlockRenderer, world: &World) {
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
                        self.update_block_vertices(x, y, renderer, world);
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

    fn update_block_vertices(&mut self, x: usize, y: usize, renderer: &BlockRenderer, world: &World) {
        // (y * CHUNK_SIZE + x) blocks in, 4 quads per block, 4 vertices per quad
        let first_index = (y * CHUNK_SIZE + x) * VERTICES_PER_BLOCK;
        if let Some(appearance) = renderer.get_appearance(self.block_at(x, y).block_type) {
            let mut index = first_index;
            let uv_offsets = appearance.get_quad_uv_offsets(world, self, x, y);
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
