use std::sync::Arc;
use winit::dpi::LogicalSize;
use winit::event::KeyEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;
use crate::graphics::GraphicsContext;
use crate::input::InputState;
use crate::logic::StateLogic;
use crate::renderer::{Renderer, WgpuRenderer};
use crate::resource_manager::ResourceManager;

pub struct RenderingContext<T: StateLogic, R: Renderer = WgpuRenderer> {
    pub context: GraphicsContext,
    pub resource_manager: ResourceManager,
    pub renderer: R,
    pub input: InputState,
    pub logic: T,
}

impl<T: StateLogic> RenderingContext<T, WgpuRenderer> {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let context = GraphicsContext::new(window).await?;
        let mut resource_manager = ResourceManager::new(context.wgpu.clone());
        let renderer = WgpuRenderer::new();
        let input = InputState::new();
        let logic = T::new_from_graphics_context(&context, &mut resource_manager).await?;

        Ok(Self {
            context,
            resource_manager,
            renderer,
            input,
            logic,
        })
    }
}

impl<T: StateLogic, R: Renderer> RenderingContext<T, R> {
    /// !!! EXPECTS LOGICAL NOT PHYSICAL PIXELS !!!
    pub fn resize(&mut self, mut width: u32, mut height: u32) {
        if width > 0 && height > 0 {
            log::info!("Resizing window to {}x{}", width, height);

            #[cfg(not(target_arch = "wasm32"))]
            {
                let scale_factor = self.context.window.scale_factor();

                width = (width as f64 * scale_factor).round() as u32;
                height = (height as f64 * scale_factor).round() as u32;
            }

            self.context.config.width = width;
            self.context.config.height = height;
            self.context
                .surface
                .configure(self.context.device(), &self.context.config);
            self.context.surface_ready_for_rendering = true;
        }
    }

    pub fn logical_window_size(&self) -> (u32, u32) {
        let size: LogicalSize<u32> = self
            .context
            .window
            .inner_size()
            .to_logical(self.context.window.scale_factor());

        (size.width.max(1), size.height.max(1))
    }

    pub fn handle_key(&mut self, event_loop: &ActiveEventLoop, event: &KeyEvent) {
        self.input.handle_key(event);
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

        self.renderer.render(&self.context, |render_pass| {
            self.logic.draw(render_pass);
        })
    }
}
