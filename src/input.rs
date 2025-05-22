use winit::event::WindowEvent;
use std::collections::HashMap;
use winit::event::{ElementState, MouseButton};

pub struct InputManager {
    keys_pressed: HashMap<winit::keyboard::PhysicalKey, bool>,
    mouse_buttons: HashMap<MouseButton, bool>,
    mouse_position: (f32, f32),
}

impl InputManager {
    pub fn new() -> Self {
        Self {
            keys_pressed: HashMap::new(),
            mouse_buttons: HashMap::new(),
            mouse_position: (0.0, 0.0),
        }
    }
    
    pub fn process_window_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput { event, .. } => {
                if let winit::event::KeyEvent { physical_key: key_code, .. } = event {
                    let is_pressed = event.state == ElementState::Pressed;
                    self.keys_pressed.insert(*key_code, is_pressed);
                }
            }
            WindowEvent::MouseInput { button, state, .. } => {
                let is_pressed = *state == ElementState::Pressed;
                self.mouse_buttons.insert(*button, is_pressed);
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_position = (position.x as f32, position.y as f32);
            }
            _ => {}
        }
    }
    
    pub fn is_key_pressed(&self, key_code: winit::keyboard::PhysicalKey) -> bool {
        *self.keys_pressed.get(&key_code).unwrap_or(&false)
    }
    
    pub fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        *self.mouse_buttons.get(&button).unwrap_or(&false)
    }
    
    pub fn get_mouse_position(&self) -> (f32, f32) {
        self.mouse_position
    }
}