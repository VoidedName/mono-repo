use crate::graphics::GraphicsContext;
use crate::renderer::Renderer;
use crate::resource_manager::ResourceManager;
use std::sync::Arc;
use winit::event::KeyEvent;
use winit::event_loop::ActiveEventLoop;

pub trait StateLogic<R: Renderer>: Sized + 'static {
    #[allow(async_fn_in_trait)]
    async fn new_from_graphics_context(
        graphics_context: &GraphicsContext,
        resource_manager: Arc<ResourceManager>,
    ) -> anyhow::Result<Self>;

    #[allow(unused_variables)]
    fn handle_key(&mut self, event_loop: &ActiveEventLoop, event: &KeyEvent) {}

    #[allow(unused_variables)]
    fn update(&mut self) {}

    fn render_target(&self) -> R::RenderTarget;
}
