extern crate glfw;

use std::collections::HashMap;
use crate::{tools::*, world::gen::WorldGenerator};
use innovus::{gfx::*, *};
use crate::world::block::BlockRenderer;

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
    window.set_scroll_polling(true);

    let shader_program = Program::from_preset(ProgramPreset::Default2DShader).unwrap();
    screen::set_clear_color(0.6, 0.8, 1.0);
    screen::set_blend(screen::Blend::Transparency);

    let mut input_state = input::InputState::new(event_receiver);

    let clock = Clock::start();
    let mut prev_time = clock.read();

    let mut current_world = world::World::new(Some(Box::new(
        world::gen::types::OverworldGenerator::new(0),
    )));
    current_world.force_get_chunk(Vector([0, -1]));
    current_world.force_get_chunk(Vector([-1, -1]));
    // let test_box = world::entity::types::TestBox::new(
    //     current_world.physics_mut(),
    //     phys::Collider::new(
    //         Rectangle::new(Vector([10.0, 4.0]), Vector([12.0, 6.0])),
    //         Vector([-1.0, -4.0]),
    //     ),
    //     [0.0, 0.0, 1.0, 1.0],
    //     true,
    // );
    // current_world.add_entity(Box::new(test_box));
    let player = world::entity::types::Player::new(
        current_world.physics_mut(),
        Vector([0.0, 0.0]),
        None,
    );
    current_world.add_entity(Box::new(player));

    let mut camera = view::Camera::new(
        Vector([0.0, 0.0]),
        Vector({
            let (width, height) = window.get_size();
            screen::set_viewport(0, 0, width, height);
            [width as f32, height as f32]
        }),
    );

    let block_renderer = BlockRenderer::new().unwrap();

    let mut selected_block_index = 0;

    while !window.should_close() {
        for event in input_state.process_events(&mut application) {
            match event {
                WindowEvent::Size(width, height) => {
                    camera.set_size(Vector([width as f32, height as f32]));
                    screen::set_viewport(0, 0, width, height);
                }
                WindowEvent::Key(Key::Tab, _, Action::Press, _) => {
                    selected_block_index += 1;
                    selected_block_index %= world::block::types::BLOCK_TYPES.len();
                }
                WindowEvent::Scroll(dx, dy) => {
                    let _ = dx;
                    camera.set_zoom(camera.zoom() * 1.125_f32.powf(dy as f32));
                }
                _ => {}
            }
        }

        let time = clock.read();
        let dt = time - prev_time;
        prev_time = time;

        if window.get_mouse_button(input::MouseButtonLeft) == Action::Press {
            let cursor_pos = Vector([input_state.cursor_pos().x() as f32, input_state.cursor_pos().y() as f32]);
            let world_pos = camera.get_world_pos(cursor_pos);
            let chunk_location = Vector([
                world_pos.x().div_euclid(world::block::CHUNK_SIZE as f32) as i64,
                world_pos.y().div_euclid(world::block::CHUNK_SIZE as f32) as i64,
            ]);
            let block_x = world_pos.x().rem_euclid(world::block::CHUNK_SIZE as f32) as usize;
            let block_y = world_pos.y().rem_euclid(world::block::CHUNK_SIZE as f32) as usize;
            current_world.user_destroy_block(chunk_location, block_x, block_y);
        }

        if window.get_mouse_button(input::MouseButtonRight) == Action::Press {
            let cursor_pos = Vector([input_state.cursor_pos().x() as f32, input_state.cursor_pos().y() as f32]);
            let world_pos = camera.get_world_pos(cursor_pos);
            let chunk_location = Vector([
                world_pos.x().div_euclid(world::block::CHUNK_SIZE as f32) as i64,
                world_pos.y().div_euclid(world::block::CHUNK_SIZE as f32) as i64,
            ]);
            let block_x = world_pos.x().rem_euclid(world::block::CHUNK_SIZE as f32) as usize;
            let block_y = world_pos.y().rem_euclid(world::block::CHUNK_SIZE as f32) as usize;
            current_world.user_place_block(chunk_location, block_x, block_y, world::block::types::BLOCK_TYPES[selected_block_index]);
        }

        current_world.update(&input_state, dt);
        camera.update(dt);

        shader_program.set_uniform("camera_view", camera.view());
        shader_program.set_uniform("camera_proj", camera.projection());

        screen::clear();
        current_world.render(dt, &block_renderer);
        window.swap_buffers();
    }
}
