use crate::graphics::GraphicsContext;
use crate::logic::StateLogic;
use crate::resource_manager::ResourceManager;
use crate::{Renderer, UiEvent};
use std::rc::Rc;
use winit::event::KeyEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

/// The main context for rendering the application, binding together graphics, resources, renderer, and logic.
pub struct RenderingContext<T: StateLogic<R>, R: Renderer> {
    pub context: Rc<GraphicsContext>,
    pub resource_manager: Rc<ResourceManager>,
    pub renderer: R,
    pub logic: T,
}

pub struct EventDispatcher<T: StateLogic<R>, R: Renderer + 'static> {
    proxy: winit::event_loop::EventLoopProxy<UiEvent<RenderingContext<T, R>, T::Event>>,
}

impl<T: StateLogic<R>, R: Renderer + 'static> EventDispatcher<T, R> {
    pub fn send_event(&self, event: T::Event) {
        if let Err(_) = self.proxy.send_event(UiEvent::Event(event)) {
            log::error!("Failed to send event");
        }
    }
}

impl<R: Renderer, T: StateLogic<R>> RenderingContext<T, R> {
    /// Creates a new rendering context for the given window.
    pub async fn new<FNew, FRet>(
        proxy: winit::event_loop::EventLoopProxy<UiEvent<Self, T::Event>>,
        window: Window,
        new_fn: Rc<FNew>,
    ) -> anyhow::Result<Self>
    where
        FNew: Fn(EventDispatcher<T, R>, Rc<GraphicsContext>, Rc<ResourceManager>) -> FRet + 'static,
        FRet: Future<Output = anyhow::Result<T>>,
    {
        let context = Rc::new(GraphicsContext::new(window).await?);
        let resource_manager = Rc::new(ResourceManager::new(
            context.clone(),
            include_bytes!("../src/text/fonts/JetBrainsMono-Regular.ttf"),
        ));

        let renderer = R::new(context.clone(), resource_manager.clone());

        let dispatcher = EventDispatcher { proxy };

        let logic = new_fn(dispatcher, context.clone(), resource_manager.clone()).await?;

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

    pub fn handle_mouse_button(
        &mut self,
        button: winit::event::MouseButton,
        state: winit::event::ElementState,
    ) {
        self.logic.handle_mouse_button(button, state);
    }

    pub fn handle_mouse_wheel(&mut self, delta_x: f32, delta_y: f32) {
        self.logic.handle_mouse_wheel(delta_x, delta_y);
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.context.window.request_redraw();

        if !*self.context.surface_ready_for_rendering.borrow() {
            return Ok(());
        }

        self.logic.update();

        let render_target = self.logic.render_target();
        let (output, view, mut encoder) = R::begin_render_frame(&self.context)?;

        let (width, height) = self.context.size();

        self.renderer.render(
            &self.context.wgpu,
            &render_target,
            &view,
            (width, height),
            &mut encoder,
        )?;

        self.context
            .wgpu
            .queue
            .submit(std::iter::once(encoder.finish()));

        output.present();

        Ok(())
    }

    pub fn handle_event(logic: &mut T, event: T::Event) {
        logic.handle_event(event);
    }
}
