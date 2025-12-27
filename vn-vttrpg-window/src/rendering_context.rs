use crate::Renderer;
use crate::graphics::GraphicsContext;
use crate::logic::StateLogic;
use crate::resource_manager::ResourceManager;
use crate::scene_renderer::SceneRenderer;
use std::sync::Arc;
use winit::event::KeyEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

/// The main context for rendering the application, binding together graphics, resources, renderer, and logic.
pub struct RenderingContext<T: StateLogic<R>, R: Renderer = SceneRenderer> {
    pub context: Arc<GraphicsContext>,
    pub resource_manager: Arc<ResourceManager>,
    pub renderer: R,
    pub logic: T,
}

impl<T: StateLogic<SceneRenderer>> RenderingContext<T, SceneRenderer> {
    /// Creates a new rendering context for the given window.
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let context = Arc::new(GraphicsContext::new(window).await?);
        let resource_manager = Arc::new(ResourceManager::new(context.wgpu.clone()));
        let renderer = SceneRenderer::new(context.clone(), resource_manager.clone());
        let logic = T::new_from_graphics_context(context.clone(), resource_manager.clone()).await?;

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

            {
                let mut config = self.context.config.borrow_mut();
                config.width = width;
                config.height = height;
                self.context
                    .surface
                    .configure(self.context.device(), &config);
            }
            *self.context.surface_ready_for_rendering.borrow_mut() = true;
            self.logic.resized(width, height);
        }
    }

    pub fn handle_key(&mut self, event_loop: &ActiveEventLoop, event: &KeyEvent) {
        self.logic.handle_key(event_loop, event);
    }

    pub fn update(&mut self) {
        self.logic.update();
    }

    pub fn handle_mouse_position(&mut self, x: f32, y: f32) {
        self.logic.handle_mouse_position(x, y);
    }

    pub fn handle_mouse_button(&mut self, button: winit::event::MouseButton, state: winit::event::ElementState) {
        self.logic.handle_mouse_button(button, state);
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.context.window.request_redraw();

        if !*self.context.surface_ready_for_rendering.borrow() {
            return Ok(());
        }

        let render_target = self.logic.render_target();

        self.resource_manager.cleanup_unused_text();
        self.renderer.render(&self.context, &render_target)
    }
}
