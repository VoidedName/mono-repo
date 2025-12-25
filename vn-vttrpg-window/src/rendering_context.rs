use crate::Renderer;
use crate::graphics::GraphicsContext;
use crate::logic::StateLogic;
use crate::resource_manager::ResourceManager;
use crate::scene_renderer::SceneRenderer;
use std::sync::Arc;
use winit::event::KeyEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

pub struct RenderingContext<T: StateLogic<R>, R: Renderer = SceneRenderer> {
    pub context: GraphicsContext,
    pub resource_manager: Arc<ResourceManager>,
    pub renderer: R,
    pub logic: T,
}

impl<T: StateLogic<SceneRenderer>> RenderingContext<T, SceneRenderer> {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let context = GraphicsContext::new(window).await?;
        let resource_manager = Arc::new(ResourceManager::new(context.wgpu.clone()));
        let renderer = SceneRenderer::new(&context, resource_manager.clone());
        let logic = T::new_from_graphics_context(&context, resource_manager.clone()).await?;

        Ok(Self {
            context,
            resource_manager,
            renderer,
            logic,
        })
    }
}

impl<T: StateLogic<R>, R: Renderer> RenderingContext<T, R> {
    /// !!! EXPECTS PHYSICAL SIZE !!!
    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            log::info!("Resizing window to {}x{}", width, height);

            self.context.config.width = width;
            self.context.config.height = height;
            self.context
                .surface
                .configure(self.context.device(), &self.context.config);
            self.context.surface_ready_for_rendering = true;
        }
    }

    pub fn handle_key(&mut self, event_loop: &ActiveEventLoop, event: &KeyEvent) {
        self.logic.handle_key(event_loop, event);
    }

    pub fn update(&mut self) {
        self.logic.update();
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.context.window.request_redraw();

        if !self.context.surface_ready_for_rendering {
            return Ok(());
        }

        let render_target = self.logic.render_target();
        self.renderer.render(&self.context, &render_target)
    }
}
