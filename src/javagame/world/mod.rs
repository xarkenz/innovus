use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::ops::DerefMut;
use crate::tools::*;

pub mod block;
pub mod entity;
pub mod gen;

pub struct World<'world> {
    generator: Option<Box<dyn gen::WorldGenerator>>,
    physics: phys::Physics,
    entities: HashMap<Uuid, Box<dyn entity::Entity + 'world>>,
    chunks: block::ChunkMap,
}

impl<'world> World<'world> where Self: 'world {
    pub fn new(generator: Option<Box<dyn gen::WorldGenerator>>) -> Self {
        Self {
            generator,
            physics: phys::Physics::new(),
            entities: HashMap::new(),
            chunks: block::ChunkMap::new(),
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

    pub fn add_entity(&mut self, entity: Box<dyn entity::Entity>) {
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
            entity.destroy(&mut self.physics);
            true
        }
        else {
            false
        }
    }

    pub fn update(&mut self, inputs: &input::InputState, dt: f32) {
        for chunk in self.chunks.values() {
            chunk.borrow_mut().update(dt);
        }
        for entity in self.entities.values_mut() {
            entity.update(dt, inputs, &mut self.physics);
        }
        self.physics.step_simulation(dt);
    }

    pub fn render(&mut self, dt: f32, block_gfx: &asset::BlockGraphics) {
        for chunk in self.chunks.values() {
            chunk.borrow_mut().render(dt, block_gfx, &self.chunks);
        }
        for entity in self.entities.values_mut() {
            entity.render(dt);
        }
    }
}
