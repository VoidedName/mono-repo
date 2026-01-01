use crate::graphics::GraphicsContext;
use crate::renderer::Renderer;
use crate::resource_manager::ResourceManager;
use std::rc::Rc;
use winit::event::KeyEvent;
use winit::event_loop::ActiveEventLoop;

pub trait StateLogic<R: Renderer>: Sized + 'static {
    #[allow(async_fn_in_trait)]
    async fn new_from_graphics_context(
        graphics_context: Rc<GraphicsContext>,
        resource_manager: Rc<ResourceManager>,
    ) -> anyhow::Result<Self>;

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
