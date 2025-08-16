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
}

impl<'world> World<'world> where Self: 'world {
    pub fn new(generator: Option<Box<dyn gen::WorldGenerator>>) -> Self {
        Self {
            generator,
            physics: phys::Physics::new(),
            chunks: block::ChunkMap::new(),
            entities: HashMap::new(),
            entity_renderer: EntityRenderer::new(),
            seconds_since_last_tick: 0.0,
        }
    }

    pub fn physics(&self) -> &phys::Physics {
        &self.physics
    }

    pub fn physics_mut(&mut self) -> &mut phys::Physics {
        &mut self.physics
    }

    pub fn get_chunk(&self, location: block::ChunkLocation) -> Option<Ref<block::Chunk>> {
        self.chunks.get(&location).map(|chunk| chunk.borrow())
    }

    pub fn get_chunk_mut(&self, location: block::ChunkLocation) -> Option<RefMut<block::Chunk>> {
        self.chunks.get(&location).map(|chunk| chunk.borrow_mut())
    }

    pub fn force_get_chunk(&mut self, location: block::ChunkLocation) -> Ref<block::Chunk> {
        self.force_get_chunk_cell(location).borrow()
    }

    pub fn force_get_chunk_mut(&mut self, location: block::ChunkLocation) -> RefMut<block::Chunk> {
        self.force_get_chunk_cell(location).borrow_mut()
    }

    fn force_get_chunk_cell(&mut self, location: block::ChunkLocation) -> &RefCell<block::Chunk> {
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

    pub fn user_place_block(&mut self, chunk_location: block::ChunkLocation, block_x: usize, block_y: usize, block_type: &'static block::BlockType) {
        self.force_get_chunk_cell(chunk_location);
        let mut chunk = self.chunks[&chunk_location].borrow_mut();
        if chunk.block_at(block_x, block_y).block_type == &block::types::AIR {
            chunk.set_block_at(block_x, block_y, block::Block::new(block_type, 0, 15), &self.chunks, &mut self.physics);
        }
    }

    pub fn user_destroy_block(&mut self, chunk_location: block::ChunkLocation, block_x: usize, block_y: usize) {
        self.force_get_chunk_cell(chunk_location);
        let mut chunk = self.chunks[&chunk_location].borrow_mut();
        if chunk.block_at(block_x, block_y).block_type != &block::types::AIR {
            chunk.set_block_at(block_x, block_y, block::Block::new(&block::types::AIR, 0, 15), &self.chunks, &mut self.physics);
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

    pub fn update(&mut self, inputs: &input::InputState, dt: f32) {
        self.seconds_since_last_tick += dt;
        if self.seconds_since_last_tick >= SECONDS_PER_TICK {
            // Advance one tick
            self.seconds_since_last_tick -= SECONDS_PER_TICK;
            // Perform tick actions
            self.entity_renderer.tick();
        }
        for chunk in self.chunks.values() {
            chunk.borrow_mut().update(dt);
        }
        for entity in self.entities.values_mut() {
            entity.update(dt, inputs, &mut self.physics, &mut self.entity_renderer);
        }
        self.physics.step_simulation(dt);
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
