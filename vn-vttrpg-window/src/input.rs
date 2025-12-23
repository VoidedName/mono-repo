use std::collections::HashSet;
use winit::event::{ElementState, KeyEvent};
use winit::keyboard::PhysicalKey;

pub struct InputState {
    keys_pressed: HashSet<PhysicalKey>,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            keys_pressed: HashSet::new(),
        }
    }

    pub fn handle_key(&mut self, event: &KeyEvent) {
        match event.state {
            ElementState::Pressed => {
                self.keys_pressed.insert(event.physical_key);
            }
            ElementState::Released => {
                self.keys_pressed.remove(&event.physical_key);
            }
        }
    }

    pub fn is_key_down(&self, key: PhysicalKey) -> bool {
        self.keys_pressed.contains(&key)
    }
}
