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
    Enum {
        default_value: u8,
        value_names: &'static [&'static str],
    },
}

impl AttributeType {
    pub fn description(&self) -> String {
        match *self {
            Self::Bool(..) => "a boolean".into(),
            Self::U8(..) => "a small unsigned integer".into(),
            Self::I8(..) => "a small signed integer".into(),
            Self::U32(..) => "an unsigned integer".into(),
            Self::I32(..) => "a signed integer".into(),
            Self::String(..) => "a string".into(),
            Self::Enum { value_names, .. } => {
                format!("a string matching one of: '{}'", value_names.join("', '"))
            }
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
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
    pub connector: fn(&Block, &Block) -> bool,
}

impl BlockType {
    pub fn get_attribute_info(&self, name: &str) -> Option<(usize, &AttributeType)> {
        self.attributes
            .iter()
            .enumerate()
            .find_map(|(index, &(attribute_name, ref attribute_type))| {
                (name == attribute_name).then_some((index, attribute_type))
            })
    }

    pub fn default_attributes(&self) -> Box<[AttributeValue]> {
        self.attributes
            .iter()
            .map(|(_, t)| match t {
                &AttributeType::Bool(b) => AttributeValue::Bool(b),
                &AttributeType::U8(n) => AttributeValue::U8(n),
                &AttributeType::I8(n) => AttributeValue::I8(n),
                &AttributeType::U32(n) => AttributeValue::U32(n),
                &AttributeType::I32(n) => AttributeValue::I32(n),
                &AttributeType::String(s) => AttributeValue::String(s.to_string()),
                &AttributeType::Enum { default_value, .. } => AttributeValue::U8(default_value),
            })
            .collect()
    }
}

impl PartialEq for BlockType {
    fn eq(&self, other: &Self) -> bool {
        // Comparing pointers is sufficient; only the static BlockType objects should be used.
        self as *const Self == other as *const Self
    }
}

impl std::fmt::Debug for BlockType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "BlockType({})", self.name)
    }
}

pub const QUADRANT_OFFSETS: [Vector2f; 4] = [
    Vector([0.0, 0.5]), // Top left
    Vector([0.5, 0.5]), // Top right
    Vector([0.0, 0.0]), // Bottom left
    Vector([0.5, 0.0]), // Bottom right
];
pub const QUADRANT_VERTEX_OFFSETS: [Vector2f; 4] = [
    Vector([0.0, 0.0]), // Bottom left
    Vector([0.0, 0.5]), // Top left
    Vector([0.5, 0.5]), // Top right
    Vector([0.5, 0.0]), // Bottom right
];
pub const VERTICES_PER_BLOCK: usize = QUADRANT_OFFSETS.len() * QUADRANT_VERTEX_OFFSETS.len();

#[derive(Clone, Debug)]
pub struct Block {
    pub block_type: &'static BlockType,
    pub attributes: Box<[AttributeValue]>,
    pub block_light: u8,
    pub sky_light: u8,
}

impl Block {
    pub fn new(block_type: &'static BlockType) -> Self {
        Self {
            block_type,
            attributes: block_type.default_attributes(),
            block_light: 0,
            sky_light: 0,
        }
    }

    pub fn inherit_environment(&mut self, predecessor: &Self) {
        self.block_light = predecessor.block_light;
        self.sky_light = predecessor.sky_light;
    }

    pub fn connects_to(&self, other: &Self) -> bool {
        (self.block_type.connector)(self, other)
    }
}

impl Default for Block {
    fn default() -> Self {
        Self::new(&types::AIR)
    }
}

pub const CHUNK_SIZE: usize = 16;

fn resolve_relative_coordinate(value: isize) -> (i64, usize) {
    (value.div_euclid(CHUNK_SIZE as isize) as i64, value.rem_euclid(CHUNK_SIZE as isize) as usize)
}

pub type ChunkLocation = Vector<i64, 2>;
pub type ChunkMap = BTreeMap<ChunkLocation, RefCell<Chunk>>;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct BlockCoord {
    pub chunk: i64,
    pub offset: usize,
}

impl BlockCoord {
    pub fn new(chunk: i64, offset: usize) -> Self {
        Self { chunk, offset }
    }
}

