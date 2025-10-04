use gl::types::*;
use crate::gfx::color::RGBColor;

pub fn bind_glfw(manager: &glfw::Glfw) {
    gl::load_with(|symbol| unsafe {
        std::mem::transmute(manager.get_proc_address_raw(symbol))
    });
}

pub fn set_clear_color(color: RGBColor) {
    unsafe {
        gl::ClearColor(color.r(), color.g(), color.b(), 1.0);
    }
}

pub fn clear() {
    unsafe {
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }
}

pub enum BlendFunc {
    None,
    Transparency,
    Custom(GLenum, GLenum),
}

pub fn set_blend_func(blend: BlendFunc) {
    unsafe {
        if let BlendFunc::None = blend {
            gl::Disable(gl::BLEND);
        }
        else {
            gl::Enable(gl::BLEND);
            match blend {
                BlendFunc::None => unreachable!(),
                BlendFunc::Transparency => gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA),
                BlendFunc::Custom(src, dest) => gl::BlendFunc(src, dest),
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
