use std::collections::{BTreeMap, HashMap};

use crate::{tools::*, *};
use crate::world::block::BlockRenderer;

pub mod block;
pub mod entity;
pub mod gen;

pub struct World<'world> {
    generator: Option<Box<dyn WorldGenerator>>,
    physics: phys::Physics,
    entities: HashMap<Uuid, Box<dyn entity::Entity + 'world>>,
    chunks: BTreeMap<block::ChunkLocation, block::Chunk>,
}

impl<'world> World<'world> where Self: 'world {
    pub fn new(generator: Option<Box<dyn WorldGenerator>>) -> Self {
        Self {
            generator,
            physics: phys::Physics::new(),
            entities: HashMap::new(),
            chunks: BTreeMap::new(),
        }
    }

    pub fn physics(&self) -> &phys::Physics {
        &self.physics
    }

    pub fn physics_mut(&mut self) -> &mut phys::Physics {
        &mut self.physics
    }

    pub fn get_chunk(&self, location: block::ChunkLocation) -> Option<&block::Chunk> {
        self.chunks.get(&location)
    }

    pub fn get_chunk_mut(&mut self, location: block::ChunkLocation) -> Option<&mut block::Chunk> {
        self.chunks.get_mut(&location)
    }

    pub fn force_get_chunk(&mut self, location: block::ChunkLocation) -> &block::Chunk {
        if !self.chunks.contains_key(&location) {
            self.chunks.insert(location, self.generate_chunk(location));
        }
        self.chunks.get(&location).unwrap()
    }

    pub fn force_get_chunk_mut(&mut self, location: block::ChunkLocation) -> &mut block::Chunk {
        if !self.chunks.contains_key(&location) {
            self.chunks.insert(location, self.generate_chunk(location));
        }
        self.chunks.get_mut(&location).unwrap()
    }

    pub fn put_chunk(&mut self, chunk: block::Chunk) {
        self.chunks.insert(chunk.location(), chunk);
    }

    pub fn generate_chunk(&self, location: block::ChunkLocation) -> block::Chunk {
        match &self.generator {
            Some(generator) => generator.get_chunk(location),
            None => block::Chunk::new(location),
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
        for chunk in self.chunks.values_mut() {
            chunk.update(dt);
        }
        for entity in self.entities.values_mut() {
            entity.update(dt, inputs, &mut self.physics);
        }
        self.physics.step_simulation(dt);
    }

    pub fn render(&mut self, dt: f32, block_renderer: &BlockRenderer) {
        let chunk_locations = Vec::from_iter(self.chunks.keys().cloned());
        for location in chunk_locations {
            // TODO: need a better way for chunks to gather information from adjacent chunks
            // We can safely unwrap because the entry is guaranteed to exist
            let mut chunk = self.chunks.remove(&location).unwrap();
            chunk.render(dt, block_renderer, self);
            self.chunks.insert(location, chunk);
        }
        for entity in self.entities.values_mut() {
            entity.render(dt);
        }
    }
}
