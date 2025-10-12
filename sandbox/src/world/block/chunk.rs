use std::cell::{Ref, RefCell, RefMut};
use std::collections::BTreeMap;
use innovus::gfx::{MeshRenderer, Vertex2D};
use innovus::tools::{Rectangle, Vector};
use innovus::tools::phys::{Collider, ColliderHandle, Physics};
use crate::tools::asset::AssetPool;
use crate::world::gen::WorldGenerator;
use super::*;

pub const CHUNK_SIZE: usize = 16;

pub fn resolve_relative_coordinate(value: isize) -> (i64, usize) {
    (
        value.div_euclid(CHUNK_SIZE as isize) as i64,
        value.rem_euclid(CHUNK_SIZE as isize) as usize,
    )
}

pub fn light_value(effective_light: u8) -> f32 {
    const AMBIENT_LIGHT: f32 = 3.0;
    const MULTIPLIER: f32 = 1.3;
    ((AMBIENT_LIGHT + effective_light as f32 * MULTIPLIER) / (AMBIENT_LIGHT + 15.0))
        .clamp(0.0, 1.0)
}

pub type ChunkLocation = Vector<i64, 2>;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct BlockCoord {
    pub chunk: i64,
    pub offset: usize,
}

impl BlockCoord {
    pub fn new(chunk: i64, offset: usize) -> Self {
        Self {
            chunk,
            offset,
        }
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

#[derive(Clone, Debug)]
pub struct BlockSlot {
    block: Block,
    block_light: u8,
    sky_light: u8,
    needs_render: bool,
}

impl BlockSlot {
    pub fn block(&self) -> &Block {
        &self.block
    }

    pub fn block_light(&self) -> u8 {
        self.block_light
    }

    pub fn sky_light(&self) -> u8 {
        self.sky_light
    }

    pub fn needs_render(&self) -> bool {
        self.needs_render
    }

    pub fn light_value(&self) -> f32 {
        light_value(self.block_light.max(self.sky_light))
    }
}

impl Default for BlockSlot {
    fn default() -> Self {
        Self {
            block: Default::default(),
            block_light: 0,
            sky_light: 0,
            needs_render: true,
        }
    }
}

const ADJACENT_OFFSETS: [(isize, isize); 4] = [(0, 1), (0, -1), (-1, 0), (1, 0)];

#[derive(Debug)]
pub struct Chunk {
    location: ChunkLocation,
    block_slots: [[BlockSlot; CHUNK_SIZE]; CHUNK_SIZE],
    collision_map: Option<[[Box<[ColliderHandle]>; CHUNK_SIZE]; CHUNK_SIZE]>,
    render_all: bool,
    mesh: MeshRenderer<Vertex2D>,
    height_map: [i64; CHUNK_SIZE],
}

impl Chunk {
    pub fn new(location: ChunkLocation) -> Self {
        Self {
            location,
            block_slots: Default::default(),
            collision_map: None,
            render_all: true,
            mesh: MeshRenderer::create(),
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
                self.block_slots[y][x].sky_light = sky_light;
            }
        }
    }

    pub fn block_slot_at(&self, x: usize, y: usize) -> &BlockSlot {
        &self.block_slots[y][x]
    }

    pub fn block_at(&self, x: usize, y: usize) -> &Block {
        self.block_slots[y][x].block()
    }

    pub fn sky_light_at(&self, x: usize, y: usize) -> u8 {
        self.block_slots[y][x].sky_light()
    }

    pub fn block_light_at(&self, x: usize, y: usize) -> u8 {
        self.block_slots[y][x].block_light()
    }

    pub fn is_dirty_at(&self, x: usize, y: usize) -> bool {
        self.block_slots[y][x].needs_render()
    }

    pub fn set_block_at(&mut self, x: usize, y: usize, block: Block, chunk_map: &ChunkMap, physics: &mut Physics) {
        if let Some(collision_map) = &mut self.collision_map {
            // Add new physics colliders and remove the old ones
            let new_colliders = Self::create_block_colliders(self.location, x, y, &block, physics);
            let old_colliders = std::mem::replace(&mut collision_map[y][x], new_colliders);
            for handle in old_colliders {
                physics.remove_collider(handle);
            }
        }

        self.block_slots[y][x].block = block;
        self.block_slots[y][x].needs_render = true;
        // Propagate the render flag to the surrounding blocks in order to update their appearances
        self.propagate_render_flag(x, y, chunk_map);
        // Update lighting around the block
        self.update_lighting(x as isize, y as isize, chunk_map);
    }

    pub fn with_block_slot<F, T>(&self, x: isize, y: isize, chunk_map: &ChunkMap, f: F) -> Option<T>
    where
        F: FnOnce(&BlockSlot) -> T,
    {
        let (chunk_offset_x, block_x) = resolve_relative_coordinate(x);
        let (chunk_offset_y, block_y) = resolve_relative_coordinate(y);

        if chunk_offset_x == 0 && chunk_offset_y == 0 {
            Some(f(self.block_slot_at(block_x, block_y)))
        }
        else {
            let other_chunk_location = self.relative_chunk_location(chunk_offset_x, chunk_offset_y);
            Some(f(chunk_map.get(other_chunk_location)?.block_slot_at(block_x, block_y)))
        }
    }

    pub fn set_all_need_render(&mut self) {
        self.render_all = true;
    }

    pub fn relative_chunk_location(&self, chunk_offset_x: i64, chunk_offset_y: i64) -> ChunkLocation {
        Vector([
            self.location.x() + chunk_offset_x,
            self.location.y() + chunk_offset_y,
        ])
    }

    fn propagate_render_flag(&mut self, origin_x: usize, origin_y: usize, chunk_map: &ChunkMap) {
        for dy in [-1, 0, 1] {
            for dx in [-1, 0, 1] {
                if dx == 0 && dy == 0 {
                    continue;
                }

                let (chunk_offset_x, block_x) = resolve_relative_coordinate(origin_x as isize + dx);
                let (chunk_offset_y, block_y) = resolve_relative_coordinate(origin_y as isize + dy);

                if chunk_offset_x == 0 && chunk_offset_y == 0 {
                    self.block_slots[block_y][block_x].needs_render = true;
                }
                else if let Some(mut chunk) = chunk_map.get_mut(self.relative_chunk_location(chunk_offset_x, chunk_offset_y)) {
                    chunk.block_slots[block_y][block_x].needs_render = true;
                }
            }
        }
    }

    pub fn update_lighting(&mut self, start_x: isize, start_y: isize, chunk_map: &ChunkMap) {
        let mut stack = Vec::new();
        stack.push((start_x, start_y));

        while let Some((x, y)) = stack.pop() {
            let (chunk_offset_x, block_x) = resolve_relative_coordinate(x);
            let (chunk_offset_y, block_y) = resolve_relative_coordinate(y);

            let current_light;
            let light_emission;
            if chunk_offset_x == 0 && chunk_offset_y == 0 {
                let slot = self.block_slot_at(block_x, block_y);
                current_light = slot.block_light();
                light_emission = slot.block().light_emission();
            }
            else {
                let other_chunk_location = self.relative_chunk_location(chunk_offset_x, chunk_offset_y);
                let Some(chunk) = chunk_map.get(other_chunk_location) else {
                    continue;
                };
                let slot = chunk.block_slot_at(block_x, block_y);
                current_light = slot.block_light();
                light_emission = slot.block().light_emission();
            }

            let max_surrounding_light = ADJACENT_OFFSETS
                .iter()
                .filter_map(|&(dx, dy)| {
                    self.with_block_slot(x + dx, y + dy, chunk_map, BlockSlot::block_light)
                })
                .max()
                .unwrap();
            let expected_light = light_emission.max(max_surrounding_light.saturating_sub(1));

            if chunk_offset_x == 0 && chunk_offset_y == 0 {
                self.block_slots[block_y][block_x].block_light = expected_light;
                self.set_all_need_render();
            }
            else {
                let other_chunk_location = self.relative_chunk_location(chunk_offset_x, chunk_offset_y);
                let mut chunk = chunk_map.get_mut(other_chunk_location).unwrap();
                chunk.block_slots[block_y][block_x].block_light = expected_light;
                chunk.set_all_need_render();
            }

            if current_light != expected_light {
                // Add adjacent blocks to update stack
                stack.extend(ADJACENT_OFFSETS.map(|(dx, dy)| (x + dx, y + dy)));
            }
        }
    }

    pub fn render(&mut self, assets: &AssetPool, chunk_map: &ChunkMap) {
        if self.mesh.is_empty() {
            let mut vertices = Vec::new();
            let mut faces = Vec::new();
            for block_y in 0..CHUNK_SIZE {
                for block_x in 0..CHUNK_SIZE {
                    for offset in QUADRANT_OFFSETS {
                        let index = vertices.len() as u32;
                        faces.push([index + 0, index + 1, index + 2]);
                        faces.push([index + 2, index + 3, index + 0]);
                        // TODO: obviously lossy
                        let vertex_position = Vector([
                            self.location.x() as f32 * CHUNK_SIZE as f32 + block_x as f32 + offset.x(),
                            self.location.y() as f32 * CHUNK_SIZE as f32 + block_y as f32 + offset.y(),
                        ]);
                        for vertex_offset in QUADRANT_VERTEX_OFFSETS {
                            vertices.push(Vertex2D::new(
                                (vertex_position + vertex_offset).with_z(0.0),
                                None,
                                None,
                            ));
                        }
                    }
                }
            }
            self.mesh.add(&vertices, &faces);
        }

        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                if self.render_all || self.block_slot_at(x, y).needs_render() {
                    self.update_block_vertices(x, y, assets, chunk_map);
                    self.block_slots[y][x].needs_render = false;
                }
            }
        }
        self.mesh.upload_vertex_buffer();
        self.render_all = false;

