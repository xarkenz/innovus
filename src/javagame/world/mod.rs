use std::cell::{Ref, RefMut};
use std::collections::HashMap;
use crate::tools::*;
use crate::world::block::CHUNK_SIZE;
use crate::world::entity::render::EntityRenderer;
use crate::world::particle::{choose_random, random_unit_vector, ParticleInfo, ParticleManager};

pub mod block;
pub mod entity;
pub mod gen;
pub mod particle;

pub const SECONDS_PER_TICK: f32 = 0.05;

pub struct World<'world> {
    physics: phys::Physics,
    chunks: block::ChunkMap,
    entities: HashMap<Uuid, Box<dyn entity::Entity + 'world>>,
    entity_renderer: EntityRenderer,
    seconds_since_last_tick: f32,
    chunk_loader_entity: Option<Uuid>,
    particles: ParticleManager,
}

impl<'world> World<'world> where Self: 'world {
    pub fn new(generator: Option<Box<dyn gen::WorldGenerator>>) -> Self {
        Self {
            physics: phys::Physics::new(),
            chunks: block::ChunkMap::new(generator),
            entities: HashMap::new(),
            entity_renderer: EntityRenderer::new(),
            seconds_since_last_tick: SECONDS_PER_TICK,
            chunk_loader_entity: None,
            particles: ParticleManager::new(),
        }
    }

    pub fn physics(&self) -> &phys::Physics {
        &self.physics
    }

    pub fn physics_mut(&mut self) -> &mut phys::Physics {
        &mut self.physics
    }

    pub fn chunks(&self) -> &block::ChunkMap {
        &self.chunks
    }

    pub fn get_chunk(&self, location: block::ChunkLocation) -> Option<Ref<'_, block::Chunk>> {
        self.chunks.get(location)
    }

    pub fn get_chunk_mut(&self, location: block::ChunkLocation) -> Option<RefMut<'_, block::Chunk>> {
        self.chunks.get_mut(location)
    }

    pub fn load_chunk(&mut self, location: block::ChunkLocation) -> Ref<'_, block::Chunk> {
        self.chunks.get_or_load(location, &mut self.physics)
    }

    pub fn load_chunk_mut(&mut self, location: block::ChunkLocation) -> RefMut<'_, block::Chunk> {
        self.chunks.get_or_load_mut(location, &mut self.physics)
    }

    pub fn unload_chunk(&mut self, location: block::ChunkLocation) {
        self.chunks.unload(location, &mut self.physics);
    }

    pub fn user_place_block(&mut self, chunk_location: block::ChunkLocation, block_x: usize, block_y: usize, hand: &'static block::BlockType) {
        if let Some(mut chunk) = self.chunks.get_mut(chunk_location) {
            if let Some(block) = chunk.block_at(block_x, block_y).right_click(hand) {
                chunk.set_block_at(block_x, block_y, block, &self.chunks, &mut self.physics);
            }
        }
    }

    pub fn user_destroy_block(&mut self, chunk_location: block::ChunkLocation, block_x: usize, block_y: usize, assets: &mut asset::AssetPool) {
        if let Some(mut chunk) = self.chunks.get_mut(chunk_location) {
            let block_type = chunk.block_at(block_x, block_y).block_type();
            if block_type != &block::types::AIR {
                chunk.set_block_at(block_x, block_y, block::Block::new(&block::types::AIR), &self.chunks, &mut self.physics);
                // Create particles coming from the center of the destroyed block
                if let Some(palette) = block_type.palette_key.and_then(|key| assets.get_color_palette(key).ok()) {
                    let position = Vector([
                        chunk_location.x() as f32 * CHUNK_SIZE as f32 + block_x as f32 + 0.5,
                        chunk_location.y() as f32 * CHUNK_SIZE as f32 + block_y as f32 + 0.5,
                    ]);
                    for _ in 0..16 {
                        let velocity = random_unit_vector() * 3.0 + random_unit_vector() * 1.0;
                        let Some(&color) = choose_random(palette.colors()) else {
                            continue;
                        };
                        self.particles.create_particle(ParticleInfo {
                            position,
                            velocity,
                            color,
                            size: 2.0,
                            ..Default::default()
                        });
                    }
                }
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

    pub fn reload_assets(&mut self, assets: &mut asset::AssetPool) {
        for mut chunk in self.chunks.iter_mut() {
            chunk.set_all_need_render();
        }
        for entity in self.entities.values_mut() {
            entity.init_appearance(assets, &mut self.entity_renderer);
        }
    }

    pub fn update(&mut self, inputs: &input::InputState, dt: f32) {
        self.seconds_since_last_tick += dt;
        if self.seconds_since_last_tick >= SECONDS_PER_TICK {
            // Advance one tick
            self.seconds_since_last_tick -= SECONDS_PER_TICK;
            // Perform tick actions
            self.tick();
        }
        for entity in self.entities.values_mut() {
            entity.update(dt, inputs, &mut self.physics, &mut self.entity_renderer, &mut self.particles);
        }
        self.physics.step_simulation(dt);
        self.particles.update(dt);
    }

    fn tick(&mut self) {
        self.entity_renderer.tick();

        let chunk_loader_pos = self.chunk_loader_entity
            .as_ref()
            .and_then(|uuid| self.entities.get(uuid))
            .map(|entity| entity.position());
        self.chunks.tick(chunk_loader_pos, &mut self.physics);
    }

    pub fn render_block_layer(&mut self, assets: &asset::AssetPool) {
        assets.block_texture().bind();
        for mut chunk in self.chunks.iter_mut() {
            chunk.render(assets, &self.chunks);
        }
        self.particles.render();
    }

    pub fn render_entity_layer(&mut self, assets: &asset::AssetPool) {
        assets.entity_texture().bind();
        self.entity_renderer.render_all();
    }
}
