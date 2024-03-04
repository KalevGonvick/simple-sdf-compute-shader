use winit::event::{ElementState, KeyboardInput};

pub struct UserIO {
    keys_changed: Vec<(u32, ElementState)>,
    mouse_motion_delta: (f64, f64)
}

impl UserIO {
    pub fn new() -> UserIO {
        UserIO {
            keys_changed: Vec::new(),
            mouse_motion_delta: (0.0, 0.0),
        }
    }

    pub fn set_mouse_delta(
        &mut self,
        delta_x: f64,
        delta_y: f64
    ) {
        self.mouse_motion_delta = (delta_x, delta_y);
    }

    pub fn set_keyboard_input(
        &mut self,
        keyboard_input: &KeyboardInput
    ) {
        self.keys_changed.push((keyboard_input.scancode, keyboard_input.state));
    }


}