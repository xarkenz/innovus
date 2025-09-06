extern crate glfw;

use glfw::{Context, GlfwReceiver, PWindow, SwapInterval, WindowEvent, WindowHint, WindowMode};

pub mod data;
pub mod gfx;
pub mod scene;
pub mod tools;

pub type WindowEventReceiver = GlfwReceiver<(f64, WindowEvent)>;

pub struct Application {
    glfw: glfw::Glfw,
}

impl Application {
    pub fn new() -> Result<Application, String> {
        use glfw::fail_on_errors;
        Ok(Application {
            glfw: glfw::init(glfw::fail_on_errors!()).map_err(|err| err.to_string())?,
        })
    }

    pub fn create_window(
        &mut self,
        width: u32,
        height: u32,
        title: &str,
        mode: WindowMode<'_>,
    ) -> Option<(PWindow, WindowEventReceiver)> {
        let (mut window, receiver) = self.glfw.create_window(width, height, title, mode)?;
        window.make_current();
        gfx::screen::bind_glfw(&self.glfw);
        self.glfw.set_swap_interval(SwapInterval::Sync(1)); // TODO: user preference
        Some((window, receiver))
    }

    pub fn set_multisampling(&mut self, samples: Option<u32>) {
        self.glfw.window_hint(WindowHint::Samples(samples));
    }

    pub fn poll_events(&mut self) {
        self.glfw.poll_events();
    }
}
