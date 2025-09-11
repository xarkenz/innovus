extern crate glfw;

use glfw::{Action, Context, Key, MouseButton};
use innovus::gfx::*;
use innovus::tools::{AnimationTimer, Clock, Easing, Transform3D, Vector};
use std::{collections::VecDeque, f32::consts, str::FromStr};
use innovus::gfx::color::RGBColor;

const BOARD_W: u32 = 10;
const BOARD_H: u32 = 10;
const BOARD_SPACE: f32 = 4.0;

/*const CAMERA_STATE: [(f32, f32, f32, f32, f32); 5] = [
    (0.0, 5.0, 30.0,  0.0, 0.0),
    (30.0, 5.0, 0.0,  0.0, consts::FRAC_PI_2),
    (0.0, 5.0, -30.0,  0.0, consts::PI),
    (-30.0, 5.0, 0.0,  0.0, 3.0 * consts::FRAC_PI_2),
    (0.0, 30.0, 0.0,  -consts::FRAC_PI_2, 0.0),
];*/

const CAMERA_STATE: [(f32, f32, f32, f32, f32); 5] = [
    (40.0, 0.0, 0.5 * consts::FRAC_PI_8, 0.0, 0.0),
    (
        40.0,
        consts::FRAC_PI_2,
        0.5 * consts::FRAC_PI_8,
        0.0,
        consts::FRAC_PI_2,
    ),
    (40.0, consts::PI, 0.5 * consts::FRAC_PI_8, 0.0, consts::PI),
    (
        40.0,
        3.0 * consts::FRAC_PI_2,
        0.5 * consts::FRAC_PI_8,
        0.0,
        3.0 * consts::FRAC_PI_2,
    ),
    (30.0, 0.0, consts::FRAC_PI_2, -consts::FRAC_PI_2, 0.0),
];

fn get_space_center(node: &(i32, i32)) -> Vector<f32, 3> {
    Vector([
        (node.0 as f32 - BOARD_W as f32 * 0.5 + 0.5) * BOARD_SPACE,
        0.0,
        (node.1 as f32 - BOARD_H as f32 * 0.5 + 0.5) * BOARD_SPACE,
    ])
}

fn load_snek_geometry(
    geometry: &mut Geometry<Vertex3D>,
    nodes: &VecDeque<(i32, i32)>,
    face: &(i32, i32),
) {
    const ARROW_COLOR: Vector<f32, 4> = Vector([0.8, 0.3, 0.1, 1.0]);
    let head = nodes.back().unwrap(); // nodes should never be empty
    geometry.clear();
    for node in nodes {
        let major = if *node == *head { 1.3 } else { 1.0 };
        //let minor = major * 0.8;
        let mut center = get_space_center(node);
        center.set_y(major);
        geometry.add_icosphere(center, major, Vector([0.1, 0.4, 0.8, 1.0]), 2);
    }
    let pos = get_space_center(&(head.0 + face.0, head.1 + face.1));
    let arrow_verts = if face.0 < 0 {
        (
            Vector([pos.x() - 1.0, 1.0, pos.z()]),
            Vector([pos.x() + 1.0, 1.0, pos.z() + 1.0]),
            Vector([pos.x() + 1.0, 1.0, pos.z() - 1.0]),
        )
    } else if face.0 > 0 {
        (
            Vector([pos.x() + 1.0, 1.0, pos.z()]),
            Vector([pos.x() - 1.0, 1.0, pos.z() - 1.0]),
            Vector([pos.x() - 1.0, 1.0, pos.z() + 1.0]),
        )
    } else if face.1 < 0 {
        (
            Vector([pos.x(), 1.0, pos.z() - 1.0]),
            Vector([pos.x() - 1.0, 1.0, pos.z() + 1.0]),
            Vector([pos.x() + 1.0, 1.0, pos.z() + 1.0]),
        )
    } else {
        (
            Vector([pos.x(), 1.0, pos.z() + 1.0]),
            Vector([pos.x() + 1.0, 1.0, pos.z() - 1.0]),
            Vector([pos.x() - 1.0, 1.0, pos.z() - 1.0]),
        )
    };
    geometry.add(
        &[
            Vertex3D::colored(arrow_verts.0, ARROW_COLOR),
            Vertex3D::colored(arrow_verts.1, ARROW_COLOR),
            Vertex3D::colored(arrow_verts.2, ARROW_COLOR),
        ],
        &[[0, 1, 2]],
    );
}

