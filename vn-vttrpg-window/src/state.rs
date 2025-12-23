use std::sync::Arc;
use winit::event::KeyEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;
use crate::graphics::GraphicsContext;
use crate::logic::StateLogic;

pub struct RenderingContext<T: StateLogic> {
    pub context: GraphicsContext,
    pub logic: T,
}

impl<T: StateLogic> RenderingContext<T> {
    pub async fn new(window: Arc<Window>, logic: T) -> anyhow::Result<Self> {
        let context = GraphicsContext::new(window).await?;

        Ok(Self { context, logic })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            log::info!("Resizing window to {}x{}", width, height);
            self.context.config.width = width;
            self.context.config.height = height;
            self.context
                .surface
                .configure(&self.context.device, &self.context.config);
            self.context.surface_ready_for_rendering = true;
        }
    }

    pub fn handle_key(&self, event_loop: &ActiveEventLoop, event: &KeyEvent) {
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

        self.logic.render(&self.context)
    }
}
