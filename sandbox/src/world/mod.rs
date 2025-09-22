use std::cell::{Ref, RefMut};
use std::collections::HashMap;
use innovus::gfx::color::RGBColor;
use innovus::tools::phys::Physics;
use crate::tools::*;
use crate::tools::asset::AssetPool;
use crate::tools::input::InputState;
use block::{light_value, Block, Chunk, ChunkLocation, ChunkMap, CHUNK_SIZE};
use camera::Camera;
use entity::Entity;
use entity::render::EntityRenderer;
use entity::types::player::{Player, PlayerMode};
use gen::WorldGenerator;
use block::preview::BlockPreview;
use particle::{choose_random, random_unit_vector, ParticleInfo, ParticleManager};
use crate::audio::AudioEngine;

pub mod block;
pub mod camera;
pub mod entity;
pub mod gen;
pub mod item;
pub mod particle;

pub const SECONDS_PER_TICK: f32 = 0.05;

pub struct World<'world> {
    seconds_since_last_tick: f32,
    camera: Camera,
    physics: Physics,
    chunks: ChunkMap,
    player: Player,
    entities: HashMap<Uuid, Box<dyn Entity + 'world>>,
    entity_renderer: EntityRenderer,
    particles: ParticleManager,
    block_preview: BlockPreview,
    sky_color: Vector<f32, 3>,
    sky_light: f32,
}

impl<'world> World<'world> {
    pub fn new(generator: Option<Box<dyn WorldGenerator>>, camera: Camera, assets: &mut AssetPool) -> Self {
        let mut world = Self {
            seconds_since_last_tick: SECONDS_PER_TICK,
            camera,
            physics: Physics::new(),
            chunks: ChunkMap::new(generator),
            entities: HashMap::new(),
            entity_renderer: EntityRenderer::new(),
            player: Player::new(generate_uuid(), Vector([-0.5, 0.0]), None, PlayerMode::Normal),
            particles: ParticleManager::new(),
            block_preview: BlockPreview::new(Vector::zero(), &item::types::AIR, 0.4),
            sky_color: Vector([0.6, 0.8, 1.0]),
            sky_light: 1.0,
        };
        world.player.attach_collision(&mut world.physics);
        world.player.attach_appearance(assets, &mut world.entity_renderer);
        world.camera.set_position(world.player.position());
        world
    }

