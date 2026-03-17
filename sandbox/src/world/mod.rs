use std::cell::{Ref, RefMut};
use std::collections::{HashMap, VecDeque};
use innovus::gfx::color::Color;
use innovus::tools::phys::Physics;
use crate::script::ScriptingEngine;
use crate::tools::*;
use crate::tools::asset::AssetPool;
use crate::tools::asset::text::TextElement;
use crate::tools::input::InputState;
use crate::world::entity::{EntityRequest, EntityResponse, EntityResponseQueue};
use self::block::{light_value, Block, BlockSide, Chunk, ChunkLocation, ChunkMap, CHUNK_SIZE};
use self::block::preview::BlockPreview;
use self::camera::Camera;
use self::entity::Entity;
use self::entity::render::EntityRenderer;
use self::entity::types::player::{Player, PlayerMode};
use self::gen::WorldGenerator;
use self::item::Item;
use self::particle::{choose_random, random_unit_vector, ParticleInfo, ParticleManager};

pub mod block;
pub mod camera;
pub mod entity;
pub mod gen;
pub mod item;
pub mod particle;

pub struct World<'world> {
    seconds_per_tick: f32,
    seconds_since_last_tick: f32,
    camera: Camera,
    physics: Physics,
    scripting: Option<ScriptingEngine>,
    chunks: ChunkMap,
    player: Player,
    entities: HashMap<Uuid, Box<dyn Entity + 'world>>,
    entity_renderer: EntityRenderer,
    particles: ParticleManager,
    block_preview: BlockPreview,
    sky_color: Vector<f32, 3>,
    sky_light: f32,
    frame_events: VecDeque<FrameEvent>,
    tick_events: VecDeque<TickEvent>,
}

