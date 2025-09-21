use glfw::{Action, Modifiers};
use innovus::tools::Vector;

pub use glfw::{Key, MouseButton};

const FIRST_KEY_VALUE: usize = Key::Space as usize;
const LAST_KEY_VALUE: usize = Key::Menu as usize;
const KEY_ARRAY_SIZE: usize = LAST_KEY_VALUE - FIRST_KEY_VALUE + 1;
const FIRST_MOUSE_BUTTON_VALUE: usize = MouseButton::Button1 as usize;
const LAST_MOUSE_BUTTON_VALUE: usize = MouseButton::Button8 as usize;
const MOUSE_BUTTON_ARRAY_SIZE: usize = LAST_MOUSE_BUTTON_VALUE - FIRST_MOUSE_BUTTON_VALUE + 1;

const fn get_key_index(key: Key) -> Option<usize> {
    let key = key as usize;
    if FIRST_KEY_VALUE <= key && key <= LAST_KEY_VALUE {
        Some(key - FIRST_KEY_VALUE)
    }
    else {
        None
    }
}

const fn get_mouse_button_index(button: MouseButton) -> Option<usize> {
    let button = button as usize;
    if FIRST_MOUSE_BUTTON_VALUE <= button && button <= LAST_MOUSE_BUTTON_VALUE {
        Some(button - FIRST_MOUSE_BUTTON_VALUE)
    }
    else {
        None
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
enum ButtonState {
    Pressed,
    Held,
    Released,
    Idle,
}

impl ButtonState {
    fn stabilize(&mut self) {
        *self = match *self {
            ButtonState::Pressed => ButtonState::Held,
            ButtonState::Released => ButtonState::Idle,
            state => state
        }
    }
}

pub struct InputState {
    keys: [ButtonState; KEY_ARRAY_SIZE],
    mouse_buttons: [ButtonState; MOUSE_BUTTON_ARRAY_SIZE],
    cursor_pos: Vector<f64, 2>,
    scroll_amount: Option<Vector<f64, 2>>,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            keys: [ButtonState::Idle; KEY_ARRAY_SIZE],
            mouse_buttons: [ButtonState::Idle; MOUSE_BUTTON_ARRAY_SIZE],
            cursor_pos: Vector::zero(),
            scroll_amount: None,
        }
    }

    pub fn reset(&mut self) {
        for state in &mut self.keys {
            state.stabilize();
        }
        for state in &mut self.mouse_buttons {
            state.stabilize();
        }
        self.scroll_amount = None;
    }

    pub fn handle_key(&mut self, key: Key, action: Action, mods: Modifiers) {
        let _ = mods;
        if let Some(key_index) = get_key_index(key) {
            match action {
                Action::Press => self.keys[key_index] = ButtonState::Pressed,
                Action::Release => self.keys[key_index] = ButtonState::Released,
                _ => {}
            }
        }
    }

    pub fn handle_mouse_button(&mut self, button: MouseButton, action: Action, mods: Modifiers) {
        let _ = mods;
        if let Some(button_index) = get_mouse_button_index(button) {
            match action {
                Action::Press => self.mouse_buttons[button_index] = ButtonState::Pressed,
                Action::Release => self.mouse_buttons[button_index] = ButtonState::Released,
                _ => {}
            }
        }
    }

    pub fn handle_cursor_pos(&mut self, x: f64, y: f64) {
        self.cursor_pos = Vector([x, y]);
    }

    pub fn handle_scroll(&mut self, dx: f64, dy: f64) {
        self.scroll_amount = Some(Vector([dx, dy]));
    }

    pub fn key_is_held(&self, key: Key) -> bool {
        get_key_index(key).is_some_and(|key_index| {
            matches!(self.keys[key_index], ButtonState::Pressed | ButtonState::Held)
        })
    }

    pub fn key_was_pressed(&self, key: Key) -> bool {
        get_key_index(key).is_some_and(|key_index| {
            matches!(self.keys[key_index], ButtonState::Pressed)
        })
    }

    pub fn key_was_released(&self, key: Key) -> bool {
        get_key_index(key).is_some_and(|key_index| {
            matches!(self.keys[key_index], ButtonState::Released)
        })
    }

    pub fn mouse_button_is_held(&self, button: MouseButton) -> bool {
        get_mouse_button_index(button).is_some_and(|button_index| {
            matches!(self.mouse_buttons[button_index], ButtonState::Pressed | ButtonState::Held)
        })
    }

    pub fn mouse_button_was_pressed(&self, button: MouseButton) -> bool {
        get_mouse_button_index(button).is_some_and(|button_index| {
            matches!(self.mouse_buttons[button_index], ButtonState::Pressed)
        })
    }

    pub fn mouse_button_was_released(&self, button: MouseButton) -> bool {
        get_mouse_button_index(button).is_some_and(|button_index| {
            matches!(self.mouse_buttons[button_index], ButtonState::Released)
        })
    }

    pub fn cursor_pos(&self) -> Vector<f64, 2> {
        self.cursor_pos
    }

    pub fn scroll_amount(&self) -> Option<Vector<f64, 2>> {
        self.scroll_amount
    }
}
