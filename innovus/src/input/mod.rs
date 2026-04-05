use winit::event::{MouseButton, MouseScrollDelta};
use winit::keyboard::KeyCode;
use crate::tools::Vector;

const MOUSE_BUTTON_ARRAY_SIZE: usize = 5;

fn get_button_index(button: MouseButton) -> Option<usize> {
    match button {
        MouseButton::Left => Some(0),
        MouseButton::Right => Some(1),
        MouseButton::Middle => Some(2),
        MouseButton::Back => Some(3),
        MouseButton::Forward => Some(4),
        _ => None
    }
}

// TODO: there's probably a better way to do this...
const FIRST_KEY_VALUE: usize = KeyCode::Backquote as usize;
const LAST_KEY_VALUE: usize = KeyCode::F35 as usize;
const KEY_ARRAY_SIZE: usize = LAST_KEY_VALUE - FIRST_KEY_VALUE + 1;

fn get_key_index(key: KeyCode) -> Option<usize> {
    let key = key as usize;
    (FIRST_KEY_VALUE..=LAST_KEY_VALUE).contains(&key).then_some(key - FIRST_KEY_VALUE)
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
enum ButtonState {
    Pressed,
    Repeated,
    Held,
    Released,
    Idle,
}

impl ButtonState {
    pub fn new(is_press: bool, is_repeat: bool) -> Self {
        match (is_press, is_repeat) {
            (true, false) => Self::Pressed,
            (true, true) => Self::Repeated,
            (false, _) => Self::Released,
        }
    }

    pub fn is_held(&self) -> bool {
        matches!(self, Self::Pressed | Self::Repeated | Self::Held)
    }

    pub fn was_pressed(&self) -> bool {
        matches!(self, Self::Pressed)
    }

    pub fn was_repeated(&self) -> bool {
        matches!(self, Self::Pressed | Self::Repeated)
    }

    pub fn was_released(&self) -> bool {
        matches!(self, Self::Released)
    }

    pub fn stabilize(&mut self) {
        *self = match *self {
            Self::Pressed | Self::Repeated => Self::Held,
            Self::Released => Self::Idle,
            state => state
        }
    }
}

pub struct InputState {
    buttons: [ButtonState; MOUSE_BUTTON_ARRAY_SIZE],
    cursor_position: Option<Vector<f64, 2>>,
    scroll_delta: Option<Vector<f64, 2>>,
    keys: [ButtonState; KEY_ARRAY_SIZE],
    entered_text: String,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            buttons: [ButtonState::Idle; MOUSE_BUTTON_ARRAY_SIZE],
            cursor_position: None,
            scroll_delta: None,
            keys: [ButtonState::Idle; KEY_ARRAY_SIZE],
            entered_text: String::new(),
        }
    }

    pub fn reset(&mut self) {
        for state in &mut self.buttons {
            state.stabilize();
        }
        self.scroll_delta = None;
        for state in &mut self.keys {
            state.stabilize();
        }
        self.entered_text.clear();
    }

    pub fn handle_mouse(&mut self, button: MouseButton, is_press: bool, is_repeat: bool) {
        if let Some(index) = get_button_index(button) {
            self.buttons[index] = ButtonState::new(is_press, is_repeat);
        }
    }

    pub fn handle_keyboard(&mut self, key: KeyCode, text: Option<&str>, is_press: bool, is_repeat: bool) {
        if let Some(index) = get_key_index(key) {
            self.keys[index] = ButtonState::new(is_press, is_repeat);
        }
        if let Some(text) = text {
            self.handle_text(text);
        }
    }

    pub fn handle_text(&mut self, text: &str) {
        self.entered_text.push_str(text);
    }

    pub fn handle_paste(&mut self, text: &str) {
        self.entered_text.push_str(text);
    }

    pub fn handle_cursor_enter(&mut self) {
        // Currently a no-op
    }

    pub fn handle_cursor_exit(&mut self) {
        self.cursor_position = None;
    }

    pub fn handle_cursor_move(&mut self, position: Vector<f64, 2>) {
        self.cursor_position = Some(position);
    }

    pub fn handle_scroll(&mut self, delta: MouseScrollDelta) {
        self.scroll_delta = Some(match delta {
            MouseScrollDelta::LineDelta(x, y) => Vector([x as f64, y as f64]),
            MouseScrollDelta::PixelDelta(pos) => Vector([pos.x, pos.y]),
        });
    }

    pub fn key_is_held(&self, key: KeyCode) -> bool {
        get_key_index(key).is_some_and(|index| {
            self.keys[index].is_held()
        })
    }

    pub fn key_was_pressed(&self, key: KeyCode) -> bool {
        get_key_index(key).is_some_and(|index| {
            self.keys[index].was_pressed()
        })
    }

    pub fn key_was_repeated(&self, key: KeyCode) -> bool {
        get_key_index(key).is_some_and(|index| {
            self.keys[index].was_repeated()
        })
    }

    pub fn key_was_released(&self, key: KeyCode) -> bool {
        get_key_index(key).is_some_and(|index| {
            self.keys[index].was_released()
        })
    }

    pub fn entered_text(&self) -> &str {
        &self.entered_text
    }

    pub fn button_is_held(&self, button: MouseButton) -> bool {
        get_button_index(button).is_some_and(|index| {
            self.buttons[index].is_held()
        })
    }

    pub fn button_was_pressed(&self, button: MouseButton) -> bool {
        get_button_index(button).is_some_and(|index| {
            self.buttons[index].was_pressed()
        })
    }

    pub fn button_was_repeated(&self, button: MouseButton) -> bool {
        get_button_index(button).is_some_and(|index| {
            self.buttons[index].was_repeated()
        })
    }

    pub fn button_was_released(&self, button: MouseButton) -> bool {
        get_button_index(button).is_some_and(|index| {
            self.buttons[index].was_released()
        })
    }

    pub fn cursor_position(&self) -> Option<Vector<f64, 2>> {
        self.cursor_position
    }

    pub fn scroll_delta(&self) -> Option<Vector<f64, 2>> {
        self.scroll_delta
    }
}
