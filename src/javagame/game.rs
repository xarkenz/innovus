use std::path::Path;
use glfw::{Key, MouseButtonLeft, MouseButtonMiddle, MouseButtonRight};
use innovus::gfx::color::RGBColor;
use innovus::gfx::screen;
use innovus::tools::{Clock, Vector};
use crate::gui::GuiManager;
use crate::tools::asset::AssetPool;
use crate::tools::input::InputState;
use crate::view::Camera;
use crate::world::block::{BLOCK_TYPES, CHUNK_SIZE};
use crate::world::entity::types::player::PlayerMode;
use crate::world::gen::WorldGenerator;
use crate::world::World;

fn select_block_index(mut index: isize, direction: isize) -> usize {
    index = index.rem_euclid(BLOCK_TYPES.len() as isize);
    if BLOCK_TYPES[index as usize] == &crate::world::block::types::AIR {
        index += direction;
        index = index.rem_euclid(BLOCK_TYPES.len() as isize);
    }
    let index = index as usize;
    index
}

pub struct Game<'world> {
    frame_clock: Clock,
    fps_tracker: [f32; 120],
    fps_tracker_index: usize,
    viewport_size: Vector<f32, 2>,
    content_scale: Vector<f32, 2>,
    assets: AssetPool,
    gui: GuiManager,
    current_world: Option<World<'world>>,
    selected_block_index: usize,
    last_block_pos: Option<(usize, usize)>,
}

impl<'world> Game<'world> {
    pub fn start(assets_path: impl AsRef<Path>, viewport_size: Vector<f32, 2>, content_scale: Vector<f32, 2>) -> Result<Self, String> {
        screen::set_blend(screen::Blend::Transparency);

        let mut game = Self {
            frame_clock: Clock::start(),
            fps_tracker: [f32::INFINITY; 120],
            fps_tracker_index: 0,
            viewport_size,
            content_scale,
            assets: AssetPool::load(assets_path)?,
            gui: GuiManager::new(viewport_size, content_scale, 8.0),
            current_world: None,
            selected_block_index: select_block_index(0, 1),
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
            self.content_scale * 48.0,
            5.0,
        );
        self.current_world = Some(World::new(generator, camera, &mut self.assets));
    }

    pub fn run_frame(&mut self, inputs: &InputState) {
        let dt = self.frame_clock.read();
        self.frame_clock.reset();
        self.fps_tracker[self.fps_tracker_index] = 1.0 / dt;
        self.fps_tracker_index = (self.fps_tracker_index + 1) % self.fps_tracker.len();

        if inputs.key_was_pressed(Key::R) && inputs.key_is_held(Key::LeftControl) {
            match self.assets.reload() {
                Err(err) => eprintln!("Failed to reload assets: {err}"),
                Ok(()) => println!("Reloaded assets."),
            }
        }

        let cursor_pos = inputs.cursor_pos().map(|x| x as f32);
        let left_held = inputs.mouse_button_is_held(MouseButtonLeft);
        let right_held = inputs.mouse_button_is_held(MouseButtonRight);
        let middle_held = inputs.mouse_button_is_held(MouseButtonMiddle);

        let clear_color;
        if let Some(world) = &mut self.current_world {
            if let Some(scroll_amount) = inputs.scroll_amount() {
                let target_zoom = world.camera().zoom() * 1.125_f32.powf(scroll_amount.y() as f32);
                world.camera_mut().set_zoom(target_zoom);
            }

            let cursor_world_pos = world.camera().get_world_pos(cursor_pos);

            if inputs.key_was_pressed(Key::Tab) {
                let offset = if inputs.key_is_held(Key::LeftShift) { -1 } else { 1 };
                self.selected_block_index = select_block_index(self.selected_block_index as isize + offset, offset);
            }
            if inputs.key_was_pressed(Key::F4) {
                let current_mode = world.player().mode();
                world.player_mut().set_mode(match current_mode {
                    PlayerMode::Normal => PlayerMode::Spectating,
                    PlayerMode::Spectating => PlayerMode::Normal,
                });
            }

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
                            .map_or(&crate::world::block::types::AIR, |chunk| {
                                chunk.block_at(block_x, block_y).block_type()
                            });
                        if block_type != &crate::world::block::types::AIR {
                            let block_index = BLOCK_TYPES
                                .iter()
                                .position(|&element| element == block_type)
                                .unwrap();
                            self.selected_block_index = select_block_index(block_index as isize, 1);
                        }
                    }
                    if left_held {
                        world.user_destroy_block(chunk_location, block_x, block_y, &mut self.assets);
                    }
                    if right_held {
                        world.user_place_block(chunk_location, block_x, block_y, BLOCK_TYPES[self.selected_block_index]);
                    }
                }
            }
            else {
                self.last_block_pos = None;
            }

            let held_item = BLOCK_TYPES[self.selected_block_index];
            self.gui.update_item_display(held_item, &self.assets);
            world.player_mut().set_held_item(held_item);
            world.set_cursor_pos(cursor_world_pos);
            world.update(inputs, dt);

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