impl From<i64> for BlockCoord {
    fn from(value: i64) -> Self {
        Self {
            chunk: value.div_euclid(CHUNK_SIZE as i64),
            offset: value.rem_euclid(CHUNK_SIZE as i64) as usize,
        }
    }
}

impl From<BlockCoord> for i64 {
    fn from(value: BlockCoord) -> Self {
        value.chunk * CHUNK_SIZE as i64 + value.offset as i64
    }
}

const ADJACENT_OFFSETS: [(isize, isize); 4] = [(0, 1), (0, -1), (-1, 0), (1, 0)];

#[derive(Debug)]
pub struct Chunk {
    location: ChunkLocation,
    blocks: [[Block; CHUNK_SIZE]; CHUNK_SIZE],
    collision_map: Option<[[Box<[phys::ColliderHandle]>; CHUNK_SIZE]; CHUNK_SIZE]>,
    blocks_dirty: [[bool; CHUNK_SIZE]; CHUNK_SIZE],
    all_dirty: bool,
    geometry: Geometry<Vertex2D>,
    height_map: [i64; CHUNK_SIZE],
}

impl Chunk {
    pub fn new(location: ChunkLocation) -> Self {
        Self {
            location,
            blocks: Default::default(),
            collision_map: None,
            blocks_dirty: [[true; CHUNK_SIZE]; CHUNK_SIZE],
            all_dirty: true,
            geometry: Geometry::new_render().unwrap(),
            height_map: Default::default(),
        }
    }

    pub fn location(&self) -> ChunkLocation {
        self.location
    }

    pub fn height_map(&self) -> &[i64; CHUNK_SIZE] {
        &self.height_map
    }

    pub fn set_height_map(&mut self, height_map: [i64; CHUNK_SIZE]) {
        self.height_map = height_map;

        // Update sky light
        for (x, terrain_height) in (0..CHUNK_SIZE).zip(height_map) {
            let base_y = self.location.y() * CHUNK_SIZE as i64;
            for y in 0..CHUNK_SIZE {
                let blocks_to_surface = terrain_height - (base_y + y as i64);
                let sky_light = 15 - blocks_to_surface.clamp(0, 15) as u8;
                self.blocks[y][x].sky_light = sky_light;
            }
        }
    }

    pub fn block_at(&self, x: usize, y: usize) -> &Block {
        &self.blocks[y][x]
    }

