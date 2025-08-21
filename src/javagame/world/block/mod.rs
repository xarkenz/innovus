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
        Self::new(&types::AIR, 0, 0)
    }
}

pub const CHUNK_SIZE: usize = 16;
const CHUNK_SIZE_F32: f32 = CHUNK_SIZE as f32;

fn resolve_relative_coordinate(value: isize) -> (i64, usize) {
    (value.div_euclid(CHUNK_SIZE as isize) as i64, value.rem_euclid(CHUNK_SIZE as isize) as usize)
}

pub type ChunkLocation = Vector<i64, 2>;
pub type ChunkMap = BTreeMap<ChunkLocation, RefCell<Chunk>>;

const ADJACENT_OFFSETS: [(isize, isize); 4] = [(0, 1), (0, -1), (-1, 0), (1, 0)];

#[derive(Debug)]
pub struct Chunk {
    location: ChunkLocation,
    blocks: [[Block; CHUNK_SIZE]; CHUNK_SIZE],
    collision_map: Option<[[Box<[phys::ColliderHandle]>; CHUNK_SIZE]; CHUNK_SIZE]>,
    blocks_dirty: [[bool; CHUNK_SIZE]; CHUNK_SIZE],
    all_dirty: bool,
    geometry: Geometry<Vertex2D>,
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
        }
    }

    pub fn location(&self) -> ChunkLocation {
        self.location
    }

    pub fn block_at(&self, x: usize, y: usize) -> &Block {
        &self.blocks[y][x]
    }

    pub fn set_block_at(&mut self, x: usize, y: usize, block: Block, chunk_map: &ChunkMap, physics: &mut phys::Physics) {
        if let Some(collision_map) = &mut self.collision_map {
            // Add new physics colliders and remove the old ones
            let new_colliders = Self::create_block_colliders(self.location, x, y, block.block_type, physics);
            let old_colliders = std::mem::replace(&mut collision_map[y][x], new_colliders);
            for handle in old_colliders {
                physics.remove_collider(handle);
            }
        }

        self.blocks[y][x] = block;
        self.set_dirty_at(x, y, true);
        // Propagate the dirty flag to the surrounding blocks in order to update their appearances
        self.propagate_dirty(x, y, chunk_map);
        // Update lighting around the block
        self.update_light(x as isize, y as isize, chunk_map);
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

    pub fn update_light(&mut self, start_x: isize, start_y: isize, chunk_map: &ChunkMap) {
        let mut stack = Vec::new();
        stack.push((start_x, start_y, true, true));

        let mut compute_count = 0_usize;
        let mut update_count = 0_usize;
        let mut unique_update = std::collections::BTreeSet::new();
        while let Some((x, y, check_block_light, check_sky_light)) = stack.pop() {
            let (chunk_offset_x, block_x) = resolve_relative_coordinate(x);
            let (chunk_offset_y, block_y) = resolve_relative_coordinate(y);

            let current_block_light;
            let current_sky_light;
            let block_type;
            if chunk_offset_x == 0 && chunk_offset_y == 0 {
                let block = self.block_at(block_x, block_y);
                current_block_light = block.block_light;
                current_sky_light = block.sky_light;
                block_type = block.block_type;
            }
            else {
                let other_chunk_location = self.relative_chunk_location(chunk_offset_x, chunk_offset_y);
                let Some(chunk) = chunk_map.get(&other_chunk_location).map(RefCell::borrow) else {
                    continue;
                };
                let block = chunk.block_at(block_x, block_y);
                current_block_light = block.block_light;
                current_sky_light = block.sky_light;
                block_type = block.block_type;
            }
            compute_count += 1;

            let mut new_block_light = None;
            if check_block_light {
                let max_surrounding_light = ADJACENT_OFFSETS
                    .iter()
                    .filter_map(|&(dx, dy)| {
                        self.with_block(x + dx, y + dy, chunk_map, |block| block.block_light)
                    })
                    .max()
                    .unwrap();
                let expected_light = block_type.light_emission.max(max_surrounding_light.saturating_sub(1));
                new_block_light = (current_block_light != expected_light).then_some(expected_light);
            }

            let mut new_sky_light = None;
            if check_sky_light {
                let mut direct_sky_light = if block_type.is_full_block { 0 } else { 15 };
                let max_surrounding_light = ADJACENT_OFFSETS
                    .iter()
                    .map(|&(dx, dy)| {
                        let sky_light = self.with_block(x + dx, y + dy, chunk_map, |block| block.sky_light)
                            .unwrap_or(if dy > 0 { 15 } else { 0 });
                        if dy > 0 && sky_light < 15 {
                            direct_sky_light = 0;
                        }
                        sky_light
                    })
                    .max()
                    .unwrap();
                let expected_light = direct_sky_light.max(max_surrounding_light.saturating_sub(1));
                new_sky_light = (current_sky_light != expected_light).then_some(expected_light);
            }

            if new_block_light.is_none() && new_sky_light.is_none() {
                continue;
            }
            update_count += 1;
            unique_update.insert((x, y));

            if chunk_offset_x == 0 && chunk_offset_y == 0 {
                if let Some(new_block_light) = new_block_light {
                    self.blocks[block_y][block_x].block_light = new_block_light;
                }
                if let Some(new_sky_light) = new_sky_light {
                    self.blocks[block_y][block_x].sky_light = new_sky_light;
                }
                self.set_all_dirty(true);
            }
            else {
                let other_chunk_location = self.relative_chunk_location(chunk_offset_x, chunk_offset_y);
                let mut chunk = chunk_map[&other_chunk_location].borrow_mut();
                if let Some(new_block_light) = new_block_light {
                    chunk.blocks[block_y][block_x].block_light = new_block_light;
                }
                if let Some(new_sky_light) = new_sky_light {
                    chunk.blocks[block_y][block_x].sky_light = new_sky_light;
                }
                chunk.set_all_dirty(true);
            }

            // Add adjacent blocks to update queue
            stack.extend(ADJACENT_OFFSETS.iter().map(|&(dx, dy)| {
                (x + dx, y + dy, new_block_light.is_some(), new_sky_light.is_some())
            }));
        }
        if compute_count > 100 {
            println!("({}.{start_x}, {}.{start_y}) : {compute_count} computed, {update_count} ({}) updated", self.location.x(), self.location.y(), unique_update.len());
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

        if let Some(appearance) = assets.get_block_appearance(block.block_type) {
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
            let uv_offsets = appearance.get_quad_uv_offsets(chunk_map, self, x, y);
            let quadrant_info = std::iter::zip(BLOCK_QUAD_OFFSETS, uv_offsets).zip(quadrant_vertex_lights);
            for ((quad_offset, uv_offset), vertex_lights) in quadrant_info {
                let vertex_info = std::iter::zip(BLOCK_QUAD_VERTEX_OFFSETS, vertex_lights);
                for (vertex_offset, vertex_light) in vertex_info {
                    let mut vertex = self.geometry.get_vertex(index);
                    vertex.color = [vertex_light, vertex_light, vertex_light, 1.0];
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
            chunk_location.x() as f32 * CHUNK_SIZE_F32 + x as f32,
            chunk_location.y() as f32 * CHUNK_SIZE_F32 + y as f32,
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
