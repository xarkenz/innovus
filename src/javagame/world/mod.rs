use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::ops::DerefMut;
use crate::tools::*;
use crate::world::entity::render::EntityRenderer;

pub mod block;
pub mod entity;
pub mod gen;

pub const SECONDS_PER_TICK: f32 = 0.05;

pub struct World<'world> {
    generator: Option<Box<dyn gen::WorldGenerator>>,
    physics: phys::Physics,
    chunks: block::ChunkMap,
    entities: HashMap<Uuid, Box<dyn entity::Entity + 'world>>,
    entity_renderer: EntityRenderer,
    seconds_since_last_tick: f32,
    chunk_load_range: Rectangle<i64>,
    chunk_simulate_range: Rectangle<i64>,
    chunk_loader_entity: Option<Uuid>,
}

impl<'world> World<'world> where Self: 'world {
    pub fn new(generator: Option<Box<dyn gen::WorldGenerator>>) -> Self {
        Self {
            generator,
            physics: phys::Physics::new(),
            chunks: block::ChunkMap::new(),
            entities: HashMap::new(),
            entity_renderer: EntityRenderer::new(),
            seconds_since_last_tick: SECONDS_PER_TICK,
            chunk_load_range: Rectangle::new(Vector([-2, -2]), Vector([2, 2])),
            chunk_simulate_range: Rectangle::new(Vector([-1, -1]), Vector([1, 1])),
            chunk_loader_entity: None,
        }
    }

    pub fn physics(&self) -> &phys::Physics {
        &self.physics
    }

    pub fn physics_mut(&mut self) -> &mut phys::Physics {
        &mut self.physics
    }

    pub fn get_chunk(&self, location: block::ChunkLocation) -> Option<Ref<'_, block::Chunk>> {
        self.chunks.get(&location).map(|chunk| chunk.borrow())
    }

    pub fn get_chunk_mut(&self, location: block::ChunkLocation) -> Option<RefMut<'_, block::Chunk>> {
        self.chunks.get(&location).map(|chunk| chunk.borrow_mut())
    }

    pub fn load_chunk(&mut self, location: block::ChunkLocation) -> Ref<'_, block::Chunk> {
        self.load_chunk_cell(location).borrow()
    }

    pub fn load_chunk_mut(&mut self, location: block::ChunkLocation) -> RefMut<'_, block::Chunk> {
        self.load_chunk_cell(location).borrow_mut()
    }

    fn load_chunk_cell(&mut self, location: block::ChunkLocation) -> &RefCell<block::Chunk> {
        if self.chunks.contains_key(&location) {
            &self.chunks[&location]
        }
        else {
            self.chunks.insert(location, RefCell::new(block::Chunk::new(location)));
            let cell = &self.chunks[&location];
            if let Some(generator) = &self.generator {
                generator.generate_chunk(cell.borrow_mut().deref_mut(), &self.chunks, &mut self.physics);
            }
            cell
        }
    }

    pub fn unload_chunk(&mut self, location: block::ChunkLocation) {
        if let Some(chunk) = self.chunks.remove(&location) {
            chunk.into_inner().detach_physics(&mut self.physics);
        }
    }

    pub fn user_place_block(&mut self, chunk_location: block::ChunkLocation, block_x: usize, block_y: usize, block_type: &'static block::BlockType) {
        if let Some(mut chunk) = self.chunks.get(&chunk_location).map(|chunk| chunk.borrow_mut()) {
            if chunk.block_at(block_x, block_y).block_type() == &block::types::AIR {
                chunk.set_block_at(block_x, block_y, block::Block::new(block_type), &self.chunks, &mut self.physics);
            }
        }
    }

    pub fn user_destroy_block(&mut self, chunk_location: block::ChunkLocation, block_x: usize, block_y: usize) {
        if let Some(mut chunk) = self.chunks.get(&chunk_location).map(|chunk| chunk.borrow_mut()) {
            if chunk.block_at(block_x, block_y).block_type() != &block::types::AIR {
                chunk.set_block_at(block_x, block_y, block::Block::new(&block::types::AIR), &self.chunks, &mut self.physics);
            }
        }
    }

    pub fn add_entity(&mut self, mut entity: Box<dyn entity::Entity>, assets: &mut asset::AssetPool) {
        entity.init_collision(&mut self.physics);
        entity.init_appearance(assets, &mut self.entity_renderer);
        self.entities.insert(entity.uuid(), entity);
    }

    pub fn get_entity(&self, uuid: Uuid) -> Option<&dyn entity::Entity> {
        self.entities.get(&uuid).map(Box::as_ref)
    }

    pub fn get_entity_mut(&mut self, uuid: Uuid) -> Option<&mut (dyn entity::Entity + 'world)> {
        self.entities.get_mut(&uuid).map(Box::as_mut)
    }

    pub fn destroy_entity(&mut self, uuid: Uuid) -> bool {
        if let Some(mut entity) = self.entities.remove(&uuid) {
            entity.destroy(&mut self.physics, &mut self.entity_renderer);
            true
        }
        else {
            false
        }
    }

    pub fn set_chunk_loader_entity(&mut self, entity: Option<Uuid>) {
        self.chunk_loader_entity = entity;
    }

    pub fn update(&mut self, inputs: &input::InputState, dt: f32) {
        self.seconds_since_last_tick += dt;
        if self.seconds_since_last_tick >= SECONDS_PER_TICK {
            // Advance one tick
            self.seconds_since_last_tick -= SECONDS_PER_TICK;
            // Perform tick actions
            self.tick();
        }
        for chunk in self.chunks.values() {
            chunk.borrow_mut().update(dt);
        }
        for entity in self.entities.values_mut() {
            entity.update(dt, inputs, &mut self.physics, &mut self.entity_renderer);
        }
        self.physics.step_simulation(dt);
    }

    fn tick(&mut self) {
        self.entity_renderer.tick();

        let chunk_loader_pos = self.chunk_loader_entity
            .as_ref()
            .and_then(|uuid| self.entities.get(uuid))
            .map(|entity| entity.position());

        let locations_to_unload: Vec<block::ChunkLocation>;
        let locations_to_detach: Vec<block::ChunkLocation>;
        if let Some(world_pos) = chunk_loader_pos {
            let center_chunk_location = Vector([
                world_pos.x().div_euclid(block::CHUNK_SIZE as f32) as i64,
                world_pos.y().div_euclid(block::CHUNK_SIZE as f32) as i64,
            ]);

            let mut chunk_load_range = self.chunk_load_range;
            chunk_load_range.shift_by(center_chunk_location);
            for chunk_y in chunk_load_range.min_y() ..= chunk_load_range.max_y() {
                for chunk_x in chunk_load_range.min_x() ..= chunk_load_range.max_x() {
                    self.load_chunk(Vector([chunk_x, chunk_y]));
                }
            }

            let mut chunk_simulate_range = self.chunk_simulate_range;
            chunk_simulate_range.shift_by(center_chunk_location);
            for chunk_y in chunk_simulate_range.min_y() ..= chunk_simulate_range.max_y() {
                for chunk_x in chunk_simulate_range.min_x() ..= chunk_simulate_range.max_x() {
                    if let Some(chunk) = self.chunks.get_mut(&Vector([chunk_x, chunk_y])) {
                        chunk.borrow_mut().attach_physics(&mut self.physics);
                    }
                }
            }

            // Unload the chunks that are out of load range
            locations_to_unload = self.chunks
                .keys()
                .copied()
                .filter(|&location| !chunk_load_range.contains_inclusive(location))
                .collect();
            // Detach physics for the chunks that are out of simulate range
            locations_to_detach = self.chunks
                .keys()
                .copied()
                .filter(|&location| !chunk_simulate_range.contains_inclusive(location))
                .collect();
        }
        else {
            // Unload all chunks
            locations_to_unload = self.chunks
                .keys()
                .copied()
                .collect();
            // Don't worry about detaching physics, unloading a chunk does that automatically
            locations_to_detach = Vec::new();
        }

        for location in locations_to_unload {
            self.unload_chunk(location);
        }
        for location in locations_to_detach {
            if let Some(chunk) = self.chunks.get_mut(&location) {
                chunk.borrow_mut().detach_physics(&mut self.physics);
            }
        }
    }

    pub fn render(&mut self, dt: f32, assets: &asset::AssetPool) {
        assets.block_texture().bind();
        for chunk in self.chunks.values() {
            chunk.borrow_mut().render(dt, assets, &self.chunks);
        }
        assets.entity_texture().bind();
        self.entity_renderer.render_all();
    }
}