        self.mesh.render();
    }

    fn update_block_vertices(&mut self, x: usize, y: usize, assets: &AssetPool, chunk_map: &ChunkMap) {
        let slot = &self.block_slots[y][x];
        // (y * CHUNK_SIZE + x) blocks in, 4 quads per block, 4 vertices per quad
        let first_index = (y * CHUNK_SIZE + x) * VERTICES_PER_BLOCK;

        if let Some(image) = assets.get_block_image(slot.block(), self.location(), x, y) {
            let quadrant_vertex_lights = {
                let x = x as isize;
                let y = y as isize;
                // u = up, d = down, l = left, r = right, c = center (all relative to current block)
                let get_light = BlockSlot::light_value;
                let block_light_cc = slot.light_value(); // Might as well use what we have
                let block_light_ul = self.with_block_slot(x - 1, y + 1, chunk_map, get_light).unwrap_or(block_light_cc);
                let block_light_uc = self.with_block_slot(x + 0, y + 1, chunk_map, get_light).unwrap_or(block_light_cc);
                let block_light_ur = self.with_block_slot(x + 1, y + 1, chunk_map, get_light).unwrap_or(block_light_cc);
                let block_light_cl = self.with_block_slot(x - 1, y + 0, chunk_map, get_light).unwrap_or(block_light_cc);
                let block_light_cr = self.with_block_slot(x + 1, y + 0, chunk_map, get_light).unwrap_or(block_light_cc);
                let block_light_dl = self.with_block_slot(x - 1, y - 1, chunk_map, get_light).unwrap_or(block_light_cc);
                let block_light_dc = self.with_block_slot(x + 0, y - 1, chunk_map, get_light).unwrap_or(block_light_cc);
                let block_light_dr = self.with_block_slot(x + 1, y - 1, chunk_map, get_light).unwrap_or(block_light_cc);

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
            let atlas_offsets = image.get_quadrant_atlas_offsets(chunk_map, self, slot.block(), x, y);
            let quadrant_info = std::iter::zip(QUADRANT_OFFSETS, atlas_offsets).zip(quadrant_vertex_lights);
            for ((quadrant_offset, atlas_offset), vertex_lights) in quadrant_info {
                let vertex_info = std::iter::zip(QUADRANT_VERTEX_OFFSETS, vertex_lights);
                for (vertex_offset, vertex_light) in vertex_info {
                    let vertex = self.mesh.vertex_at_mut(index);
                    vertex.color = Vector([vertex_light, vertex_light, vertex_light, 1.0]);
                    let total_offset = quadrant_offset + vertex_offset;
                    vertex.uv = Vector([
                        atlas_offset.x() as f32 + total_offset.x() * image.size() as f32,
                        atlas_offset.y() as f32 + (1.0 - total_offset.y()) * image.size() as f32,
                    ]);
                    index += 1;
                }
            }
        }
        else {
            // Make block invisible since it has no appearance (e.g. air)
            let mut index = first_index;
            for _ in 0..VERTICES_PER_BLOCK {
                let vertex = self.mesh.vertex_at_mut(index);
                vertex.color = Vector::zero();
                vertex.uv = Vector::filled(f32::NAN);
                index += 1;
            }
        }
    }

    pub fn attach_physics(&mut self, physics: &mut Physics) {
        if self.collision_map.is_none() {
            let mut y = 0;
            let collision_map = self.block_slots.each_ref().map(|row| {
                let mut x = 0;
                let row = row.each_ref().map(|slot| {
                    let colliders = Self::create_block_colliders(self.location, x, y, slot.block(), physics);
                    x += 1;
                    colliders
                });
                y += 1;
                row
            });
            self.collision_map = Some(collision_map);
        }
    }

    pub fn detach_physics(&mut self, physics: &mut Physics) {
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

    fn create_block_colliders(chunk_location: ChunkLocation, x: usize, y: usize, block: &Block, physics: &mut Physics) -> Box<[ColliderHandle]> {
        let block_origin = Vector([
            chunk_location.x() as f32 * CHUNK_SIZE as f32 + x as f32,
            chunk_location.y() as f32 * CHUNK_SIZE as f32 + y as f32,
        ]);
        block.block_type().colliders
            .iter()
            .map(|&bounds| {
                let mut collider_bounds = Rectangle::new(
                    Vector([
                        bounds.min.x() as f32 / 32.0,
                        bounds.min.y() as f32 / 32.0,
                    ]),
                    Vector([
                        bounds.max.x() as f32 / 32.0,
                        bounds.max.y() as f32 / 32.0,
                    ]),
                );
                collider_bounds.shift_by(block_origin);
                physics.add_collider(Collider::new_fixed(collider_bounds))
            })
            .collect()
    }
}

pub struct ChunkMap {
    generator: Option<Box<dyn WorldGenerator>>,
    chunks: BTreeMap<ChunkLocation, RefCell<Chunk>>,
    chunk_load_range: Rectangle<i64>,
    chunk_simulate_range: Rectangle<i64>,
}

impl ChunkMap {
    pub fn new(generator: Option<Box<dyn WorldGenerator>>) -> Self {
        Self {
            generator,
            chunks: BTreeMap::new(),
            chunk_load_range: Rectangle::new(Vector([-2, -2]), Vector([2, 2])),
            chunk_simulate_range: Rectangle::new(Vector([-1, -1]), Vector([1, 1])),
        }
    }

    pub fn get(&self, location: ChunkLocation) -> Option<Ref<'_, Chunk>> {
        self.chunks.get(&location).map(|chunk| chunk.borrow())
    }

    pub fn get_mut(&self, location: ChunkLocation) -> Option<RefMut<'_, Chunk>> {
        self.chunks.get(&location).map(|chunk| chunk.borrow_mut())
    }

    pub fn get_or_load(&mut self, location: ChunkLocation, physics: &mut Physics) -> Ref<'_, Chunk> {
        self.get_or_load_cell(location, physics).borrow()
    }

    pub fn get_or_load_mut(&mut self, location: ChunkLocation, physics: &mut Physics) -> RefMut<'_, Chunk> {
        self.get_or_load_cell(location, physics).borrow_mut()
    }

    fn get_or_load_cell(&mut self, location: ChunkLocation, physics: &mut Physics) -> &RefCell<Chunk> {
        if self.chunks.contains_key(&location) {
            &self.chunks[&location]
        }
        else {
            self.chunks.insert(location, RefCell::new(Chunk::new(location)));
            let cell = &self.chunks[&location];
            if let Some(generator) = &self.generator {
                generator.generate_chunk(&mut *cell.borrow_mut(), self, physics);
            }
            cell
        }
    }

    pub fn unload(&mut self, location: ChunkLocation, physics: &mut Physics) {
        if let Some(chunk) = self.chunks.remove(&location) {
            chunk.into_inner().detach_physics(physics);
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = Ref<'_, Chunk>> {
        self.chunks.values().map(RefCell::borrow)
    }

    pub fn iter_mut(&self) -> impl Iterator<Item = RefMut<'_, Chunk>> {
        self.chunks.values().map(RefCell::borrow_mut)
    }

    pub fn locations(&self) -> impl Iterator<Item = ChunkLocation> + '_ {
        self.chunks.keys().copied()
    }

    pub fn tick(&mut self, player_position: Vector<f32, 2>, physics: &mut Physics) {
        let center_chunk_location = Vector([
            player_position.x().div_euclid(CHUNK_SIZE as f32) as i64,
            player_position.y().div_euclid(CHUNK_SIZE as f32) as i64,
        ]);

        let mut chunk_load_range = self.chunk_load_range;
        chunk_load_range.shift_by(center_chunk_location);
        for chunk_y in chunk_load_range.min.y() ..= chunk_load_range.max.y() {
            for chunk_x in chunk_load_range.min.x() ..= chunk_load_range.max.x() {
                self.get_or_load_cell(Vector([chunk_x, chunk_y]), physics);
            }
        }

        let mut chunk_simulate_range = self.chunk_simulate_range;
        chunk_simulate_range.shift_by(center_chunk_location);
        for chunk_y in chunk_simulate_range.min.y() ..= chunk_simulate_range.max.y() {
            for chunk_x in chunk_simulate_range.min.x() ..= chunk_simulate_range.max.x() {
                if let Some(mut chunk) = self.get_mut(Vector([chunk_x, chunk_y])) {
                    chunk.attach_physics(physics);
                }
            }
        }

        // Unload the chunks that are out of load range
        let locations_to_unload: Vec<ChunkLocation> = self
            .locations()
            .filter(|&location| !chunk_load_range.contains_inclusive(location))
            .collect();
        // Detach physics for the chunks that are out of simulate range
        let locations_to_detach: Vec<ChunkLocation> = self
            .locations()
            .filter(|&location| !chunk_simulate_range.contains_inclusive(location))
            .collect();

        for location in locations_to_unload {
            self.unload(location, physics);
        }
        for location in locations_to_detach {
            if let Some(mut chunk) = self.get_mut(location) {
                chunk.detach_physics(physics);
            }
        }
    }
}
