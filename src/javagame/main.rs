use glfw::{Action, Context, Key, Modifiers, WindowEvent, WindowMode};
use innovus::Application;
use innovus::gfx::{screen, Program, ProgramPreset};
use innovus::tools::{Clock, Vector};
use crate::tools::{asset, input};

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
    screen::set_blend(screen::Blend::Transparency);
    let base_sky_color = Vector([0.6, 0.8, 1.0]);
    let mut sky_light = 1.0;

    let mut input_state = input::InputState::new(event_receiver);

    let clock = Clock::start();
    let mut prev_time = clock.read();

    let mut assets = asset::AssetPool::load("src/javagame/assets").unwrap();

    let mut current_world = world::World::new(
        Some(Box::new(world::gen::types::OverworldGenerator::new(0))),
    );
    let player_start = Vector([0.5, 0.0]);
    let player = world::entity::types::Player::new(
        player_start,
        None,
    );
    let player_uuid = world::entity::Entity::uuid(&player);
    current_world.add_entity(Box::new(player), &mut assets);
    current_world.set_chunk_loader_entity(Some(player_uuid));

    let mut camera = view::Camera::new(
        player_start,
        Vector({
            let (width, height) = window.get_size();
            screen::set_viewport(0, 0, width, height);
            [width as f32, height as f32]
        }),
        64.0,
        5.0,
    );
    let mut block_preview = view::block_preview::BlockPreview::new(
        Vector::zero(),
        &world::block::types::AIR,
        0.4,
    );

    fn select_block_index(mut index: isize, direction: isize) -> usize {
        index = index.rem_euclid(world::block::BLOCK_TYPES.len() as isize);
        if world::block::BLOCK_TYPES[index as usize] == &world::block::types::AIR {
            index += direction;
            index = index.rem_euclid(world::block::BLOCK_TYPES.len() as isize);
        }
        let index = index as usize;
        println!("placing {}", world::block::BLOCK_TYPES[index].name);
        index
    }
    let mut selected_block_index = select_block_index(0, 1);
    let mut last_block_pos = None;

    while !window.should_close() {
        for event in input_state.process_events(&mut application) {
            match event {
                WindowEvent::Size(width, height) => {
                    camera.set_size(Vector([width as f32, height as f32]));
                    screen::set_viewport(0, 0, width, height);
                }
                WindowEvent::Key(Key::Tab, _, Action::Press, mods) => {
                    let offset = if mods.contains(Modifiers::Shift) { -1 } else { 1 };
                    selected_block_index = select_block_index(selected_block_index as isize + offset, offset);
                }
                WindowEvent::Key(Key::R, _, Action::Press, mods) if mods.contains(Modifiers::Control) => {
                    if let Err(err) = assets.reload_block_appearances() {
                        eprintln!("failed to reload: {err}");
                    }
                    else {
                        assets.clear_entity_images();
                        println!("reloaded");
                    }
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

        let cursor_pos = input_state.cursor_pos().map(|x| x as f32);
        let world_pos = camera.get_world_pos(cursor_pos);
        let left_held = input_state.mouse_button_is_held(input::MouseButtonLeft);
        let right_held = input_state.mouse_button_is_held(input::MouseButtonRight);
        let middle_held = input_state.mouse_button_is_held(input::MouseButtonMiddle);
        if left_held || right_held || middle_held {
            let chunk_location = Vector([
                world_pos.x().div_euclid(world::block::CHUNK_SIZE as f32) as i64,
                world_pos.y().div_euclid(world::block::CHUNK_SIZE as f32) as i64,
            ]);
            let block_x = world_pos.x().rem_euclid(world::block::CHUNK_SIZE as f32) as usize;
            let block_y = world_pos.y().rem_euclid(world::block::CHUNK_SIZE as f32) as usize;

            if last_block_pos.is_none_or(|pos| pos != (block_x, block_y)) {
                last_block_pos = Some((block_x, block_y));
                if middle_held {
                    let block_type = current_world
                        .get_chunk(chunk_location)
                        .map_or(&world::block::types::AIR, |chunk| {
                            chunk.block_at(block_x, block_y).block_type()
                        });
                    if block_type != &world::block::types::AIR {
                        let block_index = world::block::BLOCK_TYPES.iter().position(|&element| element == block_type).unwrap();
                        selected_block_index = select_block_index(block_index as isize, 1);
                    }
                }
                if left_held {
                    current_world.user_destroy_block(chunk_location, block_x, block_y);
                }
                if right_held {
                    current_world.user_place_block(chunk_location, block_x, block_y, world::block::BLOCK_TYPES[selected_block_index]);
                }
            }
        }
        else {
            last_block_pos = None;
        }

        current_world.update(&input_state, dt);
        // Update block preview
        block_preview.set_position(world_pos);
        block_preview.set_block_type(world::block::BLOCK_TYPES[selected_block_index]);
        // Move camera towards player
        camera.set_target(current_world.get_entity(player_uuid).unwrap().position());
        camera.update(dt);

        shader_program.set_uniform("camera_view", camera.view());
        shader_program.set_uniform("camera_proj", camera.projection());

        let target_sky_light = {
            let camera_pos = camera.position();
            let chunk_location = Vector([
                camera_pos.x().div_euclid(world::block::CHUNK_SIZE as f32) as i64,
                camera_pos.y().div_euclid(world::block::CHUNK_SIZE as f32) as i64,
            ]);
            let block_x = camera_pos.x().rem_euclid(world::block::CHUNK_SIZE as f32) as usize;
            let block_y = camera_pos.y().rem_euclid(world::block::CHUNK_SIZE as f32) as usize;
            if let Some(chunk) = current_world.get_chunk(chunk_location) {
                world::block::slot_light_value(chunk.block_slot_at(block_x, block_y))
            }
            else {
                world::block::light_value(15)
            }
        };
        sky_light += (target_sky_light - sky_light) * dt.min(1.0);
        let sky_color = base_sky_color * sky_light;

        screen::set_clear_color(sky_color.x(), sky_color.y(), sky_color.z());
        screen::clear();
        current_world.render_block_layer(&mut assets);
        block_preview.render(&assets, &current_world);
        current_world.render_entity_layer(&mut assets);
        window.swap_buffers();
    }
}
