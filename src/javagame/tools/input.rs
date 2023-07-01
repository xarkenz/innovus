use std::sync::mpsc::Receiver;
use glfw::{Action, Modifiers, MouseButton, WindowEvent};
use innovus::{Application, WindowEventReceiver};

pub use glfw::Key;

const FIRST_KEY_VALUE: usize = glfw::ffi::KEY_SPACE as usize;
const LAST_KEY_VALUE: usize = glfw::ffi::KEY_LAST as usize;
const KEY_ARRAY_SIZE: usize = LAST_KEY_VALUE - FIRST_KEY_VALUE + 1;

const fn get_key_index(key: Key) -> Option<usize> {
    let key = key as usize;
    if FIRST_KEY_VALUE <= key && key <= LAST_KEY_VALUE {
        Some(key - FIRST_KEY_VALUE)
    } else {
        None
    }
}

pub struct InputState {
    event_receiver: WindowEventReceiver,
    held_keys: [bool; KEY_ARRAY_SIZE],
}

impl InputState {
    pub fn new(event_receiver: WindowEventReceiver) -> Self {
        Self {
            event_receiver,
            held_keys: [false; KEY_ARRAY_SIZE],
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
        // TODO
    }

    pub fn handle_cursor_pos(&mut self, x: f64, y: f64) {
        // TODO
    }

    pub fn key_is_held(&self, key: Key) -> bool {
        if let Some(key_index) = get_key_index(key) {
            self.held_keys[key_index]
        } else {
            false
        }
    }
}