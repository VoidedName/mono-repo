use crate::renderer::Renderer;
use winit::event::KeyEvent;
use winit::event_loop::ActiveEventLoop;

pub trait StateLogic<R: Renderer>: Sized + 'static {
    type Event: 'static;

    #[allow(unused_variables)]
    /// User Events send by the dispatcher are sent here.
    /// Async processes can call back into here via the event dispatcher.
    fn handle_event(&mut self, event: Self::Event) {}

    /// Called before every frame.
    fn update(&mut self) {}

    #[allow(unused_variables)]
    fn handle_key(&mut self, event_loop: &ActiveEventLoop, event: &KeyEvent) {}

    #[allow(unused_variables)]
    fn handle_mouse_position(&mut self, x: f32, y: f32) {}

    #[allow(unused_variables)]
    fn handle_mouse_button(
        &mut self,
        button: winit::event::MouseButton,
        state: winit::event::ElementState,
    ) {
    }

    #[allow(unused_variables)]
    fn handle_mouse_wheel(&mut self, delta_x: f32, delta_y: f32) {}

    #[allow(unused_variables)]
    fn resized(&mut self, width: u32, height: u32) {}

    fn render_target(&self) -> R::RenderTarget;
}
