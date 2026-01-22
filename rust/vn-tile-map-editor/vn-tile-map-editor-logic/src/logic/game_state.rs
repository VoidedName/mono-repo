use winit::event::KeyEvent;
use winit::event::MouseButton;
use winit::event::ElementState;

pub mod editor;
pub use editor::Editor;

pub trait GameStateEx {
    type Event;

    fn process_events(&mut self) -> Option<Self::Event>;

    fn render_target(&self, size: (f32, f32)) -> vn_wgpu_window::scene::WgpuScene;

    fn handle_key(&mut self, event: &KeyEvent);

    fn handle_mouse_position(&mut self, x: f32, y: f32);

    fn handle_mouse_button(
        &mut self,
        mouse_position: (f32, f32),
        button: MouseButton,
        state: ElementState,
    );
    
    fn handle_mouse_wheel(&mut self, delta_x: f32, delta_y: f32);
}

pub enum GameState {
    Editor(Editor),
}

impl GameState {
    pub fn render_target(&self, size: (f32, f32)) -> vn_wgpu_window::scene::WgpuScene {
        match self {
            GameState::Editor(inner) => inner.render_target(size),
        }
    }

    pub fn handle_key(&mut self, event: &KeyEvent) {
        match self {
            GameState::Editor(inner) => inner.handle_key(event),
        }
    }

    pub fn handle_mouse_position(&mut self, x: f32, y: f32) {
        match self {
            GameState::Editor(inner) => inner.handle_mouse_position(x, y),
        }
    }

    pub fn handle_mouse_button(
        &mut self,
        mouse_position: (f32, f32),
        button: MouseButton,
        state: ElementState,
    ) {
        match self {
            GameState::Editor(inner) => inner.handle_mouse_button(mouse_position, button, state),
        }
    }

    pub fn handle_mouse_wheel(&mut self, delta_x: f32, delta_y: f32) {
        match self {
            GameState::Editor(inner) => inner.handle_mouse_wheel(delta_x, delta_y),
        }
    }
}