    pub fn set_block_at(&mut self, x: usize, y: usize, mut block: Block, chunk_map: &ChunkMap, physics: &mut phys::Physics) {
        if let Some(collision_map) = &mut self.collision_map {
            // Add new physics colliders and remove the old ones
            let new_colliders = Self::create_block_colliders(self.location, x, y, block.block_type, physics);
            let old_colliders = std::mem::replace(&mut collision_map[y][x], new_colliders);
            for handle in old_colliders {
                physics.remove_collider(handle);
            }
        }

        block.inherit_environment(&self.blocks[y][x]);
        self.blocks[y][x] = block;
        self.set_dirty_at(x, y, true);
        // Propagate the dirty flag to the surrounding blocks in order to update their appearances
        self.propagate_dirty(x, y, chunk_map);
        // Update lighting around the block
        self.update_lighting(x as isize, y as isize, chunk_map);
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
            let other_chunk_location = self.relative_chunk_location(chunk_offset_x, chunk_offset_y);
            Some(f(chunk_map.get(&other_chunk_location)?.borrow().block_at(block_x, block_y)))
        }
    }

    pub fn relative_chunk_location(&self, chunk_offset_x: i64, chunk_offset_y: i64) -> ChunkLocation {
        Vector([
            self.location.x() + chunk_offset_x,
            self.location.y() + chunk_offset_y,
        ])
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
                else if let Some(chunk) = chunk_map.get(&self.relative_chunk_location(chunk_offset_x, chunk_offset_y)) {
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
    }

    pub fn is_all_dirty(&self) -> bool {
        self.all_dirty
    }

    pub fn set_all_dirty(&mut self, dirty: bool) {
        self.all_dirty = dirty;
    }

    pub fn update_lighting(&mut self, start_x: isize, start_y: isize, chunk_map: &ChunkMap) {
        let mut stack = Vec::new();
        stack.push((start_x, start_y));

        while let Some((x, y)) = stack.pop() {
            let (chunk_offset_x, block_x) = resolve_relative_coordinate(x);
            let (chunk_offset_y, block_y) = resolve_relative_coordinate(y);

            let current_light;
            let block_type;
            if chunk_offset_x == 0 && chunk_offset_y == 0 {
                let block = self.block_at(block_x, block_y);
                current_light = block.block_light;
                block_type = block.block_type;
            }
            else {
                let other_chunk_location = self.relative_chunk_location(chunk_offset_x, chunk_offset_y);
                let Some(chunk) = chunk_map.get(&other_chunk_location).map(RefCell::borrow) else {
                    continue;
                };
                let block = chunk.block_at(block_x, block_y);
                current_light = block.block_light;
                block_type = block.block_type;
            }

            let max_surrounding_light = ADJACENT_OFFSETS
                .iter()
                .filter_map(|&(dx, dy)| {
                    self.with_block(x + dx, y + dy, chunk_map, |block| block.block_light)
                })
                .max()
                .unwrap();
            let expected_light = block_type.light_emission.max(max_surrounding_light.saturating_sub(1));

            if current_light == expected_light {
                continue;
            }

            if chunk_offset_x == 0 && chunk_offset_y == 0 {
                self.blocks[block_y][block_x].block_light = expected_light;
                self.set_all_dirty(true);
            }
            else {
                let other_chunk_location = self.relative_chunk_location(chunk_offset_x, chunk_offset_y);
                let mut chunk = chunk_map[&other_chunk_location].borrow_mut();
                chunk.blocks[block_y][block_x].block_light = expected_light;
                chunk.set_all_dirty(true);
            }

            // Add adjacent blocks to update stack
            stack.extend(ADJACENT_OFFSETS.map(|(dx, dy)| (x + dx, y + dy)));
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
                    for offset in QUADRANT_OFFSETS {
                        let index = vertices.len() as u32;
                        faces.push([index + 0, index + 1, index + 2]);
                        faces.push([index + 2, index + 3, index + 0]);
                        // TODO: obviously lossy
                        let x = self.location.x() as f32 * CHUNK_SIZE as f32 + block_x as f32 + offset.x();
                        let y = self.location.y() as f32 * CHUNK_SIZE as f32 + block_y as f32 + offset.y();
                        for vertex_offset in QUADRANT_VERTEX_OFFSETS {
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

        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                if self.is_all_dirty() || self.is_dirty_at(x, y) {
                    self.update_block_vertices(x, y, assets, chunk_map);
                    self.set_dirty_at(x, y, false);
                }
            }
        }
        self.geometry.update_vertex_buffer();
        self.set_all_dirty(false);

        self.geometry.render();
    }

    fn update_block_vertices(&mut self, x: usize, y: usize, assets: &AssetPool, chunk_map: &ChunkMap) {
        fn get_light_value(block: &Block) -> f32 {
            const AMBIENT_LIGHT: f32 = 3.0;
            let effective_light = block.block_light.max(block.sky_light);
            (AMBIENT_LIGHT + effective_light as f32) / (AMBIENT_LIGHT + 15.0)
        }

        let block = &self.blocks[y][x];
        // (y * CHUNK_SIZE + x) blocks in, 4 quads per block, 4 vertices per quad
        let first_index = (y * CHUNK_SIZE + x) * VERTICES_PER_BLOCK;

        if let Some(image) = assets.get_block_image(block, self.location(), x, y) {
            let quadrant_vertex_lights = {
                let x = x as isize;
                let y = y as isize;
                // u = up, d = down, l = left, r = right, c = center (all relative to current block)
                let block_light_ul = self.with_block(x - 1, y + 1, chunk_map, get_light_value).unwrap_or(1.0);
                let block_light_uc = self.with_block(x + 0, y + 1, chunk_map, get_light_value).unwrap_or(1.0);
                let block_light_ur = self.with_block(x + 1, y + 1, chunk_map, get_light_value).unwrap_or(1.0);
                let block_light_cl = self.with_block(x - 1, y + 0, chunk_map, get_light_value).unwrap_or(1.0);
                let block_light_cc = get_light_value(block); // Might as well use what we have
                let block_light_cr = self.with_block(x + 1, y + 0, chunk_map, get_light_value).unwrap_or(1.0);
                let block_light_dl = self.with_block(x - 1, y - 1, chunk_map, get_light_value).unwrap_or(1.0);
                let block_light_dc = self.with_block(x + 0, y - 1, chunk_map, get_light_value).unwrap_or(1.0);
                let block_light_dr = self.with_block(x + 1, y - 1, chunk_map, get_light_value).unwrap_or(1.0);

                let corner_light_ul = (block_light_ul + block_light_uc + block_light_cl + block_light_cc) / 4.0;
                let corner_light_ur = (block_light_ur + block_light_uc + block_light_cr + block_light_cc) / 4.0;
                let corner_light_dl = (block_light_dl + block_light_dc + block_light_cl + block_light_cc) / 4.0;
                let corner_light_dr = (block_light_dr + block_light_dc + block_light_cr + block_light_cc) / 4.0;

                let edge_light_u = (corner_light_ul + corner_light_ur) / 2.0;
                let edge_light_d = (corner_light_dl + corner_light_dr) / 2.0;
                let edge_light_l = (corner_light_ul + corner_light_dl) / 2.0;
                let edge_light_r = (corner_light_ur + corner_light_dr) / 2.0;

                // Outer array (block quadrant): up left, up right, down left, down right
                // Inner array (quadrant vertices): down left, up left, up right, down right
                [
                    [edge_light_l, corner_light_ul, edge_light_u, block_light_cc],
                    [block_light_cc, edge_light_u, corner_light_ur, edge_light_r],
                    [corner_light_dl, edge_light_l, block_light_cc, edge_light_d],
                    [edge_light_d, block_light_cc, edge_light_r, corner_light_dr],
                ]
            };

            let mut index = first_index;
            let atlas_offsets = image.get_quadrant_atlas_offsets(chunk_map, self, x, y);
            let quadrant_info = std::iter::zip(QUADRANT_OFFSETS, atlas_offsets).zip(quadrant_vertex_lights);
            for ((quadrant_offset, atlas_offset), vertex_lights) in quadrant_info {
                let vertex_info = std::iter::zip(QUADRANT_VERTEX_OFFSETS, vertex_lights);
                for (vertex_offset, vertex_light) in vertex_info {
                    let mut vertex = self.geometry.get_vertex(index);
                    vertex.color = [vertex_light, vertex_light, vertex_light, 1.0];
                    vertex.tex = true;
                    let position = quadrant_offset + vertex_offset;
                    vertex.uv = [
                        atlas_offset.x() as f32 + position.x() * image.size() as f32,
                        atlas_offset.y() as f32 + (1.0 - position.y()) * image.size() as f32,
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

    pub fn attach_physics(&mut self, physics: &mut phys::Physics) {
        if self.collision_map.is_none() {
            let mut y = 0;
            let collision_map = self.blocks.each_ref().map(|row| {
                let mut x = 0;
                let row = row.each_ref().map(|block| {
                    let colliders = Self::create_block_colliders(self.location, x, y, block.block_type, physics);
                    x += 1;
                    colliders
                });
                y += 1;
                row
            });
            self.collision_map = Some(collision_map);
        }
    }

    pub fn detach_physics(&mut self, physics: &mut phys::Physics) {
        if let Some(collision_map) = self.collision_map.take() {
            for row in collision_map {
                for colliders in row {
                    for collider in colliders {
                        physics.remove_collider(collider);
                    }
                }
            }
        }
    }

    fn create_block_colliders(chunk_location: ChunkLocation, x: usize, y: usize, block_type: &'static BlockType, physics: &mut phys::Physics) -> Box<[phys::ColliderHandle]> {
        let block_origin = Vector([
            chunk_location.x() as f32 * CHUNK_SIZE as f32 + x as f32,
            chunk_location.y() as f32 * CHUNK_SIZE as f32 + y as f32,
        ]);
        block_type.colliders
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
            .collect()
    }
}
