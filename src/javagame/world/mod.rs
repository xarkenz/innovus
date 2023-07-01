use std::collections::BTreeMap;

use crate::{tools::*, *};

pub mod block;
pub mod entity;
pub mod gen;

pub struct World {
    generator: Option<Box<dyn WorldGenerator>>,
    physics: phys::Physics,
    entities: Vec<Box<dyn entity::Entity>>,
    chunks: BTreeMap<block::ChunkLocation, block::Chunk>,
}

impl World {
    pub fn new(generator: Option<Box<dyn WorldGenerator>>) -> Self {
        Self {
            generator,
            physics: phys::Physics::new(),
            entities: Vec::new(),
            chunks: BTreeMap::new(),
        }
    }

    pub fn get_chunk(&self, location: block::ChunkLocation) -> Option<&block::Chunk> {
        self.chunks.get(&location)
    }

    pub fn force_get_chunk(&mut self, location: block::ChunkLocation) -> &block::Chunk {
        if !self.chunks.contains_key(&location) {
            self.chunks.insert(location, self.generate_chunk(location));
        }
        self.get_chunk(location).unwrap()
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

    pub fn update(&mut self, inputs: &input::InputState, dt: f32) {
        for (_location, chunk) in &mut self.chunks {
            chunk.update(dt);
        }
        for entity in &mut self.entities {
            entity.update(dt, inputs, &mut self.physics);
        }
    }

    pub fn render(&mut self, dt: f32) {
        for (_location, chunk) in &mut self.chunks {
            chunk.render(dt);
        }
        for entity in &mut self.entities {
            entity.render(dt);
        }
    }
}
