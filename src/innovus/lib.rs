extern crate glfw;

use std::sync::mpsc::Receiver;

pub use glfw::{
    Action, Context, Key, MouseButton, SwapInterval, Window, WindowEvent, WindowHint, WindowMode,
};

pub mod data;
pub mod gfx;
pub mod scene;
pub mod tools;

pub type WindowEventReceiver = Receiver<(f64, WindowEvent)>;

pub struct Application {
    glfw: glfw::Glfw,
}

impl Application {
    pub fn new() -> Result<Application, String> {
        Ok(Application {
            glfw: glfw::init(glfw::FAIL_ON_ERRORS).map_err(|err| err.to_string())?,
        })
    }

    pub fn create_window(
        &mut self,
        width: u32,
        height: u32,
        title: &str,
        mode: WindowMode<'_>,
    ) -> Option<(Window, WindowEventReceiver)> {
        let mut created = self.glfw.create_window(width, height, title, mode);
        if let Some((window, _)) = &mut created {
            window.make_current();
            gfx::screen::bind_glfw(&self.glfw);
            self.glfw.set_swap_interval(SwapInterval::Sync(1)); // TODO: user preference
        }
        created
    }

    pub fn set_multisampling(&mut self, samples: Option<u32>) {
        self.glfw.window_hint(WindowHint::Samples(samples));
    }

    pub fn poll_events(&mut self) {
        self.glfw.poll_events();
    }
}
