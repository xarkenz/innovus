#![allow(unused_imports)]
extern crate glfw;
extern crate gl;
extern crate image;

pub mod util;
pub mod gfx;
pub mod scene;

pub mod snek;

use glfw::{Action, Context, Key, SwapInterval, WindowHint};
use gfx::{Shader, ShaderType, Program, Geometry, Vertex, GeometrySlice, Image, Texture};
use util::{Vector, Transform3D, Clock};


fn main() {
    snek::run_game();
}

/*fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    //glfw.window_hint(WindowHint::Samples(Some(8)));

    let (mut window, events) = glfw.create_window(800, 800, "Rust Gaming.", glfw::WindowMode::Windowed).expect("failed to create GLFW window.");
    window.maximize();
    window.make_current();
    window.set_size_polling(true);
    window.set_key_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_scroll_polling(true);
    glfw.set_swap_interval(SwapInterval::Sync(1));

    gfx::bind_glfw(&glfw);

    let vert_shader = Shader::create(include_str!("assets/default.v.glsl"), ShaderType::Vertex).unwrap();
    let geom_shader = Shader::create(include_str!("assets/default.g.glsl"), ShaderType::Geometry).unwrap();
    let frag_shader = Shader::create(include_str!("assets/default.f.glsl"), ShaderType::Fragment).unwrap();
    let shader_program = Program::from_shaders(&[vert_shader, geom_shader, frag_shader]).unwrap();

    let test_image = Image::from_file("/home/xarkenz/Projects/Rust/gametest/src/assets/xarkenz_skin.png").unwrap();
    let mut test_tex = Texture::from_image(&test_image).unwrap();
    test_tex.bind(0);

    gfx::set_clear_color(0.1, 0.1, 0.1);
    gfx::set_blend(gfx::Blend::Transparency);
    gfx::set_viewport(0, 0, 800, 800);
    gfx::set_culling(true);
    gfx::set_depth_testing(true);

    let (mut geometry, body) = gen_mc_player_geom();

    let pt_light_pos = Vector::new([10.0, 10.0, 10.0]);
    let pt_light_color = Vector::new([1.0, 1.0, 1.0]);
    let ambient_color = Vector::new([0.5, 0.4, 0.3]);

    let mut camera_pos = Vector::new([0.0, 0.0, 20.0]);
    let mut camera_view = Transform3D::zero();
    let mut camera_proj = Transform3D::zero();
    let (mut pitch, mut yaw) = (0.0_f32, 0.0_f32);
    let (mut width, mut height) = (800_i32, 800_i32);

    const LOOK_SPEED: f32 = 0.005;
    const MAX_MOVE_SPEED: f32 = 5.0;

    let mut clock = Clock::start();
    let mut prev_x: f32 = 0.0;
    let mut prev_y: f32 = 0.0;

    while !window.should_close() {
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::Size(w, h) => {
                    width = w;
                    height = h;
                    gfx::set_viewport(0, 0, w as usize, h as usize);
                },
                glfw::WindowEvent::CursorPos(x, y) => {
                    let (x, y) = (x as f32, y as f32);
                    if window.get_mouse_button(glfw::MouseButtonLeft) == Action::Press {
                        yaw += LOOK_SPEED * (prev_x - x);
                        pitch += LOOK_SPEED * (prev_y - y);
                        if pitch > std::f32::consts::FRAC_PI_2 {
                            pitch = std::f32::consts::FRAC_PI_2;
                        } else if pitch < -std::f32::consts::FRAC_PI_2 {
                            pitch = -std::f32::consts::FRAC_PI_2;
                        }
                    }
                    prev_x = x;
                    prev_y = y;
                },
                glfw::WindowEvent::Key(key, _scancode, action, _mods) => match action {
                    Action::Press => {
                        match key {
                            Key::Escape => window.set_should_close(true),
                            _ => {}
                        }
                    },
                    _ => {}
                },
                _ => {}
            }
        }

        let dt = clock.since_mark();
        let time = clock.mark();

        if window.get_key(Key::W) == Action::Press {
            camera_pos.set_x(camera_pos.x() - MAX_MOVE_SPEED * dt * yaw.sin());
            camera_pos.set_z(camera_pos.z() - MAX_MOVE_SPEED * dt * yaw.cos());
        }
        if window.get_key(Key::S) == Action::Press {
            camera_pos.set_x(camera_pos.x() + MAX_MOVE_SPEED * dt * yaw.sin());
            camera_pos.set_z(camera_pos.z() + MAX_MOVE_SPEED * dt * yaw.cos());
        }
        if window.get_key(Key::A) == Action::Press {
            camera_pos.set_x(camera_pos.x() - MAX_MOVE_SPEED * dt * yaw.cos());
            camera_pos.set_z(camera_pos.z() + MAX_MOVE_SPEED * dt * yaw.sin());
        }
        if window.get_key(Key::D) == Action::Press {
            camera_pos.set_x(camera_pos.x() + MAX_MOVE_SPEED * dt * yaw.cos());
            camera_pos.set_z(camera_pos.z() - MAX_MOVE_SPEED * dt * yaw.sin());
        }
        if window.get_key(Key::Space) == Action::Press {
            camera_pos.set_y(camera_pos.y() + MAX_MOVE_SPEED * dt);
        }
        if window.get_key(Key::LeftShift) == Action::Press {
            camera_pos.set_y(camera_pos.y() - MAX_MOVE_SPEED * dt);
        }

        let walk_cycle = (0.5 * (time % 2.0) * std::f32::consts::TAU).cos() * dt;
        geometry.rotate_y(&body[0], std::f32::consts::TAU * walk_cycle, 0.0, 0.0);
        geometry.rotate_y(&body[6], std::f32::consts::TAU * walk_cycle, 0.0, 0.0);

        camera_view.reset_to_identity();
        camera_view.rotate_x(pitch);
        camera_view.rotate_y(yaw);
        camera_view.translate(-camera_pos.at(0), -camera_pos.at(1), -camera_pos.at(2));

        camera_proj.reset_to_identity();
        camera_proj.perspective(90.0, width as f32 / height as f32, 1.0, 100.0);

        shader_program.set("time", time);
        shader_program.set("camera_pos", &camera_pos);
        shader_program.set("camera_view", &camera_view);
        shader_program.set("camera_proj", &camera_proj);
        shader_program.set("ambient_color", &ambient_color);
        shader_program.set("pt_light_pos", &pt_light_pos);
        shader_program.set("pt_light_color", &pt_light_color);
        shader_program.set("pt_light_power", 1.0);
        shader_program.set("tex_atlas", &test_tex);

        gfx::clear();
        geometry.render();

        window.swap_buffers();
    }
}


fn gen_mc_player_geom() -> (Geometry, [GeometrySlice; 12]) {
    let mut geometry = Geometry::new();
    geometry.enable_render().unwrap();
    geometry.add_icosphere(&Vector::new([0.0, -16.0, 0.0]), 8.0, [0.8, 0.6, 0.0, 1.0], 4);
    /*for i in 0..geometry.vertex_count() {
        geometry.vertices[i * 10 + 7] = 1.0;
        geometry.vertices[i * 10 + 8] = -(geometry.vertices[i * 10 + 2].atan2(geometry.vertices[i * 10]) / std::f32::consts::TAU + 0.5) * 8.0;
        geometry.vertices[i * 10 + 9] = -(0.5 + 0.1 * geometry.vertices[i * 10 + 1]) * 5.0;
    }
    geometry.update_vertex_buffer();*/
    const CUBOID: [[u32; 3]; 12] = [
        [ 0, 1, 3], [ 3, 2, 0], [ 2, 3, 5], [ 5, 4, 2], [ 4, 5, 7], [ 7, 6, 4], [ 6, 7, 9], [ 9, 8, 6], [10, 2, 4], [ 4,11,10], [12,13,14], [14,15,12],
    ];
    const CUBOID_HOLLOW: [[u32; 3]; 24] = [
        [ 0, 1, 3], [ 3, 2, 0], [ 2, 3, 5], [ 5, 4, 2], [ 4, 5, 7], [ 7, 6, 4], [ 6, 7, 9], [ 9, 8, 6], [10, 2, 4], [ 4,11,10], [12,13,14], [14,15,12],
        [ 0, 3, 1], [ 3, 0, 2], [ 2, 5, 3], [ 5, 2, 4], [ 4, 7, 5], [ 7, 4, 6], [ 6, 9, 7], [ 9, 6, 8], [10, 4, 2], [ 4,10,11], [12,14,13], [14,12,15],
    ];
    const D: f32 = 0.15;
    let head = geometry.add(&[
        Vertex::textured([-2.0,  8.0, -2.0], [0.0000, 0.1250]),
        Vertex::textured([-2.0,  4.0, -2.0], [0.0000, 0.2500]),
        Vertex::textured([-2.0,  8.0,  2.0], [0.1250, 0.1250]),
        Vertex::textured([-2.0,  4.0,  2.0], [0.1250, 0.2500]),
        Vertex::textured([ 2.0,  8.0,  2.0], [0.2500, 0.1250]),
        Vertex::textured([ 2.0,  4.0,  2.0], [0.2500, 0.2500]),
        Vertex::textured([ 2.0,  8.0, -2.0], [0.3750, 0.1250]),
        Vertex::textured([ 2.0,  4.0, -2.0], [0.3750, 0.2500]),
        Vertex::textured([-2.0,  8.0, -2.0], [0.5000, 0.1250]),
        Vertex::textured([-2.0,  4.0, -2.0], [0.5000, 0.2500]),
        Vertex::textured([-2.0,  8.0, -2.0], [0.1250, 0.0000]),
        Vertex::textured([ 2.0,  8.0, -2.0], [0.2500, 0.0000]),
        Vertex::textured([-2.0,  4.0,  2.0], [0.2500, 0.0000]),
        Vertex::textured([-2.0,  4.0, -2.0], [0.2500, 0.1250]),
        Vertex::textured([ 2.0,  4.0, -2.0], [0.3750, 0.1250]),
        Vertex::textured([ 2.0,  4.0,  2.0], [0.3750, 0.0000]),
    ], &CUBOID);
    let torso = geometry.add(&[
        Vertex::textured([-2.0,  4.0, -1.0], [0.2500, 0.3125]),
        Vertex::textured([-2.0, -2.0, -1.0], [0.2500, 0.5000]),
        Vertex::textured([-2.0,  4.0,  1.0], [0.3125, 0.3125]),
        Vertex::textured([-2.0, -2.0,  1.0], [0.3125, 0.5000]),
        Vertex::textured([ 2.0,  4.0,  1.0], [0.4375, 0.3125]),
        Vertex::textured([ 2.0, -2.0,  1.0], [0.4375, 0.5000]),
        Vertex::textured([ 2.0,  4.0, -1.0], [0.5000, 0.3125]),
        Vertex::textured([ 2.0, -2.0, -1.0], [0.5000, 0.5000]),
        Vertex::textured([-2.0,  4.0, -1.0], [0.6250, 0.3125]),
        Vertex::textured([-2.0, -2.0, -1.0], [0.6250, 0.5000]),
        Vertex::textured([-2.0,  4.0, -1.0], [0.3125, 0.2500]),
        Vertex::textured([ 2.0,  4.0, -1.0], [0.4375, 0.2500]),
        Vertex::textured([-2.0, -2.0,  1.0], [0.4375, 0.2500]),
        Vertex::textured([-2.0, -2.0, -1.0], [0.4375, 0.3125]),
        Vertex::textured([ 2.0, -2.0, -1.0], [0.5625, 0.3125]),
        Vertex::textured([ 2.0, -2.0,  1.0], [0.5625, 0.2500]),
    ], &CUBOID);
    let right_arm = geometry.add(&[
        Vertex::textured([-4.0,  4.0, -1.0], [0.6250, 0.3125]),
        Vertex::textured([-4.0, -2.0, -1.0], [0.6250, 0.5000]),
        Vertex::textured([-4.0,  4.0,  1.0], [0.6875, 0.3125]),
        Vertex::textured([-4.0, -2.0,  1.0], [0.6875, 0.5000]),
        Vertex::textured([-2.0,  4.0,  1.0], [0.7500, 0.3125]),
        Vertex::textured([-2.0, -2.0,  1.0], [0.7500, 0.5000]),
        Vertex::textured([-2.0,  4.0, -1.0], [0.8125, 0.3125]),
        Vertex::textured([-2.0, -2.0, -1.0], [0.8125, 0.5000]),
        Vertex::textured([-4.0,  4.0, -1.0], [0.8750, 0.3125]),
        Vertex::textured([-4.0, -2.0, -1.0], [0.8750, 0.5000]),
        Vertex::textured([-4.0,  4.0, -1.0], [0.6875, 0.2500]),
        Vertex::textured([-2.0,  4.0, -1.0], [0.7500, 0.2500]),
        Vertex::textured([-4.0, -2.0,  1.0], [0.7500, 0.2500]),
        Vertex::textured([-4.0, -2.0, -1.0], [0.7500, 0.3125]),
        Vertex::textured([-2.0, -2.0, -1.0], [0.8125, 0.3125]),
        Vertex::textured([-2.0, -2.0,  1.0], [0.8125, 0.2500]),
    ], &CUBOID);
    let left_arm = geometry.add(&[
        Vertex::textured([ 2.0,  4.0, -1.0], [0.5000, 0.8125]),
        Vertex::textured([ 2.0, -2.0, -1.0], [0.5000, 1.0000]),
        Vertex::textured([ 2.0,  4.0,  1.0], [0.5625, 0.8125]),
        Vertex::textured([ 2.0, -2.0,  1.0], [0.5625, 1.0000]),
        Vertex::textured([ 4.0,  4.0,  1.0], [0.6250, 0.8125]),
        Vertex::textured([ 4.0, -2.0,  1.0], [0.6250, 1.0000]),
        Vertex::textured([ 4.0,  4.0, -1.0], [0.6875, 0.8125]),
        Vertex::textured([ 4.0, -2.0, -1.0], [0.6875, 1.0000]),
        Vertex::textured([ 2.0,  4.0, -1.0], [0.7500, 0.8125]),
        Vertex::textured([ 2.0, -2.0, -1.0], [0.7500, 1.0000]),
        Vertex::textured([ 2.0,  4.0, -1.0], [0.5625, 0.7500]),
        Vertex::textured([ 4.0,  4.0, -1.0], [0.6250, 0.7500]),
        Vertex::textured([ 2.0, -2.0,  1.0], [0.6250, 0.7500]),
        Vertex::textured([ 2.0, -2.0, -1.0], [0.6250, 0.8125]),
        Vertex::textured([ 4.0, -2.0, -1.0], [0.6875, 0.8125]),
        Vertex::textured([ 4.0, -2.0,  1.0], [0.6875, 0.7500]),
    ], &CUBOID);
    let right_leg = geometry.add(&[
        Vertex::textured([-2.0, -2.0, -1.0], [0.0000, 0.3125]),
        Vertex::textured([-2.0, -8.0, -1.0], [0.0000, 0.5000]),
        Vertex::textured([-2.0, -2.0,  1.0], [0.0625, 0.3125]),
        Vertex::textured([-2.0, -8.0,  1.0], [0.0625, 0.5000]),
        Vertex::textured([ 0.0, -2.0,  1.0], [0.1250, 0.3125]),
        Vertex::textured([ 0.0, -8.0,  1.0], [0.1250, 0.5000]),
        Vertex::textured([ 0.0, -2.0, -1.0], [0.1875, 0.3125]),
        Vertex::textured([ 0.0, -8.0, -1.0], [0.1875, 0.5000]),
        Vertex::textured([-2.0, -2.0, -1.0], [0.2500, 0.3125]),
        Vertex::textured([-2.0, -8.0, -1.0], [0.2500, 0.5000]),
        Vertex::textured([-2.0, -2.0, -1.0], [0.0625, 0.2500]),
        Vertex::textured([ 0.0, -2.0, -1.0], [0.1250, 0.2500]),
        Vertex::textured([-2.0, -8.0,  1.0], [0.1250, 0.2500]),
        Vertex::textured([-2.0, -8.0, -1.0], [0.1250, 0.3125]),
        Vertex::textured([ 0.0, -8.0, -1.0], [0.1875, 0.3125]),
        Vertex::textured([ 0.0, -8.0,  1.0], [0.1875, 0.2500]),
    ], &CUBOID);
    let left_leg = geometry.add(&[
        Vertex::textured([ 0.0, -2.0, -1.0], [0.2500, 0.8125]),
        Vertex::textured([ 0.0, -8.0, -1.0], [0.2500, 1.0000]),
        Vertex::textured([ 0.0, -2.0,  1.0], [0.3125, 0.8125]),
        Vertex::textured([ 0.0, -8.0,  1.0], [0.3125, 1.0000]),
        Vertex::textured([ 2.0, -2.0,  1.0], [0.3750, 0.8125]),
        Vertex::textured([ 2.0, -8.0,  1.0], [0.3750, 1.0000]),
        Vertex::textured([ 2.0, -2.0, -1.0], [0.4375, 0.8125]),
        Vertex::textured([ 2.0, -8.0, -1.0], [0.4375, 1.0000]),
        Vertex::textured([ 0.0, -2.0, -1.0], [0.5000, 0.8125]),
        Vertex::textured([ 0.0, -8.0, -1.0], [0.5000, 1.0000]),
        Vertex::textured([ 0.0, -2.0, -1.0], [0.3125, 0.7500]),
        Vertex::textured([ 2.0, -2.0, -1.0], [0.3750, 0.7500]),
        Vertex::textured([ 0.0, -8.0,  1.0], [0.3750, 0.7500]),
        Vertex::textured([ 0.0, -8.0, -1.0], [0.3750, 0.8125]),
        Vertex::textured([ 2.0, -8.0, -1.0], [0.4375, 0.8125]),
        Vertex::textured([ 2.0, -8.0,  1.0], [0.4375, 0.7500]),
    ], &CUBOID);
    let hat = geometry.add(&[
        Vertex::textured([-2.0-D,  8.0+D, -2.0-D], [0.5000, 0.1250]),
        Vertex::textured([-2.0-D,  4.0-D, -2.0-D], [0.5000, 0.2500]),
        Vertex::textured([-2.0-D,  8.0+D,  2.0+D], [0.6250, 0.1250]),
        Vertex::textured([-2.0-D,  4.0-D,  2.0+D], [0.6250, 0.2500]),
        Vertex::textured([ 2.0+D,  8.0+D,  2.0+D], [0.7500, 0.1250]),
        Vertex::textured([ 2.0+D,  4.0-D,  2.0+D], [0.7500, 0.2500]),
        Vertex::textured([ 2.0+D,  8.0+D, -2.0-D], [0.8750, 0.1250]),
        Vertex::textured([ 2.0+D,  4.0-D, -2.0-D], [0.8750, 0.2500]),
        Vertex::textured([-2.0-D,  8.0+D, -2.0-D], [1.0000, 0.1250]),
        Vertex::textured([-2.0-D,  4.0-D, -2.0-D], [1.0000, 0.2500]),
        Vertex::textured([-2.0-D,  8.0+D, -2.0-D], [0.6250, 0.0000]),
        Vertex::textured([ 2.0+D,  8.0+D, -2.0-D], [0.7500, 0.0000]),
        Vertex::textured([-2.0-D,  4.0-D,  2.0+D], [0.7500, 0.0000]),
        Vertex::textured([-2.0-D,  4.0-D, -2.0-D], [0.7500, 0.1250]),
        Vertex::textured([ 2.0+D,  4.0-D, -2.0-D], [0.8750, 0.1250]),
        Vertex::textured([ 2.0+D,  4.0-D,  2.0+D], [0.8750, 0.0000]),
    ], &CUBOID_HOLLOW);
    let jacket = geometry.add(&[
        Vertex::textured([-2.0-D,  4.0+D, -1.0-D], [0.2500, 0.5625]),
        Vertex::textured([-2.0-D, -2.0-D, -1.0-D], [0.2500, 0.7500]),
        Vertex::textured([-2.0-D,  4.0+D,  1.0+D], [0.3125, 0.5625]),
        Vertex::textured([-2.0-D, -2.0-D,  1.0+D], [0.3125, 0.7500]),
        Vertex::textured([ 2.0+D,  4.0+D,  1.0+D], [0.4375, 0.5625]),
        Vertex::textured([ 2.0+D, -2.0-D,  1.0+D], [0.4375, 0.7500]),
        Vertex::textured([ 2.0+D,  4.0+D, -1.0-D], [0.5000, 0.5625]),
        Vertex::textured([ 2.0+D, -2.0-D, -1.0-D], [0.5000, 0.7500]),
        Vertex::textured([-2.0-D,  4.0+D, -1.0-D], [0.6250, 0.5625]),
        Vertex::textured([-2.0-D, -2.0-D, -1.0-D], [0.6250, 0.7500]),
        Vertex::textured([-2.0-D,  4.0+D, -1.0-D], [0.3125, 0.5000]),
        Vertex::textured([ 2.0+D,  4.0+D, -1.0-D], [0.4375, 0.5000]),
        Vertex::textured([-2.0-D, -2.0-D,  1.0+D], [0.4375, 0.5000]),
        Vertex::textured([-2.0-D, -2.0-D, -1.0-D], [0.4375, 0.5625]),
        Vertex::textured([ 2.0+D, -2.0-D, -1.0-D], [0.5625, 0.5625]),
        Vertex::textured([ 2.0+D, -2.0-D,  1.0+D], [0.5625, 0.5000]),
    ], &CUBOID_HOLLOW);
    let right_sleeve = geometry.add(&[
        Vertex::textured([-4.0-D,  4.0+D, -1.0-D], [0.6250, 0.5625]),
        Vertex::textured([-4.0-D, -2.0-D, -1.0-D], [0.6250, 0.7500]),
        Vertex::textured([-4.0-D,  4.0+D,  1.0+D], [0.6875, 0.5625]),
        Vertex::textured([-4.0-D, -2.0-D,  1.0+D], [0.6875, 0.7500]),
        Vertex::textured([-2.0+D,  4.0+D,  1.0+D], [0.7500, 0.5625]),
        Vertex::textured([-2.0+D, -2.0-D,  1.0+D], [0.7500, 0.7500]),
        Vertex::textured([-2.0+D,  4.0+D, -1.0-D], [0.8125, 0.5625]),
        Vertex::textured([-2.0+D, -2.0-D, -1.0-D], [0.8125, 0.7500]),
        Vertex::textured([-4.0-D,  4.0+D, -1.0-D], [0.8750, 0.5625]),
        Vertex::textured([-4.0-D, -2.0-D, -1.0-D], [0.8750, 0.7500]),
        Vertex::textured([-4.0-D,  4.0+D, -1.0-D], [0.6875, 0.5000]),
        Vertex::textured([-2.0+D,  4.0+D, -1.0-D], [0.7500, 0.5000]),
        Vertex::textured([-4.0-D, -2.0-D,  1.0+D], [0.7500, 0.5000]),
        Vertex::textured([-4.0-D, -2.0-D, -1.0-D], [0.7500, 0.5625]),
        Vertex::textured([-2.0+D, -2.0-D, -1.0-D], [0.8125, 0.5625]),
        Vertex::textured([-2.0+D, -2.0-D,  1.0+D], [0.8125, 0.5000]),
    ], &CUBOID_HOLLOW);
    let left_sleeve = geometry.add(&[
        Vertex::textured([ 2.0-D,  4.0+D, -1.0-D], [0.7500, 0.8125]),
        Vertex::textured([ 2.0-D, -2.0-D, -1.0-D], [0.7500, 1.0000]),
        Vertex::textured([ 2.0-D,  4.0+D,  1.0+D], [0.8125, 0.8125]),
        Vertex::textured([ 2.0-D, -2.0-D,  1.0+D], [0.8125, 1.0000]),
        Vertex::textured([ 4.0+D,  4.0+D,  1.0+D], [0.8750, 0.8125]),
        Vertex::textured([ 4.0+D, -2.0-D,  1.0+D], [0.8750, 1.0000]),
        Vertex::textured([ 4.0+D,  4.0+D, -1.0-D], [0.9375, 0.8125]),
        Vertex::textured([ 4.0+D, -2.0-D, -1.0-D], [0.9375, 1.0000]),
        Vertex::textured([ 2.0-D,  4.0+D, -1.0-D], [1.0000, 0.8125]),
        Vertex::textured([ 2.0-D, -2.0-D, -1.0-D], [1.0000, 1.0000]),
        Vertex::textured([ 2.0-D,  4.0+D, -1.0-D], [0.8125, 0.7500]),
        Vertex::textured([ 4.0+D,  4.0+D, -1.0-D], [0.8750, 0.7500]),
        Vertex::textured([ 2.0-D, -2.0-D,  1.0+D], [0.8750, 0.7500]),
        Vertex::textured([ 2.0-D, -2.0-D, -1.0-D], [0.8750, 0.8125]),
        Vertex::textured([ 4.0+D, -2.0-D, -1.0-D], [0.9375, 0.8125]),
        Vertex::textured([ 4.0+D, -2.0-D,  1.0+D], [0.9375, 0.7500]),
    ], &CUBOID_HOLLOW);
    let right_pant = geometry.add(&[
        Vertex::textured([-2.0-D, -2.0+D, -1.0-D], [0.0000, 0.5625]),
        Vertex::textured([-2.0-D, -8.0-D, -1.0-D], [0.0000, 0.7500]),
        Vertex::textured([-2.0-D, -2.0+D,  1.0+D], [0.0625, 0.5625]),
        Vertex::textured([-2.0-D, -8.0-D,  1.0+D], [0.0625, 0.7500]),
        Vertex::textured([ 0.0+D, -2.0+D,  1.0+D], [0.1250, 0.5625]),
        Vertex::textured([ 0.0+D, -8.0-D,  1.0+D], [0.1250, 0.7500]),
        Vertex::textured([ 0.0+D, -2.0+D, -1.0-D], [0.1875, 0.5625]),
        Vertex::textured([ 0.0+D, -8.0-D, -1.0-D], [0.1875, 0.7500]),
        Vertex::textured([-2.0-D, -2.0+D, -1.0-D], [0.2500, 0.5625]),
        Vertex::textured([-2.0-D, -8.0-D, -1.0-D], [0.2500, 0.7500]),
        Vertex::textured([-2.0-D, -2.0+D, -1.0-D], [0.0625, 0.5000]),
        Vertex::textured([ 0.0+D, -2.0+D, -1.0-D], [0.1250, 0.5000]),
        Vertex::textured([-2.0-D, -8.0-D,  1.0+D], [0.1250, 0.5000]),
        Vertex::textured([-2.0-D, -8.0-D, -1.0-D], [0.1250, 0.5625]),
        Vertex::textured([ 0.0+D, -8.0-D, -1.0-D], [0.1875, 0.5625]),
        Vertex::textured([ 0.0+D, -8.0-D,  1.0+D], [0.1875, 0.5000]),
    ], &CUBOID_HOLLOW);
    let left_pant = geometry.add(&[
        Vertex::textured([ 0.0-D, -2.0+D, -1.0-D], [0.0000, 0.8125]),
        Vertex::textured([ 0.0-D, -8.0-D, -1.0-D], [0.0000, 1.0000]),
        Vertex::textured([ 0.0-D, -2.0+D,  1.0+D], [0.0625, 0.8125]),
        Vertex::textured([ 0.0-D, -8.0-D,  1.0+D], [0.0625, 1.0000]),
        Vertex::textured([ 2.0+D, -2.0+D,  1.0+D], [0.1250, 0.8125]),
        Vertex::textured([ 2.0+D, -8.0-D,  1.0+D], [0.1250, 1.0000]),
        Vertex::textured([ 2.0+D, -2.0+D, -1.0-D], [0.1875, 0.8125]),
        Vertex::textured([ 2.0+D, -8.0-D, -1.0-D], [0.1875, 1.0000]),
        Vertex::textured([ 0.0-D, -2.0+D, -1.0-D], [0.2500, 0.8125]),
        Vertex::textured([ 0.0-D, -8.0-D, -1.0-D], [0.2500, 1.0000]),
        Vertex::textured([ 0.0-D, -2.0+D, -1.0-D], [0.0625, 0.7500]),
        Vertex::textured([ 2.0+D, -2.0+D, -1.0-D], [0.1250, 0.7500]),
        Vertex::textured([ 0.0-D, -8.0-D,  1.0+D], [0.1250, 0.7500]),
        Vertex::textured([ 0.0-D, -8.0-D, -1.0-D], [0.1250, 0.8125]),
        Vertex::textured([ 2.0+D, -8.0-D, -1.0-D], [0.1875, 0.8125]),
        Vertex::textured([ 2.0+D, -8.0-D,  1.0+D], [0.1875, 0.7500]),
    ], &CUBOID_HOLLOW);
    (geometry, [head, torso, right_arm, left_arm, right_leg, left_leg, hat, jacket, right_sleeve, left_sleeve, right_pant, left_pant])
}*/
