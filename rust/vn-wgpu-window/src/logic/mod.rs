use crate::renderer::Renderer;
use winit::event::KeyEvent;
use winit::event_loop::ActiveEventLoop;

pub trait StateLogic<R: Renderer>: Sized + 'static {
    fn process_events(&mut self) {}

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
    fn resized(&mut self, width: u32, height: u32) {}

    #[allow(unused_variables)]
    fn update(&mut self) {}

    fn render_target(&self) -> R::RenderTarget;
}
