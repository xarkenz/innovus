use std::error::Error;
use std::path::Path;
use std::sync::Arc;
use winit::event::MouseButton;
use winit::keyboard::KeyCode;
use winit::window::Window;
use innovus::Game;
use innovus::gfx::color::Color;
use innovus::gfx::Gfx;
use innovus::gfx::pipeline::BindGroup;
use innovus::input::InputState;
use innovus::tools::{Clock, Vector};
use crate::audio::AudioEngine;
use crate::gui::GuiManager;
use crate::script::ScriptingEngine;
use crate::tools::asset::AssetPool;
use crate::world::camera::Camera;
use crate::world::block::{BlockSide, CHUNK_SIZE};
use crate::world::block::types::AIR;
use crate::world::entity::Entity;
use crate::world::entity::types::player::PlayerMode;
use crate::world::generation::types::OverworldGenerator;
use crate::world::generation::WorldGenerator;
use crate::world::item::{Item, ITEM_TYPES};
use crate::world::World;

pub struct SandboxGame<'a> {
    window: Arc<Window>,
    gfx: Gfx<'static>,
    frame_clock: Clock,
    fps_tracker: [f32; 120],
    fps_tracker_index: usize,
    viewport_size: Vector<f32, 2>,
    content_scale: Vector<f32, 2>,
    assets: AssetPool,
    gui: GuiManager,
    scripting: ScriptingEngine,
    audio: AudioEngine,
    camera_layout: wgpu::BindGroupLayout,
    current_world: Option<World<'a>>,
    last_block_pos: Option<(usize, usize)>,
}

impl<'a> SandboxGame<'a> {
    pub fn create(window: Arc<Window>, gfx: Gfx<'static>, assets_path: impl AsRef<Path>) -> Result<Self, String> {
        window.set_cursor_visible(false);
        window.set_maximized(true);

        let viewport_size = Vector([
            window.inner_size().width as f32,
            window.inner_size().height as f32,
        ]);
        // let content_scale = Vector::splat(window.scale_factor() as f32);
        let content_scale = Vector::one();

        let mut assets = AssetPool::load(&gfx, assets_path)?;
        let gui = GuiManager::create(&gfx, viewport_size, content_scale, 8.0, &mut assets)?;
        let camera_layout = Camera::create_layout(gfx.device());

        Ok(Self {
            window,
            gfx,
            frame_clock: Clock::start(),
            fps_tracker: [f32::INFINITY; 120],
            fps_tracker_index: 0,
            viewport_size,
            content_scale,
            gui,
            assets,
            scripting: ScriptingEngine::new(),
            audio: AudioEngine::new()?,
            camera_layout,
            current_world: None,
            last_block_pos: None,
        })
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn gfx(&self) -> &Gfx<'static> {
        &self.gfx
    }

    pub fn gfx_mut(&mut self) -> &mut Gfx<'static> {
        &mut self.gfx
    }

    pub fn viewport_size(&self) -> Vector<f32, 2> {
        self.viewport_size
    }

    pub fn set_viewport_size(&mut self, viewport_size: Vector<f32, 2>) {
        self.viewport_size = viewport_size;

        self.gui.set_viewport_size(viewport_size);
        if let Some(world) = &mut self.current_world {
            world.camera_mut().set_size(viewport_size);
        }
    }

    pub fn content_scale(&self) -> Vector<f32, 2> {
        self.content_scale
    }

    pub fn set_content_scale(&mut self, content_scale: Vector<f32, 2>) {
        self.content_scale = content_scale;

        self.gui.set_content_scale(content_scale);
    }

    pub fn current_world(&self) -> Option<&World<'_>> {
        self.current_world.as_ref()
    }

    pub fn enter_world(&mut self, generator: Option<Box<dyn WorldGenerator>>) {
        let camera = Camera::new(
            &self.gfx,
            &self.camera_layout,
            Vector::zero(),
            self.viewport_size,
            self.content_scale.mul(48.0),
            5.0,
        );
        self.current_world = Some(World::create(&self.gfx, generator, camera, &mut self.assets));
    }
}

