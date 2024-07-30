extern crate gl;
extern crate glfw;

use gl::types::*;


pub fn bind_glfw(manager: &glfw::Glfw) {
    gl::load_with(|s| manager.get_proc_address_raw(s));
}


pub fn set_clear_color(r: GLfloat, g: GLfloat, b: GLfloat) {
    unsafe {
        gl::ClearColor(r, g, b, 1.0);
    }
}

pub enum Blend {
    None,
    Transparency,
    Custom(GLenum, GLenum),
}

pub fn set_blend(blend: Blend) {
    unsafe {
        if let Blend::None = blend {
            gl::Disable(gl::BLEND);
        }
        else {
            gl::Enable(gl::BLEND);
            match blend {
                Blend::None => unreachable!(),
                Blend::Transparency => gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA),
                Blend::Custom(src, dest) => gl::BlendFunc(src, dest),
            }
        }
    }
}

pub fn set_culling(enable: bool) {
    unsafe {
        if enable {
            gl::Enable(gl::CULL_FACE);
        }
        else {
            gl::Disable(gl::CULL_FACE);
        }
    }
}

pub fn set_depth_testing(enable: bool) {
    unsafe {
        if enable {
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LEQUAL);
        }
        else {
            gl::Disable(gl::DEPTH_TEST);
        }
    }
}

pub fn set_viewport(x: i32, y: i32, width: i32, height: i32) {
    unsafe {
        gl::Viewport(x as GLint, y as GLint, width as GLsizei, height as GLsizei);
    }
}


pub fn clear() {
    unsafe {
        gl::Clear(gl::COLOR_BUFFER_BIT);
        gl::Clear(gl::DEPTH_BUFFER_BIT);
    }
}