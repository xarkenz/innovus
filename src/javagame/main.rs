extern crate glfw;

use crate::{tools::*, world::gen::WorldGenerator};
use innovus::{gfx::*, *};

pub mod tools;
pub mod view;
pub mod world;

fn main() {
    let mut application = Application::new().unwrap();
    application.set_multisampling(Some(8));
    let (mut window, event_receiver) = application
        .create_window(1200, 800, "Even More Rust Gaming.", WindowMode::Windowed)
        .expect("failed to create GLFW window.");
    window.maximize();
    window.set_size_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_mouse_button_polling(true);
    window.set_key_polling(true);

    let shaders = Program::from_preset(ProgramPreset::Default2DShader).unwrap();
    screen::set_clear_color(0.6, 0.8, 1.0);
    screen::set_blend(screen::Blend::Transparency);
    screen::set_viewport(0, 0, 1200, 800);

    let mut input_state = input::InputState::new(event_receiver);

    let clock = Clock::start();
    let mut prev_time = clock.read();
    let (mut prev_x, mut prev_y) = (0.0f32, 0.0f32);

    let mut current_world = world::World::new(Some(Box::new(
        world::gen::types::OverworldGenerator::new(0),
    )));
    let mut camera = view::Camera::new(
        Vector([0.0, 0.0]),
        Vector({
            let (w, h) = window.get_size();
            [w as f32, h as f32]
        }),
    );

    while !window.should_close() {
        for event in input_state.process_events(&mut application) {
            match event {
                WindowEvent::Size(w, h) => {
                    camera.set_size(Vector([w as f32, h as f32]));
                    screen::set_viewport(0, 0, w as usize, h as usize);
                }
                WindowEvent::CursorPos(x, y) => {
                    let (x, y) = (x as f32, y as f32);
                    let (dx, dy) = (prev_x - x, prev_y - y);
                    if window.get_mouse_button(glfw::MouseButtonLeft) == Action::Press {
                        // drag action
                    }
                    prev_x = x;
                    prev_y = y;
                }
                _ => {}
            }
        }

        let time = clock.read();
        let dt = time - prev_time;
        prev_time = time;

        current_world.put_chunk(current_world.generate_chunk(Vector([0, -1])));
        current_world.put_chunk(current_world.generate_chunk(Vector([-1, -1])));

        current_world.update(&input_state, dt);
        camera.update(dt);

        shaders.set("camera_view", camera.view());
        shaders.set("camera_proj", camera.projection());
        //shaders.set("tex_atlas", &test_tex);

        screen::clear();
        current_world.render(dt);
        window.swap_buffers();
    }
}
