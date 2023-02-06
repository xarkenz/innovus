extern crate glfw;

use crate::world::gen::WorldGenerator;
use innovus::{gfx::*, tools::*, *};

pub mod tools;
pub mod view;
pub mod world;

fn main() {
    let mut application = Application::new().unwrap();
    application.set_multisampling(Some(8));
    let (mut window, events) = application
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
        application.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
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
                WindowEvent::MouseButton(button, action, _mods) => match action {
                    Action::Press => match button {
                        MouseButton::Button1 => {}
                        _ => {}
                    },
                    _ => {}
                },
                WindowEvent::Key(key, _scancode, action, _mods) => match action {
                    Action::Press => match key {
                        Key::Escape => window.set_should_close(true),
                        Key::Enter => {}
                        Key::Up | Key::W => {}
                        Key::Left | Key::A => {}
                        Key::Down | Key::S => {}
                        Key::Right | Key::D => {}
                        _ => {}
                    },
                    _ => {}
                },
                _ => {}
            }
        }

        let time = clock.read();
        let dt = time - prev_time;
        prev_time = time;

        current_world.put_chunk(current_world.generate_chunk(Vector([0, -1])));
        current_world.put_chunk(current_world.generate_chunk(Vector([-1, -1])));

        current_world.update(dt);
        camera.update(dt);

        shaders.set("camera_view", camera.view());
        shaders.set("camera_proj", camera.projection());
        //shaders.set("tex_atlas", &test_tex);

        screen::clear();
        current_world.render();
        window.swap_buffers();
    }
}
