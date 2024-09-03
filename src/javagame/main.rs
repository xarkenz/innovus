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
    current_world.put_chunk(current_world.generate_chunk(Vector([0, -1])));
    current_world.put_chunk(current_world.generate_chunk(Vector([-1, -1])));
    let test_box = world::entity::types::TestBox::new(
        current_world.physics_mut(),
        phys::Collider::new(
            Rectangle::new(Vector([10.0, 4.0]), Vector([12.0, 6.0])),
            Vector([-16.0, 4.0]),
        ),
        [0.0, 0.0, 1.0, 1.0],
        true,
    );
    current_world.add_entity(Box::new(test_box));
    for y in 0..16 {
        for x in 0..16 {
            let test_box = world::entity::types::TestBox::new(
                current_world.physics_mut(),
                phys::Collider::new_fixed(
                    Rectangle::new(Vector([-x as f32, y as f32]), Vector([0.5 - x as f32, 0.5 + y as f32])),
                    // Vector([-0.0625 * (x + y - 16) as f32, -0.0625 * (x + y - 16) as f32]),
                ),
                [0.125 + 0.03125 * x as f32, 0.125 + 0.03125 * y as f32, 0.0, 1.0],
                false,
            );
            current_world.add_entity(Box::new(test_box));
        }
    }

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

    let mut collision_map: HashMap<(i64, i64), Uuid> = HashMap::new();

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
            let chunk_pos = Vector([
                world_pos.x().div_euclid(world::block::CHUNK_SIZE as f32) as i64,
                world_pos.y().div_euclid(world::block::CHUNK_SIZE as f32) as i64,
            ]);
            let block_x = world_pos.x().rem_euclid(world::block::CHUNK_SIZE as f32) as usize;
            let block_y = world_pos.y().rem_euclid(world::block::CHUNK_SIZE as f32) as usize;
            let chunk = current_world.force_get_chunk_mut(chunk_pos);
            if chunk.block_at(block_x, block_y).block_type != &world::block::types::AIR {
                chunk.set_block_at(
                    block_x,
                    block_y,
                    world::block::Block::new(&world::block::types::AIR, 0, 15),
                );

                for dy in [-1, 0, 1] {
                    for dx in [-1, 0, 1] {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        let dirty_chunk_pos = chunk_pos + Vector([
                            (block_x as i64 + dx).div_euclid(world::block::CHUNK_SIZE as i64),
                            (block_y as i64 + dy).div_euclid(world::block::CHUNK_SIZE as i64),
                        ]);
                        let dirty_block_x = (block_x as i64 + dx).rem_euclid(world::block::CHUNK_SIZE as i64) as usize;
                        let dirty_block_y = (block_y as i64 + dy).rem_euclid(world::block::CHUNK_SIZE as i64) as usize;
                        if let Some(chunk) = current_world.get_chunk_mut(dirty_chunk_pos) {
                            chunk.set_dirty_at(dirty_block_x, dirty_block_y, true);
                        }
                    }
                }

                let key = (world_pos.x().floor() as i64, world_pos.y().floor() as i64);
                if let Some(uuid) = collision_map.remove(&key) {
                    current_world.destroy_entity(uuid);
                }
            }
        }

        if window.get_mouse_button(input::MouseButtonRight) == Action::Press {
            let cursor_pos = Vector([input_state.cursor_pos().x() as f32, input_state.cursor_pos().y() as f32]);
            let world_pos = camera.get_world_pos(cursor_pos);
            let chunk_pos = Vector([
                world_pos.x().div_euclid(world::block::CHUNK_SIZE as f32) as i64,
                world_pos.y().div_euclid(world::block::CHUNK_SIZE as f32) as i64,
            ]);
            let block_x = world_pos.x().rem_euclid(world::block::CHUNK_SIZE as f32) as usize;
            let block_y = world_pos.y().rem_euclid(world::block::CHUNK_SIZE as f32) as usize;
            let chunk = current_world.force_get_chunk_mut(chunk_pos);
            if chunk.block_at(block_x, block_y).block_type == &world::block::types::AIR {
                chunk.set_block_at(
                    block_x,
                    block_y,
                    world::block::Block::new(world::block::types::BLOCK_TYPES[selected_block_index], 0, 15),
                );

                for dy in [-1, 0, 1] {
                    for dx in [-1, 0, 1] {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        let dirty_chunk_pos = chunk_pos + Vector([
                            (block_x as i64 + dx).div_euclid(world::block::CHUNK_SIZE as i64),
                            (block_y as i64 + dy).div_euclid(world::block::CHUNK_SIZE as i64),
                        ]);
                        let dirty_block_x = (block_x as i64 + dx).rem_euclid(world::block::CHUNK_SIZE as i64) as usize;
                        let dirty_block_y = (block_y as i64 + dy).rem_euclid(world::block::CHUNK_SIZE as i64) as usize;
                        if let Some(chunk) = current_world.get_chunk_mut(dirty_chunk_pos) {
                            chunk.set_dirty_at(dirty_block_x, dirty_block_y, true);
                        }
                    }
                }

                let pos = Vector([world_pos.x().floor(), world_pos.y().floor()]);
                let test_box = world::entity::types::TestBox::new(
                    current_world.physics_mut(),
                    phys::Collider::new_fixed(
                        Rectangle::new(pos, pos + Vector([1.0, 1.0])),
                    ),
                    [0.0; 4],
                    false,
                );
                let key = (pos.x() as i64, pos.y() as i64);
                use world::entity::Entity;
                collision_map.insert(key, test_box.uuid());
                current_world.add_entity(Box::new(test_box));
            }
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
