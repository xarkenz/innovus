use std::error::Error;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::{KeyEvent, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};
use crate::gfx::color::Color;
use crate::gfx::Gfx;
use crate::input::InputState;
use crate::tools::Vector;

pub mod data;
pub mod gfx;
pub mod scene;
pub mod tools;
pub mod input;

pub trait Game {
    fn start(window: Arc<Window>, gfx: Gfx<'static>) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized;

    fn handle_resize(&mut self, size: Vector<u32, 2>);

    fn handle_scale_change(&mut self, scale_factor: f64) {
        let _ = scale_factor;
    }

    fn run_frame(&mut self, inputs: &InputState);
}

pub struct ApplicationState<G: Game> {
    window: Arc<Window>,
    inputs: InputState,
    game: G,
}

impl<G: Game> ApplicationState<G> {
    pub async fn new(window: Arc<Window>) -> Result<Self, Box<dyn Error>> {
        let surface_size = Vector([window.inner_size().width, window.inner_size().height]);
        let gfx = Gfx::create(window.clone(), surface_size, Color::Black)
            .await
            .map_err(|err| err.to_string())?;

        let game = G::start(window.clone(), gfx)?;

        Ok(Self {
            window,
            inputs: InputState::new(),
            game,
        })
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn handle_resize(&mut self, size: Vector<u32, 2>) {
        self.game.handle_resize(size);
    }

    pub fn handle_scale_change(&mut self, scale_factor: f64) {
        self.game.handle_scale_change(scale_factor);
    }

    pub fn handle_mouse(&mut self, button: MouseButton, is_press: bool, is_repeat: bool) {
        self.inputs.handle_mouse(button, is_press, is_repeat);
    }

    pub fn handle_keyboard(&mut self, key: KeyCode, text: Option<&str>, is_press: bool, is_repeat: bool) {
        self.inputs.handle_keyboard(key, text, is_press, is_repeat);
    }

    pub fn handle_cursor_enter(&mut self) {
        self.inputs.handle_cursor_enter();
    }

    pub fn handle_cursor_exit(&mut self) {
        self.inputs.handle_cursor_exit();
    }

    pub fn handle_cursor_move(&mut self, position: Vector<f64, 2>) {
        self.inputs.handle_cursor_move(position);
    }

    pub fn handle_scroll(&mut self, delta: MouseScrollDelta) {
        self.inputs.handle_scroll(delta);
    }

    pub fn run_frame(&mut self) {
        self.window.request_redraw();
        self.game.run_frame(&self.inputs);
        self.inputs.reset();
    }
}

pub struct Application<G: Game + 'static> {
    #[cfg(target_arch = "wasm32")]
    proxy: Option<winit::event_loop::EventLoopProxy<ApplicationState<G>>>,
    state: Option<ApplicationState<G>>,
}

impl<G: Game + 'static> Application<G> {
    pub fn new(#[cfg(target_arch = "wasm32")] event_loop: &EventLoop<ApplicationState<G>>) -> Self {
        Self {
            #[cfg(target_arch = "wasm32")]
            proxy: Some(event_loop.create_proxy()),
            state: None,
        }
    }
}

impl<G: Game + 'static> ApplicationHandler<ApplicationState<G>> for Application<G> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        #[allow(unused_mut)]
        let mut window_attributes = Window::default_attributes();

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            use winit::platform::web::WindowAttributesExtWebSys;

            const CANVAS_ID: &str = "canvas";

            let window = wgpu::web_sys::window().unwrap_throw();
            let document = window.document().unwrap_throw();
            let canvas = document.get_element_by_id(CANVAS_ID).unwrap_throw();
            let html_canvas_element = canvas.unchecked_into();
            window_attributes = window_attributes.with_canvas(Some(html_canvas_element));
        }

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        #[cfg(not(target_arch = "wasm32"))]
        {
            self.state = Some(pollster::block_on(ApplicationState::new(window)).unwrap());
        }
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(proxy) = self.proxy.take() {
                wasm_bindgen_futures::spawn_local(async move {
                    assert!(proxy.send_event(ApplicationState::new(window).await.expect("failed to create canvas")).is_ok())
                });
            }
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, #[allow(unused_mut)] mut event: ApplicationState<G>) {
        let _ = event_loop;

        #[cfg(target_arch = "wasm32")]
        {
            event.window.request_redraw();
            event.resize(
                event.window.inner_size().width,
                event.window.inner_size().height,
            );
        }

        self.state = Some(event);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        let _ = window_id;

        let Some(state) = &mut self.state else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                state.run_frame();
            }
            WindowEvent::Resized(size) => {
                state.handle_resize(Vector([size.width, size.height]));
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                state.handle_scale_change(scale_factor);
            }
            WindowEvent::MouseInput { button, state: button_state, .. } => {
                state.handle_mouse(button, button_state.is_pressed(), false);
            }
            WindowEvent::KeyboardInput { event: KeyEvent {
                physical_key: PhysicalKey::Code(key),
                text,
                state: key_state,
                repeat,
                ..
            }, .. } => {
                state.handle_keyboard(key, text.as_deref(), key_state.is_pressed(), repeat);
            }
            WindowEvent::CursorEntered { .. } => {
                state.handle_cursor_enter();
            }
            WindowEvent::CursorLeft { .. } => {
                state.handle_cursor_exit();
            }
            WindowEvent::CursorMoved { position, .. } => {
                state.handle_cursor_move(Vector([position.x, position.y]));
            }
            WindowEvent::MouseWheel { delta, .. } => {
                state.handle_scroll(delta);
            }
            _ => {}
        }
    }
}

pub fn run_game_application<G: Game + 'static>() -> Result<(), Box<dyn Error>> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
    }
    #[cfg(target_arch = "wasm32")]
    {
        console_log::init_with_level(log::Level::Info).unwrap_throw();
    }

    let event_loop = EventLoop::with_user_event().build()?;
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut application = Application::<G>::new();
        event_loop.run_app(&mut application)?;
    }
    #[cfg(target_arch = "wasm32")]
    {
        let application = Application::<G>::new(&event_loop);
        event_loop.spawn_app(application);
    }

    Ok(())
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn run_game_application_web<G: Game + 'static>() -> Result<(), wasm_bindgen::JsValue> {
    console_error_panic_hook::set_once();
    run_game_application::<G>().unwrap_throw();

    Ok(())
}
