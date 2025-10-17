use std::path::Path;
use glfw::{Key, MouseButtonLeft, MouseButtonMiddle, MouseButtonRight, Window};
use innovus::gfx::color::RGBColor;
use innovus::gfx::screen;
use innovus::tools::{Clock, Vector};
use crate::audio::AudioEngine;
use crate::gui::GuiManager;
use crate::tools::asset::AssetPool;
use crate::tools::input::InputState;
use crate::world::camera::Camera;
use crate::world::block::{BlockSide, CHUNK_SIZE};
use crate::world::block::types::AIR;
use crate::world::entity::Entity;
use crate::world::entity::types::player::PlayerMode;
use crate::world::gen::WorldGenerator;
use crate::world::item::{Item, ITEM_TYPES};
use crate::world::World;

pub struct Game<'world> {
    frame_clock: Clock,
    fps_tracker: [f32; 120],
    fps_tracker_index: usize,
    viewport_size: Vector<f32, 2>,
    content_scale: Vector<f32, 2>,
    assets: AssetPool,
    gui: GuiManager,
    audio: AudioEngine,
    current_world: Option<World<'world>>,
    last_block_pos: Option<(usize, usize)>,
}

impl<'world> Game<'world> {
    pub fn start(assets_path: impl AsRef<Path>, viewport_size: Vector<f32, 2>, content_scale: Vector<f32, 2>) -> Result<Self, String> {
        screen::set_blend_func(screen::BlendFunc::Transparency);

        let mut assets = AssetPool::load(assets_path)?;
        let mut game = Self {
            frame_clock: Clock::start(),
            fps_tracker: [f32::INFINITY; 120],
            fps_tracker_index: 0,
            viewport_size,
            content_scale,
            gui: GuiManager::new(viewport_size, content_scale, 8.0, &mut assets)?,
            assets,
            audio: AudioEngine::new()?,
            current_world: None,
            last_block_pos: None,
        };
        game.set_viewport_size(viewport_size);
        Ok(game)
    }

    pub fn viewport_size(&self) -> Vector<f32, 2> {
        self.viewport_size
    }

    pub fn set_viewport_size(&mut self, viewport_size: Vector<f32, 2>) {
        self.viewport_size = viewport_size;

        screen::set_viewport(0, 0, viewport_size.x() as i32, viewport_size.y() as i32);
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
            Vector::zero(),
            self.viewport_size,
            self.content_scale.mul(48.0),
            5.0,
        );
        self.current_world = Some(World::new(generator, camera, &mut self.assets));
    }

    pub fn run_frame(&mut self, inputs: &InputState, window: &mut Window) {
        let dt = self.frame_clock.read();
        self.frame_clock.reset();
        self.fps_tracker[self.fps_tracker_index] = 1.0 / dt;
        self.fps_tracker_index = (self.fps_tracker_index + 1) % self.fps_tracker.len();

        if inputs.key_is_held(Key::LeftControl) {
            if inputs.key_was_pressed(Key::V) {
                if let Some(pasted_text) = window.get_clipboard_string() {
                    self.gui.enter_text(&pasted_text);
                }
            }
            if inputs.key_was_pressed(Key::R) {
                match self.assets.reload() {
                    Err(err) => eprintln!("Failed to reload assets: {err}"),
                    Ok(()) => println!("Reloaded assets."),
                }
                if let Err(err) = self.gui.reload_assets(&mut self.assets) {
                    eprintln!("Failed to reload assets: {err}");
                }
            }
        }

        if self.gui.inventory_shown() {
            if inputs.key_was_repeated(Key::Backspace) {
                self.gui.backspace();
            }
            if !inputs.entered_text().is_empty() {
                self.gui.enter_text(inputs.entered_text());
            }
        }

        let cursor_pos = inputs.cursor_pos().map(|x| x as f32);
        let left_held = inputs.button_is_held(MouseButtonLeft);
        let right_held = inputs.button_is_held(MouseButtonRight);
        let middle_held = inputs.button_is_held(MouseButtonMiddle);

        self.gui.set_cursor_position(cursor_pos);

        let clear_color;
        if let Some(world) = &mut self.current_world {
            if let Some(scroll_amount) = inputs.scroll_amount() {
                let target_zoom = world.camera().zoom().mul(f32::powf(1.125, scroll_amount.y() as f32));
                world.camera_mut().set_zoom(target_zoom);
            }

            let cursor_world_pos = world.camera().get_world_pos(cursor_pos);

            if inputs.key_was_repeated(Key::Tab) {
                let offset = if inputs.key_is_held(Key::LeftShift) { -1 } else { 1 };
                let held_item_type = world.player().held_item().item_type();
                let item_index = ITEM_TYPES
                    .iter()
                    .position(|&item_type| item_type == held_item_type)
                    .unwrap();
                let next_item_index = (item_index as isize + offset).rem_euclid(ITEM_TYPES.len() as isize) as usize;
                world.player_mut().set_held_item(Item::new(
                    ITEM_TYPES[next_item_index],
                    ITEM_TYPES[next_item_index].max_count(),
                ));
            }
            if inputs.key_was_pressed(Key::F4) {
                let current_mode = world.player().mode();
                world.player_mut().set_mode(match current_mode {
                    PlayerMode::Normal => PlayerMode::Spectating,
                    PlayerMode::Spectating => PlayerMode::Normal,
                });
            }
            if inputs.key_was_pressed(Key::Escape) {
                self.gui.set_inventory_shown(!self.gui.inventory_shown());
                self.gui.clear_text();
            }
            self.gui.handle_input(inputs);

            if left_held || right_held || middle_held {
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
            world.update(inputs, dt);

            self.gui.update_item_display(world.player().held_item(), &self.assets);
            self.gui.update_player_info_display(
                world.player().position(),
                world.player().velocity(),
            );
            let average_fps = self.fps_tracker.iter().sum::<f32>() / self.fps_tracker.len() as f32;
            if average_fps.is_finite() {
                self.gui.update_fps_display(average_fps);
            }

            clear_color = world.sky_color();
        }
        else {
            clear_color = RGBColor::black();
        }

        screen::set_clear_color(clear_color);
        screen::clear();

        if let Some(world) = &mut self.current_world {
            world.render(&self.assets);
        }
        self.gui.render(&mut self.assets);
    }
}