    pub fn sky_color(&self) -> RGBColor {
        RGBColor(self.sky_color * self.sky_light)
    }

    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    pub fn camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }

    pub fn physics(&self) -> &Physics {
        &self.physics
    }

    pub fn physics_mut(&mut self) -> &mut Physics {
        &mut self.physics
    }

    pub fn chunks(&self) -> &ChunkMap {
        &self.chunks
    }

    pub fn get_chunk(&self, location: ChunkLocation) -> Option<Ref<'_, Chunk>> {
        self.chunks.get(location)
    }

    pub fn get_chunk_mut(&self, location: ChunkLocation) -> Option<RefMut<'_, Chunk>> {
        self.chunks.get_mut(location)
    }

    pub fn load_chunk(&mut self, location: ChunkLocation) -> Ref<'_, Chunk> {
        self.chunks.get_or_load(location, &mut self.physics)
    }

    pub fn load_chunk_mut(&mut self, location: ChunkLocation) -> RefMut<'_, Chunk> {
        self.chunks.get_or_load_mut(location, &mut self.physics)
    }

    pub fn unload_chunk(&mut self, location: ChunkLocation) {
        self.chunks.unload(location, &mut self.physics);
    }

    pub fn player_use_item(&mut self, chunk_location: ChunkLocation, block_x: usize, block_y: usize, assets: &AssetPool, audio: &AudioEngine) {
        if let Some(mut chunk) = self.chunks.get_mut(chunk_location) {
            let (changed_block, changed_item) = chunk
                .block_at(block_x, block_y)
                .handle_right_click(self.player.held_item());
            if let Some(block) = changed_block {
                chunk.set_block_at(block_x, block_y, block, &self.chunks, &mut self.physics);
                audio.play_sound(assets.resolve_path("sounds/block/wood_big_1.ogg")).unwrap();
            }
            if let Some(item) = changed_item {
                self.player.set_held_item(item);
            }
        }
    }

    pub fn user_destroy_block(&mut self, chunk_location: ChunkLocation, block_x: usize, block_y: usize, assets: &mut AssetPool, audio: &AudioEngine) {
        if let Some(mut chunk) = self.chunks.get_mut(chunk_location) {
            let block_type = chunk.block_at(block_x, block_y).block_type();
            if block_type != &block::types::AIR {
                chunk.set_block_at(block_x, block_y, Block::new(&block::types::AIR), &self.chunks, &mut self.physics);
                // Create particles coming from the center of the destroyed block
                if let Some(palette) = block_type.palette_key().and_then(|key| assets.get_color_palette(key).ok()) {
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
                    audio.play_sound(assets.resolve_path("sounds/block/wood_big_0.ogg")).unwrap();
                }
            }
        }
    }

    pub fn player(&mut self) -> &Player {
        &self.player
    }

    pub fn player_mut(&mut self) -> &mut Player {
        &mut self.player
    }

    pub fn add_entity(&mut self, mut entity: Box<dyn Entity>, assets: &mut AssetPool) {
        entity.attach_collision(&mut self.physics);
        entity.attach_appearance(assets, &mut self.entity_renderer);
        self.entities.insert(entity.uuid(), entity);
    }

    pub fn get_entity(&self, uuid: Uuid) -> Option<&dyn Entity> {
        match self.entities.get(&uuid) {
            Some(entity) => Some(entity.as_ref()),
            None if uuid == self.player.uuid() => Some(&self.player),
            None => None,
        }
    }

    pub fn get_entity_mut(&mut self, uuid: Uuid) -> Option<&mut (dyn Entity + 'world)> {
        match self.entities.get_mut(&uuid) {
            Some(entity) => Some(entity.as_mut()),
            None if uuid == self.player.uuid() => Some(&mut self.player),
            None => None,
        }
    }

    pub fn destroy_entity(&mut self, uuid: Uuid) -> bool {
        if let Some(mut entity) = self.entities.remove(&uuid) {
            entity.detach_collision(&mut self.physics);
            entity.detach_appearance(&mut self.entity_renderer);
            true
        }
        else {
            false
        }
    }

    pub fn set_block_preview_position(&mut self, position: Vector<f32, 2>) {
        self.block_preview.set_position(position);
    }

    pub fn reload_assets(&mut self, assets: &mut AssetPool) {
        for mut chunk in self.chunks.iter_mut() {
            chunk.set_all_need_render();
        }
        for entity in self.entities.values_mut() {
            entity.attach_appearance(assets, &mut self.entity_renderer);
        }
        self.player.attach_appearance(assets, &mut self.entity_renderer);
    }

    pub fn update(&mut self, inputs: &InputState, dt: f32) {
        self.seconds_since_last_tick += dt;
        if self.seconds_since_last_tick >= SECONDS_PER_TICK {
            // Advance one tick
            self.seconds_since_last_tick -= SECONDS_PER_TICK;
            // Perform tick actions
            self.tick();
        }

        for entity in self.entities.values_mut() {
            entity.update(
                dt,
                inputs,
                &mut self.physics,
                &mut self.entity_renderer,
                &mut self.chunks,
                &mut self.particles,
            );
        }
        self.player.update(
            dt,
            inputs,
            &mut self.physics,
            &mut self.entity_renderer,
            &mut self.chunks,
            &mut self.particles,
        );

        self.camera.set_target(self.player.position());
        self.camera.update(dt);
        self.physics.step_simulation(dt);
        self.particles.update(dt);

        let target_sky_light = {
            let camera_pos = self.camera.position();
            let chunk_location = Vector([
                camera_pos.x().div_euclid(CHUNK_SIZE as f32) as i64,
                camera_pos.y().div_euclid(CHUNK_SIZE as f32) as i64,
            ]);
            let block_x = camera_pos.x().rem_euclid(CHUNK_SIZE as f32) as usize;
            let block_y = camera_pos.y().rem_euclid(CHUNK_SIZE as f32) as usize;
            if let Some(chunk) = self.get_chunk(chunk_location) {
                light_value(chunk.block_slot_at(block_x, block_y).sky_light())
            }
            else {
                light_value(15)
            }
        };
        self.sky_light += (target_sky_light - self.sky_light) * dt.min(1.0);
    }

    fn tick(&mut self) {
        self.entity_renderer.tick();
        self.block_preview.set_item_type(self.player.held_item().item_type());
        self.chunks.tick(self.player.position(), &mut self.physics);
    }

    pub fn render(&mut self, assets: &AssetPool) {
        assets.block_texture().bind();
        assets.block_shaders().set_uniform("tex_atlas", assets.block_texture());
        assets.block_shaders().set_uniform("camera_view", self.camera.view());
        assets.block_shaders().set_uniform("camera_proj", self.camera.projection());
        for mut chunk in self.chunks.iter_mut() {
            chunk.render(assets, &self.chunks);
        }

        assets.default_shaders().set_uniform("tex_atlas", assets.block_texture());
        assets.default_shaders().set_uniform("camera_view", self.camera.view());
        assets.default_shaders().set_uniform("camera_proj", self.camera.projection());
        self.particles.render();
        self.block_preview.render(assets, &self.chunks);

        assets.entity_texture().bind();
        assets.default_shaders().set_uniform("tex_atlas", assets.entity_texture());
        self.entity_renderer.render_all();
    }
}
