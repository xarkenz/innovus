use glfw::{Context, SwapInterval, WindowEvent, WindowMode};
use innovus::tools::Vector;
use crate::game::Game;
use crate::tools::input::InputState;
use crate::world::gen::types::OverworldGenerator;

pub mod tools;
pub mod world;
pub mod game;
pub mod gui;
mod audio;

fn main() {
    let mut glfw = {
        use glfw::fail_on_errors;
        glfw::init(glfw::fail_on_errors!()).unwrap()
    };

    let (mut window, event_receiver) = glfw.create_window(
        1200,
        800,
        "2D Sandbox Game",
        WindowMode::Windowed,
    ).unwrap();

    window.make_current();
    window.maximize();
    window.set_framebuffer_size_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_mouse_button_polling(true);
    window.set_key_polling(true);
    window.set_scroll_polling(true);

    glfw.set_swap_interval(SwapInterval::Sync(1));
    innovus::gfx::screen::bind_glfw(&glfw);

    let mut input_state = InputState::new();

    let viewport_size = {
        let (width, height) = window.get_framebuffer_size();
        Vector([width as f32, height as f32])
    };
    let _content_scale = {
        let (x, y) = window.get_content_scale();
        Vector([x, y])
    };

    let mut game = Game::start("src/javagame/assets", viewport_size, Vector::one()).unwrap();
    game.enter_world(Some(Box::new(OverworldGenerator::new(0))));

    while !window.should_close() {
        input_state.reset();

        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&event_receiver) {
            match event {
                WindowEvent::FramebufferSize(width, height) => {
                    game.set_viewport_size(Vector([width as f32, height as f32]));
                }
                WindowEvent::Key(key, _scancode, action, mods) => {
                    input_state.handle_key(key, action, mods);
                }
                WindowEvent::MouseButton(button, action, mods) => {
                    input_state.handle_mouse_button(button, action, mods);
                }
                WindowEvent::CursorPos(x, y) => {
                    input_state.handle_cursor_pos(x, y);
                }
                WindowEvent::Scroll(dx, dy) => {
                    input_state.handle_scroll(dx, dy);
                }
                _ => {}
            }
        }

        game.run_frame(&input_state);

        window.swap_buffers();
    }
}
