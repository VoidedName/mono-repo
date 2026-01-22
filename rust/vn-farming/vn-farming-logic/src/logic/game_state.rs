use vn_ui::*;

pub mod start_menu;
pub use start_menu::StartMenu;

pub mod playing;
pub use playing::Playing;

pub const MENU_FONT: &str = "menu-font";

pub trait GameStateEx {
    type Event;

    fn process_events(&mut self) -> Option<Self::Event>;

    fn render_target(&self, size: (f32, f32)) -> vn_wgpu_window::scene::WgpuScene;

    fn handle_key(&mut self, event: &KeyEvent);

    fn handle_mouse_position(&mut self, x: f32, y: f32);

    fn handle_mouse_button(
        &mut self,
        mouse_position: (f32, f32),
        button: winit::event::MouseButton,
        state: winit::event::ElementState,
    );
}

pub enum GameState {
    StartMenu(StartMenu),
    Playing(Playing),
}

macro_rules! delegate {
    ($self:ident, $inner:ident, $body:expr) => {
        match $self {
            GameState::StartMenu($inner) => $body,
            GameState::Playing($inner) => $body,
        }
    };
}

impl GameState {
    pub fn render_target(&self, size: (f32, f32)) -> vn_wgpu_window::scene::WgpuScene {
        delegate!(self, inner, inner.render_target(size))
    }

    pub fn handle_key(&mut self, event: &KeyEvent) {
        delegate!(self, inner, inner.handle_key(event))
    }

    pub fn handle_mouse_position(&mut self, x: f32, y: f32) {
        delegate!(self, inner, inner.handle_mouse_position(x, y))
    }

    pub fn handle_mouse_button(
        &mut self,
        mouse_position: (f32, f32),
        button: winit::event::MouseButton,
        state: winit::event::ElementState,
    ) {
        delegate!(
            self,
            inner,
            inner.handle_mouse_button(mouse_position, button, state)
        )
    }
}
