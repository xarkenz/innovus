use std::collections::VecDeque;
use std::path::Path;
use glfw::{Key, Window};
use innovus::gfx::color::Color;
use innovus::gfx::screen;
use innovus::tools::{Clock, Vector};
use crate::audio::AudioEngine;
use crate::gui::{GuiManager, GuiRequest, GuiResponse, GuiResponseQueue};
use crate::tools::asset::AssetPool;
use crate::tools::input::InputState;
use crate::world::camera::Camera;
use crate::world::gen::WorldGenerator;
use crate::world::{World, WorldRequest, WorldResponse, WorldResponseQueue};

pub struct Game<'world> {
    frame_clock: Clock,
    fps_tracker: [f32; 120],
    fps_tracker_index: usize,
    viewport_size: Vector<f32, 2>,
    content_scale: Vector<f32, 2>,
    assets: AssetPool,
    gui: GuiManager,
    gui_responses: GuiResponseQueue,
    audio: AudioEngine,
    current_world: Option<World<'world>>,
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
            gui_responses: GuiResponseQueue::new(),
            assets,
            audio: AudioEngine::new()?,
            current_world: None,
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
        let _ = window;
        let dt = self.frame_clock.read();
        self.frame_clock.reset();
        self.fps_tracker[self.fps_tracker_index] = 1.0 / dt;
        self.fps_tracker_index = (self.fps_tracker_index + 1) % self.fps_tracker.len();

        let mut world_responses = WorldResponseQueue::new();

        if inputs.key_is_held(Key::LeftControl) {
            if inputs.key_was_pressed(Key::R) {
                match self.assets.reload() {
                    Err(err) => eprintln!("Failed to reload assets: {err}"),
                    Ok(()) => println!("Reloaded assets."),
                }
                self.gui.update(GuiRequest::ReloadAssets(&mut self.assets), &mut self.gui_responses);
                if let Some(world) = &mut self.current_world {
                    world.update(WorldRequest::ReloadAssets(&mut self.assets), &mut world_responses);
                }
            }
        }

        self.gui.update(GuiRequest::HandleMouse(inputs), &mut self.gui_responses);
        self.gui.update(GuiRequest::HandleKeyboard(inputs), &mut self.gui_responses);

        // Flush GUI responses, including those from the previous frame
        let mut world_requests = VecDeque::new();
        while let Some(response) = self.gui_responses.pop_front() {
            match response {
                GuiResponse::WorldHandleMouse => {
                    world_requests.push_back(WorldRequest::HandleMouse(inputs));
                }
                GuiResponse::WorldHandleKeyboard => {
                    world_requests.push_back(WorldRequest::HandleKeyboard(inputs));
                }
                GuiResponse::PlayerChat { content } => {
                    world_requests.push_back(WorldRequest::PlayerChat {
                        content,
                    });
                }
                GuiResponse::PlaySound { path } => {
                    self.audio.play_sound(self.assets.resolve_path(&path)).ok();
                }
                GuiResponse::AssetError { message } => {
                    eprintln!("Failed to load assets: {message}");
                }
            }
        }

        let clear_color;
        if let Some(world) = &mut self.current_world {
            for request in world_requests {
                world.update(request, &mut world_responses);
            }
            world.update(WorldRequest::RunFrame {
                dt,
                assets: &mut self.assets,
            }, &mut world_responses);

            for response in world_responses {
                match response {
                    WorldResponse::Inventory { items } => {
                        let _ = items;
                        todo!()
                    }
                    WorldResponse::PlayerInfo { position, velocity, held_item } => {
                        self.gui.update(GuiRequest::DebugPlayerInfo {
                            position,
                            velocity,
                        }, &mut self.gui_responses);
                        self.gui.update(GuiRequest::HeldItem(held_item, &self.assets), &mut self.gui_responses);
                    }
                    WorldResponse::ChatMessage { content, color } => {
                        self.gui.update(GuiRequest::ChatMessage {
                            content,
                            color,
                            assets: &self.assets,
                        }, &mut self.gui_responses);
                    }
                    WorldResponse::PlaySound { path } => {
                        self.audio.play_sound(self.assets.resolve_path(&path)).ok();
                    }
                }
            }

            clear_color = world.sky_color();
        }
        else {
            clear_color = Color::Black;
        }

        let min_fps = self.fps_tracker.into_iter().reduce(f32::min).unwrap_or(f32::NAN);
        if min_fps.is_finite() {
            self.gui.update(GuiRequest::DebugFPS(min_fps), &mut self.gui_responses);
        }

        screen::set_clear_color(clear_color.into());
        screen::clear();

        if let Some(world) = &mut self.current_world {
            world.render(&self.assets);
        }
        self.gui.render(&mut self.assets);
    }
}