impl<'a> Game for SandboxGame<'a> {
    fn start(window: Arc<Window>, gfx: Gfx<'static>) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized,
    {
        let mut game = SandboxGame::create(window, gfx, "sandbox/assets")?;
        game.enter_world(Some(Box::new(OverworldGenerator::new(0))));
        Ok(game)
    }

    fn handle_resize(&mut self, size: Vector<u32, 2>) {
        self.gfx.resize(size);
        self.set_viewport_size(size.map(|x| x as f32));
    }

    // fn handle_scale_change(&mut self, scale_factor: f64) {
    //     self.set_content_scale(Vector::splat(scale_factor as f32));
    // }

    fn run_frame(&mut self, inputs: &InputState) {
        let dt = self.frame_clock.read();
        self.frame_clock.reset();
        self.fps_tracker[self.fps_tracker_index] = 1.0 / dt;
        self.fps_tracker_index = (self.fps_tracker_index + 1) % self.fps_tracker.len();

        if inputs.key_is_held(KeyCode::ControlLeft) {
            if inputs.key_was_pressed(KeyCode::KeyR) {
                match self.assets.reload(&self.gfx) {
                    Err(err) => eprintln!("Failed to reload assets: {err}"),
                    Ok(()) => println!("Reloaded assets."),
                }
                if let Err(err) = self.gui.reload_assets(&mut self.assets) {
                    eprintln!("Failed to reload assets: {err}");
                }
            }
        }

        let cursor_position = inputs.cursor_position().map(|position| {
            position.map(|x| x as f32)
        });
        let left_held = inputs.button_is_held(MouseButton::Left);
        let right_held = inputs.button_is_held(MouseButton::Right);
        let middle_held = inputs.button_is_held(MouseButton::Middle);

        self.gui.set_cursor_position(cursor_position);

        if let Some(world) = &mut self.current_world {
            if let Some(scroll_amount) = inputs.scroll_delta() {
                let target_zoom = world.camera().zoom().mul(f32::powf(1.125, scroll_amount.y() as f32));
                world.camera_mut().set_zoom(target_zoom);
            }

            let cursor_world_pos = cursor_position.map(|position| world.camera().get_world_pos(position));

            if inputs.key_was_repeated(KeyCode::Tab) {
                let offset = if inputs.key_is_held(KeyCode::ShiftLeft) { -1 } else { 1 };
                let held_item_type = world.player().held_item().item_type();
                let item_index = ITEM_TYPES
                    .iter()
                    .position(|&item_type| item_type == held_item_type)
                    .unwrap();
                let next_item_index = (item_index as isize + offset).rem_euclid(ITEM_TYPES.len() as isize) as usize;
                world.player_mut().set_held_item(Item::with_max_count(ITEM_TYPES[next_item_index]));
            }
            if inputs.key_was_pressed(KeyCode::F4) {
                let current_mode = world.player().mode();
                world.player_mut().set_mode(match current_mode {
                    PlayerMode::Normal => PlayerMode::Spectating,
                    PlayerMode::Spectating => PlayerMode::Normal,
                });
            }
            // TODO: actually handle the part where these should capture input
            self.gui.handle_cursor(inputs, &self.scripting, world, &self.assets);
            self.gui.handle_keyboard(inputs, &self.scripting, world, &self.assets);

            if cursor_world_pos.is_some() && left_held || right_held || middle_held {
                let cursor_world_pos = cursor_world_pos.unwrap();

                let chunk_location = Vector([
                    cursor_world_pos.x().div_euclid(CHUNK_SIZE as f32) as i64,
                    cursor_world_pos.y().div_euclid(CHUNK_SIZE as f32) as i64,
                ]);
                let block_x = cursor_world_pos.x().rem_euclid(CHUNK_SIZE as f32) as usize;
                let block_y = cursor_world_pos.y().rem_euclid(CHUNK_SIZE as f32) as usize;

                if self.last_block_pos.is_none_or(|pos| pos != (block_x, block_y)) {
                    self.last_block_pos = Some((block_x, block_y));
                    if middle_held {
                        let block_type = world
                            .get_chunk(chunk_location)
                            .map_or(&AIR, |chunk| {
                                chunk.block_at(block_x, block_y).block_type()
                            });
                        if let Some(item_type) = block_type.item_type() {
                            world.player_mut().set_held_item(Item::new(
                                item_type,
                                item_type.max_count(),
                            ));
                        }
                    }
                    if left_held {
                        world.user_destroy_block(
                            chunk_location,
                            block_x,
                            block_y,
                            &mut self.assets,
                            &self.audio,
                        );
                    }
                    if right_held {
                        world.player_use_item(
                            chunk_location,
                            block_x,
                            block_y,
                            BlockSide::from_position(cursor_world_pos),
                            &self.assets,
                            &self.audio,
                        );
                    }
                }
            }
            else {
                self.last_block_pos = None;
            }

            world.set_block_preview_position(cursor_world_pos);
            world.update(&self.gfx, inputs, dt);

            self.gui.update_item_display(world.player().held_item(), &self.assets);
            self.gui.update_player_info_display(
                world.player().position(),
                world.player().velocity(),
            );
            let min_fps = self.fps_tracker.into_iter().reduce(f32::min).unwrap_or(f32::NAN);
            if min_fps.is_finite() {
                self.gui.update_fps_display(min_fps);
            }

            self.gfx.set_clear_color(world.sky_color());
        }
        else {
            self.gfx.set_clear_color(Color::Black);
        }

        self.gfx.render(|render_pass| {
            if let Some(world) = &mut self.current_world {
                world.render(render_pass, &self.assets);
            }
            self.gui.render(render_pass, &mut self.assets);
        });
    }
}
