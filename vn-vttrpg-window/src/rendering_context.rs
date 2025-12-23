use std::sync::Arc;
use winit::dpi::LogicalSize;
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
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let context = GraphicsContext::new(window).await?;
        let logic = T::new_from_graphics_context(&context).await?;
        
        Ok(Self { context, logic })
    }

    /// !!! EXPECTS LOGICAL NOT PHYSICAL PIXELS !!!
    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            log::info!("Resizing window to {}x{}", width, height);

            let scale_factor = self.context.window.scale_factor();

            // Remark (platform differences): This is a bit of a hack. The web canvas appears to
            //  expect logical pixel sizes while the native one wants physical ones.
            //  I may refactor this later to be prettier... Maybe there is something I can do on
            //  the web side i.e. in the CSS / HTML? Or did I fuck up some setting somewhere?
            //  LLMs are in their typical way of being wrong completely useless for figuring this
            //  out either. (Thx LLM for suggesting that you are "useless" yourself with just the
            //  input of "LLMs are ")
            #[cfg(not(target_arch = "wasm32"))]
            let width = (width as f64 * scale_factor).round() as u32;
            #[cfg(not(target_arch = "wasm32"))]
            let height = (height as f64 * scale_factor).round() as u32;

            self.context.config.width = width;
            self.context.config.height = height;
            self.context
                .surface
                .configure(&self.context.device, &self.context.config);
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