fn main() {
    use glfw::fail_on_errors;
    let mut glfw = glfw::init(glfw::fail_on_errors!()).unwrap();
    glfw.window_hint(glfw::WindowHint::Samples(Some(8)));

    let (mut window, events) = glfw
        .create_window(800, 800, "Rust Gaming.", glfw::WindowMode::Windowed)
        .expect("failed to create GLFW window.");
    window.maximize();
    window.make_current();
    window.set_size_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_mouse_button_polling(true);
    window.set_key_polling(true);
    glfw.set_swap_interval(glfw::SwapInterval::Sync(1));

    screen::bind_glfw(&glfw);

    let shader_program = Program::from_preset(ProgramPreset::Default3DShader).unwrap();

    let test_image = Image::load_file("src/snek/assets/koopa_red.png").unwrap();
    let mut test_tex = Texture2D::new(0);
    test_tex.set_minify_filter(TextureSampling::Linear);
    test_tex.set_magnify_filter(TextureSampling::Linear);
    test_tex.set_wrap_s(TextureWrap::MirroredRepeat);
    test_tex.set_wrap_t(TextureWrap::MirroredRepeat);
    test_tex.upload_image(&test_image);

    screen::set_clear_color(RGBColor::new(0.6, 0.9, 1.0));
    screen::set_blend(screen::Blend::Transparency);
    screen::set_viewport(0, 0, 800, 800);
    screen::set_culling(true);
    screen::set_depth_testing(true);

    let mut board_geometry = Geometry::from_str(include_str!("assets/snek_board.obj")).unwrap();
    board_geometry.enable_render().unwrap();
    let mut snek_geometry = Geometry::new();
    snek_geometry.enable_render().unwrap();
    let mut kooper = Geometry::from_str(include_str!("assets/koopa.obj")).unwrap();
    kooper.enable_render().unwrap();

    {
        let kooper_vertices = kooper.vertices().to_vec();
        for vertex in kooper.vertices_mut() {
            vertex.norm = Vector::zero();
            let mut count: f32 = 0.0;
            for test in kooper_vertices.iter() {
                if test.pos == vertex.pos {
                    vertex.norm[0] += test.norm[0];
                    vertex.norm[1] += test.norm[1];
                    vertex.norm[2] += test.norm[2];
                    count += 1.0;
                }
            }
            vertex.norm[0] /= count;
            vertex.norm[1] /= count;
            vertex.norm[2] /= count;
        }
        let mut transform_thingy = Transform3D::identity();
        transform_thingy.rotate_z(-consts::FRAC_PI_2);
        transform_thingy.rotate_y(consts::FRAC_PI_2);
        transform_thingy.translate(Vector([0.0, 0.0, -10.0]));
        kooper.transform(&kooper.as_slice(), transform_thingy);
    }

    let pt_light_pos = Vector([0.0, 50.0, 0.0]);
    let pt_light_color = Vector([1.0, 1.0, 1.0]);
    let ambient_color = Vector([0.5, 0.4, 0.3]);

    let (mut width, mut height) = (800_i32, 800_i32);
    let clock = Clock::start();
    let mut prev_time = clock.read();
    let (mut prev_x, mut prev_y) = (0.0_f32, 0.0_f32);

    let mut camera_state: usize = 0;
    let mut camera_pos = Vector([0.0, 8.0, 20.0]);
    let mut camera_view = Transform3D::zero();
    let mut camera_proj = Transform3D::zero();

    let mut cam_r_anim =
        AnimationTimer::new(&clock, Easing::SineOut, 1.0, CAMERA_STATE[camera_state].0);
    let mut cam_h_anim =
        AnimationTimer::new(&clock, Easing::SineOut, 1.0, CAMERA_STATE[camera_state].1);
    let mut cam_v_anim =
        AnimationTimer::new(&clock, Easing::SineOut, 1.0, CAMERA_STATE[camera_state].2);
    let mut cam_p_anim =
        AnimationTimer::new(&clock, Easing::SineOut, 1.0, CAMERA_STATE[camera_state].3);
    let mut cam_a_anim =
        AnimationTimer::new(&clock, Easing::SineOut, 1.0, CAMERA_STATE[camera_state].4);

    let mut snek_nodes: VecDeque<(i32, i32)> = VecDeque::from([(1, 5), (2, 5), (3, 5)]);
    let mut snek_face: (i32, i32) = (1, 0);
    let mut snek_prev_face: (i32, i32) = (1, 0);
    let mut snek_prev_tail: (i32, i32) = (0, 5);
    load_snek_geometry(&mut snek_geometry, &snek_nodes, &snek_face);

    while !window.should_close() {
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::Size(w, h) => {
                    width = w;
                    height = h;
                    screen::set_viewport(0, 0, w, h);
                }
                glfw::WindowEvent::CursorPos(x, y) => {
                    let (x, y) = (x as f32, y as f32);
                    let (dx, dy) = (prev_x - x, prev_y - y);
                    if window.get_mouse_button(glfw::MouseButtonLeft) == Action::Press {
                        // drag action
                    }
                    prev_x = x;
                    prev_y = y;
                }
                glfw::WindowEvent::MouseButton(button, action, _mods) => match action {
                    Action::Press => match button {
                        MouseButton::Button1 => {
                            snek_prev_tail = snek_nodes.pop_front().unwrap();
                            let head = snek_nodes.back().unwrap();
                            snek_nodes.push_back((head.0 + snek_face.0, head.1 + snek_face.1));
                            load_snek_geometry(&mut snek_geometry, &snek_nodes, &snek_face);
                            snek_prev_face = snek_face;
                        }
                        _ => {}
                    },
                    _ => {}
                },
                glfw::WindowEvent::Key(key, _scancode, action, _mods) => match action {
                    Action::Press => match key {
                        Key::Escape => window.set_should_close(true),
                        Key::Enter => {
                            //camera_state = (camera_state + 1) % CAMERA_STATE.len();
                            camera_state = if camera_state == 0 { 4 } else { 0 };
                            cam_r_anim.set_target(CAMERA_STATE[camera_state].0);
                            cam_h_anim.set_target(CAMERA_STATE[camera_state].1);
                            cam_v_anim.set_target(CAMERA_STATE[camera_state].2);
                            cam_p_anim.set_target(CAMERA_STATE[camera_state].3);
                            cam_a_anim.set_target(CAMERA_STATE[camera_state].4);
                        }
                        Key::Up | Key::W => {
                            if snek_prev_face.0 != 0 || snek_prev_face.1 <= 0 {
                                snek_face = (0, -1);
                                load_snek_geometry(&mut snek_geometry, &snek_nodes, &snek_face);
                            }
                        }
                        Key::Left | Key::A => {
                            if snek_prev_face.1 != 0 || snek_prev_face.0 <= 0 {
                                snek_face = (-1, 0);
                                load_snek_geometry(&mut snek_geometry, &snek_nodes, &snek_face);
                            }
                        }
                        Key::Down | Key::S => {
                            if snek_prev_face.0 != 0 || snek_prev_face.1 >= 0 {
                                snek_face = (0, 1);
                                load_snek_geometry(&mut snek_geometry, &snek_nodes, &snek_face);
                            }
                        }
                        Key::Right | Key::D => {
                            if snek_prev_face.1 != 0 || snek_prev_face.0 >= 0 {
                                snek_face = (1, 0);
                                load_snek_geometry(&mut snek_geometry, &snek_nodes, &snek_face);
                            }
                        }
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

        if camera_state == 0 {
            let angle = (time * consts::FRAC_PI_8 + consts::PI) % consts::TAU - consts::PI;
            if cam_h_anim.origin() > angle {
                cam_h_anim.set_origin(cam_h_anim.origin() - consts::TAU);
                cam_a_anim.set_origin(cam_a_anim.origin() - consts::TAU);
            }
            cam_h_anim.set_target(angle);
            cam_a_anim.set_target(angle);
        }

        let cam_r = cam_r_anim.value();
        let cam_h = cam_h_anim.value();
        let cam_v = cam_v_anim.value();
        camera_pos.set_x(cam_r * cam_h.sin() * cam_v.cos());
        camera_pos.set_y(cam_r * cam_v.sin());
        camera_pos.set_z(cam_r * cam_h.cos() * cam_v.cos());

        camera_view.reset_to_identity();
        camera_view.rotate_x(cam_p_anim.value());
        camera_view.rotate_y(cam_a_anim.value());
        camera_view.translate(-camera_pos);

        camera_proj.reset_to_identity();
        camera_proj.perspective(90.0, width as f32 / height as f32, 1.0, 100.0);

        shader_program.set_uniform("time", time);
        shader_program.set_uniform("camera_pos", camera_pos);
        shader_program.set_uniform("camera_view", camera_view);
        shader_program.set_uniform("camera_proj", camera_proj);
        shader_program.set_uniform("ambient_color", ambient_color);
        shader_program.set_uniform("pt_light_pos", pt_light_pos);
        shader_program.set_uniform("pt_light_color", pt_light_color);
        shader_program.set_uniform("pt_light_power", 1.0_f32);
        shader_program.set_uniform("tex_atlas", &test_tex);

        screen::clear();
        board_geometry.render();
        snek_geometry.render();
        // kooper.render();
        window.swap_buffers();
    }
}
