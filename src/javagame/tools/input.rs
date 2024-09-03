use glfw::{Action, Modifiers, WindowEvent};
use innovus::{Application, WindowEventReceiver};

pub use glfw::{Key, MouseButton, MouseButtonLeft, MouseButtonRight, MouseButtonMiddle};
use innovus::tools::Vector;

const FIRST_KEY_VALUE: usize = glfw::ffi::KEY_SPACE as usize;
const LAST_KEY_VALUE: usize = glfw::ffi::KEY_LAST as usize;
const KEY_ARRAY_SIZE: usize = LAST_KEY_VALUE - FIRST_KEY_VALUE + 1;
const FIRST_MOUSE_BUTTON_VALUE: usize = glfw::ffi::MOUSE_BUTTON_1 as usize;
const LAST_MOUSE_BUTTON_VALUE: usize = glfw::ffi::MOUSE_BUTTON_LAST as usize;
const MOUSE_BUTTON_ARRAY_SIZE: usize = LAST_MOUSE_BUTTON_VALUE - FIRST_MOUSE_BUTTON_VALUE + 1;

const fn get_key_index(key: Key) -> Option<usize> {
    let key = key as usize;
    if FIRST_KEY_VALUE <= key && key <= LAST_KEY_VALUE {
        Some(key - FIRST_KEY_VALUE)
    } else {
        None
    }
}

const fn get_mouse_button_index(button: MouseButton) -> Option<usize> {
    let button = button as usize;
    if FIRST_MOUSE_BUTTON_VALUE <= button && button <= LAST_MOUSE_BUTTON_VALUE {
        Some(button - FIRST_MOUSE_BUTTON_VALUE)
    } else {
        None
    }
}

pub struct InputState {
    event_receiver: WindowEventReceiver,
    held_keys: [bool; KEY_ARRAY_SIZE],
    held_mouse_buttons: [bool; MOUSE_BUTTON_ARRAY_SIZE],
    cursor_pos: Vector<f64, 2>,
}

impl InputState {
    pub fn new(event_receiver: WindowEventReceiver) -> Self {
        Self {
            event_receiver,
            held_keys: [false; KEY_ARRAY_SIZE],
            held_mouse_buttons: [false; MOUSE_BUTTON_ARRAY_SIZE],
            cursor_pos: Vector::zero(),
        }
    }

    pub fn process_events(&mut self, application: &mut Application) -> Vec<WindowEvent> {
        application.poll_events();
        let mut events: Vec<_> = glfw::flush_messages(&self.event_receiver)
            .map(|(_time, event)| event)
            .collect();
        for event in &events {
            match *event {
                WindowEvent::Key(key, _scancode, action, mods) => self.handle_key(key, action, mods),
                WindowEvent::MouseButton(button, action, mods) => self.handle_mouse_button(button, action, mods),
                WindowEvent::CursorPos(x, y) => self.handle_cursor_pos(x, y),
                _ => {}
            }
        }
        events
    }

    pub fn handle_key(&mut self, key: Key, action: Action, mods: Modifiers) {
        if let Some(key_index) = get_key_index(key) {
            match action {
                Action::Press => self.held_keys[key_index] = true,
                Action::Release => self.held_keys[key_index] = false,
                _ => {}
            }
        }
    }

    pub fn handle_mouse_button(&mut self, button: MouseButton, action: Action, mods: Modifiers) {
        if let Some(button_index) = get_mouse_button_index(button) {
            match action {
                Action::Press => self.held_mouse_buttons[button_index] = true,
                Action::Release => self.held_mouse_buttons[button_index] = false,
                _ => {}
            }
        }
    }

    pub fn handle_cursor_pos(&mut self, x: f64, y: f64) {
        self.cursor_pos = Vector([x, y]);
    }

    pub fn key_is_held(&self, key: Key) -> bool {
        get_key_index(key)
            .is_some_and(|key_index| self.held_keys[key_index])
    }

    pub fn mouse_button_is_held(&self, button: MouseButton) -> bool {
        get_mouse_button_index(button)
            .is_some_and(|button_index| self.held_mouse_buttons[button_index])
    }

    pub fn cursor_pos(&self) -> Vector<f64, 2> {
        self.cursor_pos
    }
}