pub enum WorldRequest<'a> {
    RunFrame {
        dt: f32,
        assets: &'a mut AssetPool,
    },
    HandleMouse(&'a InputState),
    HandleKeyboard(&'a InputState),
    ReloadAssets(&'a mut AssetPool),
    GetPlayerInventory,
    GetPlayerHotbar,
    PlayerChat {
        content: String,
    },
}

pub enum WorldResponse {
    Inventory {
        items: Vec<Item>,
    },
    PlayerInfo {
        position: Vector<f32, 2>,
        velocity: Vector<f32, 2>,
        held_item: Item,
    },
    ChatMessage {
        content: TextElement,
        color: Color,
    },
    PlaySound {
        path: String,
    },
}

pub type WorldResponseQueue = VecDeque<WorldResponse>;

pub enum FrameEvent {
    CreateParticle {
        particle: ParticleInfo,
        palette_key: Option<&'static str>,
    },
}

pub enum TickEvent {
    PlayerChat {
        content: String,
    },
}

impl<'world> World<'world> {
    pub fn new(generator: Option<Box<dyn WorldGenerator>>, camera: Camera, assets: &mut AssetPool) -> Self {
        let seconds_per_tick = 0.05;
        let mut world = Self {
            seconds_per_tick,
            seconds_since_last_tick: seconds_per_tick,
            camera,
            physics: Physics::new(),
            scripting: Some(ScriptingEngine::new()),
            chunks: ChunkMap::new(generator),
            entities: HashMap::new(),
            entity_renderer: EntityRenderer::new(),
            player: Player::new(generate_uuid(), Vector([-0.5, 0.0]), None, PlayerMode::Normal),
            particles: ParticleManager::new(),
            block_preview: BlockPreview::new(Vector::zero(), Default::default(), 0.4),
            sky_color: Vector([0.6, 0.8, 1.0]),
            sky_light: 1.0,
            frame_events: VecDeque::new(),
            tick_events: VecDeque::new(),
        };
        world.player.attach_collision(&mut world.physics);
        world.player.attach_appearance(assets, &mut world.entity_renderer);
        world.camera.set_position(world.player.position());
        world
    }

    pub fn sky_color(&self) -> Color {
        Color::RGB(self.sky_color.mul(self.sky_light))
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

    pub fn scripting(&self) -> &ScriptingEngine {
        self.scripting.as_ref().expect("scripting is inaccessible while a command is running")
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

    pub fn update(&mut self, request: WorldRequest, responses: &mut WorldResponseQueue) {
        match request {
            WorldRequest::RunFrame { dt, assets } => {
                self.run_frame(dt, assets, responses);
            }
            WorldRequest::HandleMouse(inputs) => {
                if let Some(scroll_amount) = inputs.scroll_amount() {
                    let target_zoom = self.camera.zoom().mul(f32::powf(1.125, scroll_amount.y() as f32));
                    self.camera.set_zoom(target_zoom);
                }
                let cursor_pos = self.camera.get_world_pos(inputs.cursor_pos().map(|x| x as f32));
                self.block_preview.set_position(cursor_pos);
                self.player.handle_mouse(inputs, cursor_pos);
            }
            WorldRequest::HandleKeyboard(inputs) => {
                self.player.handle_keyboard(inputs);
            }
            WorldRequest::ReloadAssets(assets) => {
                self.reload_assets(assets, responses);
            }
            WorldRequest::GetPlayerInventory => {
                todo!()
            }
            WorldRequest::GetPlayerHotbar => {
                todo!()
            }
            WorldRequest::PlayerChat { content } => {
                self.tick_events.push_back(TickEvent::PlayerChat {
                    content,
                });
            }
        }
    }

    fn run_frame(&mut self, dt: f32, assets: &mut AssetPool, responses: &mut WorldResponseQueue) {
        self.seconds_since_last_tick += dt;
        if self.seconds_since_last_tick >= self.seconds_per_tick {
            // Advance one tick
            self.seconds_since_last_tick -= self.seconds_per_tick;
            // Perform tick actions
            self.tick(responses);
        }

        // Run frame for all entities
        let mut entity_responses = EntityResponseQueue::new();
        for entity in self.entities.values_mut() {
            entity.update(EntityRequest::RunFrame {
                dt,
                physics: &mut self.physics,
                renderer: &mut self.entity_renderer,
            }, &mut entity_responses);
        }
        self.player.update(EntityRequest::RunFrame {
            dt,
            physics: &mut self.physics,
            renderer: &mut self.entity_renderer,
        }, &mut entity_responses);
        self.process_entity_responses(entity_responses, responses);

        // Process queued frame events, if any
        while let Some(event) = self.frame_events.pop_front() {
            match event {
                FrameEvent::CreateParticle { mut particle, palette_key } => {
                    if let Some(&color) = palette_key
                        .and_then(|key| assets.get_color_palette(key).ok())
                        .and_then(|palette| choose_random(palette.colors()))
                    {
                        particle.color = color.into();
                    }
                    self.particles.create_particle(particle);
                }
            }
        }

        self.camera.set_target(self.player.position());
        self.camera.update(dt);
        self.physics.step_simulation(dt);
        self.particles.update(dt);

        responses.push_back(WorldResponse::PlayerInfo {
            position: self.player.position(),
            velocity: self.player.velocity(),
            held_item: self.player.held_item().clone(),
        });

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

    fn tick(&mut self, responses: &mut WorldResponseQueue) {
        // Process queued tick events, if any
        while let Some(event) = self.tick_events.pop_front() {
            match event {
                TickEvent::PlayerChat { content } => {
                    self.process_player_chat(content, responses);
                }
            }
        }

        // Tick all entities
        let mut entity_responses = EntityResponseQueue::new();
        for entity in self.entities.values_mut() {
            entity.update(EntityRequest::Tick {
                chunks: &self.chunks,
            }, &mut entity_responses);
        }
        self.player.update(EntityRequest::Tick {
            chunks: &self.chunks,
        }, &mut entity_responses);
        self.process_entity_responses(entity_responses, responses);

        self.entity_renderer.tick();
        self.block_preview.set_item_type(self.player.held_item().item_type());
        self.chunks.tick(self.player.position(), &mut self.physics);
    }

    fn process_player_chat(&mut self, content: String, responses: &mut WorldResponseQueue) {
        let content = content.trim();
        if !content.is_empty() {
            if content.starts_with('/') {
                let scripting = self.scripting.take().expect("scripting is inaccessible");
                match scripting.dispatch_command(&content, self) {
                    Ok(content) => {
                        responses.push_back(WorldResponse::ChatMessage {
                            content,
                            color: Color::Green,
                        });
                    }
                    Err(content) => {
                        responses.push_back(WorldResponse::ChatMessage {
                            content,
                            color: Color::Red,
                        });
                    }
                }
                self.scripting = Some(scripting);
            }
            else {
                responses.push_back(WorldResponse::ChatMessage {
                    content: content.into(),
                    color: Color::White,
                });
            }
        }
    }

    fn process_entity_responses(&mut self, mut entity_responses: EntityResponseQueue, responses: &mut WorldResponseQueue) {
        while let Some((uuid, entity_response)) = entity_responses.pop_front() {
            match entity_response {
                EntityResponse::FrameEvent(event) => {
                    self.frame_events.push_back(event);
                }
                EntityResponse::TickEvent(event) => {
                    self.tick_events.push_back(event);
                }
                EntityResponse::Die => {
                    self.destroy_entity(uuid);
                }
                EntityResponse::UseHeldItem { item, chunk_location, block_x, block_y, side } => {
                    self.entity_use_item(uuid, item, chunk_location, block_x, block_y, side, &mut entity_responses, responses);
                }
                EntityResponse::DestroyBlock { chunk_location, block_x, block_y } => {
                    self.entity_destroy_block(uuid, chunk_location, block_x, block_y, &mut entity_responses, responses);
                }
            }
        }
    }

    fn entity_use_item(&mut self, uuid: Uuid, item: Item, chunk_location: ChunkLocation, block_x: usize, block_y: usize, side: BlockSide, entity_responses: &mut EntityResponseQueue, responses: &mut WorldResponseQueue) {
        let Some(mut chunk) = self.chunks.get_mut(chunk_location) else {
            return;
        };

        let (changed_block, changed_item) = chunk
            .block_at(block_x, block_y)
            .handle_use_item(&item, side);
        if let Some(block) = changed_block {
            chunk.set_block_at(block_x, block_y, block, &self.chunks, &mut self.physics);
            responses.push_back(WorldResponse::PlaySound {
                path: "sounds/block/wood_big_1.ogg".into(),
            });
        }
        // This is necessary; the compiler complains about get_entity_mut otherwise.
        // Honestly, this the first time I've actually had to do this explicitly. If this were a
        // plain old &mut Chunk, the compiler would notice that the reference can get dropped here.
        // (I could have scoped the logic differently so this wouldn't be needed, but whatever.)
        drop(chunk);

        if let Some(item) = changed_item {
            if let Some(entity) = self.get_entity_mut(uuid) {
                entity.update(EntityRequest::SetHeldItem(item), entity_responses);
            }
        }
    }

    fn entity_destroy_block(&mut self, uuid: Uuid, chunk_location: ChunkLocation, block_x: usize, block_y: usize, entity_responses: &mut EntityResponseQueue, responses: &mut WorldResponseQueue) {
        let _ = (uuid, entity_responses);
        let Some(mut chunk) = self.chunks.get_mut(chunk_location) else {
            return;
        };

        let block_type = chunk.block_at(block_x, block_y).block_type();
        if !block_type.is_air() {
            chunk.set_block_at(block_x, block_y, Block::default(), &self.chunks, &mut self.physics);
            let position = Vector([
                chunk_location.x() as f32 * CHUNK_SIZE as f32 + block_x as f32 + 0.5,
                chunk_location.y() as f32 * CHUNK_SIZE as f32 + block_y as f32 + 0.5,
            ]);
            // Create particles coming from the center of the destroyed block
            for _ in 0..16 {
                let velocity = random_unit_vector().mul(3.0) + random_unit_vector().mul(1.0);
                let particle = ParticleInfo {
                    position: position + velocity.mul(0.25),
                    velocity,
                    size: 2.0,
                    ..Default::default()
                };
                self.frame_events.push_back(FrameEvent::CreateParticle {
                    particle,
                    palette_key: block_type.palette_key(),
                });
            }
            responses.push_back(WorldResponse::PlaySound {
                path: "sounds/block/wood_big_0.ogg".into(),
            });
        }
    }

    fn reload_assets(&mut self, assets: &mut AssetPool, responses: &mut WorldResponseQueue) {
        let _ = responses;

        for mut chunk in self.chunks.iter_mut() {
            chunk.set_all_need_render();
        }
        for entity in self.entities.values_mut() {
            entity.attach_appearance(assets, &mut self.entity_renderer);
        }
        self.player.attach_appearance(assets, &mut self.entity_renderer);
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
