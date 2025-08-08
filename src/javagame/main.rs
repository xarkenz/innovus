use innovus::{gfx::*, *};
use crate::tools::*;

pub mod tools;
pub mod view;
pub mod world;

fn main() {
    let mut application = Application::new().unwrap();
    //application.set_multisampling(Some(8));
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
    shader_program.set_uniform("tex_atlas", 0_u32);
    screen::set_clear_color(0.6, 0.8, 1.0);
    screen::set_blend(screen::Blend::Transparency);

    let mut input_state = input::InputState::new(event_receiver);

    let clock = Clock::start();
    let mut prev_time = clock.read();

    let mut assets = asset::AssetPool::new().unwrap();

    let mut current_world = world::World::new(Some(Box::new(
        world::gen::types::OverworldGenerator::new(0),
    )));
    current_world.force_get_chunk(Vector([0, -1]));
    current_world.force_get_chunk(Vector([-1, -1]));
    let player_start = Vector([0.0, 0.0]);
    let player = world::entity::types::Player::new(
        player_start,
        None,
    );
    let player_uuid = world::entity::Entity::uuid(&player);
    current_world.add_entity(Box::new(player), &mut assets);

    let mut camera = view::Camera::new(
        player_start,
        Vector({
            let (width, height) = window.get_size();
            screen::set_viewport(0, 0, width, height);
            [width as f32, height as f32]
        }),
    );

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
        // Move camera towards player
        camera.set_target(current_world.get_entity(player_uuid).unwrap().position());
        camera.update(dt);

        shader_program.set_uniform("camera_view", camera.view());
        shader_program.set_uniform("camera_proj", camera.projection());

        screen::clear();
        current_world.render(dt, &mut assets);
        window.swap_buffers();
    }
}
